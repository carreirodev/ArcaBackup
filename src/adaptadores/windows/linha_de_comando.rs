//! Serializacao de argumentos para a linha de comando do Windows.
//!
//! Este e o ponto onde C-7 e C-8 vivem. Ao relancar o ARCA com elevacao, os
//! argumentos deixam de ser um vetor e viram uma string unica; do outro lado,
//! quem a reparte de volta e o `CommandLineToArgvW` — o parser do **Windows**,
//! nao o do PowerShell. Escapar com crase, que e o que o PowerShell entende,
//! faz a linha ser repartida no lugar errado e um argumento se perder pelo
//! caminho. Foi assim que `--dry-run` virou execucao real uma vez.
//!
//! A regra do `CommandLineToArgvW`, que este modulo implementa ao contrario:
//!
//! - `2n` barras invertidas seguidas de aspa produzem `n` barras e abrem ou
//!   fecham a citacao;
//! - `2n+1` barras invertidas seguidas de aspa produzem `n` barras e uma aspa
//!   literal;
//! - barras invertidas que nao antecedem uma aspa sao literais.

/// Caracteres que obrigam a citar o argumento. Espaco e tabulacao porque
/// separam argumentos; a aspa porque delimita; a quebra de linha e a
/// tabulacao vertical por seguranca.
const EXIGEM_CITACAO: [char; 5] = [' ', '\t', '\n', '\u{b}', '"'];

/// Cita um argumento para que o `CommandLineToArgvW` o devolva identico.
///
/// Argumento vazio vira um par de aspas — sem isso ele desapareceria da linha.
pub fn citar(argumento: &str) -> String {
    if !argumento.is_empty() && !argumento.contains(EXIGEM_CITACAO) {
        return argumento.to_string();
    }

    let mut saida = String::with_capacity(argumento.len() + 2);
    saida.push('"');
    let mut barras = 0usize;

    for caractere in argumento.chars() {
        match caractere {
            '\\' => {
                barras += 1;
                saida.push('\\');
            }
            '"' => {
                // As `barras` ja emitidas precisam dobrar, e a aspa precisa da
                // sua propria barra: `2n+1` barras e entao a aspa literal.
                for _ in 0..=barras {
                    saida.push('\\');
                }
                saida.push('"');
                barras = 0;
            }
            _ => {
                barras = 0;
                saida.push(caractere);
            }
        }
    }

    // Barras encostadas na aspa de fecho tambem dobram; do contrario a aspa
    // de fecho seria escapada e o argumento engoliria o proximo.
    for _ in 0..barras {
        saida.push('\\');
    }
    saida.push('"');
    saida
}

/// Monta a porcao de parametros de uma linha de comando — tudo menos o
/// programa, que o `ShellExecuteExW` recebe separado.
pub fn montar_parametros<T: AsRef<str>>(argumentos: &[T]) -> String {
    argumentos
        .iter()
        .map(|argumento| citar(argumento.as_ref()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Monta uma linha de comando completa, com o programa na frente.
pub fn montar_linha<T: AsRef<str>>(programa: &str, argumentos: &[T]) -> String {
    let parametros = montar_parametros(argumentos);
    if parametros.is_empty() {
        citar(programa)
    } else {
        format!("{} {}", citar(programa), parametros)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn argumento_simples_passa_intacto() {
        assert_eq!(citar("--dry-run"), "--dry-run");
        assert_eq!(citar("backup"), "backup");
        assert_eq!(citar("2026-08-22_Apps"), "2026-08-22_Apps");
    }

    #[test]
    fn argumento_vazio_vira_par_de_aspas() {
        assert_eq!(citar(""), r#""""#);
    }

    #[test]
    fn espaco_obriga_citacao() {
        assert_eq!(citar("nome com espaco"), r#""nome com espaco""#);
    }

    #[test]
    fn aspa_e_escapada_com_barra_invertida_e_nunca_com_crase() {
        let citado = citar(r#"diz "ola""#);
        assert_eq!(citado, r#""diz \"ola\"""#);
        assert!(!citado.contains('`'), "escape por crase e o erro de C-8");
    }

    #[test]
    fn barras_encostadas_na_aspa_dobram() {
        // Uma barra literal seguida de aspa vira duas barras (a literal) mais
        // uma barra e a aspa (a escapada).
        assert_eq!(citar(r#"a\""#), r#""a\\\"""#);
    }

    #[test]
    fn barras_no_fim_dobram_antes_do_fecho() {
        // So ha aspa de fecho quando ha citacao, e so ha citacao por causa do
        // espaco. Sem dobrar, a barra final escaparia a aspa e o argumento
        // engoliria o proximo.
        assert_eq!(
            citar(r"E:\Program Files\ARCA\"),
            r#""E:\Program Files\ARCA\\""#
        );
    }

    #[test]
    fn barras_no_fim_sem_citacao_ficam_como_estao() {
        // Fora de aspas nao ha o que escapar: barra invertida so tem
        // significado especial quando antecede uma aspa.
        assert_eq!(citar(r"E:\ARCA\"), r"E:\ARCA\");
    }

    #[test]
    fn barras_no_meio_nao_obrigam_citacao() {
        assert_eq!(citar(r"E:\ARCA\imagem"), r"E:\ARCA\imagem");
    }

    #[test]
    fn parametros_sao_unidos_por_espaco() {
        let argumentos = ["backup", "2026-08-22_Apps", "--dry-run"];
        assert_eq!(
            montar_parametros(&argumentos),
            "backup 2026-08-22_Apps --dry-run"
        );
    }

    #[test]
    fn linha_completa_cita_o_programa_quando_preciso() {
        assert_eq!(
            montar_linha(r"C:\Program Files\arca.exe", &["--version"]),
            r#""C:\Program Files\arca.exe" --version"#
        );
        assert_eq!(montar_linha("arca.exe", &[] as &[&str]), "arca.exe");
    }
}
