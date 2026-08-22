//! As fronteiras perigosas, cada uma atras de uma porta.
//!
//! Sao tres: o firmware (`bcdedit`), a enumeracao de discos e o sistema de
//! arquivos. Toda conversa do ARCA com o mundo passa por uma delas. E o que
//! permite que o parser do `bcdedit`, o validador da receita e a regra de
//! espaco tenham teste sem hardware, com os duplos de [`crate::duplos`].
//!
//! # S-1 e uma propriedade destas assinaturas
//!
//! Nenhuma assinatura deste modulo entrega um handle de dispositivo, um
//! caminho de dispositivo bruto nem um deslocamento em setores. O que as
//! portas oferecem e metadado — rotulo, tamanho, modelo, espaco livre — e
//! conversa com ferramentas do proprio Windows, pelas quais o Windows
//! responde. Quem lê e escreve disco e o Clonezilla, do outro lado do
//! reinicio, e essa divisao e o que S-1 protege.
//!
//! Uma porta que precisasse abrir o disco em modo raw nao teria como ser
//! acrescentada aqui sem que a assinatura denunciasse. O teste de arquitetura
//! em `tests/s1_nenhum_acesso_raw.rs` cobra isso a cada build.

pub mod arquivos;
pub mod discos;
pub mod firmware;
pub mod privilegios;
pub mod relogio;

pub use arquivos::{Arquivos, Entrada};
pub use discos::{DiscoFisico, Discos, TipoDeMidia, Volume};
pub use firmware::Firmware;
pub use privilegios::Privilegios;
pub use relogio::Relogio;
