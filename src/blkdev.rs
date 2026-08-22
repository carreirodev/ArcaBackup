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
//! # A comparacao de modelo, e o unico ajuste que ela precisa
//!
//! O modelo e comparado sem caixa e sem os caracteres que nao sao letra nem
//! digito, porque o WMI escreve `KGSSE100 256` e o `lsblk` escreve
//! `KGSSE100256` — o mesmo texto com um espaco a mais.
//!
//! Ha um sufixo a tirar, e ele e um artefato conhecido do Windows: um disco
//! sem driver proprio aparece como `<modelo> SCSI Disk Device`. Medido nesta
//! maquina — `KGSSE100 256 SCSI Disk Device` no WMI, `KGSSE100256` no `lsblk`.
//! Com o sufixo fora, os dois casam.
//!
//! **Nao casar e recusa, e nunca um palpite.** Um nome de disco errado numa
//! receita destrutiva e o pior desfecho possivel deste modulo.

use crate::receita::Disco;
use std::fmt;

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

        let ate_o_fim = |de: usize| -> String {
            caracteres[de.min(caracteres.len())..].iter().collect()
        };

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
        let Some(nome) = ate_o_fim(nome_em).split_whitespace().next().map(str::to_string) else {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origem {
    /// Lido do `blkdev.list` de uma imagem, casando o modelo com o que o WMI
    /// diz do disco onde o `C:` mora.
    LidoDaImagem { imagem: String, modelo: String },
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
    /// Nenhuma imagem no dispositivo traz um `blkdev.list` legivel.
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
                "nenhuma imagem do dispositivo traz um `blkdev.list` legivel, e e dele que sai o nome que o Linux da ao disco. O Windows nao conhece esse nome, e o ARCA nao o inventa"
            ),
            SemNome::ModeloNaoCasa { modelo } => write!(
                f,
                "nenhum disco dos `blkdev.list` das imagens tem o modelo `{modelo}`, que e o do disco onde o Windows esta. As imagens deste dispositivo vieram de outra maquina"
            ),
            SemNome::ModeloAmbiguo { modelo, quantos } => write!(
                f,
                "{quantos} discos com o modelo `{modelo}` aparecem no `blkdev.list`, e nao ha como saber qual e o de origem. O ARCA nao escolhe um disco no chute"
            ),
            SemNome::NomeInvalido { tem } => write!(
                f,
                "o `blkdev.list` traz `{tem}` como nome de disco, e ele nao tem a forma de um nome do Linux"
            ),
        }
    }
}

/// O nome Linux do disco de `modelo`, procurado nos `blkdev.list` dados.
///
/// Recebe os arquivos ja lidos, com o nome da imagem de onde cada um veio, e
/// nao os lê: manter isto puro e o que permite testar as quatro recusas sem
/// dispositivo conectado.
pub fn nome_do_disco(
    modelo_do_windows: &str,
    listas: &[(String, String)],
) -> Result<NomeDoDisco, SemNome> {
    let procurado = normalizar(modelo_do_windows);
    if procurado.is_empty() {
        return Err(SemNome::ModeloNaoCasa {
            modelo: modelo_do_windows.to_string(),
        });
    }

    let mut houve_oraculo = false;
    let mut achados: Vec<(String, String)> = Vec::new();

    for (imagem, texto) in listas {
        let discos = ler(texto);
        if discos.is_empty() {
            continue;
        }
        houve_oraculo = true;

        for disco in discos {
            if normalizar(&disco.modelo) == procurado
                && !achados.iter().any(|(nome, _)| *nome == disco.nome)
            {
                achados.push((disco.nome, imagem.clone()));
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
            let (nome, imagem) = achados.remove(0);
            match Disco::novo(&nome) {
                Ok(disco) => Ok(NomeDoDisco {
                    disco,
                    origem: Origem::LidoDaImagem {
                        imagem,
                        modelo: modelo_do_windows.to_string(),
                    },
                }),
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

/// Um modelo comparavel entre o WMI e o `lsblk`.
///
/// Maiusculas, so letra e digito, e sem o `SCSI Disk Device` que o Windows
/// acrescenta a disco sem driver proprio. Medido nesta maquina:
/// `KGSSE100 256 SCSI Disk Device` e `KGSSE100256` casam assim, e
/// `KINGSTON SNV3S500G` casa consigo mesmo sem precisar de nada.
fn normalizar(modelo: &str) -> String {
    let so_alfanumerico: String = modelo
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    so_alfanumerico
        .strip_suffix("SCSIDISKDEVICE")
        .unwrap_or(&so_alfanumerico)
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

    fn listas() -> Vec<(String, String)> {
        vec![(
            "2026-08-21_WindowsCompleto".to_string(),
            DO_DISPOSITIVO.to_string(),
        )]
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

    // ─────────────────────── as quatro recusas ───────────────────────

    #[test]
    fn sem_imagem_nenhuma_o_nome_fica_por_determinar() {
        // O oraculo so existe depois do primeiro backup. Isto e uma resposta,
        // e nao uma falha a contornar com um palpite.
        assert_eq!(nome_do_disco("KINGSTON SNV3S500G", &[]), Err(SemNome::SemOraculo));
    }

    #[test]
    fn blkdev_list_ilegivel_conta_como_nao_haver_oraculo() {
        let lixo = vec![("X".to_string(), "alguma coisa\nque nao e um lsblk\n".to_string())];
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

        match nome_do_disco("IGUAL", &[("X".to_string(), dois.to_string())]).unwrap_err() {
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
            ("2026-08-21_WindowsCompleto".to_string(), DO_DISPOSITIVO.to_string()),
            ("ARCA-TESTE-03".to_string(), DO_DISPOSITIVO.to_string()),
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
            nome_do_disco("KINGSTON SNV3S500G", &[("X".to_string(), torto)]),
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

    // ──────────────────────── a normalizacao ────────────────────────

    #[test]
    fn a_normalizacao_junta_o_que_e_o_mesmo_disco() {
        assert_eq!(normalizar("KGSSE100 256 SCSI Disk Device"), "KGSSE100256");
        assert_eq!(normalizar("KGSSE100256"), "KGSSE100256");
        assert_eq!(normalizar("KINGSTON SNV3S500G"), "KINGSTONSNV3S500G");
    }

    #[test]
    fn a_normalizacao_nao_junta_discos_diferentes() {
        // O outro lado, e o que importa: normalizar demais faria dois discos
        // distintos casarem, e ai a receita nomearia o errado.
        assert_ne!(normalizar("SAMSUNG 990 PRO"), normalizar("SAMSUNG 980 PRO"));
        assert_ne!(normalizar("WDC WD10"), normalizar("WDC WD100"));
    }
}
