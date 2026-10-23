//! O pacote do Clonezilla: versão fixada, SHA256 embutido e a conferência
//! (PR-1, PR-2, PR-3).
//!
//! # Por que a versão é fixada, e por que **esta**
//!
//! PR-1 manda fixar a versão e compilar o SHA256 no binário. A escolha da
//! versão não é gosto: **`3.3.3-15` é a que está bootando nesta mesa**. O
//! `grub.cfg` do dispositivo traz `hostname=cl-3.3.3-15` em cada `menuentry`, e
//! é sobre esse ambiente que rodaram o backup de 22/08, a restauração de 23/08
//! e a verificação da E11.
//!
//! Fixar a versão que já provou que funciona é o mesmo movimento do
//! [ADR-0014]: transcrever o que boota, em vez de instalar o que deveria
//! bootar.
//!
//! # O SHA256, e o que "baixado junto do arquivo não verifica nada" quer dizer
//!
//! Um checksum servido pelo mesmo servidor que serve o arquivo não prova coisa
//! alguma: quem pudesse trocar um trocaria o outro. Por isso ele é **constante
//! de código** — está aqui, no binário, e viaja com o ARCA.
//!
//! O número foi obtido em 23/08/2026 de **duas fontes independentes**, e as
//! duas dizem o mesmo:
//!
//! | Fonte | O que ela é |
//! |---|---|
//! | `https://free.nchc.org.tw/clonezilla-live/stable/CHECKSUMS.TXT` | O mirror do próprio projeto, em Taiwan, onde o Clonezilla é desenvolvido |
//! | O arquivo baixado do SourceForge, medido com `certutil -hashfile ... SHA256` | Outro servidor, outra rota |
//!
//! Servidores diferentes, o mesmo número. É o mais perto de verificação
//! independente que este caso admite, e está registrado em
//! `recursos/capturas/PROVENIENCIA.md`.
//!
//! # O zip, e não o ISO — e a diferença entre eles foi medida
//!
//! O Clonezilla publica os dois. O zip é o formato que se extrai direto numa
//! partição FAT32, que é exatamente o que `arca prepare` faz.
//!
//! **E o dispositivo desta mesa veio do ISO.** Medido em 23/08/2026: o
//! `boot/grub/grub.cfg` do zip e o `grub-clonezilla-original.cfg` do
//! dispositivo diferem em duas coisas, e só duas — o zip tem `noeject` em cada
//! um dos treze `menuentry`, e o carimbo de hora do rodapé difere em **seis
//! segundos** (`04:11:28` contra `04:11:22`). Seis segundos é o `ocs-live-dev`
//! gerando os dois artefatos na mesma execução: é a **mesma build**.
//!
//! Tirado o `noeject`, os dois arquivos são idênticos byte a byte.
//!
//! Isso não é motivo para preferir o ISO — é o contrário. `noeject` é o
//! parâmetro certo para mídia removível: ejetar um USB no desligamento é o
//! oposto do que se quer, e o ISO não o tem porque mídia óptica se ejeta
//! mesmo. O dispositivo desta mesa é que carrega o parâmetro de outra mídia.
//!
//! O que muda para o resto do ARCA: **nada que dependa do texto**. O estado
//! inerte se reconstrói do `grub.cfg` corrente ([ADR-0005]) e o bloco do ARCA
//! deriva do `live-toram` do próprio dispositivo ([ADR-0007]) — as duas
//! decisões existem justamente para o ARCA funcionar sobre o arquivo que
//! estiver lá. O `noeject` viaja junto, de graça.
//!
//! # E o zip entrega um `grub.cfg` que **não** está inerte
//!
//! O `set default` do pacote é `"0"`, e o [ADR-0005] tem uma seção sobre
//! exatamente isso: `"0"` aponta por **posição**, e a posição muda quando o
//! bloco do ARCA entra antes do `live-default`. Um dispositivo com `"0"` fica
//! armado no instante em que o bloco é inserido, sem que ninguém toque no `set
//! default` — *não é o estado inerte, é um estado que parece inerte*.
//!
//! Por isso `arca prepare` **desarma o que acabou de instalar**: extraído o
//! pacote, o `grub.cfg` passa por [`crate::grub::desarmar`] antes de o
//! dispositivo ser declarado pronto. Não é zelo — é o que faz o §4.4 valer
//! para um dispositivo recém-preparado.
//!
//! # O pacote guardado, e por que nenhum download vem depois do passo 5
//!
//! Ate 04/10/2026 o `arca prepare` baixava o pacote no passo 6, depois de o
//! passo 5 ter apagado o disco, sem limite para uma transferencia parada e
//! sem andamento na tela. Em 03/10 e 04/10/2026 ele ficou parado calado ate
//! o usuario cancelar, e cada tentativa custou o disco antes de falhar. E cada
//! preparo baixava 535,5 MB de novo, porque o passo 5 apaga tambem a copia de
//! PR-3.
//!
//! Desde entao vale PR-7: o pacote e obtido no pre-voo e mora no PC, na pasta
//! do registro, sob o nome de [`ARQUIVO`] — e so existe sob esse nome depois
//! de passar no SHA256. O download em andamento mora num `.parcial`, que se
//! retoma, e as [`FONTES`] sao tentadas em ordem. Ver o [ADR-0028].
//!
//! [ADR-0005]: ../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md
//! [ADR-0007]: ../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md
//! [ADR-0014]: ../docs/adr/0014-o-arca-particiona-o-dispositivo.md
//! [ADR-0028]: ../docs/adr/0028-nenhum-download-depois-do-ponto-sem-volta.md

use crate::andamento::Lentidao;
use crate::formato::tamanho;
use crate::portas::{Limites, Transferencia};
use crate::resumo::{Algoritmo, RecusaDoResumo, Resumo};
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A versão do Clonezilla que o ARCA instala.
///
/// É a que está no `hostname=cl-3.3.3-15` do `grub.cfg` deste dispositivo, e a
/// que rodou os três marcos em hardware deste projeto.
pub const VERSAO: &str = "3.3.3-15";

/// O nome do arquivo, como o projeto o publica.
pub const ARQUIVO: &str = "clonezilla-live-3.3.3-15-amd64.zip";

/// Uma fonte do pacote: o nome que a tela mostra e de onde baixar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fonte {
    /// O host, e a pasta quando o host serve o pacote de duas. E o que separa
    /// as duas fontes do mirror do projeto numa linha de tela.
    pub rotulo: &'static str,
    pub url: &'static str,
}

/// De onde baixar, na ordem em que se tenta (PR-1, PR-7).
///
/// # Tres, e nao uma
///
/// Com uma URL so, uma fonte parada voltava a ser um preparo perdido. E foi o
/// SourceForge que travou em 03/10 e 04/10/2026.
///
/// # A ordem, e o que a decidiu
///
/// O mirror do projeto vem primeiro: medido em 04/10/2026, 20 MB de cada
/// fonte, ele deu 1,5 MB/s e o SourceForge 0,9 MB/s. O mirror move a versao
/// de `stable/` para `oldstable/<versao>/` quando sai a seguinte — e a
/// `3.3.3-15` ja estava em `oldstable/` naquele dia —, por isso ele entra com
/// as duas pastas, e um 404 custa um segundo. O SourceForge resolve para um
/// mirror proximo de quem baixa, e o `curl` segue o redirecionamento com `-L`.
///
/// Nenhuma delas e confiavel, e por isso elas sao intercambiaveis: quem diz se
/// o que veio e o pacote e o [`SHA256`] compilado. Ao trocar a versao, as tres
/// mudam junto com [`VERSAO`], [`ARQUIVO`], [`SHA256`] e [`TAMANHO_BYTES`].
pub const FONTES: [Fonte; 3] = [
    Fonte {
        rotulo: "free.nchc.org.tw (oldstable)",
        url: "https://free.nchc.org.tw/clonezilla-live/oldstable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip",
    },
    Fonte {
        rotulo: "free.nchc.org.tw (stable)",
        url: "https://free.nchc.org.tw/clonezilla-live/stable/clonezilla-live-3.3.3-15-amd64.zip",
    },
    Fonte {
        rotulo: "downloads.sourceforge.net",
        url: "https://downloads.sourceforge.net/project/clonezilla/clonezilla_live_stable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip",
    },
];

/// Quanto uma fonte tem para conectar, e quanto ela pode ficar sem entregar
/// nada, antes de ser abandonada (PR-7).
///
/// Uma fonte de verdade entrega centenas de quilobytes por segundo; um
/// minuto sem um byte e uma fonte parada, e nao uma lenta. Os dois numeros
/// sao os do design de 04/10/2026, ajustaveis se um mirror lento os disparar
/// sem estar parado.
pub const LIMITES: Limites = Limites {
    conectar: Duration::from_secs(30),
    parado: Duration::from_secs(60),
};

/// Quanto de medida julga uma fonte lenta, e de quanto e a velocidade que a
/// tela mostra (PR-7, WPC-220).
///
/// # Por que a fonte lenta e julgada pelo que falta
///
/// Em 23/10/2026 o navegador baixava este pacote a 63,9 KB/s numa conexao de
/// 800 Mb, e o mirror do projeto, no mesmo dia, o entregava a 5,1 MB/s. O
/// limite de 60 s de [`LIMITES`] so larga uma fonte que nao manda um byte, e
/// uma fonte a 64 KB/s manda: o ARCA ficaria nela mais de duas horas. Uma
/// velocidade minima julgaria igual o comeco e o fim do pacote; o que falta
/// nao — perto do fim, uma fonte lenta termina.
///
/// Os dois numeros moram aqui, com os 30 s e os 60 s, e sao ajustaveis do
/// mesmo jeito. Ficam fora de [`Limites`] de proposito: quem julga e codigo
/// puro, em [`crate::andamento`], e nao o adaptador (ADR-0029).
pub const JANELA_DA_VELOCIDADE: Duration = Duration::from_secs(20);

/// Uma fonte da primeira volta que, na velocidade de
/// [`JANELA_DA_VELOCIDADE`], levaria mais do que isto para terminar perde a
/// vez para a seguinte.
pub const LENTA_SE_FALTAREM_MAIS_DE: Duration = Duration::from_secs(600);

/// O pacote guardado, dentro da pasta do ARCA no PC (PR-7).
pub fn guardado_em(pasta: &Path) -> PathBuf {
    pasta.join(ARQUIVO)
}

/// O download em andamento, ao lado do pacote guardado e com outro nome.
///
/// O nome e outro de proposito: um pacote so existe sob [`ARQUIVO`] depois de
/// passar no SHA256, e um download pela metade com o nome do pacote seria
/// tomado por um por quem olhasse a pasta.
pub fn parcial_em(pasta: &Path) -> PathBuf {
    pasta.join(format!("{ARQUIVO}.parcial"))
}

/// Por onde comecar um download, dado o `.parcial` que ja esta la.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comeco {
    /// Zerar o `.parcial` e baixar o pacote inteiro.
    DoZero,

    /// Continuar do byte em que o `.parcial` termina.
    Retomando(u64),
}

/// Retomar ou recomecar — e quem decide e o [`TAMANHO_BYTES`] compilado.
///
/// Menor que o pacote, o `.parcial` e um download interrompido, e se retoma.
/// Do tamanho do pacote ou maior, ele ja foi inteiro e nao bateu com o
/// SHA256, ou nao e o pacote: retomar pediria um byte que o pacote nao tem.
pub fn como_comecar(tamanho_do_parcial: Option<u64>) -> Comeco {
    match tamanho_do_parcial {
        Some(bytes) if bytes < TAMANHO_BYTES => Comeco::Retomando(bytes),
        _ => Comeco::DoZero,
    }
}

/// O codigo de saida do `curl` para o tempo esgotado.
///
/// Com `--connect-timeout` e sem `--speed-time` nem `--max-time`, a conexao e
/// a unica coisa que o `curl` cronometra — medido em 04/10/2026, inclusive
/// depois de um redirecionamento, quando o `http_code` vem `302`.
const CURL_TEMPO_ESGOTADO: i32 = 28;

/// O codigo de saida do `curl` para a escrita que falhou.
///
/// Medido em 04/10/2026, com o `.parcial` somente-leitura: com bytes para
/// retomar, o `curl` abre o destino **antes de conectar** e sai com 23 sem ter
/// pedido nada a fonte; vazio, ele conecta e sai com 23 no primeiro byte.
const CURL_NAO_ESCREVEU: i32 = 23;

/// O que uma fonte respondeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resposta {
    Entregou,
    NaoRespondeu,
    Parou {
        recebido: u64,
    },
    Http(u16),

    /// A fonte perdeu a vez na primeira volta: o ARCA pediu ao adaptador para
    /// encerrar o `curl`, e o `.parcial` ficou com o que veio. E a unica
    /// resposta que leva a fonte a segunda volta (WPC-220).
    Lenta(Lentidao),

    /// O `curl` nao conseguiu escrever no `.parcial`. Nao e uma resposta da
    /// fonte, e por isso nenhuma outra fonte e tentada: todas cairiam no
    /// mesmo arquivo (WPC-134, decidido pelo usuario em 04/10/2026).
    NaoConseguiuEscrever(String),

    /// O texto do `curl`, para qualquer outra falha: sem rede, nome que nao
    /// resolve, servidor que nao aceita retomada.
    Outra(String),
}

impl fmt::Display for Resposta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resposta::Entregou => f.write_str("entregou"),
            Resposta::NaoRespondeu => {
                write!(f, "nao respondeu em {} s", LIMITES.conectar.as_secs())
            }
            Resposta::Parou { recebido } => write!(
                f,
                "parou de mandar por {} s, com {} de {}",
                LIMITES.parado.as_secs(),
                tamanho(*recebido),
                tamanho(TAMANHO_BYTES)
            ),
            Resposta::Http(codigo) => write!(f, "HTTP {codigo}"),
            Resposta::Lenta(lentidao) => write!(f, "{lentidao}"),
            Resposta::NaoConseguiuEscrever(texto) | Resposta::Outra(texto) => f.write_str(texto),
        }
    }
}

/// O que a fonte respondeu, julgado pelo codigo e nunca pela frase.
///
/// A frase do `curl` so aparece quando nenhum codigo diz o que houve, e
/// aparece inteira: e o que quem lê pode procurar.
pub fn o_que_a_fonte_respondeu(transferencia: &Transferencia) -> Resposta {
    if transferencia.parou {
        return Resposta::Parou {
            recebido: transferencia.tamanho_no_destino,
        };
    }
    if transferencia.codigo == 0 {
        return Resposta::Entregou;
    }
    if transferencia.codigo == CURL_TEMPO_ESGOTADO {
        return Resposta::NaoRespondeu;
    }
    if let Some(codigo) = transferencia.http.filter(|codigo| *codigo >= 400) {
        return Resposta::Http(codigo);
    }
    if transferencia.codigo == CURL_NAO_ESCREVEU {
        return Resposta::NaoConseguiuEscrever(frase_do_curl(transferencia));
    }
    Resposta::Outra(frase_do_curl(transferencia))
}

/// A frase do `curl` sobre a falha: a linha `curl: (N) ...`, que e a que diz
/// o que houve.
///
/// A ultima, e nao a primeira: medido em 04/10/2026, com o destino
/// somente-leitura o `curl` escreve antes dela o caminho do arquivo quebrado em
/// tres linhas, e a primeira e `curl: cannot open`.
fn frase_do_curl(transferencia: &Transferencia) -> String {
    let linhas: Vec<&str> = transferencia
        .texto
        .lines()
        .map(str::trim)
        .filter(|linha| !linha.is_empty())
        .collect();

    linhas
        .iter()
        .rev()
        .find(|linha| linha.starts_with("curl: ("))
        .or(linhas.first())
        .map(|frase| frase.to_string())
        .unwrap_or_else(|| format!("o curl saiu com codigo {}", transferencia.codigo))
}

/// O SHA256 esperado, **compilado no binário** (PR-1).
///
/// Publicado pelo mirror do projeto em `CHECKSUMS.TXT` e conferido contra o
/// arquivo baixado do SourceForge em 23/08/2026 — duas fontes, o mesmo número.
/// Ver o cabeçalho deste módulo.
pub const SHA256: &str = "00cee7700433e63017e2ea9eb40519108829710132364a8028a6c039a6046304";

/// O tamanho do arquivo, medido no download de 23/08/2026.
///
/// Não é uma segunda verificação — o SHA256 já responde tudo o que o tamanho
/// responderia e mais. Serve para a tela poder dizer quanto vai baixar antes de
/// começar, num download de meio giga que leva minutos.
pub const TAMANHO_BYTES: u64 = 561_478_648;

/// Os quatro caminhos que o pacote precisa ter dentro, e que o `arca prepare`
/// confere depois de extrair.
///
/// São o que o §4 do PRD descreve como conteúdo do `ARCABOOT`. Conferi-los é o
/// que separa "o `bsdtar` saiu com zero" de "o dispositivo tem o que boota" —
/// e a distinção já custou caro neste projeto três vezes.
pub const CAMINHOS_OBRIGATORIOS: [&str; 4] = [
    "EFI/boot/bootx64.efi",
    "live/vmlinuz",
    "live/initrd.img",
    "boot/grub/grub.cfg",
];

/// Por que o pacote não serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoPacote {
    /// PR-2: o `--iso` nomeia um arquivo que não está lá.
    ///
    /// # Por que este caso tem nome próprio, e o que a falta dele custava
    ///
    /// Medido em 27/08/2026: um caminho errado seguia até o `certutil`, que
    /// responde `0x80070002 (WIN32: 2 ERROR_FILE_NOT_FOUND)`. O que chegava a
    /// quem digitou o caminho era o código de erro de uma ferramenta do Windows
    /// dentro de duas camadas de recusa do ARCA — uma frase sobre conferência
    /// de resumo para um problema que é de caminho. Ninguém lê `0x80070002` e
    /// pensa *"errei a pasta"*.
    ///
    /// E ilegível era o menor dos dois defeitos: a recusa só nascia no passo
    /// 7, **depois** do ponto sem volta. Cada tentativa com o caminho errado
    /// custava o disco inteiro, e a tentativa seguinte custava outro.
    ///
    /// # Por que a mensagem pode prometer que nada foi apagado
    ///
    /// Porque esta variante nasce em **um lugar só** —
    /// `crate::comandos::prepare::conferir_o_pacote_local`, que roda antes
    /// do passo 0 —, e é isso que autoriza a promessa. A conferência que
    /// acontece de novo no passo 7 não a levanta: lá o disco já foi apagado, e
    /// a frase seria mentira.
    NaoEstaLa { caminho: PathBuf },

    /// PR-2: o `--iso` nomeia uma pasta, e não o arquivo.
    ///
    /// # Por que a pasta não cai em `NaoEstaLa`
    ///
    /// Porque a pasta **está** lá. Medido em 28/09/2026 (WPC-64): ela passava
    /// pela pergunta de `NaoEstaLa` e chegava ao `certutil`, que responde para
    /// uma pasta exatamente o que responde para um arquivo ausente —
    /// `0x80070002 (WIN32: 2 ERROR_FILE_NOT_FOUND)`. Passar a pasta de
    /// downloads no lugar do arquivo é o jeito mais provável de cometer o erro
    /// que `NaoEstaLa` descreve, e era o único que ela não pegava.
    ///
    /// # Por que ela aponta o pacote, e não segue com ele
    ///
    /// `dentro` é o caminho do pacote, quando ele está dentro da pasta, e a
    /// mensagem o dá inteiro para passar. O ARCA não o usa sozinho: o `--iso`
    /// nomeia o arquivo (PR-2), e seguir com o de dentro seria deduzir o
    /// arquivo de um caminho que nomeia outra coisa — o `prepare` oferece e
    /// não deduz ([ADR-0024]).
    ///
    /// # Por que a mensagem pode prometer que nada foi apagado
    ///
    /// Pelo mesmo motivo de `NaoEstaLa`: esta variante nasce só no pré-voo do
    /// `arca prepare`, que roda antes do passo 0. A conferência do passo 7 não
    /// pergunta por pasta: lá o disco já foi apagado, e a frase seria mentira.
    ///
    /// [ADR-0024]: ../docs/adr/0024-o-prepare-oferece-a-lista-e-nao-deduz-o-disco.md
    EUmaPasta {
        caminho: PathBuf,
        dentro: Option<PathBuf>,
    },

    /// PR-2: o Windows não deixou saber se o arquivo do `--iso` está lá.
    ///
    /// # Por que isto não é `NaoEstaLa`
    ///
    /// Porque "não consegui olhar" nunca vira "não há nada lá". Até 29/09/2026
    /// (WPC-68) o pré-voo perguntava com `Path::exists`, que transforma
    /// qualquer erro dos metadados em `false`, e um caminho que o Windows não
    /// deixava olhar saía como `NaoEstaLa`: o ARCA dizia que procurou e não
    /// achou nada.
    ///
    /// Vale também para o pacote de dentro de uma pasta. `caminho` é então o
    /// do zip de dentro, e a recusa deixa de dizer que ele não está na pasta,
    /// que é o que `EUmaPasta` sem `dentro` diria.
    ///
    /// `motivo` é o que o Windows respondeu, e o texto foi aprovado pelo
    /// usuário em 29/09/2026, no mesmo formato da linha `NAO DEU PARA LER` do
    /// `arca verify`.
    ///
    /// # Por que a mensagem pode prometer que nada foi apagado
    ///
    /// Pelo mesmo motivo de `NaoEstaLa`: esta variante nasce só no pré-voo do
    /// `arca prepare`, que roda antes do passo 0.
    NaoDeuParaOlhar { caminho: PathBuf, motivo: String },

    /// O `certutil` não resumiu o arquivo.
    NaoDeuParaResumir(RecusaDoResumo),

    /// O resumo saiu e **não é** o esperado. É o caso que PR-1 existe para
    /// pegar.
    ResumoDivergente { esperado: String, veio: String },

    /// Extraído, o pacote não tem o que faz um dispositivo bootar.
    PacoteIncompleto { faltando: Vec<String> },

    /// PR-7: nenhuma das [`FONTES`] entregou o pacote.
    ///
    /// `tentativas` e cada fonte, uma vez e na ordem de [`FONTES`], com o que
    /// ela respondeu na ultima vez em que foi tentada. Ate a WPC-220 era cada
    /// tentativa, e as duas coisas eram a mesma: cada fonte ia uma vez. Desde
    /// a segunda volta, uma fonte lenta vai duas, e a recusa diz o que ela
    /// respondeu por ultimo. O texto foi aprovado pelo usuario em 04/10/2026
    /// (WPC-134), e termina no caminho que nao depende do download do ARCA: o
    /// navegador e o `--iso`.
    ///
    /// # Por que a mensagem pode prometer que nada foi apagado
    ///
    /// Porque o pacote e obtido no pre-voo do `arca prepare`, antes do passo
    /// 0, e nenhum download acontece depois dele.
    NenhumaFonteEntregou {
        tentativas: Vec<(&'static str, Resposta)>,
    },

    /// PR-7: a pasta do pacote guardado nao aceita escrita — ela nao se deixa
    /// criar, ou o `.parcial` nao se deixa criar nem zerar. `motivo` e o que o
    /// Windows respondeu.
    ///
    /// Nasce no pre-voo, antes de qualquer download, e por isso pode prometer
    /// que nada foi apagado.
    NaoDeuParaGuardar { pasta: PathBuf, motivo: String },

    /// PR-7: o volume da pasta do pacote guardado tem menos espaco livre do
    /// que falta baixar. Nasce no pre-voo, antes de qualquer download.
    FaltaEspaco {
        pasta: PathBuf,
        falta_bytes: u64,
        livre_bytes: u64,
    },
}

/// Um megabyte, na base 1024 de [`crate::formato::tamanho`].
pub(crate) const MEGABYTE: u64 = 1024 * 1024;

impl fmt::Display for RecusaDoPacote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoPacote::NaoEstaLa { caminho } => write!(
                f,
                "o arquivo `{}` nao esta la, e e o caminho que o `--iso` deu. O `--iso` nomeia o arquivo, e nao a pasta onde ele esta: ele termina em `{ARQUIVO}`. Um caminho relativo vale a partir da pasta de onde o comando foi digitado — passe o caminho completo, entre aspas se ele tiver espaco. Nada foi apagado (PR-2)",
                caminho.display()
            ),
            RecusaDoPacote::EUmaPasta {
                caminho,
                dentro: Some(pacote),
            } => write!(
                f,
                "`{}` e uma pasta, e o `--iso` nomeia o arquivo. O pacote esta dentro dela — passe o caminho inteiro, entre aspas: \"{}\". Nada foi apagado (PR-2)",
                caminho.display(),
                pacote.display()
            ),
            RecusaDoPacote::EUmaPasta {
                caminho,
                dentro: None,
            } => write!(
                f,
                "`{}` e uma pasta, e o `--iso` nomeia o arquivo, que termina em `{ARQUIVO}`. Esse arquivo nao esta nesta pasta. Nada foi apagado (PR-2)",
                caminho.display()
            ),
            RecusaDoPacote::NaoDeuParaOlhar { caminho, motivo } => write!(
                f,
                "nao deu para saber se `{}` esta la ({motivo}). Isto NAO e o mesmo que ele nao estar la. Nada foi apagado (PR-2)",
                caminho.display()
            ),
            RecusaDoPacote::NaoDeuParaResumir(porque) => write!(
                f,
                "nao deu para conferir o SHA256 do pacote do Clonezilla: {porque}. Sem essa conferencia o ARCA nao instala nada — um pacote que nao se conferiu e um pacote que nao se sabe o que e (PR-1)"
            ),
            RecusaDoPacote::ResumoDivergente { esperado, veio } => write!(
                f,
                "o SHA256 do pacote NAO bate. Esperado `{esperado}`, veio `{veio}`. O ARCA para aqui e nao extrai nada. O numero esperado esta compilado neste binario e nao veio junto do download — e por isso que ele vale alguma coisa (PR-1). Ou o download veio corrompido, ou o arquivo do outro lado nao e o que este ARCA conhece"
            ),
            RecusaDoPacote::PacoteIncompleto { faltando } => write!(
                f,
                "o pacote foi extraido e nao tem o que faz um dispositivo bootar: falta {}. O `bsdtar` ter saido com codigo zero nao e prova de que o conteudo esta la",
                faltando.join(", ")
            ),
            RecusaDoPacote::NenhumaFonteEntregou { tentativas } => {
                writeln!(
                    f,
                    "nao deu para baixar o Clonezilla {VERSAO} de nenhuma das {} fontes:",
                    tentativas.len()
                )?;
                for (rotulo, resposta) in tentativas {
                    writeln!(f, "  {rotulo} — {resposta}")?;
                }
                write!(
                    f,
                    "O que ja veio fica guardado, e a proxima tentativa continua de onde parou. Para nao depender do download do ARCA, baixe pelo navegador\n  {}\ne passe o arquivo com --iso. Nada foi apagado (PR-7)",
                    FONTES[0].url
                )
            }
            RecusaDoPacote::NaoDeuParaGuardar { pasta, motivo } => write!(
                f,
                "nao deu para guardar o pacote em {} ({motivo}). Nada foi apagado (PR-7)",
                pasta.display()
            ),
            // O que falta arredonda para cima e o que ha livre para baixo: uma
            // recusa que dissesse "faltam 536 MB, e ha 536 MB livres" seria
            // ilegivel.
            RecusaDoPacote::FaltaEspaco {
                pasta,
                falta_bytes,
                livre_bytes,
            } => write!(
                f,
                "faltam {} MB para baixar o pacote, e {} tem {} MB livres. Nada foi apagado (PR-7)",
                falta_bytes.div_ceil(MEGABYTE),
                pasta.display(),
                livre_bytes / MEGABYTE
            ),
        }
    }
}

/// Confere o resumo medido contra o que está compilado aqui (PR-1).
///
/// # A ordem, e ela não é estética
///
/// A recusa de **não ter conseguido medir** vem antes da de **não bater**,
/// porque as duas pedem coisas diferentes de quem lê: uma manda olhar o
/// `certutil`, a outra manda desconfiar do arquivo. É a mesma distinção que a
/// E5 pagou caro para existir — *"não consegui olhar" nunca vira "não há nada
/// lá"*.
pub fn conferir_o_resumo(medido: Result<Resumo, RecusaDoResumo>) -> Result<Resumo, RecusaDoPacote> {
    let medido = medido.map_err(RecusaDoPacote::NaoDeuParaResumir)?;
    let esperado = Resumo::novo(Algoritmo::Sha256, SHA256)
        .expect("a constante SHA256 deste modulo tem de ser um resumo valido");

    if medido != esperado {
        return Err(RecusaDoPacote::ResumoDivergente {
            esperado: esperado.como_texto().to_string(),
            veio: medido.como_texto().to_string(),
        });
    }
    Ok(medido)
}

/// Quais dos caminhos obrigatórios não estão na lista extraída.
///
/// A comparação normaliza `\` para `/` porque quem lista é o `bsdtar`, que usa
/// `/`, e quem confere é o Windows, que usa `\` — e trocar os dois é a fonte de
/// um "não achei" que não quer dizer nada.
pub fn o_que_falta<'a>(presentes: impl Iterator<Item = &'a str>) -> Vec<String> {
    let presentes: Vec<String> = presentes
        .map(|caminho| caminho.replace('\\', "/").trim_matches('/').to_lowercase())
        .collect();

    CAMINHOS_OBRIGATORIOS
        .iter()
        .filter(|obrigatorio| {
            let procurado = obrigatorio.to_lowercase();
            !presentes.contains(&procurado)
        })
        .map(|obrigatorio| obrigatorio.to_string())
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::portas::SaidaDeFerramenta;
    use crate::resumo::do_certutil;

    /// A resposta do `certutil` sobre o pacote baixado em 23/08/2026.
    fn certutil_do_pacote() -> SaidaDeFerramenta {
        SaidaDeFerramenta {
            codigo: 0,
            texto: format!(
                "SHA256 hash de clonezilla-live-3.3.3-15-amd64.zip:\r\n{SHA256}\r\nCertUtil: -hashfile : comando concluido com exito.\r\n"
            ),
        }
    }

    #[test]
    fn a_constante_e_um_sha256_de_verdade() {
        // Uma constante mal digitada — 63 dígitos, um `g` no meio — só
        // apareceria na hora de conferir um download de meio giga. Aqui ela
        // aparece a cada build.
        let resumo = Resumo::novo(Algoritmo::Sha256, SHA256).expect("o SHA256 embutido e valido");
        assert_eq!(resumo.como_texto().chars().count(), 64);
        assert_eq!(resumo.algoritmo(), Algoritmo::Sha256);
    }

    #[test]
    fn a_versao_e_a_que_esta_bootando_nesta_mesa() {
        // O oráculo é o `grub.cfg` capturado do dispositivo: ele traz
        // `hostname=cl-3.3.3-15` em cada `menuentry`, e é sobre esse ambiente
        // que rodaram os três marcos em hardware deste projeto.
        //
        // Fixar outra versão passaria a instalar um Clonezilla que nada neste
        // repositório viu funcionar — e este teste falaria.
        const GRUB: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");
        assert!(
            GRUB.contains(&format!("hostname=cl-{VERSAO}")),
            "a versao fixada nao e a que o dispositivo desta mesa roda"
        );

        // E o nome do arquivo e as fontes carregam a mesma versao. Trocar a
        // constante e esquecer de trocar uma fonte baixaria outra coisa.
        assert!(ARQUIVO.contains(VERSAO), "o nome do arquivo: {ARQUIVO}");
        for fonte in FONTES {
            assert!(fonte.url.contains(VERSAO), "a URL: {}", fonte.url);
            assert!(
                fonte.url.ends_with(ARQUIVO),
                "a URL nao termina no arquivo: {}",
                fonte.url
            );
        }
    }

    // ─────────────── as fontes e o pacote guardado (PR-7) ───────────────

    #[test]
    fn as_fontes_sao_as_tres_na_ordem_e_carregam_a_versao() {
        // O oraculo e o criterio 20 da WPC-134, conferido nos dois servidores
        // em 04/10/2026: a `3.3.3-15` estava em `oldstable/3.3.3-15/` no
        // mirror do projeto e em `clonezilla_live_stable/3.3.3-15/` no
        // SourceForge, com o mesmo SHA256 e o mesmo tamanho.
        let esperadas = [
            (
                "free.nchc.org.tw (oldstable)",
                "https://free.nchc.org.tw/clonezilla-live/oldstable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip",
            ),
            (
                "free.nchc.org.tw (stable)",
                "https://free.nchc.org.tw/clonezilla-live/stable/clonezilla-live-3.3.3-15-amd64.zip",
            ),
            (
                "downloads.sourceforge.net",
                "https://downloads.sourceforge.net/project/clonezilla/clonezilla_live_stable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip",
            ),
        ];

        let tem: Vec<(&str, &str)> = FONTES.iter().map(|f| (f.rotulo, f.url)).collect();
        assert_eq!(tem, esperadas);

        for fonte in FONTES {
            assert!(fonte.url.contains(VERSAO), "sem a versao: {}", fonte.url);
            assert!(fonte.url.ends_with(ARQUIVO), "sem o arquivo: {}", fonte.url);
        }
    }

    #[test]
    fn os_limites_de_pr_7_continuam_30_e_60_s() {
        // O criterio 17 da WPC-220: a fonte lenta entra ao lado dos dois
        // limites de 04/10/2026, e nao no lugar deles.
        assert_eq!(
            LIMITES,
            Limites {
                conectar: Duration::from_secs(30),
                parado: Duration::from_secs(60),
            }
        );
        assert_eq!(JANELA_DA_VELOCIDADE, Duration::from_secs(20));
        assert_eq!(LENTA_SE_FALTAREM_MAIS_DE, Duration::from_secs(600));
    }

    #[test]
    fn o_tamanho_compilado_decide_entre_retomar_e_recomecar() {
        // O espaco inteiro de entrada, com as duas bordas dos dois lados.
        for (tamanho_do_parcial, esperado) in [
            (None, Comeco::DoZero),
            (Some(0), Comeco::Retomando(0)),
            (Some(561_478_647), Comeco::Retomando(561_478_647)),
            (Some(561_478_648), Comeco::DoZero),
            (Some(561_478_649), Comeco::DoZero),
        ] {
            assert_eq!(
                como_comecar(tamanho_do_parcial),
                esperado,
                "{tamanho_do_parcial:?}"
            );
        }
    }

    /// Uma transferencia de mentira, com o que o adaptador devolveria.
    fn transferencia(codigo: i32, http: Option<u16>, texto: &str) -> Transferencia {
        Transferencia {
            codigo,
            texto: texto.to_string(),
            http,
            parou: false,
            encerrada: false,
            tamanho_no_destino: 0,
        }
    }

    #[test]
    fn o_que_cada_fonte_respondeu_e_julgado_pelo_codigo() {
        // Os codigos e os `http_code` sao os medidos em 04/10/2026, com o
        // `curl` 8.21.0 do `System32` contra servidores locais. As frases
        // vieram junto, e nenhuma decide nada: estao aqui para provar isso.
        let casos = [
            (
                transferencia(28, None, "curl: (28) Connection timed out after 3001 milliseconds"),
                Resposta::NaoRespondeu,
                "nao respondeu em 30 s",
            ),
            (
                // O redirecionamento respondeu, e o destino dele nao conectou:
                // o `http_code` e o do `302`.
                transferencia(28, Some(302), "curl: (28) Connection timed out after 3000 milliseconds"),
                Resposta::NaoRespondeu,
                "nao respondeu em 30 s",
            ),
            (
                transferencia(22, Some(404), "curl: (22) The requested URL returned error: 404"),
                Resposta::Http(404),
                "HTTP 404",
            ),
            (
                transferencia(
                    7,
                    None,
                    "curl: (7) Failed to connect to 127.0.0.1:9 after 2014 ms: Could not connect to server\r\n",
                ),
                Resposta::Outra(
                    "curl: (7) Failed to connect to 127.0.0.1:9 after 2014 ms: Could not connect to server"
                        .to_string(),
                ),
                "curl: (7) Failed to connect to 127.0.0.1:9 after 2014 ms: Could not connect to server",
            ),
            (transferencia(0, Some(206), ""), Resposta::Entregou, "entregou"),
        ];

        for (transferencia, resposta, dita) in casos {
            assert_eq!(o_que_a_fonte_respondeu(&transferencia), resposta);
            assert_eq!(resposta.to_string(), dita);
        }

        // A escrita que falhou, com o stderr medido em 04/10/2026 contra um
        // `.parcial` somente-leitura: o caminho vem quebrado em tres linhas
        // antes da frase que importa, que e a ultima.
        let nao_escreveu = transferencia(
            23,
            None,
            "curl: cannot open \r\ncurl: 'C:\\Users\\Eduardo\\AppData\\Local\\Temp\\arca-medir-curl-3\\somente-leitura-10\r\ncurl: 0.parcial'\r\ncurl: (23) Failed writing received data to disk/application\r\n",
        );
        assert_eq!(
            o_que_a_fonte_respondeu(&nao_escreveu),
            Resposta::NaoConseguiuEscrever(
                "curl: (23) Failed writing received data to disk/application".to_string()
            )
        );

        // Parada e do adaptador, e vale mais do que o codigo: quem encerrou o
        // `curl` foi ele.
        let parada = Transferencia {
            parou: true,
            encerrada: false,
            tamanho_no_destino: 125_829_120,
            ..transferencia(1, Some(200), "")
        };
        let resposta = o_que_a_fonte_respondeu(&parada);
        assert_eq!(
            resposta,
            Resposta::Parou {
                recebido: 125_829_120
            }
        );
        assert_eq!(
            resposta.to_string(),
            "parou de mandar por 60 s, com 120,0 MB de 535,5 MB"
        );
    }

    #[test]
    fn a_recusa_de_nenhuma_fonte_e_o_texto_aprovado() {
        // O texto do criterio 6 da WPC-134, aprovado em 04/10/2026, byte a
        // byte.
        let recusa = RecusaDoPacote::NenhumaFonteEntregou {
            tentativas: vec![
                (FONTES[0].rotulo, Resposta::NaoRespondeu),
                (
                    FONTES[1].rotulo,
                    Resposta::Parou {
                        recebido: 125_829_120,
                    },
                ),
                (FONTES[2].rotulo, Resposta::Http(404)),
            ],
        };

        assert_eq!(
            recusa.to_string(),
            "nao deu para baixar o Clonezilla 3.3.3-15 de nenhuma das 3 fontes:\n  \
             free.nchc.org.tw (oldstable) — nao respondeu em 30 s\n  \
             free.nchc.org.tw (stable) — parou de mandar por 60 s, com 120,0 MB de 535,5 MB\n  \
             downloads.sourceforge.net — HTTP 404\n\
             O que ja veio fica guardado, e a proxima tentativa continua de onde parou. Para nao depender do download do ARCA, baixe pelo navegador\n  \
             https://free.nchc.org.tw/clonezilla-live/oldstable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip\n\
             e passe o arquivo com --iso. Nada foi apagado (PR-7)"
        );
    }

    #[test]
    fn as_recusas_de_onde_guardar_sao_o_texto_aprovado() {
        // Os dois textos do criterio 8 da WPC-134, aprovados em 04/10/2026.
        let pasta = PathBuf::from(r"C:\Users\Ana Paula\AppData\Local\ARCA");

        assert_eq!(
            RecusaDoPacote::NaoDeuParaGuardar {
                pasta: pasta.clone(),
                motivo: "Acesso negado. (os error 5)".to_string(),
            }
            .to_string(),
            r"nao deu para guardar o pacote em C:\Users\Ana Paula\AppData\Local\ARCA (Acesso negado. (os error 5)). Nada foi apagado (PR-7)"
        );

        // 561.478.648 bytes sao 535,47 MB: faltam 536, e nao 535.
        assert_eq!(
            RecusaDoPacote::FaltaEspaco {
                pasta,
                falta_bytes: TAMANHO_BYTES,
                livre_bytes: 100 * MEGABYTE,
            }
            .to_string(),
            r"faltam 536 MB para baixar o pacote, e C:\Users\Ana Paula\AppData\Local\ARCA tem 100 MB livres. Nada foi apagado (PR-7)"
        );
    }

    #[test]
    fn o_parcial_mora_ao_lado_do_guardado_com_outro_nome() {
        let pasta = Path::new(r"C:\Users\Ana Paula\AppData\Local\ARCA");
        assert_eq!(
            guardado_em(pasta),
            PathBuf::from(
                r"C:\Users\Ana Paula\AppData\Local\ARCA\clonezilla-live-3.3.3-15-amd64.zip"
            )
        );
        assert_eq!(
            parcial_em(pasta),
            PathBuf::from(
                r"C:\Users\Ana Paula\AppData\Local\ARCA\clonezilla-live-3.3.3-15-amd64.zip.parcial"
            )
        );
    }

    #[test]
    fn o_pacote_medido_em_23_08_passa_na_conferencia() {
        // O oráculo desta etapa: esta resposta é a do `certutil` sobre o
        // arquivo que veio do SourceForge, e a constante veio do
        // `CHECKSUMS.TXT` do mirror do projeto. Dois servidores, o mesmo
        // número — e o teste não pode ser ajustado para passar sem trocar os
        // dois lados.
        let resumo = conferir_o_resumo(do_certutil(&certutil_do_pacote(), Algoritmo::Sha256))
            .expect("o pacote medido tem de passar");
        assert_eq!(resumo.como_texto(), SHA256);
    }

    #[test]
    fn um_bit_trocado_no_pacote_e_recusa() {
        // O caso que PR-1 existe para pegar. **O último dígito muda, e nada
        // mais** — um pacote trocado no meio do caminho não anuncia isso de
        // outra forma.
        let ultimo_trocado = format!(
            "{}{}",
            &SHA256[..SHA256.len() - 1],
            if SHA256.ends_with('4') { '5' } else { '4' }
        );
        assert_ne!(ultimo_trocado, SHA256, "o teste tem de trocar alguma coisa");

        let mut torto = certutil_do_pacote();
        torto.texto = torto.texto.replace(SHA256, &ultimo_trocado);

        let recusa = conferir_o_resumo(do_certutil(&torto, Algoritmo::Sha256)).unwrap_err();
        assert!(matches!(recusa, RecusaDoPacote::ResumoDivergente { .. }));

        // E a mensagem diz por que o numero esperado vale alguma coisa: ele
        // nao veio junto do download.
        assert!(
            recusa.to_string().contains("nao veio junto do download"),
            "{recusa}"
        );
    }

    #[test]
    fn o_caminho_que_nao_existe_e_recusa_de_caminho_e_nao_de_certutil() {
        // A mensagem tem de falar do que quem digitou pode consertar. O
        // `0x80070002` do `certutil` diz a verdade e nao ajuda ninguem: o
        // problema e o caminho, e a frase e sobre o caminho.
        let recusa = RecusaDoPacote::NaoEstaLa {
            caminho: PathBuf::from(r"C:\Users\Ana Paula\Downloads\clonezilla.zip"),
        };
        let dito = recusa.to_string();

        assert!(
            dito.contains(r"C:\Users\Ana Paula\Downloads\clonezilla.zip"),
            "{dito}"
        );
        assert!(dito.contains(ARQUIVO), "o nome do arquivo esperado: {dito}");
        assert!(dito.contains("Nada foi apagado"), "{dito}");
    }

    #[test]
    fn nao_ter_conseguido_medir_e_outra_recusa_e_nao_a_de_divergencia() {
        // "Nao consegui olhar" nunca vira "nao bate". As duas pedem coisas
        // diferentes de quem lê: uma manda olhar o `certutil`, a outra manda
        // desconfiar do arquivo.
        let falhou = SaidaDeFerramenta {
            codigo: -2147024894,
            texto: "CertUtil: -hashfile comando FALHOU: 0x80070002\r\n".to_string(),
        };

        assert!(matches!(
            conferir_o_resumo(do_certutil(&falhou, Algoritmo::Sha256)),
            Err(RecusaDoPacote::NaoDeuParaResumir(_))
        ));
    }

    #[test]
    fn um_md5_no_lugar_do_sha256_nao_passa_por_acidente() {
        // Os dois sao hexadecimais e so o comprimento os separa. Pedir SHA256
        // e receber 32 digitos e a resposta errada, e nao uma resposta curta.
        let md5 = SaidaDeFerramenta {
            codigo: 0,
            texto: "MD5 hash de x:\r\n2ae9c9d58b70a340ceaad0e2da3a491f\r\n".to_string(),
        };

        assert!(conferir_o_resumo(do_certutil(&md5, Algoritmo::Sha256)).is_err());
    }

    // ─────────────── o que o pacote precisa ter dentro ───────────────

    /// A listagem de verdade do pacote, abreviada — extraída com o `bsdtar` do
    /// `System32` em 23/08/2026. São 356 entradas no total.
    const LISTAGEM: &[&str] = &[
        ".disk/info",
        "Clonezilla-Live-Version",
        "EFI/boot/bootx64.efi",
        "boot/grub/grub.cfg",
        "home/partimag/",
        "live/filesystem.squashfs",
        "live/initrd.img",
        "live/vmlinuz",
        "syslinux/isolinux.cfg",
        "utils/",
    ];

    #[test]
    fn o_pacote_de_verdade_tem_os_quatro_caminhos() {
        assert!(
            o_que_falta(LISTAGEM.iter().copied()).is_empty(),
            "os quatro caminhos obrigatorios tem de estar na listagem medida"
        );
    }

    #[test]
    fn um_pacote_sem_o_efi_e_recusado() {
        // Um zip que extrai sem erro e sem o `bootx64.efi` produz um
        // dispositivo que nao boota — e isso so se descobre depois de o
        // Windows ter sido apagado, que e quando alguem precisa dele.
        let sem_efi: Vec<&str> = LISTAGEM
            .iter()
            .copied()
            .filter(|caminho| !caminho.contains("bootx64"))
            .collect();

        assert_eq!(
            o_que_falta(sem_efi.into_iter()),
            vec!["EFI/boot/bootx64.efi".to_string()]
        );
    }

    #[test]
    fn a_barra_do_windows_nao_faz_um_caminho_sumir() {
        // Quem lista e o `bsdtar`, que usa `/`; quem confere e o Windows, que
        // usa `\`. Trocar os dois produziria um "nao achei" que nao quer dizer
        // nada — e a consequencia seria recusar um pacote bom depois de o
        // disco ja ter sido apagado.
        let com_barra_invertida = [
            r"EFI\boot\bootx64.efi",
            r"live\vmlinuz",
            r"live\initrd.img",
            r"boot\grub\grub.cfg",
        ];

        assert!(o_que_falta(com_barra_invertida.into_iter()).is_empty());
    }

    #[test]
    fn um_prefixo_a_mais_no_zip_e_recusa_e_nao_um_sim() {
        // Um zip que extraisse tudo dentro de `clonezilla-live/` produziria um
        // `ARCABOOT` cujo `EFI/` esta um nivel abaixo — e o firmware nao acha
        // o `.efi`. O pacote de hoje extrai na raiz, e este teste guarda essa
        // premissa.
        let aninhado: Vec<String> = LISTAGEM
            .iter()
            .map(|caminho| format!("clonezilla-live/{caminho}"))
            .collect();

        assert_eq!(
            o_que_falta(aninhado.iter().map(String::as_str)).len(),
            CAMINHOS_OBRIGATORIOS.len()
        );
    }

    #[test]
    fn a_caixa_do_nome_nao_derruba_o_pacote() {
        let em_maiuscula = [
            "EFI/BOOT/BOOTX64.EFI",
            "LIVE/VMLINUZ",
            "LIVE/INITRD.IMG",
            "BOOT/GRUB/GRUB.CFG",
        ];
        assert!(o_que_falta(em_maiuscula.into_iter()).is_empty());
    }
}
