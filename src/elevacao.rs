//! Garantir que o ARCA rode elevado, com os argumentos intactos (C-7).
//!
//! Ha dois caminhos ate a elevacao, e o ARCA usa os dois. O primeiro e o
//! manifesto `requireAdministrator` embutido no executavel: o Windows eleva
//! antes de o programa comecar e repassa a linha de comando ele mesmo, sem
//! que nada nosso a serialize. O segundo e este modulo, para quando o
//! manifesto nao vigora — o processo se descobre sem privilegio e se relanca.
//!
//! O que este modulo nao faz, de proposito, e reconstruir os argumentos a
//! partir do que o `clap` entendeu. Repassa os **brutos**, como chegaram. Foi
//! por reconstrui-los que `--dry-run` virou execucao real uma vez: a flag
//! existia no vetor original, nao existia na reconstrucao, e ninguem avisou.

use crate::erro::Resultado;
use crate::portas::Privilegios;

#[derive(Debug, PartialEq, Eq)]
pub enum Rumo {
    /// Este processo esta elevado: segue e faz o trabalho.
    Seguir,
    /// Quem fez o trabalho foi o processo elevado; propaga o codigo dele.
    Propagar(i32),
}

/// Tudo que veio depois do nome do executavel, na ordem em que veio.
pub fn argumentos_a_repassar(brutos: &[String]) -> Vec<String> {
    brutos.iter().skip(1).cloned().collect()
}

/// Segue elevado, ou relanca e devolve o codigo do processo elevado.
pub fn garantir(privilegios: &dyn Privilegios, brutos: &[String]) -> Resultado<Rumo> {
    if privilegios.elevado()? {
        return Ok(Rumo::Seguir);
    }
    let codigo = privilegios.relancar_elevado(&argumentos_a_repassar(brutos))?;
    Ok(Rumo::Propagar(codigo))
}

/// Traduz o codigo do processo elevado para o byte que este processo pode
/// devolver.
///
/// O que nao cabe num byte vira `1`, nunca `0`. As terminacoes anormais do
/// Windows sao codigos negativos — `0xC000013A` quando a janela e fechada, por
/// exemplo — e reduzi-las a zero diria "deu certo" sobre um processo que
/// morreu no meio.
pub fn codigo_de_saida_local(codigo_do_elevado: i32) -> u8 {
    u8::try_from(codigo_do_elevado).unwrap_or(1)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::PrivilegiosDeMentira;
    use crate::erro::Erro;

    fn brutos(argumentos: &[&str]) -> Vec<String> {
        argumentos.iter().map(|a| a.to_string()).collect()
    }

    #[test]
    fn o_executavel_nao_entra_no_repasse() {
        let repassados =
            argumentos_a_repassar(&brutos(&[r"C:\ARCA\arca.exe", "backup", "2026-08-22_Apps"]));
        assert_eq!(repassados, vec!["backup", "2026-08-22_Apps"]);
    }

    #[test]
    fn sem_argumentos_o_repasse_e_vazio() {
        assert!(argumentos_a_repassar(&brutos(&["arca.exe"])).is_empty());
    }

    #[test]
    fn processo_elevado_segue_sem_relancar() {
        let privilegios = PrivilegiosDeMentira::elevado();
        let rumo = garantir(&privilegios, &brutos(&["arca.exe", "--version"])).unwrap();

        assert_eq!(rumo, Rumo::Seguir);
        assert!(privilegios.ultimo_repasse().is_none());
    }

    #[test]
    fn dry_run_atravessa_a_elevacao() {
        let privilegios = PrivilegiosDeMentira::sem_elevacao();
        let originais = brutos(&["arca.exe", "backup", "2026-08-22_Apps", "--dry-run"]);

        let rumo = garantir(&privilegios, &originais).unwrap();

        assert_eq!(rumo, Rumo::Propagar(0));
        assert_eq!(
            privilegios.ultimo_repasse().unwrap(),
            vec!["backup", "2026-08-22_Apps", "--dry-run"],
            "perder o --dry-run aqui e a armadilha que transformou um ensaio em execucao real"
        );
    }

    #[test]
    fn o_codigo_do_processo_elevado_e_propagado() {
        let mut privilegios = PrivilegiosDeMentira::sem_elevacao();
        privilegios.codigo_do_relancamento = 2;

        assert_eq!(
            garantir(&privilegios, &brutos(&["arca.exe", "status"])).unwrap(),
            Rumo::Propagar(2)
        );
    }

    #[test]
    fn nao_saber_se_esta_elevado_nao_dispara_relancamento() {
        // Tratar "nao sei" como "nao elevado" poria o ARCA numa fila de
        // prompts de UAC sem fim: cada filho falharia na mesma consulta e
        // relancaria outro, com o pai preso esperando.
        let privilegios = PrivilegiosDeMentira::indeterminado();
        let erro = garantir(&privilegios, &brutos(&["arca.exe", "status"])).unwrap_err();

        assert!(matches!(erro, Erro::ElevacaoIndeterminada(_)));
        assert!(
            privilegios.ultimo_repasse().is_none(),
            "nao pode ter relancado"
        );
    }

    #[test]
    fn terminacao_anormal_do_processo_elevado_nao_vira_sucesso() {
        // `0xC000013A` e o que o Windows devolve quando a janela do processo
        // elevado e fechada. Truncar isso para um byte daria zero.
        assert_eq!(codigo_de_saida_local(-1073741510), 1);
        assert_eq!(codigo_de_saida_local(-1), 1);
        assert_eq!(codigo_de_saida_local(300), 1);
    }

    #[test]
    fn codigo_que_cabe_num_byte_atravessa_intacto() {
        assert_eq!(codigo_de_saida_local(0), 0);
        assert_eq!(codigo_de_saida_local(2), 2);
        assert_eq!(codigo_de_saida_local(255), 255);
    }

    #[test]
    fn elevacao_recusada_tem_mensagem_propria() {
        let privilegios = PrivilegiosDeMentira::recusando();
        let erro = garantir(&privilegios, &brutos(&["arca.exe", "list"])).unwrap_err();

        assert!(matches!(erro, Erro::ElevacaoRecusada));
        assert!(erro.to_string().contains("elevacao recusada"));
    }
}
