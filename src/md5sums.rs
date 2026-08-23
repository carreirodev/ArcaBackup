//! O `MD5SUMS` que o Clonezilla deixa em toda imagem, e o que ele lista
//! (V-1, L-2, B-3).
//!
//! Este arquivo tem dois empregos no ARCA, e eles vinham em ordem inversa de
//! importancia. Desde a E1 ele e o que **separa imagem de residuo**
//! ([`crate::imagens`]) — pela existencia, sem ninguem nunca ter olhado
//! dentro. A E11 e a primeira etapa que o abre.
//!
//! # A forma e do Clonezilla, e foi medida antes de este modulo existir
//!
//! Lido do dispositivo desta mesa em 23/08/2026 e preservado em
//! `recursos/capturas/md5sums-2026-08-22_Apps.txt`, com a medicao ao lado em
//! `verificacao-md5-medida-2026-08-23.txt`:
//!
//! ```text
//! 2129 bytes, 39 linhas
//! LF (0x0A): 39     CR (0x0D): 0     ultimo byte: 0x0a
//! linha: <32 hex minusculos><dois espacos><nome do arquivo>
//! ```
//!
//! Sem cabecalho, sem comentario, sem linha em branco. E o formato do
//! `md5sum` do GNU, no modo texto — o modo binario traria ` *` no lugar dos
//! dois espacos, e **nenhuma das 39 linhas o usa**.
//!
//! # As quatro coisas que este modulo recusa, e cada uma tem motivo medido
//!
//! O criterio e o do
//! [ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md):
//! **o que nao se entende e recusado, nao adivinhado.** Aqui a recusa e barata
//! — quem for verificar tenta de novo — e o que ela evita e caro: uma
//! verificacao que pula linhas mal formadas diria `aprovada` tendo conferido
//! menos do que a imagem tem.
//!
//! - **Nome com separador de caminho, ou com `..`.** Os 39 nomes medidos sao
//!   planos, porque uma imagem do Clonezilla e uma pasta plana. Um
//!   `../../Windows/System32/config/SAM` no `MD5SUMS` faria o ARCA abrir e ler
//!   arquivo fora da imagem, e o `MD5SUMS` vem do dispositivo — que e a coisa
//!   que a verificacao existe para desconfiar.
//! - **Nome repetido.** Duas linhas para o mesmo arquivo nao dizem qual vale, e
//!   escolher e adivinhar. E o mesmo raciocinio de
//!   [`crate::desfecho::NaoEDesfecho::SeloRepetido`].
//! - **Arquivo vazio.** Zero linhas nao e "uma lista de nada a conferir": e um
//!   arquivo que nao se deixou escrever. Aprovar uma imagem tendo conferido
//!   zero arquivos e o pior desfecho possivel desta etapa.
//! - **Linha que nao case a forma.** Inclusive a linha em branco: o arquivo
//!   medido nao tem nenhuma.
//!
//! # A caixa e normalizada aqui, e no [`crate::receita::Selo`] nao e
//!
//! Os dois guardam digitos hexadecimais e as regras sao opostas, o que merece
//! explicacao. O selo **e** a identidade do job, e a caixa e parte dela: um
//! selo que mudasse de caixa entre o `estado.json` e o `arca-fim.txt` deixaria
//! de casar, e casar e a unica coisa que o selo faz.
//!
//! Uma soma MD5 nao e identidade: e um **numero em base 16**, e `AB` e `ab`
//! sao o mesmo numero. Do outro lado da comparacao esta o `certutil`, que
//! responde em minusculas, e o `md5sum` do GNU, que escreve em minusculas —
//! mas um `MD5SUMS` gerado por outra ferramenta em maiusculas continuaria
//! certo, e recusa-lo seria o ARCA reprovando uma imagem boa por causa de
//! caixa.

use crate::resumo::{Algoritmo, RecusaDoResumo, Resumo};
use std::fmt;

/// O arquivo, dentro da pasta da imagem.
///
/// Mora aqui, e nao em [`crate::imagens`], pelo mesmo motivo de
/// [`crate::imagens::CHECK_LOG`] e [`crate::receita::ARCA_FIM`] morarem cada um
/// no modulo que os entende: **um nome so, num lugar so.** Quem o usa sao dois
/// — a E1, que separa imagem de residuo pela existencia dele, e a E11, que o
/// abre — e os dois nao podem divergir em silencio.
pub const ARQUIVO: &str = "MD5SUMS";

/// Uma linha do `MD5SUMS`: o resumo e o arquivo a que ele pertence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    pub soma: Resumo,

    /// O nome do arquivo dentro da pasta da imagem. **Sempre plano** — ver a
    /// recusa [`RecusaDoMd5sums::NomeComCaminho`].
    pub arquivo: String,
}

/// Por que um `MD5SUMS` foi recusado.
///
/// Toda variante nomeia a linha em que aconteceu, contada a partir de 1: quem
/// for olhar o arquivo a mao precisa achar o lugar, e "o MD5SUMS esta
/// malformado" nao ajuda ninguem a fazer isso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoMd5sums {
    /// O arquivo nao tem linha nenhuma. Nao e "nada a conferir".
    Vazio,

    /// A linha nao tem a forma `<32 hex><separador><nome>`.
    LinhaSemForma { numero: usize, tem: String },

    /// O que esta no lugar do resumo nao e um MD5. A recusa vem inteira de
    /// [`crate::resumo`], que e quem sabe a forma dos dois algoritmos.
    SomaInvalida(RecusaDoResumo),

    /// O nome tem separador de caminho, componente `..`, ou e absoluto. Uma
    /// imagem do Clonezilla e uma pasta plana.
    NomeComCaminho { numero: usize, tem: String },

    /// Duas linhas para o mesmo arquivo.
    NomeRepetido { numero: usize, tem: String },
}

impl fmt::Display for RecusaDoMd5sums {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoMd5sums::Vazio => write!(
                f,
                "o `MD5SUMS` desta imagem nao tem linha nenhuma. Isto nao e `nada a conferir`: e um arquivo que nao se deixou escrever, e uma imagem aprovada sobre zero arquivos conferidos seria pior do que nenhuma resposta"
            ),
            RecusaDoMd5sums::LinhaSemForma { numero, tem } => write!(
                f,
                "a linha {numero} do `MD5SUMS` nao tem a forma que o Clonezilla escreve — {} digitos hexadecimais, dois espacos e o nome do arquivo. Ela tras `{tem}`",
                Algoritmo::Md5.digitos()
            ),
            RecusaDoMd5sums::SomaInvalida(recusa) => write!(f, "no `MD5SUMS`: {recusa}"),
            RecusaDoMd5sums::NomeComCaminho { numero, tem } => write!(
                f,
                "a linha {numero} do `MD5SUMS` nomeia `{tem}`, que sai da pasta da imagem. Uma imagem do Clonezilla e uma pasta plana, e o ARCA nao abre arquivo apontado por um `MD5SUMS` que aponta para fora dela"
            ),
            RecusaDoMd5sums::NomeRepetido { numero, tem } => write!(
                f,
                "a linha {numero} do `MD5SUMS` repete o arquivo `{tem}`, que ja tinha resumo. Duas linhas para o mesmo arquivo nao dizem qual vale, e o ARCA nao escolhe"
            ),
        }
    }
}

/// Lê o `MD5SUMS` inteiro, ou recusa dizendo em que linha parou.
///
/// Recusa o arquivo **inteiro** na primeira linha ruim, em vez de pular a
/// linha e seguir. Pular faria a verificacao conferir menos arquivos do que a
/// imagem tem e ainda assim dizer `aprovada`, e o ARCA nao tem como saber o
/// que havia na linha que ele nao entendeu.
pub fn ler(texto: &str) -> Result<Vec<Entrada>, RecusaDoMd5sums> {
    let mut entradas: Vec<Entrada> = Vec::new();

    for (indice, linha) in texto.lines().enumerate() {
        let numero = indice + 1;

        // `lines()` ja tirou o `\r\n` e o `\n`. O arquivo medido e LF puro,
        // porque quem o escreve e o Linux; o `\r` chegaria de uma copia feita
        // por ferramenta do Windows, e nesse caso o conteudo continua bom.
        let sem_forma = || RecusaDoMd5sums::LinhaSemForma {
            numero,
            tem: linha.to_string(),
        };

        if linha.chars().count() < Algoritmo::Md5.digitos() + 2 {
            return Err(sem_forma());
        }

        let (bruto, resto) = linha.split_at(Algoritmo::Md5.digitos());
        let soma = Resumo::novo(Algoritmo::Md5, bruto).map_err(RecusaDoMd5sums::SomaInvalida)?;

        // O `md5sum` do GNU escreve `<hash><espaco><modo><nome>`, com o modo
        // em branco no texto e `*` no binario. Medido neste dispositivo: as 39
        // linhas usam dois espacos, e nenhuma usa `*`. As duas sao aceitas
        // porque as duas sao o mesmo formato; o teste `o_modo_binario_...`
        // marca qual delas tem original aqui e qual nao tem.
        let arquivo = match resto.strip_prefix(' ') {
            Some(depois) => match depois.strip_prefix([' ', '*']) {
                Some(nome) => nome,
                None => return Err(sem_forma()),
            },
            None => return Err(sem_forma()),
        };

        if arquivo.is_empty() {
            return Err(sem_forma());
        }
        if sai_da_pasta(arquivo) {
            return Err(RecusaDoMd5sums::NomeComCaminho {
                numero,
                tem: arquivo.to_string(),
            });
        }
        // Sem diferenciar caixa, porque quem vai abrir o arquivo e o Windows,
        // onde `DISK` e `disk` sao o mesmo arquivo. Duas linhas que so diferem
        // na caixa apontariam para o mesmo lugar com resumos diferentes, e uma
        // delas reprovaria a imagem sozinha.
        if let Some(ja) = entradas
            .iter()
            .find(|entrada| entrada.arquivo.eq_ignore_ascii_case(arquivo))
        {
            return Err(RecusaDoMd5sums::NomeRepetido {
                numero,
                tem: ja.arquivo.clone(),
            });
        }

        entradas.push(Entrada {
            soma,
            arquivo: arquivo.to_string(),
        });
    }

    if entradas.is_empty() {
        return Err(RecusaDoMd5sums::Vazio);
    }

    Ok(entradas)
}

/// Se o nome alcanca alguma coisa fora da pasta da imagem.
///
/// Recusa a barra nas duas formas, o `..` como componente inteiro, e o nome
/// absoluto do Windows (`C:`). Nao tenta normalizar nem resolver: quem
/// normaliza acaba discutindo com o `\\?\` e com o `~1` do nome curto, e a
/// resposta certa aqui e nao abrir o arquivo.
fn sai_da_pasta(nome: &str) -> bool {
    nome.contains('/')
        || nome.contains('\\')
        || nome == ".."
        || nome == "."
        // `C:algo` e relativo ao diretorio corrente **daquela unidade**, e nao
        // a pasta da imagem. Nao precisa de barra para sair daqui.
        || nome.chars().nth(1) == Some(':')
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O `MD5SUMS` de `2026-08-22_Apps`, copiado do dispositivo em
    /// 23/08/2026. **E o oraculo desta etapa**: nenhum teste deste modulo pode
    /// ser ajustado para passar, porque o alvo e o arquivo que o Clonezilla
    /// escreveu.
    const CAPTURA: &str = include_str!("../recursos/capturas/md5sums-2026-08-22_Apps.txt");

    /// A primeira linha da captura, para os testes que precisam de uma linha
    /// boa ao lado de uma ruim.
    const UMA_LINHA: &str = "bf6850d736dc6b480994de0cee9c0f63  blkdev.json";

    #[test]
    fn a_captura_do_dispositivo_e_lida_inteira() {
        let entradas = ler(CAPTURA).expect("o MD5SUMS que o Clonezilla escreveu");

        assert_eq!(entradas.len(), 39, "a captura tem 39 linhas");
        assert_eq!(entradas[0].arquivo, "blkdev.json");
        assert_eq!(
            entradas[0].soma.como_texto(),
            "bf6850d736dc6b480994de0cee9c0f63"
        );
        assert_eq!(entradas[38].arquivo, "parts");
        assert_eq!(
            entradas[38].soma.como_texto(),
            "4492c326855a1f1cbfab9086bed27251"
        );
    }

    #[test]
    fn a_captura_lista_os_arquivos_da_imagem_e_nao_so_os_metadados() {
        // O achado que quase passou batido ao olhar so as pontas do arquivo:
        // a ordem **nao** e alfabetica pura, e os `nvme0n1p*` — os 39,7 GB —
        // ficam no meio, entre o `nvme0n1-mbr` e o `nvme0n1-pt.parted`. Quem
        // olhasse as primeiras e as ultimas linhas concluiria que o `MD5SUMS`
        // cobre so os metadados, e V-1 inteiro estaria construido sobre isso.
        let entradas = ler(CAPTURA).unwrap();
        let de_imagem = entradas
            .iter()
            .filter(|entrada| entrada.arquivo.contains("-ptcl-img.zst."))
            .count();

        assert_eq!(
            de_imagem, 14,
            "as quatro particoes do nvme0n1, com o p3 partido em onze pedacos"
        );
    }

    #[test]
    fn a_captura_e_modo_texto_e_nenhuma_linha_usa_o_binario() {
        // O que **tem** original neste dispositivo: dois espacos, e nenhum
        // asterisco. Fixado para que a aceitacao do modo binario abaixo nao
        // possa ser confundida com algo que foi medido aqui.
        assert!(
            CAPTURA.lines().all(|linha| linha
                .chars()
                .nth(Algoritmo::Md5.digitos() + 1)
                .is_some_and(|c| c == ' ')),
            "alguma linha da captura nao tem dois espacos"
        );
    }

    #[test]
    fn a_captura_e_lf_puro_como_saiu_do_linux() {
        // Quem escreve e o Linux, e por isso nao ha CR. Fixado porque o
        // `.gitattributes` marca `recursos/capturas/** -text` justamente para
        // que o git nao normalize isto — se alguem tirar aquela linha, este
        // teste fala.
        assert!(
            !CAPTURA.contains('\r'),
            "a captura ganhou CR: o `.gitattributes` deixou de proteger `recursos/capturas/`"
        );
    }

    #[test]
    fn o_modo_binario_do_md5sum_tambem_e_aceito() {
        // **Sem original neste dispositivo.** E a outra metade do formato do
        // `md5sum` — `md5sum -b` escreve ` *` no lugar dos dois espacos —, e
        // recusa-la faria o ARCA reprovar um `MD5SUMS` legitimo por causa de
        // um caractere que nao muda o resumo.
        let entradas = ler("bf6850d736dc6b480994de0cee9c0f63 *blkdev.json").unwrap();
        assert_eq!(entradas[0].arquivo, "blkdev.json");
    }

    #[test]
    fn a_caixa_do_resumo_e_normalizada() {
        // Ao contrario do selo, onde a caixa e parte da identidade. Aqui o
        // resumo e um numero em base 16, e o que esta do outro lado da
        // comparacao e o `certutil`, que responde em minusculas.
        let entradas = ler("BF6850D736DC6B480994DE0CEE9C0F63  blkdev.json").unwrap();
        assert_eq!(
            entradas[0].soma.como_texto(),
            "bf6850d736dc6b480994de0cee9c0f63"
        );
    }

    #[test]
    fn um_md5sums_vazio_e_recusa_e_nao_lista_vazia() {
        // "Nada a conferir" seria a resposta pior: uma imagem aprovada sobre
        // zero arquivos.
        assert_eq!(ler(""), Err(RecusaDoMd5sums::Vazio));

        // Um arquivo com so uma quebra de linha nao chega ao `Vazio`: ele tem
        // uma linha, e ela nao tem forma. As duas recusas sao respostas, e a
        // que importa e nao haver caminho por onde um `MD5SUMS` ilegivel
        // produza uma lista vazia que passe por conferida.
        assert!(ler("\n").is_err());
    }

    #[test]
    fn a_linha_em_branco_e_recusa_e_nao_e_pulada() {
        // O arquivo medido nao tem nenhuma. Pular linhas que nao se entende e
        // conferir menos do que a imagem tem, dizendo `aprovada` no fim.
        let erro = ler(&format!(
            "{UMA_LINHA}\n\ncee6e84e46cf5e1971efb6aac331eb18  blkdev.list"
        ))
        .unwrap_err();
        assert!(matches!(
            erro,
            RecusaDoMd5sums::LinhaSemForma { numero: 2, .. }
        ));
    }

    #[test]
    fn a_recusa_nomeia_a_linha_contada_a_partir_de_um() {
        let erro = ler(&format!("{UMA_LINHA}\nlixo")).unwrap_err();
        match erro {
            RecusaDoMd5sums::LinhaSemForma { numero, tem } => {
                assert_eq!(numero, 2, "quem olha o arquivo a mao conta de 1");
                assert_eq!(tem, "lixo");
            }
            outro => panic!("esperava linha sem forma, veio {outro:?}"),
        }
    }

    #[test]
    fn resumo_que_nao_e_md5_e_recusado() {
        // Curto, longo e com caractere fora do alfabeto.
        assert!(ler("abc  disk").is_err());
        assert!(matches!(
            ler("zf6850d736dc6b480994de0cee9c0f63  disk").unwrap_err(),
            RecusaDoMd5sums::SomaInvalida(_)
        ));
        assert!(ler("bf6850d736dc6b480994de0cee9c0f6333  disk").is_err());
    }

    #[test]
    fn um_separador_que_nao_e_o_do_formato_e_recusado() {
        // Um espaco so, tabulacao, ou nenhum: nenhum deles e o que o
        // Clonezilla escreve, e adivinhar qual era a intencao e o que este
        // modulo nao faz.
        for linha in [
            "bf6850d736dc6b480994de0cee9c0f63 blkdev.json",
            "bf6850d736dc6b480994de0cee9c0f63\t\tblkdev.json",
            "bf6850d736dc6b480994de0cee9c0f63blkdev.json",
        ] {
            assert!(ler(linha).is_err(), "aceitou `{linha}`");
        }
    }

    #[test]
    fn nome_vazio_depois_do_separador_e_recusado() {
        assert!(ler("bf6850d736dc6b480994de0cee9c0f63  ").is_err());
    }

    #[test]
    fn nome_que_sai_da_pasta_da_imagem_e_recusado() {
        // O `MD5SUMS` vem do dispositivo, que e a coisa que a verificacao
        // existe para desconfiar. Um nome com caminho faria o ARCA abrir e ler
        // arquivo de fora da imagem — inclusive do disco do sistema.
        for nome in [
            r"..\..\Windows\System32\config\SAM",
            "../../etc/shadow",
            "sub/disk",
            r"sub\disk",
            "..",
            ".",
            r"C:\Windows\win.ini",
            "C:disk",
        ] {
            let erro = ler(&format!("bf6850d736dc6b480994de0cee9c0f63  {nome}")).unwrap_err();
            assert!(
                matches!(erro, RecusaDoMd5sums::NomeComCaminho { .. }),
                "`{nome}` passou como nome plano"
            );
        }
    }

    #[test]
    fn nome_com_ponto_no_meio_continua_valendo() {
        // A recusa acima nao pode pegar os nomes de verdade: quatorze dos 39
        // arquivos da captura tem ponto no nome, e um deles e
        // `nvme0n1-pt.parted.compact`, com dois.
        let entradas = ler("bf6850d736dc6b480994de0cee9c0f63  nvme0n1-pt.parted.compact").unwrap();
        assert_eq!(entradas[0].arquivo, "nvme0n1-pt.parted.compact");
    }

    #[test]
    fn arquivo_repetido_e_recusado_inclusive_com_a_caixa_trocada() {
        // Quem vai abrir e o Windows, onde `DISK` e `disk` sao o mesmo
        // arquivo: duas linhas apontariam para o mesmo lugar com resumos
        // diferentes, e uma delas reprovaria a imagem sozinha.
        let erro = ler("bf6850d736dc6b480994de0cee9c0f63  disk
cee6e84e46cf5e1971efb6aac331eb18  DISK")
        .unwrap_err();

        match erro {
            RecusaDoMd5sums::NomeRepetido { numero, tem } => {
                assert_eq!(numero, 2);
                assert_eq!(tem, "disk", "a mensagem nomeia o que ja tinha resumo");
            }
            outro => panic!("esperava nome repetido, veio {outro:?}"),
        }
    }

    #[test]
    fn crlf_nao_atrapalha_a_leitura() {
        // O arquivo medido e LF puro, e uma copia feita por ferramenta do
        // Windows chegaria com CRLF sem que o conteudo mudasse. O `\r` no fim
        // do nome faria o ARCA procurar um arquivo que nao existe.
        let entradas = ler("bf6850d736dc6b480994de0cee9c0f63  blkdev.json\r\n").unwrap();
        assert_eq!(entradas[0].arquivo, "blkdev.json");
    }

    #[test]
    fn a_recusa_diz_o_que_houve_em_vez_de_um_codigo() {
        // Toda recusa deste modulo produz mensagem propria: nenhum desfecho do
        // ARCA e silencio (§5.5).
        for recusa in [
            RecusaDoMd5sums::Vazio,
            RecusaDoMd5sums::LinhaSemForma {
                numero: 2,
                tem: "lixo".to_string(),
            },
            RecusaDoMd5sums::SomaInvalida(RecusaDoResumo::Invalido {
                algoritmo: Algoritmo::Md5,
                tem: "abc".to_string(),
            }),
            RecusaDoMd5sums::NomeComCaminho {
                numero: 3,
                tem: "../x".to_string(),
            },
            RecusaDoMd5sums::NomeRepetido {
                numero: 4,
                tem: "disk".to_string(),
            },
        ] {
            let texto = recusa.to_string();
            assert!(texto.len() > 40, "mensagem curta demais: {texto}");
        }
    }
}
