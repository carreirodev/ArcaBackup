//! `arca backup <nome>` — o pre-voo do §5.2, terminando antes de armar.
//!
//! Desarma (C-1), enumera imagens e discos, julga B-3, B-4, C-6 e C-10, lê a
//! Inicializacao Rapida (B-5) e roda o `chkdsk` (B-6). **Termina antes da
//! confirmacao digitada**: confirmar e armar sao a etapa E7, e um comando que
//! armasse aqui pularia o que o plano poe entre os dois.
//!
//! Com `--dry-run`, imprime tambem a receita inteira — e nao desarma.
//!
//! # A receita de restauracao saiu daqui na E9
//!
//! Da E3 a E8 este ensaio imprimia **duas** receitas, e a segunda vinha
//! marcada como *"previa; quem a arma e a etapa E9"*. A razao era boa: a E3
//! cobre R-4 e R-5, e sem aparecer aqui a unica receita destrutiva do sistema
//! ficaria seis etapas sem ninguem poder olhar para ela.
//!
//! A E9 chegou, e a razao acabou. `arca restore --dry-run` imprime aquela
//! receita com o disco de **destino** escolhido e conferido, e nao com o de
//! origem por coincidencia de desenho — que era o que saia daqui. Manter as
//! duas seria duas fontes da mesma receita, e duas versoes da mesma coisa
//! divergem na primeira mudanca.
//!
//! E ha o motivo de sempre: a frase *"quem a arma e a etapa E9"* passou a ser
//! falsa no instante em que a E9 ficou pronta. E a mesma armadilha que a E7
//! pagou duas vezes — **depois de corrigir, releia o que a correcao encostou.**

use crate::app::Contexto;
use crate::armar;
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
            arma_em_seguida: !contexto.dry_run,
        })
    );

    if contexto.dry_run {
        print!("{}", ensaio_das_receitas(contexto, &dispositivo, &nome, &disco)?);
        return Ok(());
    }

    armar_e_reiniciar(contexto, &dispositivo, &nome, &disco)
}

/// A segunda metade da etapa E7: confirmar, armar, avisar e reiniciar.
///
/// # A ordem, e o que ela impede
///
/// 1. **O disco de origem**, antes da confirmacao. Recusar depois de a pessoa
///    ter digitado o nome inteiro seria fazer o trabalho na ordem errada.
/// 2. **A confirmacao digitada** (S-2), antes de qualquer escrita.
/// 3. **Armar** (C-3, C-4, C-5, C-6, C-11) — o ponto sem volta, com a
///    releitura dentro.
/// 4. **O aviso de C-9**, depois de armado e **antes** de reiniciar.
/// 5. **Reiniciar**, por ultimo.
///
/// O 5 depois do 3 e o que separa este comando de um que dispararia o reinicio
/// sem saber se armou: a releitura de C-3 mora dentro do passo 3, e um erro
/// ali sobe antes de a maquina ir a lugar nenhum.
///
/// O 4 antes do 5 e C-9 na letra. Depois do reinicio nao ha tela.
fn armar_e_reiniciar(
    contexto: &Contexto,
    dispositivo: &Dispositivo,
    nome: &Nome,
    disco: &DiscoDeOrigem,
) -> Resultado<()> {
    // # Por que nao ha "digite o nome do disco"
    //
    // A E6 deixou a pergunta em aberto — pedir ao usuario, ou recusar — e a
    // resposta e recusar. `nvme0n1` e um nome do **Linux**, e quem o digitaria
    // esta no Windows, onde nao ha nada contra o que confer-lo: um `nvme1n1`
    // digitado por engano passaria por bom, iria para a receita, e nomearia o
    // disco errado. O oraculo e o `blkdev.list` de dentro de uma imagem
    // (§4.5), e um valor digitado nao tem oraculo nenhum.
    //
    // O custo e conhecido e limitado: num dispositivo sem imagem alguma, o
    // primeiro backup precisa ser feito uma vez pelo menu do Clonezilla. Dali
    // em diante o `blkdev.list` dele responde para sempre. O custo do outro
    // lado seria uma receita destrutiva (E9) nomeando um disco por suposicao.
    let disco = match disco {
        DiscoDeOrigem::Descoberto(achado) => &achado.disco,
        DiscoDeOrigem::PorDeterminar(porque) => {
            return Err(Erro::DiscoDeOrigemPorDeterminar {
                porque: porque.to_string(),
            });
        }
    };

    // S-2: texto digitado, nunca `s`. A recusa acontece antes de qualquer
    // escrita — o dispositivo continua inerte, porque o desarmar de C-1 ja
    // passou por aqui.
    confirmar(contexto, nome)?;

    let armado = armar::executar(
        contexto.arquivos,
        contexto.firmware,
        contexto.entropia,
        contexto.relogio,
        &armar::Pedir {
            dispositivo,
            operacao: Operacao::Backup,
            nome,
            disco: Some(disco),
        },
    )?;

    contexto.registro.info(format!(
        "armado `{nome}` · selo {} · disco {disco} · entrada {} ({}) · desfecho em {}",
        armado.selo,
        armado.identificador,
        match &armado.entrada {
            armar::Entrada::JaEraDoArca => "ja era do ARCA".to_string(),
            armar::Entrada::MigradaDaLegada { de } => format!("migrada de `{de}`"),
        },
        armado.pasta_do_desfecho
    ));

    print!("{}", montar_o_armado(&armado));

    // C-9, e so entao o reinicio. Um erro do `shutdown` chega aqui com o
    // dispositivo **armado**, e a mensagem tem de dizer isso: a maquina
    // continua no Windows e o proximo reinicio, venha de onde vier, vai para o
    // dispositivo.
    contexto.sistema.reiniciar().inspect_err(|_| {
        eprintln!(
            "\nO dispositivo FICOU ARMADO e a maquina nao reiniciou. O proximo reinicio,\n\
             seja qual for a causa, vai bootar no dispositivo e rodar a receita.\n\
             Para desfazer:  arca desarmar"
        );
    })
}

/// S-2: o nome da imagem por extenso, lido do console.
///
/// O julgamento mora em [`crate::confirmacao`] desde a E9, quando o
/// `arca restore` passou a precisar do mesmo — e a tela dele e outra, mais
/// dura, porque a operacao apaga um disco. O que se compartilha e a regra.
fn confirmar(contexto: &Contexto, nome: &Nome) -> Resultado<()> {
    crate::confirmacao::pedir(contexto, "Digite o nome do backup para confirmar", nome)
}

/// O que se imprime depois de armado, com o aviso de C-9 no fim.
///
/// O aviso e a **ultima** coisa antes do reinicio, e nao a primeira: e o que a
/// pessoa acabou de lê quando a tela apaga. O §5.1 explica por que ele nao e
/// zelo — depois de uma restauracao seguida de `poweroff`, o boot seguinte foi
/// para o dispositivo removivel sem `bootsequence` pendente. Causa nao
/// determinada, nao reproduzido; remover o SSD elimina o cenario.
pub fn montar_o_armado(armado: &armar::Armado) -> String {
    // As cinco linhas moram em [`crate::armar::montar_as_linhas`] desde a E9,
    // quando um segundo comando passou a armar. Elas sao a releitura de C-3
    // impressa, e as duas telas mostram as mesmas: duas copias divergiriam na
    // primeira mudanca, e uma delas passaria a dizer sobre a releitura algo
    // que nao e verdade. O que e proprio deste comando e o que vem depois.
    let mut saida = String::from("\n");
    saida.push_str(&armar::montar_as_linhas(armado));

    // O que se vê do outro lado do reinício é igual nos três comandos que
    // armam, e mora em [`armar::montar_o_que_vem_pela_frente`] desde a E11 —
    // ver lá por que ele existe.
    saida.push_str(armar::montar_o_que_vem_pela_frente());

    saida.push_str(concat!(
        "\nAO TERMINAR: remova o SSD antes de religar.\n",
        "\nReiniciando...\n"
    ));

    saida
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

    let backup = Receita::montar(&Pedido {
        operacao: Operacao::Backup,
        nome: nome.clone(),
        disco: Some(o_disco.clone()),
        selo: selo.clone(),
    })
    .map_err(Erro::ReceitaRecusada)?;

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
        "Receita de backup — e esta que o comando sem --dry-run armaria",
        ensaio.backup,
    ));

    // Estas duas frases falavam da E7 no futuro, e a E7 chegou. Ficaram
    // erradas no instante em que o comando passou a armar — e o modo de falha
    // e o pior que um `--dry-run` tem: dizer alguma coisa sobre o que o
    // comando de verdade faz que nao e mais verdade.
    saida.push_str(concat!(
        "\nO selo acima e de ensaio (so zeros), e por isso esta receita nao serviria: o\n",
        "de verdade nasce **ao armar**, de uma fonte de entropia do sistema. E ele que\n",
        "liga o job ao desfecho que voltar.\n",
        "\nNada foi armado, e o dispositivo nao foi nem desarmado — no ensaio, C-1 nao\n",
        "acontece. O mesmo comando sem `--dry-run` desarma, pede a confirmacao por\n",
        "extenso, arma e reinicia.\n",
        "\nA receita de restauracao nao sai mais daqui: ela e montada com o disco de\n",
        "DESTINO, que este comando nao escolhe. Para ve-la:  arca restore --dry-run\n"
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
    use crate::duplos::{ParticionadorDeMentira, 
        ArquivosEmMemoria, DiscosDeMentira, ConsoleDeMentira, EntropiaDeMentira, FirmwareDeMentira, RelogioParado,
        SistemaDeMentira,
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
            disco: Some(Disco::novo(DISCO_DE_EXEMPLO).unwrap()),
            selo: Selo::de_ensaio(),
        })
        .unwrap()
    }

    fn ensaio_montado_com(de_exemplo: bool) -> String {
        let dispositivo = dispositivo_conectado();
        let nome = Nome::novo("2026-08-22_Apps").unwrap();
        let disco = Disco::novo(DISCO_DE_EXEMPLO).unwrap();
        let backup = receita(Operacao::Backup);

        montar(&Ensaio {
            dispositivo: &dispositivo,
            nome: &nome,
            disco: &disco,
            de_exemplo,
            backup: &backup,
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
    fn o_ensaio_imprime_a_receita_de_backup_inteira() {
        // O criterio de aceite da etapa. "Inteira" quer dizer que o que sai
        // impresso e a string que seria gravada, e nao um resumo dela.
        let saida = ensaio_montado();
        let esperada = receita(Operacao::Backup);

        assert!(
            saida.contains(esperada.comando()),
            "faltou a receita de backup inteira:\n{saida}"
        );
        assert!(
            saida.contains(&esperada.parametros_do_grub()),
            "faltou a linha do grub.cfg:\n{saida}"
        );
    }

    #[test]
    fn o_ensaio_diz_que_e_ensaio_e_que_nada_foi_armado() {
        let saida = ensaio_montado();
        assert!(saida.contains("--dry-run"), "{saida}");
        assert!(saida.contains("Nada foi armado"), "{saida}");

        // O ensaio nao desarma, e diz isso. E a unica coisa que o §5.2 mostra
        // acontecendo antes do julgamento, e o `--dry-run` deste projeto ja
        // mentiu sobre exatamente ela uma vez (§11).
        assert!(saida.contains("nem desarmado"), "{saida}");

        // **Nenhuma frase do ensaio pode adiar o armar para uma etapa
        // futura.** Ate a E6 o rodape dizia "Armar e a etapa E7", e aquilo era
        // verdade; no instante em que a E7 ficou pronta, virou a pior mentira
        // que um `--dry-run` pode contar — uma afirmacao sobre o que o comando
        // de verdade faz. Este teste e o que impede a proxima frase dessas de
        // sobreviver a etapa que a torna falsa.
        assert!(
            !saida.contains("etapa E7"),
            "o ensaio ainda adia o armar para uma etapa que ja chegou:\n{saida}"
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
    fn o_ensaio_so_imprime_a_receita_que_este_comando_arma() {
        // Da E3 a E8 este ensaio imprimia tambem a de restauracao, marcada
        // como "quem a arma e a etapa E9" — e a frase virou mentira no
        // instante em que a E9 ficou pronta. O `arca restore --dry-run` monta
        // aquela receita com o disco de **destino**, que este comando nem
        // escolhe.
        let saida = ensaio_montado();

        assert!(
            saida.contains("o comando sem --dry-run armaria"),
            "{saida}"
        );
        assert!(
            !saida.contains("etapa E9"),
            "a E9 existe; nenhuma frase pode continuar prometendo-a:\n{saida}"
        );
        assert!(
            !saida.contains(receita(Operacao::Restauracao).comando()),
            "a receita de restauracao nao sai mais daqui:\n{saida}"
        );
        assert!(
            saida.contains("arca restore --dry-run"),
            "e o ensaio tem de dizer onde ela esta:\n{saida}"
        );
    }

    // ───────────────────────── o comando inteiro ─────────────────────────

    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        sistema: SistemaDeMentira,
        entropia: EntropiaDeMentira,
        console: ConsoleDeMentira,

        /// A quinta porta. Nenhum destes comandos a usa — ela existe aqui
        /// porque o `Contexto` e um so, e o duplo **registra** o que lhe
        /// mandaram fazer: com ele na bancada, `particionou()` e uma
        /// pergunta que qualquer teste pode fazer.
        particionador: ParticionadorDeMentira,
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

        /// O que o usuario digita na confirmacao de S-2.
        fn digitando(mut self, linhas: &[&str]) -> Bancada {
            self.console = ConsoleDeMentira::respondendo(linhas);
            self
        }

        fn com(discos: DiscosDeMentira, arquivos: ArquivosEmMemoria) -> Bancada {
            Bancada {
                arquivos,
                discos,
                firmware: FirmwareDeMentira::novo(),
                relogio: RelogioParado::em("2026-08-22T11:42:03"),
                sistema: SistemaDeMentira::novo(),
                entropia: EntropiaDeMentira::com(&[0xa3, 0xf1, 0xc9, 0xe0, 0x7b, 0x2d, 0x48, 0x56]),
                console: ConsoleDeMentira::mudo(),
                particionador: ParticionadorDeMentira::desta_mesa(),
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
                entropia: &self.entropia,
                console: &self.console,
                particionador: &self.particionador,
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

    /// O `bcdedit` desta maquina: entrada `ARCA`, sem boot unico antes do
    /// armar e com ele depois.
    const FIRMWARE_PT: &str =
        include_str!("../../recursos/capturas/bcdedit-enum-firmware-pt.txt");

    /// O `bcdedit` desta maquina, **modelado**: o comando desarma e depois
    /// arma, e as duas escritas caem no mesmo `{fwbootmgr}`.
    ///
    /// A ordem permanente e a medida em 22/08/2026 — so o `{bootmgr}`, com a
    /// entrada do ARCA **fora** dela. E sobre essa configuracao que o boot
    /// unico tem de funcionar (C-5).
    fn firmware_que_obedece() -> FirmwareDeMentira {
        FirmwareDeMentira::novo()
            .respondendo("firmware", FIRMWARE_PT)
            .modelando_o_fwbootmgr(&["{bootmgr}"])
    }

    fn bancada_completa() -> Bancada {
        Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", GRUB_INERTE),
        )
        .com_firmware(firmware_que_obedece())
    }

    #[test]
    fn sem_confirmacao_digitada_nada_e_armado() {
        // S-2. O caminho que **nao** pode existir: o pre-voo passa, a pessoa
        // nao digita o nome, e a maquina reinicia mesmo assim. Aqui ninguem
        // digitou nada, que e o que um `stdin` fechado produz.
        let bancada = bancada_completa();

        match executar(&bancada.contexto(false), "2026-08-22_Apps").unwrap_err() {
            Erro::ConfirmacaoNaoBate { esperado, digitado } => {
                assert_eq!(esperado, "2026-08-22_Apps");
                assert_eq!(digitado, "");
            }
            outro => panic!("esperava a confirmacao recusada, veio {outro}"),
        }

        assert!(
            bancada.arquivos.conteudo_de(r"R:\arca\estado.json").is_none(),
            "gravou estado de job sem confirmacao"
        );
        assert_eq!(
            bancada.arquivos.conteudo_de(r"R:\boot\grub\grub.cfg").as_deref(),
            Some(GRUB_INERTE),
            "armou o grub.cfg sem confirmacao"
        );
        // O desarmar de C-1 escreve no firmware, e tem de escrever — o que
        // nao pode e uma escrita que **arme**.
        let escritas = bancada.firmware.executados.borrow();
        assert!(
            escritas
                .iter()
                .all(|argumentos| argumentos.first().map(String::as_str) == Some("/deletevalue")),
            "escreveu no firmware alem do desarmar de C-1: {escritas:?}"
        );
        assert_eq!(bancada.sistema.reinicios(), 0, "reiniciou sem confirmacao");
    }

    #[test]
    fn a_confirmacao_e_o_nome_por_extenso_e_nunca_um_s() {
        // S-2 na letra. `s`, `sim` e o prefixo do nome sao todos recusados —
        // e todos deixam o dispositivo como estava.
        for digitado in ["s", "S", "sim", "2026-08-22", "2026-08-22_apps", ""] {
            let bancada = bancada_completa().digitando(&[digitado]);

            assert!(
                matches!(
                    executar(&bancada.contexto(false), "2026-08-22_Apps"),
                    Err(Erro::ConfirmacaoNaoBate { .. })
                ),
                "`{digitado}` foi aceito como confirmacao"
            );
            assert_eq!(bancada.sistema.reinicios(), 0);
        }
    }

    #[test]
    fn com_a_confirmacao_certa_o_comando_arma_e_so_entao_reinicia() {
        // O caminho inteiro da E7. A ordem importa e esta coberta em
        // `crate::armar`; o que se cobra aqui e que o comando faca as tres
        // coisas e **reinicie por ultimo**.
        let bancada = bancada_completa().digitando(&["2026-08-22_Apps"]);

        executar(&bancada.contexto(false), "2026-08-22_Apps").expect("arma e reinicia");

        let estado = bancada
            .arquivos
            .conteudo_de(r"R:\arca\estado.json")
            .expect("estado gravado");
        assert!(estado.contains("\"selo\": \"a3f1c9e07b2d4856\""));
        assert!(estado.contains("\"situacao\": \"armado\""));

        let grub = bancada
            .arquivos
            .conteudo_de(r"R:\boot\grub\grub.cfg")
            .expect("grub gravado");
        assert!(grub.contains("set default=\"arca-backup\""));
        assert!(grub.contains("ARCA_SELO=a3f1c9e07b2d4856"));

        assert!(
            bancada
                .firmware
                .executados
                .borrow()
                .iter()
                .any(|argumentos| argumentos.contains(&"bootsequence".to_string())),
            "nao marcou o boot unico"
        );
        assert_eq!(bancada.sistema.reinicios(), 1);
    }

    #[test]
    fn um_armar_que_falha_no_meio_nao_reinicia() {
        // O que separa este comando de um que dispara o reinicio sem saber se
        // armou. Aqui o `bcdedit` responde "êxito" e a releitura mostra que a
        // marca nao pegou — e a maquina fica onde esta.
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", GRUB_INERTE),
        )
        .com_firmware(
            // Um firmware que responde "êxito" e **nao** poe a marca: o
            // `{fwbootmgr}` sai sempre inerte, escreva-se o que se escrever.
            FirmwareDeMentira::novo()
                .respondendo("firmware", FIRMWARE_PT)
                .respondendo("{fwbootmgr}", FWBOOTMGR_INERTE),
        )
        .digitando(&["2026-08-22_Apps"]);

        assert!(matches!(
            executar(&bancada.contexto(false), "2026-08-22_Apps").unwrap_err(),
            Erro::BootUnicoNaoArmou { .. }
        ));
        assert_eq!(
            bancada.sistema.reinicios(),
            0,
            "reiniciou sem saber se tinha armado"
        );
    }

    #[test]
    fn sem_nome_de_disco_o_comando_recusa_antes_da_confirmacao() {
        // A pendencia que a E6 deixou, decidida: recusar, e nao pedir. Um
        // nome de disco do Linux digitado do lado Windows nao tem oraculo, e a
        // recusa acontece **antes** de a pessoa digitar o nome inteiro.
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            // Sem `blkdev.list`: nao ha de onde lê o nome do disco.
            ArquivosEmMemoria::novo()
                .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc")
                .com(r"R:\boot\grub\grub.cfg", GRUB_INERTE),
        )
        .com_firmware(firmware_que_obedece())
        .digitando(&["2026-08-22_Apps"]);

        assert!(matches!(
            executar(&bancada.contexto(false), "2026-08-22_Apps").unwrap_err(),
            Erro::DiscoDeOrigemPorDeterminar { .. }
        ));
        assert_eq!(
            bancada.console.lidas.get(),
            0,
            "pediu a confirmacao antes de saber que ia recusar"
        );
        assert!(bancada.arquivos.conteudo_de(r"R:\arca\estado.json").is_none());
    }

    #[test]
    fn o_pre_voo_desarma_de_verdade_como_c1_manda() {
        // C-1 nao e condicional a chegar ao armar: desarmar e o primeiro passo
        // de todo comando. Um dispositivo armado com receita velha nao pode
        // sair daqui com "pre-voo concluido" e continuar com a receita velha —
        // e isto vale mesmo quando a confirmacao recusa logo depois, que e o
        // caso construido aqui.
        let armado = include_str!("../../recursos/capturas/grub-backup-arca-teste-03.cfg");
        let bancada = Bancada::com(
            DiscosDeMentira::com_dispositivo(),
            vault_com_as_imagens().com(r"R:\boot\grub\grub.cfg", armado),
        )
        .com_firmware(firmware_que_obedece());

        assert!(matches!(
            executar(&bancada.contexto(false), "2026-08-22_Apps").unwrap_err(),
            Erro::ConfirmacaoNaoBate { .. }
        ));

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
        let bancada = bancada_completa().digitando(&["2026-08-22_Apps"]);

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
                medida: None,
                em_uso_bytes: 1000,
                tipo_de_midia: TipoDeMidia::DiscoExterno,
                letras: vec!['E', 'R'],
            },
            DiscoFisico {
                indice: 1,
                modelo: "O WINDOWS".to_string(),
                tamanho_bytes: 500_105_249_280,
                medida: None,
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
