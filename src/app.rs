//! O despacho dos comandos.
//!
//! O contexto carrega as portas e a decisao de `--dry-run`. Cada etapa do
//! plano preenche um ramo do `match`; ate la o ramo diz qual etapa o entrega,
//! que e mais util do que o comando nao existir.

use crate::cli::{Cli, Comando};
use crate::comandos;
use crate::erro::Resultado;
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

    /// Quem apaga a tabela de particao e cria as duas do dispositivo (PR-5).
    ///
    /// A quinta porta, e a unica cuja operacao **destroi um disco**. Ela entra
    /// na E10, quando P1 foi revisado e o ARCA passou a particionar — e entra
    /// aqui, no `Contexto`, porque so um comando a usa e ele precisa dela.
    ///
    /// Nada mais do ARCA a alcanca: `arca backup` recebe o mesmo `Contexto` e
    /// nao tem o que fazer com ela, e o `DiscoFisico` que ele lê continua vindo
    /// de [`Discos`], que so lê.
    pub particionador: &'a dyn crate::portas::Particionador,
}

pub fn executar(cli: &Cli, contexto: &Contexto) -> Resultado<()> {
    contexto.registro.info(format!(
        "comando `{}`{}",
        cli.comando.nome(),
        if contexto.dry_run { " (ensaio)" } else { "" }
    ));

    match &cli.comando {
        Comando::List => comandos::list::executar(contexto),
        Comando::Status => comandos::status::executar(contexto),

        // C-1: desarmar acontece incondicionalmente e sem consultar estado
        // nenhum. Continua sendo o primeiro passo dos comandos que armam — a
        // E7 e a E8 o chamam de dentro; aqui ele tambem e alcancavel sozinho,
        // que e o que responde ao caso "o boot nao aconteceu" do §5.5.
        Comando::Desarmar => comandos::desarmar::executar(contexto),

        // Com `--dry-run` o backup ja monta e imprime as receitas (E3); sem
        // ele, quem arma e a E7 — e e o proprio comando que diz isso, porque
        // o nome ainda precisa ser julgado por B-2 antes de qualquer resposta.
        Comando::Backup { nome } => comandos::backup::executar(contexto, nome),

        Comando::Resultado => comandos::resultado::executar(contexto),

        // A operacao do ARCA que destroi o disco de sistema, e a etapa E9 e
        // quem a entrega. Ela desarma (C-1), lista sem oferecer residuo (L-2,
        // R-1), confere o destino contra a propria imagem (R-2, R-7), pede o
        // nome por extenso (R-3, S-2) e so entao arma. Nao ha destino a
        // escolher desde o ADR-0015 — o unico valido e o disco de origem.
        Comando::Restore { nome } => comandos::restore::executar(contexto, nome.as_deref()),

        // V-1 lê os `MD5SUMS` aqui mesmo; `--completo` arma o boot unico que
        // so roda o `ocs-chkimg` (V-2). Os dois recusam residuo antes de
        // qualquer coisa (L-2), e o segundo desarma primeiro (C-1).
        Comando::Verify { nome, completo } => comandos::verify::executar(contexto, nome, *completo),

        // A quarta operacao, e a unica que nao chama programa nenhum do
        // Clonezilla: ela arma um boot unico que roda `lsblk`, grava a saida
        // no `ARCAVAULT` e desliga. E o que da ao §4.5 um oraculo num
        // dispositivo que nunca teve imagem (E12, P-26).
        Comando::Sondar => comandos::sondar::executar(contexto),

        // O comando que transforma um disco qualquer num dispositivo ARCA, e
        // o unico que roda antes de existirem os rotulos de que todos os
        // outros dependem (B-1, S-3, C-10). Ele julga o disco pelas sete
        // defesas de PR-5, imprime o plano, pergunta, **relê o disco**, pede a
        // confirmacao digitada e so entao apaga.
        //
        // Sem `--dispositivo` ele lista os discos e pergunta o numero, como o
        // `restore` sem nome (ADR-0024). O numero resolve para indice e cai
        // neste mesmo caminho — nada abaixo dele muda.
        Comando::Prepare { dispositivo, iso } => {
            comandos::prepare::executar(contexto, *dispositivo, iso.as_deref())
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::Erro;
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
        particionador: crate::duplos::ParticionadorDeMentira,
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
                particionador: crate::duplos::ParticionadorDeMentira::com_discos(Vec::new()),
                registro: Registro::em(
                    std::env::temp_dir().join(format!("arca-{etiqueta}-{}", std::process::id())),
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
                particionador: &self.particionador,
            }
        }
    }

    impl Drop for Bancada {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.registro.caminho().parent().unwrap());
        }
    }

    #[test]
    fn nao_sobrou_comando_por_construir() {
        // **Esta lista esvaziou na E10.** Ate aqui ela guardava os comandos
        // que a superficie do `clap` aceitava e nenhuma etapa tinha entregue —
        // e o `prepare` era o ultimo. `Erro::AindaNaoImplementado` continua no
        // codigo para o proximo comando que nascer antes da etapa dele.
        //
        // O que este teste cobra e o §8 do PRD inteiro: todo comando de la
        // **faz** alguma coisa. Um comando novo no `cli.rs` sem ramo no
        // `match` nem compila, entao a cobranca util e a de baixo — nenhum
        // deles responde `AindaNaoImplementado`.
        let bancada = Bancada::nova("despacho");
        let contexto = bancada.contexto();

        for argumentos in [
            vec!["arca", "prepare", "--dispositivo", "9"],
            vec!["arca", "list"],
            vec!["arca", "status"],
            vec!["arca", "backup", "2026-08-22_Apps"],
            vec!["arca", "resultado"],
            vec!["arca", "restore"],
            vec!["arca", "verify", "2026-08-22_Apps"],
            vec!["arca", "sondar"],
            vec!["arca", "desarmar"],
        ] {
            let saida = executar(&Cli::parse_from(&argumentos), &contexto);
            assert!(
                !matches!(saida, Err(Erro::AindaNaoImplementado { .. })),
                "{argumentos:?} ainda diz que nao foi construido"
            );
        }
    }

    #[test]
    fn os_comandos_do_dispositivo_recusam_pela_descoberta() {
        // `list` e `status` desde a E1 e a E2; `backup` entrou na E6, quando
        // deixou de responder "armar e a E7" para rodar o pre-voo do §5.2, e
        // passou a armar de verdade na **E7**. O `resultado` entrou na **E8**,
        // o `restore` na **E9**, o `verify` na **E11** e o `sondar` na **E12**.
        //
        // Sem dispositivo conectado, os oito devolvem a recusa da descoberta.
        // O `verify` esta aqui nas duas formas: sem `--completo` ele so lê, e
        // com ele arma, e as duas precisam do dispositivo antes de qualquer
        // outra coisa.
        //
        // **E o `sondar` esta aqui apesar de nao precisar de imagem nenhuma**,
        // que e a razao de ele existir: o que ele nao precisa e de imagem, e
        // nao de dispositivo — a receita dele e gravada no `grub.cfg` do
        // `ARCABOOT`, e a saida do `lsblk` vai para o `ARCAVAULT`.
        //
        // **O `prepare` nao esta nesta lista, e a ausencia e o ponto**: ele e o
        // unico comando que nao se localiza pelos rotulos, porque no disco que
        // ele vai preparar eles ainda nao existem (§7.1). Ver o teste abaixo.
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
            vec!["arca", "sondar"],
        ] {
            let erro = executar(&Cli::parse_from(&argumentos), &contexto).unwrap_err();
            assert!(
                matches!(erro, Erro::DispositivoAusente),
                "{argumentos:?}: esperava a recusa da descoberta, veio {erro}"
            );
        }
    }

    #[test]
    fn o_prepare_nao_procura_dispositivo_nenhum() {
        // §7.1: `arca prepare` e o unico comando do ARCA que **nao consegue se
        // localizar pelos rotulos**, porque no disco que ele vai preparar o
        // `ARCAVAULT` e o `ARCABOOT` ainda nao existem. Ele roda num mundo em
        // que B-1, S-3 e C-10 nao tem o que fazer.
        //
        // Numa bancada sem dispositivo nenhum, ele recusa pelo **disco**, e
        // nao pela descoberta — o que prova que ele nem tentou.
        let bancada = Bancada::nova("prepare");
        let contexto = bancada.contexto();

        let erro = executar(
            &Cli::parse_from(["arca", "prepare", "--dispositivo", "9"]),
            &contexto,
        )
        .unwrap_err();

        assert!(
            matches!(erro, Erro::PreparacaoRecusada(_)),
            "esperava a recusa do disco, veio {erro}"
        );
        assert!(
            !matches!(erro, Erro::DispositivoAusente),
            "o prepare foi procurar dispositivo pelos rotulos"
        );
    }
}
