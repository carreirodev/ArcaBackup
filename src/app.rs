//! O despacho dos comandos.
//!
//! O contexto carrega as portas e a decisao de `--dry-run`. Cada etapa do
//! plano preenche um ramo do `match`; ate la o ramo diz qual etapa o entrega,
//! que e mais util do que o comando nao existir.

use crate::cli::{Cli, Comando};
use crate::comandos;
use crate::erro::{Erro, Resultado};
use crate::portas::{Arquivos, Console, Discos, Entropia, Firmware, Relogio, Sistema};
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

    /// As operacoes do proprio sistema: Inicializacao Rapida (B-5), `chkdsk`
    /// (B-6) e o reinicio da etapa E7.
    pub sistema: &'a dyn Sistema,

    /// De onde sai o selo (C-11).
    ///
    /// A E5 construiu a porta e **nao** a pôs aqui, de proposito: nada em
    /// producao gerava selo, e um campo que nenhum comando lê e peso morto. A
    /// E7 e quem arma, e armar e o instante em que o job passa a existir —
    /// entao e agora que ela entra.
    pub entropia: &'a dyn Entropia,

    /// O que o usuario digita. Existe por S-2: a confirmacao por extenso e o
    /// que separa "armou" de "nao armou", e sem porta ela nao teria teste.
    pub console: &'a dyn Console,
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

        Comando::Resultado => return comandos::resultado::executar(contexto),

        // A unica operacao do ARCA que destroi dados, e a etapa E9 e quem a
        // entrega. Ela desarma (C-1), lista sem oferecer residuo (L-2, R-1),
        // confere o destino contra a propria imagem (R-2, R-7), pede o nome
        // por extenso (R-3, S-2) e so entao arma.
        Comando::Restore { nome, destino } => {
            return comandos::restore::executar(contexto, nome.as_deref(), *destino);
        }

        // V-1 lê os `MD5SUMS` aqui mesmo; `--completo` arma o boot unico que
        // so roda o `ocs-chkimg` (V-2). Os dois recusam residuo antes de
        // qualquer coisa (L-2), e o segundo desarma primeiro (C-1).
        Comando::Verify { nome, completo } => {
            return comandos::verify::executar(contexto, nome, *completo);
        }

        Comando::Prepare { .. } => ("prepare", "E10"),
    };

    Err(Erro::AindaNaoImplementado { comando, etapa })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{
        ArquivosEmMemoria, ConsoleDeMentira, DiscosDeMentira, EntropiaDeMentira, FirmwareDeMentira,
        RelogioParado, SistemaDeMentira,
    };
    use clap::Parser;

    /// As portas de um despacho sem dispositivo na mesa.
    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        sistema: SistemaDeMentira,
        entropia: EntropiaDeMentira,
        console: ConsoleDeMentira,
        registro: Registro,
    }

    impl Bancada {
        fn nova(etiqueta: &str) -> Bancada {
            Bancada {
                arquivos: ArquivosEmMemoria::novo(),
                discos: DiscosDeMentira::default(),
                firmware: FirmwareDeMentira::novo(),
                relogio: RelogioParado::em("2026-08-22T11:42:03"),
                sistema: SistemaDeMentira::novo(),
                entropia: EntropiaDeMentira::com(&[0; 8]),
                console: ConsoleDeMentira::mudo(),
                registro: Registro::em(
                    std::env::temp_dir()
                        .join(format!("arca-{etiqueta}-{}", std::process::id())),
                    Box::new(RelogioDoSistema),
                ),
            }
        }

        fn contexto(&self) -> Contexto<'_> {
            Contexto {
                dry_run: false,
                registro: &self.registro,
                firmware: &self.firmware,
                discos: &self.discos,
                arquivos: &self.arquivos,
                relogio: &self.relogio,
                sistema: &self.sistema,
                entropia: &self.entropia,
                console: &self.console,
            }
        }
    }

    impl Drop for Bancada {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.registro.caminho().parent().unwrap());
        }
    }

    #[test]
    fn cada_comando_nao_construido_nomeia_a_etapa_que_o_entrega() {
        let bancada = Bancada::nova("despacho");
        let contexto = bancada.contexto();

        // Sobrou um. O `verify` saiu desta lista na E11, e o `prepare` sai na
        // E10 — que vem **depois** dela nesta sessao, porque a E11 roda no
        // dispositivo que ja existe e a E10 precisa de um segundo, que ela
        // destroi de proposito.
        let erro = executar(&Cli::parse_from(["arca", "prepare"]), &contexto).unwrap_err();

        match erro {
            Erro::AindaNaoImplementado { comando, etapa } => {
                assert_eq!(comando, "prepare");
                assert_eq!(etapa, "E10");
            }
            outro => panic!("esperava etapa nomeada, veio {outro}"),
        }
    }

    #[test]
    fn os_comandos_ja_construidos_fazem_o_trabalho_em_vez_de_nomear_etapa() {
        // `list` e `status` desde a E1 e a E2; `backup` entrou na E6, quando
        // deixou de responder "armar e a E7" para rodar o pre-voo do §5.2, e
        // passou a armar de verdade na **E7**. O `resultado` entrou na **E8**,
        // o `restore` na **E9** e o `verify` na **E11**.
        //
        // Sem dispositivo conectado, os seis devolvem a recusa da descoberta
        // — e nunca `AindaNaoImplementado`. O `verify` esta aqui nas duas
        // formas: sem `--completo` ele so lê, e com ele arma, e as duas
        // precisam do dispositivo antes de qualquer outra coisa.
        let bancada = Bancada::nova("construidos");
        let contexto = bancada.contexto();

        for argumentos in [
            vec!["arca", "list"],
            vec!["arca", "status"],
            vec!["arca", "backup", "2026-08-22_Apps"],
            vec!["arca", "resultado"],
            vec!["arca", "restore"],
            vec!["arca", "verify", "2026-08-22_Apps"],
            vec!["arca", "verify", "2026-08-22_Apps", "--completo"],
        ] {
            let erro = executar(&Cli::parse_from(&argumentos), &contexto).unwrap_err();
            assert!(
                matches!(erro, Erro::DispositivoAusente),
                "{argumentos:?}: esperava a recusa da descoberta, veio {erro}"
            );
        }
    }
}
