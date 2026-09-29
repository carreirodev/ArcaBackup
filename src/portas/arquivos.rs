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

/// O que o sistema de arquivos diz que ha num caminho. Ver
/// [`Arquivos::o_que_ha`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OQueHa {
    /// O sistema de arquivos respondeu que nada existe ali. "Nao consegui
    /// olhar" nunca chega aqui: e erro.
    Nada,
    Arquivo,
    Pasta,
}

pub trait Arquivos {
    /// Se ha alguma coisa neste caminho, arquivo ou pasta.
    ///
    /// Um `false` daqui nao separa "nao ha nada" de "nao consegui olhar": o
    /// adaptador responde com `Path::exists`, que transforma qualquer erro dos
    /// metadados em `false`. Para concluir que alguma coisa esta ausente, a
    /// pergunta e [`Arquivos::o_que_ha`] (WPC-68).
    fn existe(&self, caminho: &Path) -> bool;

    /// O que ha neste caminho: nada, um arquivo ou uma pasta. Quando o
    /// sistema de arquivos nao deixa saber, e erro, e nao [`OQueHa::Nada`].
    ///
    /// # Por que nao basta [`Arquivos::existe`]
    ///
    /// Por duas razoes, e cada uma ja fez o ARCA dizer "nao esta la" do que
    /// estava.
    ///
    /// `existe` responde "ha alguma coisa aqui", arquivo ou pasta. Medido em
    /// 28/09/2026 (WPC-64): uma pasta no `--iso` passava pelo `existe` do
    /// pre-voo do `arca prepare` e chegava ao `certutil`, que responde para
    /// ela o mesmo `0x80070002` de um arquivo ausente.
    ///
    /// E `existe` so tem `bool` para responder, e um `bool` nao diz "nao sei".
    /// Ate 29/09/2026 (WPC-68), um `--iso` ou um arquivo do `MD5SUMS` que o
    /// Windows nao deixava olhar saia como ausente.
    fn o_que_ha(&self, caminho: &Path) -> Resultado<OQueHa>;

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

    /// Copia um arquivo, sobrescrevendo o destino se ele existir.
    ///
    /// # Por que isto nao fura B-10
    ///
    /// B-10 diz que o ARCA nunca **apaga** nada, e sobrescrever nao e apagar —
    /// e a mesma distincao que o [ADR-0008] usou para o `estado.json` de um job
    /// colhido: *"ele fica no dispositivo ate o proximo `arca backup` gravar
    /// por cima, que e substituicao e nao exclusao"*.
    ///
    /// Os dois usos sao do `arca prepare` e os dois escrevem onde nada havia:
    /// o binario do ARCA no `ARCABOOT` e a copia do pacote no `ARCAVAULT`
    /// (PR-3), num disco que o proprio comando acabou de particionar.
    ///
    /// # Por que uma copia, e nao lê e escrever
    ///
    /// Porque [`Arquivos::escrever_atomico`] recebe `&str`, e um `.exe` e um
    /// `.zip` nao sao texto. Passar meio giga por uma `String` seria pior de
    /// todas as formas.
    ///
    /// [ADR-0008]: ../../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md
    fn copiar(&self, origem: &Path, destino: &Path) -> Resultado<()>;

    /// Espaco livre no volume que contem `caminho`.
    fn espaco_livre(&self, caminho: &Path) -> Resultado<u64>;
}
