//! A porta da enumeracao de discos.
//!
//! Le metadado, nunca conteudo. O dispositivo se acha pelos rotulos
//! `ARCABOOT` e `ARCAVAULT` — nunca por letra, `sda` ou numero de serie
//! (S-3) —, e a letra que aparece em [`Volume`] serve para montar caminho de
//! arquivo do lado Windows, jamais para enderecar destino de receita.

use crate::erro::Resultado;

/// Como o Windows classifica a midia. A distincao importa porque o `bcdedit`
/// **rejeita `Removable Media` em silencio** — responde "êxito" e mantem o
/// valor antigo (C-6). Um pendrive nunca serve de dispositivo ARCA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeMidia {
    /// `External hard disk media` — o que o `bcdedit` aceita.
    DiscoExterno,
    /// `Removable Media` — recusado por C-6.
    Removivel,
    DiscoFixo,
    Desconhecido,
}

/// Uma particao montada, vista do lado Windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// O rotulo — `ARCABOOT` ou `ARCAVAULT` num dispositivo ARCA.
    pub rotulo: Option<String>,
    /// A letra atribuida pelo Windows, quando ha uma.
    pub letra: Option<char>,
    pub sistema_de_arquivos: String,
    pub total_bytes: u64,
    pub livre_bytes: u64,
    pub tipo_de_midia: TipoDeMidia,
}

/// Um disco fisico, pelo que o Windows sabe dele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoFisico {
    pub modelo: String,
    pub tamanho_bytes: u64,
    /// Quanto dos volumes deste disco esta em uso — a base da regra de espaco
    /// de B-4.
    pub em_uso_bytes: u64,
}

pub trait Discos {
    fn volumes(&self) -> Resultado<Vec<Volume>>;
    fn discos_fisicos(&self) -> Resultado<Vec<DiscoFisico>>;
}
