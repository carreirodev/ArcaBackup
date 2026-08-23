//! Resumos criptograficos de arquivo, e o leitor do `certutil` (V-1, PR-1).
//!
//! Dois usos, um mecanismo: o **MD5** de cada arquivo listado no `MD5SUMS`
//! (V-1, etapa E11) e o **SHA256** do pacote do Clonezilla contra a constante
//! compilada no binario (PR-1, etapa E10).
//!
//! # Sem dependencia nova, e isso foi medido antes de decidir
//!
//! O caminho de sempre seria um crate de hash — `md-5`, `sha2` — e a arvore
//! que vem com ele. O `certutil.exe` do `System32` faz as duas coisas, e o
//! padrao de falar com o sistema por processo filho atras de porta existe
//! desde a E6 (`powercfg`, `chkdsk`, `shutdown`).
//!
//! O argumento de desempenho, que seria o unico a favor do crate, **nao
//! existe**: medido em 23/08/2026 sobre a `2026-08-22_Apps` inteira, os 39
//! arquivos saem a 200,5 MB/s, e um arquivo sozinho de 812 MB sai a
//! 202,2 MB/s. As duas taxas sao a mesma dentro do ruido — os 39 processos
//! `certutil` nao custam nada perto da leitura, porque quem manda e o USB. Um
//! MD5 em Rust puro leria pelo mesmo cabo.
//!
//! As tres dependencias do projeto continuam tres.
//!
//! # As duas regras do leitor, e as duas ja custaram caro neste projeto
//!
//! Medido em 23/08/2026, e preservado em
//! `recursos/capturas/verificacao-md5-medida-2026-08-23.txt`:
//!
//! ```text
//! [0] <MD5 hash de D:\2026-08-22_Apps\disk:>
//! [1] <abfcd722bf8588a8377df1f5df0726b3>
//! [2] <CertUtil: -hashfile : comando concluido com exito.>
//! exit=0
//! ```
//!
//! **Julgar pelo codigo de saida, nunca pelo texto.** As linhas 0 e 2 vem
//! traduzidas, como o `chkdsk` de B-6 e o `powercfg` de B-5 — e parsear frase
//! traduzida e o erro que a E2 nomeou e que a correcao D10 do plano registrou.
//! Um arquivo ausente responde `exit=-2147024894` e duas linhas de frase, sem
//! hash nenhum.
//!
//! **Achar o hash pela forma, nunca pela posicao.** O hash e a unica linha que
//! e *exatamente* N digitos hexadecimais: a linha 0 sempre tras prefixo e
//! sufixo em volta do caminho, e a linha 2 sempre comeca por `CertUtil:`.
//! Depender de "e a segunda linha" seria depender de o `certutil` nunca
//! acrescentar uma linha — e a forma nao depende disso nem de idioma.
//!
//! Havendo mais de uma linha com a forma, isto **recusa** em vez de escolher a
//! primeira. E o mesmo raciocinio do
//! [`crate::desfecho::NaoEDesfecho::SeloRepetido`]: duas respostas nao dizem
//! qual vale.

use crate::portas::SaidaDeFerramenta;
use std::fmt;

/// Qual resumo se pede, e como o `certutil` o chama.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algoritmo {
    /// O do `MD5SUMS` que o Clonezilla escreve em toda imagem (V-1).
    Md5,

    /// O do pacote do Clonezilla, conferido contra a constante compilada no
    /// binario do ARCA (PR-1). Quem o usa e a etapa E10.
    Sha256,
}

impl Algoritmo {
    /// Quantos digitos hexadecimais o resumo tem.
    pub fn digitos(self) -> usize {
        match self {
            Algoritmo::Md5 => 32,
            Algoritmo::Sha256 => 64,
        }
    }

    /// O nome que entra na linha de comando do `certutil`.
    ///
    /// Maiusculo como as duas capturas o mostram. O `certutil` aceita as duas
    /// caixas; escreve-se como foi medido.
    pub fn como_certutil_o_chama(self) -> &'static str {
        match self {
            Algoritmo::Md5 => "MD5",
            Algoritmo::Sha256 => "SHA256",
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Algoritmo::Md5 => "MD5",
            Algoritmo::Sha256 => "SHA256",
        }
    }
}

/// Um resumo conferido, sempre em minusculas.
///
/// So se constroi por [`Resumo::novo`] ou [`do_certutil`], que conferem a
/// forma e normalizam a caixa. Ter um em maos e ter a garantia de que os dois
/// lados da comparacao falam o mesmo alfabeto — o `certutil` responde em
/// minusculas e o `md5sum` do GNU escreve em minusculas, e mesmo assim
/// normaliza-se, porque `AB` e `ab` sao o mesmo numero em base 16 e reprovar
/// uma imagem boa por causa de caixa seria o pior desfecho desta etapa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resumo {
    algoritmo: Algoritmo,
    digitos: String,
}

impl Resumo {
    pub fn novo(algoritmo: Algoritmo, bruto: &str) -> Result<Resumo, RecusaDoResumo> {
        if bruto.chars().count() != algoritmo.digitos()
            || !bruto.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(RecusaDoResumo::Invalido {
                algoritmo,
                tem: bruto.to_string(),
            });
        }

        Ok(Resumo {
            algoritmo,
            digitos: bruto.to_ascii_lowercase(),
        })
    }

    pub fn algoritmo(&self) -> Algoritmo {
        self.algoritmo
    }

    pub fn como_texto(&self) -> &str {
        &self.digitos
    }

    /// Os primeiros doze digitos, para caber numa linha de tela.
    ///
    /// Doze porque e o que basta para uma pessoa reconhecer que dois resumos
    /// impressos lado a lado sao diferentes, e nunca para **decidir** que sao
    /// iguais: quem decide e a comparacao, sobre o valor inteiro.
    pub fn abreviado(&self) -> String {
        self.digitos.chars().take(12).collect()
    }
}

impl fmt::Display for Resumo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.digitos)
    }
}

/// Por que a resposta do `certutil` nao produziu um resumo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoResumo {
    /// O texto nao tem a forma de um resumo daquele algoritmo.
    Invalido { algoritmo: Algoritmo, tem: String },

    /// O `certutil` saiu com codigo diferente de zero.
    FerramentaRecusou { codigo: i32, saida: String },

    /// Saiu com codigo zero e nenhuma linha tem a forma de um resumo.
    SemResumoNaSaida { algoritmo: Algoritmo, saida: String },

    /// Saiu com codigo zero e **mais de uma** linha tem a forma. Nao se
    /// escolhe a primeira.
    ResumoAmbiguo { quantos: usize, saida: String },
}

impl fmt::Display for RecusaDoResumo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoResumo::Invalido { algoritmo, tem } => write!(
                f,
                "`{tem}` nao e um resumo {}: sao {} digitos hexadecimais",
                algoritmo.nome(),
                algoritmo.digitos()
            ),
            RecusaDoResumo::FerramentaRecusou { codigo, saida } => write!(
                f,
                "o `certutil` saiu com codigo {codigo} e nao resumiu o arquivo: {saida}"
            ),
            RecusaDoResumo::SemResumoNaSaida { algoritmo, saida } => write!(
                f,
                "o `certutil` saiu com exito e nenhuma linha da resposta e um resumo {} de {} digitos. O ARCA acha o resumo pela forma, e nao pela posicao nem pelo texto — que vem traduzido. A resposta foi: {saida}",
                algoritmo.nome(),
                algoritmo.digitos()
            ),
            RecusaDoResumo::ResumoAmbiguo { quantos, saida } => write!(
                f,
                "a resposta do `certutil` tras {quantos} linhas com forma de resumo, e o ARCA nao escolhe entre elas. A resposta foi: {saida}"
            ),
        }
    }
}

/// O resumo que o `certutil` respondeu, julgado pelo codigo de saida e achado
/// pela forma.
///
/// Ver o cabecalho deste modulo para por que estas duas regras, e nao "a
/// segunda linha".
pub fn do_certutil(
    saida: &SaidaDeFerramenta,
    algoritmo: Algoritmo,
) -> Result<Resumo, RecusaDoResumo> {
    // 1. O codigo de saida decide se houve resposta. Antes de olhar o texto,
    //    porque o texto de uma falha tambem tem linhas — duas, medidas — e
    //    nenhuma delas e hash.
    if saida.codigo != 0 {
        return Err(RecusaDoResumo::FerramentaRecusou {
            codigo: saida.codigo,
            saida: saida.resumo(3),
        });
    }

    // 2. A forma acha a linha. `trim` porque o `certutil` nao indenta a linha
    //    do hash, e um espaco a mais nao muda o que ela e.
    let candidatas: Vec<&str> = saida
        .texto
        .lines()
        .map(str::trim)
        .filter(|linha| {
            linha.chars().count() == algoritmo.digitos()
                && linha.chars().all(|c| c.is_ascii_hexdigit())
        })
        .collect();

    match candidatas.as_slice() {
        [uma] => Resumo::novo(algoritmo, uma),
        [] => Err(RecusaDoResumo::SemResumoNaSaida {
            algoritmo,
            saida: saida.resumo(3),
        }),
        muitas => Err(RecusaDoResumo::ResumoAmbiguo {
            quantos: muitas.len(),
            saida: saida.resumo(4),
        }),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    /// A resposta de `certutil -hashfile D:\2026-08-22_Apps\disk MD5`,
    /// transcrita da medicao de 23/08/2026. As linhas 0 e 2 estao **em
    /// portugues**, que e como esta maquina responde.
    const CERTUTIL_MD5: &str = concat!(
        "MD5 hash de D:\\2026-08-22_Apps\\disk:\r\n",
        "abfcd722bf8588a8377df1f5df0726b3\r\n",
        "CertUtil: -hashfile : comando concluido com exito.\r\n"
    );

    /// A mesma chamada com `SHA256`, medida no mesmo arquivo.
    const CERTUTIL_SHA256: &str = concat!(
        "SHA256 hash de D:\\2026-08-22_Apps\\disk:\r\n",
        "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10\r\n",
        "CertUtil: -hashfile : comando concluido com exito.\r\n"
    );

    /// O que sai quando o arquivo nao existe. Duas linhas, nenhuma e hash.
    const CERTUTIL_FALHOU: &str = concat!(
        "CertUtil: -hashfile comando FALHOU: 0x80070002 (WIN32: 2 ERROR_FILE_NOT_FOUND)\r\n",
        "CertUtil: O sistema nao pode encontrar o arquivo especificado.\r\n"
    );

    fn saida(codigo: i32, texto: &str) -> SaidaDeFerramenta {
        SaidaDeFerramenta {
            codigo,
            texto: texto.to_string(),
        }
    }

    #[test]
    fn o_md5_sai_da_resposta_medida_nesta_maquina() {
        let resumo = do_certutil(&saida(0, CERTUTIL_MD5), Algoritmo::Md5).unwrap();
        assert_eq!(resumo.como_texto(), "abfcd722bf8588a8377df1f5df0726b3");

        // E o valor bate com o que o `MD5SUMS` da imagem lista para `disk` —
        // duas fontes independentes do mesmo numero, uma escrita pelo Linux
        // em 22/08 e outra pelo Windows em 23/08. E o oraculo desta etapa: o
        // teste nao pode ser ajustado para passar, porque os dois lados sao
        // arquivos que ferramentas de outra gente escreveram.
        const MD5SUMS: &str = include_str!("../recursos/capturas/md5sums-2026-08-22_Apps.txt");
        assert!(
            MD5SUMS.contains(&format!("{resumo}  disk")),
            "o MD5 que o certutil mediu nao e o que o Clonezilla registrou"
        );
    }

    #[test]
    fn o_sha256_sai_da_mesma_forma() {
        // Quem usa `Sha256` e a etapa E10 (PR-1). O leitor entra aqui com
        // original — esta resposta foi medida em 23/08/2026, no mesmo arquivo
        // e na mesma sessao do MD5 acima.
        let resumo = do_certutil(&saida(0, CERTUTIL_SHA256), Algoritmo::Sha256).unwrap();
        assert_eq!(
            resumo.como_texto(),
            "1d76439d27aa20ec74a8e22c486dcfab67473b6fd6bbc7376a806fede0293b10"
        );
        assert_eq!(resumo.algoritmo(), Algoritmo::Sha256);
    }

    #[test]
    fn o_codigo_de_saida_decide_antes_do_texto() {
        // O texto de uma falha tambem tem linhas, e nenhuma e hash. Olhar o
        // texto primeiro daria a recusa errada — "nao achei resumo" em vez de
        // "a ferramenta recusou" —, e as duas pedem coisas diferentes de quem
        // lê.
        let erro = do_certutil(&saida(-2147024894, CERTUTIL_FALHOU), Algoritmo::Md5).unwrap_err();
        match erro {
            RecusaDoResumo::FerramentaRecusou { codigo, .. } => assert_eq!(codigo, -2147024894),
            outro => panic!("esperava recusa da ferramenta, veio {outro:?}"),
        }
    }

    #[test]
    fn um_codigo_zero_com_texto_de_falha_e_recusa_e_nao_hash() {
        // O modo de falha de P-6 aplicado ao `certutil`: sair zero sem ter
        // feito. Nao ha resumo na saida, e o ARCA diz isso em vez de inventar.
        let erro = do_certutil(&saida(0, CERTUTIL_FALHOU), Algoritmo::Md5).unwrap_err();
        assert!(matches!(erro, RecusaDoResumo::SemResumoNaSaida { .. }));
    }

    #[test]
    fn a_linha_do_caminho_nunca_e_confundida_com_o_hash() {
        // O caso que "a segunda linha" resolveria por acidente e a forma
        // resolve por construcao: um arquivo **chamado** como um hash. A
        // linha 0 tras prefixo e sufixo em volta do caminho, entao ela nunca
        // e exatamente 32 digitos.
        let texto = concat!(
            "MD5 hash de D:\\x\\abfcd722bf8588a8377df1f5df0726b3:\r\n",
            "0123456789abcdef0123456789abcdef\r\n",
            "CertUtil: -hashfile : comando concluido com exito.\r\n"
        );
        let resumo = do_certutil(&saida(0, texto), Algoritmo::Md5).unwrap();
        assert_eq!(resumo.como_texto(), "0123456789abcdef0123456789abcdef");
    }

    #[test]
    fn a_posicao_nao_importa_e_o_idioma_tambem_nao() {
        // A mesma resposta em ingles, e com uma linha a mais na frente. Se o
        // leitor dependesse de "linha 1", as duas quebrariam.
        let texto = concat!(
            "\r\n",
            "MD5 hash of file disk:\r\n",
            "abfcd722bf8588a8377df1f5df0726b3\r\n",
            "CertUtil: -hashfile command completed successfully.\r\n"
        );
        let resumo = do_certutil(&saida(0, texto), Algoritmo::Md5).unwrap();
        assert_eq!(resumo.como_texto(), "abfcd722bf8588a8377df1f5df0726b3");
    }

    #[test]
    fn duas_linhas_com_forma_de_resumo_sao_recusa_e_nao_a_primeira() {
        // Nao se escolhe entre duas respostas. Mesmo raciocinio do selo
        // repetido em `crate::desfecho`.
        let texto = concat!(
            "MD5 hash de x:\r\n",
            "abfcd722bf8588a8377df1f5df0726b3\r\n",
            "0123456789abcdef0123456789abcdef\r\n"
        );
        let erro = do_certutil(&saida(0, texto), Algoritmo::Md5).unwrap_err();
        match erro {
            RecusaDoResumo::ResumoAmbiguo { quantos, .. } => assert_eq!(quantos, 2),
            outro => panic!("esperava ambiguidade, veio {outro:?}"),
        }
    }

    #[test]
    fn um_md5_nao_passa_por_sha256_nem_o_contrario() {
        // Os dois sao hexadecimais e so o comprimento os separa. Pedir SHA256
        // e receber 32 digitos e a resposta errada, e nao uma resposta curta.
        assert!(do_certutil(&saida(0, CERTUTIL_MD5), Algoritmo::Sha256).is_err());
        assert!(do_certutil(&saida(0, CERTUTIL_SHA256), Algoritmo::Md5).is_err());
    }

    #[test]
    fn a_caixa_e_normalizada_na_leitura() {
        let texto = "MD5 hash de x:\r\nABFCD722BF8588A8377DF1F5DF0726B3\r\n";
        let resumo = do_certutil(&saida(0, texto), Algoritmo::Md5).unwrap();
        assert_eq!(resumo.como_texto(), "abfcd722bf8588a8377df1f5df0726b3");
    }

    #[test]
    fn dois_resumos_iguais_em_caixas_diferentes_sao_o_mesmo() {
        let minusculo = Resumo::novo(Algoritmo::Md5, "abfcd722bf8588a8377df1f5df0726b3").unwrap();
        let maiusculo = Resumo::novo(Algoritmo::Md5, "ABFCD722BF8588A8377DF1F5DF0726B3").unwrap();
        assert_eq!(minusculo, maiusculo);
    }

    #[test]
    fn o_abreviado_nunca_e_o_bastante_para_decidir() {
        let resumo = Resumo::novo(Algoritmo::Md5, "abfcd722bf8588a8377df1f5df0726b3").unwrap();
        assert_eq!(resumo.abreviado(), "abfcd722bf85");
        assert!(resumo.abreviado().chars().count() < Algoritmo::Md5.digitos());
    }

    #[test]
    fn os_digitos_de_cada_algoritmo_sao_os_do_padrao() {
        assert_eq!(Algoritmo::Md5.digitos(), 32);
        assert_eq!(Algoritmo::Sha256.digitos(), 64);
        assert_eq!(Algoritmo::Md5.como_certutil_o_chama(), "MD5");
        assert_eq!(Algoritmo::Sha256.como_certutil_o_chama(), "SHA256");
    }
}
