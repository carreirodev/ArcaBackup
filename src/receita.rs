//! A receita, e o porteiro que a valida antes de qualquer gravacao (C-2).
//!
//! A receita e a string que o Clonezilla executa sozinho no boot desatendido.
//! Ela mora dentro de tres aninhamentos, e cada um deles tem uma forma de ser
//! quebrado:
//!
//! ```text
//! $linux_cmd ... ocs_live_run="bash -c '<a receita mora aqui>'" ...
//!                             ^^^^^^^^^^                     ^^
//!                             |         |                     |
//!                             |         aspa simples          aspa dupla
//!                             a linha do grub.cfg
//! ```
//!
//! Uma aspa simples na receita fecha o `bash -c`. Uma aspa dupla fecha o
//! `ocs_live_run`. Um pipe invalida a string inteira: o Clonezilla a descarta
//! e abre o menu interativo, sem executar nada e sem avisar (§3.2 do PRD) —
//! que e indistinguivel de "o boot nao funcionou".
//!
//! # O que aqui e transcricao e o que aqui e codigo novo
//!
//! Tres receitas rodaram em hardware e estao preservadas em
//! `recursos/capturas/` (ver `PROVENIENCIA.md`). Elas sao o original de uma
//! parte disto, e de outra parte **nao ha original nenhum**. A diferenca esta
//! marcada em cada constante e cobrada nos testes:
//!
//! | Parte | Origem |
//! |---|---|
//! | As flags do `ocs-sr`, na ordem | Transcrito das tres capturas |
//! | O `ocs-chkimg` com saida redirecionada | Transcrito de `ARCA-TESTE-03` |
//! | `ocs_repository`, `locales`, `keyboard-layouts`, `ocs_live_batch` | Transcrito das tres |
//! | A forma `bash -c '...'` com `;` entre os passos | Transcrito das tres |
//! | O `if/then/else` de R-5 | **Codigo novo** — nenhuma receita real o usou |
//! | O `arca-fim.txt`, o selo, o `ARCA_FIM` | **Codigo novo** — nenhuma receita real o escreveu |
//! | O `ARCA_VEREDITO=` no `arca-check.log` | **Codigo novo** — ver ADR-0003 |
//! | O `lsblk` da sondagem, e o `ARCA_PROBE=` | **Codigo novo** — ver [`montar_sondagem`] |
//! | As flags do `lsblk` | **Reconstrucao**, e ha uma terceira coluna para isso — ver [`FLAGS_DE_SONDAGEM`] |
//!
//! A terceira procedencia nasceu na etapa E12, e ela nao e nenhuma das duas
//! anteriores: das outras receitas se tem a **linha de comando** que rodou; da
//! sondagem se tem o **resultado** que se quer reproduzir, e a linha e
//! reconstruida a partir dele.
//!
//! O `ARCA_RESTORE=OK` que existe no dispositivo, e o `ARCA_VEREDITO=APROVADA`
//! que o ADR-0003 encontrou, **nao saem de receita nenhuma**: sairam do
//! trabalho manual de validacao em volta dela. Sao a forma que se quer, e nao
//! evidencia de que ela ja rodou.

// Os dois nomes de arquivo que a receita **escreve** e outro modulo **lê**:
// a pasta de logs de [`crate::dispositivo`] e o `arca-check.log` de
// [`crate::imagens`]. Importados em vez de repetidos, para que mudar um nome
// mude os dois lados de uma vez.
use crate::dispositivo::ARCA_LOGS;
use crate::imagens::CHECK_LOG;
use crate::nome::Nome;
use std::fmt;

/// Onde o Clonezilla monta o `ARCAVAULT`. Nao e escolha: e o ponto de
/// montagem do proprio Clonezilla, e todo caminho da receita parte dele.
const PARTIMAG: &str = "/home/partimag";

/// O arquivo do desfecho (§4.3, C-11, S-4).
///
/// Publico porque quem **escreve** este arquivo e a receita, deste lado do
/// reinicio, e quem o lê e [`crate::desfecho`], do outro. Um nome so, num
/// lugar so — o mesmo motivo de [`crate::imagens::CHECK_LOG`] ser publico.
pub const ARCA_FIM: &str = "arca-fim.txt";

/// O marcador que abre o `arca-fim.txt` e carrega o selo (§4.3, C-11).
///
/// Publico pela mesma razao: a receita escreve `ARCA_SELO=<selo>` na primeira
/// linha, e [`crate::desfecho`] lê de la. Mudar o marcador aqui muda os dois
/// lados de uma vez.
pub const MARCA_DO_SELO: &str = "ARCA_SELO=";

/// A linha que separa um desfecho completo de um truncado por desligamento no
/// meio (§5.5).
pub const MARCA_DO_FIM: &str = "ARCA_FIM";

/// As flags de B-8, **na ordem em que rodaram** nas duas capturas de backup.
///
/// Transcrito de `recursos/capturas/grub-backup-arca-teste-02.cfg` e
/// `grub-backup-arca-teste-03.cfg`, que trazem a mesma sequencia.
///
/// # Tres divergencias com o B-8 publicado, e por que a captura ganha
///
/// B-8 pedia `-batch -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -scs`. O hardware
/// rodou outra coisa:
///
/// - **`-batch` no fim, e nao no comeco.** So a posicao muda; o efeito, nao.
///   Fica onde rodou. O help desta versao do `ocs-sr` explica por que e
///   `-batch` e nao `-b`: *"You have to use '-batch' instead of '-b' when you
///   want to use it in the boot parameters. Otherwise the program init on
///   system will honor '-b', too."* Fecha P-15.
/// - **`-p true` presente, e B-8 nao o listava.** Nao e enfeite: o help diz
///   que o padrao de `-p|--postaction` e **`reboot`**. Sem `-p true` o
///   `ocs-sr` reiniciaria a maquina assim que terminasse de gravar, e o
///   `ocs-chkimg` que B-9 exige nunca chegaria a rodar. Quem desliga e o
///   `poweroff` do fim da receita.
/// - **`-scs` ausente, e B-8 o pedia sempre.** `-scs` e
///   `--skip-check-restorable`: *"By default Clonezilla will check the image
///   if restorable after it is created. This option allows you to skip
///   that."* Ele **pula** uma verificacao, que e o contrario do que B-9 quer.
///   Fica de fora: o backup validado rodou sem ele, e sem ele ha dois sinais
///   independentes sobre a imagem — a conferencia nativa, que alimenta o
///   codigo de saida que o `if` de R-5 lê, e o `ocs-chkimg` explicito, que
///   nao depende dele.
const FLAGS_DE_BACKUP: &str = "-q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true";

/// As flags da restauracao, transcritas de
/// `recursos/capturas/grub-restauracao-arca-teste-02.cfg`, com uma unica
/// mudanca deliberada.
///
/// # O que R-4 nao listava e rodou assim mesmo
///
/// R-4 pede `-batch -k0 -iefi -j2`. A restauracao validada rodou
/// `-e1 auto -e2 -batch -j2 -k0 -iefi -p poweroff`. Os dois a mais ficam:
///
/// - **`-e1 auto`** — *"Force to change the CHS (cylinders, heads, sectors)
///   value of NTFS boot partition after image is restored"*, com `auto`
///   deixando o Clonezilla achar a particao de boot NTFS sozinho.
/// - **`-e2`** — *"Force to use the CHS from EDD when creating partition
///   table by sfdisk"*.
///
/// Restaurando no **mesmo** disco, os dois sao inocuos: a geometria de
/// destino e a de origem. Restaurando em **outro** disco — que a decisao 5 do
/// plano permite —, sao exatamente o que faz a particao de boot NTFS bater
/// com a geometria do disco novo. Nao ha argumento para tirar de uma receita
/// destrutiva o que estava na unica execucao dela que deu certo.
///
/// # A mudanca deliberada: `-p poweroff` vira `-p true`
///
/// A restauracao validada terminava no proprio `ocs-sr`, com `-p poweroff`.
/// Nao ha mais como fazer isso: S-4 e R-5 exigem escrever o desfecho **depois**
/// de o `ocs-sr` sair, e com `-p poweroff` a maquina desliga antes de o `echo`
/// acontecer. `-p true` e o mesmo que a receita de backup usou, pelo mesmo
/// motivo — deixar a receita continuar —, e quem desliga e o `poweroff` do
/// fim.
const FLAGS_DE_RESTAURACAO: &str = "-e1 auto -e2 -batch -j2 -k0 -iefi -p true";

/// O `ocs-chkimg` de B-9, transcrito de
/// `recursos/capturas/grub-backup-arca-teste-03.cfg`.
const FLAGS_DE_VERIFICACAO: &str = "-b -or";

/// As flags do `lsblk` da sondagem — e elas sao **reconstrucao**, nao
/// transcricao.
///
/// # A diferenca, e por que ela e dita aqui e nao so num documento
///
/// Das outras tres receitas temos a **linha de comando** que rodou: ela esta
/// dentro do `ocs_live_run` das capturas de `grub.cfg`, e o codigo acima a
/// copia caractere a caractere. Da sondagem temos o **resultado** — o
/// `blkdev.list` de dentro das imagens, com o cabecalho
/// `KNAME NAME SIZE TYPE FSTYPE MOUNTPOINT MODEL` — e nao temos a linha: ela
/// mora nos scripts do Clonezilla, dentro do `filesystem.squashfs`, que este
/// repositorio nunca abriu.
///
/// Reconstruir as colunas a partir do cabecalho e honesto. Chamar isso de
/// transcricao nao seria, e o §3.5 do PRD conta cinco vezes em que esse
/// segundo movimento custou caro.
///
/// # O que a reconstrucao decidiu, item a item
///
/// - **As sete colunas, nesta ordem**, saem do cabecalho do arquivo capturado
///   — ver `DO_DISPOSITIVO` em [`crate::blkdev`], que e a copia dele.
///   [`crate::blkdev::ler`] usa tres delas (`NAME`, `TYPE`, `MODEL`) e acha
///   cada uma **pelo cabecalho**; as outras quatro ficam porque o arquivo que
///   se quer reproduzir as tem, e porque quem for olhar o arquivo a mao
///   procura o `MOUNTPOINT` para saber se o repositorio estava montado.
/// - **`-i`, que e `--ascii`.** O arquivo capturado desenha a arvore com
///   `|-` e `` ` `` — ASCII. O `lsblk` so escolhe esses simbolos quando o
///   `CODESET` do locale nao e UTF-8, e a receita boota com
///   `locales=en_US.UTF-8` (§3.2, obrigatorio): sem `-i` a arvore sairia com
///   os simbolos de caixa do Unicode, e o arquivo deixaria de ter a forma do
///   que ele imita. Nao muda o que o parser lê — as linhas de `disk` nao tem
///   prefixo de arvore nenhum —, e muda o que uma pessoa vê ao abrir o
///   arquivo ao lado de um `blkdev.list` de imagem.
///
/// # O modo de falha, que e o que torna a reconstrucao aceitavel
///
/// Uma flag que esta versao do util-linux nao conheca faz o `lsblk` sair com
/// codigo diferente de zero, e o `if` de [`montar_sondagem`] escreve
/// `ARCA_PROBE=FALHOU`. O `2>&1` manda a mensagem de erro para dentro do
/// proprio `blkdev.list`, entao **qual** flag foi recusada fica escrito no
/// dispositivo em vez de sumir com o `poweroff`. Custa um reinicio, e diz o
/// que consertar.
const FLAGS_DE_SONDAGEM: &str = "-i -o KNAME,NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT,MODEL";

/// O arquivo que a sondagem escreve no `ARCAVAULT`.
///
/// Importado de [`crate::blkdev`] em vez de repetido, pela razao de
/// [`ARCA_FIM`] e de [`crate::imagens::CHECK_LOG`]: quem escreve e a receita,
/// deste lado do reinicio, e quem lê e o parser, do outro. Um nome so, num
/// lugar so.
use crate::blkdev::ARQUIVO as ARQUIVO_DA_SONDAGEM;

/// O `COMMAND_LINE_SIZE` do kernel Linux no x86_64.
///
/// E o tamanho maximo da linha que o `grub` entrega ao kernel. Estourar nao
/// da erro: a linha e **truncada em silencio** — e uma receita truncada e uma
/// string invalida, que o Clonezilla descarta para abrir o menu interativo
/// (§3.2). O modo de falha e a maquina reiniciar e ficar parada num menu em
/// ingles tecnico, esperando alguem que ja saiu de perto.
const TETO_DA_LINHA_DE_COMANDO: usize = 2048;

/// O que se reserva, do teto, para o resto da linha `$linux_cmd`.
///
/// A E3 gera cinco parametros; o resto — `boot=live`, `hostname`, `vga`,
/// `toram`, as blacklists de driver — vem do `menuentry` base do Clonezilla,
/// que a E4 reproduz. Medido nas tres capturas: 206, 369 e 369 caracteres.
/// Reservar 512 e ficar quase 40% acima do maior ja visto.
const RESERVADO_PARA_O_MENUENTRY: usize = 512;

/// Quanto sobra para o que esta receita gera.
const TETO_DOS_PARAMETROS: usize = TETO_DA_LINHA_DE_COMANDO - RESERVADO_PARA_O_MENUENTRY;

/// Quanto a receita espera antes de desligar.
///
/// Vem de §10.1 do PRD; nenhuma captura o tem. Existe para o `echo ARCA_FIM`
/// chegar ao disco antes de o `poweroff` cortar: sem `ARCA_FIM`, a colheita
/// da E8 lê o desfecho como truncado, que e falha (§5.5). Vinte segundos numa
/// operacao de dezenas de minutos nao custam nada.
const ESPERA_ANTES_DE_DESLIGAR: u32 = 20;

/// Qual das operacoes a receita executa.
///
/// # A terceira nasceu na etapa E11, e a pergunta era se ela devia nascer
///
/// `Backup` e `Restauracao` atravessam este modulo, o `estado.json`, o
/// marcador do `arca-fim.txt`, a [`pasta_do_log`], o `arca resultado` e o
/// `arca status`. Acrescentar uma terceira toca tudo isso, e o que decidiu foi
/// **a pasta do log**.
///
/// [`pasta_do_log`] existe porque toda receita comeca truncando o proprio
/// `arca-fim.txt` com um `>`. A revisao da E3 pegou o backup e a restauracao
/// dividindo o mesmo caminho: um `arca restore X` rodado antes de o backup de
/// X ser colhido apagava o desfecho dele, e o §5.5 lia um backup bem-sucedido
/// como desfecho ausente. O selo nao cobre isso — ele julga um desfecho
/// **encontrado**, e nao serve para nada quando o arquivo ja foi por cima.
///
/// Uma verificacao armada que reusasse `Backup` cometeria o mesmo defeito pela
/// terceira vez, e agora contra um desfecho que o ARCA acabou de produzir. Ela
/// precisa de pasta propria, pasta propria vem do nome da operacao, e o nome
/// da operacao e isto aqui.
///
/// Ver `docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md`.
///
/// # A quarta nasceu na etapa E12, e ela e a primeira que nao nomeia imagem
///
/// As tres primeiras operam sobre uma imagem — duas a escrevem ou a leem, a
/// terceira a confere. A **sondagem** nao: ela roda `lsblk`, grava a saida e
/// desliga, e existe justamente para o dispositivo que **nao tem imagem
/// nenhuma** (§4.5, P-26). Por isso o `nome` do [`Pedido`] e do
/// [`crate::estado::Estado`] passa a ser opcional, como o `disco` passou a ser
/// na E11 — e pelo mesmo argumento, que esta em [`Operacao::nomeia_imagem`].
///
/// Ver `docs/adr/0019-a-sondagem-e-a-quarta-operacao.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operacao {
    Backup,
    Restauracao,

    /// `arca verify --completo` (V-2): so o `ocs-chkimg`, e desliga.
    Verificacao,

    /// `arca sondar` (SD-1): so o `lsblk`, e desliga. Nao chama o `ocs-sr`
    /// nem o `ocs-chkimg`, e nao escreve fora do `ARCAVAULT`.
    Sondagem,
}

impl Operacao {
    /// O marcador que a receita escreve no `arca-fim.txt` (§4.3, S-4).
    ///
    /// O `ARCA_VERIFY` e **codigo novo**, como o `ARCA_BACKUP` e o
    /// `ARCA_RESTORE` eram antes de 22/08/2026: nenhuma receita real o
    /// escreveu. A forma e transcrita dos dois que rodaram — `ARCA_` mais o
    /// nome da operacao em ingles, maiusculo —, e o que nao tem original e o
    /// marcador em si.
    fn marcador(self) -> &'static str {
        match self {
            Operacao::Backup => "ARCA_BACKUP",
            Operacao::Restauracao => "ARCA_RESTORE",
            Operacao::Verificacao => "ARCA_VERIFY",
            // Mesma forma dos tres, e mesma procedencia: `ARCA_` mais o nome
            // da operacao em ingles, maiusculo. `PROBE` e nao `SONDAR` porque
            // os outros tres estao em ingles, e um marcador em portugues no
            // meio de tres em ingles seria o comeco de duas convencoes.
            Operacao::Sondagem => "ARCA_PROBE",
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Operacao::Backup => "backup",
            Operacao::Restauracao => "restauracao",
            Operacao::Verificacao => "verificacao",
            Operacao::Sondagem => "sondagem",
        }
    }

    /// Se esta operacao nomeia um disco na receita.
    ///
    /// O `savedisk` e o `restoredisk` nomeiam; o `ocs-chkimg` opera sobre a
    /// **imagem**, e nao sobre disco nenhum, e o `lsblk` da sondagem olha
    /// todos. E o que faz o `disco` do [`Pedido`] e do
    /// [`crate::estado::Estado`] ser opcional a partir da E11.
    pub fn nomeia_disco(self) -> bool {
        match self {
            Operacao::Backup | Operacao::Restauracao => true,
            Operacao::Verificacao | Operacao::Sondagem => false,
        }
    }

    /// Se esta operacao nomeia uma imagem.
    ///
    /// # Por que isto e um segundo eixo, e nao o mesmo de [`nomeia_disco`]
    ///
    /// Os dois separam coisas diferentes, e a verificacao e a prova: ela **nao**
    /// nomeia disco e **nomeia** imagem. Juntar os dois num campo so faria a
    /// coerencia do `estado.json` parar de cobrir metade das combinacoes.
    ///
    /// A sondagem e a unica que nao nomeia nenhuma das duas: ela pergunta
    /// *"que discos ha nesta maquina?"*, e a pergunta nao tem sujeito a
    /// escolher. E por isso que a pasta do log dela nao leva nome
    /// ([`pasta_do_log`]) e que o `nome` do [`Pedido`] e opcional.
    ///
    /// [`nomeia_disco`]: Operacao::nomeia_disco
    pub fn nomeia_imagem(self) -> bool {
        match self {
            Operacao::Backup | Operacao::Restauracao | Operacao::Verificacao => true,
            Operacao::Sondagem => false,
        }
    }
}

/// O disco que a receita nomeia, com o nome que o Linux lhe da: `nvme0n1`,
/// `sda`.
///
/// E o unico lugar do ARCA onde um disco e nomeado por `sda`, e ele esta do
/// lado de la do reinicio — S-3 fala do **destino da receita**, que continua
/// resolvido por LABEL no `ocs_repository`. Quem descobre este nome e a
/// enumeracao de discos da E6; ate la ele chega de fora.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disco(String);

impl Disco {
    /// So `[a-z][a-z0-9]*`, que e a forma de todo nome de disco do Linux.
    ///
    /// Sem `/`, e de proposito: `/dev/nvme0n1` e aceito pelo `ocs-sr`, mas
    /// abrir a porta para barra abriria junto a de um caminho inventado.
    pub fn novo(bruto: &str) -> Result<Disco, RecusaDaReceita> {
        let Some(primeiro) = bruto.chars().next() else {
            return Err(RecusaDaReceita::DiscoVazio);
        };

        for caractere in bruto.chars() {
            if !(caractere.is_ascii_lowercase() || caractere.is_ascii_digit()) {
                return Err(RecusaDaReceita::DiscoInvalido { caractere });
            }
        }

        if !primeiro.is_ascii_lowercase() {
            return Err(RecusaDaReceita::DiscoNaoComecaComLetra);
        }

        Ok(Disco(bruto.to_string()))
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Disco {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// O selo que liga o job ao seu desfecho (§4.3, C-11).
///
/// Aqui ele so e **validado e embutido**: quem o gera e quem o grava no
/// `estado.json` e a etapa E5. A receita e o segundo dos tres lugares por
/// onde ele passa, e o unico que precisa saber que ele nao pode conter nada
/// que quebre uma string de shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selo(String);

/// Quantos digitos hexadecimais tem um selo, como §10.1 do PRD o mostra.
const DIGITOS_DO_SELO: usize = 16;

/// Quantos bytes de entropia produzem esses digitos.
///
/// Cada byte vira dois digitos hexadecimais. Publico porque quem pede os bytes
/// a [`crate::portas::Entropia`] e [`crate::estado::gerar_selo`], e o tamanho
/// do buffer tem de sair daqui — nao de um `8` digitado la.
pub const BYTES_DO_SELO: usize = DIGITOS_DO_SELO / 2;

impl Selo {
    pub fn novo(bruto: &str) -> Result<Selo, RecusaDaReceita> {
        // Minusculas so, e nao `is_ascii_hexdigit`: um selo que mudasse de
        // caixa entre o `estado.json` e o `arca-fim.txt` deixaria de casar, e
        // casar e a unica coisa que o selo faz.
        let hexadecimal =
            |caractere: char| caractere.is_ascii_digit() || ('a'..='f').contains(&caractere);

        if bruto.chars().count() != DIGITOS_DO_SELO || !bruto.chars().all(hexadecimal) {
            return Err(RecusaDaReceita::SeloInvalido {
                tem: bruto.to_string(),
            });
        }

        Ok(Selo(bruto.to_string()))
    }

    /// O selo a partir dos bytes que [`crate::portas::Entropia`] entregou.
    ///
    /// Funcao pura de proposito: quem fala com o sistema e
    /// [`crate::estado::gerar_selo`], e o que este modulo sabe fazer e a
    /// conversao — oito bytes viram os dezesseis digitos hexadecimais
    /// **minusculos** que [`Selo::novo`] exige. Nao ha caminho por onde um
    /// selo gerado saia recusado pelo proprio validador; ha teste cobrando.
    pub fn de_bytes(bytes: &[u8; BYTES_DO_SELO]) -> Selo {
        let mut texto = String::with_capacity(DIGITOS_DO_SELO);
        for byte in bytes {
            // `{:02x}` e minusculo e sempre de dois digitos: sem o `02`, um
            // byte abaixo de 16 sairia com um digito so e o selo encolheria.
            texto.push_str(&format!("{byte:02x}"));
        }
        Selo(texto)
    }

    /// Um selo para o `--dry-run`.
    ///
    /// Zeros, e nao um valor plausivel: um ensaio que imprima um selo com cara
    /// de real convida a comparar com um `arca-fim.txt` de verdade.
    pub fn de_ensaio() -> Selo {
        Selo("0".repeat(DIGITOS_DO_SELO))
    }

    pub fn como_texto(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Selo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Por que uma receita — ou uma das suas pecas — foi recusada (C-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDaReceita {
    /// §3.2: o Clonezilla descarta a string inteira e abre o menu interativo.
    Pipe,
    AspaSimples,
    AspaDupla,
    SubstituicaoDeComando {
        marca: &'static str,
    },
    CaractereDeControle {
        codigo: u32,
    },
    NaoAscii {
        caractere: char,
    },
    DiscoVazio,
    DiscoInvalido {
        caractere: char,
    },
    DiscoNaoComecaComLetra,
    SeloInvalido {
        tem: String,
    },

    /// A linha nao caberia no `COMMAND_LINE_SIZE` do kernel, e o kernel a
    /// truncaria em silencio. Ver [`TETO_DOS_PARAMETROS`].
    LinhaLongaDemais {
        tem: usize,
        teto: usize,
    },

    /// A operacao e o disco nao combinam: um `savedisk` sem disco, ou um
    /// `ocs-chkimg` com um.
    DiscoIncoerente {
        operacao: Operacao,
        tem_disco: bool,
    },

    /// A operacao e a imagem nao combinam: um `savedisk` sem nome, ou uma
    /// sondagem com um.
    NomeIncoerente {
        operacao: Operacao,
        tem_nome: bool,
    },
}

impl fmt::Display for RecusaDaReceita {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDaReceita::Pipe => write!(
                f,
                "a receita tem um pipe (`|`): o Clonezilla descarta a string inteira e abre o menu interativo, sem executar nada e sem avisar. Na receita valem so `>` e `>>`"
            ),
            RecusaDaReceita::AspaSimples => write!(
                f,
                "a receita tem aspa simples: ela mora dentro de um `bash -c '...'`, e uma aspa simples o fecha no meio"
            ),
            RecusaDaReceita::AspaDupla => write!(
                f,
                "a receita tem aspa dupla: ela mora dentro de um `ocs_live_run=\"...\"`, e uma aspa dupla o fecha no meio"
            ),
            RecusaDaReceita::SubstituicaoDeComando { marca } => write!(
                f,
                "a receita tem `{marca}`: substituicao de comando faz a string virar outra coisa entre a gravacao e a execucao, e o que se validou deixa de ser o que roda"
            ),
            RecusaDaReceita::CaractereDeControle { codigo } => write!(
                f,
                "a receita tem um caractere de controle (U+{codigo:04X}): a receita e uma linha so do `grub.cfg`, e nada nela pode quebrar linha"
            ),
            RecusaDaReceita::NaoAscii { caractere } => write!(
                f,
                "a receita tem `{caractere}`, que nao e ASCII: o que atravessa o grub e o live system e ASCII"
            ),
            RecusaDaReceita::DiscoVazio => write!(f, "nenhum disco foi nomeado para a operacao"),
            RecusaDaReceita::DiscoInvalido { caractere } => write!(
                f,
                "`{caractere}` nao e aceito em nome de disco: o Linux os nomeia com letras minusculas e digitos, como `nvme0n1` ou `sda`"
            ),
            RecusaDaReceita::DiscoNaoComecaComLetra => write!(
                f,
                "nome de disco tem de comecar por letra, como `nvme0n1` ou `sda`"
            ),
            RecusaDaReceita::SeloInvalido { tem } => write!(
                f,
                "`{tem}` nao e um selo: sao {DIGITOS_DO_SELO} digitos hexadecimais minusculos"
            ),
            RecusaDaReceita::LinhaLongaDemais { tem, teto } => write!(
                f,
                "a receita ocuparia {tem} caracteres na linha do grub.cfg, e o orcamento e {teto}: acima disso o kernel trunca a linha em silencio, e uma receita truncada faz o Clonezilla abrir o menu interativo. Use um nome de imagem mais curto"
            ),
            RecusaDaReceita::DiscoIncoerente {
                operacao,
                tem_disco: true,
            } => write!(
                f,
                "a operacao `{}` nao nomeia disco nenhum na receita — quem o faz sao o `savedisk` e o `restoredisk` —, e veio um. Um disco carregado ate uma receita que nao o usa e um valor que ninguem confere",
                operacao.nome()
            ),
            RecusaDaReceita::DiscoIncoerente {
                operacao,
                tem_disco: false,
            } => write!(
                f,
                "a operacao `{}` nomeia um disco na receita e nenhum foi dado",
                operacao.nome()
            ),
            RecusaDaReceita::NomeIncoerente {
                operacao,
                tem_nome: true,
            } => write!(
                f,
                "a operacao `{}` nao opera sobre imagem nenhuma — ela lê os discos da maquina —, e veio um nome. Um nome carregado ate uma receita que nao o usa e um valor que ninguem confere",
                operacao.nome()
            ),
            RecusaDaReceita::NomeIncoerente {
                operacao,
                tem_nome: false,
            } => write!(
                f,
                "a operacao `{}` opera sobre uma imagem e nenhuma foi nomeada",
                operacao.nome()
            ),
        }
    }
}

/// O que se pede a uma receita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pedido {
    pub operacao: Operacao,

    /// A imagem sobre a qual a receita opera, quando ha uma.
    ///
    /// `None` **so** na sondagem, e [`Receita::montar`] cobra a coerencia nos
    /// dois sentidos, como ja faz com o [`Pedido::disco`]. A sondagem pergunta
    /// *"que discos ha nesta maquina?"*, e a pergunta nao tem imagem por
    /// sujeito — ela existe justamente para o dispositivo que ainda nao tem
    /// nenhuma (§4.5, P-26).
    pub nome: Option<Nome>,

    /// O disco que a receita nomeia, quando a operacao nomeia algum.
    ///
    /// `None` **so** na verificacao, e [`Receita::montar`] cobra a coerencia
    /// nos dois sentidos: um backup sem disco e recusado, e uma verificacao
    /// **com** disco tambem. O segundo importa tanto quanto o primeiro — um
    /// disco carregado ate uma receita que nao o usa e um valor que ninguem
    /// confere, num arquivo que arma operacao destrutiva.
    pub disco: Option<Disco>,

    pub selo: Selo,
}

/// Um parametro da linha `$linux_cmd` que a receita exige para rodar
/// desatendida.
///
/// Os cinco vem das tres capturas, que concordam neles. O resto da linha —
/// `hostname`, `vga`, `toram`, as blacklists de driver — e do `menuentry`
/// base do Clonezilla, nao da receita: quem o reproduz e a E4, a partir do
/// `grub.cfg` que ja esta no dispositivo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parametro {
    pub nome: &'static str,
    pub valor: String,
    /// Se o `grub.cfg` o escreve entre aspas duplas. As capturas usam aspas
    /// nos `ocs_*` e nao usam em `locales` e `keyboard-layouts`.
    pub entre_aspas: bool,
}

impl fmt::Display for Parametro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.entre_aspas {
            write!(f, "{}=\"{}\"", self.nome, self.valor)
        } else {
            write!(f, "{}={}", self.nome, self.valor)
        }
    }
}

/// Uma receita pronta e ja validada por C-2.
///
/// So se constroi por [`Receita::montar`], que valida antes de devolver. Ter
/// uma em maos e ter a garantia de C-2 — nenhuma gravacao precisa validar de
/// novo, e nenhuma consegue pular a validacao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receita {
    operacao: Operacao,
    comando: String,
    parametros: Vec<Parametro>,
}

impl Receita {
    /// Monta e valida (C-2). Recusa antes de devolver, nunca depois de gravar.
    pub fn montar(pedido: &Pedido) -> Result<Receita, RecusaDaReceita> {
        // A coerencia entre a operacao e o disco e cobrada **aqui**, nos dois
        // sentidos, e nao dentro de cada montagem: e a unica porta por onde uma
        // receita nasce, e um `Option` destravado dentro de `montar_backup`
        // deixaria a incoerencia virar um `unwrap` em vez de uma recusa com
        // nome.
        // A coerencia da **imagem** vem primeiro, e por um motivo pratico: as
        // tres montagens que nomeiam imagem recebem o `Nome` ja destravado, e
        // sem esta porta cada uma teria de destrava-lo por conta — que e o
        // `unwrap` que este bloco existe para nao haver.
        if pedido.operacao.nomeia_imagem() != pedido.nome.is_some() {
            return Err(RecusaDaReceita::NomeIncoerente {
                operacao: pedido.operacao,
                tem_nome: pedido.nome.is_some(),
            });
        }

        let comando = match (pedido.operacao, &pedido.nome, &pedido.disco) {
            (Operacao::Backup, Some(nome), Some(disco)) => montar_backup(pedido, nome, disco),
            (Operacao::Restauracao, Some(nome), Some(disco)) => {
                montar_restauracao(pedido, nome, disco)
            }
            (Operacao::Verificacao, Some(nome), None) => montar_verificacao(pedido, nome),
            (Operacao::Sondagem, None, None) => montar_sondagem(pedido),
            (operacao, _, disco) => {
                return Err(RecusaDaReceita::DiscoIncoerente {
                    operacao,
                    tem_disco: disco.is_some(),
                });
            }
        };

        validar(&comando)?;

        let parametros = vec![
            // §3.2: `locales=` vazio abre tela de idioma mesmo em batch.
            Parametro {
                nome: "locales",
                valor: "en_US.UTF-8".to_string(),
                entre_aspas: false,
            },
            Parametro {
                nome: "keyboard-layouts",
                valor: "NONE".to_string(),
                entre_aspas: false,
            },
            // S-3: por LABEL, sempre. E o que elimina a ambiguidade
            // `sda`/`sdb` e o que torna os dispositivos intercambiaveis.
            Parametro {
                nome: "ocs_repository",
                valor: format!("dev:///LABEL={}", crate::dispositivo::ARCAVAULT),
                entre_aspas: true,
            },
            Parametro {
                nome: "ocs_live_run",
                valor: format!("bash -c '{comando}'"),
                entre_aspas: true,
            },
            Parametro {
                nome: "ocs_live_batch",
                valor: "yes".to_string(),
                entre_aspas: true,
            },
        ];

        let receita = Receita {
            operacao: pedido.operacao,
            comando,
            parametros,
        };

        // A ultima conferencia, e a unica que so faz sentido sobre a linha
        // pronta: ela cabe no que o kernel aceita? Estourar nao da erro em
        // lugar nenhum — a linha e truncada em silencio, e o Clonezilla abre
        // o menu. Recusar aqui e a diferenca entre um nome recusado no
        // Windows e uma maquina parada esperando alguem.
        let ocupa = receita.parametros_do_grub().chars().count();
        if ocupa > TETO_DOS_PARAMETROS {
            return Err(RecusaDaReceita::LinhaLongaDemais {
                tem: ocupa,
                teto: TETO_DOS_PARAMETROS,
            });
        }

        Ok(receita)
    }

    pub fn operacao(&self) -> Operacao {
        self.operacao
    }

    /// O miolo do `bash -c '...'`: o que o Clonezilla executa.
    pub fn comando(&self) -> &str {
        &self.comando
    }

    pub fn parametros(&self) -> &[Parametro] {
        &self.parametros
    }

    /// Os parametros na forma em que entram na linha `$linux_cmd`, separados
    /// por espaco — como as capturas os escrevem.
    pub fn parametros_do_grub(&self) -> String {
        self.parametros
            .iter()
            .map(|parametro| parametro.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A pasta de log deste job, dentro do `ARCAVAULT` (D2 do plano).
///
/// Leva a **operacao** junto do nome, e nao so o nome. Sem isso, o backup de
/// uma imagem e a restauracao dela escreveriam no mesmo `arca-fim.txt`, e o
/// segundo apagaria o primeiro: o `echo ARCA_SELO=... >` que abre toda
/// receita trunca o arquivo. Um `arca backup X` colhido tarde demais — depois
/// de um `arca restore X` — perderia o desfecho para sempre, e o §5.5 leria
/// um backup bem-sucedido como desfecho ausente.
///
/// O selo nao resolve isso: ele diz se um desfecho **encontrado** pertence ao
/// job corrente, e nao serve para nada quando o arquivo ja foi por cima.
///
/// Publica porque o mesmo nome de pasta tem de ser montado dos **dois lados do
/// reinicio**: aqui, dentro de um caminho Linux que a receita escreve, e em
/// [`crate::desfecho`], dentro de um caminho Windows que o ARCA lê na volta.
/// Devolve so o nome da pasta, e nao o caminho, justamente para que nenhum dos
/// dois lados precise conhecer o do outro.
///
/// # A sondagem tem pasta **fixa**, e a anterior e substituida
///
/// Ela nao nomeia imagem ([`Operacao::nomeia_imagem`]), entao nao ha o que pôr
/// depois do hifen: a pasta e `sondagem`, e duas sondagens escrevem no mesmo
/// lugar. **Aqui isso e o comportamento certo**, e nao o defeito que a revisao
/// da E3 pegou entre o backup e a restauracao.
///
/// O que se perdia la era o desfecho de **outro job**, sobre outra pergunta,
/// que ninguem mais ia reproduzir. O que se perde aqui e a **medicao anterior
/// da mesma pergunta** — *que discos ha nesta maquina?* —, e a resposta mais
/// recente e a que vale: uma sondagem velha descrevendo uma maquina que mudou
/// e pior do que nenhuma. Ela e barata de refazer, ao contrario de um backup.
///
/// O que isto custa esta dito e e o mesmo das outras tres com o mesmo nome:
/// uma sondagem armada por cima de outra ainda **nao colhida** trunca o
/// `arca-fim.txt` dela. So ha um `estado.json` por dispositivo, entao aquele
/// job ja estava perdido antes de a pasta ser tocada.
///
/// `sondagem` nao colide com nenhuma das outras tres: as delas sao
/// `{operacao}-{nome}`, e [`Nome`] nunca e vazio.
pub fn pasta_do_log(operacao: Operacao, nome: Option<&Nome>) -> String {
    match nome {
        Some(nome) => format!("{}-{nome}", operacao.nome()),
        None => operacao.nome().to_string(),
    }
}

fn log_do_job(operacao: Operacao, nome: Option<&Nome>) -> String {
    format!("{PARTIMAG}/{ARCA_LOGS}/{}", pasta_do_log(operacao, nome))
}

/// Onde a receita grava o desfecho (S-4, C-11).
fn arquivo_do_desfecho(operacao: Operacao, nome: Option<&Nome>) -> String {
    format!("{}/{ARCA_FIM}", log_do_job(operacao, nome))
}

/// A receita de backup.
///
/// A ordem tem uma razao em cada passo:
///
/// 1. `mkdir -p` do log — sem ele o primeiro `>` falha e o desfecho inteiro
///    se perde. E o unico passo desta lista que tem original: a receita de
///    `ARCA-TESTE-03` comeca com um `mkdir -p`, pelo mesmo motivo;
/// 2. o selo, com `>`, que e o que garante que ele e a **primeira** linha do
///    `arca-fim.txt` (§4.3);
/// 3. o `savedisk` dentro de um `if`, porque `;` nao olha codigo de saida
///    (R-5);
/// 4. a verificacao de B-9 **dentro do ramo de exito**: com o backup
///    falhando, a pasta da imagem pode nem existir, e ate o `else` do
///    `ocs-chkimg` falharia ao tentar escrever nela;
/// 5. o `ARCA_FIM`, que e o que separa um desfecho completo de um truncado
///    por desligamento no meio (§5.5);
/// 6. a espera e o `poweroff`.
fn montar_backup(pedido: &Pedido, nome: &Nome, disco: &Disco) -> String {
    let Pedido { selo, .. } = pedido;

    let marcador = pedido.operacao.marcador();
    let log = log_do_job(pedido.operacao, Some(nome));
    let desfecho = arquivo_do_desfecho(pedido.operacao, Some(nome));
    let veredito = format!("{PARTIMAG}/{nome}/{}", CHECK_LOG);

    let passos = [
        format!("mkdir -p {log}"),
        format!("echo {MARCA_DO_SELO}{selo} > {desfecho}"),
        format!(
            "if ocs-sr {FLAGS_DE_BACKUP} savedisk {nome} {disco}; \
             then echo {marcador}=OK >> {desfecho}; \
             if ocs-chkimg {FLAGS_DE_VERIFICACAO} {PARTIMAG} {nome} > {veredito} 2>&1; \
             then echo ARCA_VEREDITO=APROVADA >> {veredito}; \
             else echo ARCA_VEREDITO=REPROVADA >> {veredito}; fi; \
             else echo {marcador}=FALHOU >> {desfecho}; fi"
        ),
        format!("echo {MARCA_DO_FIM} >> {desfecho}"),
        format!("sleep {ESPERA_ANTES_DE_DESLIGAR}"),
        "poweroff".to_string(),
    ];

    passos.join("; ")
}

/// A receita de restauracao.
///
/// Sem verificacao: B-9 e do backup, e aqui nao ha imagem nova para conferir.
/// E por isso que P-6 — o `ocs-sr` devolver zero ao falhar — dói mais deste
/// lado: no backup o `ocs-chkimg` e um segundo sinal independente, e aqui o
/// unico juiz e o Windows subir ou nao.
fn montar_restauracao(pedido: &Pedido, nome: &Nome, disco: &Disco) -> String {
    let Pedido { selo, .. } = pedido;

    let marcador = pedido.operacao.marcador();
    let log = log_do_job(pedido.operacao, Some(nome));
    let desfecho = arquivo_do_desfecho(pedido.operacao, Some(nome));

    // §10.2: o log mora no `ARCAVAULT`, que a restauracao nao toca. A imagem
    // substitui o `nvme0n1`, e o desfecho sobrevive num disco que nao estava
    // no caminho. A captura mandava para `/home/partimag/restore.log`, na
    // raiz — um caminho fixo que a restauracao seguinte sobrescreveria.
    let registro_do_clonezilla = format!("{log}/arca-restore.log");

    let passos = [
        format!("mkdir -p {log}"),
        format!("echo {MARCA_DO_SELO}{selo} > {desfecho}"),
        format!(
            "if ocs-sr {FLAGS_DE_RESTAURACAO} restoredisk {nome} {disco} > {registro_do_clonezilla} 2>&1; \
             then echo {marcador}=OK >> {desfecho}; \
             else echo {marcador}=FALHOU >> {desfecho}; fi"
        ),
        format!("echo {MARCA_DO_FIM} >> {desfecho}"),
        format!("sleep {ESPERA_ANTES_DE_DESLIGAR}"),
        "poweroff".to_string(),
    ];

    passos.join("; ")
}

/// A receita da verificacao armada (V-2), e a menor das tres.
///
/// # O que aqui e transcricao e o que aqui e invencao
///
/// | Parte | Origem |
/// |---|---|
/// | `ocs-chkimg -b -or /home/partimag <nome>` | Transcrito de `ARCA-TESTE-03` |
/// | O `if` sobre o `ocs-chkimg` escrevendo `ARCA_VEREDITO=` | Transcrito da receita de backup, que rodou em 22/08/2026 |
/// | O `mkdir -p`, o `ARCA_SELO=` com `>`, o `ARCA_FIM`, o `sleep`, o `poweroff` | Transcrito das duas que rodaram |
/// | A forma `bash -c '...'` com `;` e os cinco parametros | Transcrito das tres capturas |
/// | **O `ARCA_VERIFY=`** | **Codigo novo** — nenhuma receita real o escreveu |
/// | **O `ocs-chkimg` como comando principal, fora de um `savedisk`** | **Codigo novo** — nas capturas ele so aparece aninhado |
///
/// As duas ultimas sao o que esta etapa estreia, e e menos do que a E7
/// estreou: o `if/then/else` e o `arca-fim.txt` ja rodaram.
///
/// # O `>>` no `arca-check.log`, e o backup usa `>`
///
/// A receita de backup redireciona o `ocs-chkimg` com `>`, porque a imagem
/// acabou de nascer e o log nao existe. Aqui ele existe: e o veredito do
/// backup que criou a imagem, e o `>` o **destruiria**.
///
/// Isso custaria duas coisas. O §6.3 mostra `Imagem de origem: APROVADA —
/// veredito do backup que a criou`, e a frase viraria mentira depois da
/// primeira verificacao. E o `>` **trunca ao abrir**, antes de o comando
/// rodar (§5.5, medido): um desligamento nessa janela deixaria uma imagem boa
/// com o `arca-check.log` em zero byte, e ela apareceria `sem veredito` na
/// listagem — perda de informacao sobre uma imagem que nao tem nada de errado.
///
/// # O marco desmentiu a metade otimista disto, e o `>>` fica assim mesmo
///
/// A versao anterior deste comentario dizia que com `>>` o
/// [ADR-0003](../docs/adr/0003-veredito-lido-do-arca-check-log.md) valeria como
/// esta escrito — *"uma imagem verificada duas vezes fica com as duas marcas"*.
/// **Nao ficou.** Medido em 23/08/2026, depois da verificacao armada da
/// `2026-08-22_Apps`: o `arca-check.log` tem **uma** ocorrencia de
/// `ARCA_VEREDITO=`, no fim, e **uma** inicializacao de terminal — ou seja, uma
/// execucao do `ocs-chkimg`, e nao duas. O log do backup sumiu.
///
/// A receita tinha `>>` — o `--dry-run` a imprimiu assim, e
/// `recursos/ensaio-da-receita.sh` prova que `>>` acrescenta num bash de
/// verdade. **A causa nao esta determinada**, e e P-25.
///
/// **O `>>` fica, com a razao trocada.** Ele nao compra a preservacao do log
/// antigo; o que ele compra e nao ter a janela do `>`, que **trunca ao abrir**
/// e deixaria uma imagem boa com o log em zero byte se o desligamento pegasse
/// ali. E o mesmo movimento do
/// [ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md):
/// a defesa fica, e a razao muda para a que sobreviveu a medicao.
///
/// A ordem "toda forma de reprovar antes de toda forma de aprovar" decide o
/// que sai **quando** as duas marcas estao no mesmo arquivo: uma imagem que ja
/// reprovou uma vez continua reprovada mesmo que a verificacao nova aprove. E
/// o lado conservador de proposito — mídia que falha de forma intermitente e o
/// caso em que a segunda leitura mente —, e e o que S-5 pede. Continua sem
/// original, como o ADR-0003 ja registrava.
fn montar_verificacao(pedido: &Pedido, nome: &Nome) -> String {
    let Pedido { selo, .. } = pedido;

    let marcador = pedido.operacao.marcador();
    let log = log_do_job(pedido.operacao, Some(nome));
    let desfecho = arquivo_do_desfecho(pedido.operacao, Some(nome));
    let veredito = format!("{PARTIMAG}/{nome}/{}", CHECK_LOG);

    let passos = [
        format!("mkdir -p {log}"),
        format!("echo {MARCA_DO_SELO}{selo} > {desfecho}"),
        format!(
            "if ocs-chkimg {FLAGS_DE_VERIFICACAO} {PARTIMAG} {nome} >> {veredito} 2>&1; \
             then echo ARCA_VEREDITO=APROVADA >> {veredito}; \
             echo {marcador}=OK >> {desfecho}; \
             else echo ARCA_VEREDITO=REPROVADA >> {veredito}; \
             echo {marcador}=FALHOU >> {desfecho}; fi"
        ),
        format!("echo {MARCA_DO_FIM} >> {desfecho}"),
        format!("sleep {ESPERA_ANTES_DE_DESLIGAR}"),
        "poweroff".to_string(),
    ];

    passos.join("; ")
}

/// A receita da sondagem (SD-1 a SD-3), e a unica das quatro que nao chama
/// programa nenhum do Clonezilla.
///
/// # O que aqui e transcricao, o que rodou, e o que e codigo novo
///
/// | Parte | Origem |
/// |---|---|
/// | O `mkdir -p`, o `ARCA_SELO=` com `>`, o `ARCA_FIM`, o `sleep`, o `poweroff` | Transcrito das tres receitas que rodaram |
/// | A forma `bash -c '...'` com `;` e os cinco parametros | Transcrito das tres capturas |
/// | O `if` sobre o comando principal, escrevendo `OK` ou `FALHOU` | Transcrito das tres |
/// | Escrever em `/home/partimag` **antes** de qualquer comando do Clonezilla | **Rodou** — a verificacao armada da E11, em 23/08/2026 |
/// | O **formato** do arquivo de saida | Transcrito do `blkdev.list` de dentro das imagens |
/// | **O `lsblk` como comando principal** | **Codigo novo** — nenhuma receita deste projeto o chamou |
/// | **O `ARCA_PROBE=`** | **Codigo novo** |
/// | **As flags do `lsblk`** | **Reconstrucao** — ver [`FLAGS_DE_SONDAGEM`] |
///
/// # O pressuposto perigoso ja tinha original, e ninguem tinha notado
///
/// Esta receita escreve em `/home/partimag` **antes** de qualquer comando do
/// Clonezilla — as outras tres so escrevem ali depois de o `ocs-sr` ou o
/// `ocs-chkimg` terem rodado. Se o repositorio nao estivesse montado nesse
/// instante, o `mkdir` criaria a pasta no tmpfs da RAM e o `poweroff` levaria
/// tudo embora: falha silenciosa, sem nada no dispositivo para investigar.
///
/// Esta provado, e a prova e da E11. [`montar_verificacao`] tem exatamente
/// esta forma — passo 1 `mkdir -p`, passo 2 `echo ARCA_SELO= >`, e so no passo
/// 3 o `ocs-chkimg` —, rodou em 23/08/2026 as 16:53, e o resultado esta em
/// `recursos/capturas/arca-fim-verificacao-2026-08-22_Apps.txt`: cinquenta e
/// um bytes que sairam daqueles dois primeiros passos. **Quem monta o
/// `/home/partimag` e o `ocs_repository=` do boot, e nao o `ocs-sr`.**
///
/// E ha um segundo sinal, de graca: o `lsblk` roda com o repositorio montado,
/// entao a linha da particao do `ARCAVAULT` sai com `/home/partimag` no
/// `MOUNTPOINT` — como ja sai nos `blkdev.list` capturados. O proprio arquivo
/// testemunha que foi escrito no lugar certo.
///
/// # O `if` nao e enfeite, e a primeira forma escrita desta receita nao o tinha
///
/// A forma proposta na mesa encadeava com `;`:
///
/// ```text
/// lsblk -o ... > .../blkdev.list; echo ARCA_PROBE=OK >> .../arca-fim.txt;
/// ```
///
/// O `;` nao olha codigo de saida. Com o `lsblk` falhando — uma flag que esta
/// versao do util-linux nao conheca basta —, o desfecho diria **`OK`** assim
/// mesmo. E R-5, e e o passo 3 de [`montar_backup`] desde a E3.
///
/// O estrago nao e abstrato: [`crate::blkdev::ler`] devolveria lista vazia, o
/// disco de origem sairia `POR DETERMINAR`, e a tela diria isso **logo depois**
/// de o `arca resultado` ter dito que a sondagem concluiu com sucesso. Duas
/// afirmacoes contraditorias, as duas do ARCA, na mesma sessao.
///
/// # O `2>&1` aponta para o proprio `blkdev.list`, e nao para um log a parte
///
/// As outras receitas mandam o erro para o log da operacao. Aqui nao ha log
/// separado, e mandar o erro para o arquivo de saida e melhor do que parece:
/// falhando o `lsblk`, o `blkdev.list` fica com a mensagem dele em vez de
/// vazio, e a proxima sessao lê **qual** flag foi recusada em vez de deduzir.
/// Um arquivo com a mensagem de erro nao e lido como oraculo — o cabecalho nao
/// bate, e [`crate::blkdev::ler`] devolve lista vazia, que e o que ela devolve
/// para tudo o que nao entende.
fn montar_sondagem(pedido: &Pedido) -> String {
    let Pedido { selo, .. } = pedido;

    let marcador = pedido.operacao.marcador();
    let log = log_do_job(pedido.operacao, None);
    let desfecho = arquivo_do_desfecho(pedido.operacao, None);
    let saida = format!("{log}/{ARQUIVO_DA_SONDAGEM}");

    let passos = [
        format!("mkdir -p {log}"),
        format!("echo {MARCA_DO_SELO}{selo} > {desfecho}"),
        format!(
            "if lsblk {FLAGS_DE_SONDAGEM} > {saida} 2>&1; \
             then echo {marcador}=OK >> {desfecho}; \
             else echo {marcador}=FALHOU >> {desfecho}; fi"
        ),
        format!("echo {MARCA_DO_FIM} >> {desfecho}"),
        format!("sleep {ESPERA_ANTES_DE_DESLIGAR}"),
        "poweroff".to_string(),
    ];

    passos.join("; ")
}

/// O porteiro de C-2, sobre a string ja montada.
///
/// E a ultima barreira, e nao a primeira: [`crate::nome::Nome`] ja recusou o
/// que o usuario digitou, e [`Disco`] e [`Selo`] ja recusaram o que veio de
/// dentro. Esta funcao existe para o caso que nenhum dos tres previu — uma
/// peca nova acrescentada por uma etapa futura, um caminho montado errado.
///
/// # Por que recusa *toda* aspa, e nao aspa desbalanceada
///
/// Um par balanceado de aspas simples dentro do `bash -c '...'` **nao** e
/// inofensivo: a primeira fecha a string do `bash -c`, e a segunda abre outra.
/// O que sai disso e sintaticamente valido e semanticamente outra coisa.
/// Contar aspas so daria a impressao de estar checando.
pub fn validar(comando: &str) -> Result<(), RecusaDaReceita> {
    if comando.contains('|') {
        return Err(RecusaDaReceita::Pipe);
    }
    if comando.contains('\'') {
        return Err(RecusaDaReceita::AspaSimples);
    }
    if comando.contains('"') {
        return Err(RecusaDaReceita::AspaDupla);
    }

    // As duas primeiras antes do `$` solto, para a mensagem nomear a marca
    // que quem lê vai procurar na string.
    for marca in ["$(", "${", "`", "$"] {
        if comando.contains(marca) {
            return Err(RecusaDaReceita::SubstituicaoDeComando { marca });
        }
    }

    for caractere in comando.chars() {
        if caractere.is_control() {
            return Err(RecusaDaReceita::CaractereDeControle {
                codigo: caractere as u32,
            });
        }
        if !caractere.is_ascii() {
            return Err(RecusaDaReceita::NaoAscii { caractere });
        }
    }

    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    /// As tres receitas que rodaram em hardware, preservadas em
    /// `recursos/capturas/` — ver `PROVENIENCIA.md`.
    const BACKUP_02: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-02.cfg");
    const BACKUP_03: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg");
    const RESTAURACAO_02: &str =
        include_str!("../recursos/capturas/grub-restauracao-arca-teste-02.cfg");
    const HELP: &str = include_str!("../recursos/capturas/ocs-sr-help.txt");

    /// O miolo do `bash -c '...'` de uma captura de `grub.cfg`.
    ///
    /// Extrai da captura em vez de repetir a string no teste: uma string
    /// repetida a mao prova que eu sei copiar, e o arquivo prova o que o
    /// hardware executou.
    fn receita_da_captura(grub_cfg: &str) -> String {
        const ABERTURA: &str = "ocs_live_run=\"bash -c '";
        const FECHAMENTO: &str = "'\"";

        let linha = grub_cfg
            .lines()
            .find(|linha| linha.contains(ABERTURA))
            .expect("a captura tem a linha $linux_cmd com a receita");

        let depois = &linha[linha.find(ABERTURA).unwrap() + ABERTURA.len()..];
        depois[..depois.find(FECHAMENTO).expect("a receita fecha")].to_string()
    }

    fn pedido(operacao: Operacao, nome: &str, disco: &str) -> Pedido {
        Pedido {
            operacao,
            nome: Some(Nome::novo(nome).expect("nome valido")),
            disco: Some(Disco::novo(disco).expect("disco valido")),
            selo: Selo::novo("a3f1c9e07b2d4856").expect("selo valido"),
        }
    }

    fn backup() -> Receita {
        Receita::montar(&pedido(Operacao::Backup, "2026-08-22_Apps", "nvme0n1")).unwrap()
    }

    fn restauracao() -> Receita {
        Receita::montar(&pedido(Operacao::Restauracao, "2026-08-22_Apps", "nvme0n1")).unwrap()
    }

    /// A terceira, da E11. Sem disco: o `ocs-chkimg` opera sobre a imagem.
    fn verificacao() -> Receita {
        Receita::montar(&Pedido {
            operacao: Operacao::Verificacao,
            nome: Some(Nome::novo("2026-08-22_Apps").expect("nome valido")),
            disco: None,
            selo: Selo::novo("a3f1c9e07b2d4856").expect("selo valido"),
        })
        .unwrap()
    }

    // ───────────────────────── transcricao ─────────────────────────
    //
    // Estes testes comparam com o que rodou em hardware, caractere a
    // caractere. Nenhum deles pode ser ajustado para passar: o oraculo e o
    // arquivo em `recursos/capturas/`.

    #[test]
    fn as_duas_capturas_de_backup_trazem_as_mesmas_flags() {
        // A premissa de tudo que vem depois: as duas execucoes de backup que
        // deram certo usaram a mesma sequencia de flags. Se divergissem, nao
        // haveria "o que rodou" no singular.
        for captura in [BACKUP_02, BACKUP_03] {
            assert!(
                receita_da_captura(captura).contains(FLAGS_DE_BACKUP),
                "a captura nao traz `{FLAGS_DE_BACKUP}`"
            );
        }
    }

    #[test]
    fn o_ocs_sr_do_backup_e_o_que_rodou_em_hardware() {
        let gerada =
            Receita::montar(&pedido(Operacao::Backup, "ARCA-TESTE-02", "nvme0n1")).unwrap();
        let original = receita_da_captura(BACKUP_02);

        let trecho = format!("ocs-sr {FLAGS_DE_BACKUP} savedisk ARCA-TESTE-02 nvme0n1");
        assert!(
            original.contains(&trecho),
            "o trecho nao esta na captura:\n  esperado: {trecho}\n  captura:  {original}"
        );
        assert!(
            gerada.comando().contains(&trecho),
            "a receita gerada nao traz o `ocs-sr` que rodou:\n{}",
            gerada.comando()
        );
    }

    #[test]
    fn o_ocs_chkimg_e_o_que_rodou_em_hardware() {
        // B-9 com saida redirecionada, transcrito de ARCA-TESTE-03 — a unica
        // das tres que redirecionou.
        let gerada =
            Receita::montar(&pedido(Operacao::Backup, "ARCA-TESTE-03", "nvme0n1")).unwrap();
        let original = receita_da_captura(BACKUP_03);

        let trecho = format!(
            "ocs-chkimg {FLAGS_DE_VERIFICACAO} {PARTIMAG} ARCA-TESTE-03 > {PARTIMAG}/ARCA-TESTE-03/{CHECK_LOG} 2>&1"
        );
        assert!(
            original.contains(&trecho),
            "o trecho nao esta na captura:\n  esperado: {trecho}\n  captura:  {original}"
        );
        assert!(
            gerada.comando().contains(&trecho),
            "a receita gerada nao traz o `ocs-chkimg` que rodou:\n{}",
            gerada.comando()
        );
    }

    #[test]
    fn o_ocs_sr_da_restauracao_muda_so_o_p_do_que_rodou() {
        // A unica divergencia deliberada com a restauracao validada:
        // `-p poweroff` vira `-p true`, porque S-4 e R-5 exigem escrever o
        // desfecho depois de o `ocs-sr` sair — e com `-p poweroff` a maquina
        // desliga antes do `echo`.
        let original = receita_da_captura(RESTAURACAO_02);
        let que_rodou = "ocs-sr -e1 auto -e2 -batch -j2 -k0 -iefi -p poweroff restoredisk ARCA-TESTE-02 nvme0n1";
        assert!(original.contains(que_rodou), "a captura mudou:\n{original}");

        let gerada =
            Receita::montar(&pedido(Operacao::Restauracao, "ARCA-TESTE-02", "nvme0n1")).unwrap();
        assert!(
            gerada.comando().contains(
                "ocs-sr -e1 auto -e2 -batch -j2 -k0 -iefi -p true restoredisk ARCA-TESTE-02 nvme0n1"
            ),
            "a receita gerada divergiu em mais do que o `-p`:\n{}",
            gerada.comando()
        );

        // E a diferenca e mesmo *so* essa: trocado o `-p`, as duas batem.
        assert_eq!(
            que_rodou.replace("-p poweroff", "-p true"),
            "ocs-sr -e1 auto -e2 -batch -j2 -k0 -iefi -p true restoredisk ARCA-TESTE-02 nvme0n1"
        );
    }

    #[test]
    fn os_parametros_de_boot_sao_os_das_tres_capturas() {
        // `ocs_repository`, `locales`, `keyboard-layouts` e `ocs_live_batch`
        // aparecem identicos nas tres — e sem eles a receita abre tela (§3.2).
        for captura in [BACKUP_02, BACKUP_03, RESTAURACAO_02] {
            let linha = captura
                .lines()
                .find(|linha| linha.contains("ocs_live_run=\"bash -c '"))
                .unwrap();

            for esperado in [
                "ocs_repository=\"dev:///LABEL=ARCAVAULT\"",
                "locales=en_US.UTF-8",
                "keyboard-layouts=NONE",
                "ocs_live_batch=\"yes\"",
            ] {
                assert!(linha.contains(esperado), "faltou `{esperado}` na captura");
            }
        }

        let parametros = backup().parametros_do_grub();
        for esperado in [
            "ocs_repository=\"dev:///LABEL=ARCAVAULT\"",
            "locales=en_US.UTF-8",
            "keyboard-layouts=NONE",
            "ocs_live_batch=\"yes\"",
        ] {
            assert!(
                parametros.contains(esperado),
                "faltou `{esperado}` nos parametros gerados:\n{parametros}"
            );
        }
    }

    #[test]
    fn a_receita_entra_no_ocs_live_run_como_nas_capturas() {
        // A forma da linha: `ocs_live_run="bash -c '<receita>'"`. E o que o
        // ADR-0002 decidiu e o que as tres capturas mostram.
        let receita = backup();
        let esperado = format!("ocs_live_run=\"bash -c '{}'\"", receita.comando());

        assert!(
            receita.parametros_do_grub().contains(&esperado),
            "a receita nao entra na linha como as capturas a escrevem:\n{}",
            receita.parametros_do_grub()
        );
    }

    #[test]
    fn o_destino_e_resolvido_por_label_e_nunca_por_letra() {
        // S-3. O `sda`/`nvme0n1` da receita e a **origem** do savedisk; o
        // destino e o `ocs_repository`, e ele e por LABEL.
        assert!(
            backup()
                .parametros_do_grub()
                .contains("dev:///LABEL=ARCAVAULT")
        );
    }

    // ───────────────────────── codigo novo ─────────────────────────
    //
    // Daqui para baixo nao havia original. Nenhuma receita real escreveu
    // `arca-fim.txt`, nenhuma usou `if/then/else`, nenhuma gravou selo. Estes
    // testes cobram a forma que se quer, e o marco da E7 e da E8 a confirmou
    // em hardware **pelo lado do backup**, em 22/08/2026: o `arca-fim.txt`
    // com selo e `ARCA_FIM` esta em `recursos/capturas/`.
    //
    // Pelo lado da **restauracao** continua sem original, e e o marco da E9 —
    // que muda o marcador (`ARCA_RESTORE=`) e o redirecionamento do
    // `arca-restore.log`, e nao a forma.

    #[test]
    fn nenhuma_receita_real_escreveu_arca_fim_txt() {
        // O achado que muda a etapa, provado aqui para nao virar folclore: o
        // mecanismo de desfecho de que a E5 e a E8 dependem nunca rodou. O
        // `arca-fim.txt` que existe no dispositivo veio de trabalho manual, o
        // mesmo padrao que o ADR-0003 registrou para o `ARCA_VEREDITO=`.
        for captura in [BACKUP_02, BACKUP_03, RESTAURACAO_02] {
            let original = receita_da_captura(captura);
            assert!(
                !original.contains(ARCA_FIM),
                "uma receita real escreve arca-fim.txt, e isto muda o que a E3 pode chamar de transcricao:\n{original}"
            );
            assert!(
                !original.contains("ARCA_SELO"),
                "uma receita real grava selo:\n{original}"
            );
            assert!(
                !original.contains("if "),
                "uma receita real usa if/then/else:\n{original}"
            );
        }
    }

    #[test]
    fn a_receita_grava_o_selo_na_primeira_linha_do_desfecho() {
        // §4.3: o Clonezilla devolve o selo na primeira linha do
        // `arca-fim.txt`. O `>` inicial e o que garante que ela e a primeira.
        let receita = backup();
        assert!(
            receita.comando().contains(&format!(
                "echo ARCA_SELO=a3f1c9e07b2d4856 > {PARTIMAG}/{ARCA_LOGS}/backup-2026-08-22_Apps/{ARCA_FIM}"
            )),
            "{}",
            receita.comando()
        );
    }

    #[test]
    fn o_desfecho_sai_do_codigo_de_saida_e_nunca_de_um_encadeamento() {
        // R-5. Encadear com `;` nao olha codigo de saida: uma falha deixaria
        // exatamente o mesmo rastro de um sucesso.
        for receita in [backup(), restauracao()] {
            let comando = receita.comando();
            let marcador = receita.operacao().marcador();

            assert!(
                comando.contains("if ocs-sr "),
                "a operacao nao esta dentro de um `if`:\n{comando}"
            );
            assert!(
                comando.contains(&format!("echo {marcador}=OK >>")),
                "faltou o ramo de exito:\n{comando}"
            );
            assert!(
                comando.contains(&format!("echo {marcador}=FALHOU >>")),
                "faltou o ramo de falha:\n{comando}"
            );
        }
    }

    #[test]
    fn a_receita_fecha_com_arca_fim_para_o_truncado_ser_reconhecivel() {
        // §5.5: "selo bate, sem ARCA_FIM" e desligamento no meio, e e falha.
        // Sem esta linha, truncado e completo sao indistinguiveis.
        for receita in [backup(), restauracao()] {
            assert!(
                receita.comando().contains("echo ARCA_FIM >>"),
                "{}",
                receita.comando()
            );
        }
    }

    #[test]
    fn a_verificacao_grava_o_veredito_com_marcador() {
        // ADR-0003: o marcador `ARCA_VEREDITO=` decide quando esta presente.
        // A E3 e quem o escreve — e e por isso que a decisao era desta etapa.
        let receita = backup();
        let comando = receita.comando();
        assert!(
            comando.contains("echo ARCA_VEREDITO=APROVADA >>"),
            "{comando}"
        );
        assert!(
            comando.contains("echo ARCA_VEREDITO=REPROVADA >>"),
            "{comando}"
        );
    }

    #[test]
    fn o_veredito_vai_para_dentro_da_pasta_da_imagem() {
        // E de la que `crate::imagens` o lê. Escrever em outro lugar faria o
        // `arca list` dizer "sem veredito" sobre toda imagem nova.
        assert!(
            backup()
                .comando()
                .contains(&format!("{PARTIMAG}/2026-08-22_Apps/{CHECK_LOG}")),
            "{}",
            backup().comando()
        );
    }

    #[test]
    fn a_verificacao_so_roda_se_houve_o_que_verificar() {
        // Com o `savedisk` falhando, a pasta da imagem pode nem existir: o
        // redirecionamento do `ocs-chkimg` falharia, e o ramo `else` dele
        // falharia junto. A verificacao mora dentro do ramo de exito.
        let receita = backup();
        let comando = receita.comando();

        let exito = comando.find("echo ARCA_BACKUP=OK").expect("ramo de exito");
        let falha = comando
            .find("echo ARCA_BACKUP=FALHOU")
            .expect("ramo de falha");
        let verificacao = comando.find("ocs-chkimg").expect("a verificacao de B-9");

        assert!(
            exito < verificacao && verificacao < falha,
            "a verificacao tem de estar entre o `then` e o `else` do backup:\n{comando}"
        );
    }

    #[test]
    fn a_restauracao_nao_verifica_nada_porque_nao_ha_o_que_verificar() {
        // B-9 e do backup. Na restauracao nao ha imagem nova para conferir —
        // e e por isso que P-6 dói mais la (ver o plano).
        assert!(!restauracao().comando().contains("ocs-chkimg"));
    }

    #[test]
    fn a_receita_espera_antes_de_desligar() {
        for receita in [backup(), restauracao()] {
            let comando = receita.comando();
            assert!(
                comando.contains(&format!("sleep {ESPERA_ANTES_DE_DESLIGAR}")),
                "{comando}"
            );
            assert!(comando.ends_with("poweroff"), "{comando}");
        }
    }

    #[test]
    fn o_log_do_job_e_criado_antes_de_qualquer_escrita_nele() {
        // Sem o `mkdir -p`, o primeiro `>` falha e o desfecho inteiro se
        // perde. A receita de ARCA-TESTE-03 usou `mkdir -p` pelo mesmo motivo.
        for receita in [backup(), restauracao()] {
            let comando = receita.comando();
            let esperado = format!(
                "mkdir -p {PARTIMAG}/{ARCA_LOGS}/{}-2026-08-22_Apps;",
                receita.operacao().nome()
            );
            assert!(comando.starts_with(&esperado), "{comando}");
        }
    }

    #[test]
    fn o_backup_e_a_restauracao_da_mesma_imagem_nao_dividem_o_desfecho() {
        // Toda receita comeca truncando o proprio `arca-fim.txt` com um `>`.
        // Com o caminho dependendo so do nome da imagem, um `arca restore X`
        // rodado antes de o backup de X ser colhido apagaria o desfecho dele
        // — e o §5.5 leria um backup bem-sucedido como desfecho ausente. O
        // selo nao salva isso: ele julga um desfecho encontrado, e nao serve
        // para nada quando o arquivo ja foi por cima.
        let nome = Nome::novo("2026-08-22_Apps").unwrap();

        assert_ne!(
            arquivo_do_desfecho(Operacao::Backup, Some(&nome)),
            arquivo_do_desfecho(Operacao::Restauracao, Some(&nome))
        );
    }

    #[test]
    fn o_desfecho_da_restauracao_mora_fora_da_imagem() {
        // §10.2: o `LOG` mora no `ARCAVAULT`, que a restauracao nao toca — a
        // imagem substitui o `nvme0n1`, e o desfecho sobrevive num disco que
        // nao estava no caminho.
        let receita = restauracao();
        let comando = receita.comando();
        assert!(
            comando.contains(&format!(
                "{PARTIMAG}/{ARCA_LOGS}/restauracao-2026-08-22_Apps/{ARCA_FIM}"
            )),
            "{comando}"
        );
        assert!(
            !comando.contains(&format!("{PARTIMAG}/2026-08-22_Apps/")),
            "o desfecho da restauracao nao pode morar dentro da imagem, que e a origem:\n{comando}"
        );
    }

    // ───────────────────────── C-2, o porteiro ─────────────────────────

    #[test]
    fn a_receita_montada_passa_pelo_proprio_validador() {
        for receita in [backup(), restauracao()] {
            assert_eq!(validar(receita.comando()), Ok(()));
        }
    }

    #[test]
    fn as_receitas_que_rodaram_em_hardware_passam_pelo_validador() {
        // Se o validador recusasse o que ja rodou, ele estaria errado —
        // recusar demais custa tanto quanto recusar de menos.
        for captura in [BACKUP_02, BACKUP_03, RESTAURACAO_02] {
            let original = receita_da_captura(captura);
            assert_eq!(
                validar(&original),
                Ok(()),
                "recusou o que rodou: {original}"
            );
        }
    }

    #[test]
    fn pipe_e_recusado() {
        // §3.2: medido. O Clonezilla descarta a string e abre o menu.
        assert_eq!(validar("ocs-sr | tee /log"), Err(RecusaDaReceita::Pipe));
        assert_eq!(validar("a || b"), Err(RecusaDaReceita::Pipe));
    }

    #[test]
    fn redirecionamento_simples_continua_valendo() {
        // A decisao 2 do plano: sem pipes, so `>` e `>>`. E `2>&1`, que as
        // tres capturas usam.
        assert_eq!(validar("echo x > /a; echo y >> /a; cmd > /b 2>&1"), Ok(()));
    }

    #[test]
    fn aspa_simples_e_recusada() {
        assert_eq!(validar("echo 'oi'"), Err(RecusaDaReceita::AspaSimples));
    }

    #[test]
    fn aspa_dupla_e_recusada() {
        assert_eq!(validar("echo \"oi\""), Err(RecusaDaReceita::AspaDupla));
    }

    #[test]
    fn substituicao_de_comando_e_recusada() {
        // O que se validou deixaria de ser o que roda.
        assert!(matches!(
            validar("echo $(whoami)"),
            Err(RecusaDaReceita::SubstituicaoDeComando { .. })
        ));
        assert!(matches!(
            validar("echo `whoami`"),
            Err(RecusaDaReceita::SubstituicaoDeComando { .. })
        ));
        assert!(matches!(
            validar("echo ${HOME}"),
            Err(RecusaDaReceita::SubstituicaoDeComando { .. })
        ));
    }

    #[test]
    fn quebra_de_linha_e_recusada() {
        // A receita e uma linha so do `grub.cfg`. Uma quebra dentro dela
        // transformaria o resto em outra diretiva do grub.
        assert!(matches!(
            validar("echo a\necho b"),
            Err(RecusaDaReceita::CaractereDeControle { .. })
        ));
        assert!(matches!(
            validar("echo a\r\necho b"),
            Err(RecusaDaReceita::CaractereDeControle { .. })
        ));
    }

    #[test]
    fn caractere_nao_ascii_e_recusado() {
        assert!(matches!(
            validar("echo Antônio"),
            Err(RecusaDaReceita::NaoAscii { .. })
        ));
    }

    // ────────────── a verificacao armada, etapa E11 (V-2) ──────────────

    #[test]
    fn a_verificacao_transcreve_o_ocs_chkimg_da_captura() {
        // O unico pedaco desta receita que tem original: a chamada do
        // `ocs-chkimg`, tirada de `grub-backup-arca-teste-03.cfg` e ja usada
        // pela receita de backup desde a E3. O oraculo e o arquivo.
        let comando = verificacao().comando().to_string();

        assert!(
            comando.contains("ocs-chkimg -b -or /home/partimag 2026-08-22_Apps"),
            "{comando}"
        );
        assert!(
            !comando.contains("savedisk") && !comando.contains("restoredisk"),
            "a verificacao nao toca em disco nenhum: {comando}"
        );
        assert!(
            !comando.contains("ocs-sr"),
            "o `ocs-sr` nao entra nesta receita: {comando}"
        );
    }

    #[test]
    fn a_verificacao_acrescenta_ao_check_log_em_vez_de_trunca_lo() {
        // **A diferenca que separa esta receita da de backup.** La o
        // redirecionamento e `>`, porque a imagem acabou de nascer e o log nao
        // existe. Aqui ele existe — e o veredito do backup que criou a imagem
        // —, e o `>` o destruiria: o §6.3 mostra `Imagem de origem: APROVADA —
        // veredito do backup que a criou`, e a frase viraria mentira.
        //
        // Pior: o `>` **trunca ao abrir**, antes de o comando rodar (§5.5,
        // medido). Um desligamento nessa janela deixaria uma imagem boa com o
        // log em zero byte, e ela apareceria `sem veredito` na listagem.
        let comando = verificacao().comando().to_string();

        assert!(
            comando.contains(">> /home/partimag/2026-08-22_Apps/arca-check.log 2>&1"),
            "a verificacao tem de ACRESCENTAR ao arca-check.log: {comando}"
        );
        // Com o espaco na frente, para nao casar dentro do proprio `>>` — que
        // e o que este teste pegou de si mesmo na primeira vez que rodou.
        assert!(
            !comando.contains(" > /home/partimag/2026-08-22_Apps/arca-check.log 2>&1"),
            "sobrou um `>` truncando o log: {comando}"
        );

        // E o backup continua com `>`, que e o certo la: a pasta da imagem
        // acabou de nascer, e o log com ela.
        let do_backup = backup().comando().to_string();
        assert!(
            do_backup.contains(" > /home/partimag/2026-08-22_Apps/arca-check.log 2>&1"),
            "a receita de backup nao pode ter mudado junto: {do_backup}"
        );
        assert!(
            !do_backup.contains(">> /home/partimag/2026-08-22_Apps/arca-check.log 2>&1"),
            "o backup nao acrescenta, ele cria: {do_backup}"
        );
    }

    #[test]
    fn a_verificacao_escreve_o_desfecho_e_o_veredito_nos_dois_ramos() {
        // O `ARCA_VERIFY=` e **codigo novo** — nenhuma receita real o
        // escreveu. O que ele acompanha e transcricao: o `if` sobre o
        // `ocs-chkimg` escrevendo `ARCA_VEREDITO=` rodou em 22/08/2026, dentro
        // do ramo de exito do backup.
        let comando = verificacao().comando().to_string();

        for marca in [
            "ARCA_VERIFY=OK",
            "ARCA_VERIFY=FALHOU",
            "ARCA_VEREDITO=APROVADA",
            "ARCA_VEREDITO=REPROVADA",
        ] {
            assert!(comando.contains(marca), "falta `{marca}`: {comando}");
        }
    }

    #[test]
    fn a_verificacao_tem_pasta_de_log_propria() {
        // **A razao de a verificacao ser uma `Operacao` e nao reusar
        // `Backup`.** Toda receita comeca truncando o proprio `arca-fim.txt`
        // com um `>`; dividir a pasta faria a verificacao apagar o desfecho de
        // um backup ainda nao colhido. A revisao da E3 pegou exatamente isto
        // entre o backup e a restauracao, e o selo nao cobre — ele julga um
        // desfecho **encontrado**.
        let nome = Nome::novo("2026-08-22_Apps").unwrap();
        assert_eq!(
            pasta_do_log(Operacao::Verificacao, Some(&nome)),
            "verificacao-2026-08-22_Apps"
        );

        let quatro = [
            pasta_do_log(Operacao::Backup, Some(&nome)),
            pasta_do_log(Operacao::Restauracao, Some(&nome)),
            pasta_do_log(Operacao::Verificacao, Some(&nome)),
            pasta_do_log(Operacao::Sondagem, None),
        ];
        let unicas: std::collections::BTreeSet<&String> = quatro.iter().collect();
        assert_eq!(unicas.len(), 4, "as quatro pastas tem de ser diferentes");
    }

    // ─────────────────── a quarta receita, da etapa E12 ───────────────────

    /// A sondagem. Sem nome e sem disco: ela lê os discos da maquina.
    fn sondagem() -> Receita {
        Receita::montar(&Pedido {
            operacao: Operacao::Sondagem,
            nome: None,
            disco: None,
            selo: Selo::novo("a3f1c9e07b2d4856").expect("selo valido"),
        })
        .expect("a receita da sondagem monta")
    }

    #[test]
    fn a_sondagem_nao_chama_programa_nenhum_do_clonezilla() {
        // SD-1, e e o que faz esta ser a unica etapa deste projeto cujo pior
        // caso nao envolve gravacao: sem `ocs-sr` nao ha `savedisk` nem
        // `restoredisk`, e nada e escrito fora do `ARCAVAULT`.
        let comando = sondagem().comando().to_string();

        for programa in ["ocs-sr", "ocs-chkimg", "savedisk", "restoredisk"] {
            assert!(
                !comando.contains(programa),
                "a sondagem chama `{programa}`: {comando}"
            );
        }
        assert!(comando.contains("lsblk"), "{comando}");
    }

    #[test]
    fn a_sondagem_poe_o_lsblk_dentro_de_um_if() {
        // **R-5, e a primeira forma escrita desta receita nao o tinha.** A
        // proposta encadeava com `;`, e o `;` nao olha codigo de saida: com o
        // `lsblk` falhando — uma flag que esta versao do util-linux nao conheca
        // basta —, o desfecho diria `OK` assim mesmo.
        //
        // O estrago e uma contradicao dentro da mesma sessao: o `arca resultado`
        // diria que a sondagem concluiu, e a tela seguinte diria
        // `Disco de origem ... POR DETERMINAR`.
        let comando = sondagem().comando().to_string();

        assert!(comando.contains("if lsblk "), "{comando}");
        assert!(comando.contains("then echo ARCA_PROBE=OK"), "{comando}");
        assert!(comando.contains("else echo ARCA_PROBE=FALHOU"), "{comando}");

        // E a forma que a proposta tinha nao pode voltar por descuido: um
        // `lsblk ... > ...; echo OK` e exatamente o que este teste proibe.
        assert!(
            !comando.contains(&format!("{ARQUIVO_DA_SONDAGEM}; echo")),
            "o `;` voltou entre o lsblk e o echo: {comando}"
        );
    }

    #[test]
    fn as_colunas_do_lsblk_sao_as_do_cabecalho_capturado() {
        // **Reconstrucao, e nao transcricao.** Temos o resultado — o
        // `blkdev.list` de dentro das imagens — e nao a linha de comando que o
        // produziu, que mora nos scripts dentro do `filesystem.squashfs`.
        //
        // O teste confere a reconstrucao contra o **original**: cada coluna do
        // cabecalho capturado tem de estar no `-o`, e na mesma ordem. Escrever
        // as colunas a mao aqui provaria que eu sei copiar a constante; ler do
        // arquivo prova que ela reproduz o que o Clonezilla escreveu.
        const CABECALHO: &str =
            "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL";

        let comando = sondagem().comando().to_string();
        let colunas: Vec<&str> = CABECALHO.split_whitespace().collect();

        assert!(
            comando.contains(&format!("-o {}", colunas.join(","))),
            "as colunas nao batem com o cabecalho de `{CABECALHO}`: {comando}"
        );

        // E o `-i`, que e o que mantem a arvore em ASCII: o boot passa
        // `locales=en_US.UTF-8`, e sem ele o `lsblk` desenharia a arvore com os
        // simbolos de caixa do Unicode — o arquivo deixaria de ter a forma do
        // que ele imita.
        assert!(comando.contains("lsblk -i "), "{comando}");
    }

    #[test]
    fn a_sondagem_escreve_o_erro_do_lsblk_dentro_do_proprio_arquivo() {
        // O que torna a reconstrucao das flags aceitavel: o modo de falha e
        // **barato e visivel**. Com `2>&1` apontando para o `blkdev.list`, uma
        // flag recusada deixa a mensagem do `lsblk` no dispositivo em vez de
        // sumir com o `poweroff` — e a proxima sessao lê qual foi.
        let comando = sondagem().comando().to_string();
        assert!(
            comando.contains(&format!("{ARQUIVO_DA_SONDAGEM} 2>&1")),
            "{comando}"
        );
    }

    #[test]
    fn a_sondagem_escreve_no_partimag_antes_de_qualquer_outra_coisa() {
        // **O unico pressuposto genuinamente novo da sondagem**, e ele ja tinha
        // original: se o repositorio nao estivesse montado neste instante, o
        // `mkdir` criaria a pasta no tmpfs da RAM e o `poweroff` levaria tudo
        // embora — falha silenciosa, sem nada no dispositivo para investigar.
        //
        // Quem monta o `/home/partimag` e o `ocs_repository=` do boot, e nao o
        // `ocs-sr`. A prova e da E11: a receita da verificacao tem esta mesma
        // forma e rodou em 23/08/2026, e os 51 bytes de
        // `recursos/capturas/arca-fim-verificacao-2026-08-22_Apps.txt` sairam
        // dos dois primeiros passos dela.
        let comando = sondagem().comando().to_string();
        let passos: Vec<&str> = comando.split("; ").collect();

        assert!(
            passos[0].starts_with("mkdir -p /home/partimag/"),
            "{comando}"
        );
        assert!(passos[1].starts_with("echo ARCA_SELO="), "{comando}");
        assert!(passos[2].starts_with("if lsblk"), "{comando}");
    }

    #[test]
    fn a_sondagem_grava_o_blkdev_list_ao_lado_do_proprio_desfecho() {
        // SD-4: pasta fixa, os dois arquivos juntos. `ARCA-LOGS\sondagem\` esta
        // **fora** da listagem de imagens (`imagens::RESERVADAS`), e por isso
        // um `blkdev.list` ali nao vira imagem nem residuo no `arca list`.
        let comando = sondagem().comando().to_string();

        assert!(
            comando.contains("/home/partimag/ARCA-LOGS/sondagem/blkdev.list"),
            "{comando}"
        );
        assert!(
            comando.contains("/home/partimag/ARCA-LOGS/sondagem/arca-fim.txt"),
            "{comando}"
        );
    }

    #[test]
    fn a_sondagem_termina_como_as_outras_tres() {
        // O que ela **transcreve**: o `ARCA_FIM` que separa desfecho completo
        // de truncado (§5.5), a espera que deixa o `echo` chegar ao disco e o
        // `poweroff`. Nada disto e novo, e por isso nada disto se reescreve.
        let comando = sondagem().comando().to_string();
        assert!(comando.ends_with("; sleep 20; poweroff"), "{comando}");
        assert!(comando.contains("echo ARCA_FIM >>"), "{comando}");
    }

    #[test]
    fn a_sondagem_com_nome_de_imagem_e_recusada() {
        // A coerencia do **nome**, no sentido que quase ninguem testa: a
        // sondagem nao opera sobre imagem nenhuma, e um nome carregado ate uma
        // receita que nao o usa e um valor que ninguem confere. E o mesmo
        // argumento do `disco` da E11, no outro eixo.
        let erro = Receita::montar(&Pedido {
            operacao: Operacao::Sondagem,
            nome: Some(Nome::novo("2026-08-22_Apps").unwrap()),
            disco: None,
            selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
        })
        .unwrap_err();

        assert_eq!(
            erro,
            RecusaDaReceita::NomeIncoerente {
                operacao: Operacao::Sondagem,
                tem_nome: true
            }
        );
    }

    #[test]
    fn as_tres_que_nomeiam_imagem_sem_nome_sao_recusadas() {
        // O outro sentido, e e o que dói: a pasta do desfecho sai do nome, e um
        // backup sem nome escreveria o desfecho na pasta da sondagem.
        for operacao in [
            Operacao::Backup,
            Operacao::Restauracao,
            Operacao::Verificacao,
        ] {
            let erro = Receita::montar(&Pedido {
                operacao,
                nome: None,
                disco: operacao
                    .nomeia_disco()
                    .then(|| Disco::novo("nvme0n1").unwrap()),
                selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
            })
            .unwrap_err();

            assert_eq!(
                erro,
                RecusaDaReceita::NomeIncoerente {
                    operacao,
                    tem_nome: false
                },
                "`{}` sem nome tinha de ser recusada",
                operacao.nome()
            );
        }
    }

    #[test]
    fn a_sondagem_passa_pelo_porteiro_de_c2_e_cabe_na_linha() {
        // A receita mais curta das quatro, e ainda assim ela passa pelas mesmas
        // barreiras: sem pipe, sem aspa, sem substituicao de comando, e dentro
        // do orcamento do `COMMAND_LINE_SIZE`.
        let receita = sondagem();
        assert!(validar(receita.comando()).is_ok());
        assert!(
            receita.parametros_do_grub().chars().count() < TETO_DOS_PARAMETROS,
            "a receita mais curta das quatro estourou o orcamento"
        );
    }

    #[test]
    fn a_verificacao_com_disco_e_recusada() {
        // A coerencia nos **dois** sentidos. Um disco carregado ate uma
        // receita que nao o usa e um valor que ninguem confere, e o lugar de
        // recusar e a unica porta por onde uma receita nasce.
        let erro = Receita::montar(&Pedido {
            operacao: Operacao::Verificacao,
            nome: Some(Nome::novo("2026-08-22_Apps").unwrap()),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
        })
        .unwrap_err();

        assert_eq!(
            erro,
            RecusaDaReceita::DiscoIncoerente {
                operacao: Operacao::Verificacao,
                tem_disco: true
            }
        );
    }

    #[test]
    fn backup_e_restauracao_sem_disco_sao_recusados() {
        for operacao in [Operacao::Backup, Operacao::Restauracao] {
            let erro = Receita::montar(&Pedido {
                operacao,
                nome: Some(Nome::novo("2026-08-22_Apps").unwrap()),
                disco: None,
                selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
            })
            .unwrap_err();

            assert_eq!(
                erro,
                RecusaDaReceita::DiscoIncoerente {
                    operacao,
                    tem_disco: false
                },
                "{} sem disco tinha de ser recusado",
                operacao.nome()
            );
        }
    }

    #[test]
    fn a_verificacao_passa_por_c2_como_as_outras_duas() {
        // O porteiro e o mesmo, e a receita menor nao ganha passe. Ela e
        // montada e validada pela mesma `Receita::montar`, entao ter uma em
        // maos ja e a garantia de C-2 — e este teste cobra que a string de
        // fato passa pelo validador.
        assert!(validar(verificacao().comando()).is_ok());
    }

    // ───────────────────────── o ensaio em bash ─────────────────────────

    /// O que o `bash` faz com a receita, e nao o que ela contem.
    ///
    /// Os testes acima provam que a string tem os pedacos certos. Nenhum
    /// deles prova que o `if/then/else` **aninhado** ramifica como se quer —
    /// e ele e codigo novo, que nenhuma execucao real exercitou (P-16 do
    /// PRD). Quem responde isso e `recursos/ensaio-da-receita.sh`, que roda
    /// as duas receitas num bash de verdade com o Clonezilla substituido por
    /// comandos falsos, e confere o rastro de cada desfecho.
    ///
    /// Ele mora fora do `cargo test` porque precisa de bash, que nem toda
    /// maquina Windows tem. O preco disso e ele poder ficar para tras quando
    /// a receita mudar — e e esse preco que este teste paga: se as strings
    /// divergirem, o ensaio esta ensaiando outra coisa.
    const ENSAIO: &str = include_str!("../recursos/ensaio-da-receita.sh");

    #[test]
    fn o_ensaio_em_bash_ensaia_a_receita_de_hoje() {
        for receita in [backup(), restauracao(), verificacao(), sondagem()] {
            let comando = receita
                .comando()
                .replace("2026-08-22_Apps", "ARCA-TESTE-02");
            assert!(
                ENSAIO.contains(&comando),
                "`recursos/ensaio-da-receita.sh` ficou para tras da receita de {}.\n\
                 Rode `cargo run --example receita_ao_lado_da_que_rodou`, cole a string\n\
                 nova no script e rode `bash recursos/ensaio-da-receita.sh` de novo.\n\
                 \nA receita de hoje:\n{comando}",
                receita.operacao().nome()
            );
        }
    }

    #[test]
    fn a_receita_cabe_na_linha_de_comando_do_kernel_com_o_nome_mais_longo() {
        // O modo de falha aqui e o pior que existe: estourar o
        // `COMMAND_LINE_SIZE` nao da erro, o kernel **trunca em silencio**, e
        // uma receita truncada faz o Clonezilla abrir o menu interativo — que
        // e indistinguivel de "o boot nao funcionou".
        //
        // O nome mais longo que B-2 aceita tem de caber com folga, e a folga
        // e o que este teste mede. Se um dia nao couber, ou a receita
        // encurta, ou `crate::nome::LIMITE` baixa.
        let mais_longo = "n".repeat(crate::nome::LIMITE);

        for operacao in [Operacao::Backup, Operacao::Restauracao] {
            let receita = Receita::montar(&Pedido {
                operacao,
                nome: Some(Nome::novo(&mais_longo).expect("o nome mais longo que B-2 aceita")),
                disco: Some(Disco::novo("nvme0n1").unwrap()),
                selo: Selo::de_ensaio(),
            })
            .unwrap_or_else(|recusa| {
                panic!(
                    "o nome mais longo que B-2 aceita nao cabe na receita de {}: {recusa}",
                    operacao.nome()
                )
            });

            let ocupa = receita.parametros_do_grub().chars().count();
            assert!(
                ocupa <= TETO_DOS_PARAMETROS,
                "a {} com o nome mais longo ocupa {ocupa}, e o orcamento e {TETO_DOS_PARAMETROS}",
                operacao.nome()
            );
        }
    }

    #[test]
    fn o_orcamento_da_linha_e_maior_do_que_tudo_que_ja_rodou() {
        // A reserva para o `menuentry` base foi medida nas capturas. Se um
        // `grub.cfg` novo trouxer uma base maior do que o reservado, o
        // orcamento esta errado — e este teste avisa antes de a linha
        // estourar em hardware.
        for captura in [BACKUP_02, BACKUP_03, RESTAURACAO_02] {
            let linha = captura
                .lines()
                .find(|linha| linha.contains("ocs_live_run=\"bash -c '"))
                .unwrap()
                .trim();

            let inicio = linha.find("locales=").expect("a linha tem `locales=`");
            let fim = linha
                .find("ocs_live_batch=\"yes\"")
                .expect("e `ocs_live_batch`")
                + "ocs_live_batch=\"yes\"".len();

            let base = linha.chars().count() - linha[inicio..fim].chars().count();
            assert!(
                base <= RESERVADO_PARA_O_MENUENTRY,
                "o menuentry base desta captura ocupa {base}, e a reserva e {RESERVADO_PARA_O_MENUENTRY}"
            );
        }
    }

    #[test]
    fn uma_receita_que_nao_coubesse_na_linha_seria_recusada() {
        // A recusa existe de verdade, e nao so no papel: um nome longo o
        // bastante a dispara. Ele nao passaria por B-2 — e e por isso que
        // este teste monta o `Nome` por dentro, para exercitar a barreira que
        // fica **depois** dela.
        let receita = Receita::montar(&Pedido {
            operacao: Operacao::Backup,
            nome: Some(Nome::sem_julgar_para_teste(&"n".repeat(400))),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            selo: Selo::de_ensaio(),
        });

        assert!(matches!(
            receita,
            Err(RecusaDaReceita::LinhaLongaDemais { .. })
        ));
    }

    #[test]
    fn cada_recusa_da_receita_tem_mensagem_propria() {
        let recusas = [
            RecusaDaReceita::Pipe,
            RecusaDaReceita::AspaSimples,
            RecusaDaReceita::AspaDupla,
            RecusaDaReceita::SubstituicaoDeComando { marca: "$(" },
            RecusaDaReceita::CaractereDeControle { codigo: 10 },
            RecusaDaReceita::NaoAscii { caractere: 'ô' },
            RecusaDaReceita::DiscoVazio,
            RecusaDaReceita::DiscoInvalido { caractere: '/' },
            RecusaDaReceita::DiscoNaoComecaComLetra,
            RecusaDaReceita::SeloInvalido {
                tem: "xyz".to_string(),
            },
        ];

        let mensagens: Vec<String> = recusas.iter().map(|r| r.to_string()).collect();
        for mensagem in &mensagens {
            assert!(!mensagem.is_empty());
            assert_eq!(
                mensagens.iter().filter(|outra| *outra == mensagem).count(),
                1,
                "duas recusas com a mesma mensagem: {mensagem}"
            );
        }
    }

    // ───────────────────────── disco e selo ─────────────────────────

    #[test]
    fn os_discos_reais_passam() {
        // `nvme0n1` e o disco desta maquina, nomeado nas tres capturas.
        for bruto in ["nvme0n1", "sda", "sdb", "hda", "mmcblk0"] {
            assert!(Disco::novo(bruto).is_ok(), "`{bruto}` foi recusado");
        }
    }

    #[test]
    fn disco_com_caminho_ou_metacaractere_e_recusado() {
        for bruto in ["/dev/sda", "sda;poweroff", "sda 1", "SDA", "sda'", "../sda"] {
            assert!(Disco::novo(bruto).is_err(), "`{bruto}` passou");
        }
        assert_eq!(Disco::novo("").unwrap_err(), RecusaDaReceita::DiscoVazio);
        assert_eq!(
            Disco::novo("1sda").unwrap_err(),
            RecusaDaReceita::DiscoNaoComecaComLetra
        );
    }

    #[test]
    fn o_selo_e_hexadecimal_de_dezesseis_digitos() {
        // A forma de §10.1 do PRD.
        assert!(Selo::novo("a3f1c9e07b2d4856").is_ok());
        assert!(Selo::novo("7e02b4d1af963c85").is_ok());

        for bruto in [
            "",
            "a3f1c9e07b2d485",   // curto
            "a3f1c9e07b2d48567", // longo
            "A3F1C9E07B2D4856",  // maiuscula
            "g3f1c9e07b2d4856",  // fora do hexadecimal
            "a3f1c9e0 7b2d4856",
        ] {
            assert!(Selo::novo(bruto).is_err(), "`{bruto}` passou como selo");
        }
    }

    #[test]
    fn o_selo_de_ensaio_tem_a_forma_de_um_selo_e_nao_se_confunde_com_um() {
        let ensaio = Selo::de_ensaio();
        assert!(Selo::novo(ensaio.como_texto()).is_ok());
        assert_eq!(ensaio.como_texto(), "0000000000000000");
    }

    // ───────────────────────── o help, como evidencia ─────────────────────────

    #[test]
    fn o_help_desta_versao_confirma_o_que_as_flags_fazem() {
        // As decisoes sobre `-scs`, `-p` e `-batch` foram tomadas com este
        // arquivo na mao. Se um dia o help mudar, este teste avisa antes de a
        // decisao ficar sem base.
        assert!(
            HELP.contains("-scs, --skip-check-restorable"),
            "`-scs` nao e mais o que pula a verificacao"
        );
        assert!(
            HELP.contains(
                "By default Clonezilla will check the image if restorable after it is created"
            ),
            "a conferencia nativa deixou de ser o padrao"
        );
        assert!(
            HELP.contains("-p, --postaction [choose|poweroff|reboot|command|CMD]"),
            "`-p` mudou de forma"
        );
        assert!(
            HELP.contains("reboot (default)"),
            "o padrao de `-p` deixou de ser reboot, e e por isso que `-p true` existe na receita"
        );
        assert!(
            HELP.contains("You have to use '-batch' instead of '-b' when you want to use it in the boot parameters"),
            "a razao de ser `-batch` e nao `-b` sumiu do help"
        );
    }

    #[test]
    fn o_help_diz_que_destino_menor_ja_e_recusado_pelo_proprio_clonezilla() {
        // Anotado na E3 como fora de escopo, **resolvido na E9**: a premissa
        // de R-7 estava errada. O help diz que por padrao o Clonezilla confere
        // o tamanho do destino e **desiste**, em vez de corromper, e que
        // `-icds` e quem desligaria isso.
        //
        // A recusa do ARCA fica, e a razao passa a ser **onde** ela acontece:
        // a do Clonezilla e do outro lado do reinicio, e custa um boot de uma
        // operacao destrutiva. Este teste continua guardando o que importa
        // aqui — que a receita nao desligue a conferencia dele. Ver
        // `docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md`.
        assert!(
            HELP.contains("-icds, --ignore-chk-dsk-size-pt"),
            "`-icds` sumiu do help"
        );
        assert!(
            HELP.contains("By default it will be checked and if the size is smaller than the source disk, quit"),
            "a conferencia de tamanho deixou de ser o padrao"
        );
        assert!(
            !restauracao().comando().contains("-icds"),
            "a receita nao pode desligar a conferencia de tamanho: e a defesa que R-7 supunha nao existir"
        );
    }
}
