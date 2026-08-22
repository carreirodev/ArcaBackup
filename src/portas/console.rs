//! A porta do que o usuario digita.
//!
//! Uma porta para um `read_line` parece exagero pela mesma razao que a porta
//! da entropia parecia, e nao e pela mesma razao: **o que atravessa esta
//! fronteira e S-2**. "Operacao destrutiva exige texto digitado, nunca so `s`"
//! e um requisito de seguranca, e um requisito de seguranca sem teste e uma
//! frase.
//!
//! Sem porta, a unica forma de exercitar a confirmacao seria escrever no
//! `stdin` do processo de teste — o que faria o teste depender de como o
//! `cargo test` foi invocado — ou nao exercita-la, deixando o caminho que
//! separa "armou" de "nao armou" sem cobertura nenhuma. A E7 e a primeira
//! etapa em que digitar errado tem consequencia, entao e agora que ela entra.
//!
//! # S-1 continua valendo
//!
//! Como as outras, esta porta nao entrega handle de dispositivo, caminho bruto
//! nem deslocamento em setores. Ela entrega uma linha de texto.

use crate::erro::Resultado;

pub trait Console {
    /// Uma linha do que o usuario digitou, sem a quebra de linha.
    ///
    /// Fim de entrada devolve linha vazia, e nao erro: um `stdin` fechado e
    /// alguem que nao digitou nada, e nao digitar nada **nunca** confirma
    /// coisa alguma. Transformar isso em erro daria dois caminhos para a mesma
    /// recusa, e o que importa e que nenhum deles arme.
    fn ler_linha(&self) -> Resultado<String>;
}
