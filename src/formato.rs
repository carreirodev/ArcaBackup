//! Numeros e datas do jeito que o §5.4 do PRD os mostra.
//!
//! Codigo puro: nao toca em disco, nao pergunta a hora. Existe separado
//! porque a saida do `arca list` e criterio de aceite da etapa E1, e criterio
//! de aceite merece teste que rode sem o dispositivo conectado.

use chrono::{DateTime, Local};

/// Base 1024, que e a do Explorador do Windows.
///
/// Nao e detalhe: a imagem validada em hardware ocupa 38,8 bilhoes de bytes e
/// o PRD a chama de `36,2 GB`. Em base 1000 ela sairia `38,8 GB`, e a saida
/// deixaria de bater com o documento que a especifica.
const UNIDADE: u64 = 1024;

/// Tamanho com uma casa decimal e virgula, como `36,2 GB`.
pub fn tamanho(bytes: u64) -> String {
    const ESCALAS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut valor = bytes as f64;
    let mut escala = 0;

    // A comparacao e com o valor **ja arredondado**, senao 2^30-1 fica em
    // 1023,99999 MB, escapa da subida de escala e aparece como `1024,0 MB` —
    // um numero que ninguem escreve, ao lado de um `36,2 GB` na mesma coluna.
    while escala + 1 < ESCALAS.len() && uma_casa(valor) >= UNIDADE as f64 {
        valor /= UNIDADE as f64;
        escala += 1;
    }

    if escala == 0 {
        return format!("{bytes} B");
    }
    format!("{} {}", com_virgula(valor), ESCALAS[escala])
}

/// O valor como ele vai aparecer: com uma casa decimal.
fn uma_casa(valor: f64) -> f64 {
    (valor * 10.0).round() / 10.0
}

/// Gigabytes inteiros, como o `183 GB livres` do §5.4.
///
/// Sem casa decimal de proposito: a pergunta que o rodape responde e "cabe
/// mais uma imagem?", e cem megabytes a mais ou a menos nao a mudam.
pub fn gigabytes(bytes: u64) -> String {
    let gigabytes = bytes as f64 / (UNIDADE * UNIDADE * UNIDADE) as f64;
    format!("{} GB", gigabytes.round() as u64)
}

/// `21/08`, como as imagens aparecem na listagem.
///
/// A data vem do sistema de arquivos, escrita pelo Clonezilla, que roda 3 h
/// adiantado (P-7). Serve para o usuario reconhecer a imagem, e nada mais —
/// quem liga um job ao seu desfecho e o selo (S-6).
pub fn dia_e_mes(momento: Option<DateTime<Local>>) -> String {
    match momento {
        Some(momento) => momento.format("%d/%m").to_string(),
        // Uma data que o sistema de arquivos nao soube dar nao vira hoje.
        None => "--/--".to_string(),
    }
}

/// Uma casa decimal, com a virgula do portugues.
fn com_virgula(valor: f64) -> String {
    format!("{valor:.1}").replace('.', ",")
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::momento;

    /// A soma dos arquivos de `2026-08-21_WindowsCompleto`, medida no
    /// dispositivo. O PRD chama esta imagem de `36,2 GB`.
    const IMAGEM_VALIDADA_BYTES: u64 = 38_823_623_035;

    #[test]
    fn a_imagem_validada_em_hardware_aparece_como_o_prd_a_chama() {
        assert_eq!(tamanho(IMAGEM_VALIDADA_BYTES), "36,2 GB");
    }

    #[test]
    fn o_tamanho_sobe_de_escala_e_usa_virgula() {
        assert_eq!(tamanho(0), "0 B");
        assert_eq!(tamanho(999), "999 B");
        assert_eq!(tamanho(1024), "1,0 KB");
        assert_eq!(tamanho(1024 * 1024 * 3 / 2), "1,5 MB");
        assert_eq!(tamanho(1024 * 1024 * 1024), "1,0 GB");
        assert_eq!(tamanho(1024 * 1024 * 1024 * 1024 * 2), "2,0 TB");
    }

    #[test]
    fn nas_bordas_a_escala_sobe_em_vez_de_dizer_1024() {
        assert_eq!(tamanho(1024 * 1024 - 1), "1,0 MB");
        assert_eq!(tamanho(1024 * 1024 * 1024 - 1), "1,0 GB");
        assert_eq!(tamanho(1024 * 1024 * 1024 * 1024 - 1), "1,0 TB");

        // O que arredonda para menos de 1024 fica onde esta.
        assert_eq!(tamanho(1024 * 1023), "1023,0 KB");
    }

    #[test]
    fn o_espaco_livre_do_rodape_e_inteiro() {
        assert_eq!(gigabytes(196_400_000_000), "183 GB");
        assert_eq!(gigabytes(0), "0 GB");
    }

    #[test]
    fn a_data_da_imagem_e_dia_e_mes() {
        assert_eq!(dia_e_mes(Some(momento("2026-08-21T12:56:31"))), "21/08");
    }

    #[test]
    fn sem_data_no_sistema_de_arquivos_a_coluna_diz_que_nao_sabe() {
        assert_eq!(dia_e_mes(None), "--/--");
    }
}
