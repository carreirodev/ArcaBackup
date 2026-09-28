//! `arca sondar` — descobrir os discos desta maquina sem fazer backup nem
//! restauracao (E12, SD-1 a SD-6).
//!
//! # O buraco que ele fecha, em uma frase
//!
//! O §4.5 diz que o nome do disco no Linux sai do `blkdev.list` de dentro de
//! uma imagem. Um dispositivo recem-preparado **nao tem imagem**, logo nao tem
//! o nome, logo `arca backup` recusa — e `arca restore` e
//! `arca verify --completo` tambem. Nenhum dos tres comandos que armam
//! funcionava num dispositivo recem-nascido, e a saida que o `arca prepare`
//! oferecia era um backup pelo menu do Clonezilla: exatamente aquilo que este
//! app existe para nao precisar.
//!
//! A sondagem da uma **segunda fonte para o mesmo arquivo**, e ela nao depende
//! de imagem nenhuma. O parser nao muda: [`crate::blkdev`] continua sendo o
//! unico lugar que lê aquele formato.
//!
//! # Ele arma como os outros tres, e nao faz mais nada
//!
//! Desarma (C-1), imprime o que ja aconteceu, recusa o dispositivo que C-6 e
//! C-10 recusam, pergunta, arma, avisa C-9 e reinicia. O que muda e a
//! receita: sem `ocs-sr`, sem `ocs-chkimg`, sem `savedisk` e sem
//! `restoredisk` — so `lsblk`, e nada e escrito fora do `ARCAVAULT`.
//!
//! # Por que ele nao lê imagem nenhuma antes de armar
//!
//! Os outros tres comandos enumeram o `ARCAVAULT` porque precisam julgar uma
//! imagem: B-3 recusa nome repetido, L-2 recusa residuo, R-1 lista o que ha.
//! A sondagem nao tem imagem por sujeito, e o dispositivo em que ela mais
//! importa **nao tem nenhuma**. Enumerar aqui seria trabalho cujo resultado
//! nao muda nada — e uma tela dizendo "Nenhuma imagem" logo antes de um
//! comando que existe para esse caso.
//!
//! O que ele **imprime** e o que a sondagem vai substituir, quando ha uma:
//! quem sonda pela segunda vez merece saber que a medicao anterior vai embora
//! (SD-4).

use crate::app::Contexto;
use crate::armar;
use crate::blkdev;
use crate::desarme;
use crate::dispositivo::{self, Dispositivo};
use crate::erro::{Erro, Resultado};
use crate::formato::{dia_e_hora, gigabytes, linha};
use crate::prevoo;
use crate::receita::{Operacao, Pedido, Receita, Selo};
use crate::sondagem;

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let caminho_do_grub = dispositivo.caminho_do_grub()?;

    // A consulta ao WMI vem **antes** do desarme, no mesmo lugar que o
    // `arca backup` e o `arca restore` a fazem: se ela falhar depois do
    // desarme, o erro sobe antes do cabecalho e a noticia de que o desarme
    // aconteceu se perde. Custa uns 2 s, e a E12 nasceu sem ela (WPC-53).
    let discos = contexto.discos.discos_fisicos()?;

    // A leitura acontece **antes** do desarme, e nao depois: e a sondagem
    // anterior, e e o que a tela vai dizer que sera substituido. Desarmar nao
    // toca no `ARCAVAULT`, entao a ordem nao muda o valor — muda o que se tem
    // em maos se o desarme falhar.
    let anterior = sondagem::ler(contexto.arquivos, &raiz_do_vault);

    // C-1, incondicionalmente e como primeiro passo: este comando arma.
    let desarme = if contexto.dry_run {
        None
    } else {
        Some(desarme::executar(
            contexto.arquivos,
            contexto.firmware,
            &caminho_do_grub,
        )?)
    };

    // O que ja aconteceu, impresso antes de qualquer recusa poder cortar a
    // saida. E a armadilha que a revisao da E7 pegou no `arca backup`, que a
    // E9 cometeu de novo no `arca restore` e que a E11 ja escreveu certo.
    print!(
        "{}",
        montar_o_cabecalho(&dispositivo, anterior.as_ref(), &caminho_do_grub, &desarme)
    );

    // C-6 e C-10, antes da pergunta: sao sobre o **dispositivo**, e valem para
    // toda operacao que arma. A E12 nasceu sem elas, como a E9 tinha nascido
    // — ver [`crate::prevoo::julgar_o_dispositivo`] para os dois furos.
    prevoo::julgar_o_dispositivo(&dispositivo, &discos).map_err(Erro::PreVooRecusou)?;

    if contexto.dry_run {
        print!("{}", ensaio_da_receita()?);
        return Ok(());
    }

    print!("{}", montar_o_que_vai_acontecer());

    // SD-6: uma tecla, com o padrao no nao. Nao ha alvo a confirmar por
    // extenso — ver [`crate::confirmacao::perguntar_se_pode`].
    if !crate::confirmacao::perguntar_se_pode(contexto, "Reiniciar agora e sondar?")? {
        println!("Nada foi armado, e o dispositivo esta inerte.\n");
        return Ok(());
    }

    let armado = armar::executar(
        contexto.arquivos,
        contexto.firmware,
        contexto.entropia,
        contexto.relogio,
        &armar::Pedir {
            dispositivo: &dispositivo,
            operacao: Operacao::Sondagem,
            // A sondagem nao opera sobre imagem nenhuma, e nao nomeia disco: o
            // `lsblk` olha todos. Quem cobra essa coerencia e `Receita::montar`.
            nome: None,
            disco: None,
        },
    )?;

    contexto.registro.info(format!(
        "armada sondagem · selo {} · desfecho em {}",
        armado.selo, armado.pasta_do_desfecho
    ));

    print!("{}", montar_o_armado(&armado));

    contexto.sistema.reiniciar().inspect_err(|_| {
        eprintln!(
            "\nO dispositivo FICOU ARMADO e a maquina nao reiniciou. O proximo reinicio,\n\
             seja qual for a causa, vai bootar no dispositivo e rodar a sondagem.\n\
             Para desfazer:  arca desarmar"
        );
    })
}

/// O dispositivo, o desarme e o que ha de sondagem hoje.
pub fn montar_o_cabecalho(
    dispositivo: &Dispositivo,
    anterior: Option<&blkdev::Lista>,
    caminho_do_grub: &std::path::Path,
    desarme: &Option<desarme::Desarme>,
) -> String {
    let mut saida = format!(
        "\nDispositivo ARCA: {} ({}) · {} livres\n\n",
        dispositivo::ARCAVAULT,
        dispositivo
            .vault
            .letra
            .map_or("sem letra".to_string(), |letra| format!("{letra}:")),
        gigabytes(dispositivo.vault.livre_bytes)
    );

    saida.push_str(&desarme::linha_do_desarme(
        desarme.as_ref(),
        &caminho_do_grub.to_string_lossy(),
    ));

    // SD-4 dito na tela, e nao so no ADR: a pasta e fixa, e a sondagem nova
    // escreve por cima da anterior. Quem sonda pela segunda vez merece lê isso
    // antes, e nao descobrir depois que a medicao de ontem sumiu.
    saida.push_str(&linha(
        "Sondagem de hoje",
        &match anterior {
            Some(lista) => {
                let quando = match &lista.fonte {
                    blkdev::Fonte::Sondagem { quando } => *quando,
                    blkdev::Fonte::Imagem(_) => None,
                };
                format!(
                    "de {} · {} disco(s) · SERA SUBSTITUIDA pela de agora",
                    dia_e_hora(quando),
                    blkdev::ler(&lista.texto).len()
                )
            }
            None => "nenhuma · esta sera a primeira".to_string(),
        },
    ));

    saida
}

/// O que a sondagem faz, dito antes da pergunta.
///
/// # Ela e a mais barata das quatro operacoes, e a tela diz isso
///
/// Nao ha `ocs-sr`: nao ha `savedisk`, nao ha `restoredisk`, e nada e escrito
/// fora do `ARCAVAULT`. O pior caso e a maquina parar num menu, que e chato e
/// nao destroi nada.
///
/// **O que a tela nao diz e quanto tempo leva**, e a ausencia e deliberada: o
/// custo de um boot do Clonezilla isolado nao esta medido neste repositorio —
/// toda execucao anterior tinha uma operacao longa depois dele —, e esta etapa
/// existe, entre outras coisas, para medi-lo. Pôr aqui um palpite seria
/// exatamente o que o §3.5 do PRD conta ter custado caro cinco vezes.
pub fn montar_o_que_vai_acontecer() -> &'static str {
    concat!(
        "\nA SONDAGEM NAO FAZ BACKUP NEM RESTAURACAO. Ela reinicia a maquina, roda o\n",
        "`lsblk` no Linux do Clonezilla, grava a saida no ARCAVAULT e desliga.\n",
        "Nenhum programa do Clonezilla e chamado, e nada e escrito fora do ARCAVAULT.\n",
        "\n",
        "O QUE VOCE GANHA: o nome que o LINUX da ao disco desta maquina (`nvme0n1`), que\n",
        "e o que a receita de backup e a de restauracao precisam nomear e que o Windows\n",
        "nao conhece (§4.5). Sem ele, `arca backup` recusa.\n",
        "\n",
        "O QUE ISSO CUSTA: um reinicio, e o que estiver aberto se perde. A maquina\n",
        "desliga sozinha ao terminar.\n"
    )
}

/// O que se imprime depois de armado, com o aviso de C-9 no fim.
///
/// As cinco linhas do meio sao as mesmas dos outros tres comandos que armam —
/// [`armar::montar_as_linhas`]. O que muda e o que vem depois: aqui nada esta
/// sendo gravado nem apagado, e o que se colhe na volta e um nome de disco.
pub fn montar_o_armado(armado: &armar::Armado) -> String {
    let mut saida = String::from("\n");
    saida.push_str(&armar::montar_as_linhas(armado));
    saida.push_str(armar::montar_o_que_vem_pela_frente());

    saida.push_str(concat!(
        "\nAO TERMINAR: remova o SSD antes de religar.\n",
        "\nDepois de religar, `arca resultado` colhe o desfecho — e dali em diante\n",
        "`arca backup <nome>` acha o disco de origem sozinho.\n",
        "\nReiniciando...\n"
    ));

    saida
}

/// A receita inteira, so no `--dry-run`.
fn ensaio_da_receita() -> Resultado<String> {
    // O selo de verdade nasce ao armar. Este e de ensaio, e a saida o diz.
    let receita = Receita::montar(&Pedido {
        operacao: Operacao::Sondagem,
        nome: None,
        disco: None,
        selo: Selo::de_ensaio(),
    })
    .map_err(Erro::ReceitaRecusada)?;

    Ok(format!(
        concat!(
            "\nEnsaio: nada foi armado e o dispositivo nao foi desarmado.\n",
            "\nA receita da sondagem, como iria para o grub.cfg:\n\n{}\n",
            "\nO selo acima e de ensaio — dezesseis zeros. O de verdade nasce ao armar.\n",
            "\nAS FLAGS DO `lsblk` SAO RECONSTRUCAO, e nao transcricao: temos o formato do\n",
            "arquivo que se quer produzir — o `blkdev.list` de dentro das imagens — e nao\n",
            "a linha de comando que o produziu, que mora nos scripts do Clonezilla. Uma\n",
            "flag recusada vira `ARCA_PROBE=FALHOU`, e a mensagem do `lsblk` fica dentro\n",
            "do proprio `blkdev.list` para a proxima sessao lê.\n"
        ),
        receita.comando()
    ))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::blkdev::{Fonte, Lista};
    use crate::duplos::{
        ArquivosEmMemoria, ConsoleDeMentira, DiscosDeMentira, EntropiaDeMentira, FirmwareDeMentira,
        ParticionadorDeMentira, RelogioParado, SistemaDeMentira, momento,
    };
    use crate::portas::{DiscoFisico, TipoDeMidia};
    use crate::prevoo::RecusaDoPreVoo;
    use crate::registro::Registro;

    fn dispositivo_conectado() -> Dispositivo {
        dispositivo::encontrar(&DiscosDeMentira::com_dispositivo()).unwrap()
    }

    /// O `blkdev.list` como a sondagem o grava.
    const DA_SONDAGEM: &str = concat!(
        "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
        "sda       sda         238.5G disk                                               KGSSE100256\n",
        "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
    );

    fn caminho_do_grub() -> std::path::PathBuf {
        std::path::PathBuf::from(r"F:\boot\grub\grub.cfg")
    }

    #[test]
    fn sem_sondagem_anterior_a_tela_diz_que_esta_e_a_primeira() {
        let saida = montar_o_cabecalho(&dispositivo_conectado(), None, &caminho_do_grub(), &None);

        assert!(saida.contains("Sondagem de hoje"), "{saida}");
        assert!(saida.contains("esta sera a primeira"), "{saida}");
        assert!(
            !saida.contains("SERA SUBSTITUIDA"),
            "nao ha o que substituir: {saida}"
        );
    }

    #[test]
    fn havendo_sondagem_anterior_a_tela_avisa_que_ela_vai_embora() {
        // SD-4 na tela. A pasta e fixa e a segunda sondagem escreve por cima;
        // quem sonda de novo tem de lê isso **antes**, e nao descobrir depois
        // que a medicao de ontem sumiu.
        let anterior = Lista {
            fonte: Fonte::Sondagem {
                quando: Some(momento("2026-08-23T21:14:07")),
            },
            texto: DA_SONDAGEM.to_string(),
        };

        let saida = montar_o_cabecalho(
            &dispositivo_conectado(),
            Some(&anterior),
            &caminho_do_grub(),
            &None,
        );

        assert!(saida.contains("SERA SUBSTITUIDA"), "{saida}");
        assert!(saida.contains("23/08 21:14"), "{saida}");
        assert!(saida.contains("2 disco(s)"), "{saida}");
    }

    #[test]
    fn a_tela_de_antes_da_pergunta_diz_o_que_a_sondagem_nao_faz() {
        // O que separa esta operacao das outras tres e o que ela **nao** faz,
        // e e isso que decide se alguem aperta `s`.
        let saida = montar_o_que_vai_acontecer();

        assert!(saida.contains("NAO FAZ BACKUP NEM RESTAURACAO"));
        assert!(saida.contains("reinicio"), "o custo real esta dito");
        assert!(saida.contains("desliga sozinha"));
    }

    #[test]
    fn a_tela_nao_promete_tempo_nenhum() {
        // O custo de um boot do Clonezilla isolado nao esta medido neste
        // repositorio, e esta etapa existe para medi-lo. Um `~2 minutos` aqui
        // seria palpite vestido de medicao — o padrao que o §3.5 do PRD conta
        // ter custado caro cinco vezes.
        let saida = montar_o_que_vai_acontecer();

        for palpite in ["minuto", "segundo", "min ", " s.", "demora"] {
            assert!(
                !saida.contains(palpite),
                "a tela promete tempo (`{palpite}`): {saida}"
            );
        }
    }

    #[test]
    fn o_ensaio_imprime_a_receita_e_diz_que_o_selo_e_de_mentira() {
        let saida = ensaio_da_receita().unwrap();

        assert!(saida.contains("lsblk"), "{saida}");
        assert!(saida.contains("ARCA_PROBE=OK"), "{saida}");
        assert!(saida.contains("ARCA_PROBE=FALHOU"), "{saida}");
        assert!(saida.contains("0000000000000000"), "{saida}");
        assert!(saida.contains("de ensaio"), "{saida}");
    }

    #[test]
    fn o_ensaio_diz_que_as_flags_sao_reconstrucao() {
        // A distincao que a E12 estreia: das outras receitas ha a linha de
        // comando que rodou, e desta ha so o resultado que se quer reproduzir.
        // Deixar isso so no codigo faria a tela apresentar reconstrucao como
        // transcricao — que e o padrao que o §3.5 nomeia.
        let saida = ensaio_da_receita().unwrap();

        assert!(saida.contains("RECONSTRUCAO"), "{saida}");
        assert!(saida.contains("nao transcricao"), "{saida}");
    }

    #[test]
    fn a_receita_do_ensaio_nao_chama_nada_do_clonezilla() {
        // SD-1, e e o que torna esta a operacao mais barata do projeto: sem
        // `ocs-sr` nao ha `savedisk` nem `restoredisk`, e nada e escrito fora
        // do `ARCAVAULT`.
        let saida = ensaio_da_receita().unwrap();

        for programa in ["ocs-sr", "ocs-chkimg", "savedisk", "restoredisk"] {
            assert!(
                !saida.contains(programa),
                "a receita da sondagem chama `{programa}`: {saida}"
            );
        }
    }

    #[test]
    fn o_armado_manda_colher_e_diz_o_que_vem_depois() {
        // A sondagem so vale se alguem colher: o `blkdev.list` fica no
        // dispositivo, e quem encerra o job e o `arca resultado`.
        let armado = armar::Armado {
            caminho_do_estado: std::path::PathBuf::from(r"F:\arca\estado.json"),
            caminho_do_grub: caminho_do_grub(),
            selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
            entrada: armar::Entrada::JaEraDoArca,
            identificador: "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string(),
            alvo: crate::firmware::Alvo::ParticaoComLetra('F'),
            caminho_do_desfecho: std::path::PathBuf::from(r"E:\ARCA-LOGS\sondagem\arca-fim.txt"),
            pasta_do_desfecho: "sondagem".to_string(),
        };

        let saida = montar_o_armado(&armado);

        assert!(saida.contains("arca resultado"), "{saida}");
        assert!(saida.contains("arca backup"), "{saida}");
        assert!(saida.contains("remova o SSD"), "C-9: {saida}");
    }

    // ─────────────────────────── o comando inteiro ───────────────────────────

    const GRUB_INERTE: &str = include_str!("../../recursos/capturas/grub-inerte-arcaboot.cfg");

    /// O `bcdedit` desta maquina, com o `{fwbootmgr}` modelado: o comando
    /// desarma e depois arma, e as duas escritas caem no mesmo lugar.
    const FIRMWARE_PT: &str = include_str!("../../recursos/capturas/bcdedit-enum-firmware-pt.txt");

    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        sistema: SistemaDeMentira,
        entropia: EntropiaDeMentira,
        console: ConsoleDeMentira,

        /// O `Contexto` e um so, e a porta vem junto mesmo sem uso aqui.
        particionador: ParticionadorDeMentira,
        registro: Registro,
    }

    impl Bancada {
        /// O dispositivo com os discos pedidos, o `grub.cfg` inerte, e alguem
        /// que responde **sim** a pergunta de SD-6. Com o sim dado, a unica
        /// coisa que pode separar o comando de armar e a recusa.
        fn com_discos(discos: Vec<DiscoFisico>) -> Bancada {
            Bancada {
                arquivos: ArquivosEmMemoria::novo().com(r"R:\boot\grub\grub.cfg", GRUB_INERTE),
                discos: DiscosDeMentira::com_dispositivo().com_discos(discos),
                firmware: FirmwareDeMentira::novo()
                    .respondendo("firmware", FIRMWARE_PT)
                    .modelando_o_fwbootmgr(&["{bootmgr}"]),
                relogio: RelogioParado::em("2026-09-28T10:30:00"),
                sistema: SistemaDeMentira::novo(),
                entropia: EntropiaDeMentira::com(&[0xa3, 0xf1, 0xc9, 0xe0, 0x7b, 0x2d, 0x48, 0x56]),
                console: ConsoleDeMentira::respondendo(&["s"]),
                particionador: ParticionadorDeMentira::desta_mesa(),
                registro: Registro::em(
                    std::env::temp_dir().join(format!(
                        "arca-sondar-{}-{:?}",
                        std::process::id(),
                        std::thread::current().id()
                    )),
                    Box::new(RelogioDoSistema),
                ),
            }
        }

        fn contexto(&self) -> Contexto<'_> {
            Contexto {
                dry_run: false,
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

        /// O mesmo contexto, com `--dry-run`.
        fn ensaio(&self) -> Contexto<'_> {
            Contexto {
                dry_run: true,
                ..self.contexto()
            }
        }

        /// O que "recusou sem armar nada" quer dizer, uma coisa por linha.
        ///
        /// O desarmar de C-1 escreve no firmware e tem de escrever: ele vem
        /// antes de qualquer recusa. O que nao pode e uma escrita que **arme**.
        fn nada_foi_armado(&self) {
            assert_eq!(
                self.console.lidas.get(),
                0,
                "fez a pergunta antes de saber que ia recusar"
            );
            assert!(
                self.arquivos.conteudo_de(r"R:\arca\estado.json").is_none(),
                "gravou estado de job"
            );
            assert_eq!(
                self.arquivos
                    .conteudo_de(r"R:\boot\grub\grub.cfg")
                    .as_deref(),
                Some(GRUB_INERTE),
                "armou o grub.cfg"
            );
            let escritas = self.firmware.executados();
            assert!(
                escritas.iter().all(
                    |argumentos| argumentos.first().map(String::as_str) == Some("/deletevalue")
                ),
                "escreveu no firmware alem do desarmar de C-1: {escritas:?}"
            );
            assert_eq!(self.sistema.reinicios(), 0, "reiniciou");
        }
    }

    impl Drop for Bancada {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.registro.caminho().parent().unwrap());
        }
    }

    /// Dois dispositivos meio prontos: o `ARCAVAULT` (`E:`) no disco desta
    /// mesa, e o `ARCABOOT` (`R:`) sozinho num outro.
    fn discos_partidos() -> Vec<DiscoFisico> {
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1].letras = vec!['E'];
        discos.push(DiscoFisico {
            indice: 2,
            modelo: "OUTRO SSD".to_string(),
            tamanho_bytes: 1_000,
            medida: None,
            em_uso_bytes: 0,
            tipo_de_midia: TipoDeMidia::DiscoExterno,
            letras: vec!['R'],
        });
        discos
    }

    #[test]
    fn o_sondar_recusa_o_dispositivo_partido_antes_da_pergunta() {
        // C-10, que a E12 nasceu sem (WPC-53, 28/09/2026). Com dois
        // dispositivos meio prontos na mesa — o `ARCAVAULT` num, o `ARCABOOT`
        // noutro —, cada rotulo aparece uma vez e `dispositivo::encontrar`
        // passa. O `estado.json` iria para um e o `arca-fim.txt` da sondagem
        // para o outro, e a colheita procuraria o desfecho no lugar errado.
        let bancada = Bancada::com_discos(discos_partidos());

        let erro = executar(&bancada.contexto()).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido {
                    vault: 'E',
                    boot: 'R'
                })
            ),
            "veio {erro}"
        );
        bancada.nada_foi_armado();
    }

    #[test]
    fn o_sondar_recusa_midia_removivel_antes_da_pergunta() {
        // C-6 (WPC-53, 28/09/2026). O `armar` pegaria a rejeicao silenciosa
        // do `bcdedit` na releitura do `device` — mas so **depois** do sim.
        // O `MediaType` do WMI sabe antes, e a pergunta nao chega a ser feita.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1].tipo_de_midia = TipoDeMidia::Removivel;
        let bancada = Bancada::com_discos(discos);

        let erro = executar(&bancada.contexto()).unwrap_err();

        assert!(
            matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::MidiaRemovivel)),
            "veio {erro}"
        );
        bancada.nada_foi_armado();
    }

    #[test]
    fn o_ensaio_do_sondar_tambem_recusa_o_dispositivo_partido() {
        // O `--dry-run` mostra o que o comando faria, e com o dispositivo
        // partido ele recusaria. Um ensaio que imprimisse a receita diria que
        // a sondagem ia armar, e ela nao ia. Decidido em 28/09/2026 (WPC-53),
        // igual ao `arca backup` e ao `arca restore`.
        let bancada = Bancada::com_discos(discos_partidos());

        let erro = executar(&bancada.ensaio()).unwrap_err();

        assert!(
            matches!(
                erro,
                Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido {
                    vault: 'E',
                    boot: 'R'
                })
            ),
            "veio {erro}"
        );
        // O ensaio nao desarma: nem o `/deletevalue` de C-1 chega ao firmware.
        assert!(
            bancada.firmware.executados().is_empty(),
            "o ensaio escreveu no firmware: {:?}",
            bancada.firmware.executados()
        );
        bancada.nada_foi_armado();
    }

    #[test]
    fn com_o_sim_o_sondar_arma_e_so_entao_reinicia() {
        // O controle das duas recusas acima: sem ele, um `sondar` que
        // recusasse sempre passaria nelas. O dispositivo desta mesa tem os
        // dois rotulos no mesmo disco externo, e passa.
        let bancada = Bancada::com_discos(crate::duplos::discos_desta_mesa());

        executar(&bancada.contexto()).expect("arma e reinicia");

        let estado = bancada
            .arquivos
            .conteudo_de(r"R:\arca\estado.json")
            .expect("estado gravado");
        assert!(estado.contains("\"situacao\": \"armado\""), "{estado}");

        let grub = bancada
            .arquivos
            .conteudo_de(r"R:\boot\grub\grub.cfg")
            .expect("grub gravado");
        assert!(grub.contains("ARCA_PROBE"), "{grub}");

        assert!(
            bancada
                .firmware
                .executados()
                .iter()
                .any(|argumentos| argumentos.contains(&"bootsequence".to_string())),
            "nao marcou o boot unico"
        );
        assert_eq!(bancada.sistema.reinicios(), 1);
    }
}
