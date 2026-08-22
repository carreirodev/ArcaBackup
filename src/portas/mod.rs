//! As fronteiras perigosas, cada uma atras de uma porta.
//!
//! As tres do primeiro dia sao o firmware (`bcdedit`), a enumeracao de discos
//! e o sistema de arquivos. Toda conversa do ARCA com o mundo passa por uma
//! delas. E o que permite que o parser do `bcdedit`, o validador da receita e
//! a regra de espaco tenham teste sem hardware, com os duplos de
//! [`crate::duplos`].
//!
//! A etapa E5 acrescentou [`entropia`], de onde sai o selo — pequena, e pela
//! mesma razao das outras: sem duplo, nenhum teste sobre o `estado.json`
//! saberia que selo esperar.
//!
//! A E6 acrescentou [`sistema`], para o que **nao e firmware** e mesmo assim
//! atravessa a fronteira: a Inicializacao Rapida (B-5) e o `chkdsk` (B-6).
//! Pendura-las na porta do firmware faria ela mentir sobre o que e; deixa-las
//! soltas num `Command::new` no meio de um comando tiraria o teste sem
//! hardware de B-5 e B-6.
//!
//! "As tres fronteiras perigosas" era uma descricao do que havia, e nao um
//! limite — o que a lista tem em comum e serem lugares onde o ARCA fala com o
//! mundo, e cada um so entra aqui quando alguma etapa precisa dele.
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
pub mod console;
pub mod discos;
pub mod entropia;
pub mod firmware;
pub mod privilegios;
pub mod relogio;
pub mod sistema;

pub use arquivos::{Arquivos, Entrada};
pub use console::Console;
pub use discos::{DiscoFisico, Discos, TipoDeMidia, Volume};
pub use entropia::Entropia;
pub use firmware::Firmware;
pub use privilegios::Privilegios;
pub use relogio::Relogio;
pub use sistema::{SaidaDeFerramenta, Sistema};
