//! A confirmacao digitada de S-2, e o julgamento dela.
//!
//! Nasceu dentro de `src/comandos/backup.rs`, na etapa E7, e saiu de la na E9
//! quando um **segundo** comando destrutivo passou a precisar da mesma regra.
//! O que se compartilha e o julgamento; a tela e de cada comando, e as duas
//! sao diferentes de proposito — o §5.2 pede o nome do backup e o §6.1 avisa
//! antes que a operacao APAGA o disco de destino.
//!
//! Duas versoes da mesma regra divergem na primeira mudanca. Esta e a regra que
//! separa "armou" de "nao armou" nos dois comandos que armam.

use crate::app::Contexto;
use crate::erro::{Erro, Resultado};
use crate::nome::Nome;

/// Pede o texto e recusa se ele nao bater.
///
/// Uma tentativa, e nao um laco. Quem digitou errado tem o comando inteiro
/// para repetir, e o comando e barato: ate aqui ele nao armou nada. Insistir
/// transformaria a confirmacao numa formalidade a atravessar.
///
/// `pergunta` e o texto que vai antes dos dois-pontos, sem eles — cada comando
/// escreve o seu.
pub fn pedir(contexto: &Contexto, pergunta: &str, nome: &Nome) -> Resultado<()> {
    pedir_texto(contexto, pergunta, nome.como_texto())
}

/// A mesma regra, sobre um texto que não é um [`Nome`].
///
/// # Por que ela existe, e por que o julgamento é o mesmo
///
/// O `arca prepare` da E10 pede **o modelo do disco**, e modelo não passa por
/// B-2: `KGSSE100 256` tem espaço, e `JMicron Generic SCSI Disk Device` tem
/// quatro. Um `Nome` ali seria mentira de tipo.
///
/// O que **não** muda é o julgamento — comparação exata, sem ignorar caixa,
/// sem aceitar prefixo, uma tentativa. Duas versões da mesma regra divergem na
/// primeira mudança, e esta é a regra que separa "apagou o disco" de "não
/// apagou" em três comandos.
///
/// E o modelo é o texto certo a pedir aqui pelo mesmo motivo que a restauração
/// pede o nome da imagem: **é o que está na tela, e digitá-lo custa lê-lo**. Um
/// índice — `1` — é curto demais para custar alguma coisa, e é justamente o
/// número que muda de uma conexão para outra.
pub fn pedir_texto(contexto: &Contexto, pergunta: &str, esperado: &str) -> Resultado<()> {
    use std::io::Write;

    print!("\n{pergunta}: ");
    let _ = std::io::stdout().flush();

    let digitado = contexto.console.ler_linha()?;
    println!();

    if digitado.trim() != esperado {
        return Err(Erro::ConfirmacaoNaoBate {
            esperado: esperado.to_string(),
            digitado: digitado.trim().to_string(),
        });
    }
    Ok(())
}

/// A pergunta de uma tecla, com o padrao no **nao**.
///
/// # Ela nao e S-2, e nao finge ser
///
/// S-2 pede o **alvo** por extenso, e existe para custar lê-lo: o nome da
/// imagem que vai ser gravada, o modelo do disco que vai ser apagado. O que
/// ela impede e agir sobre a coisa errada.
///
/// Esta pergunta impede outra coisa: **agir sem ter lido a tela**. O
/// `arca prepare` a usa como primeiro tempo de PR-4, antes de S-2, para dar a
/// chance de sair depois de lê o plano; e o `arca sondar` a usa **sozinha**
/// (SD-6), porque a sondagem nao tem alvo a confirmar — ela nao apaga nada e
/// nao escolhe nada, e o que ela faz de irreversivel e reiniciar a maquina.
///
/// **Pedir a palavra `sondar` por extenso seria ruido**: quem acabou de
/// digitar `arca sondar` a ecoaria sem lê nada, e uma confirmacao que so ecoa
/// o comando ensina a digitar sem lê — que e o contrario do que S-2 compra.
///
/// Saiu de `src/comandos/prepare.rs` na E12, quando um segundo comando passou
/// a precisar dela. Duas copias divergiriam na primeira mudanca, e uma delas
/// passaria a aceitar um `sim` que a outra recusa.
pub fn perguntar_se_pode(contexto: &Contexto, pergunta: &str) -> Resultado<bool> {
    use std::io::Write;

    print!("\n{pergunta} (s/N): ");
    let _ = std::io::stdout().flush();

    let resposta = contexto.console.ler_linha()?;
    println!();

    Ok(e_sim(&resposta))
}

/// Se a resposta e um sim.
///
/// Lista de permissao, como B-2 e pelo mesmo motivo: o que nao esta aqui e
/// **nao**. Um Enter vazio, um `n`, um `talvez` e um `S1M` sao todos nao — e o
/// padrao ser o nao e o que faz a tecla valer alguma coisa.
pub fn e_sim(resposta: &str) -> bool {
    matches!(resposta.trim(), "s" | "S" | "sim" | "SIM")
}

/// Se o texto digitado confirma este nome (S-2).
///
/// # Exato, e nao "parecido"
///
/// Poda espaco das pontas, porque um Enter deixa `\r\n` atras e ninguem digita
/// espaco de proposito. **Nao** ignora caixa: B-2 aceita maiuscula e
/// minuscula, e `2026-08-22_apps` e um nome diferente de `2026-08-22_Apps` —
/// aceitar os dois faria a confirmacao dizer sim para uma imagem que nao e a
/// que vai ser gravada, ou restaurada. E nao aceita prefixo, nem `s`, nem
/// vazio: a confirmacao existe para custar o trabalho de lê o nome inteiro.
pub fn bate(digitado: &str, nome: &Nome) -> bool {
    digitado.trim() == nome.como_texto()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn nome() -> Nome {
        Nome::novo("2026-08-22_Apps").expect("nome valido por B-2")
    }

    #[test]
    fn o_nome_exato_confirma() {
        assert!(bate("2026-08-22_Apps", &nome()));
    }

    #[test]
    fn o_enter_deixa_para_tras_e_nao_atrapalha() {
        assert!(bate("2026-08-22_Apps\r\n", &nome()));
        assert!(bate("  2026-08-22_Apps  ", &nome()));
    }

    #[test]
    fn a_caixa_importa() {
        // B-2 aceita as duas caixas, entao sao dois nomes de imagem
        // diferentes. Ignorar a caixa faria a confirmacao dizer sim para uma
        // imagem que nao e a que vai ser tocada.
        assert!(!bate("2026-08-22_apps", &nome()));
        assert!(!bate("2026-08-22_APPS", &nome()));
    }

    #[test]
    fn prefixo_sozinho_e_vazio_nao_confirmam() {
        assert!(!bate("2026-08-22", &nome()));
        assert!(!bate("s", &nome()));
        assert!(!bate("", &nome()));
        assert!(!bate("   ", &nome()));
    }

    #[test]
    fn sufixo_a_mais_nao_confirma() {
        assert!(!bate("2026-08-22_Apps2", &nome()));
    }

    // ────────── a pergunta de uma tecla, e o padrao dela ──────────

    #[test]
    fn o_padrao_da_pergunta_de_uma_tecla_e_o_nao() {
        // **E o que faz a tecla valer alguma coisa.** Ela e a unica barreira do
        // `arca sondar` (SD-6), e o que ela impede e o reinicio de quem digitou
        // o comando sem ler o que ele faz. Um Enter distraido que passasse por
        // sim nao impediria nada.
        assert!(e_sim("s"));
        assert!(e_sim("S"));
        assert!(e_sim("sim"));
        assert!(e_sim("SIM"));
        assert!(e_sim(" s \r\n"), "o Enter deixa `\\r\\n` para tras");

        for nao in ["", "   ", "\r\n", "n", "N", "nao", "talvez", "sm", "y", "1"] {
            assert!(!e_sim(nao), "`{nao}` passou por sim");
        }
    }
}
