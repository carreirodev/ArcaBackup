//! A porta do sistema de arquivos.
//!
//! Caminhos de arquivo, nunca dispositivos. A escrita atomica esta no
//! contrato porque o `estado.json` do `ARCABOOT` nao pode existir pela
//! metade: um desligamento no meio da gravacao deixaria um job pendente
//! ilegivel, e e justamente o job pendente que decide o que fazer na volta.
//!
//! # B-10 e uma propriedade destas assinaturas
//!
//! Nao ha metodo de exclusao aqui, e nao ha por descuido. O ARCA nunca apaga
//! nada (B-10) — nem imagem, nem residuo, nem log. Quem quisesse apagar
//! precisaria primeiro acrescentar o metodo, e `tests/b10_nada_e_apagado.rs`
//! cobra isso a cada build. Um residuo se apaga a mao, de proposito.

use crate::erro::Resultado;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    pub caminho: PathBuf,
    pub diretorio: bool,
    pub tamanho_bytes: u64,

    /// Quando o sistema de arquivos diz que a entrada mudou pela ultima vez.
    ///
    /// Serve para **exibir** a data de uma imagem, e nada mais. Uma imagem e
    /// escrita pelo Clonezilla, que lê o RTC como UTC e roda 3 h adiantado
    /// (P-7): esta data nunca decide se um desfecho pertence a um job — quem
    /// faz isso e o selo (S-6). `None` quando o sistema nao soube responder.
    pub modificado_em: Option<DateTime<Local>>,
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

    /// Lê um texto que **outro programa** escreveu, trocando por `U+FFFD` o
    /// que nao for UTF-8 valido.
    ///
    /// O `arca-check.log` e saida de terminal do Clonezilla: vem cheio de
    /// escapes ANSI e nada garante que cada byte forme UTF-8. Recusar o
    /// arquivo inteiro por causa de um byte solto esconderia o veredito, que
    /// e justamente o que diz se a imagem presta. Para o que o proprio ARCA
    /// escreve continua valendo [`Arquivos::ler_texto`], onde byte invalido e
    /// erro de verdade.
    fn ler_texto_alheio(&self, caminho: &Path) -> Resultado<String>;

    /// Escreve por arquivo temporario mais renomeacao: ou o conteudo antigo
    /// esta la, ou o novo, nunca um pedaco dos dois.
    fn escrever_atomico(&self, caminho: &Path, conteudo: &str) -> Resultado<()>;

    fn criar_diretorio(&self, caminho: &Path) -> Resultado<()>;
    fn listar(&self, caminho: &Path) -> Resultado<Vec<Entrada>>;

    /// Espaco livre no volume que contem `caminho`.
    fn espaco_livre(&self, caminho: &Path) -> Resultado<u64>;
}
