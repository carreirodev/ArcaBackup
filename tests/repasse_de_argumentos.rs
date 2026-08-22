//! C-7 e C-8, verificados contra o Windows de verdade.
//!
//! Nao ha como testar o clique no UAC. O que da para testar — e o que
//! realmente falhou uma vez — e a fronteira em que o vetor de argumentos vira
//! uma string e volta a ser um vetor. Aqui isso e feito duas vezes: pelo
//! `CommandLineToArgvW`, que e o parser que o Windows usa, e por um processo
//! de verdade, que atravessa o sistema inteiro e imprime o que recebeu.

#![cfg(windows)]

use arca::adaptadores::windows::linha_de_comando::{montar_linha, montar_parametros};
use arca::cli::{Cli, Comando};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

/// As linhas de comando que o ARCA precisa atravessar sem perder nada.
/// `--dry-run` aparece em todas as posicoes possiveis de proposito: foi
/// perdendo essa flag que um ensaio virou execucao real.
fn casos() -> Vec<Vec<String>> {
    let cru = vec![
        vec!["--version"],
        vec!["--dry-run", "backup", "2026-08-22_Apps"],
        vec!["backup", "2026-08-22_Apps", "--dry-run"],
        vec!["backup", "nome com espaco"],
        vec!["verify", "2026-08-22_Apps", "--completo"],
        vec!["prepare", "--iso", r"D:\imagens\clonezilla-live.zip"],
        vec!["prepare", "--iso", r"C:\Program Files\arca\clonezilla.zip"],
        vec!["backup", r#"aspa"no"meio"#],
        vec!["backup", r"termina em barra\"],
        vec!["backup", r#"barra e aspa \" juntas"#],
        vec!["backup", "acentuado-restauração-ção"],
        vec!["backup", ""],
        vec!["status"],
    ];
    cru.into_iter()
        .map(|caso| caso.into_iter().map(String::from).collect())
        .collect()
}

/// O parser do Windows, chamado de verdade. E ele quem reparte a linha do
/// outro lado da elevacao, e por isso e ele quem julga se o escape esta certo.
fn repartir_como_o_windows(linha: &str) -> Vec<String> {
    use windows_sys::Win32::Foundation::{HLOCAL, LocalFree};
    use windows_sys::Win32::UI::Shell::CommandLineToArgvW;

    let largo: Vec<u16> = linha.encode_utf16().chain(std::iter::once(0)).collect();
    let mut quantos: i32 = 0;

    // SEGURANCA: `largo` termina em NUL e vive ate o fim da funcao; o vetor
    // devolvido e copiado para `String` antes de ser liberado.
    unsafe {
        let vetor = CommandLineToArgvW(largo.as_ptr(), &mut quantos);
        assert!(!vetor.is_null(), "o Windows recusou a linha: {linha}");

        let mut argumentos = Vec::with_capacity(quantos as usize);
        for indice in 0..quantos as usize {
            let ponteiro = *vetor.add(indice);
            let mut comprimento = 0usize;
            while *ponteiro.add(comprimento) != 0 {
                comprimento += 1;
            }
            argumentos.push(String::from_utf16_lossy(std::slice::from_raw_parts(
                ponteiro,
                comprimento,
            )));
        }

        LocalFree(vetor as HLOCAL);
        argumentos
    }
}

#[test]
fn o_windows_devolve_cada_argumento_identico() {
    for original in casos() {
        let linha = montar_linha("arca.exe", &original);
        let repartido = repartir_como_o_windows(&linha);

        assert_eq!(
            repartido.first().map(String::as_str),
            Some("arca.exe"),
            "linha: {linha}"
        );
        assert_eq!(
            &repartido[1..],
            original.as_slice(),
            "a linha `{linha}` nao devolveu o que entrou"
        );
    }
}

#[test]
fn o_escape_nunca_usa_crase() {
    for original in casos() {
        let linha = montar_linha("arca.exe", &original);
        assert!(
            !linha.contains('`'),
            "C-8: quem reparte a linha e o parser do Windows, e ele nao entende crase — {linha}"
        );
    }
}

#[test]
fn o_clap_entende_o_mesmo_dos_dois_lados() {
    // O que importa nao e so o vetor voltar igual: e o ARCA do outro lado
    // decidir a mesma coisa. Sobretudo sobre `--dry-run`.
    for original in casos() {
        let Ok(antes) =
            Cli::try_parse_from(std::iter::once("arca".to_string()).chain(original.clone()))
        else {
            continue; // `--version` e afins nao produzem um `Cli`.
        };

        let linha = montar_linha("arca.exe", &original);
        let depois = Cli::try_parse_from(repartir_como_o_windows(&linha))
            .expect("o que era valido antes da elevacao continua valido depois");

        assert_eq!(antes, depois, "linha: {linha}");
    }
}

#[test]
fn dry_run_sobrevive_a_travessia() {
    let original: Vec<String> = ["backup", "2026-08-22_Apps", "--dry-run"]
        .iter()
        .map(|a| a.to_string())
        .collect();

    let linha = montar_linha("arca.exe", &original);
    let cli = Cli::try_parse_from(repartir_como_o_windows(&linha)).unwrap();

    assert!(
        cli.dry_run,
        "perder o --dry-run aqui e o ensaio virar execucao real"
    );
    assert_eq!(
        cli.comando,
        Comando::Backup {
            nome: "2026-08-22_Apps".to_string()
        }
    );
}

/// O caminho do `eco_argumentos`, ao lado do executavel deste teste.
fn eco() -> PathBuf {
    let mut caminho = std::env::current_exe().expect("caminho do executavel de teste");
    caminho.pop(); // deps
    caminho.pop(); // debug ou release
    caminho.push("examples");
    caminho.push("eco_argumentos.exe");
    assert!(
        caminho.exists(),
        "o exemplo `eco_argumentos` precisa estar construido: cargo build --example eco_argumentos"
    );
    caminho
}

#[test]
fn um_processo_de_verdade_recebe_os_argumentos_intactos() {
    use std::os::windows::process::CommandExt;

    for original in casos() {
        // `raw_arg` entrega a linha ao Windows sem que o `std` a reescreva:
        // e o nosso escape que esta sendo julgado, nao o dele.
        let saida = Command::new(eco())
            .raw_arg(montar_parametros(&original))
            .output()
            .expect("o eco roda sem elevacao");

        assert!(saida.status.success(), "o eco falhou para {original:?}");

        let recebidos: Vec<String> = String::from_utf8_lossy(&saida.stdout)
            .lines()
            .map(|linha| linha.to_string())
            .collect();

        // Um argumento vazio nao produz linha distinguivel na saida do eco;
        // o `CommandLineToArgvW` acima ja cobre esse caso.
        let esperados: Vec<String> = original.iter().filter(|a| !a.is_empty()).cloned().collect();
        let recebidos: Vec<String> = recebidos.into_iter().filter(|a| !a.is_empty()).collect();

        assert_eq!(recebidos, esperados, "para {original:?}");
    }
}
