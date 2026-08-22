//! `arca backup <nome>` — o pre-voo do §5.2, terminando antes de armar.
//!
//! Desarma (C-1), enumera imagens e discos, julga B-3, B-4, C-6 e C-10, lê a
//! Inicializacao Rapida (B-5) e roda o `chkdsk` (B-6). **Termina antes da
//! confirmacao digitada**: confirmar e armar sao a etapa E7, e um comando que
//! armasse aqui pularia o que o plano poe entre os dois.
//!
//! Com `--dry-run`, imprime tambem as duas receitas inteiras — e nao desarma.
//!
//! # Por que o ensaio imprime tambem a receita de restauracao
//!
//! A E3 cobre R-4 e R-5, e a restauracao so ganha comando na E9. Sem
//! aparecer aqui, a unica receita destrutiva do sistema ficaria seis etapas
//! sem ninguem poder olhar para ela. Ela sai marcada como previa: e o que a
//! E9 armaria, e nao o que este comando faria.

use crate::app::Contexto;
use crate::blkdev;
use crate::desarme;
use crate::dispositivo::{self, Dispositivo};
use crate::erro::{Erro, Resultado};
use crate::formato::{gigabytes, linha};
use crate::imagens::{self, Pasta};
use crate::nome::Nome;
use crate::portas::{Arquivos, DiscoFisico};
use crate::prevoo::{self, Chkdsk, DiscoDeOrigem, InicializacaoRapida, PreVoo};
use crate::receita::{Disco, Operacao, Pedido, Receita, Selo};
use std::path::Path;

/// O disco de origem quando a descoberta nao tem de onde tirar o nome.
///
/// Ate a etapa E6 isto era uma constante usada **sempre**, com o comentario
/// dizendo que a E6 a substituiria. Ela sobrevive so no ensaio, e so quando
/// nao ha `blkdev.list` de onde lê — e a saida diz, com todas as letras, que
/// o nome nao foi determinado. Ver [`crate::blkdev`] para por que o ARCA nao
/// deriva esse nome do indice do Windows.
const DISCO_DE_EXEMPLO: &str = "nvme0n1";

/// O que o ensaio tem para mostrar, antes de virar texto.
pub struct Ensaio<'a> {
    pub dispositivo: &'a Dispositivo,
    pub nome: &'a Nome,
    pub disco: &'a Disco,

    /// Se este disco e so um exemplo, por nao haver `blkdev.list` de onde lê o
    /// nome de verdade. A E6 acrescentou o campo: ate ela o disco era
    /// **sempre** suposto e a distincao nao existia no codigo.
    pub de_exemplo: bool,

    pub backup: &'a Receita,
    pub restauracao: &'a Receita,
}

pub fn executar(contexto: &Contexto, nome_bruto: &str) -> Resultado<()> {
    // B-2 primeiro, e antes de tocar no dispositivo: um nome recusado nao
    // precisa de SSD conectado para ser recusado.
    let nome = Nome::novo(nome_bruto).map_err(Erro::NomeRecusado)?;

    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let caminho_do_grub = dispositivo.caminho_do_grub()?;

    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;

    // A enumeracao de discos custa uma consulta ao WMI, e e ela que traz tres
    // coisas de uma vez: o disco de origem para B-4, o `MediaType` de C-6, e a
    // prova de que os dois rotulos estao no mesmo dispositivo fisico (C-10).
    let discos = contexto.discos.discos_fisicos()?;
    let origem = disco_de_origem(&discos, &dispositivo)?;
    let espaco = prevoo::estimar(&pastas, origem, dispositivo.vault.livre_bytes);

    // C-1: desarmar acontece **incondicionalmente**, como primeiro passo, sem
    // consultar estado nenhum. Nao e um passo do pre-voo que se possa deixar
    // para a E7: a primeira linha do §5.2 diz "Desarmando receita anterior",
    // e ela tem de ser verdade. Num dispositivo ja inerte isto nao escreve
    // nada — a E4 mediu que o `grub.cfg` que sai igual ao que entrou nao e
    // regravado.
    let desarme = if contexto.dry_run {
        None
    } else {
        Some(desarme::executar(
            contexto.arquivos,
            contexto.firmware,
            &caminho_do_grub,
        )?)
    };

    // # Por que o cabecalho e impresso **antes** de julgar
    //
    // Esta etapa ja errou nas duas direcoes, e a segunda foi a revisao que
    // pegou. Primeiro a linha do desarmar dizia "ok" sem ter desarmado.
    // Corrigido isso, o desarmar passou a acontecer antes das recusas do
    // pre-voo — e, com a recusa subindo como erro, **nada era impresso**:
    // quem rodasse `arca backup <nome-que-ja-existe>` num dispositivo armado
    // veria so "ja ha uma imagem chamada ...", e o job armado teria sumido em
    // silencio. A acao acontecia e a saida nao contava.
    //
    // Mover o desarmar para depois do julgamento seria fura-lo: C-1 diz
    // incondicionalmente. A saida e imprimir o que ja aconteceu antes de a
    // recusa poder cortar o resto.
    print!(
        "{}",
        prevoo::montar_cabecalho(&prevoo::Cabecalho {
            dispositivo: &dispositivo,
            nome: &nome,
            origem,
            espaco,
            desarme: desarme.as_ref(),
            caminho_do_grub: &caminho_do_grub.to_string_lossy(),
        })
    );

    prevoo::julgar(&nome, &pastas, &espaco, &dispositivo, &discos).map_err(Erro::PreVooRecusou)?;

    let disco = descobrir_o_disco(contexto.arquivos, &raiz_do_vault, &pastas, origem);

    // B-5 e B-6, nesta ordem: a leitura do registro e instantanea, e o
    // `chkdsk /scan` leva dezesseis segundos nesta maquina. Quem for recusado
    // por qualquer coisa acima nao espera por ele.
    let inicializacao_rapida =
        InicializacaoRapida::do_registro(contexto.sistema.inicializacao_rapida()?);

    // Confere o volume do **sistema**, e nao o do dispositivo: e o `C:` que
    // vai ser lido pelo Clonezilla, e um sistema de arquivos sujo e o que faz
    // uma imagem sair com estado inconsistente dentro.
    let chkdsk = Chkdsk::da_saida(&contexto.sistema.conferir_volume(letra_do_sistema())?);

    contexto.registro.info(format!(
        "pre-voo de `{nome}` · origem {} · disco {} · espaco {:?} · inicializacao rapida {inicializacao_rapida:?} · chkdsk {}",
        origem.modelo,
        match &disco {
            DiscoDeOrigem::Descoberto(achado) => achado.disco.to_string(),
            DiscoDeOrigem::PorDeterminar(_) => "por determinar".to_string(),
        },
        espaco.veredito,
        match &chkdsk {
            Chkdsk::Limpo => "limpo".to_string(),
            Chkdsk::Acusou { codigo, .. } => format!("codigo {codigo}"),
        }
    ));

    print!(
        "{}",
        prevoo::montar_o_resto(&PreVoo {
            disco: &disco,
            inicializacao_rapida,
            chkdsk,
        })
    );

    if contexto.dry_run {
        print!("{}", ensaio_das_receitas(contexto, &dispositivo, &nome, &disco)?);
    }

    Ok(())
}

/// O disco onde o Windows mora — o que a receita vai clonar.
///
/// Achado pela letra do volume do sistema, e nao pelo indice: **o disco 0 nao
/// e necessariamente o do Windows**, e supor que e daria a origem errada numa
/// maquina com dois discos.
fn disco_de_origem<'a>(
    discos: &'a [DiscoFisico],
    dispositivo: &Dispositivo,
) -> Resultado<&'a DiscoFisico> {
    let do_dispositivo: Vec<u32> = dispositivo
        .vault
        .letra
        .into_iter()
        .chain(dispositivo.boot.as_ref().and_then(|boot| boot.letra))
        .filter_map(|letra| {
            discos
                .iter()
                .find(|disco| disco.tem_a_letra(letra))
                .map(|disco| disco.indice)
        })
        .collect();

    let sistema = letra_do_sistema();
    discos
        .iter()
        .find(|disco| disco.tem_a_letra(sistema) && !do_dispositivo.contains(&disco.indice))
        .ok_or(Erro::OrigemDesconhecida)
}

/// Onde o Windows mora, perguntado ao Windows.
///
/// # Por que nao `'C'` fixo
///
/// A primeira versao desta etapa tinha a letra como constante, e uma funcao
/// que recebia dois parametros e os ignorava para devolve-la. Era o mesmo erro
/// que esta etapa combate em dois outros lugares — nao supor que a origem e o
/// disco 0, nao derivar o nome Linux do indice do Windows —, cometido no
/// terceiro. Numa instalacao em outra letra, o `chkdsk` de B-6 conferiria o
/// volume errado e a origem sairia como desconhecida.
///
/// `%SystemDrive%` e uma variavel de ambiente **do sistema**, e nao do console
/// de quem chamou: ela atravessa a elevacao por UAC, ao contrario do ambiente
/// que a §C-7 discute. Sem ela, `'C'` e a suposicao menos ruim que sobra — e
/// ela aparece como origem desconhecida em vez de silenciosamente errada.
fn letra_do_sistema() -> char {
    std::env::var_os("SystemDrive")
        .and_then(|valor| valor.to_string_lossy().chars().next())
        .filter(|letra| letra.is_ascii_alphabetic())
        .map(|letra| letra.to_ascii_uppercase())
        .unwrap_or('C')
}

/// O nome que o Linux da ao disco de origem, lido do `blkdev.list` das imagens.
///
/// Uma leitura que falhe nao derruba o pre-voo: a imagem pode estar num setor
/// ruim, e o resultado disso e o nome ficar por determinar — que ja e um
/// desfecho previsto e dito na tela.
fn descobrir_o_disco(
    arquivos: &dyn Arquivos,
    raiz_do_vault: &Path,
    pastas: &[Pasta],
    origem: &DiscoFisico,
) -> DiscoDeOrigem {
    let listas: Vec<(String, String)> = pastas
        .iter()
        .filter(|pasta| pasta.e_imagem())
        .filter_map(|pasta| {
            let caminho = raiz_do_vault.join(&pasta.nome).join("blkdev.list");
            arquivos
                .ler_texto_alheio(&caminho)
                .ok()
                .map(|texto| (pasta.nome.clone(), texto))
        })
        .collect();

    match blkdev::nome_do_disco(&origem.modelo, &listas) {
        Ok(achado) => DiscoDeOrigem::Descoberto(achado),
        Err(porque) => DiscoDeOrigem::PorDeterminar(porque),
    }
}

/// As duas receitas inteiras, so no `--dry-run`.
fn ensaio_das_receitas(
    contexto: &Contexto,
    dispositivo: &Dispositivo,
    nome: &Nome,
    disco: &DiscoDeOrigem,
) -> Resultado<String> {
    // Sem nome de disco descoberto, o ensaio imprime a receita com um disco de
    // **exemplo** e diz isso. Recusar seria pior: quem quer conferir a forma da
    // receita antes do primeiro backup nao tem imagem de onde tirar o nome.
    let (o_disco, de_exemplo) = match disco {
        DiscoDeOrigem::Descoberto(achado) => (achado.disco.clone(), false),
        DiscoDeOrigem::PorDeterminar(_) => (
            Disco::novo(DISCO_DE_EXEMPLO).map_err(Erro::ReceitaRecusada)?,
            true,
        ),
    };

    // O selo de verdade nasce ao armar, na E7. Este e de ensaio, e a saida o
    // diz — ver [`Selo::de_ensaio`].
    let selo = Selo::de_ensaio();

    let montar_para = |operacao| {
        Receita::montar(&Pedido {
            operacao,
            nome: nome.clone(),
            disco: o_disco.clone(),
            selo: selo.clone(),
        })
        .map_err(Erro::ReceitaRecusada)
    };

    let backup = montar_para(Operacao::Backup)?;
    let restauracao = montar_para(Operacao::Restauracao)?;

    contexto.registro.info(format!(
        "ensaio de backup `{nome}` · disco {o_disco}{} · receita de {} caracteres · validada por C-2",
        if de_exemplo { " (de exemplo)" } else { "" },
        backup.comando().chars().count()
    ));

    Ok(montar(&Ensaio {
        dispositivo,
        nome,
        disco: &o_disco,
        de_exemplo,
        backup: &backup,
        restauracao: &restauracao,
    }))
}

/// O ensaio inteiro, em texto.
pub fn montar(ensaio: &Ensaio) -> String {
    let mut saida = String::new();

    saida.push_str("\nEnsaio (--dry-run): nada e gravado, nada e armado.\n\n");

    saida.push_str(&format!(
        "Dispositivo ARCA: {} ({}) · {} livres\n",
        dispositivo::ARCAVAULT,
        match ensaio.dispositivo.vault.letra {
            Some(letra) => format!("{letra}:"),
            None => "sem letra".to_string(),
        },
        gigabytes(ensaio.dispositivo.vault.livre_bytes)
    ));
    saida.push_str(&format!("Imagem: {}\n", ensaio.nome));
    saida.push_str(&format!(
        "Disco de origem: {}{}\n\n",
        ensaio.disco,
        if ensaio.de_exemplo {
            " · DE EXEMPLO: o nome de verdade nao foi determinado, e esta receita nao serviria"
        } else {
            " · lido do blkdev.list de uma imagem"
        }
    ));

    saida.push_str(&linha("Nome validado (B-2)", "ok"));
    saida.push_str(&linha("Receita validada (C-2)", "ok"));
    saida.push('\n');

    saida.push_str(&secao(
        "Receita de backup — e esta que a etapa E7 armaria",
        ensaio.backup,
    ));
    saida.push('\n');
    saida.push_str(&secao(
        "Receita de restauracao — previa; quem a arma e a etapa E9",
        ensaio.restauracao,
    ));

    saida.push_str(concat!(
        "\nO selo acima e de ensaio (so zeros). O de verdade nasce **ao armar**, e\n",
        "quem arma e a etapa E7 — a E5 escreveu o gerador, e nao o momento em que\n",
        "ele e chamado. E o selo que liga o job ao desfecho.\n",
        "\nNenhuma receita foi armada. Armar e a etapa E7.\n"
    ));

    saida
}

/// Uma receita: o que o Clonezilla executa, e como ela entra no `grub.cfg`.
///
/// As duas formas, e nao so uma. A primeira e o que se lê para conferir se a
/// operacao esta certa; a segunda e o que de fato vai para o disco, com o
/// aninhamento de aspas que C-2 existe para proteger. Conferir a receita numa
/// forma e gravar a outra foi o que deixou o `--dry-run` mentir uma vez.
fn secao(titulo: &str, receita: &Receita) -> String {
    format!(
        "{titulo}\n\n  O que o Clonezilla executa:\n\n    {}\n\n  Como entra na linha do grub.cfg:\n\n    {}\n",
        receita.comando(),
        receita.parametros_do_grub()
    )
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{
        ArquivosEmMemoria, DiscosDeMentira, FirmwareDeMentira, RelogioParado, SistemaDeMentira,
    };
    use crate::portas::Volume;
    use crate::registro::Registro;

    fn dispositivo_conectado() -> Dispositivo {
        dispositivo::encontrar(&DiscosDeMentira::com_dispositivo()).unwrap()
    }

    fn receita(operacao: Operacao) -> Receita {
        Receita::montar(&Pedido {
            operacao,
            nome: Nome::novo("2026-08-22_Apps").unwrap(),
            disco: Disco::novo(DISCO_DE_EXEMPLO).unwrap(),
            selo: Selo::de_ensaio(),
        })
        .unwrap()
    }

    fn ensaio_montado_com(de_exemplo: bool) -> String {
        let dispositivo = dispositivo_conectado();
        let nome = Nome::novo("2026-08-22_Apps").unwrap();
        let disco = Disco::novo(DISCO_DE_EXEMPLO).unwrap();
        let backup = receita(Operacao::Backup);
        let restauracao = receita(Operacao::Restauracao);

        montar(&Ensaio {
            dispositivo: &dispositivo,
            nome: &nome,
            disco: &disco,
            de_exemplo,
            backup: &backup,
            restauracao: &restauracao,
        })
    }

    fn ensaio_montado() -> String {
        ensaio_montado_com(false)
    }

    #[test]
    fn o_ensaio_nomeia_a_imagem_e_o_dispositivo() {
        // Quem lê precisa saber sobre o que a receita fala antes de ler a
        // receita — nao depois, achando o nome no meio de uma linha de 700
        // caracteres.
        let saida = ensaio_montado();
        assert!(saida.contains("Imagem: 2026-08-22_Apps"), "{saida}");
        assert!(saida.contains("ARCAVAULT (E:)"), "{saida}");
    }

    #[test]
    fn o_ensaio_imprime_as_duas_receitas_inteiras() {
        // O criterio de aceite da etapa. "Inteiras" quer dizer que o que sai
        // impresso e a string que seria gravada, e nao um resumo dela.
        let saida = ensaio_montado();

        for operacao in [Operacao::Backup, Operacao::Restauracao] {
            let esperada = receita(operacao);
            assert!(
                saida.contains(esperada.comando()),
                "faltou a receita de {} inteira:\n{saida}",
                operacao.nome()
            );
            assert!(
                saida.contains(&esperada.parametros_do_grub()),
                "faltou a linha do grub.cfg da {}:\n{saida}",
                operacao.nome()
            );
        }
    }

    #[test]
    fn o_ensaio_diz_que_e_ensaio_e_que_nada_foi_armado() {
        let saida = ensaio_montado();
        assert!(saida.contains("--dry-run"), "{saida}");
        assert!(saida.contains("Nenhuma receita foi armada"), "{saida}");
        assert!(saida.contains("Armar e a etapa E7"), "{saida}");

        // E o selo diz que quem o cria e a E7, e nao a E5: a E5 escreveu o
        // gerador, e nao o momento em que ele e chamado.
        assert!(
            saida.contains("quem arma e a etapa E7"),
            "o rodape ainda atribui o selo a etapa errada:\n{saida}"
        );
    }

    #[test]
    fn o_ensaio_diz_de_onde_o_nome_do_disco_veio_nos_dois_casos() {
        // O padrao da E3: uma receita destrutiva que nomeasse um disco sem
        // dizer de onde ele veio e pior do que nao imprimir nada. A E6 tornou
        // a distincao real — antes o disco era **sempre** suposto —, e por
        // isso os dois lados precisam de teste.
        let descoberto = ensaio_montado_com(false);
        assert!(descoberto.contains("lido do blkdev.list"), "{descoberto}");
        assert!(!descoberto.contains("DE EXEMPLO"), "{descoberto}");

        let de_exemplo = ensaio_montado_com(true);
        assert!(de_exemplo.contains("DE EXEMPLO"), "{de_exemplo}");
        assert!(
            de_exemplo.contains("nao serviria"),
            "faltou dizer que esta receita nao vale:\n{de_exemplo}"
        );
    }

    #[test]
    fn o_ensaio_avisa_que_o_selo_nao_e_de_verdade() {
        let saida = ensaio_montado();
        assert!(saida.contains("de ensaio"), "{saida}");
        assert!(saida.contains("ARCA_SELO=0000000000000000"), "{saida}");
    }

    #[test]
    fn o_ensaio_diz_qual_etapa_arma_cada_receita() {
        // A de restauracao aparece aqui porque a E3 a cobre e a E9 e quem a
        // arma. Sem essa marca, ela leria como algo que este comando faria.
        let saida = ensaio_montado();
        assert!(saida.contains("etapa E7 armaria"), "{saida}");
        assert!(saida.contains("quem a arma e a etapa E9"), "{saida}");
    }

    // ───────────────────────── o comando inteiro ─────────────────────────

    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        sistema: SistemaDeMentira,
        registro: Registro,
    }

    impl Bancada {
        fn nova(discos: DiscosDeMentira) -> Bancada {
            Bancada::com(discos, ArquivosEmMemoria::novo())
        }

        fn com_firmware(mut self, firmware: FirmwareDeMentira) -> Bancada {
            self.firmware = firmware;
            self
        }

        fn com(discos: DiscosDeMentira, arquivos: ArquivosEmMemoria) -> Bancada {
            Bancada {
                arquivos,
                discos,
                firmware: FirmwareDeMentira::novo(),
                relogio: RelogioParado::em("2026-08-22T11:42:03"),
                sistema: SistemaDeMentira::novo(),
                registro: Registro::em(
                    std::env::temp_dir().join(format!(
                        "arca-backup-{}-{:?}",
                        std::process::id(),
                        std::thread::current().id()
                    )),
                    Box::new(RelogioDoSistema),
                ),
            }
        }

        fn contexto(&self, dry_run: bool) -> Contexto<'_> {
            Contexto {
                dry_run,
                registro: &self.registro,
                firmware: &self.firmware,
                discos: &self.discos,
                arquivos: &self.arquivos,
                relogio: &self.relogio,
                sistema: &self.sistema,
            }
        }
    }

    impl Drop for Bancada {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.registro.caminho().parent().unwrap());
        }
    }

    /// O `ARCAVAULT` desta mesa, como o pre-voo o encontra.
    fn vault_com_as_imagens() -> ArquivosEmMemoria {
        ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc")
            .com(r"E:\2026-08-21_WindowsCompleto\blkdev.list", BLKDEV)
            .com(r"E:\ARCA-TESTE-03\MD5SUMS", "abc")
    }

    /// O `blkdev.list` do dispositivo, com as colunas reais do `lsblk`.
    const BLKDEV: &str = concat!(
"KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
"sda       sda         238.5G disk                                               KGSSE100256\n",
"nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
    );

    /// Um `{fwbootmgr}` sem boot unico, como o `bcdedit` o enumera.
    const FWBOOTMGR_INERTE: &str = concat!(
        "\r\nGerenciador de Inicializacao de Firmware\r\n",
        "----------------------------------------\r\n",
        "identificador           {fwbootmgr}\r\n",
        "displayorder            {bootmgr}\r\n",
        "timeout                 1\r\n"
    );

    const GRUB_INERTE: &str = include_str!("../../recursos/capturas/grub-inerte-arcaboot.cfg");

    fn bancada_completa() -> Bancada {
        Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", GRUB_INERTE),
        )
        .com_firmware(FirmwareDeMentira::novo().respondendo("{fwbootmgr}", FWBOOTMGR_INERTE))
    }

    #[test]
    fn sem_dry_run_o_backup_roda_o_pre_voo_e_para_antes_de_armar() {
        // Ate a E5 este comando respondia "armar e a E7" e nao fazia mais
        // nada. A E6 o poe a trabalhar: ele roda o dialogo do §5.2 inteiro e
        // **termina antes da confirmacao**. Armar continua sendo a E7.
        let bancada = bancada_completa();

        executar(&bancada.contexto(false), "2026-08-22_Apps").expect("o pre-voo roda");

        // Nao armou: o estado do job nao existe, e o `grub.cfg` saiu como
        // entrou — o desarmar de C-1 num arquivo ja inerte nao regrava nada
        // (medido na E4).
        assert!(
            bancada.arquivos.conteudo_de(r"R:\arca\estado.json").is_none(),
            "o pre-voo gravou estado de job"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(r"R:\boot\grub\grub.cfg").as_deref(),
            Some(GRUB_INERTE),
            "o pre-voo mexeu no grub.cfg"
        );
    }

    #[test]
    fn o_pre_voo_desarma_de_verdade_como_c1_manda() {
        // C-1 nao e condicional a chegar ao armar: desarmar e o primeiro passo
        // de todo comando. Um dispositivo armado com receita velha nao pode
        // sair daqui com "pre-voo concluido" e continuar armado.
        let armado = include_str!("../../recursos/capturas/grub-backup-arca-teste-03.cfg");
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", armado),
        )
        .com_firmware(FirmwareDeMentira::novo().respondendo("{fwbootmgr}", FWBOOTMGR_INERTE));

        executar(&bancada.contexto(false), "2026-08-22_Apps").expect("roda");

        assert_eq!(
            bancada.arquivos.conteudo_de(r"R:\boot\grub\grub.cfg").as_deref(),
            Some(GRUB_INERTE),
            "o pre-voo disse que desarmou e nao desarmou"
        );
    }

    #[test]
    fn a_recusa_do_pre_voo_nao_esconde_que_o_desarmar_aconteceu() {
        // Achado pela revisao da E6, e e o espelho do defeito que a execucao
        // real tinha pegado. Corrigida a linha que dizia "ok" sem desarmar, o
        // desarmar passou a acontecer **antes** das recusas do pre-voo — e,
        // com a recusa subindo como erro, nada era impresso. Quem rodasse
        // `arca backup <nome-que-ja-existe>` num dispositivo armado veria so
        // "ja ha uma imagem chamada ...", e o job armado teria sumido em
        // silencio. A acao acontecia e a saida nao contava.
        let armado = include_str!("../../recursos/capturas/grub-backup-arca-teste-03.cfg");
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", armado),
        )
        .com_firmware(FirmwareDeMentira::novo().respondendo("{fwbootmgr}", FWBOOTMGR_INERTE));

        // Um nome que B-3 recusa: a imagem ja existe.
        let erro = executar(&bancada.contexto(false), "2026-08-21_WindowsCompleto").unwrap_err();
        assert!(matches!(erro, Erro::PreVooRecusou(_)), "veio {erro}");

        // O desarmar aconteceu — C-1 e incondicional...
        assert_eq!(
            bancada.arquivos.conteudo_de(r"R:\boot\grub\grub.cfg").as_deref(),
            Some(GRUB_INERTE),
            "a recusa pulou o desarmar, e C-1 diz incondicionalmente"
        );

        // ...e o cabecalho ja tinha sido montado quando a recusa subiu. Isso
        // se prova pela ordem: o cabecalho nao depende do julgamento, e a
        // funcao que o monta e chamada antes dele.
        let cabecalho = crate::prevoo::montar_cabecalho(&crate::prevoo::Cabecalho {
            dispositivo: &dispositivo_conectado(),
            nome: &Nome::novo("2026-08-21_WindowsCompleto").unwrap(),
            origem: &crate::duplos::discos_desta_mesa()[0],
            espaco: crate::espaco::avaliar(0, 1000, 1_000_000),
            desarme: Some(&crate::desarme::Desarme {
                caminho_do_grub: std::path::PathBuf::from(r"R:\boot\grub\grub.cfg"),
                blocos_removidos: 1,
                default_devolvido: true,
                grub_regravado: true,
                boot_unico: crate::desarme::MarcaDeBootUnico::NaoHavia,
            }),
            caminho_do_grub: r"R:\boot\grub\grub.cfg",
        });

        assert!(
            cabecalho.contains("havia receita armada"),
            "o cabecalho nao conta que desarmou:\n{cabecalho}"
        );
    }

    #[test]
    fn o_ensaio_nao_desarma_nem_diz_que_desarmou() {
        // `--dry-run` e flag de primeira classe: no ensaio nada acontece, e a
        // saida nao pode dizer que aconteceu.
        let armado = include_str!("../../recursos/capturas/grub-backup-arca-teste-03.cfg");
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", armado),
        )
        .com_firmware(FirmwareDeMentira::novo().respondendo("{fwbootmgr}", FWBOOTMGR_INERTE));

        executar(&bancada.contexto(true), "2026-08-22_Apps").expect("o ensaio roda");

        assert_eq!(
            bancada.arquivos.conteudo_de(r"R:\boot\grub\grub.cfg").as_deref(),
            Some(armado),
            "o ensaio desarmou o dispositivo"
        );
        assert!(
            bancada.firmware.executados.borrow().is_empty(),
            "o ensaio mandou o bcdedit fazer alguma coisa"
        );
    }

    #[test]
    fn o_pre_voo_confere_o_volume_do_sistema_e_nao_o_do_dispositivo() {
        // E o `C:` que o Clonezilla vai lê. Conferir o `E:` daria um `ok`
        // sobre o disco errado — e um sistema de arquivos sujo no `C:` e o que
        // faz a imagem sair com estado inconsistente dentro.
        let bancada = bancada_completa();

        executar(&bancada.contexto(false), "2026-08-22_Apps").expect("roda");
        assert_eq!(*bancada.sistema.conferidos.borrow(), vec!['C']);
    }

    #[test]
    fn o_ensaio_nao_escreve_nada_em_lugar_nenhum() {
        // "Nao toca em nada" e criterio de aceite, e nao promessa.
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens(),
        );
        executar(&bancada.contexto(true), "2026-08-22_Apps").expect("o ensaio roda");

        for caminho in [
            r"R:\boot\grub\grub.cfg",
            r"R:\arca\estado.json",
            r"E:\2026-08-22_Apps",
        ] {
            assert!(
                bancada.arquivos.conteudo_de(caminho).is_none(),
                "o ensaio escreveu em {caminho}"
            );
        }

        assert!(
            bancada.firmware.executados.borrow().is_empty(),
            "o ensaio mandou o bcdedit fazer alguma coisa"
        );
    }

    #[test]
    fn o_disco_de_origem_e_descoberto_do_blkdev_list_da_imagem() {
        // O caminho inteiro, pelo comando: o WMI diz o modelo do disco onde o
        // `C:` mora, e o `blkdev.list` de uma imagem diz que nome o Linux lhe
        // da. Nenhuma das duas pontas e chutada.
        let discos = crate::duplos::discos_desta_mesa();
        let dispositivo = dispositivo_conectado();
        let arquivos = vault_com_as_imagens();
        let pastas = imagens::enumerar(&arquivos, std::path::Path::new(r"E:\")).unwrap();

        let origem = disco_de_origem(&discos, &dispositivo).expect("acha a origem");
        assert_eq!(origem.modelo, "KINGSTON SNV3S500G");

        match descobrir_o_disco(&arquivos, std::path::Path::new(r"E:\"), &pastas, origem) {
            DiscoDeOrigem::Descoberto(achado) => {
                assert_eq!(achado.disco.como_texto(), "nvme0n1");
            }
            outro => panic!("esperava o disco descoberto, veio {outro:?}"),
        }
    }

    #[test]
    fn sem_blkdev_list_o_disco_fica_por_determinar_e_nao_e_chutado() {
        // O oraculo so existe depois do primeiro backup. Chutar `nvme0n1`
        // porque e o nome mais comum seria inventar uma derivacao e documenta-la
        // como descoberta — o padrao que este projeto ja pagou tres vezes.
        let discos = crate::duplos::discos_desta_mesa();
        let dispositivo = dispositivo_conectado();
        let vazio = ArquivosEmMemoria::novo().com_pasta_vazia(r"E:\");
        let origem = disco_de_origem(&discos, &dispositivo).unwrap();

        assert!(matches!(
            descobrir_o_disco(&vazio, std::path::Path::new(r"E:\"), &[], origem),
            DiscoDeOrigem::PorDeterminar(_)
        ));
    }

    #[test]
    fn a_origem_nao_e_o_disco_zero_por_suposicao() {
        // Numa maquina em que o dispositivo ARCA fosse o disco 0 e o Windows o
        // disco 1, supor "a origem e o indice 0" faria a receita clonar o
        // proprio dispositivo de backup.
        use crate::portas::{DiscoFisico, TipoDeMidia};

        let invertidos = vec![
            DiscoFisico {
                indice: 0,
                modelo: "O DISPOSITIVO".to_string(),
                tamanho_bytes: 256_052_966_400,
                em_uso_bytes: 1000,
                tipo_de_midia: TipoDeMidia::DiscoExterno,
                letras: vec!['E', 'R'],
            },
            DiscoFisico {
                indice: 1,
                modelo: "O WINDOWS".to_string(),
                tamanho_bytes: 500_105_249_280,
                em_uso_bytes: 112_973_562_368,
                tipo_de_midia: TipoDeMidia::DiscoFixo,
                letras: vec!['C'],
            },
        ];

        let origem = disco_de_origem(&invertidos, &dispositivo_conectado()).unwrap();
        assert_eq!(origem.modelo, "O WINDOWS");
    }

    #[test]
    fn sem_disco_de_origem_o_comando_recusa_em_vez_de_escolher_um() {
        let so_o_dispositivo = vec![crate::duplos::discos_desta_mesa()[1].clone()];
        assert!(matches!(
            disco_de_origem(&so_o_dispositivo, &dispositivo_conectado()),
            Err(Erro::OrigemDesconhecida)
        ));
    }

    #[test]
    fn o_nome_e_recusado_antes_de_o_dispositivo_ser_procurado() {
        // B-2 nao precisa de SSD conectado. Recusar o nome primeiro poupa
        // quem digitou errado de ouvir "conecte o dispositivo".
        let bancada = Bancada::nova(DiscosDeMentira::default());
        let erro = executar(&bancada.contexto(true), "meu backup").unwrap_err();

        assert!(
            matches!(erro, Erro::NomeRecusado(_)),
            "esperava a recusa do nome, veio {erro}"
        );
    }

    #[test]
    fn o_nome_e_recusado_tambem_sem_dry_run() {
        // Senao um nome invalido sairia como "chega na etapa E7", e quem
        // digitou nunca saberia que o nome era o problema.
        let bancada = Bancada::nova(DiscosDeMentira::com_dispositivo());
        let erro = executar(&bancada.contexto(false), "backup;poweroff").unwrap_err();

        assert!(
            matches!(erro, Erro::NomeRecusado(_)),
            "esperava a recusa do nome, veio {erro}"
        );
    }

    #[test]
    fn sem_dispositivo_o_ensaio_recusa_em_vez_de_inventar_um() {
        let bancada = Bancada::nova(DiscosDeMentira::com_volumes(vec![Volume {
            rotulo: Some("Windows".to_string()),
            ..crate::duplos::volume("Windows", 'C', 1000, 500)
        }]));

        assert!(matches!(
            executar(&bancada.contexto(true), "2026-08-22_Apps").unwrap_err(),
            Erro::DispositivoAusente
        ));
    }
}
