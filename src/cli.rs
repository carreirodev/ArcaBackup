//! A superficie de linha de comando (secao 8 do PRD).
//!
//! Todos os comandos ja existem aqui. Os que ainda nao tem etapa construida
//! respondem dizendo qual etapa os entrega — mais util do que nao existirem,
//! e e o que faz a fundacao ser executavel de verdade.

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
    /// Instala o Clonezilla e o ARCA num dispositivo ja particionado.
    Prepare {
        /// Instala de um arquivo local, sem baixar nada.
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
    Restore,

    /// Confere os MD5SUMS de uma imagem, sem reiniciar.
    Verify {
        #[arg(value_name = "NOME")]
        nome: String,

        /// Arma boot unico que so roda o `ocs-chkimg`.
        #[arg(long)]
        completo: bool,
    },

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
            Comando::Restore => "restore",
            Comando::Verify { .. } => "verify",
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
        assert_eq!(analisar(&["arca", "prepare"]).comando.nome(), "prepare");
        assert_eq!(analisar(&["arca", "backup", "n"]).comando.nome(), "backup");
        assert_eq!(analisar(&["arca", "resultado"]).comando.nome(), "resultado");
        assert_eq!(analisar(&["arca", "list"]).comando.nome(), "list");
        assert_eq!(analisar(&["arca", "restore"]).comando.nome(), "restore");
        assert_eq!(analisar(&["arca", "verify", "n"]).comando.nome(), "verify");
        assert_eq!(analisar(&["arca", "status"]).comando.nome(), "status");
        assert_eq!(analisar(&["arca", "desarmar"]).comando.nome(), "desarmar");
    }

    #[test]
    fn prepare_aceita_iso_local() {
        let cli = analisar(&["arca", "prepare", "--iso", r"D:\clonezilla.zip"]);
        assert_eq!(
            cli.comando,
            Comando::Prepare {
                iso: Some(PathBuf::from(r"D:\clonezilla.zip"))
            }
        );
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
