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
//! Desarma (C-1), imprime o que ja aconteceu, pergunta, arma, avisa C-9 e
//! reinicia. O que muda e a receita: sem `ocs-sr`, sem `ocs-chkimg`, sem
//! `savedisk` e sem `restoredisk` — so `lsblk`, e nada e escrito fora do
//! `ARCAVAULT`.
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
use crate::receita::{Operacao, Pedido, Receita, Selo};
use crate::sondagem;

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let caminho_do_grub = dispositivo.caminho_do_grub()?;

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
    use crate::blkdev::{Fonte, Lista};
    use crate::duplos::{DiscosDeMentira, momento};

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
}
