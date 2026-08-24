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

/// `23/08 21:14`, para a sondagem (E12).
///
/// # Por que ela leva a hora e as imagens nao
///
/// A imagem se reconhece pelo nome, e a data e so uma dica; duas sondagens do
/// mesmo dia nao tem nome nenhum que as separe — a pasta e fixa e a segunda
/// substitui a primeira. Sem a hora, `lido da sondagem de 23/08` nao distingue
/// a medicao de cinco minutos atras da de manhã, e e justamente essa distancia
/// que decide se ela ainda descreve esta maquina.
///
/// **Continua sendo informativo**: nada compara este valor com nada (S-6).
///
/// # E o carimbo e do relogio do Clonezilla, nao do Windows
///
/// Quem escreve o `blkdev.list` e o `lsblk`, do outro lado do reinicio; o
/// Windows so lê o `mtime`. O valor sai **tres horas atras** do relogio daqui,
/// que e P-7 — medido de novo no marco da E12: sondagem armada as 14:56:55, e o
/// arquivo carimbado 11:58.
///
/// Esta funcao **nao corrige nada**: somar tres horas seria fabricar um
/// instante que ninguem mediu. Quem imprime e que diz de quem e o carimbo — ver
/// [`crate::blkdev::NomeDoDisco`].
pub fn dia_e_hora(momento: Option<DateTime<Local>>) -> String {
    match momento {
        Some(momento) => momento.format("%d/%m %H:%M").to_string(),
        None => "--/-- --:--".to_string(),
    }
}

/// Uma casa decimal, com a virgula do portugues.
fn com_virgula(valor: f64) -> String {
    format!("{valor:.1}").replace('.', ",")
}

/// Onde a coluna de valores comeca nas linhas pontilhadas do §5.2 do PRD.
///
/// Medida no proprio documento: `Desarmando receita anterior .....` e
/// `chkdsk /scan ....................` tem os dois 33 caracteres.
const COLUNA: usize = 33;

/// A linha `Rotulo ....... valor` do §5.2 do PRD.
///
/// Rotulo que nao caiba na coluna nao trunca nem quebra o alinhamento dos
/// outros: ele estoura, com um ponto so. Um diagnostico que esconde metade do
/// nome do que esta diagnosticando nao serve para nada.
pub fn linha(rotulo: &str, valor: &str) -> String {
    // Contado em caracteres, e nao em bytes: `{:<n$}` conta bytes, e um rotulo
    // com acento sairia desalinhado dos demais.
    let usados = rotulo.chars().count() + 1;
    let pontos = COLUNA.saturating_sub(usados).max(1);
    format!("  {rotulo} {} {valor}\n", ".".repeat(pontos))
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

    #[test]
    fn a_linha_pontilhada_e_a_do_paragrafo_5_2_do_prd() {
        assert_eq!(
            linha("Desarmando receita anterior", "ok"),
            "  Desarmando receita anterior ..... ok\n"
        );
        assert_eq!(
            linha("chkdsk /scan", "limpo"),
            "  chkdsk /scan .................... limpo\n"
        );
    }

    /// Em que coluna o valor comeca, contada como o console a desenha: em
    /// caracteres, e nao nos bytes que os representam.
    fn coluna_do_valor(linha: &str) -> usize {
        linha.chars().count() - linha.chars().rev().take_while(|c| *c != '.').count()
    }

    #[test]
    fn os_valores_ficam_todos_na_mesma_coluna() {
        assert_eq!(
            coluna_do_valor(&linha("Boot unico", "nao armado")),
            coluna_do_valor(&linha("x", "y"))
        );
    }

    #[test]
    fn rotulo_com_acento_nao_desalinha() {
        // Contado em bytes, `Descrição` ocuparia uma casa a mais do que ocupa,
        // e a coluna sairia torta ao lado das outras.
        assert_eq!(
            coluna_do_valor(&linha("Descrição", "ARCA")),
            coluna_do_valor(&linha("Descricao", "ARCA"))
        );
    }

    #[test]
    fn rotulo_longo_demais_estoura_em_vez_de_truncar() {
        let saida = linha(&"x".repeat(40), "ok");
        assert!(saida.contains(&"x".repeat(40)), "o rotulo foi truncado");
        assert!(saida.contains(". ok"), "sumiu o separador");
    }
}
