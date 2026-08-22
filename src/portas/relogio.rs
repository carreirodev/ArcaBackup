//! A porta do relogio.
//!
//! # S-6 mora aqui
//!
//! O que este relogio produz serve para o registro local e para exibir data
//! ao usuario. **Nunca** para decidir se um desfecho pertence ao job
//! pendente: o Clonezilla lê o RTC — hora local do Windows — como se fosse
//! UTC e roda 3 h adiantado, de forma permanente. Quem liga um job ao seu
//! desfecho e o selo (C-11), nunca o tempo. Uma trava construida sobre
//! comparacao de datas ja reprovou um backup perfeito.

use chrono::{DateTime, Local};

pub trait Relogio {
    fn agora(&self) -> DateTime<Local>;
}
