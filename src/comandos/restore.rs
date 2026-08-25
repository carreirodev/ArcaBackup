//! `arca restore` — a §6.1 do PRD (R-1, R-2, R-3, R-7, S-2).
//!
//! **E a unica operacao do ARCA que destroi dados.** Tudo aqui existe para que
//! o disco que vai ser apagado seja o disco que a pessoa quis apagar.
//!
//! # Fiacao, outra vez, e isso continua sendo o desenho
//!
//! Como a E8, quase nada aqui e mecanismo novo. A receita de restauracao esta
//! montada e validada desde a E3; [`crate::armar`] recebe a operacao como
//! parametro e nao sabe qual e — grava o estado, deriva o bloco, arma o
//! `grub.cfg`, migra a entrada (C-4), aponta o `device` (C-6), marca o boot
//! unico e relê tudo (C-3, C-5); [`crate::desfecho`] ja lê `ARCA_RESTORE=` por
//! sufixo; `arca resultado` ja colhe as duas operacoes. O que a E9 escreve e:
//! **a escolha, a conferencia e a recusa.**
//!
//! O que e codigo novo, e esta marcado como tal:
//!
//! | Parte | Origem |
//! |---|---|
//! | A receita e as flags de R-4 | Transcrito — ver [`crate::receita`] |
//! | Armar, e a releitura de C-3 | Rodou em hardware em 22/08/2026 (E7) |
//! | A lista numerada e a escolha (R-1) | **Codigo novo** |
//! | A conferencia da imagem (R-2) | **Codigo novo** |
//! | A recusa por identidade do disco (R-7) | **Codigo novo** — ADR-0010, ADR-0015 |
//!
//! # A ordem, e o que cada posicao impede
//!
//! 1. **Desarmar** (C-1), incondicionalmente, como primeiro passo.
//! 2. **Escolher a imagem** — antes de qualquer conferencia, porque nao ha o
//!    que conferir sem imagem escolhida.
//! 3. **Conferir a imagem** (R-2): `disk`, `blkdev.list` e o `sgdisk`, os tres
//!    de dentro dela.
//! 4. **Escolher e julgar o destino** (R-7), inclusive a recusa dura do
//!    proprio dispositivo.
//! 5. **A tela do §6.1**, com os dois discos nomeados.
//! 6. **A confirmacao digitada** (S-2, R-3), antes de qualquer escrita.
//! 7. **Armar** — o ponto sem volta, com a releitura de C-3 dentro.
//! 8. **Os avisos**, depois de armado e antes de reiniciar (C-9).
//! 9. **Reiniciar**, por ultimo.
//!
//! Toda recusa acontece **antes** do 6. Ninguem digita o nome inteiro de uma
//! imagem para ouvir um nao depois — a mesma regra que a E7 aplicou ao disco
//! de origem.

use crate::app::Contexto;
use crate::armar;
use crate::blkdev;
use crate::confirmacao;
use crate::desarme::{self, Desarme};
use crate::dispositivo::{self, Dispositivo};
use crate::erro::{Erro, Resultado};
use crate::formato::{linha, tamanho};
use crate::gpt::{self, OrigemDaImagem, SemMedida};
use crate::imagens::{self, Especie, Pasta, Veredito};
use crate::nome::Nome;
use crate::portas::{Arquivos, DiscoFisico};
use crate::prevoo;
use crate::receita::{Disco, Operacao, Pedido, Receita, Selo};
use std::fmt;
use std::path::Path;

use super::status;

/// O arquivo em que o Clonezilla escreve o nome Linux do disco de origem.
const ARQUIVO_DISK: &str = "disk";

/// O `lsblk` que a imagem carrega, e de onde sai o par nome-modelo.
///
/// Importado de [`crate::blkdev`] desde a E12, quando a sondagem passou a
/// escrever um arquivo com o mesmo nome e o mesmo formato — ele deixou de ser
/// so "o arquivo de dentro da imagem".
use crate::blkdev::ARQUIVO as ARQUIVO_BLKDEV;

/// A linha de comando que criou a imagem, escrita pelo proprio Clonezilla.
///
/// Nao e conferencia de nada — e procedencia. Numa tela que apaga um disco,
/// dizer com que comando aquela imagem foi feita custa uma linha e responde a
/// pergunta que ninguem tem como responder depois.
const ARQUIVO_COMANDO: &str = "Info-saved-by-cmd.txt";

/// O alvo do `/enum` que traz as entradas de boot **e** o bloco do
/// `{fwbootmgr}` — os dois na mesma leitura, que e como o `arca status` lê
/// desde a E2.
const FIRMWARE: &str = "firmware";

// ─────────────────────────── as recusas ───────────────────────────

/// Por que a restauracao foi recusada, e sempre antes da confirmacao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDaRestauracao {
    /// Nao ha imagem para oferecer, e as pastas que ha nao contam — cada uma
    /// pela sua razao, e as duas sao ditas.
    ///
    /// Contar so os residuos era o furo: um `ARCAVAULT` com uma pasta de
    /// imagem cujo nome nao passa por B-2 dizia "nao ha imagem no ARCAVAULT
    /// para restaurar" enquanto o `arca list` mostrava a pasta. E a mesma
    /// omissao que a [`montar_a_lista`] argumenta contra para o residuo — so
    /// que aquele caminho nunca e alcancado quando nao ha imagem nenhuma.
    NadaAOferecer {
        residuos: usize,
        sem_nome_valido: usize,
    },

    /// O nome pedido na linha de comando nao esta no dispositivo.
    ImagemDesconhecida { nome: String },

    /// O nome pedido existe e e residuo. L-2: nunca oferecido.
    ImagemEResiduo { nome: String },

    /// A pasta e uma imagem e o nome dela nao passa por B-2. Ele iria para a
    /// receita, e a receita e uma linha so de shell (C-2).
    NomeNaoCabeNaReceita { nome: String, porque: String },

    /// A escolha digitada nao e um dos indices da lista.
    EscolhaInvalida { digitado: String, quantas: usize },

    /// A imagem nao traz o arquivo `disk`, ou ele nao se deixou lê.
    SemArquivoDaOrigem { arquivo: String, motivo: String },

    /// Os dois arquivos da imagem discordam sobre que disco ela retratou.
    ImagemInconsistente { disse: String, e_disse: String },

    /// Nao deu para medir o disco de origem dentro da imagem (R-7).
    SemMedidaDaOrigem(SemMedida),

    /// Nao ha disco no Windows com o modelo do disco de origem.
    ///
    /// Desde o ADR-0015 isto e o fim do caminho, e nao um convite a nomear
    /// outro disco: o unico destino valido e o disco de onde a imagem veio, e
    /// ele nao esta aqui.
    SemDestinoObvio { modelo: String },

    /// Mais de um disco casa com o modelo da origem, e o ARCA **para**.
    ///
    /// Ate o ADR-0015 esta recusa mandava nomear o destino com
    /// `--destino <indice>`. Pedir que alguem aponte transformaria uma duvida
    /// do ARCA numa afirmacao do usuario sobre a qual nao ha como conferir
    /// nada — o mesmo raciocinio da E7 ao nao pedir o nome do disco do Linux
    /// (§4.5).
    DestinoAmbiguo { modelo: String, quantos: usize },

    /// O destino escolhido e o **proprio dispositivo ARCA**, pelas letras que
    /// ele carrega no Windows.
    DestinoEODispositivo { modelo: String, letras: String },

    /// O nome que o **Linux** dara ao destino e o mesmo que ele dara ao
    /// dispositivo ARCA — e e o nome do Linux que entra na receita.
    ///
    /// A recusa por letra nao pega este caso: os dois discos sao diferentes
    /// para o Windows, e viram o mesmo do outro lado do reinicio.
    DestinoResolveNoDispositivo { disco: String, modelo: String },

    /// O `MSFT_Disk` nao respondeu por este disco, e sem medida R-7 nao se
    /// responde.
    SemMedidaDoDestino { modelo: String },

    /// O setor logico do destino nao e o da origem.
    SetorDivergente { origem: u64, destino: u64 },

    /// R-7: o disco na mesa **nao e** o disco de que a imagem veio.
    ///
    /// A comparacao e de **identidade**, e nao de capacidade, desde o
    /// [ADR-0015]: nao batendo os setores — para mais ou para menos —, este e
    /// outro disco. Sobrar espaco nao e permissao para nada, porque o unico
    /// destino valido e a origem.
    ///
    /// [ADR-0015]: ../../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md
    NaoEODiscoDeOrigem {
        origem_setores: u64,
        destino_setores: u64,
        bytes_por_setor: u64,
    },

    /// Nao se sabe que nome o Linux da ao disco de destino (§4.5).
    SemNomeDoDestino(blkdev::SemNome),
}

impl fmt::Display for RecusaDaRestauracao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDaRestauracao::NadaAOferecer {
                residuos: 0,
                sem_nome_valido: 0,
            } => write!(f, "nao ha imagem no ARCAVAULT para restaurar"),
            RecusaDaRestauracao::NadaAOferecer {
                residuos,
                sem_nome_valido,
            } => {
                write!(f, "nao ha imagem no ARCAVAULT que o ARCA possa restaurar")?;
                if *residuos > 0 {
                    write!(
                        f,
                        ". Ha {residuos} pasta(s) sem `MD5SUMS`, e residuo nunca e oferecido (L-2): sao rastros de backup interrompido, e nao ha imagem inteira neles"
                    )?;
                }
                if *sem_nome_valido > 0 {
                    write!(
                        f,
                        ". E ha {sem_nome_valido} pasta(s) que sao imagens e cujo nome nao pode entrar numa receita (B-2, C-2) — renomeie-as e rode de novo"
                    )?;
                }
                Ok(())
            }
            RecusaDaRestauracao::ImagemDesconhecida { nome } => write!(
                f,
                "nao ha imagem chamada `{nome}` no ARCAVAULT. Rode `arca list` para ver o que ha"
            ),
            RecusaDaRestauracao::ImagemEResiduo { nome } => write!(
                f,
                "a pasta `{nome}` nao tem `MD5SUMS`: e residuo de um backup interrompido, e nao uma imagem. O ARCA nunca a oferece para restaurar (L-2), porque nao ha imagem inteira ali"
            ),
            RecusaDaRestauracao::NomeNaoCabeNaReceita { nome, porque } => write!(
                f,
                "a pasta `{nome}` e uma imagem, e o nome dela nao pode entrar numa receita: {porque}. A receita e uma linha so de shell dentro do `grub.cfg` (C-2), e o nome da imagem atravessa ela dez vezes. Renomeie a pasta e rode de novo"
            ),
            RecusaDaRestauracao::EscolhaInvalida { digitado, quantas } => write!(
                f,
                "`{digitado}` nao e uma das {quantas} imagens da lista. Rode o comando de novo e digite o numero entre colchetes — nada foi armado"
            ),
            RecusaDaRestauracao::SemArquivoDaOrigem { arquivo, motivo } => write!(
                f,
                "a imagem nao traz o `{arquivo}` legivel ({motivo}), e e dele que sai a identidade do disco que ela retratou. Sem isso o ARCA nao confere o destino (R-2), e uma restauracao sem essa conferencia e um disco apagado no escuro"
            ),
            RecusaDaRestauracao::ImagemInconsistente { disse, e_disse } => write!(
                f,
                "os arquivos da imagem discordam sobre que disco ela retratou: o `{ARQUIVO_DISK}` diz `{disse}` e o `sgdisk` diz `{e_disse}`. Duas fontes da mesma imagem nao podem divergir, e o ARCA nao escolhe entre elas"
            ),
            RecusaDaRestauracao::SemMedidaDaOrigem(porque) => write!(f, "{porque}"),
            RecusaDaRestauracao::SemDestinoObvio { modelo } => write!(
                f,
                "nenhum disco desta maquina tem o modelo `{modelo}`, que e o do disco de onde a imagem veio. **O unico destino valido e o disco de origem** (R-7, ADR-0015), e ele nao esta aqui: nao ha outro disco a oferecer, e o ARCA nao aceita que se aponte um. Trocado o disco, o caminho e reinstalar o Windows"
            ),
            RecusaDaRestauracao::DestinoAmbiguo { modelo, quantos } => write!(
                f,
                "{quantos} discos desta maquina tem o modelo `{modelo}`, que e o do disco de onde a imagem veio, e o ARCA **nao sabe qual dos dois e**. O modelo e a unica coisa que liga a imagem a um disco desta mesa (§4.5), e ela nao os distingue. Pedir que voce apontasse transformaria esta duvida numa afirmacao sobre a qual nao ha nada contra o que conferir — e a operacao apaga o disco. Desconecte o que nao for a origem e rode de novo"
            ),
            RecusaDaRestauracao::DestinoEODispositivo { modelo, letras } => write!(
                f,
                "o destino escolhido e o **proprio dispositivo ARCA** (`{modelo}`, em {letras}). Restaurar nele apagaria o Clonezilla e todas as imagens no meio da operacao — inclusive a que esta sendo restaurada. O ARCA recusa isto sempre, e nao ha confirmacao que o libere"
            ),
            RecusaDaRestauracao::DestinoResolveNoDispositivo { disco, modelo } => write!(
                f,
                "o destino escolhido (`{modelo}`) e um disco diferente do dispositivo ARCA para o Windows, e o **mesmo** para o Linux: os dois resolvem em `{disco}`, que e o nome que entraria na receita. Restaurar assim apagaria o dispositivo. O nome do Linux sai do `blkdev.list` das imagens, que casa por MODELO — e dois discos do mesmo modelo sao indistinguiveis por ele (§4.5). O ARCA nao escolhe entre os dois"
            ),
            RecusaDaRestauracao::SemMedidaDoDestino { modelo } => write!(
                f,
                "o Windows nao respondeu o tamanho do disco `{modelo}` pelo `MSFT_Disk`, e e essa a unica regua que casa com a que a imagem registra. Sem medir o destino nao da para responder se ele **e** o disco de origem (R-7), e \"nao consegui medir\" nao vira \"e ele\""
            ),
            RecusaDaRestauracao::SetorDivergente { origem, destino } => write!(
                f,
                "o disco de origem tem setor logico de {origem} bytes e o de destino tem {destino}. A tabela de particao da imagem e escrita em setores da origem, e `-k0` a copia inteira: num disco de outro setor ela enderecaria outro lugar. Isto nao esta medido neste projeto, e o ARCA nao adivinha"
            ),
            RecusaDaRestauracao::NaoEODiscoDeOrigem {
                origem_setores,
                destino_setores,
                bytes_por_setor,
            } => write!(
                f,
                "R-7: este NAO e o disco de que a imagem veio — {destino_setores} setores contra {origem_setores}, de {bytes_por_setor} bytes cada ({} contra {}). O modelo casou e o tamanho nao, entao e outro disco. **O unico destino valido e o de origem** (ADR-0015), e por isso a comparacao e de igualdade: {}. Trocado o disco, o caminho e reinstalar o Windows",
                tamanho(destino_setores.saturating_mul(*bytes_por_setor)),
                tamanho(origem_setores.saturating_mul(*bytes_por_setor)),
                if destino_setores > origem_setores {
                    "sobrar espaco nao e permissao para restaurar aqui"
                } else {
                    "faltar espaco tambem faria o proprio Clonezilla desistir, do outro lado do reinicio"
                }
            ),
            RecusaDaRestauracao::SemNomeDoDestino(porque) => write!(
                f,
                "{porque}. A receita nomeia o disco pelo nome que o **Linux** lhe da, e o Windows nao conhece esse nome (§4.5)"
            ),
        }
    }
}

// ─────────────────────── a escolha da imagem (R-1, L-2) ───────────────────────

/// O que se pode restaurar, e o que ficou de fora.
///
/// # Duas razões para uma pasta não ser oferecida, e as duas são ditas
///
/// A primeira é L-2: **resíduo nunca é oferecido**. A segunda apareceu
/// relendo `Nome::novo` com a restauração na mão: o nome da imagem escolhida
/// **vai para a receita**, e a receita passa por C-2. Uma pasta chamada
/// `Backup Antigo`, criada à mão no `ARCAVAULT`, tem `MD5SUMS` e é uma imagem
/// para [`crate::imagens`] — mas o espaço no nome quebraria a string do
/// `bash -c`, e B-2 a recusa.
///
/// Sem esta separação ela seria **oferecida na lista e recusada depois da
/// escolha**, com a mensagem de B-2 — *"recusar nome com espaço… escolha outro
/// nome"* —, que numa restauração não faz sentido nenhum: o nome não está
/// sendo escolhido, ele é o da pasta que está lá. Recusar antes de oferecer é
/// o mesmo raciocínio de §4.5 sobre o disco de origem: **ninguém digita o nome
/// inteiro de uma imagem para ouvir um não depois.**
pub struct Oferta<'a> {
    /// So imagem restauravel, e na ordem em que a lista as numera.
    pub imagens: Vec<&'a Pasta>,

    /// Os residuos, que aparecem nomeados e **nunca numerados** (L-2).
    pub residuos: Vec<&'a Pasta>,

    /// Imagens cujo nome o ARCA nao sabe pôr numa receita, com o motivo.
    pub sem_nome_valido: Vec<(&'a Pasta, crate::nome::Recusa)>,
}

impl<'a> Oferta<'a> {
    pub fn de(pastas: &'a [Pasta]) -> Oferta<'a> {
        let mut oferta = Oferta {
            imagens: Vec::new(),
            residuos: Vec::new(),
            sem_nome_valido: Vec::new(),
        };

        for pasta in pastas {
            if !pasta.e_imagem() {
                oferta.residuos.push(pasta);
            } else if let Err(porque) = Nome::novo(&pasta.nome) {
                oferta.sem_nome_valido.push((pasta, porque));
            } else {
                oferta.imagens.push(pasta);
            }
        }

        oferta
    }
}

/// A lista numerada do §6.1.
///
/// # Por que ela nao reusa `list::montar`
///
/// A E8 estabeleceu que o `arca resultado` reusa `list::montar` em vez de
/// formatar imagens de novo, e a regra continua boa. Esta lista **diverge**, e
/// a razao e que ela responde outra pergunta.
///
/// `list::montar` responde *"o que ha no dispositivo"*, e por isso mostra
/// residuo — mostrar e o servico que ela presta, e L-2 pede exatamente que ele
/// apareca **marcado** como residuo. Esta responde *"o que da para
/// restaurar"*, e L-2 pede que residuo **nunca seja oferecido**. Um numero ao
/// lado de um residuo seria um numero que nao se pode digitar; pior, ele
/// ocuparia um indice, e ai os numeros da lista passariam a depender de coisas
/// que nao sao escolhiveis. Uma pessoa que digitasse `2` olhando a lista do
/// `arca list` escolheria a imagem errada.
///
/// Os residuos continuam ditos, embaixo e sem numero. Omiti-los faria a lista
/// parecer incompleta para quem sabe que ha outra pasta la — e a E8 ja pagou
/// essa licao ao descobrir que a §5.4 do PRD escondia a `ARCA-TESTE-03`.
pub fn montar_a_lista(oferta: &Oferta) -> String {
    // A linha em branco pertence a lista, e nao a quem a imprime: ela e o que
    // separa o cabecalho — que conta o desarmar — da escolha.
    let mut saida = format!("\nImagens em {}:\n", dispositivo::ARCAVAULT);

    let coluna = oferta
        .imagens
        .iter()
        .map(|pasta| pasta.nome.chars().count())
        .max()
        .unwrap_or(0)
        + 3;

    for (indice, pasta) in oferta.imagens.iter().enumerate() {
        // O preenchimento e contado a mao: `{:<n$}` conta bytes, e um nome com
        // acento sairia desalinhado.
        let recuo = " ".repeat(coluna - pasta.nome.chars().count());
        saida.push_str(&format!(
            "  [{}] {}{recuo}{} · {} · {}\n",
            indice + 1,
            pasta.nome,
            crate::formato::dia_e_mes(pasta.modificado_em),
            tamanho(pasta.tamanho_bytes),
            parecer(&pasta.especie),
        ));
    }

    if !oferta.residuos.is_empty() {
        saida.push_str(&format!(
            "\n  Sem numero, e nao oferecido (L-2): {}\n",
            oferta
                .residuos
                .iter()
                .map(|pasta| pasta.nome.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        saida.push_str("  Pasta sem `MD5SUMS` e residuo de backup interrompido, e nao imagem.\n");
    }

    for (pasta, porque) in &oferta.sem_nome_valido {
        saida.push_str(&format!(
            "\n  Sem numero, e o ARCA nao pode restaurar: `{}`\n\
             \x20 {porque}\n\
             \x20 O nome da imagem entra na receita que o Clonezilla executa, e ela e uma\n\
             \x20 linha so de shell (C-2). Renomeie a pasta e rode de novo.\n",
            pasta.nome
        ));
    }

    saida
}

/// A ultima coluna da lista: o que o `ocs-chkimg` disse daquela imagem.
///
/// # Reprovada continua sendo oferecida, e isso e decisao
///
/// L-2 fala de **residuo**, e nao de veredito. Uma imagem reprovada tem
/// `MD5SUMS` e e uma imagem; o que ela nao tem e a garantia de que restaura.
/// Recusa-la aqui seria o ARCA decidir por quem esta na frente da tela — e o
/// caso em que isso e caro e justamente o que motiva o projeto: com o disco de
/// origem morto, uma imagem reprovada pode ser tudo que restou.
///
/// O que o ARCA faz e dizer, na lista e outra vez na tela de confirmacao. Ver
/// [`avisos_da_imagem`].
fn parecer(especie: &Especie) -> &'static str {
    match especie {
        Especie::Imagem {
            veredito: Some(Veredito::Aprovada),
        } => "aprovada",
        Especie::Imagem {
            veredito: Some(Veredito::Reprovada),
        } => "REPROVADA",
        Especie::Imagem { veredito: None } => "sem veredito",
        Especie::Residuo => "residuo",
    }
}

// ─────────────────── a conferencia da imagem (R-2) ───────────────────

/// O disco que a imagem retratou, conferido contra ela mesma.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Retrato {
    /// O nome Linux, do arquivo `disk`.
    pub disco: String,

    /// A GPT, do `<disco>-gpt.sgdisk`: setores, setor logico e modelo.
    pub origem: OrigemDaImagem,

    /// O que o `blkdev.list` diz do modelo daquele disco, quando ele o diz.
    pub modelo_no_blkdev: Option<String>,

    /// A linha de comando que criou a imagem, escrita pelo Clonezilla.
    pub comando_que_criou: Option<String>,
}

/// R-2: confere o destino contra o conteudo da propria imagem.
///
/// # O que R-2 passou a conferir na E9, e por que
///
/// O requisito diz "`disk`/`blkdev.list`". Abrindo uma imagem de verdade, ha
/// mais — e duas coisas a mais mudam o que a etapa pode fazer:
///
/// - **O `<disco>-gpt.sgdisk`** traz o total de setores, o tamanho do setor e
///   o modelo, escritos pela mesma ferramenta no mesmo instante. E a unica
///   medida da origem que existe do lado Windows, e sem ela R-7 nao tem contra
///   o que comparar. Ver [`crate::gpt`].
/// - **O `Info-saved-by-cmd.txt`** traz a linha de comando que criou a imagem.
///   Nao confere nada; e procedencia, e vai para a tela.
///
/// E o `disk` e o `sgdisk` sao conferidos **um contra o outro**. Sao dois
/// arquivos independentes da mesma pasta dizendo que disco foi retratado; se
/// discordarem, aquela pasta nao e uma imagem coerente, e escolher entre as
/// duas fontes seria adivinhar num comando que apaga um disco.
fn conferir_a_imagem(
    arquivos: &dyn Arquivos,
    pasta_da_imagem: &Path,
) -> Result<Retrato, RecusaDaRestauracao> {
    let ler = |arquivo: &str| {
        arquivos
            .ler_texto_alheio(&pasta_da_imagem.join(arquivo))
            .map_err(|erro| RecusaDaRestauracao::SemArquivoDaOrigem {
                arquivo: arquivo.to_string(),
                motivo: erro.to_string(),
            })
    };

    // O `disk` passa pelo mesmo validador da receita, e nao por um `trim`.
    // Duas razoes, e a segunda e a que importa: o nome sai daqui e vira parte
    // de um **caminho de arquivo** (`<disco>-gpt.sgdisk`), e um `disk` com
    // barra ou `..` faria a leitura sair da pasta da imagem. O conteudo vem do
    // Clonezilla e nao do usuario, mas ele vem de dentro de uma pasta que o
    // usuario copia de onde quiser — e `Disco::novo` ja recusa tudo que nao
    // seja `[a-z][a-z0-9]*`, que e a forma de todo nome de disco do Linux.
    let bruto = ler(ARQUIVO_DISK)?.trim().to_string();
    let disco = match Disco::novo(&bruto) {
        Ok(disco) => disco.como_texto().to_string(),
        Err(porque) => {
            return Err(RecusaDaRestauracao::SemArquivoDaOrigem {
                arquivo: ARQUIVO_DISK.to_string(),
                motivo: if bruto.is_empty() {
                    "o arquivo esta vazio".to_string()
                } else {
                    porque.to_string()
                },
            });
        }
    };

    let arquivo_do_sgdisk = gpt::arquivo_do_disco(&disco);
    let origem = gpt::ler(&arquivo_do_sgdisk, &ler(&arquivo_do_sgdisk)?)
        .map_err(RecusaDaRestauracao::SemMedidaDaOrigem)?;

    if origem.disco != disco {
        return Err(RecusaDaRestauracao::ImagemInconsistente {
            disse: disco,
            e_disse: origem.disco,
        });
    }

    // O `blkdev.list` e o `Info-saved-by-cmd.txt` sao informativos, e uma
    // leitura que falhe nao derruba a restauracao: o que R-2 exige ja foi
    // conferido acima, e o modelo tem uma segunda fonte no proprio `sgdisk`.
    let modelo_no_blkdev = arquivos
        .ler_texto_alheio(&pasta_da_imagem.join(ARQUIVO_BLKDEV))
        .ok()
        .and_then(|texto| {
            blkdev::ler(&texto)
                .into_iter()
                .find(|achado| achado.nome == disco)
                .map(|achado| achado.modelo)
        });

    // Duas fontes da mesma imagem discordando sobre o modelo e o mesmo caso do
    // nome: recusa, e nao escolha.
    if let Some(no_blkdev) = &modelo_no_blkdev {
        if !blkdev::mesmo_modelo(no_blkdev, &origem.modelo) {
            return Err(RecusaDaRestauracao::ImagemInconsistente {
                disse: format!("{disco} e um `{no_blkdev}`"),
                e_disse: format!("um `{}`", origem.modelo),
            });
        }
    }

    let comando_que_criou = arquivos
        .ler_texto_alheio(&pasta_da_imagem.join(ARQUIVO_COMANDO))
        .ok()
        .map(|texto| texto.trim().to_string())
        .filter(|texto| !texto.is_empty());

    Ok(Retrato {
        disco,
        origem,
        modelo_no_blkdev,
        comando_que_criou,
    })
}

// ─────────────────────────── o destino (R-7) ───────────────────────────

/// O disco que vai ser apagado, e ele so pode ser o de origem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Destino {
    /// O indice do Windows. Serve para nomear na tela, e **nunca** chega a
    /// receita: o indice do Windows nao e o do Linux.
    pub indice: u32,
    pub modelo: String,

    /// O nome Linux, lido do `blkdev.list` de alguma imagem (§4.5).
    pub disco: Disco,

    pub setores: u64,
    pub bytes_por_setor: u64,
}

impl Destino {
    pub fn bytes(&self) -> u64 {
        self.setores.saturating_mul(self.bytes_por_setor)
    }
}

/// Acha o disco de origem nesta mesa e julga se e mesmo ele (R-7).
///
/// # Nao ha o que escolher, e e isso que o ADR-0015 mudou
///
/// Ate a E9 esta funcao aceitava um `pedido: Option<u32>` vindo de
/// `--destino <indice>`, porque destino divergente era permitido. Nao e mais:
/// **o unico destino valido e o disco de que a imagem veio**. O que sobrou e
/// achar esse disco e provar que e ele — e nao havendo prova, parar.
///
/// # A ordem das recusas, e ela nao e estetica
///
/// 1. **O proprio dispositivo**, sempre e antes de tudo. E a unica recusa
///    desta funcao que nenhuma confirmacao libera.
/// 2. **A medida**, porque sem ela nada abaixo se responde.
/// 3. **O setor logico**, porque um destino de outro setor torna a comparacao
///    de tamanho sem sentido antes de ela acontecer.
/// 4. **A identidade** (R-7): os setores batem exatamente, ou e outro disco.
/// 5. **O nome Linux**, por ultimo, porque e a unica que depende de haver
///    imagem no dispositivo de onde lê-lo — e a mais barata de resolver.
fn escolher_o_destino(
    discos: &[DiscoFisico],
    dispositivo: &Dispositivo,
    retrato: &Retrato,
    listas: &[blkdev::Lista],
) -> Result<Destino, RecusaDaRestauracao> {
    let do_dispositivo: Vec<char> = dispositivo
        .vault
        .letra
        .into_iter()
        .chain(dispositivo.boot.as_ref().and_then(|boot| boot.letra))
        .collect();
    let e_o_dispositivo =
        |disco: &DiscoFisico| do_dispositivo.iter().any(|letra| disco.tem_a_letra(*letra));

    // O destino e o disco de que a imagem veio, achado pelo **modelo** — que e
    // o que liga a imagem a um disco desta mesa. Nao e o `%SystemDrive%`: numa
    // maquina com dois Windows, o disco onde o `C:` mora nao e necessariamente
    // o que a imagem retratou, e a imagem sabe qual e.
    //
    // O disco do dispositivo fica **fora** desta busca, e nao por economia. Se
    // ele tiver o mesmo modelo do disco de origem — dois SSDs iguais, um
    // interno e um na mesa —, sem o filtro haveria dois candidatos e o comando
    // recusaria por `DestinoAmbiguo`, quando um dos dois nunca poderia ser
    // destino.
    let casam: Vec<&DiscoFisico> = discos
        .iter()
        .filter(|disco| {
            !e_o_dispositivo(disco) && blkdev::mesmo_modelo(&disco.modelo, &retrato.origem.modelo)
        })
        .collect();

    let escolhido = match casam.len() {
        0 => {
            return Err(RecusaDaRestauracao::SemDestinoObvio {
                modelo: retrato.origem.modelo.clone(),
            });
        }
        1 => casam[0],

        // Dois discos do mesmo modelo, e o ARCA para aqui. Nao ha `--destino`
        // a oferecer desde o ADR-0015: o modelo e a unica ligacao que existe
        // entre a imagem e um disco desta mesa, e ele nao os distingue. Pedir
        // que alguem apontasse seria transformar esta duvida numa afirmacao
        // sem oraculo, num comando que apaga o disco.
        quantos => {
            return Err(RecusaDaRestauracao::DestinoAmbiguo {
                modelo: retrato.origem.modelo.clone(),
                quantos,
            });
        }
    };

    julgar_o_destino(escolhido, discos, &do_dispositivo, retrato, listas)
}

/// Julga o disco candidato: e ele mesmo, e nao e o dispositivo?
///
/// # Por que isto e uma funcao separada, e nao o resto de [`escolher_o_destino`]
///
/// Porque **as duas recusas de R-8 deixaram de ser alcancaveis pelo caminho
/// normal**, e continuam valendo. A busca acima filtra o dispositivo fora dos
/// candidatos — precisa filtrar, senao um dispositivo do mesmo modelo da
/// origem produziria uma ambiguidade falsa —, e com isso nenhum `escolhido`
/// que chegue aqui pode ser o dispositivo.
///
/// O [ADR-0015] previu exatamente isso ao decidir que R-8 fica: *"com o
/// destino amarrado ao disco de origem ela vira redundante no caminho normal,
/// e e por isso que fica: a revisao da E9 mostrou que a recusa por letra tinha
/// um contorno por acidente de modelo, e uma segunda barreira custa nada"*.
///
/// Uma barreira redundante sem teste e uma barreira que ninguem sabe se
/// funciona. Separando o julgamento da escolha, ela continua **exercitada** —
/// os testes a alcancam por aqui — sem que a escolha precise deixar o
/// dispositivo entrar na lista de candidatos para isso.
///
/// [ADR-0015]: ../../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md
fn julgar_o_destino(
    escolhido: &DiscoFisico,
    discos: &[DiscoFisico],
    do_dispositivo: &[char],
    retrato: &Retrato,
    listas: &[blkdev::Lista],
) -> Result<Destino, RecusaDaRestauracao> {
    let e_o_dispositivo =
        |disco: &DiscoFisico| do_dispositivo.iter().any(|letra| disco.tem_a_letra(*letra));

    // 1. O dispositivo ARCA nunca e destino. Ele carrega o Clonezilla que esta
    //    executando a receita e as imagens que ela lê: apaga-lo seria serrar o
    //    galho no meio da operacao, inclusive a imagem de origem.
    if e_o_dispositivo(escolhido) {
        return Err(RecusaDaRestauracao::DestinoEODispositivo {
            modelo: escolhido.modelo.clone(),
            letras: do_dispositivo
                .iter()
                .filter(|letra| escolhido.tem_a_letra(**letra))
                .map(|letra| format!("{letra}:"))
                .collect::<Vec<_>>()
                .join(" e "),
        });
    }

    // 2. A medida da regua certa, e nunca a do `Win32_DiskDrive`. Ver
    //    [`crate::portas::Medida`] e o ADR-0010.
    let medida = escolhido
        .medida
        .ok_or_else(|| RecusaDaRestauracao::SemMedidaDoDestino {
            modelo: escolhido.modelo.clone(),
        })?;

    // 3. Mesma unidade dos dois lados, ou a comparacao do passo 4 nao quer
    //    dizer nada.
    if medida.bytes_por_setor != retrato.origem.bytes_por_setor {
        return Err(RecusaDaRestauracao::SetorDivergente {
            origem: retrato.origem.bytes_por_setor,
            destino: medida.bytes_por_setor,
        });
    }

    // 4. R-7, em setores, com as duas pontas na mesma regua — e por
    //    **igualdade**. O modelo casar diz que e o mesmo tipo de disco; o
    //    tamanho bater exatamente e o que diz que e o mesmo disco. Um `>=`
    //    aqui aceitaria um gemeo maior, e o unico destino valido e a origem
    //    (ADR-0015).
    let setores = medida.setores();
    if setores != retrato.origem.setores {
        return Err(RecusaDaRestauracao::NaoEODiscoDeOrigem {
            origem_setores: retrato.origem.setores,
            destino_setores: setores,
            bytes_por_setor: medida.bytes_por_setor,
        });
    }

    // 5. O nome do Linux, pelo mesmo oraculo de §4.5 que o backup usa. Um
    //    disco que nenhum `blkdev.list` conhece nao entra numa receita.
    let achado = blkdev::nome_do_disco(&escolhido.modelo, listas)
        .map_err(RecusaDaRestauracao::SemNomeDoDestino)?;

    // 6. **E o nome que saiu nao pode ser o do dispositivo.**
    //
    // A recusa do passo 1 e por **letra do Windows**, e o nome que vai para a
    // receita e do **Linux** — sao dois canais de identidade diferentes, e o
    // vao entre eles apaga o dispositivo. O caso: um segundo SSD interno do
    // mesmo modelo do dispositivo. `--destino <o interno>` passa pelo passo 1
    // (as letras sao outras), passa pela medida e pelo tamanho, e o passo 5
    // resolve o modelo nos `blkdev.list` — onde o unico disco daquele modelo e
    // o **dispositivo**, que ali se chama `sda`. A receita sairia
    // `restoredisk <imagem> sda`, que e exatamente o desfecho que o passo 1
    // existe para impedir.
    //
    // A defesa e resolver o nome do Linux do dispositivo pelo **mesmo
    // oraculo** e comparar. Com os dois do mesmo modelo, os dois resolvem no
    // mesmo nome e a recusa dispara; e se um dia o `blkdev.list` passar a
    // distinguir dois discos do mesmo modelo, esta comparacao continua certa
    // pelo mesmo motivo.
    //
    // Achado pela revisao de codigo da E9, e ele e o defeito mais grave da
    // etapa: uma recusa dura que se podia contornar por acidente de modelo.
    if let Some(disco_do_dispositivo) = discos.iter().find(|disco| e_o_dispositivo(disco))
        && let Ok(no_linux) = blkdev::nome_do_disco(&disco_do_dispositivo.modelo, listas)
        && no_linux.disco == achado.disco
    {
        return Err(RecusaDaRestauracao::DestinoResolveNoDispositivo {
            disco: achado.disco.como_texto().to_string(),
            modelo: escolhido.modelo.clone(),
        });
    }

    Ok(Destino {
        indice: escolhido.indice,
        modelo: escolhido.modelo.clone(),
        disco: achado.disco,
        setores,
        bytes_por_setor: medida.bytes_por_setor,
    })
}

// ─────────────────────────── a tela do §6.1 ───────────────────────────

/// A primeira metade da tela: o que ja aconteceu quando ela e impressa.
///
/// # Por que a tela sai em duas metades, e a E9 cometeu o erro antes de
/// corrigi-lo
///
/// A primeira versao deste comando montava a tela inteira **depois** de
/// escolher a imagem, conferi-la e julgar o destino — e com toda recusa
/// subindo como erro, nada era impresso. Um `arca restore --destino 1` num
/// dispositivo armado imprimia so *"o destino escolhido e o proprio
/// dispositivo ARCA"*, e o desarmar de C-1, que ja tinha acontecido, sumia em
/// silencio.
///
/// **E o mesmo defeito que a revisao da E7 pegou no `arca backup`**, cometido
/// de novo num comando escrito duas etapas depois, com o comentario que o
/// descreve a poucas linhas de distancia. C-1 nao deixa mover o desarmar para
/// depois do julgamento — ele diz incondicionalmente, como primeiro passo. A
/// saida e imprimir o que ja aconteceu antes de a recusa poder cortar o resto.
pub struct Cabecalho<'a> {
    pub dispositivo: &'a Dispositivo,

    /// O que o desarmar de C-1 fez, e `None` no ensaio, em que ele nao
    /// aconteceu. Mesma regra da §5.2: um `ok` sobre uma acao que nao
    /// aconteceu e a mentira que o `--dry-run` deste projeto ja contou uma vez.
    pub desarme: Option<&'a Desarme>,
    pub caminho_do_grub: &'a str,
}

/// O dispositivo e o desarmar, impressos **antes** de qualquer recusa.
pub fn montar_cabecalho(cabecalho: &Cabecalho) -> String {
    let mut saida = format!(
        "Dispositivo ARCA: {} ({}) · {} livres\n\n",
        dispositivo::ARCAVAULT,
        match cabecalho.dispositivo.vault.letra {
            Some(letra) => format!("{letra}:"),
            None => "sem letra".to_string(),
        },
        crate::formato::gigabytes(cabecalho.dispositivo.vault.livre_bytes)
    );

    // A montagem mora em [`crate::desarme::linha_do_desarme`] desde a E11 — ver
    // o comentario la para por que os quatro comandos que armam a compartilham.
    saida.push_str(&crate::desarme::linha_do_desarme(
        cabecalho.desarme,
        cabecalho.caminho_do_grub,
    ));

    saida
}

/// A segunda metade: o que so se colhe depois de a imagem e o destino
/// passarem.
pub struct Plano<'a> {
    pub imagem: &'a Pasta,
    pub retrato: &'a Retrato,
    pub destino: &'a Destino,

    /// Se o proximo passo e a confirmacao e o armar, ou se e o ensaio.
    pub arma_em_seguida: bool,
}

/// A §6.1 da escolha ate a linha antes da confirmacao.
pub fn montar(plano: &Plano) -> String {
    let mut saida = String::new();

    saida.push_str(&linha("Imagem escolhida", &plano.imagem.nome));

    // As duas pontas, uma embaixo da outra, com o mesmo par de numeros: e a
    // comparacao de R-7 impressa, e nao o veredito dela resumido. Quem lê tem
    // de poder refazer a conta.
    saida.push_str(&linha(
        "Origem da imagem",
        &format!(
            "{} · {} · {} setores de {} B · {}",
            plano.retrato.origem.modelo,
            plano.retrato.disco,
            plano.retrato.origem.setores,
            plano.retrato.origem.bytes_por_setor,
            tamanho(plano.retrato.origem.bytes())
        ),
    ));
    saida.push_str(&linha(
        "Destino",
        &format!(
            "{} · disco {} do Windows · {} · {} setores de {} B · {}",
            plano.destino.modelo,
            plano.destino.indice,
            plano.destino.disco,
            plano.destino.setores,
            plano.destino.bytes_por_setor,
            tamanho(plano.destino.bytes())
        ),
    ));

    // A linha de **identidade**, e nao mais a de capacidade. Ate o ADR-0015
    // ela dizia `Cabe (R-7)` e podia sair com uma sobra; agora o unico destino
    // valido e a origem, e o que ela afirma e que os dois numeros acima sao o
    // mesmo numero. Ela so e alcancavel quando bateram — nao batendo, a
    // recusa aconteceu antes de a tela chegar aqui.
    saida.push_str(&linha(
        "E o disco de origem (R-7)",
        "ok · mesmos setores, na mesma regua — o destino e o disco de que a imagem veio",
    ));
    saida.push_str(&linha(
        "Conferido contra a imagem",
        &format!(
            "ok · `{ARQUIVO_DISK}`, `{}`{}",
            gpt::arquivo_do_disco(&plano.retrato.disco),
            match &plano.retrato.modelo_no_blkdev {
                Some(_) => format!(" e `{ARQUIVO_BLKDEV}`"),
                None => String::new(),
            }
        ),
    ));

    if let Some(comando) = &plano.retrato.comando_que_criou {
        saida.push_str(&linha("Imagem criada por", comando));
    }

    saida.push_str(&avisos_da_imagem(plano));

    saida.push_str(if plano.arma_em_seguida {
        concat!(
            "\nATENCAO: a restauracao APAGA o disco de destino.\n",
            "Tudo que estiver nele sera perdido.\n"
        )
    } else {
        concat!(
            "\nEnsaio (--dry-run): nada foi desarmado, nada sera confirmado e nada sera\n",
            "armado. O mesmo comando sem `--dry-run` APAGA o disco de destino.\n"
        )
    });

    saida
}

/// O que precisa de mais de uma linha, antes da confirmacao.
///
/// Cada aviso diz **o que muda para quem esta na frente da tela**. Um aviso que
/// so diz "isto esta estranho" empurra o problema de volta para quem nao sabe
/// resolve-lo — a mesma regra dos avisos do pre-voo.
fn avisos_da_imagem(plano: &Plano) -> String {
    let mut saida = String::new();

    match &plano.imagem.especie {
        Especie::Imagem {
            veredito: Some(Veredito::Reprovada),
        } => saida.push_str(concat!(
            "\n  ESTA IMAGEM FOI REPROVADA pelo ocs-chkimg (S-5). O ARCA a oferece assim\n",
            "  mesmo — com o disco de origem perdido, uma imagem reprovada pode ser tudo\n",
            "  que restou, e recusa-la seria decidir por quem esta aqui. Mas o que se\n",
            "  espera dela e uma restauracao que falha, ou um Windows que nao sobe.\n"
        )),
        Especie::Imagem { veredito: None } => saida.push_str(concat!(
            "\n  ESTA IMAGEM ESTA SEM VEREDITO: nao ha `arca-check.log`, ou ele nao diz\n",
            "  nada reconhecivel. Imagem nao verificada e suposicao, e o ARCA nao a\n",
            "  apresenta como aprovada. `arca verify` confere os MD5SUMS sem reiniciar.\n"
        )),
        _ => {}
    }

    saida
}

/// O que se imprime depois de armado, com os dois avisos e o reinicio no fim.
///
/// # Sao dois avisos, e o segundo e da restauracao
///
/// O de C-9 — remover o SSD antes de religar — e o mesmo do backup, e continua
/// sendo a ultima coisa que alguem lê. O segundo e novo, e ele existe porque o
/// [ADR-0009] mediu uma janela que na restauracao muda de gravidade.
///
/// Depois de um boot pelo dispositivo, esta maquina fica com ele **a frente**
/// da ordem permanente — o firmware reescreve a entrada, o Windows a recria, e
/// ela entra no `displayorder`. A janela em que o `grub.cfg` fica armado vai do
/// fim da receita ao `arca resultado`: oito minutos em 22/08. **No backup isso
/// roda um backup de novo; aqui isso apaga o disco de novo**, e desta vez com
/// o Windows recem-restaurado dentro.
///
/// O ARCA **avisa e nao conserta**, porque consertar e escrever na ordem
/// permanente, que e o que C-5 proibe e o que o ADR-0009 decidiu nao fazer.
/// Devolver o `{bootmgr}` a frente e P-20, e a E10 e quem decide — a E9 nao
/// supersede aquela decisao por conta propria. O que ela faz e dizer o perigo
/// com o tamanho que ele tem aqui, e dize-lo **pelo que leu do firmware**, e
/// nao por suposicao.
///
/// [ADR-0009]: ../../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md
pub fn montar_o_armado(armado: &armar::Armado, ordem: OrdemDeBoot) -> String {
    // As mesmas cinco linhas do `arca backup`, e de propósito: elas são a
    // releitura de C-3 impressa, e o que elas afirmam vale igual nos dois
    // comandos. Ver [`crate::armar::montar_as_linhas`].
    let mut saida = String::from("\n");
    saida.push_str(&armar::montar_as_linhas(armado));

    // O que se vê do outro lado do reinício é igual nos três comandos que
    // armam, e mora em [`armar::montar_o_que_vem_pela_frente`] desde a E11 —
    // ver lá por que ele existe. Aqui ele vale mais do que nos outros dois:
    // desligar durante o menu numa restauração é desligar antes de a receita
    // começar, e a máquina fica com o disco intacto e o job pendente.
    saida.push_str(armar::montar_o_que_vem_pela_frente());

    saida.push_str("\nAO TERMINAR: remova o SSD antes de religar.\n");

    saida.push_str(match ordem {
        OrdemDeBoot::DispositivoEmPrimeiro => concat!(
            "\n  E REMOVER O SSD NAO E ZELO NESTA OPERACAO. O dispositivo esta em\n",
            "  PRIMEIRO na ordem permanente de boot: enquanto ele estiver conectado,\n",
            "  todo reinicio boota nele — sem boot unico nenhum. Entre o fim da\n",
            "  restauracao e o `arca resultado` o `grub.cfg` continua armado, e um\n",
            "  reinicio nessa janela RESTAURA DE NOVO, por cima do Windows que acabou\n",
            "  de voltar. Foram oito minutos no backup de 22/08.\n",
            "  O ARCA nao mexe na ordem permanente (C-5, ADR-0009): ele lê e avisa.\n",
            "  Remova o SSD ao desligar, religue, e so entao reconecte para\n",
            "  `arca resultado`.\n"
        ),
        OrdemDeBoot::OutraCoisaAntes => concat!(
            "\n  A ordem permanente hoje nao leva ao dispositivo em primeiro, e ela muda\n",
            "  sozinha no ciclo de boot (ADR-0009): depois desta operacao ele tende a\n",
            "  ficar la. Enquanto o `grub.cfg` estiver armado — do fim da restauracao\n",
            "  ate o `arca resultado` — religar com o SSD conectado RESTAURARIA DE\n",
            "  NOVO, por cima do Windows que acabou de voltar. Remover o SSD elimina o\n",
            "  cenario.\n"
        ),
        // C-14, e pelo mesmo motivo do ramo abaixo: a ordem foi lida, e o que
        // ela tem a frente do dispositivo e uma entrada que nao declara alvo.
        // Nao ha como afirmar que ela nao leva a ele.
        OrdemDeBoot::SemAlvoAntes => concat!(
            "\n  UMA ENTRADA A FRENTE DO DISPOSITIVO NAO DIZ PARA ONDE APONTA, e o ARCA\n",
            "  nao supoe que isso queira dizer que ela nao leva a ele (P-28): quem a\n",
            "  resolve e o firmware, no POST, pelo que estiver conectado. Trate como se\n",
            "  levasse: enquanto o `grub.cfg` estiver armado — do fim da restauracao ate\n",
            "  o `arca resultado` — um reinicio com o SSD conectado RESTAURARIA DE NOVO,\n",
            "  por cima do Windows que acabou de voltar. Remova o SSD ao desligar.\n"
        ),
        // "Nao entendi a resposta" nao vira uma afirmacao de seguranca. Sem
        // saber a ordem, o aviso e o **duro** — que e o unico dos dois que nao
        // custa nada estar errado.
        OrdemDeBoot::NaoDeuParaLer => concat!(
            "\n  NAO FOI POSSIVEL LÊ A ORDEM PERMANENTE DE BOOT, e o ARCA nao supoe que\n",
            "  isso queira dizer que ela nao leva ao dispositivo. Trate como se\n",
            "  levasse: enquanto o `grub.cfg` estiver armado — do fim da restauracao\n",
            "  ate o `arca resultado` — um reinicio com o SSD conectado RESTAURARIA DE\n",
            "  NOVO, por cima do Windows que acabou de voltar. Remova o SSD ao\n",
            "  desligar.\n"
        ),
    });

    saida.push_str("\nReiniciando...\n");
    saida
}

// ─────────────────────────── o comando ───────────────────────────

pub fn executar(contexto: &Contexto, nome_pedido: Option<&str>) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let caminho_do_grub = dispositivo.caminho_do_grub()?;

    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;
    let discos = contexto.discos.discos_fisicos()?;

    // C-1, incondicionalmente e como primeiro passo. Vale a mesma licao que a
    // E7 pagou no backup: o desarmar acontece antes de qualquer recusa, e por
    // isso a linha que conta que ele aconteceu tem de sair **antes** de a
    // recusa poder cortar o resto da tela.
    let desarme = if contexto.dry_run {
        None
    } else {
        Some(desarme::executar(
            contexto.arquivos,
            contexto.firmware,
            &caminho_do_grub,
        )?)
    };

    // O cabecalho **antes** de julgar qualquer coisa. Ver [`Cabecalho`]: o
    // desarmar ja aconteceu quando esta linha e impressa, e uma recusa nao
    // pode engolir a noticia de que ele aconteceu.
    print!(
        "{}",
        montar_cabecalho(&Cabecalho {
            dispositivo: &dispositivo,
            desarme: desarme.as_ref(),
            caminho_do_grub: &caminho_do_grub.to_string_lossy(),
        })
    );

    // C-6 e C-10, antes de tudo que custa leitura de arquivo: sao sobre o
    // **dispositivo**, e valem para toda operacao que arma. Ver
    // [`crate::prevoo::julgar_o_dispositivo`] para os dois furos que a primeira
    // versao deste comando tinha por nao as chamar.
    prevoo::julgar_o_dispositivo(&dispositivo, &discos).map_err(Erro::PreVooRecusou)?;

    let oferta = Oferta::de(&pastas);
    let imagem = escolher_a_imagem(contexto, &oferta, nome_pedido)?;

    let retrato = conferir_a_imagem(contexto.arquivos, &raiz_do_vault.join(&imagem.nome))
        .map_err(Erro::RestauracaoRecusada)?;

    // As duas fontes do §4.5, e nao so as imagens.
    //
    // Este comando resolve **dois** nomes pelo oraculo: o do disco de destino
    // (passo 5 de `julgar_o_destino`) e o do proprio dispositivo, na recusa que
    // a revisao da E9 achou. Os dois falam do hardware que esta na mesa
    // **agora**, e e isso que a sondagem descreve — a imagem descreve a maquina
    // de quando o backup foi feito.
    //
    // A lista vem de `backup::fontes_do_oraculo`, e nao de uma copia daqui: uma
    // segunda montagem deixaria o `arca backup` achando o disco por uma fonte
    // que o `arca restore` nao lê, sobre a mesma maquina e no mesmo minuto.
    let listas = super::backup::fontes_do_oraculo(contexto.arquivos, &raiz_do_vault, &pastas);
    let destino = escolher_o_destino(&discos, &dispositivo, &retrato, &listas)
        .map_err(Erro::RestauracaoRecusada)?;

    let nome = Nome::novo(&imagem.nome).map_err(Erro::NomeRecusado)?;

    contexto.registro.info(format!(
        "restauracao de `{nome}` · origem {} ({} setores) · destino {} disco {} ({} setores) · e o disco de origem (R-7)",
        retrato.origem.modelo,
        retrato.origem.setores,
        destino.disco,
        destino.indice,
        destino.setores,
    ));

    print!(
        "{}",
        montar(&Plano {
            imagem,
            retrato: &retrato,
            destino: &destino,
            arma_em_seguida: !contexto.dry_run,
        })
    );

    if contexto.dry_run {
        print!("{}", ensaio_da_receita(contexto, &nome, &destino)?);
        return Ok(());
    }

    // S-2 e R-3: o nome da imagem por extenso, antes de qualquer escrita.
    confirmacao::pedir(contexto, "Digite o nome da imagem para confirmar", &nome)?;

    let armado = armar::executar(
        contexto.arquivos,
        contexto.firmware,
        contexto.entropia,
        contexto.relogio,
        &armar::Pedir {
            dispositivo: &dispositivo,
            operacao: Operacao::Restauracao,
            nome: Some(&nome),
            disco: Some(&destino.disco),
        },
    )?;

    contexto.registro.info(format!(
        "armada restauracao de `{nome}` · selo {} · disco {} · desfecho em {}",
        armado.selo, destino.disco, armado.pasta_do_desfecho
    ));

    // A ordem de boot e lida **depois** de armar, e nao antes: o aviso fala do
    // que acontece a partir de agora, e agora o dispositivo esta armado. Ler
    // antes daria a mesma resposta nesta maquina e responderia outra pergunta.
    print!(
        "{}",
        montar_o_armado(&armado, ordem_de_boot(contexto, &dispositivo))
    );

    contexto.sistema.reiniciar().inspect_err(|_| {
        eprintln!(
            "\nO dispositivo FICOU ARMADO e a maquina nao reiniciou. O proximo reinicio,\n\
             seja qual for a causa, vai bootar no dispositivo e APAGAR o disco de destino.\n\
             Para desfazer:  arca desarmar"
        );
    })
}

/// O que a ordem permanente diz sobre o proximo reinicio.
///
/// Tres estados, e nao dois. "Nao consegui lê" **nao vira** "o dispositivo nao
/// esta a frente": essa e a resposta tranquilizadora, e transformar uma falha
/// de leitura nela e o erro que C-3 existe para nao cometer — e que o ADR-0009
/// nomeou de novo ao pôr a guarda de `viu_o_gerenciador` no `arca status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrdemDeBoot {
    /// Alguma entrada que leva ao `ARCABOOT` esta em **primeiro**: todo
    /// reinicio boota no dispositivo, sem boot unico nenhum.
    DispositivoEmPrimeiro,

    /// Ha alguma coisa antes dele, ou ele nao esta na ordem.
    OutraCoisaAntes,

    /// **Ha uma entrada a frente dele que nao diz para onde aponta** (P-28).
    /// Nao e "outra coisa antes": ali se sabe o que vem antes, e aqui nao.
    SemAlvoAntes,

    /// O `{fwbootmgr}` nao se deixou lê.
    NaoDeuParaLer,
}

/// Lê a ordem permanente e diz o que ela significa (C-5: **lê e nunca
/// escreve**).
///
/// Uma chamada so, e ao `firmware` — nao ao `{fwbootmgr}`. O `bcdedit /enum
/// firmware` traz o bloco do gerenciador **junto** das entradas, e e assim que
/// o `arca status` lê desde a E2. Duas chamadas dariam duas leituras de
/// momentos diferentes coladas numa so, que e exatamente a armadilha do §11.
fn ordem_de_boot(contexto: &Contexto, dispositivo: &Dispositivo) -> OrdemDeBoot {
    let Ok(texto) = contexto.firmware.enumerar(FIRMWARE) else {
        return OrdemDeBoot::NaoDeuParaLer;
    };
    let leitura = crate::firmware::ler(&texto);
    if !leitura.viu_o_gerenciador {
        return OrdemDeBoot::NaoDeuParaLer;
    }

    let lugar = status::lugar_do_dispositivo(&leitura, dispositivo);
    if lugar.em_primeiro() {
        OrdemDeBoot::DispositivoEmPrimeiro
    } else if lugar.sem_alvo_a_frente.is_some() {
        // P-28: uma entrada sem `device` a frente do dispositivo nao autoriza
        // o aviso brando, que **afirma** que a ordem nao leva a ele.
        OrdemDeBoot::SemAlvoAntes
    } else {
        OrdemDeBoot::OutraCoisaAntes
    }
}

/// A escolha da imagem: pelo nome dado na linha de comando, ou pelo indice.
///
/// # Por que as duas leituras do §6.1 ficam
///
/// A tela do documento pede um indice (`Qual restaurar? 2`) e depois o nome por
/// extenso (R-3). Parece redundancia e nao e: sao dois atos diferentes.
/// **Escolher** e apontar numa lista, e um numero e a forma mais curta de
/// apontar. **Confirmar** e comprometer-se, e S-2 existe justamente para que
/// isso custe o trabalho de lê e digitar o nome inteiro. Trocar a segunda pela
/// primeira faria um `2` apagar um disco.
///
/// O nome na linha de comando e o atalho de quem ja leu a lista, e ele e o que
/// torna `arca restore <nome> --dry-run` utilizavel sem console.
fn escolher_a_imagem<'a>(
    contexto: &Contexto,
    oferta: &Oferta<'a>,
    nome_pedido: Option<&str>,
) -> Resultado<&'a Pasta> {
    if let Some(pedido) = nome_pedido {
        // L-2 tem de responder antes de "nao existe": uma pasta que esta la e
        // e residuo merece a mensagem que diz por que ela nao e oferecida, e
        // nao um "nao ha imagem com esse nome" que manda a pessoa procurar um
        // erro de digitacao.
        if let Some(residuo) = oferta
            .residuos
            .iter()
            .find(|pasta| pasta.nome.eq_ignore_ascii_case(pedido))
        {
            return Err(Erro::RestauracaoRecusada(
                RecusaDaRestauracao::ImagemEResiduo {
                    nome: residuo.nome.clone(),
                },
            ));
        }

        // E a mesma regra para o nome que nao cabe numa receita: a mensagem
        // que diz **por que** vem antes da que manda procurar erro de
        // digitacao.
        if let Some((pasta, porque)) = oferta
            .sem_nome_valido
            .iter()
            .find(|(pasta, _)| pasta.nome.eq_ignore_ascii_case(pedido))
        {
            return Err(Erro::RestauracaoRecusada(
                RecusaDaRestauracao::NomeNaoCabeNaReceita {
                    nome: pasta.nome.clone(),
                    porque: porque.to_string(),
                },
            ));
        }

        return oferta
            .imagens
            .iter()
            .copied()
            .find(|pasta| pasta.nome.eq_ignore_ascii_case(pedido))
            .ok_or_else(|| {
                Erro::RestauracaoRecusada(RecusaDaRestauracao::ImagemDesconhecida {
                    nome: pedido.to_string(),
                })
            });
    }

    if oferta.imagens.is_empty() {
        return Err(Erro::RestauracaoRecusada(
            RecusaDaRestauracao::NadaAOferecer {
                residuos: oferta.residuos.len(),
                sem_nome_valido: oferta.sem_nome_valido.len(),
            },
        ));
    }

    use std::io::Write;
    print!("{}", montar_a_lista(oferta));
    print!("\nQual restaurar? ");
    let _ = std::io::stdout().flush();

    // Um console que nao se deixou lê **sobe como erro de leitura**, e nao
    // como escolha invalida. Sao coisas diferentes: uma diz "voce digitou
    // errado" e a outra diz "nao consegui ouvir", e trocar a segunda pela
    // primeira e a mesma familia de erro que o §5.5 nomeia — "nao consegui
    // olhar" nao vira veredito.
    let digitado = contexto.console.ler_linha()?;
    println!();

    // Uma tentativa, e nao um laco — a mesma regra da confirmacao. Quem errou
    // repete o comando, que ate aqui nao armou nada.
    let escolhido = digitado.trim();
    escolher_pelo_indice(oferta, escolhido).ok_or_else(|| {
        Erro::RestauracaoRecusada(RecusaDaRestauracao::EscolhaInvalida {
            digitado: escolhido.to_string(),
            quantas: oferta.imagens.len(),
        })
    })
}

/// A imagem de indice `1..=n` da lista, ou `None`.
///
/// Separada da leitura do console para que o julgamento tenha teste sem duplo:
/// e ele que decide qual disco vai ser apagado a partir de um numero digitado.
fn escolher_pelo_indice<'a>(oferta: &Oferta<'a>, digitado: &str) -> Option<&'a Pasta> {
    // `parse::<usize>` recusa `-1` e `+2` de graca, e `1..=n` fecha o resto. O
    // `0` cai fora por baixo, e e o erro mais provavel de quem conta de zero.
    digitado
        .parse::<usize>()
        .ok()
        .filter(|numero| *numero >= 1 && *numero <= oferta.imagens.len())
        .map(|numero| oferta.imagens[numero - 1])
}

/// A receita inteira, so no `--dry-run`.
fn ensaio_da_receita(contexto: &Contexto, nome: &Nome, destino: &Destino) -> Resultado<String> {
    // O selo de verdade nasce ao armar. Este e de ensaio, e a saida o diz.
    let receita = Receita::montar(&Pedido {
        operacao: Operacao::Restauracao,
        nome: Some(nome.clone()),
        disco: Some(destino.disco.clone()),
        selo: Selo::de_ensaio(),
    })
    .map_err(Erro::ReceitaRecusada)?;

    contexto.registro.info(format!(
        "ensaio de restauracao `{nome}` · disco {} · receita de {} caracteres · validada por C-2",
        destino.disco,
        receita.comando().chars().count()
    ));

    Ok(format!(
        "\nReceita de restauracao — e esta que o comando sem --dry-run armaria\n\n  \
         O que o Clonezilla executa:\n\n    {}\n\n  \
         Como entra na linha do grub.cfg:\n\n    {}\n\
         \nO selo acima e de ensaio (so zeros), e por isso esta receita nao serviria: o\n\
         de verdade nasce **ao armar**, de uma fonte de entropia do sistema.\n",
        receita.comando(),
        receita.parametros_do_grub()
    ))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{DiscosDeMentira, momento};
    use crate::portas::{Medida, TipoDeMidia};

    /// O `nvme0n1-gpt.sgdisk` de `E:\2026-08-22_Apps`, copiado do dispositivo.
    const SGDISK: &str = "\
Disk /dev/nvme0n1: 976773168 sectors, 465.8 GiB
Model: KINGSTON SNV3S500G
Sector size (logical/physical): 512/512 bytes
";

    /// O `blkdev.list` da mesma imagem, com as larguras de coluna do arquivo.
    const BLKDEV: &str = concat!(
        "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
        "sda       sda         238.5G disk                                               KGSSE100256\n",
        "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
    );

    const COMANDO: &str = "/usr/sbin/ocs-sr -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true savedisk 2026-08-22_Apps nvme0n1";

    fn imagem(nome: &str, veredito: Option<Veredito>) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes: 42_614_112_256,
            modificado_em: Some(momento("2026-08-22T09:14:02")),
            especie: Especie::Imagem { veredito },
        }
    }

    fn residuo(nome: &str) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes: 512,
            modificado_em: Some(momento("2026-08-22T03:11:00")),
            especie: Especie::Residuo,
        }
    }

    fn retrato() -> Retrato {
        Retrato {
            disco: "nvme0n1".to_string(),
            origem: gpt::ler("nvme0n1-gpt.sgdisk", SGDISK).expect("sgdisk legivel"),
            modelo_no_blkdev: Some("KINGSTON SNV3S500G".to_string()),
            comando_que_criou: Some(COMANDO.to_string()),
        }
    }

    fn listas() -> Vec<blkdev::Lista> {
        vec![blkdev::Lista {
            fonte: blkdev::Fonte::Imagem("2026-08-22_Apps".to_string()),
            texto: BLKDEV.to_string(),
        }]
    }

    fn dispositivo_conectado() -> Dispositivo {
        dispositivo::encontrar(&DiscosDeMentira::com_dispositivo()).unwrap()
    }

    fn escolher(discos: &[DiscoFisico]) -> Result<Destino, RecusaDaRestauracao> {
        escolher_o_destino(discos, &dispositivo_conectado(), &retrato(), &listas())
    }

    // ─────────────────── R-7 e as duas reguas ───────────────────

    #[test]
    fn o_disco_desta_maquina_e_reconhecido_como_o_de_origem() {
        // O teste que a E9 existiu para escrever, e que o ADR-0015 endureceu.
        // Com o destino medido pelo `Win32_DiskDrive` (500.105.249.280) e a
        // origem pela GPT (500.107.862.016), este disco nao seria reconhecido
        // como ele proprio — 5.103 setores de diferenca, que e menos de um
        // cilindro CHS.
        let destino = escolher(&crate::duplos::discos_desta_mesa())
            .expect("o disco de origem tem de ser reconhecido como destino");

        assert_eq!(destino.disco.como_texto(), "nvme0n1");
        assert_eq!(destino.setores, 976_773_168);
        assert_eq!(destino.setores, retrato().origem.setores);
    }

    #[test]
    fn um_destino_medido_pela_regua_errada_seria_recusado() {
        // O mesmo disco, com a medida do `Win32_DiskDrive` no lugar da do
        // `MSFT_Disk`. E o defeito que o ADR-0010 nomeia, e ele **falha** —
        // que e a prova de que a fonte importa, e nao uma questao de gosto.
        // Com o `==` do ADR-0015 ele falha ainda mais cedo: a diferenca de
        // 5.103 setores agora e "nao e este disco", e nao "nao cabe".
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[0].medida = Some(Medida {
            bytes: discos[0].tamanho_bytes,
            bytes_por_setor: 512,
        });

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::NaoEODiscoDeOrigem {
                origem_setores: 976_773_168,
                destino_setores: 976_768_065,
                bytes_por_setor: 512,
            })
        );
    }

    #[test]
    fn um_disco_menor_com_o_modelo_da_origem_e_recusado() {
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[0].medida = Some(Medida {
            bytes: 256_060_514_304,
            bytes_por_setor: 512,
        });

        assert!(matches!(
            escolher(&discos),
            Err(RecusaDaRestauracao::NaoEODiscoDeOrigem { .. })
        ));
    }

    #[test]
    fn um_disco_maior_com_o_modelo_da_origem_tambem_e_recusado() {
        // **Este teste mudou de sentido no ADR-0015**, e a mudanca e o ponto
        // da decisao. Ate a E9 ele se chamava
        // `um_destino_maior_passa_e_a_sobra_aparece` e cobrava que a
        // restauracao seguisse, porque R-7 perguntava *"cabe?"*. Agora ela
        // pergunta *"e ele mesmo?"*, e um disco de 1 TB com o modelo do de
        // 500 GB e **outro disco** — provavelmente um gemeo maior da mesma
        // linha, que e exatamente o caso que a igualdade pega e o `>=` deixava
        // passar.
        //
        // Sobrar espaco nao e permissao para nada: o unico destino valido e a
        // origem.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[0].medida = Some(Medida {
            bytes: 1_000_204_886_016,
            bytes_por_setor: 512,
        });

        let recusa = escolher(&discos).expect_err("um disco maior nao e o de origem");
        assert!(matches!(
            recusa,
            RecusaDaRestauracao::NaoEODiscoDeOrigem { .. }
        ));

        // E a mensagem diz **por que** sobrar espaco nao resolve, em vez de
        // deixar quem lê achar que trocou por um SSD maior e ficou melhor.
        assert!(
            recusa
                .to_string()
                .contains("sobrar espaco nao e permissao para restaurar aqui"),
            "a mensagem nao trata o caso do disco maior: {recusa}"
        );
    }

    #[test]
    fn sem_medida_do_destino_e_recusa_e_nao_um_sim() {
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[0].medida = None;

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::SemMedidaDoDestino {
                modelo: "KINGSTON SNV3S500G".to_string()
            })
        );
    }

    #[test]
    fn setor_logico_diferente_e_recusa() {
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[0].medida = Some(Medida {
            bytes: 500_107_862_016,
            bytes_por_setor: 4096,
        });

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::SetorDivergente {
                origem: 512,
                destino: 4096
            })
        );
    }

    // ─────── C-6 e C-10 valem aqui tambem, e a E9 nao as tinha ───────

    #[test]
    fn o_restore_recusa_midia_removivel_antes_da_confirmacao() {
        // C-6. O `armar` pegaria a rejeicao silenciosa na releitura do
        // `device`, mas so **depois** de a pessoa digitar o nome da imagem que
        // vai apagar um disco. O `MediaType` do WMI sabe antes.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1].tipo_de_midia = TipoDeMidia::Removivel;

        assert!(matches!(
            prevoo::julgar_o_dispositivo(&dispositivo_conectado(), &discos),
            Err(crate::prevoo::RecusaDoPreVoo::MidiaRemovivel)
        ));
    }

    #[test]
    fn o_restore_recusa_o_dispositivo_partido() {
        // C-10. Com o `ARCAVAULT` num disco e o `ARCABOOT` noutro, o
        // `estado.json` iria para um e o desfecho para o outro — e a colheita
        // procuraria o desfecho deste job no lugar errado.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1].letras = vec!['E'];
        discos.push(DiscoFisico {
            indice: 2,
            modelo: "OUTRO SSD".to_string(),
            tamanho_bytes: 1_000,
            medida: None,
            em_uso_bytes: 0,
            tipo_de_midia: TipoDeMidia::DiscoExterno,
            letras: vec!['R'],
        });

        assert!(matches!(
            prevoo::julgar_o_dispositivo(&dispositivo_conectado(), &discos),
            Err(crate::prevoo::RecusaDoPreVoo::DispositivoPartido { .. })
        ));
    }

    #[test]
    fn esta_mesa_passa_pelas_duas() {
        assert!(
            prevoo::julgar_o_dispositivo(
                &dispositivo_conectado(),
                &crate::duplos::discos_desta_mesa()
            )
            .is_ok(),
            "o dispositivo desta mesa tem de passar, senao os dois acima nao provam nada"
        );
    }

    // ─────────────────── o dispositivo nunca e destino ───────────────────

    /// As letras do dispositivo desta mesa, como [`escolher_o_destino`] as
    /// monta antes de julgar.
    fn letras_do_dispositivo() -> Vec<char> {
        let dispositivo = dispositivo_conectado();
        dispositivo
            .vault
            .letra
            .into_iter()
            .chain(dispositivo.boot.as_ref().and_then(|boot| boot.letra))
            .collect()
    }

    /// Julga um disco **ja escolhido**, que e o que a busca por modelo nunca
    /// entrega quando esse disco e o dispositivo. Ver [`julgar_o_destino`].
    fn julgar(
        escolhido: &DiscoFisico,
        discos: &[DiscoFisico],
    ) -> Result<Destino, RecusaDaRestauracao> {
        julgar_o_destino(
            escolhido,
            discos,
            &letras_do_dispositivo(),
            &retrato(),
            &listas(),
        )
    }

    #[test]
    fn a_busca_por_modelo_nunca_entrega_o_dispositivo() {
        // A primeira barreira de R-8, e a que roda no caminho normal: o
        // dispositivo esta **fora** dos candidatos. Nao ha entrada por onde
        // ele chegue ao julgamento — e por isso a segunda barreira precisa de
        // teste proprio, logo abaixo.
        let discos = crate::duplos::discos_desta_mesa();
        let destino = escolher(&discos).expect("o disco de origem serve");

        assert_eq!(destino.indice, 0, "o escolhido tem de ser o disco interno");
        assert!(
            !destino.disco.como_texto().eq("sda"),
            "o dispositivo virou destino"
        );
    }

    #[test]
    fn o_proprio_dispositivo_nunca_e_destino() {
        // A segunda barreira, alcancada por [`julgar_o_destino`]. O disco 1
        // desta mesa e o SSD externo, com o ARCAVAULT e o ARCABOOT: restaurar
        // nele apagaria o Clonezilla que esta executando a receita e a imagem
        // que ela esta lendo.
        //
        // Ela e redundante desde o ADR-0015 e fica de proposito — a revisao da
        // E9 mostrou que uma recusa dura pode ter contorno por acidente de
        // modelo, e uma segunda barreira custa nada.
        let discos = crate::duplos::discos_desta_mesa();

        assert_eq!(
            julgar(&discos[1], &discos),
            Err(RecusaDaRestauracao::DestinoEODispositivo {
                modelo: "KGSSE100 256 SCSI Disk Device".to_string(),
                letras: "E: e R:".to_string(),
            })
        );
    }

    #[test]
    fn a_recusa_do_dispositivo_vem_antes_de_qualquer_outra() {
        // O disco do dispositivo tambem tem tamanho diferente do da origem, e
        // tambem nao tem nome Linux util aqui. Se a ordem das recusas mudasse,
        // a mensagem passaria a falar de tamanho — e quem lesse acharia que
        // outro disco resolveria.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1].medida = None;

        assert!(matches!(
            julgar(&discos[1].clone(), &discos),
            Err(RecusaDaRestauracao::DestinoEODispositivo { .. })
        ));
    }

    #[test]
    fn um_gemeo_do_dispositivo_nao_passa_pela_recusa_por_letra() {
        // **O defeito mais grave da etapa**, achado pela revisao de codigo.
        //
        // A recusa do dispositivo e por **letra do Windows**; o nome que vai
        // para a receita e do **Linux**, e sai de um casamento por MODELO nos
        // `blkdev.list`. Sao dois canais de identidade diferentes, e o vao
        // entre eles apagava o dispositivo.
        //
        // O caso: um segundo disco, interno, do **mesmo modelo** do
        // dispositivo. Ele passa pela recusa por letra (as letras sao outras),
        // passa pela medida e pelo tamanho — e o passo 5 resolve o modelo no
        // `blkdev.list`, onde o unico disco daquele modelo e o dispositivo,
        // que ali se chama `sda`. A receita sairia `restoredisk <imagem> sda`.
        //
        // Na E9 o caminho ate aqui era `--destino 2`. Sem a flag, o teste
        // julga o candidato direto — o furo continua sendo o mesmo, e a
        // barreira que o fecha continua no mesmo lugar.
        let mut discos = crate::duplos::discos_desta_mesa();
        let gemeo = DiscoFisico {
            indice: 2,
            modelo: "KGSSE100 256 SCSI Disk Device".to_string(),
            tamanho_bytes: 500_105_249_280,
            medida: Some(Medida {
                bytes: 500_107_862_016,
                bytes_por_setor: 512,
            }),
            em_uso_bytes: 0,
            tipo_de_midia: TipoDeMidia::DiscoFixo,
            letras: vec!['D'],
        };
        discos.push(gemeo.clone());

        assert_eq!(
            julgar(&gemeo, &discos),
            Err(RecusaDaRestauracao::DestinoResolveNoDispositivo {
                disco: "sda".to_string(),
                modelo: "KGSSE100 256 SCSI Disk Device".to_string(),
            }),
            "o gemeo do dispositivo resolveria em `sda` na receita, e `sda` e o dispositivo"
        );
    }

    #[test]
    fn o_dispositivo_com_o_modelo_da_origem_nao_vira_destino_ambiguo() {
        // O outro lado do mesmo furo. Com o dispositivo tendo o modelo do
        // disco de origem, sem o filtro do `casam` haveria **dois** candidatos
        // e o comando recusaria por `DestinoAmbiguo` — que hoje e recusa
        // terminal, e mandaria desconectar um disco que ja e o certo.
        //
        // Com o filtro, a recusa passa a ser a que diz o problema de verdade:
        // os dois discos resolvem no **mesmo nome do Linux**, porque o
        // `blkdev.list` casa por modelo e nada mais.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1].modelo = "KINGSTON SNV3S500G".to_string();

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::DestinoResolveNoDispositivo {
                disco: "nvme0n1".to_string(),
                modelo: "KINGSTON SNV3S500G".to_string(),
            }),
            "a recusa tem de dizer o problema real, e nao mandar nomear um destino"
        );
    }

    #[test]
    fn o_caminho_normal_desta_mesa_diz_que_e_o_disco_de_origem() {
        // A rede embaixo dos dois testes acima: nesta mesa, com o dispositivo
        // sendo um `KGSSE100` e a origem um `KINGSTON`, o destino sai como o
        // disco de origem. Sem este teste, os dois acima passariam com um
        // `escolher` que recusasse tudo.
        let destino =
            escolher(&crate::duplos::discos_desta_mesa()).expect("o caminho normal tem de passar");

        assert_eq!(destino.indice, 0);
        assert_eq!(destino.disco.como_texto(), "nvme0n1");
        assert_eq!(destino.setores, retrato().origem.setores);
    }

    #[test]
    fn sem_disco_do_modelo_da_origem_o_arca_nao_escolhe() {
        let discos = vec![DiscoFisico {
            indice: 0,
            modelo: "OUTRO MODELO".to_string(),
            tamanho_bytes: 500_105_249_280,
            medida: Some(Medida {
                bytes: 500_107_862_016,
                bytes_por_setor: 512,
            }),
            em_uso_bytes: 0,
            tipo_de_midia: TipoDeMidia::DiscoFixo,
            letras: vec!['C'],
        }];

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::SemDestinoObvio {
                modelo: "KINGSTON SNV3S500G".to_string()
            })
        );
    }

    #[test]
    fn dois_discos_do_mesmo_modelo_sao_ambiguos() {
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1] = DiscoFisico {
            indice: 2,
            modelo: "KINGSTON SNV3S500G".to_string(),
            letras: vec!['D'],
            ..discos[0].clone()
        };

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::DestinoAmbiguo {
                modelo: "KINGSTON SNV3S500G".to_string(),
                quantos: 2
            })
        );
    }

    #[test]
    fn um_destino_sem_nome_no_blkdev_nao_entra_na_receita() {
        // §4.5: o nome do Linux sai de uma medicao, e nunca de derivacao. Um
        // disco que nenhuma imagem viu nao tem nome, e a receita nao o nomeia.
        //
        // O cenario: um disco na mesa com o modelo que a **imagem** registra
        // como origem, que nenhum `blkdev.list` viu. Acontece quando a imagem
        // veio de outro dispositivo, com outra colecao de listas.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[0].modelo = "DISCO NOVO EM FOLHA".to_string();

        let mut retrato_de_outro_dispositivo = retrato();
        retrato_de_outro_dispositivo.origem.modelo = "DISCO NOVO EM FOLHA".to_string();

        assert!(matches!(
            escolher_o_destino(
                &discos,
                &dispositivo_conectado(),
                &retrato_de_outro_dispositivo,
                &listas(),
            ),
            Err(RecusaDaRestauracao::SemNomeDoDestino(_))
        ));
    }

    // ─────────────────── R-2: a conferencia da imagem ───────────────────

    fn arquivos_da_imagem() -> crate::duplos::ArquivosEmMemoria {
        crate::duplos::ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-22_Apps\disk", "nvme0n1\n")
            .com(r"E:\2026-08-22_Apps\nvme0n1-gpt.sgdisk", SGDISK)
            .com(r"E:\2026-08-22_Apps\blkdev.list", BLKDEV)
            .com(r"E:\2026-08-22_Apps\Info-saved-by-cmd.txt", COMANDO)
    }

    #[test]
    fn a_imagem_desta_mesa_passa_pela_conferencia() {
        let conferido = conferir_a_imagem(&arquivos_da_imagem(), Path::new(r"E:\2026-08-22_Apps"))
            .expect("a imagem do dispositivo tem de passar");

        assert_eq!(conferido.disco, "nvme0n1");
        assert_eq!(conferido.origem.setores, 976_773_168);
        assert_eq!(
            conferido.modelo_no_blkdev.as_deref(),
            Some("KINGSTON SNV3S500G")
        );
        assert_eq!(conferido.comando_que_criou.as_deref(), Some(COMANDO));
    }

    #[test]
    fn sem_o_arquivo_disk_a_restauracao_para() {
        let arquivos =
            crate::duplos::ArquivosEmMemoria::novo().com(r"E:\imagem\nvme0n1-gpt.sgdisk", SGDISK);

        assert!(matches!(
            conferir_a_imagem(&arquivos, Path::new(r"E:\imagem")),
            Err(RecusaDaRestauracao::SemArquivoDaOrigem { .. })
        ));
    }

    #[test]
    fn sem_o_sgdisk_nao_ha_medida_e_a_restauracao_para() {
        let arquivos = crate::duplos::ArquivosEmMemoria::novo()
            .com(r"E:\imagem\disk", "nvme0n1\n")
            .com(r"E:\imagem\blkdev.list", BLKDEV);

        assert!(matches!(
            conferir_a_imagem(&arquivos, Path::new(r"E:\imagem")),
            Err(RecusaDaRestauracao::SemArquivoDaOrigem { arquivo, .. })
                if arquivo == "nvme0n1-gpt.sgdisk"
        ));
    }

    #[test]
    fn um_disk_que_nao_e_nome_de_disco_do_linux_e_recusado() {
        // O `disk` vira parte de um caminho de arquivo — `<disco>-gpt.sgdisk`
        // —, e a pasta da imagem e copiada de onde o usuario quiser. Um `disk`
        // com barra ou `..` faria a leitura sair de dentro dela.
        for bruto in ["../../etc", r"..\..\x", "nvme0n1/", "NVME0N1", "", "  "] {
            let arquivos = crate::duplos::ArquivosEmMemoria::novo()
                .com(r"E:\imagem\disk", bruto)
                .com(r"E:\imagem\nvme0n1-gpt.sgdisk", SGDISK);

            assert!(
                matches!(
                    conferir_a_imagem(&arquivos, Path::new(r"E:\imagem")),
                    Err(RecusaDaRestauracao::SemArquivoDaOrigem { .. })
                ),
                "`{bruto}` nao pode passar por nome de disco"
            );
        }
    }

    #[test]
    fn um_gemeo_do_mesmo_tamanho_nao_e_desempatado_pelo_tamanho() {
        // Ate a E9 este teste guardava o campo `e_o_da_origem`, que decidia se
        // a tela avisava sobre `-iefi` e `bcdboot`. O campo saiu com o
        // ADR-0015, e o que ele protegia continua de pe e ficou mais duro:
        // **com um gemeo na maquina, o ARCA nao escolhe nenhum dos dois.**
        //
        // A igualdade de setores nao desempata — o gemeo aqui tem exatamente a
        // mesma medida —, e e por isso que a ambiguidade e recusa terminal e
        // nao uma pergunta ao usuario: nao ha nada nesta mesa contra o que
        // conferir a resposta.
        let mut discos = crate::duplos::discos_desta_mesa();
        discos[1] = DiscoFisico {
            indice: 2,
            modelo: "KINGSTON SNV3S500G".to_string(),
            letras: vec!['D'],
            ..discos[0].clone()
        };

        assert_eq!(
            escolher(&discos),
            Err(RecusaDaRestauracao::DestinoAmbiguo {
                modelo: "KINGSTON SNV3S500G".to_string(),
                quantos: 2
            })
        );

        // E com um disco so daquele modelo, o caminho normal segue.
        assert!(escolher(&crate::duplos::discos_desta_mesa()).is_ok());
    }

    #[test]
    fn duas_fontes_da_imagem_discordando_do_disco_e_recusa() {
        let arquivos = crate::duplos::ArquivosEmMemoria::novo()
            .com(r"E:\imagem\disk", "sda\n")
            .com(
                r"E:\imagem\sda-gpt.sgdisk",
                "Disk /dev/nvme0n1: 976773168 sectors, 465.8 GiB\nModel: KINGSTON SNV3S500G\nSector size (logical/physical): 512/512 bytes\n",
            );

        assert_eq!(
            conferir_a_imagem(&arquivos, Path::new(r"E:\imagem")),
            Err(RecusaDaRestauracao::ImagemInconsistente {
                disse: "sda".to_string(),
                e_disse: "nvme0n1".to_string(),
            })
        );
    }

    #[test]
    fn duas_fontes_da_imagem_discordando_do_modelo_e_recusa() {
        let arquivos = crate::duplos::ArquivosEmMemoria::novo()
            .com(r"E:\imagem\disk", "nvme0n1\n")
            .com(r"E:\imagem\nvme0n1-gpt.sgdisk", SGDISK)
            .com(
                r"E:\imagem\blkdev.list",
                concat!(
"KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
"nvme0n1   nvme0n1     465.8G disk                                               OUTRA COISA\n",
                ),
            );

        assert!(matches!(
            conferir_a_imagem(&arquivos, Path::new(r"E:\imagem")),
            Err(RecusaDaRestauracao::ImagemInconsistente { .. })
        ));
    }

    #[test]
    fn o_blkdev_e_o_comando_sao_informativos_e_nao_derrubam() {
        let arquivos = crate::duplos::ArquivosEmMemoria::novo()
            .com(r"E:\imagem\disk", "nvme0n1\n")
            .com(r"E:\imagem\nvme0n1-gpt.sgdisk", SGDISK);

        let conferido =
            conferir_a_imagem(&arquivos, Path::new(r"E:\imagem")).expect("os dois sao opcionais");
        assert_eq!(conferido.modelo_no_blkdev, None);
        assert_eq!(conferido.comando_que_criou, None);
    }

    // ─────────────────── R-1 e L-2: a lista ───────────────────

    #[test]
    fn a_lista_numera_so_imagens_e_nomeia_os_residuos() {
        let pastas = vec![
            imagem("2026-08-21_WindowsCompleto", Some(Veredito::Aprovada)),
            imagem("2026-08-22_Apps", Some(Veredito::Aprovada)),
            residuo("2026-08-22_Interrompido"),
        ];
        let saida = montar_a_lista(&Oferta::de(&pastas));

        assert!(saida.contains("[1] 2026-08-21_WindowsCompleto"), "{saida}");
        assert!(saida.contains("[2] 2026-08-22_Apps"), "{saida}");
        assert!(
            !saida.contains("[3]"),
            "residuo nao pode ganhar indice (L-2):\n{saida}"
        );
        assert!(
            saida.contains("Sem numero, e nao oferecido (L-2): 2026-08-22_Interrompido"),
            "o residuo tem de aparecer dito:\n{saida}"
        );
    }

    #[test]
    fn o_indice_nao_desliza_quando_ha_residuo_antes() {
        // A pasta de indice 1 tem de ser a primeira **imagem**, e nao a
        // primeira pasta. Se o residuo ocupasse um lugar, quem digitasse `1`
        // olhando a lista escolheria a imagem errada.
        let pastas = vec![residuo("AAA_residuo"), imagem("ZZZ_imagem", None)];
        let oferta = Oferta::de(&pastas);

        assert_eq!(oferta.imagens.len(), 1);
        assert_eq!(oferta.imagens[0].nome, "ZZZ_imagem");
        assert!(montar_a_lista(&oferta).contains("[1] ZZZ_imagem"));
    }

    #[test]
    fn a_lista_marca_reprovada_em_maiuscula() {
        let pastas = vec![imagem("2026-08-22_Apps", Some(Veredito::Reprovada))];
        assert!(montar_a_lista(&Oferta::de(&pastas)).contains("· REPROVADA"));
    }

    #[test]
    fn imagem_com_nome_que_nao_cabe_na_receita_nao_e_oferecida() {
        // Achado relendo `Nome::novo` com a restauracao na mao: o nome da
        // imagem **vai para a receita**, e uma pasta com espaco no nome
        // quebraria a string do `bash -c`. Ate aqui ela seria numerada, e
        // recusada depois da escolha — com a mensagem de B-2, que fala em
        // "escolha outro nome" numa tela onde o nome nao esta sendo escolhido.
        let pastas = vec![
            imagem("Backup Antigo", Some(Veredito::Aprovada)),
            imagem("2026-08-22_Apps", Some(Veredito::Aprovada)),
        ];
        let oferta = Oferta::de(&pastas);

        assert_eq!(oferta.imagens.len(), 1);
        assert_eq!(oferta.imagens[0].nome, "2026-08-22_Apps");
        assert_eq!(oferta.sem_nome_valido.len(), 1);

        let saida = montar_a_lista(&oferta);
        assert!(saida.contains("[1] 2026-08-22_Apps"), "{saida}");
        assert!(!saida.contains("[2]"), "{saida}");
        assert!(
            saida.contains("o ARCA nao pode restaurar: `Backup Antigo`"),
            "a pasta tem de aparecer dita, e com o motivo:\n{saida}"
        );
    }

    #[test]
    fn o_indice_escolhe_de_um_ate_n_e_recusa_o_resto() {
        let pastas = vec![
            imagem("primeira", None),
            residuo("no_meio_do_alfabeto"),
            imagem("segunda", None),
        ];
        let oferta = Oferta::de(&pastas);

        assert_eq!(
            escolher_pelo_indice(&oferta, "1").map(|p| p.nome.as_str()),
            Some("primeira")
        );
        assert_eq!(
            escolher_pelo_indice(&oferta, "2").map(|p| p.nome.as_str()),
            Some("segunda")
        );

        // O `0` e o erro mais provavel de quem conta de zero, e o `3` e o
        // indice que o residuo teria se ele contasse. Nenhum dos dois escolhe.
        for fora in [
            "0",
            "3",
            "-1",
            "",
            " ",
            "um",
            "1.0",
            "1 2",
            "9999999999999999999999",
        ] {
            assert!(
                escolher_pelo_indice(&oferta, fora).is_none(),
                "`{fora}` nao pode escolher um disco para apagar"
            );
        }

        // Espaco das pontas nao atrapalha: um Enter deixa `\r\n` atras.
        assert!(escolher_pelo_indice(&oferta, "2".trim()).is_some());
    }

    // ─────────────────── a tela do §6.1 ───────────────────

    fn plano_com<'a>(imagem: &'a Pasta, destino: &'a Destino, retrato: &'a Retrato) -> Plano<'a> {
        Plano {
            imagem,
            retrato,
            destino,
            arma_em_seguida: true,
        }
    }

    #[test]
    fn a_tela_traz_os_dois_discos_com_a_mesma_regua() {
        let pasta = imagem("2026-08-22_Apps", Some(Veredito::Aprovada));
        let destino = escolher(&crate::duplos::discos_desta_mesa()).unwrap();
        let retrato = retrato();
        let saida = montar(&plano_com(&pasta, &destino, &retrato));

        // Os dois lados com o mesmo numero de setores: e a comparacao de R-7
        // impressa, e nao o veredito dela resumido.
        assert_eq!(
            saida.matches("976773168 setores de 512 B").count(),
            2,
            "origem e destino tem de sair na mesma regua:\n{saida}"
        );
        assert!(saida.contains("ATENCAO: a restauracao APAGA o disco de destino"));
        assert!(
            saida.contains("Imagem criada por"),
            "a procedencia da imagem tem de aparecer:\n{saida}"
        );
    }

    #[test]
    fn a_tela_nao_repete_o_498_7_gb_de_outra_maquina() {
        // O §6.1 do PRD trazia `Destino: KINGSTON SNV3S500G · 498,7 GB`, que e
        // o tamanho da particao `C:` — o mesmo numero medido na coisa errada
        // que a E6 corrigiu no §5.2, sobrevivendo aqui. Esta e a sexta vez do
        // padrao, e ela tem teste.
        let pasta = imagem("2026-08-22_Apps", None);
        let destino = escolher(&crate::duplos::discos_desta_mesa()).unwrap();
        let retrato = retrato();
        let saida = montar(&plano_com(&pasta, &destino, &retrato));

        assert!(!saida.contains("498,7"), "{saida}");
        assert!(saida.contains("465,8 GB"), "{saida}");
    }

    #[test]
    fn a_tela_afirma_identidade_e_nao_capacidade() {
        // **Este teste mudou de sentido no ADR-0015.** Ele se chamava
        // `a_tela_avisa_quando_o_destino_nao_e_o_disco_de_origem` e cobrava o
        // paragrafo sobre `-iefi` e `bcdboot` num destino divergente. Nao ha
        // destino divergente: a tela so e alcancada quando o destino **e** a
        // origem, e o que ela tem de dizer e isso.
        let pasta = imagem("2026-08-22_Apps", Some(Veredito::Aprovada));
        let destino = escolher(&crate::duplos::discos_desta_mesa()).unwrap();
        let retrato = retrato();

        let saida = montar(&plano_com(&pasta, &destino, &retrato));

        assert!(saida.contains("E o disco de origem (R-7)"), "{saida}");
        assert!(
            !saida.contains("bcdboot"),
            "o aviso de disco novo sobreviveu ao ADR-0015:\n{saida}"
        );
        assert!(
            !saida.contains("sobram"),
            "a tela continua falando de sobra de espaco:\n{saida}"
        );
    }

    #[test]
    fn a_tela_avisa_sobre_imagem_reprovada_e_sem_veredito() {
        let destino = escolher(&crate::duplos::discos_desta_mesa()).unwrap();
        let retrato = retrato();

        let reprovada = imagem("x", Some(Veredito::Reprovada));
        assert!(
            montar(&plano_com(&reprovada, &destino, &retrato))
                .contains("ESTA IMAGEM FOI REPROVADA")
        );

        let sem = imagem("x", None);
        assert!(
            montar(&plano_com(&sem, &destino, &retrato)).contains("ESTA IMAGEM ESTA SEM VEREDITO")
        );

        let aprovada = imagem("x", Some(Veredito::Aprovada));
        let saida = montar(&plano_com(&aprovada, &destino, &retrato));
        assert!(
            !saida.contains("REPROVADA") && !saida.contains("SEM VEREDITO"),
            "{saida}"
        );
    }

    #[test]
    fn no_ensaio_a_tela_nao_diz_que_vai_apagar_agora() {
        let pasta = imagem("2026-08-22_Apps", Some(Veredito::Aprovada));
        let destino = escolher(&crate::duplos::discos_desta_mesa()).unwrap();
        let retrato = retrato();
        let mut plano = plano_com(&pasta, &destino, &retrato);
        plano.arma_em_seguida = false;

        let saida = montar(&plano);
        assert!(!saida.contains("ATENCAO: a restauracao APAGA"), "{saida}");
        assert!(saida.contains("Ensaio (--dry-run)"), "{saida}");
    }

    // ─────────── o cabecalho, e o que ele impede a recusa de engolir ───────────

    fn cabecalho_com(desarme: Option<&Desarme>) -> String {
        montar_cabecalho(&Cabecalho {
            dispositivo: &dispositivo_conectado(),
            desarme,
            caminho_do_grub: r"R:\boot\grub\grub.cfg",
        })
    }

    fn desarme_que_achou_receita() -> Desarme {
        Desarme {
            caminho_do_grub: r"R:\boot\grub\grub.cfg".into(),
            blocos_removidos: 1,
            default_devolvido: true,
            grub_regravado: true,
            boot_unico: crate::desarme::MarcaDeBootUnico::Removida {
                entradas: vec!["{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string()],
            },
        }
    }

    #[test]
    fn o_cabecalho_conta_o_desarmar_que_ja_aconteceu() {
        // Esta linha e a razao de o cabecalho existir separado: ela sai antes
        // de qualquer recusa, para que um `arca restore --destino <errado>`
        // num dispositivo armado nao faca o desarmar sumir em silencio. Foi o
        // defeito que a E7 pegou no `arca backup`, cometido de novo aqui e
        // achado rodando o comando de verdade.
        let desarme = desarme_que_achou_receita();
        let saida = cabecalho_com(Some(&desarme));

        assert!(
            saida.contains("Dispositivo ARCA: ARCAVAULT (E:)"),
            "{saida}"
        );
        assert!(saida.contains("ok · havia receita armada"), "{saida}");
    }

    #[test]
    fn no_ensaio_a_linha_do_desarmar_nao_diz_ok() {
        let saida = cabecalho_com(None);
        assert!(saida.contains("nao, e ensaio"), "{saida}");
        assert!(!saida.contains(" ok · "), "{saida}");
    }

    // ─────────────────── os avisos depois de armado ───────────────────

    fn armado_de_teste() -> armar::Armado {
        armar::Armado {
            caminho_do_estado: r"R:\arca\estado.json".into(),
            caminho_do_grub: r"R:\boot\grub\grub.cfg".into(),
            selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
            entrada: armar::Entrada::JaEraDoArca,
            identificador: "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string(),
            alvo: crate::firmware::Alvo::ParticaoComLetra('R'),
            caminho_do_desfecho: r"E:\ARCA-LOGS\restauracao-x\arca-fim.txt".into(),
            pasta_do_desfecho: "restauracao-x".to_string(),
        }
    }

    #[test]
    fn com_o_dispositivo_a_frente_o_aviso_diz_que_religar_restaura_de_novo() {
        let saida = montar_o_armado(&armado_de_teste(), OrdemDeBoot::DispositivoEmPrimeiro);

        assert!(
            saida.contains("AO TERMINAR: remova o SSD antes de religar."),
            "{saida}"
        );
        assert!(saida.contains("RESTAURA DE NOVO"), "{saida}");
        assert!(saida.contains("PRIMEIRO na ordem permanente"), "{saida}");
    }

    #[test]
    fn sem_o_dispositivo_a_frente_o_aviso_continua_falando_da_janela() {
        // A ordem permanente muda **sozinha** no ciclo de boot (ADR-0009): nao
        // estar a frente agora nao quer dizer nao estar a frente depois desta
        // operacao, que e justamente quando a janela abre.
        let saida = montar_o_armado(&armado_de_teste(), OrdemDeBoot::OutraCoisaAntes);

        assert!(
            saida.contains("AO TERMINAR: remova o SSD antes de religar."),
            "{saida}"
        );
        assert!(saida.contains("muda\n  sozinha"), "{saida}");
        assert!(saida.contains("RESTAURARIA DE\n  NOVO"), "{saida}");
    }

    #[test]
    fn firmware_ilegivel_ganha_o_aviso_duro_e_nao_o_brando() {
        // "Nao consegui lê" nao vira "o dispositivo nao esta a frente", que e
        // a resposta tranquilizadora — o mesmo erro que a guarda de
        // `viu_o_gerenciador` existe para nao cometer no `arca status`.
        let saida = montar_o_armado(&armado_de_teste(), OrdemDeBoot::NaoDeuParaLer);

        assert!(
            saida.contains("NAO FOI POSSIVEL LÊ A ORDEM PERMANENTE"),
            "{saida}"
        );
        assert!(saida.contains("Trate como se"), "{saida}");

        // A frase que **afirma** o estado da ordem e a do ramo brando, e ela
        // nao pode aparecer aqui. Casar por `nao leva ao dispositivo` nao
        // serviria: o proprio aviso duro contem essas palavras, dentro de
        // "o ARCA nao supoe que isso queira dizer que ela nao leva ao
        // dispositivo" — que e a negacao da afirmacao, e nao a afirmacao.
        assert!(
            !saida.contains("A ordem permanente hoje"),
            "sem lê a ordem, o ARCA nao afirma nada sobre ela:\n{saida}"
        );
        assert_ne!(
            saida,
            montar_o_armado(&armado_de_teste(), OrdemDeBoot::OutraCoisaAntes),
            "os dois avisos tem de ser diferentes, senao o terceiro estado nao existe"
        );
    }

    #[test]
    fn a_entrada_sem_alvo_a_frente_nao_ganha_o_aviso_brando() {
        // P-28. O ramo brando **afirma** — "a ordem permanente hoje nao leva ao
        // dispositivo em primeiro" —, e essa afirmacao nao se sustenta sobre
        // uma entrada que nao diz para onde aponta. E a mesma distincao que
        // separa `NaoDeuParaLer` de `OutraCoisaAntes`, um degrau adiante: aqui
        // a ordem foi lida, e o que ela tem a frente e que e opaco.
        let saida = montar_o_armado(&armado_de_teste(), OrdemDeBoot::SemAlvoAntes);

        assert!(saida.contains("NAO DIZ PARA ONDE APONTA"), "{saida}");
        assert!(saida.contains("Trate como se"), "{saida}");
        assert!(saida.contains("P-28"), "{saida}");
        assert!(
            !saida.contains("A ordem permanente hoje"),
            "o aviso brando afirma o que esta leitura nao sustenta:\n{saida}"
        );
        assert_ne!(
            saida,
            montar_o_armado(&armado_de_teste(), OrdemDeBoot::NaoDeuParaLer),
            "os dois avisos tem de ser diferentes: um leu a ordem e o outro nao"
        );
    }

    #[test]
    fn os_quatro_avisos_falam_da_janela_e_vem_antes_do_reiniciando() {
        for ordem in [
            OrdemDeBoot::DispositivoEmPrimeiro,
            OrdemDeBoot::OutraCoisaAntes,
            OrdemDeBoot::SemAlvoAntes,
            OrdemDeBoot::NaoDeuParaLer,
        ] {
            let saida = montar_o_armado(&armado_de_teste(), ordem);

            let aviso = saida.find("remova o SSD").expect("o aviso de C-9");
            let reinicio = saida.find("Reiniciando...").expect("a ultima linha");
            assert!(
                aviso < reinicio,
                "C-9 e a ultima coisa que alguem lê: {ordem:?}"
            );

            // Nenhum dos tres pode deixar a pessoa sem saber do perigo. O que
            // muda entre eles e a dureza da frase — e ela e o que a leitura
            // sustenta, nunca mais do que isso.
            assert!(
                saida.to_uppercase().contains("RESTAURA"),
                "o aviso de {ordem:?} nao fala da janela:\n{saida}"
            );
        }
    }
}
