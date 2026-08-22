//! A porta do sistema de arquivos.
//!
//! Caminhos de arquivo, nunca dispositivos. A escrita atomica esta no
//! contrato porque o `estado.json` do `ARCABOOT` nao pode existir pela
//! metade: um desligamento no meio da gravacao deixaria um job pendente
//! ilegivel, e e justamente o job pendente que decide o que fazer na volta.

use crate::erro::Resultado;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    pub caminho: PathBuf,
    pub diretorio: bool,
    pub tamanho_bytes: u64,
}

impl Entrada {
    /// O nome da entrada, sem o caminho. Vazio so num caminho degenerado.
    pub fn nome(&self) -> String {
        self.caminho
            .file_name()
            .map(|nome| nome.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

pub trait Arquivos {
    fn existe(&self, caminho: &Path) -> bool;
    fn ler_texto(&self, caminho: &Path) -> Resultado<String>;

    /// Escreve por arquivo temporario mais renomeacao: ou o conteudo antigo
    /// esta la, ou o novo, nunca um pedaco dos dois.
    fn escrever_atomico(&self, caminho: &Path, conteudo: &str) -> Resultado<()>;

    fn criar_diretorio(&self, caminho: &Path) -> Resultado<()>;
    fn listar(&self, caminho: &Path) -> Resultado<Vec<Entrada>>;

    /// Espaco livre no volume que contem `caminho`.
    fn espaco_livre(&self, caminho: &Path) -> Resultado<u64>;
}
