//! A superficie de linha de comando (secao 8 do PRD).
//!
//! Todos os comandos existem aqui desde a E0, e **desde a E10 todos fazem o
//! trabalho**. Ate aqui os que ainda nao tinham etapa construida respondiam
//! dizendo qual etapa os entregava — mais util do que nao existirem, e e o que
//! fez a fundacao ser executavel de verdade desde o primeiro dia.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(
    name = "arca",
    version,
    about = "Automatizador de Clonezilla para backup e restauracao de imagem de disco",
    long_about = "O ARCA nunca lê nem escreve disco. Ele prepara o ambiente, monta a receita,\n\
                  dispara o boot unico e colhe o que o Clonezilla deixou escrito.",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Imprime a receita e o que seria feito; nao arma nada.
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Nao segura a janela ao terminar. Para quem chama o ARCA de um script.
    #[arg(long, global = true, hide = true)]
    pub sem_pausa: bool,

    #[command(subcommand)]
    pub comando: Comando,
}

/// A flag que dispensa a pausa final.
///
/// E flag de linha de comando, e nao variavel de ambiente, porque o processo
/// que o UAC eleva **nao herda o ambiente de quem o chamou**: quem o cria e o
/// servico AppInfo. O que atravessa a elevacao e a linha de comando, que e
/// justamente o assunto de C-7.
pub const SEM_PAUSA: &str = "--sem-pausa";

impl Cli {
    /// Se a janela deve ser segurada ao terminar, lido dos argumentos
    /// **brutos**.
    ///
    /// Le dos brutos, e nao de um `Cli`, porque a decisao tambem vale quando o
    /// `clap` recusou a linha de comando — que e exatamente quando ha uma
    /// mensagem que o usuario precisa ler antes de a janela sumir.
    pub fn pausa_pedida(brutos: &[String]) -> bool {
        !brutos.iter().any(|argumento| argumento == SEM_PAUSA)
    }
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Comando {
    /// Particiona um disco, instala o Clonezilla e o ARCA, e cria a entrada
    /// de boot.
    Prepare {
        /// O disco a preparar, pelo **indice do Windows**. Obrigatorio.
        ///
        /// # Por que obrigatorio, mesmo havendo um candidato so
        ///
        /// P1 revisado: *o ARCA destroi dados quando o usuario nomeou o alvo e
        /// confirmou por escrito, e nunca por deducao*. Deduzir o disco seria
        /// o ARCA escolhendo o que apagar, e e exatamente isso que o principio
        /// proibe — mesmo quando a deducao pareceria obvia.
        ///
        /// **E o indice nao e identidade**, o que torna a confirmacao de S-2
        /// necessaria por cima dele: medido em 23/08/2026, o dispositivo desta
        /// mesa era o disco 1 e virou o disco 2 quando um segundo SSD foi
        /// conectado. Por isso a confirmacao pede o **modelo**, que a tela
        /// acabou de imprimir, e nao o numero que se digitou aqui.
        #[arg(long, value_name = "INDICE")]
        dispositivo: u32,

        /// Instala de um arquivo local, sem baixar nada (PR-2).
        ///
        /// E o que salva quando a maquina que precisa preparar o dispositivo e
        /// justamente a que esta sem Windows — e sem rede.
        #[arg(long, value_name = "CAMINHO")]
        iso: Option<PathBuf>,
    },

    /// Monta a receita, arma o boot unico e reinicia.
    Backup {
        /// Nome da imagem. Sem espaco, sem acento, nunca sobrescrito.
        #[arg(value_name = "NOME")]
        nome: String,
    },

    /// Lê o desfecho do job pendente e desarma o dispositivo.
    Resultado,

    /// Lista as imagens do dispositivo conectado.
    List,

    /// Lista, confirma e reinicia para restaurar.
    ///
    /// **Nao ha flag de destino**, e a ausencia e decisao. O `--destino
    /// <indice>` existiu da E9 ate 23/08/2026 e saiu com o
    /// [ADR-0015](../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md):
    /// o unico destino valido e o disco de que a imagem veio, e sem destino
    /// divergente a flag passa a ser um jeito de apontar um disco para apagar
    /// — que e o que P1 revisado proibe.
    Restore {
        /// Nome da imagem. Omitido, o comando lista e pergunta o numero — que
        /// e a tela do §6.1. Dado, ele pula a lista; a confirmacao por extenso
        /// (R-3, S-2) continua obrigatoria nos dois caminhos.
        #[arg(value_name = "NOME")]
        nome: Option<String>,
    },

    /// Confere os MD5SUMS de uma imagem, sem reiniciar.
    Verify {
        #[arg(value_name = "NOME")]
        nome: String,

        /// Arma boot unico que so roda o `ocs-chkimg`.
        #[arg(long)]
        completo: bool,
    },

    /// Descobre os discos desta maquina, sem fazer backup nem restauracao.
    ///
    /// # Por que ele nao tem argumento nenhum
    ///
    /// Os outros tres comandos que armam nomeiam uma imagem. A sondagem nao
    /// opera sobre imagem nenhuma — ela pergunta *"que discos ha nesta
    /// maquina?"* —, e ela existe justamente para o dispositivo que **ainda
    /// nao tem imagem** (§4.5, P-26). Um argumento aqui seria um valor que
    /// receita nenhuma usa.
    Sondar,

    /// Diagnostico: dispositivo, firmware, job pendente.
    Status,

    /// Devolve o dispositivo ao estado inerte: tira a receita do `grub.cfg` e
    /// limpa a marca de boot unico.
    ///
    /// Desarmar continua sendo o primeiro passo de todo comando que arma
    /// (C-1). Ele ganha um comando proprio para o caso em que o boot nao
    /// aconteceu e o dispositivo ficou armado sem nada a colher — ver o
    /// modulo [`crate::comandos::desarmar`].
    Desarmar,
}

impl Comando {
    /// O nome pelo qual o comando aparece na linha de comando e no registro.
    pub fn nome(&self) -> &'static str {
        match self {
            Comando::Prepare { .. } => "prepare",
            Comando::Backup { .. } => "backup",
            Comando::Resultado => "resultado",
            Comando::List => "list",
            Comando::Restore { .. } => "restore",
            Comando::Verify { .. } => "verify",
            Comando::Sondar => "sondar",
            Comando::Status => "status",
            Comando::Desarmar => "desarmar",
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn analisar(argumentos: &[&str]) -> Cli {
        Cli::try_parse_from(argumentos).expect("linha de comando valida")
    }

    #[test]
    fn dry_run_e_global_e_vale_antes_do_comando() {
        let cli = analisar(&["arca", "--dry-run", "backup", "2026-08-22_Apps"]);
        assert!(cli.dry_run);
        assert_eq!(
            cli.comando,
            Comando::Backup {
                nome: "2026-08-22_Apps".to_string()
            }
        );
    }

    #[test]
    fn dry_run_e_global_e_vale_depois_do_comando() {
        let cli = analisar(&["arca", "backup", "2026-08-22_Apps", "--dry-run"]);
        assert!(cli.dry_run);
    }

    #[test]
    fn sem_dry_run_a_flag_fica_desligada() {
        assert!(!analisar(&["arca", "backup", "nome"]).dry_run);
    }

    #[test]
    fn todos_os_comandos_do_prd_existem() {
        assert_eq!(
            analisar(&["arca", "prepare", "--dispositivo", "1"])
                .comando
                .nome(),
            "prepare"
        );
        assert_eq!(analisar(&["arca", "backup", "n"]).comando.nome(), "backup");
        assert_eq!(analisar(&["arca", "resultado"]).comando.nome(), "resultado");
        assert_eq!(analisar(&["arca", "list"]).comando.nome(), "list");
        assert_eq!(analisar(&["arca", "restore"]).comando.nome(), "restore");
        assert_eq!(analisar(&["arca", "verify", "n"]).comando.nome(), "verify");
        assert_eq!(analisar(&["arca", "sondar"]).comando.nome(), "sondar");
        assert_eq!(analisar(&["arca", "status"]).comando.nome(), "status");
        assert_eq!(analisar(&["arca", "desarmar"]).comando.nome(), "desarmar");
    }

    #[test]
    fn restore_sem_nome_e_o_caminho_da_lista_numerada() {
        assert_eq!(
            analisar(&["arca", "restore"]).comando,
            Comando::Restore { nome: None }
        );
    }

    #[test]
    fn restore_aceita_o_nome_e_pula_a_lista() {
        assert_eq!(
            analisar(&["arca", "restore", "2026-08-22_Apps"]).comando,
            Comando::Restore {
                nome: Some("2026-08-22_Apps".to_string()),
            }
        );
    }

    #[test]
    fn nao_ha_como_apontar_um_disco_de_destino() {
        // ADR-0015: o unico destino valido e o disco de origem, e o ARCA o
        // acha sozinho pelo modelo (§4.5). O `--destino <indice>` existiu da
        // E9 ate 23/08/2026 e saiu junto com o destino divergente — sem ele,
        // a flag seria um jeito de apontar um disco para apagar.
        //
        // O teste vale como recusa **da superficie**: um script antigo que a
        // passasse recebe erro de uso, e nao um argumento ignorado em
        // silencio.
        for tentativa in [
            vec!["arca", "restore", "--destino", "0"],
            vec!["arca", "restore", "2026-08-22_Apps", "--destino", "1"],
        ] {
            assert!(
                Cli::try_parse_from(&tentativa).is_err(),
                "{tentativa:?} devia ser recusado"
            );
        }
    }

    #[test]
    fn prepare_aceita_iso_local() {
        let cli = analisar(&[
            "arca",
            "prepare",
            "--dispositivo",
            "1",
            "--iso",
            r"D:\clonezilla.zip",
        ]);
        assert_eq!(
            cli.comando,
            Comando::Prepare {
                dispositivo: 1,
                iso: Some(PathBuf::from(r"D:\clonezilla.zip"))
            }
        );
    }

    #[test]
    fn prepare_exige_o_dispositivo() {
        // P1 revisado: *o ARCA destroi dados quando o usuario nomeou o alvo, e
        // nunca por deducao*. Um `arca prepare` sem alvo teria de escolher um
        // disco sozinho, e e exatamente isso que o principio proibe — mesmo
        // havendo um candidato so.
        assert!(Cli::try_parse_from(["arca", "prepare"]).is_err());
        assert!(Cli::try_parse_from(["arca", "prepare", "--iso", "x.zip"]).is_err());
    }

    #[test]
    fn o_dispositivo_e_indice_e_nao_letra_nem_rotulo() {
        // §7.1: `arca prepare` e o unico comando que **nao** se localiza pelos
        // rotulos, porque no disco que ele vai preparar eles ainda nao
        // existem. Aceitar `--dispositivo ARCAVAULT` seria pedir pelo caminho
        // que este comando nao tem.
        for tentativa in ["ARCAVAULT", "E:", "sda", "disco1"] {
            assert!(
                Cli::try_parse_from(["arca", "prepare", "--dispositivo", tentativa]).is_err(),
                "`{tentativa}` devia ser recusado"
            );
        }
    }

    #[test]
    fn verify_aceita_completo() {
        let cli = analisar(&["arca", "verify", "2026-08-22_Apps", "--completo"]);
        assert_eq!(
            cli.comando,
            Comando::Verify {
                nome: "2026-08-22_Apps".to_string(),
                completo: true
            }
        );
    }

    #[test]
    fn backup_exige_nome() {
        assert!(Cli::try_parse_from(["arca", "backup"]).is_err());
    }

    #[test]
    fn sondar_nao_aceita_argumento_nenhum() {
        // A sondagem nao opera sobre imagem nenhuma — ela pergunta *"que discos
        // ha nesta maquina?"* —, e ela existe justamente para o dispositivo que
        // ainda nao tem imagem. Um nome aqui seria um valor que receita nenhuma
        // usa, e a superficie o recusa em vez de ignorar em silencio.
        assert_eq!(analisar(&["arca", "sondar"]).comando, Comando::Sondar);
        assert!(Cli::try_parse_from(["arca", "sondar", "2026-08-22_Apps"]).is_err());
    }

    #[test]
    fn sondar_aceita_o_dry_run() {
        // Ele arma, entao `--dry-run` e de primeira classe nele como nos outros
        // tres: imprime a receita e nao arma nada (S-2).
        assert!(analisar(&["arca", "sondar", "--dry-run"]).dry_run);
    }

    #[test]
    fn comando_e_obrigatorio() {
        assert!(Cli::try_parse_from(["arca"]).is_err());
        assert!(Cli::try_parse_from(["arca", "--dry-run"]).is_err());
    }

    #[test]
    fn a_pausa_e_o_padrao_e_a_flag_a_dispensa() {
        let com_flag: Vec<String> = ["arca.exe", "status", SEM_PAUSA]
            .iter()
            .map(|a| a.to_string())
            .collect();
        let sem_flag: Vec<String> = ["arca.exe", "status"]
            .iter()
            .map(|a| a.to_string())
            .collect();

        assert!(!Cli::pausa_pedida(&com_flag));
        assert!(Cli::pausa_pedida(&sem_flag));
    }

    #[test]
    fn a_flag_de_pausa_e_aceita_em_qualquer_comando() {
        // Ela tem de ser aceita pelo `clap` tambem, senao a linha inteira e
        // recusada e o comando nao roda.
        assert!(analisar(&["arca", "status", SEM_PAUSA]).sem_pausa);
        assert!(analisar(&["arca", SEM_PAUSA, "backup", "n"]).sem_pausa);
        assert!(!analisar(&["arca", "status"]).sem_pausa);
    }

    #[test]
    fn version_curto_circuita_sem_comando() {
        let erro = Cli::try_parse_from(["arca", "--version"]).unwrap_err();
        assert_eq!(erro.kind(), clap::error::ErrorKind::DisplayVersion);
    }
}
