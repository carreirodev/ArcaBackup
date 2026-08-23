//! ARCA — automatizador de Clonezilla para backup e restauracao de imagem de
//! disco.
//!
//! O ARCA nunca lê nem escreve disco. Ele prepara o ambiente, monta a receita,
//! dispara o boot unico e colhe o que o Clonezilla deixou escrito.
//!
//! O vocabulario deste codigo e o do `CONTEXT.md` na raiz do repositorio:
//! dispositivo, receita, job, armar, desarmar, selo, desfecho, veredito,
//! residuo. Onde o codigo diverge do glossario, e o codigo que esta errado.

pub mod adaptadores;
pub mod app;
pub mod armar;
pub mod blkdev;
pub mod cli;
pub mod comandos;
pub mod confirmacao;
pub mod desarme;
pub mod desfecho;
pub mod dispositivo;
pub mod duplos;
pub mod elevacao;
pub mod erro;
pub mod espaco;
pub mod estado;
pub mod firmware;
pub mod formato;
pub mod gpt;
pub mod grub;
pub mod imagens;
pub mod menuentry;
pub mod nome;
pub mod ordem;
pub mod portas;
pub mod prevoo;
pub mod receita;
pub mod registro;

pub use erro::{Erro, Resultado};
