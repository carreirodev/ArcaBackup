//! A porta da enumeracao de discos, implementada sobre a API de volumes do
//! Windows.
//!
//! A etapa E1 preenche: achar o dispositivo pelos rotulos `ARCABOOT` e
//! `ARCAVAULT` (B-1, S-3) e recusar mais de um conectado (C-10). Ate la, a
//! porta existe para que a forma dos dados esteja fechada e os duplos possam
//! substitui-la.

use crate::erro::Resultado;
use crate::portas::{DiscoFisico, Discos, Volume};

#[derive(Debug, Clone, Copy, Default)]
pub struct VolumesDoWindows;

impl Discos for VolumesDoWindows {
    fn volumes(&self) -> Resultado<Vec<Volume>> {
        // E1.
        Ok(Vec::new())
    }

    fn discos_fisicos(&self) -> Resultado<Vec<DiscoFisico>> {
        // E1.
        Ok(Vec::new())
    }
}
