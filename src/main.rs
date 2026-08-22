//! O binario do ARCA.
//!
//! Fino de proposito: colhe os argumentos, registra a invocacao, garante a
//! elevacao e despacha. A ordem importa.
//!
//! A invocacao e registrada **antes** de o `clap` analisar qualquer coisa,
//! porque `--version` e `--help` curto-circuitam ali dentro. Sem essa
//! anotacao no registro nao haveria como provar, do lado de fora, que a linha
//! de comando atravessou a elevacao intacta — que e exatamente o que C-7
//! exige e o que uma vez falhou em silencio.

use std::process::ExitCode;

use arca::adaptadores::windows::firmware::Bcdedit;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::adaptadores::{ArquivosDoSistema, RelogioDoSistema};
use arca::app::{self, Contexto};
use arca::cli::Cli;
use arca::elevacao::{self, Rumo};
use arca::registro::Registro;
use clap::Parser;

#[cfg(windows)]
use arca::adaptadores::windows::console;
#[cfg(windows)]
use arca::adaptadores::windows::privilegios::PrivilegiosDoWindows;

/// O que sai de `executar`: o codigo de saida e se a janela ainda tem algo a
/// mostrar.
struct Desfecho {
    saida: ExitCode,
    /// Falso quando quem fez o trabalho foi o processo elevado. A janela que o
    /// usuario leu foi a dele; segurar esta mostraria uma segunda janela,
    /// vazia, pedindo Enter de novo.
    segurar_janela: bool,
}

fn main() -> ExitCode {
    let brutos: Vec<String> = std::env::args().collect();
    let desfecho = executar(&brutos);

    // A janela que o UAC abre nao e a mesma de onde o comando foi digitado:
    // sem esta pausa, a saida de `arca list` piscaria e sumiria.
    #[cfg(windows)]
    console::pausar_antes_de_fechar(desfecho.segurar_janela && Cli::pausa_pedida(&brutos));

    desfecho.saida
}

fn executar(brutos: &[String]) -> Desfecho {
    let registro = Registro::padrao(Box::new(RelogioDoSistema));

    let privilegios = privilegios();
    registro.info(format!(
        "arca {} · elevado={} · linha={:?}",
        env!("CARGO_PKG_VERSION"),
        match privilegios.elevado() {
            Ok(true) => "sim",
            Ok(false) => "nao",
            Err(_) => "indeterminado",
        },
        elevacao::argumentos_a_repassar(brutos)
    ));

    // Analisar antes de elevar: uma linha de comando errada merece a
    // mensagem do `clap` no console de onde foi digitada, sem pedir UAC.
    let cli = match Cli::try_parse_from(brutos) {
        Ok(cli) => cli,
        Err(reclamacao) => {
            let _ = reclamacao.print();
            let saida = match reclamacao.kind() {
                clap::error::ErrorKind::DisplayHelp
                | clap::error::ErrorKind::DisplayVersion
                | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
                    registro.info("linha de comando respondida pelo proprio clap");
                    ExitCode::SUCCESS
                }
                outro => {
                    // Registrar a recusa importa: uma linha de comando que
                    // chegou repartida no lugar errado aparece aqui, e nao
                    // em nenhum outro lugar.
                    registro.aviso(format!("linha de comando recusada: {outro:?}"));
                    ExitCode::from(2)
                }
            };
            return Desfecho {
                saida,
                segurar_janela: true,
            };
        }
    };

    match elevacao::garantir(privilegios.as_ref(), brutos) {
        Ok(Rumo::Seguir) => {}
        Ok(Rumo::Propagar(codigo)) => {
            registro.info(format!("processo elevado terminou com codigo {codigo}"));
            return Desfecho {
                saida: ExitCode::from(elevacao::codigo_de_saida_local(codigo)),
                segurar_janela: false,
            };
        }
        Err(falha) => {
            registro.erro(falha.to_string());
            eprintln!("erro: {falha}");
            return Desfecho {
                saida: ExitCode::from(falha.codigo_de_saida()),
                segurar_janela: true,
            };
        }
    }

    let firmware = Bcdedit;
    let discos = VolumesDoWindows;
    let arquivos = ArquivosDoSistema;
    let relogio = RelogioDoSistema;

    let contexto = Contexto {
        dry_run: cli.dry_run,
        registro: &registro,
        firmware: &firmware,
        discos: &discos,
        arquivos: &arquivos,
        relogio: &relogio,
    };

    let saida = match app::executar(&cli, &contexto) {
        Ok(()) => ExitCode::SUCCESS,
        Err(falha) => {
            registro.erro(falha.to_string());
            eprintln!("erro: {falha}");
            ExitCode::from(falha.codigo_de_saida())
        }
    };

    Desfecho {
        saida,
        segurar_janela: true,
    }
}

#[cfg(windows)]
fn privilegios() -> Box<dyn arca::portas::Privilegios> {
    Box::new(PrivilegiosDoWindows)
}

/// Fora do Windows o ARCA nao tem o que fazer, mas compilar em outra
/// plataforma mantem honesto o que e portatil e o que nao e.
#[cfg(not(windows))]
fn privilegios() -> Box<dyn arca::portas::Privilegios> {
    Box::new(arca::duplos::PrivilegiosDeMentira::elevado())
}
