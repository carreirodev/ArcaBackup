use crate::portas::Relogio;
use chrono::{DateTime, Local};

/// O relogio do Windows. Ver S-6 em [`crate::portas::relogio`].
#[derive(Debug, Clone, Copy, Default)]
pub struct RelogioDoSistema;

impl Relogio for RelogioDoSistema {
    fn agora(&self) -> DateTime<Local> {
        Local::now()
    }
}
