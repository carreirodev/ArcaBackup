//! Um modulo por comando da §8 do PRD.
//!
//! Cada um recebe o [`crate::app::Contexto`] com as portas e devolve o que
//! imprimir. A montagem da saida fica em funcao pura, separada da impressao,
//! porque a saida do `arca list` e criterio de aceite da etapa E1 — e
//! criterio de aceite merece teste que rode sem o dispositivo conectado.

pub mod backup;
pub mod desarmar;
pub mod list;
pub mod prepare;
pub mod restore;
pub mod resultado;
pub mod sondar;
pub mod status;
pub mod verify;
