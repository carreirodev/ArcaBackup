//! O tamanho do disco de origem, lido de dentro da imagem (R-7, R-2).
//!
//! [`crate::blkdev`] descobre **que nome** o Linux da ao disco; este modulo
//! descobre **de que tamanho** ele era. Sao duas perguntas, dois arquivos, e a
//! E6 so precisava da primeira. R-7 precisa da segunda: "recusar sempre que o
//! destino for menor que a origem" exige uma medida da origem, e a unica que
//! existe do lado Windows esta dentro da imagem.
//!
//! # A fonte e o `<disco>-gpt.sgdisk`, e ela e uma so
//!
//! Toda imagem do Clonezilla carrega a saida do `sgdisk -p` do disco de
//! origem. Da imagem `2026-08-22_Apps`, byte a byte:
//!
//! ```text
//! Disk /dev/nvme0n1: 976773168 sectors, 465.8 GiB
//! Model: KINGSTON SNV3S500G
//! Sector size (logical/physical): 512/512 bytes
//! ```
//!
//! Tres coisas no mesmo arquivo, escritas pela mesma ferramenta no mesmo
//! instante: o total de setores, o tamanho do setor e o modelo. **E por isso
//! que a fonte e uma so.** O `<disco>-pt.sf` traz `last-lba: 976773134` e
//! `sector-size: 512`, e daria para derivar o mesmo numero por outro caminho —
//! mas `last-lba` e o ultimo setor **utilizavel**, e nao o tamanho do disco:
//! os 34 setores da GPT secundaria ficam de fora. Somar 34 seria conhecimento
//! de formato disfarcado de medicao, e a diferenca entre as duas leituras e
//! exatamente do tamanho do erro que R-7 existe para pegar.
//!
//! # A regua, e o que ela custou descobrir
//!
//! Medido nesta maquina em 23/08/2026, para o **mesmo** disco:
//!
//! | Fonte | Bytes | Setores |
//! |---|---|---|
//! | `MSFT_Disk` (`Get-Disk`) | 500.107.862.016 | 976.773.168 |
//! | `Win32_DiskDrive.Size` | 500.105.249.280 | 976.768.065 |
//! | `nvme0n1-gpt.sgdisk` desta imagem | — | **976.773.168** |
//!
//! O `Win32_DiskDrive` fica 2.612.736 bytes atras, e o numero dele nao e
//! arredondamento qualquer: `60801 x 255 x 63 x 512` da exatamente
//! `500.105.249.280` — e o produto da geometria CHS legada, **truncado no
//! ultimo cilindro inteiro**. A diferenca de 5.103 setores e menor que um
//! cilindro (16.065), que e a assinatura desse truncamento.
//!
//! **O `MSFT_Disk` bate byte a byte com o que a imagem registra**, e por isso
//! e ele que R-7 usa. Medir a origem pela GPT de dentro da imagem e o destino
//! pelo `Win32_DiskDrive` faria a comparacao sair de **duas reguas**, e o
//! destino apareceria 2,6 MB menor mesmo quando origem e destino sao
//! fisicamente o mesmo disco — um disco que nao cabe em si mesmo. Para B-4 a
//! fonte antiga continua servindo, porque la ela superestima o em uso, que e o
//! lado seguro de "cabe uma imagem?". Para R-7 ela para de servir.
//!
//! Ver `docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md`.

use std::fmt;

/// O sufixo do arquivo que o Clonezilla escreve por disco.
///
/// Publico porque quem monta o caminho e [`crate::comandos::restore`], do lado
/// Windows, e o nome tem de sair daqui — nao de um literal la.
pub const SUFIXO_DO_SGDISK: &str = "-gpt.sgdisk";

/// O disco de origem, como a imagem o registra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrigemDaImagem {
    /// O nome que o Linux dava ao disco: `nvme0n1`. Sai do `/dev/...` da
    /// primeira linha, e serve para conferir contra o arquivo `disk` (R-2).
    pub disco: String,

    /// Quantos setores o disco inteiro tinha.
    pub setores: u64,

    /// O tamanho do setor **logico**, que e o que o `MSFT_Disk` responde por
    /// `LogicalSectorSize`. O `sgdisk` imprime os dois — logico e fisico — e
    /// so o logico e comparavel: e a unidade em que a tabela de particao e
    /// escrita.
    pub bytes_por_setor: u64,

    /// O modelo, como o `sgdisk` o escreveu. Confere com o `blkdev.list` e com
    /// o que o Windows diz do destino (R-2).
    pub modelo: String,
}

impl OrigemDaImagem {
    pub fn bytes(&self) -> u64 {
        self.setores.saturating_mul(self.bytes_por_setor)
    }
}

/// Por que nao deu para medir a origem.
///
/// Uma variante por motivo, como toda recusa deste projeto — e nenhuma delas
/// vira "zero setores". Um zero aqui faria R-7 aprovar qualquer destino, que e
/// o inverso exato do que o requisito quer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemMedida {
    /// A imagem nao tem o `<disco>-gpt.sgdisk`, ou ele nao se deixou lê.
    SemArquivo { arquivo: String, motivo: String },

    /// O arquivo esta la e nao traz a linha `Disk /dev/...: N sectors`.
    SemLinhaDoDisco { arquivo: String },

    /// O arquivo nao traz o `Sector size (logical/physical)`.
    SemTamanhoDeSetor { arquivo: String },

    /// Traz as linhas e um dos numeros e zero. Um disco de zero setor nao
    /// existe, e tratar isso como medida seria pior do que nao ter nenhuma.
    MedidaZerada { arquivo: String },

    /// O arquivo nao traz a linha `Model:`, ou ela esta vazia.
    ///
    /// **Exigido, e nao opcional.** A primeira versao deste modulo deixava o
    /// modelo cair para cadeia vazia, e o vazio viajava: a conferencia de R-2
    /// comparava `blkdev.list` contra `""` e recusava uma imagem coerente por
    /// "as fontes discordam"; a busca do destino dizia "nenhum disco desta
    /// maquina tem o modelo ``"; e a tela do §6.1 imprimia
    /// `Origem da imagem:  · nvme0n1`. E o mesmo raciocinio do leitor do WMI,
    /// que exige o `Model` em vez de supor: **o modelo e a identidade do
    /// disco**, e uma identidade vazia casa com tudo ou com nada, conforme o
    /// lado de que se olha. Achado pela revisao de codigo da E9.
    SemModelo { arquivo: String },
}

impl fmt::Display for SemMedida {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemMedida::SemArquivo { arquivo, motivo } => write!(
                f,
                "a imagem nao tem o `{arquivo}` legivel ({motivo}), e e dele que sai o tamanho do disco de origem. Sem ele nao da para responder se o destino cabe (R-7)"
            ),
            SemMedida::SemLinhaDoDisco { arquivo } => write!(
                f,
                "o `{arquivo}` nao traz a linha `Disk /dev/...: N sectors`, que e onde o sgdisk escreve o tamanho do disco"
            ),
            SemMedida::SemTamanhoDeSetor { arquivo } => write!(
                f,
                "o `{arquivo}` nao traz a linha `Sector size (logical/physical)`, e sem o tamanho do setor os setores nao viram bytes"
            ),
            SemMedida::MedidaZerada { arquivo } => write!(
                f,
                "o `{arquivo}` traz zero setores ou zero byte por setor: isso nao e um disco, e o ARCA nao trata como medida"
            ),
            SemMedida::SemModelo { arquivo } => write!(
                f,
                "o `{arquivo}` nao traz a linha `Model:`, e e ela que diz de que disco a imagem veio. O modelo e a identidade do disco, e o ARCA nao confere um destino contra uma identidade vazia (R-2)"
            ),
        }
    }
}

/// O nome do arquivo que este modulo lê, para um disco.
pub fn arquivo_do_disco(disco: &str) -> String {
    format!("{disco}{SUFIXO_DO_SGDISK}")
}

/// A origem, a partir do texto do `<disco>-gpt.sgdisk`.
///
/// Puro de proposito: quem lê o arquivo e o comando, e as quatro recusas se
/// testam sem dispositivo conectado.
pub fn ler(arquivo: &str, texto: &str) -> Result<OrigemDaImagem, SemMedida> {
    let mut disco = None;
    let mut setores = None;
    let mut bytes_por_setor = None;
    let mut modelo = String::new();

    for linha in texto.lines().map(str::trim) {
        if let Some(resto) = linha.strip_prefix("Disk ") {
            // `Disk /dev/nvme0n1: 976773168 sectors, 465.8 GiB`
            //
            // O `465.8 GiB` do fim e do proprio sgdisk e **nao** e usado: e
            // texto arredondado para leitura humana, e comparar tamanho por
            // ele seria a mesma familia de erro que o `498,7 GB` do §5.2.
            let Some((caminho, depois)) = resto.split_once(':') else {
                continue;
            };
            let Some(quantos) = depois.split_whitespace().next() else {
                continue;
            };
            let Ok(quantos) = quantos.parse::<u64>() else {
                continue;
            };
            disco = Some(
                caminho
                    .trim()
                    .rsplit('/')
                    .next()
                    .unwrap_or(caminho)
                    .to_string(),
            );
            setores = Some(quantos);
        } else if let Some(resto) = linha.strip_prefix("Model:") {
            modelo = resto.trim().to_string();
        } else if let Some(resto) = linha.strip_prefix("Sector size (logical/physical):") {
            // `512/512 bytes` — o logico e o primeiro, e e o unico que
            // interessa: e a unidade da tabela de particao, e e o que o
            // `MSFT_Disk` responde por `LogicalSectorSize`.
            let logico = resto.trim().split('/').next().unwrap_or_default().trim();
            if let Ok(quantos) = logico.parse::<u64>() {
                bytes_por_setor = Some(quantos);
            }
        }
    }

    let arquivo = arquivo.to_string();
    let (Some(disco), Some(setores)) = (disco, setores) else {
        return Err(SemMedida::SemLinhaDoDisco { arquivo });
    };
    let Some(bytes_por_setor) = bytes_por_setor else {
        return Err(SemMedida::SemTamanhoDeSetor { arquivo });
    };
    if setores == 0 || bytes_por_setor == 0 {
        return Err(SemMedida::MedidaZerada { arquivo });
    }
    if modelo.is_empty() {
        return Err(SemMedida::SemModelo { arquivo });
    }

    Ok(OrigemDaImagem {
        disco,
        setores,
        bytes_por_setor,
        modelo,
    })
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O `nvme0n1-gpt.sgdisk` de `E:\2026-08-22_Apps`, copiado byte a byte do
    /// dispositivo em 23/08/2026.
    ///
    /// **E o oraculo deste modulo, e por isso ele e o arquivo e nao uma cadeia
    /// escrita aqui**: um teste contra texto inventado provaria que eu sei
    /// imaginar o formato do `sgdisk`. Nenhuma asserção abaixo pode ser
    /// ajustada para passar.
    const DESTA_IMAGEM: &str =
        include_str!("../recursos/capturas/nvme0n1-gpt-2026-08-22_Apps.sgdisk");

    fn desta_imagem() -> OrigemDaImagem {
        ler("nvme0n1-gpt.sgdisk", DESTA_IMAGEM).expect("o sgdisk desta imagem tem de ser legivel")
    }

    #[test]
    fn le_o_sgdisk_desta_imagem() {
        assert_eq!(
            desta_imagem(),
            OrigemDaImagem {
                disco: "nvme0n1".to_string(),
                setores: 976_773_168,
                bytes_por_setor: 512,
                modelo: "KINGSTON SNV3S500G".to_string(),
            }
        );
    }

    #[test]
    fn a_medida_da_imagem_bate_com_o_msft_disk_desta_maquina() {
        // O numero do `Get-Disk` desta maquina, medido em 23/08/2026. E o
        // ponto inteiro do ADR-0010: as duas pontas da comparacao de R-7 tem
        // de sair da mesma regua, e saem.
        assert_eq!(desta_imagem().bytes(), 500_107_862_016);
    }

    #[test]
    fn a_medida_da_imagem_nao_bate_com_o_win32_diskdrive() {
        // O outro lado da mesma medicao, e ele vale teste: o dia em que
        // alguem trocar a fonte do destino de volta para o `Win32_DiskDrive`,
        // este teste continua verde e o de cima tambem — mas a comparacao de
        // R-7 passa a dizer que o disco nao cabe em si mesmo. Deixar o numero
        // errado escrito, e nomeado, e o que impede a troca silenciosa.
        const WIN32_DISKDRIVE: u64 = 500_105_249_280;
        assert_ne!(desta_imagem().bytes(), WIN32_DISKDRIVE);
        assert_eq!(desta_imagem().bytes() - WIN32_DISKDRIVE, 2_612_736);

        // E a diferenca e o truncamento CHS, e nao ruido: o produto da
        // geometria legada da exatamente o numero do WMI.
        assert_eq!(60_801u64 * 255 * 63 * 512, WIN32_DISKDRIVE);
    }

    #[test]
    fn o_gib_arredondado_do_sgdisk_nao_e_usado() {
        // `465.8 GiB` esta na mesma linha e nao entra em lugar nenhum. Se
        // entrasse, R-7 compararia texto arredondado — o mesmo erro que o
        // `498,7 GB` do §5.2 custou uma etapa para aparecer.
        assert!(!format!("{:?}", desta_imagem()).contains("465.8"));
    }

    #[test]
    fn sem_a_linha_do_disco_e_recusa_e_nao_zero() {
        let texto = "Model: KINGSTON SNV3S500G\nSector size (logical/physical): 512/512 bytes\n";
        assert_eq!(
            ler("x-gpt.sgdisk", texto),
            Err(SemMedida::SemLinhaDoDisco {
                arquivo: "x-gpt.sgdisk".to_string()
            })
        );
    }

    #[test]
    fn sem_o_tamanho_do_setor_e_recusa() {
        let texto = "Disk /dev/sda: 500103450 sectors, 238.5 GiB\nModel: KGSSE100256\n";
        assert_eq!(
            ler("sda-gpt.sgdisk", texto),
            Err(SemMedida::SemTamanhoDeSetor {
                arquivo: "sda-gpt.sgdisk".to_string()
            })
        );
    }

    #[test]
    fn zero_setor_nao_vira_medida() {
        let texto =
            "Disk /dev/sda: 0 sectors, 0 GiB\nSector size (logical/physical): 512/512 bytes\n";
        assert_eq!(
            ler("sda-gpt.sgdisk", texto),
            Err(SemMedida::MedidaZerada {
                arquivo: "sda-gpt.sgdisk".to_string()
            })
        );
    }

    #[test]
    fn texto_vazio_e_recusa() {
        assert!(ler("vazio", "").is_err());
    }

    #[test]
    fn o_nome_do_arquivo_sai_do_nome_do_disco() {
        assert_eq!(arquivo_do_disco("nvme0n1"), "nvme0n1-gpt.sgdisk");
        assert_eq!(arquivo_do_disco("sda"), "sda-gpt.sgdisk");
    }

    #[test]
    fn um_setor_de_4096_multiplica_certo() {
        // Nao ha original deste caso nesta mesa — os dois discos daqui tem
        // setor logico de 512. Ele existe porque a multiplicacao e o unico
        // lugar onde o tamanho do setor entra, e um `bytes()` que ignorasse o
        // campo passaria em todos os testes acima.
        let texto = "Disk /dev/sdb: 1000 sectors, 3.9 MiB\nModel: DISCO DE 4K\nSector size (logical/physical): 4096/4096 bytes\n";
        let origem = ler("sdb-gpt.sgdisk", texto).expect("legivel");
        assert_eq!(origem.bytes_por_setor, 4096);
        assert_eq!(origem.bytes(), 4_096_000);
    }

    #[test]
    fn sem_a_linha_do_modelo_e_recusa_e_nao_cadeia_vazia() {
        // Achado pela revisao de codigo da E9. O modelo e a **identidade** do
        // disco, e uma identidade vazia casa com tudo ou com nada conforme o
        // lado de que se olha: a conferencia de R-2 recusaria uma imagem
        // coerente por "as fontes discordam", e a busca do destino diria
        // "nenhum disco desta maquina tem o modelo ``".
        let texto = "Disk /dev/sda: 500103450 sectors, 238.5 GiB\nSector size (logical/physical): 512/512 bytes\n";
        assert_eq!(
            ler("sda-gpt.sgdisk", texto),
            Err(SemMedida::SemModelo {
                arquivo: "sda-gpt.sgdisk".to_string()
            })
        );

        // E um `Model:` presente e so com espaco tambem e vazio: o sgdisk
        // preenche essa coluna ate uma largura fixa, e o `trim` a esvazia.
        let so_espaco = "Disk /dev/sda: 500103450 sectors, 238.5 GiB\nModel:                    \nSector size (logical/physical): 512/512 bytes\n";
        assert!(matches!(
            ler("sda-gpt.sgdisk", so_espaco),
            Err(SemMedida::SemModelo { .. })
        ));
    }
}
