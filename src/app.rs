//! O despacho dos comandos.
//!
//! O contexto carrega as portas e a decisao de `--dry-run`. Cada etapa do
//! plano preenche um ramo do `match`; ate la o ramo diz qual etapa o entrega,
//! que e mais util do que o comando nao existir.

use crate::cli::{Cli, Comando};
use crate::erro::{Erro, Resultado};
use crate::portas::{Arquivos, Discos, Firmware, Relogio};
use crate::registro::Registro;

pub struct Contexto<'a> {
    /// Imprime o que seria feito e nao arma nada. Flag de primeira classe:
    /// todo comando que arma a respeita.
    pub dry_run: bool,
    pub registro: &'a Registro,
    pub firmware: &'a dyn Firmware,
    pub discos: &'a dyn Discos,
    pub arquivos: &'a dyn Arquivos,
    pub relogio: &'a dyn Relogio,
}

pub fn executar(cli: &Cli, contexto: &Contexto) -> Resultado<()> {
    contexto.registro.info(format!(
        "comando `{}`{}",
        cli.comando.nome(),
        if contexto.dry_run { " (ensaio)" } else { "" }
    ));

    let (comando, etapa) = match &cli.comando {
        Comando::Backup { .. } => ("backup", "E7"),
        Comando::Resultado => ("resultado", "E8"),
        Comando::List => ("list", "E1"),
        Comando::Restore => ("restore", "E9"),
        Comando::Verify { .. } => ("verify", "E11"),
        Comando::Status => ("status", "E2"),
        Comando::Prepare { .. } => ("prepare", "E10"),
    };

    Err(Erro::AindaNaoImplementado { comando, etapa })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{ArquivosEmMemoria, DiscosDeMentira, FirmwareDeMentira, RelogioParado};
    use clap::Parser;

    #[test]
    fn cada_comando_nao_construido_nomeia_a_etapa_que_o_entrega() {
        let arquivos = ArquivosEmMemoria::novo();
        let discos = DiscosDeMentira::default();
        let firmware = FirmwareDeMentira::novo();
        let relogio = RelogioParado::em("2026-08-22T11:42:03");
        let registro = Registro::em(
            std::env::temp_dir().join(format!("arca-despacho-{}", std::process::id())),
            Box::new(RelogioDoSistema),
        );

        let contexto = Contexto {
            dry_run: false,
            registro: &registro,
            firmware: &firmware,
            discos: &discos,
            arquivos: &arquivos,
            relogio: &relogio,
        };

        for (argumentos, etapa_esperada) in [
            (vec!["arca", "backup", "n"], "E7"),
            (vec!["arca", "resultado"], "E8"),
            (vec!["arca", "list"], "E1"),
            (vec!["arca", "restore"], "E9"),
            (vec!["arca", "verify", "n"], "E11"),
            (vec!["arca", "status"], "E2"),
            (vec!["arca", "prepare"], "E10"),
        ] {
            let cli = Cli::parse_from(&argumentos);
            let erro = executar(&cli, &contexto).unwrap_err();

            match erro {
                Erro::AindaNaoImplementado { etapa, .. } => {
                    assert_eq!(etapa, etapa_esperada, "para {argumentos:?}")
                }
                outro => panic!("esperava etapa nomeada, veio {outro}"),
            }
        }

        let _ = std::fs::remove_dir_all(registro.caminho().parent().unwrap());
    }
}
