//! O nome que o **Linux** da ao disco de origem, e de onde ele vem.
//!
//! `src/comandos/backup.rs` trazia `DISCO_SUPOSTO: &str = "nvme0n1"` fixo, com
//! um comentario dizendo que a E6 o descobriria. O problema e mais duro do que
//! a constante deixava parecer: **`nvme0n1` e o nome do disco no Linux, e o
//! Windows nao o conhece.** Nenhuma API do Windows responde por ele.
//!
//! # Os dois caminhos, e por que so um deles entrou
//!
//! **O que entrou: o `blkdev.list` de dentro de cada imagem.** Toda imagem do
//! Clonezilla carrega a tabela inteira do `lsblk`, com o nome Linux e o modelo
//! lado a lado. Conferido nas duas imagens deste dispositivo:
//!
//! ```text
//! sda       sda         238.5G disk                     KGSSE100256
//! nvme0n1   nvme0n1     465.8G disk                     KINGSTON SNV3S500G
//! ```
//!
//! O WMI diz que o disco onde o `C:` mora e `KINGSTON SNV3S500G`. Os dois
//! concordam, e o nome sai de uma **medicao**, nao de uma regra. O preco e que
//! o oraculo so existe depois do primeiro backup.
//!
//! **O que ficou de fora: derivar do `BusType` e do `Index`.** O WMI responde
//! `NVMe` e `0`, e `NVMe + indice N -> nvmeNn1` e plausivel. Nao entrou porque
//! **nao e medido**: o indice do Windows nao e o do Linux por construcao, e
//! aqui os dois coincidem por acaso — esta maquina tem um NVMe so. Numa com
//! dois, um `nvme1n1` viraria `nvme0n1` e a receita nomearia o disco errado.
//!
//! Este projeto ja documentou tres vezes como fundacao validada algo que veio
//! do trabalho de validacao em volta dela (ADR-0003, ADR-0004, ADR-0005).
//! Inventar uma derivacao e chama-la de descoberta seria a quarta. **Nao
//! havendo imagem de onde lê, o nome fica por determinar** — e isso e uma
//! resposta, desde que escrita, e desde que a E7 saiba que herdou.
//!
//! # A comparacao de modelo, e os dois ajustes que ela precisa
//!
//! O modelo e comparado sem caixa e sem os caracteres que nao sao letra nem
//! digito, porque o WMI escreve `KGSSE100 256` e o `lsblk` escreve
//! `KGSSE100256` — o mesmo texto com um espaco a mais.
//!
//! Sobre isso ha **dois** afixos a tirar, e os dois sao a mesma coisa: o
//! Windows fala com disco por uma traducao SCSI, e o que ela poe em volta do
//! modelo entra no `Win32_DiskDrive.Model` como se fosse do fabricante.
//!
//! | | Windows | `lsblk` | medido em |
//! |---|---|---|---|
//! | sufixo `SCSI Disk Device` | `KGSSE100 256 SCSI Disk Device` | `KGSSE100256` | 22/08/2026 |
//! | prefixo `NVMe` | `NVMe EG6 KIOXIA 1024GB` | `EG6 KIOXIA 1024GB` | 31/08/2026 |
//!
//! O prefixo custou um `arca backup` recusado numa maquina que estava inteira:
//! dispositivo preparado, sondagem colhida, e `POR DETERMINAR` na linha do
//! disco de origem. Que o disco desta mesa (`KINGSTON SNV3S500G`) nao tenha o
//! prefixo e propriedade **do disco**, e nao da regra — quem o poe e a
//! traducao SCSI, que preenche o *vendor* com `NVMe` quando o controlador nao
//! da outro. O proprio Windows entrega a decomposicao, e foi assim que se viu:
//! o `MSFT_Disk` do mesmo disco responde `Manufacturer=NVMe` e
//! `Model=EG6 KIOXIA 1024GB`, que e exatamente o que o `lsblk` diz.
//!
//! **Ler o modelo do `MSFT_Disk` em vez de tirar o prefixo foi considerado e
//! recusado**: o `Win32_DiskDrive` sempre responde e o `MSFT_Disk` depende do
//! servico de armazenamento estar de pe (é por isso que so a *medida* de
//! ADR-0010 vem de la, e com `Option`). Pendurar o campo que nomeia o disco de
//! origem numa fonte que pode faltar troca uma recusa por outra.
//!
//! **Nao casar e recusa, e nunca um palpite.** Um nome de disco errado numa
//! receita destrutiva e o pior desfecho possivel deste modulo.

use crate::receita::Disco;
use chrono::{DateTime, Local};
use std::fmt;

/// O nome do arquivo que carrega este formato, nas **duas** fontes.
///
/// Dentro de cada imagem quem o escreve e o Clonezilla; em
/// `ARCA-LOGS\sondagem\` quem o escreve e a receita da sondagem (E12). O nome
/// e o mesmo de proposito: e o mesmo formato e o mesmo parser — este modulo —,
/// e um nome diferente sugeriria um segundo formato, que nao existe.
///
/// Mora aqui, e nao em quem escreve, porque quem lê e um so e quem escreve sao
/// dois.
pub const ARQUIVO: &str = "blkdev.list";

/// Uma linha do `blkdev.list` que descreve um disco inteiro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoDaImagem {
    /// O nome do Linux: `nvme0n1`, `sda`.
    pub nome: String,
    /// O modelo, como o `lsblk` o escreveu.
    pub modelo: String,
}

/// As colunas que o cabecalho do `lsblk` precisa trazer para o arquivo servir.
const COLUNAS: [&str; 3] = ["NAME", "TYPE", "MODEL"];

/// Os discos que um `blkdev.list` descreve.
///
/// # Por deslocamento de coluna, e nao por contagem de campos
///
/// O `lsblk` alinha as colunas em largura fixa, e **o `MODEL` tem espaco
/// dentro** (`KINGSTON SNV3S500G`). Repartir a linha por espaco daria dois
/// campos onde ha um. Colunas vazias pioram: a linha do `sda` nao tem
/// `FSTYPE` nem `MOUNTPOINT`, e contar campos da posicoes diferentes por
/// linha.
///
/// O deslocamento sai do **cabecalho do proprio arquivo**, e nao de larguras
/// escritas aqui: o `lsblk` dimensiona cada coluna pelo maior valor daquela
/// execucao. Como o `MODEL` e a ultima, tudo dali ate o fim da linha e o
/// modelo — o que sobrevive a qualquer coluna anterior ter crescido.
pub fn ler(texto: &str) -> Vec<DiscoDaImagem> {
    let mut linhas = texto.lines();
    let Some(cabecalho) = linhas.next() else {
        return Vec::new();
    };

    // O que nao se entende e recusado, e nao adivinhado: um cabecalho sem as
    // colunas esperadas nao vira uma lista vazia por acaso — vira uma lista
    // vazia de proposito, e quem chama trata como "nao ha oraculo".
    let mut deslocamentos = Vec::with_capacity(COLUNAS.len());
    for coluna in COLUNAS {
        match achar_coluna(cabecalho, coluna) {
            Some(posicao) => deslocamentos.push(posicao),
            None => return Vec::new(),
        }
    }
    let (nome_em, tipo_em, modelo_em) = (deslocamentos[0], deslocamentos[1], deslocamentos[2]);

    let mut discos = Vec::new();
    for linha in linhas {
        let caracteres: Vec<char> = linha.chars().collect();

        // Uma linha curta demais para alcancar a coluna do tipo nao descreve
        // dispositivo nenhum.
        if caracteres.len() <= tipo_em {
            continue;
        }

        let ate_o_fim =
            |de: usize| -> String { caracteres[de.min(caracteres.len())..].iter().collect() };

        // `disk` e o que separa um disco de uma particao (`part`) e de um
        // dispositivo de loop (`loop`). So o disco interessa: e ele que a
        // receita nomeia.
        let tipo = ate_o_fim(tipo_em);
        if tipo.split_whitespace().next() != Some("disk") {
            continue;
        }

        // Da coluna `NAME`, e nao do primeiro campo da linha — que e o
        // `KNAME`. Os dois coincidem para `sda` e `nvme0n1`, e e por isso que
        // a primeira versao disto passava em todo teste: ela calculava o
        // deslocamento de `NAME`, avisava no comentario que confundi-lo com
        // `KNAME` daria a coluna errada, e entao **descartava o deslocamento**
        // para pegar o primeiro campo. Num disco em que os dois diferem — um
        // multipath tem `NAME=mpatha` e `KNAME=dm-0` — o nome lido seria o do
        // dispositivo de baixo, e a receita nomearia outra coisa.
        let Some(nome) = ate_o_fim(nome_em)
            .split_whitespace()
            .next()
            .map(str::to_string)
        else {
            continue;
        };
        let modelo = ate_o_fim(modelo_em).trim().to_string();
        if modelo.is_empty() {
            continue;
        }

        discos.push(DiscoDaImagem { nome, modelo });
    }

    discos
}

/// A coluna no cabecalho, casada como **palavra inteira**.
///
/// `NAME` aparece dentro de `KNAME`, e casar por substring daria a coluna
/// errada — a primeira, que e a do `KNAME`.
fn achar_coluna(cabecalho: &str, coluna: &str) -> Option<usize> {
    let mut de = 0usize;
    while let Some(achado) = cabecalho[de..].find(coluna) {
        let inicio = de + achado;
        let fim = inicio + coluna.len();

        let antes_e_branco = inicio == 0
            || cabecalho[..inicio]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace);
        let depois_e_branco = cabecalho[fim..]
            .chars()
            .next()
            .is_none_or(char::is_whitespace);

        if antes_e_branco && depois_e_branco {
            // Em caracteres, e nao em bytes: o alinhamento que o `lsblk`
            // desenha e de caracteres.
            return Some(cabecalho[..inicio].chars().count());
        }
        de = fim;
    }
    None
}

/// De onde o nome do disco veio, e a saida **sempre diz**.
///
/// A E3 estabeleceu o padrao: ela imprime `disco nvme0n1 (suposto)` e diz de
/// onde ele veio. Uma receita destrutiva que nomeie um disco sem dizer a
/// origem do nome e pior do que nao imprimir nada.
///
/// # A segunda variante nasceu na E12, e ela **tinha** de nascer
///
/// Ate a E12 havia uma fonte so, e a tela imprimia `lido de <imagem>/blkdev.list`
/// literalmente. Com a sondagem, uma segunda fonte responde a mesma pergunta —
/// e deixa-la se apresentar como imagem seria a falha que o `arca prepare`
/// acabou de pagar na E10: uma tela afirmando o que nao aconteceu. Nao ha
/// imagem nenhuma no dispositivo em que a sondagem mais importa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origem {
    /// Lido do `blkdev.list` de uma imagem, casando o modelo com o que o WMI
    /// diz do disco onde o `C:` mora.
    LidoDaImagem { imagem: String, modelo: String },

    /// Lido do `blkdev.list` que **a sondagem** gravou (SD-2, SD-5).
    LidoDaSondagem {
        modelo: String,

        /// Quando o arquivo da sondagem foi escrito — o `mtime` que o sistema
        /// de arquivos devolve, **carimbado pelo relogio do Clonezilla**.
        ///
        /// # Por que a tela precisa disto, e por que ele nao decide nada
        ///
        /// Uma sondagem descreve a maquina do instante em que rodou. Sem a
        /// data, `lido da sondagem` nao distingue a de cinco minutos atras da
        /// de um mês, e a segunda pode estar descrevendo um disco que nao esta
        /// mais na maquina.
        ///
        /// **E informativo, nunca comparado** (S-6): quem julga se o disco
        /// achado e o certo continua sendo o **modelo**, e nao o tempo. Este
        /// campo so e impresso, como o `dia_e_mes` das imagens no `arca list`.
        /// `None` quando o sistema de arquivos nao soube responder.
        ///
        /// # De quem e este relogio, e a primeira versao disto errou
        ///
        /// **Quem escreve o arquivo e o `lsblk`, do outro lado do reinicio**, e
        /// o Windows so o lê. A doc deste campo dizia *"pelo relogio do
        /// Windows, e nao do live"* — exatamente ao contrario —, e o marco de
        /// 24/08/2026 desmentiu em uma linha: a sondagem foi armada as
        /// **14:56:55** e a tela imprimiu `lido da sondagem de 24/08 11:58`.
        /// Tres horas atras, que e P-7 pelo lado de sempre.
        ///
        /// O valor fica como esta, e **nao** se soma nada a ele: corrigir por
        /// deducao seria fabricar um instante que ninguem mediu. O que muda e a
        /// tela, que passa a dizer de quem e o carimbo — ver [`NomeDoDisco`].
        /// Para o que este campo existe, o deslocamento nao atrapalha: duas
        /// sondagens vem do **mesmo** relogio, e a distancia entre elas e real.
        quando: Option<DateTime<Local>>,

        /// O que as imagens diziam do mesmo modelo, quando elas dizem outra
        /// coisa (SD-5).
        ///
        /// A sondagem ganha — ela descreve a maquina de **agora**, e a imagem
        /// descreve a de quando o backup foi feito —, e a divergencia e
        /// **dita**, nunca resolvida em silencio. `None` e o caso normal: ou
        /// nao ha imagem, ou as duas concordam.
        divergencia: Option<Divergencia>,
    },
}

/// Duas fontes do §4.5 respondendo nomes diferentes para o mesmo modelo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergencia {
    /// A imagem cujo `blkdev.list` diz outra coisa.
    pub imagem: String,
    /// O nome que ela da ao disco daquele modelo.
    pub disco: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NomeDoDisco {
    pub disco: Disco,
    pub origem: Origem,
}

impl fmt::Display for NomeDoDisco {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.origem {
            Origem::LidoDaImagem { imagem, modelo } => write!(
                f,
                "{} · lido de {imagem}/blkdev.list, casando o modelo `{modelo}`",
                self.disco
            ),
            Origem::LidoDaSondagem {
                modelo,
                quando,
                divergencia,
            } => {
                // O carimbo vem nomeado, e não é zelo: ele está **três horas
                // atrás** do relógio do Windows (P-7), porque quem escreveu o
                // arquivo foi o `lsblk` do outro lado do reinício. Sem dizer de
                // quem é, quem comparasse com o `armado_em` do `estado.json`
                // concluiria que a sondagem é mais velha do que é — e essa é a
                // conta que S-6 existe para ninguém fazer.
                write!(
                    f,
                    "{} · lido da sondagem de {} (carimbo do Clonezilla, P-7), casando o modelo `{modelo}`",
                    self.disco,
                    crate::formato::dia_e_hora(*quando)
                )?;
                // A divergencia sai na **mesma linha** de propósito: quem lê o
                // nome do disco tem de lê junto que ha outra fonte dizendo
                // outra coisa. Uma linha separada seria pulavel.
                if let Some(divergencia) = divergencia {
                    write!(
                        f,
                        " · DIVERGE de {}/blkdev.list, que diz `{}`",
                        divergencia.imagem, divergencia.disco
                    )?;
                }
                Ok(())
            }
        }
    }
}

/// Por que o nome do disco nao foi determinado.
///
/// Cada motivo tem mensagem propria porque as saidas sao diferentes: sem
/// imagem, o usuario faz o primeiro backup de outro jeito; com modelo que nao
/// casa, ha alguma coisa a olhar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemNome {
    /// Nenhuma das duas fontes traz um `blkdev.list` legivel: nao ha sondagem,
    /// e nenhuma imagem tem um.
    ///
    /// **Desde a E12 esta recusa tem saida**, e e por isso que ela e a unica
    /// da sondagem que deixa as imagens falar: quem cai aqui roda
    /// `arca sondar`, e um reinicio depois o oraculo existe. Ate a E11 a saida
    /// era um backup pelo menu do Clonezilla — que e o que este app existe
    /// para nao precisar.
    SemOraculo,

    /// Ha `blkdev.list`, e nenhum disco nele casa com o modelo do disco de
    /// origem.
    ModeloNaoCasa { modelo: String },

    /// Mais de um disco casa. Nao se escolhe: dois discos do mesmo modelo sao
    /// indistinguiveis por aqui, e chutar seria nomear um disco na receita.
    ModeloAmbiguo { modelo: String, quantos: usize },

    /// O nome achado nao passa pelo validador de [`Disco`].
    NomeInvalido { tem: String },
}

impl fmt::Display for SemNome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemNome::SemOraculo => write!(
                f,
                "nao ha `blkdev.list` legivel no dispositivo — nem de sondagem, nem dentro de imagem nenhuma —, e e dele que sai o nome que o Linux da ao disco. O Windows nao conhece esse nome, e o ARCA nao o inventa. Para produzi-lo:  arca sondar"
            ),
            SemNome::ModeloNaoCasa { modelo } => write!(
                f,
                "nenhum disco dos `blkdev.list` do dispositivo tem o modelo `{modelo}`, que e o do disco onde o Windows esta. O que ha ali foi escrito noutra maquina. Para descrever esta:  arca sondar"
            ),
            SemNome::ModeloAmbiguo { modelo, quantos } => write!(
                f,
                "{quantos} discos com o modelo `{modelo}` aparecem no `blkdev.list`, e nao ha como saber qual e o de origem. O ARCA nao escolhe um disco no chute"
            ),
            SemNome::NomeInvalido { tem } => write!(
                f,
                "o `blkdev.list` traz `{tem}` como nome de disco, e ele nao tem a forma de um nome do Linux. Para gravar um arquivo novo:  arca sondar"
            ),
        }
    }
}

/// Um `blkdev.list` lido, e de onde ele veio.
///
/// # Por que a fonte viaja junto do texto
///
/// Desde a E12 ha **duas** fontes para o mesmo formato, e a tela tem de dizer
/// qual respondeu ([`Origem`]). Passar so os textos obrigaria quem chama a
/// lembrar a ordem em que os pôs na lista, e "lembrar a ordem" e como se
/// escreve um erro que nenhum teste pega.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lista {
    pub fonte: Fonte,
    pub texto: String,
}

/// De onde um [`Lista`] veio.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fonte {
    /// O `blkdev.list` de dentro de uma imagem, pelo nome dela.
    Imagem(String),

    /// O `blkdev.list` que a sondagem gravou, com o `mtime` do arquivo.
    Sondagem { quando: Option<DateTime<Local>> },
}

/// O nome Linux do disco de `modelo`, procurado nos `blkdev.list` dados.
///
/// Recebe os arquivos ja lidos, com a fonte de cada um, e nao os lê: manter
/// isto puro e o que permite testar as quatro recusas sem dispositivo
/// conectado.
///
/// # A sondagem ganha das imagens, e a divergencia e dita (SD-5)
///
/// As duas fontes respondem a mesma pergunta sobre instantes diferentes: a
/// sondagem descreve a maquina de **agora**, e a imagem descreve a de quando o
/// backup foi feito. Um disco trocado entre as duas faz a imagem responder o
/// nome de um disco que nao esta mais la.
///
/// Entao a sondagem e consultada primeiro, sozinha. Respondendo, e a resposta
/// — e o que as imagens dizem do mesmo modelo entra como
/// [`Origem::LidoDaSondagem::divergencia`], para a tela. **Nao** respondendo —
/// nao ha sondagem, ela esta ilegivel, ou o modelo nao casa nela —, as imagens
/// respondem exatamente como respondiam antes da E12.
///
/// Vale registrar que a defesa velha continua embaixo desta: o casamento e por
/// **modelo**, e uma sondagem obsoleta que descrevesse outro disco cai em
/// [`SemNome::ModeloNaoCasa`], que e recusa e nao palpite.
/// # E `SemOraculo` e a **unica** recusa da sondagem que deixa as imagens falar
///
/// As outras tres sao afirmacoes sobre a maquina de agora, e nao a ausencia de
/// uma. `ModeloAmbiguo` diz *"ha dois discos deste modelo aqui, neste
/// instante"* — resolver isso por um `blkdev.list` de um backup antigo e
/// exatamente o chute que aquela recusa existe para nao dar. Entao ela vence,
/// e o comando para.
pub fn nome_do_disco(modelo_do_windows: &str, listas: &[Lista]) -> Result<NomeDoDisco, SemNome> {
    let da_sondagem = |lista: &&Lista| matches!(lista.fonte, Fonte::Sondagem { .. });

    let sondagens: Vec<&Lista> = listas.iter().filter(da_sondagem).collect();
    let imagens: Vec<&Lista> = listas.iter().filter(|lista| !da_sondagem(lista)).collect();

    let pelas_imagens = procurar(modelo_do_windows, &imagens);

    match procurar(modelo_do_windows, &sondagens) {
        Ok((disco, Fonte::Sondagem { quando })) => {
            // A divergencia so existe quando as imagens **respondem** outra
            // coisa. Elas nao responderem e o caso normal do dispositivo
            // recem-preparado, e nao uma discordancia.
            let divergencia = match &pelas_imagens {
                Ok((outro, Fonte::Imagem(imagem))) if *outro != disco => Some(Divergencia {
                    imagem: imagem.clone(),
                    disco: outro.como_texto().to_string(),
                }),
                _ => None,
            };

            Ok(NomeDoDisco {
                disco,
                origem: Origem::LidoDaSondagem {
                    modelo: modelo_do_windows.to_string(),
                    quando,
                    divergencia,
                },
            })
        }
        // Inalcancavel: so entraram sondagens, e [`procurar`] devolve a fonte
        // que achou. Cai para as imagens em vez de entrar em panico — e o que
        // esta certo se um dia a filtragem acima mudar.
        Ok((_, Fonte::Imagem(_))) => achado(modelo_do_windows, pelas_imagens),

        // A sondagem nao tem o que dizer: e o dispositivo sem sondagem, que e
        // o caminho de antes da E12, e as imagens decidem.
        Err(SemNome::SemOraculo) => achado(modelo_do_windows, pelas_imagens),

        // A sondagem falou, e o que ela disse foi uma recusa.
        Err(recusa) => Err(recusa),
    }
}

/// O par que [`procurar`] devolve, virando o [`NomeDoDisco`] que a tela lê.
fn achado(
    modelo_do_windows: &str,
    procurado: Result<(Disco, Fonte), SemNome>,
) -> Result<NomeDoDisco, SemNome> {
    let (disco, fonte) = procurado?;
    Ok(NomeDoDisco {
        disco,
        origem: match fonte {
            Fonte::Imagem(imagem) => Origem::LidoDaImagem {
                imagem,
                modelo: modelo_do_windows.to_string(),
            },
            Fonte::Sondagem { quando } => Origem::LidoDaSondagem {
                modelo: modelo_do_windows.to_string(),
                quando,
                divergencia: None,
            },
        },
    })
}

/// A procura de sempre, sobre um conjunto de listas.
///
/// Separada de [`nome_do_disco`] na E12, quando passou a haver duas fontes: o
/// que mudou foi **quem se consulta primeiro**, e nao como se procura. Duas
/// copias desta funcao — uma por fonte — divergiriam na primeira mudanca, e
/// uma delas passaria a resolver um nome de disco por outra regra.
///
/// Devolve o disco e a **fonte que o respondeu**; quem monta a [`Origem`] e
/// quem tem as duas fontes em maos, porque so ele sabe se ha divergencia.
fn procurar(modelo_do_windows: &str, listas: &[&Lista]) -> Result<(Disco, Fonte), SemNome> {
    let procurado = normalizar(modelo_do_windows);
    if procurado.is_empty() {
        return Err(SemNome::ModeloNaoCasa {
            modelo: modelo_do_windows.to_string(),
        });
    }

    let mut houve_oraculo = false;
    let mut achados: Vec<(String, Fonte)> = Vec::new();

    for lista in listas {
        let discos = ler(&lista.texto);
        if discos.is_empty() {
            continue;
        }
        houve_oraculo = true;

        for disco in discos {
            if normalizar(&disco.modelo) == procurado
                && !achados.iter().any(|(nome, _)| *nome == disco.nome)
            {
                achados.push((disco.nome, lista.fonte.clone()));
            }
        }
    }

    if !houve_oraculo {
        return Err(SemNome::SemOraculo);
    }

    match achados.len() {
        0 => Err(SemNome::ModeloNaoCasa {
            modelo: modelo_do_windows.to_string(),
        }),
        1 => {
            let (nome, fonte) = achados.remove(0);
            match Disco::novo(&nome) {
                Ok(disco) => Ok((disco, fonte)),
                Err(_) => Err(SemNome::NomeInvalido { tem: nome }),
            }
        }
        // Dois nomes Linux diferentes com o mesmo modelo: dois discos iguais
        // na mesma maquina. Escolher seria nomear um disco no chute.
        quantos => Err(SemNome::ModeloAmbiguo {
            modelo: modelo_do_windows.to_string(),
            quantos,
        }),
    }
}

/// Se dois modelos, escritos por ferramentas diferentes, sao do mesmo disco.
///
/// Publica desde a E9, que precisa comparar o modelo que o Windows da ao disco
/// de **destino** com o que o `sgdisk` de dentro da imagem deu ao de origem
/// (R-2, R-7). E a mesma pergunta que [`nome_do_disco`] ja fazia, feita fora
/// dela: uma segunda normalizacao escrita a mao divergiria da primeira na
/// primeira mudanca.
///
/// Dois modelos vazios **nao** casam. Um `Model:` que o `sgdisk` nao trouxe e
/// um disco sem identidade, e casar tudo com tudo faria a conferencia de R-2
/// aprovar qualquer destino.
pub fn mesmo_modelo(um: &str, outro: &str) -> bool {
    let um = normalizar(um);
    !um.is_empty() && um == normalizar(outro)
}

/// O prefixo que a traducao SCSI do Windows poe em disco NVMe, no lugar do
/// fabricante. Medido em 31/08/2026 num `EG6 KIOXIA 1024GB`.
const PREFIXO_DO_WINDOWS: &str = "NVME";

/// O sufixo que o Windows acrescenta a disco sem driver proprio. Medido em
/// 22/08/2026 num `KGSSE100 256`.
const SUFIXO_DO_WINDOWS: &str = "SCSIDISKDEVICE";

/// Um modelo comparavel entre o WMI e o `lsblk`.
///
/// Maiusculas, so letra e digito, e sem os dois afixos da traducao SCSI do
/// Windows. Medido: `KGSSE100 256 SCSI Disk Device` casa com `KGSSE100256`,
/// `NVMe EG6 KIOXIA 1024GB` casa com `EG6 KIOXIA 1024GB`, e
/// `KINGSTON SNV3S500G` casa consigo mesmo sem precisar de nenhum dos dois.
///
/// O prefixo sai **quantas vezes aparecer**, e nao uma. Nao ha medicao de um
/// disco cujo modelo comece por `NVMe` — o que haveria ali seria
/// `NVMe NVMe ...` de um lado e `NVMe ...` do outro —, e tirar uma vez so
/// deixaria justamente esse par sem casar. Repetir custa a mesma chamada e
/// torna `normalizar` idempotente, que e o que se espera de um normalizador.
/// Sobrar prefixo demais nao nomeia disco errado: dois modelos que colidissem
/// por causa disto viram [`SemNome::ModeloAmbiguo`], que e recusa.
fn normalizar(modelo: &str) -> String {
    let so_alfanumerico: String = modelo
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    so_alfanumerico
        .strip_suffix(SUFIXO_DO_WINDOWS)
        .unwrap_or(&so_alfanumerico)
        .trim_start_matches(PREFIXO_DO_WINDOWS)
        .to_string()
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O `blkdev.list` de `2026-08-21_WindowsCompleto`, copiado do dispositivo
    /// em 22/08/2026. As larguras de coluna sao as do arquivo.
    const DO_DISPOSITIVO: &str = concat!(
        "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
        "loop0     loop0       466.2M loop squashfs /run/live/rootfs/filesystem.squashfs \n",
        "sda       sda         238.5G disk                                               KGSSE100256\n",
        "sda1      |-sda1      236.9G part ntfs     /home/partimag                       \n",
        "sda2      `-sda2        1.6G part vfat                                          \n",
        "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
        "nvme0n1p1 |-nvme0n1p1   300M part vfat                                          \n",
        "nvme0n1p2 |-nvme0n1p2    16M part                                               \n",
        "nvme0n1p3 |-nvme0n1p3 464.5G part ntfs                                          \n",
        "nvme0n1p4 `-nvme0n1p4     1G part ntfs                                          \n",
    );

    fn listas() -> Vec<Lista> {
        vec![da_imagem("2026-08-21_WindowsCompleto", DO_DISPOSITIVO)]
    }

    fn da_imagem(imagem: &str, texto: &str) -> Lista {
        Lista {
            fonte: Fonte::Imagem(imagem.to_string()),
            texto: texto.to_string(),
        }
    }

    fn da_sondagem(texto: &str) -> Lista {
        Lista {
            fonte: Fonte::Sondagem {
                quando: Some(crate::duplos::momento("2026-08-23T21:14:07")),
            },
            texto: texto.to_string(),
        }
    }

    #[test]
    fn le_os_dois_discos_e_ignora_particao_e_loop() {
        let discos = ler(DO_DISPOSITIVO);

        assert_eq!(
            discos,
            vec![
                DiscoDaImagem {
                    nome: "sda".to_string(),
                    modelo: "KGSSE100256".to_string()
                },
                DiscoDaImagem {
                    nome: "nvme0n1".to_string(),
                    modelo: "KINGSTON SNV3S500G".to_string()
                },
            ]
        );
    }

    #[test]
    fn o_modelo_com_espaco_chega_inteiro() {
        // `KINGSTON SNV3S500G` tem espaco dentro. Repartir a linha por espaco
        // daria `KINGSTON` — e ai o modelo nao casaria com nada.
        let discos = ler(DO_DISPOSITIVO);
        assert_eq!(discos[1].modelo, "KINGSTON SNV3S500G");
    }

    #[test]
    fn o_nome_sai_da_coluna_name_e_nao_da_kname() {
        // Achado pela revisao da E6. As duas colunas coincidem para `sda` e
        // `nvme0n1`, o que fazia a versao errada passar em todo teste. Num
        // multipath elas diferem — `NAME=mpatha`, `KNAME=dm-0` — e ler o
        // `KNAME` daria o dispositivo de baixo em vez do disco.
        let multipath = concat!(
            "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
            "dm-0      mpatha      465.8G disk                                               ACME ARRAY\n",
        );

        assert_eq!(
            ler(multipath),
            vec![DiscoDaImagem {
                nome: "mpatha".to_string(),
                modelo: "ACME ARRAY".to_string()
            }]
        );
    }

    #[test]
    fn a_coluna_name_nao_e_confundida_com_kname() {
        // `NAME` aparece dentro de `KNAME`, que e a **primeira** coluna. Casar
        // por substring daria o deslocamento errado para todas as outras.
        let cabecalho = "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT   MODEL";
        assert_eq!(achar_coluna(cabecalho, "KNAME"), Some(0));
        assert_eq!(achar_coluna(cabecalho, "NAME"), Some(10));
        assert!(achar_coluna(cabecalho, "MODEL").unwrap() > 30);
    }

    #[test]
    fn a_descoberta_acha_o_nvme0n1_pelo_modelo_do_windows() {
        // O caminho que importa: o WMI diz o modelo do disco onde o `C:` mora,
        // e o `blkdev.list` diz que nome o Linux lhe da.
        let achado = nome_do_disco("KINGSTON SNV3S500G", &listas()).expect("os dois concordam");

        assert_eq!(achado.disco.como_texto(), "nvme0n1");
        assert_eq!(
            achado.origem,
            Origem::LidoDaImagem {
                imagem: "2026-08-21_WindowsCompleto".to_string(),
                modelo: "KINGSTON SNV3S500G".to_string()
            }
        );
    }

    #[test]
    fn a_saida_diz_de_onde_o_nome_veio() {
        // O padrao que a E3 estabeleceu. Uma receita destrutiva que nomeie um
        // disco sem dizer a origem do nome e pior do que nao imprimir nada.
        let dito = nome_do_disco("KINGSTON SNV3S500G", &listas())
            .unwrap()
            .to_string();

        assert!(dito.contains("nvme0n1"), "{dito}");
        assert!(dito.contains("blkdev.list"), "{dito}");
        assert!(dito.contains("2026-08-21_WindowsCompleto"), "{dito}");
    }

    #[test]
    fn o_sufixo_scsi_disk_device_do_windows_nao_impede_o_casamento() {
        // Medido: o WMI escreve `KGSSE100 256 SCSI Disk Device` e o `lsblk`
        // escreve `KGSSE100256`. E o mesmo disco.
        let achado = nome_do_disco("KGSSE100 256 SCSI Disk Device", &listas()).expect("casa");
        assert_eq!(achado.disco.como_texto(), "sda");
    }

    /// A sondagem da maquina `SCI-3403`, colhida em 30/08/2026 as 16:03, e
    /// copiada de `D:\ARCA-LOGS\sondagem\blkdev.list` sem retoque. As larguras
    /// de coluna sao as do arquivo.
    ///
    /// O que ela tem de diferente do dispositivo desta mesa e o unico ponto:
    /// o disco de origem e um KIOXIA, e o `Win32_DiskDrive.Model` dele vem com
    /// o prefixo `NVMe`.
    const DA_OUTRA_MAQUINA: &str = concat!(
        "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
        "loop0     loop0       466.2M loop squashfs /run/live/rootfs/filesystem.squashfs \n",
        "sda       sda         447.1G disk                                               Maxtor Z1 SSD 480GB\n",
        "sda1      |-sda1      445.6G part ntfs     /home/partimag                       \n",
        "sda2      `-sda2        1.6G part vfat                                          \n",
        "nvme0n1   nvme0n1     953.9G disk                                               EG6 KIOXIA 1024GB\n",
        "nvme0n1p1 |-nvme0n1p1   200M part vfat                                          \n",
        "nvme0n1p2 |-nvme0n1p2    16M part                                               \n",
        "nvme0n1p3 |-nvme0n1p3 952.8G part ntfs                                          \n",
        "nvme0n1p4 `-nvme0n1p4   901M part ntfs                                          \n",
    );

    #[test]
    fn o_prefixo_nvme_do_windows_nao_impede_o_casamento() {
        // Medido em 31/08/2026, na maquina `SCI-3403`: o WMI escreve
        // `NVMe EG6 KIOXIA 1024GB` e o `lsblk` escreve `EG6 KIOXIA 1024GB`. E
        // o mesmo disco, e sem tirar o prefixo o `arca backup` daquela maquina
        // parava em `POR DETERMINAR` com uma sondagem boa no dispositivo.
        let achado = nome_do_disco("NVMe EG6 KIOXIA 1024GB", &[da_sondagem(DA_OUTRA_MAQUINA)])
            .expect("casa");
        assert_eq!(achado.disco.como_texto(), "nvme0n1");
    }

    #[test]
    fn o_prefixo_nvme_nao_faz_o_dispositivo_passar_por_disco_de_origem() {
        // A ponte USB responde `JMicron Generic SCSI Disk Device` ao Windows e
        // o `lsblk` le `Maxtor Z1 SSD 480GB` atras dela (PRD §2084). Tirar o
        // prefixo de um lado nao pode aproximar esses dois: se aproximasse, o
        // `sda` — que e o proprio dispositivo — entraria numa receita como
        // disco de origem.
        assert_eq!(
            nome_do_disco(
                "JMicron Generic SCSI Disk Device",
                &[da_sondagem(DA_OUTRA_MAQUINA)]
            ),
            Err(SemNome::ModeloNaoCasa {
                modelo: "JMicron Generic SCSI Disk Device".to_string()
            })
        );
    }

    // ─────────────────────── as quatro recusas ───────────────────────

    #[test]
    fn sem_imagem_nenhuma_o_nome_fica_por_determinar() {
        // O oraculo so existe depois do primeiro backup. Isto e uma resposta,
        // e nao uma falha a contornar com um palpite.
        assert_eq!(
            nome_do_disco("KINGSTON SNV3S500G", &[]),
            Err(SemNome::SemOraculo)
        );
    }

    #[test]
    fn blkdev_list_ilegivel_conta_como_nao_haver_oraculo() {
        let lixo = vec![da_imagem("X", "alguma coisa\nque nao e um lsblk\n")];
        assert_eq!(
            nome_do_disco("KINGSTON SNV3S500G", &lixo),
            Err(SemNome::SemOraculo)
        );
    }

    #[test]
    fn modelo_que_nao_aparece_e_recusa_e_nao_o_primeiro_disco_da_lista() {
        // O modo de falha que este teste guarda: devolver `sda` porque era o
        // primeiro faria a receita nomear o **proprio dispositivo de backup**
        // como origem.
        match nome_do_disco("SAMSUNG 990 PRO", &listas()).unwrap_err() {
            SemNome::ModeloNaoCasa { modelo } => assert_eq!(modelo, "SAMSUNG 990 PRO"),
            outro => panic!("esperava a recusa por modelo, veio {outro}"),
        }
    }

    #[test]
    fn dois_discos_do_mesmo_modelo_sao_ambiguidade_e_nao_escolha() {
        let dois = concat!(
            "KNAME  NAME    SIZE TYPE FSTYPE MOUNTPOINT MODEL\n",
            "nvme0n1 nvme0n1 465G disk                  IGUAL\n",
            "nvme1n1 nvme1n1 465G disk                  IGUAL\n",
        );

        match nome_do_disco("IGUAL", &[da_imagem("X", dois)]).unwrap_err() {
            SemNome::ModeloAmbiguo { quantos, .. } => assert_eq!(quantos, 2),
            outro => panic!("esperava a ambiguidade, veio {outro}"),
        }
    }

    #[test]
    fn o_mesmo_disco_em_duas_imagens_nao_vira_ambiguidade() {
        // As duas imagens deste dispositivo tem o mesmo `blkdev.list`. Contar
        // duas vezes o mesmo `nvme0n1` faria a descoberta recusar justamente o
        // caso normal.
        let duas = vec![
            da_imagem("2026-08-21_WindowsCompleto", DO_DISPOSITIVO),
            da_imagem("ARCA-TESTE-03", DO_DISPOSITIVO),
        ];

        assert_eq!(
            nome_do_disco("KINGSTON SNV3S500G", &duas)
                .expect("uma so resposta")
                .disco
                .como_texto(),
            "nvme0n1"
        );
    }

    #[test]
    fn um_nome_que_nao_passa_pelo_validador_de_disco_e_recusado() {
        // O nome vai para dentro de uma receita destrutiva. Um `blkdev.list`
        // adulterado nao pode contrabandear texto para la.
        // Construido **sobre o arquivo de verdade**, trocando so o nome por
        // outro do mesmo tamanho. Duas versoes anteriores deste teste montaram
        // uma tabela a mao e erraram o alinhamento das colunas: a linha era
        // descartada antes de chegar ao validador, e o teste "passava" pelo
        // motivo errado — ele nao provava a recusa, provava que a linha nao
        // era lida. E a licao da revisao da E4 outra vez.
        let torto = DO_DISPOSITIVO.replace("nvme0n1   nvme0n1", "NVME0N1   NVME0N1");

        // A linha continua sendo lida: o disco esta la, com o modelo certo.
        assert!(
            ler(&torto)
                .iter()
                .any(|disco| disco.modelo == "KINGSTON SNV3S500G"),
            "a linha foi descartada antes do validador, e o teste provaria nada"
        );

        assert!(matches!(
            nome_do_disco("KINGSTON SNV3S500G", &[da_imagem("X", &torto)]),
            Err(SemNome::NomeInvalido { .. })
        ));
    }

    #[test]
    fn cada_recusa_tem_mensagem_propria() {
        let todas = [
            SemNome::SemOraculo,
            SemNome::ModeloNaoCasa {
                modelo: "X".to_string(),
            },
            SemNome::ModeloAmbiguo {
                modelo: "X".to_string(),
                quantos: 2,
            },
            SemNome::NomeInvalido {
                tem: "X".to_string(),
            },
        ];

        for recusa in todas {
            assert!(
                recusa.to_string().chars().count() > 30,
                "{recusa:?} sem mensagem propria"
            );
        }
    }

    #[test]
    fn cada_recusa_manda_sondar_so_quando_sondar_resolve() {
        // **Conselho que sai sempre vira ruido**, e a E10 pagou por essa licao
        // no `arca resultado`. Cada recusa diz a saida **dela**, e a sondagem
        // nao e saida para todas:
        //
        // | recusa | `arca sondar` resolve? |
        // |---|---|
        // | `SemOraculo` — nao ha `blkdev.list` nenhum | **sim**, e e para isso que ele existe |
        // | `ModeloNaoCasa` — o que ha descreve outra maquina | **sim**: a sondagem descreve esta |
        // | `NomeInvalido` — o arquivo traz nome torto | **sim**: ele grava um novo |
        // | `ModeloAmbiguo` — dois discos do mesmo modelo | **nao**: sondar de novo veria os dois outra vez |
        //
        // O aviso do pre-voo **nao** repete isto, e a razao apareceu rodando o
        // comando de verdade: com a linha fixa la e a mensagem aqui, a tela
        // dizia `arca sondar` duas vezes em quatro linhas.
        for resolve in [
            SemNome::SemOraculo,
            SemNome::ModeloNaoCasa {
                modelo: "KINGSTON SNV3S500G".to_string(),
            },
            SemNome::NomeInvalido {
                tem: "NVME0N1".to_string(),
            },
        ] {
            assert!(
                resolve.to_string().contains("arca sondar"),
                "{resolve:?} nao diz a saida"
            );
        }

        let ambiguo = SemNome::ModeloAmbiguo {
            modelo: "KINGSTON SNV3S500G".to_string(),
            quantos: 2,
        };
        assert!(
            !ambiguo.to_string().contains("arca sondar"),
            "sondar de novo nao desfaz uma ambiguidade: {ambiguo}"
        );
    }

    // ──────────────────────── a normalizacao ────────────────────────

    #[test]
    fn a_normalizacao_junta_o_que_e_o_mesmo_disco() {
        assert_eq!(normalizar("KGSSE100 256 SCSI Disk Device"), "KGSSE100256");
        assert_eq!(normalizar("KGSSE100256"), "KGSSE100256");
        assert_eq!(normalizar("KINGSTON SNV3S500G"), "KINGSTONSNV3S500G");
        assert_eq!(normalizar("NVMe EG6 KIOXIA 1024GB"), "EG6KIOXIA1024GB");
        assert_eq!(normalizar("EG6 KIOXIA 1024GB"), "EG6KIOXIA1024GB");
    }

    #[test]
    fn a_normalizacao_e_idempotente() {
        // O que ela devolve ja esta normalizado. Sem isto, um modelo que
        // comecasse por `NVMe` sairia diferente de cada lado da comparacao:
        // `NVMe NVMe X` do Windows viraria `NVMEX`, e `NVMe X` do `lsblk`
        // viraria `X`, e o mesmo disco nao casaria consigo.
        for modelo in [
            "KGSSE100 256 SCSI Disk Device",
            "NVMe EG6 KIOXIA 1024GB",
            "NVMe NVMe X",
            "KINGSTON SNV3S500G",
        ] {
            let uma_vez = normalizar(modelo);
            assert_eq!(normalizar(&uma_vez), uma_vez, "modelo `{modelo}`");
        }
    }

    #[test]
    fn a_normalizacao_nao_junta_discos_diferentes() {
        // O outro lado, e o que importa: normalizar demais faria dois discos
        // distintos casarem, e ai a receita nomearia o errado.
        assert_ne!(normalizar("SAMSUNG 990 PRO"), normalizar("SAMSUNG 980 PRO"));
        assert_ne!(normalizar("WDC WD10"), normalizar("WDC WD100"));

        // O prefixo sai do comeco, e nao de dentro nem do fim: `NVME` no meio
        // de um modelo e parte do modelo.
        assert_ne!(normalizar("ACME NVME 500"), normalizar("ACME 500"));
        assert_ne!(normalizar("KIOXIA NVME"), normalizar("KIOXIA"));
    }

    // ─────────── a segunda fonte, e a precedencia (E12, SD-5) ───────────

    /// O que a sondagem veria nesta maquina hoje: o mesmo par nome-modelo do
    /// `blkdev.list` das imagens.
    const DA_SONDAGEM: &str = concat!(
        "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
        "sda       sda         238.5G disk                                               KGSSE100256\n",
        "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
    );

    #[test]
    fn a_sondagem_sozinha_ja_responde_o_nome_do_disco() {
        // **O buraco que a E12 fecha, em um teste.** Ate ela, um dispositivo
        // sem imagem nenhuma nao tinha oraculo, e os tres comandos que armam
        // recusavam. Agora a sondagem responde sozinha.
        let achado = nome_do_disco("KINGSTON SNV3S500G", &[da_sondagem(DA_SONDAGEM)])
            .expect("a sondagem responde sem imagem nenhuma");

        assert_eq!(achado.disco.como_texto(), "nvme0n1");
        assert!(
            matches!(achado.origem, Origem::LidoDaSondagem { .. }),
            "{:?}",
            achado.origem
        );
    }

    #[test]
    fn a_saida_diz_que_o_nome_veio_da_sondagem_e_quando() {
        // Uma sondagem que se apresentasse como imagem seria a mesma falha que
        // o `arca prepare` pagou na E10: uma tela afirmando o que nao
        // aconteceu — nao ha imagem nenhuma no dispositivo em que a sondagem
        // mais importa.
        //
        // E a **hora** vai junto porque uma sondagem de um mês atras pode estar
        // descrevendo um disco que nao esta mais na maquina.
        let dito = nome_do_disco("KINGSTON SNV3S500G", &[da_sondagem(DA_SONDAGEM)])
            .unwrap()
            .to_string();

        assert!(dito.contains("nvme0n1"), "{dito}");
        assert!(dito.contains("sondagem"), "{dito}");
        assert!(dito.contains("23/08 21:14"), "{dito}");
        assert!(
            !dito.contains("imagem"),
            "a sondagem se apresentou como imagem: {dito}"
        );
    }

    #[test]
    fn com_as_duas_fontes_concordando_a_sondagem_e_a_que_responde() {
        // A sondagem descreve a maquina de **agora**; a imagem descreve a de
        // quando o backup foi feito. Concordando, o nome e o mesmo — e quem o
        // respondeu importa para a tela, que diz de onde ele veio.
        let listas = vec![
            da_sondagem(DA_SONDAGEM),
            da_imagem("2026-08-21_WindowsCompleto", DO_DISPOSITIVO),
        ];

        let achado = nome_do_disco("KINGSTON SNV3S500G", &listas).unwrap();

        assert_eq!(achado.disco.como_texto(), "nvme0n1");
        assert_eq!(
            achado.origem,
            Origem::LidoDaSondagem {
                modelo: "KINGSTON SNV3S500G".to_string(),
                quando: Some(crate::duplos::momento("2026-08-23T21:14:07")),
                divergencia: None,
            },
            "concordando, nao ha divergencia a dizer"
        );
    }

    #[test]
    fn discordando_a_sondagem_ganha_e_a_divergencia_e_dita() {
        // **SD-5.** O disco foi trocado depois do backup: a imagem guarda o
        // nome que o Linux dava ao disco de então, e a sondagem sabe o de
        // agora. A sondagem ganha — e a divergencia sai na tela, nunca
        // resolvida em silencio.
        let na_imagem = DO_DISPOSITIVO.replace("nvme0n1   nvme0n1", "nvme1n1   nvme1n1");
        let listas = vec![
            da_sondagem(DA_SONDAGEM),
            da_imagem("2026-08-21_WindowsCompleto", &na_imagem),
        ];

        let achado = nome_do_disco("KINGSTON SNV3S500G", &listas).unwrap();

        assert_eq!(achado.disco.como_texto(), "nvme0n1", "a sondagem ganhou");
        assert_eq!(
            achado.origem,
            Origem::LidoDaSondagem {
                modelo: "KINGSTON SNV3S500G".to_string(),
                quando: Some(crate::duplos::momento("2026-08-23T21:14:07")),
                divergencia: Some(Divergencia {
                    imagem: "2026-08-21_WindowsCompleto".to_string(),
                    disco: "nvme1n1".to_string(),
                }),
            }
        );

        let dito = achado.to_string();
        assert!(dito.contains("DIVERGE"), "{dito}");
        assert!(dito.contains("nvme1n1"), "{dito}");
    }

    #[test]
    fn sem_sondagem_as_imagens_respondem_como_antes_da_e12() {
        // A garantia de que a E12 nao mexeu no caminho que ja existia: um
        // dispositivo com imagens e sem sondagem responde exatamente o que
        // respondia — inclusive na `Origem`, que e o que a tela imprime.
        let achado = nome_do_disco("KINGSTON SNV3S500G", &listas()).unwrap();

        assert_eq!(
            achado.origem,
            Origem::LidoDaImagem {
                imagem: "2026-08-21_WindowsCompleto".to_string(),
                modelo: "KINGSTON SNV3S500G".to_string(),
            }
        );
    }

    #[test]
    fn uma_sondagem_ilegivel_deixa_as_imagens_responderem() {
        // O `lsblk` falhou e o `2>&1` deixou a mensagem de erro dentro do
        // arquivo: o cabecalho nao bate, `ler` devolve lista vazia, e a
        // sondagem simplesmente **nao participa**. Ela nao pode engolir o
        // oraculo que as imagens ja davam.
        let listas = vec![
            da_sondagem("lsblk: unknown column: FLAGQUENAOEXISTE\n"),
            da_imagem("2026-08-21_WindowsCompleto", DO_DISPOSITIVO),
        ];

        let achado = nome_do_disco("KINGSTON SNV3S500G", &listas).unwrap();

        assert_eq!(achado.disco.como_texto(), "nvme0n1");
        assert!(
            matches!(achado.origem, Origem::LidoDaImagem { .. }),
            "a sondagem quebrada respondeu no lugar das imagens: {:?}",
            achado.origem
        );
    }

    #[test]
    fn uma_ambiguidade_vista_pela_sondagem_nao_e_resolvida_pela_imagem() {
        // **A unica recusa da sondagem que NAO deixa as imagens falar é
        // `SemOraculo`; as outras vencem.** `ModeloAmbiguo` diz *"ha dois
        // discos deste modelo aqui, neste instante"* — e resolver isso por um
        // `blkdev.list` de um backup antigo e exatamente o chute que aquela
        // recusa existe para nao dar.
        //
        // O caso concreto: alguem conectou um segundo disco igual ao de
        // origem. A imagem, feita antes, so conhece um.
        let com_gemeo = DA_SONDAGEM.replace(
            "sda       sda         238.5G disk                                               KGSSE100256",
            "sdb       sdb         465.8G disk                                               KINGSTON SNV3S500G",
        );
        let listas = vec![
            da_sondagem(&com_gemeo),
            da_imagem("2026-08-21_WindowsCompleto", DO_DISPOSITIVO),
        ];

        match nome_do_disco("KINGSTON SNV3S500G", &listas).unwrap_err() {
            SemNome::ModeloAmbiguo { quantos, .. } => assert_eq!(quantos, 2),
            outro => panic!("a imagem antiga desfez a ambiguidade de agora: {outro}"),
        }
    }

    #[test]
    fn a_precedencia_e_por_fonte_e_nunca_pela_data() {
        // **S-6 no lugar onde ele quase deixou de valer.** A E12 pôs um
        // `DateTime` dentro de [`Origem`], e este modulo e quem decide **qual
        // disco entra numa receita destrutiva**. A regra que se escreve sem
        // pensar seria *"a fonte mais recente ganha"* — e ela e errada por
        // dois motivos, nesta ordem:
        //
        // 1. **A sondagem ganha por ser sondagem**, e nao por ser nova: ela
        //    descreve a maquina de agora **por construcao**, e uma imagem
        //    gravada cinco minutos atras continua descrevendo a maquina de
        //    quando aquele backup rodou.
        // 2. As duas datas viriam do `mtime` — e o `mtime` de uma imagem foi
        //    escrito pelo Clonezilla, que roda 3 h adiantado (P-7). Compara-las
        //    e literalmente o que S-6 proibe.
        //
        // O teste e de comportamento, e nao uma varredura de texto: uma
        // sondagem **mais antiga** que a imagem continua ganhando.
        let antiga = Lista {
            fonte: Fonte::Sondagem {
                quando: Some(crate::duplos::momento("2020-01-01T00:00:00")),
            },
            texto: DA_SONDAGEM.to_string(),
        };
        let recente = DO_DISPOSITIVO.replace("nvme0n1   nvme0n1", "nvme1n1   nvme1n1");

        let achado = nome_do_disco(
            "KINGSTON SNV3S500G",
            &[antiga, da_imagem("de-hoje", &recente)],
        )
        .unwrap();

        assert_eq!(
            achado.disco.como_texto(),
            "nvme0n1",
            "a fonte mais recente ganhou: a precedencia virou temporal"
        );
    }

    #[test]
    fn sem_fonte_nenhuma_a_recusa_manda_sondar() {
        // A recusa passou a ter saida na E12, e ela e um comando — nao mais um
        // backup pelo menu do Clonezilla, que era a resposta anterior e que
        // este app existe para nao precisar.
        let recusa = nome_do_disco("KINGSTON SNV3S500G", &[]).unwrap_err();

        assert_eq!(recusa, SemNome::SemOraculo);
        assert!(recusa.to_string().contains("arca sondar"), "{recusa}");
    }
}
