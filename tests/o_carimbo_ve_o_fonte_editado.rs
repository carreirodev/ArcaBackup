//! O carimbo do `arca --version` é refeito quando muda o que ele descreve.
//!
//! O `build.rs` só roda de novo quando muda um dos caminhos que ele declara em
//! `rerun-if-changed`: declarar um só já tira do cargo o padrão de olhar o
//! pacote inteiro. Até 29/09/2026 a lista tinha o manifesto, o próprio
//! `build.rs` e dois arquivos do git. Editar um fonte não mexe em nenhum deles,
//! e a WPC-92 mediu o resultado: um `arca.exe` compilado com o `src/main.rs`
//! modificado saiu `0.1.0 (3f0b05f 2026-09-29)`, sem `arvore suja`. É esse
//! binário que o `arca prepare` copia para o `ARCABOOT`.
//!
//! O que este teste cobra é a lista, e não o comportamento: provar o
//! comportamento exigiria compilar o pacote duas vezes dentro da suíte, o que
//! o hook de pre-commit pagaria em todo commit.

use std::path::Path;

/// Os caminhos que o `build.rs` declara em `rerun-if-changed`, fora das linhas
/// de comentário.
fn o_que_o_build_rs_vigia() -> Vec<String> {
    let alvo = Path::new(env!("CARGO_MANIFEST_DIR")).join("build.rs");
    let codigo = std::fs::read_to_string(&alvo)
        .unwrap_or_else(|erro| panic!("não consegui ler {}: {erro}", alvo.display()));

    codigo
        .lines()
        .filter(|linha| !linha.trim_start().starts_with("//"))
        .filter_map(|linha| linha.split_once("rerun-if-changed="))
        .map(|(_, resto)| resto.split('"').next().unwrap_or(resto).to_string())
        .collect()
}

#[test]
fn o_carimbo_e_refeito_quando_o_que_entra_no_binario_muda() {
    // `src` é uma pasta, e o cargo confere a pasta inteira. `Cargo.toml` e
    // `Cargo.lock` decidem quais dependências entram, e uma troca de versão
    // ainda não commitada muda o binário tanto quanto um fonte editado.
    let vigiados = o_que_o_build_rs_vigia();
    for caminho in ["src", "Cargo.toml", "Cargo.lock"] {
        assert!(
            vigiados.iter().any(|vigiado| vigiado == caminho),
            "o `build.rs` não vigia `{caminho}`: uma edição ali, sem comando git no meio, \
             sai num binário que o carimbo diz ser de árvore limpa (WPC-92). Vigiados: \
             {vigiados:?}"
        );
    }
}

#[test]
fn o_carimbo_e_refeito_quando_o_commit_muda() {
    // O outro lado do carimbo: um commit que só toca documentação não mexe em
    // fonte nenhum, e sem estes dois o carimbo apontaria para o commit anterior.
    let vigiados = o_que_o_build_rs_vigia();
    for caminho in [".git/HEAD", ".git/index"] {
        assert!(
            vigiados.iter().any(|vigiado| vigiado == caminho),
            "o `build.rs` não vigia `{caminho}`, e um commit deixaria o carimbo no commit \
             anterior. Vigiados: {vigiados:?}"
        );
    }
}
