//! O despacho dos comandos.
//!
//! O contexto carrega as portas e a decisao de `--dry-run`. Cada etapa do
//! plano preenche um ramo do `match`; ate la o ramo diz qual etapa o entrega,
//! que e mais util do que o comando nao existir.

use crate::cli::{Cli, Comando};
use crate::comandos;
use crate::erro::{Erro, Resultado};
use crate::portas::{Arquivos, Discos, Firmware, Relogio, Sistema};
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

    /// As operacoes do proprio sistema: Inicializacao Rapida (B-5) e `chkdsk`
    /// (B-6). Quem as usa e o pre-voo da etapa E6.
    pub sistema: &'a dyn Sistema,
}

pub fn executar(cli: &Cli, contexto: &Contexto) -> Resultado<()> {
    contexto.registro.info(format!(
        "comando `{}`{}",
        cli.comando.nome(),
        if contexto.dry_run { " (ensaio)" } else { "" }
    ));

    let (comando, etapa) = match &cli.comando {
        Comando::List => return comandos::list::executar(contexto),
        Comando::Status => return comandos::status::executar(contexto),

        // C-1: desarmar acontece incondicionalmente e sem consultar estado
        // nenhum. Continua sendo o primeiro passo dos comandos que armam — a
        // E7 e a E8 o chamam de dentro; aqui ele tambem e alcancavel sozinho,
        // que e o que responde ao caso "o boot nao aconteceu" do §5.5.
        Comando::Desarmar => return comandos::desarmar::executar(contexto),

        // Com `--dry-run` o backup ja monta e imprime as receitas (E3); sem
        // ele, quem arma e a E7 — e e o proprio comando que diz isso, porque
        // o nome ainda precisa ser julgado por B-2 antes de qualquer resposta.
        Comando::Backup { nome } => return comandos::backup::executar(contexto, nome),

        Comando::Resultado => ("resultado", "E8"),
        Comando::Restore => ("restore", "E9"),
        Comando::Verify { .. } => ("verify", "E11"),
        Comando::Prepare { .. } => ("prepare", "E10"),
    };

    Err(Erro::AindaNaoImplementado { comando, etapa })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{
        ArquivosEmMemoria, DiscosDeMentira, FirmwareDeMentira, RelogioParado, SistemaDeMentira,
    };
    use clap::Parser;

    #[test]
    fn cada_comando_nao_construido_nomeia_a_etapa_que_o_entrega() {
        let arquivos = ArquivosEmMemoria::novo();
        let discos = DiscosDeMentira::default();
        let firmware = FirmwareDeMentira::novo();
        let relogio = RelogioParado::em("2026-08-22T11:42:03");
        let sistema = SistemaDeMentira::novo();
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
            sistema: &sistema,
        };

        for (argumentos, etapa_esperada) in [
            (vec!["arca", "resultado"], "E8"),
            (vec!["arca", "restore"], "E9"),
            (vec!["arca", "verify", "n"], "E11"),
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

    #[test]
    fn os_comandos_ja_construidos_fazem_o_trabalho_em_vez_de_nomear_etapa() {
        // `list` e `status` desde a E1 e a E2; `backup` entrou nesta lista na
        // **E6**, quando deixou de responder "armar e a E7" para rodar o
        // pre-voo do §5.2. Ele continua terminando antes de armar — quem
        // confirma e arma e a E7 —, mas o que ele faz ate la e trabalho de
        // verdade, e nao um aviso.
        //
        // Sem dispositivo conectado, os tres devolvem a recusa da descoberta —
        // e nunca `AindaNaoImplementado`.
        let arquivos = ArquivosEmMemoria::novo();
        let discos = DiscosDeMentira::default();
        let firmware = FirmwareDeMentira::novo();
        let relogio = RelogioParado::em("2026-08-22T11:42:03");
        let sistema = SistemaDeMentira::novo();
        let registro = Registro::em(
            std::env::temp_dir().join(format!("arca-list-{}", std::process::id())),
            Box::new(RelogioDoSistema),
        );

        let contexto = Contexto {
            dry_run: false,
            registro: &registro,
            firmware: &firmware,
            discos: &discos,
            arquivos: &arquivos,
            relogio: &relogio,
            sistema: &sistema,
        };

        for argumentos in [
            vec!["arca", "list"],
            vec!["arca", "status"],
            vec!["arca", "backup", "2026-08-22_Apps"],
        ] {
            let erro = executar(&Cli::parse_from(&argumentos), &contexto).unwrap_err();
            assert!(
                matches!(erro, Erro::DispositivoAusente),
                "{argumentos:?}: esperava a recusa da descoberta, veio {erro}"
            );
        }

        let _ = std::fs::remove_dir_all(registro.caminho().parent().unwrap());
    }
}
