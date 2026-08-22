//! Os adaptadores do lado Windows.
//!
//! [`linha_de_comando`] e codigo puro sobre a convencao de argumentos do
//! Windows e roda em qualquer lugar; o resto conversa com a API do sistema.

pub mod linha_de_comando;

#[cfg(windows)]
pub mod console;
#[cfg(windows)]
pub mod entropia;
#[cfg(windows)]
pub mod firmware;
#[cfg(windows)]
pub mod privilegios;
#[cfg(windows)]
pub mod sistema;
#[cfg(windows)]
pub mod texto;
#[cfg(windows)]
pub mod volumes;
#[cfg(windows)]
pub mod wmi;
