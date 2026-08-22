//! As implementacoes de verdade das portas.

pub mod arquivos_do_sistema;
pub mod console_do_usuario;
pub mod relogio_do_sistema;
pub mod windows;

pub use arquivos_do_sistema::ArquivosDoSistema;
pub use console_do_usuario::ConsoleDoUsuario;
pub use relogio_do_sistema::RelogioDoSistema;
