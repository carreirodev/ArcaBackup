//! A porta do privilegio administrativo.
//!
//! O ARCA nao tem operacao que rode sem elevacao, e o manifesto embutido faz
//! o Windows elevar antes de o programa comecar. Esta porta existe para o
//! caso em que o manifesto nao vigora: ela detecta e relanca, repassando os
//! argumentos **originais** — nunca reconstruidos a partir do que o parser
//! entendeu, que e onde `--dry-run` se perde (C-7).

use crate::erro::Resultado;

pub trait Privilegios {
    /// Se este processo esta elevado.
    ///
    /// Devolve erro, e nunca `false`, quando a consulta em si falha: tratar
    /// "nao sei" como "nao elevado" faria o ARCA se relancar, o filho falhar
    /// na mesma consulta e se relancar de novo — uma fila de prompts de UAC
    /// sem fim, cada pai preso esperando o filho.
    fn elevado(&self) -> Resultado<bool>;

    /// Relanca o ARCA com elevacao e devolve o codigo de saida do processo
    /// elevado, para propagacao.
    fn relancar_elevado(&self, argumentos: &[String]) -> Resultado<i32>;
}
