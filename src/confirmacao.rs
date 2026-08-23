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
}
