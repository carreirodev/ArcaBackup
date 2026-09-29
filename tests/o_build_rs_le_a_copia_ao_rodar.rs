//! O `build.rs` não grava nada de quando ele próprio compilou.
//!
//! Um `env!` dentro do `build.rs` é lido quando o **script** compila, e não
//! quando ele roda. O cargo deu às pastas do script o mesmo hash em cópias do
//! repositório guardadas em lugares diferentes. Com a pasta `target/`
//! compartilhada entre duas cópias, uma roda o script que a outra compilou, e o
//! valor que o `env!` gravou é o da outra.
//!
//! A pergunta nasceu em 28/09/2026, na WPC-65. Uma worktree descartável
//! compilou na `target/` da árvore real e foi apagada. O commit seguinte da
//! árvore real reprovou no link com `LNK1327`, procurando
//! `recursos/arca.manifest` dentro da worktree que já não existia. Em
//! 29/09/2026 a WPC-82 reproduziu o `LNK1327` com duas cópias e uma `target/`
//! só, por dois caminhos. Num deles, o script compilado numa cópia roda na
//! outra e passa ao linker o manifesto da primeira; é esse que o `env!` abria.
//! No outro, o cargo nem roda o script, e este teste não o alcança. O
//! comentário do issue guarda o passo a passo dos dois.
//!
//! O que este teste cobra é o padrão, e não o comportamento: provar o
//! comportamento exigiria compilar o pacote duas vezes dentro da suíte, o que
//! o hook de pre-commit pagaria em todo commit.

use std::path::Path;

fn o_build_rs() -> String {
    let alvo = Path::new(env!("CARGO_MANIFEST_DIR")).join("build.rs");
    std::fs::read_to_string(&alvo)
        .unwrap_or_else(|erro| panic!("não consegui ler {}: {erro}", alvo.display()))
}

#[test]
fn o_build_rs_nao_le_o_ambiente_da_propria_compilacao() {
    // As linhas de comentário ficam de fora porque o próprio `build.rs` explica
    // por que não usa a macro, e a explicação precisa poder nomeá-la. O
    // `option_env!(` também cai aqui, porque termina em `env!(`.
    let codigo = o_build_rs();
    let achadas: Vec<(usize, &str)> = codigo
        .lines()
        .enumerate()
        .filter(|(_, linha)| !linha.trim_start().starts_with("//"))
        .filter(|(_, linha)| linha.contains("env!("))
        .map(|(indice, linha)| (indice + 1, linha.trim()))
        .collect();

    assert!(
        achadas.is_empty(),
        "o `build.rs` lê o ambiente de quando ele compilou, e esse valor pode ser o de outra \
         cópia do repositório que compartilhe a `target/` (WPC-82). Leia com \
         `std::env::var_os`, quando o script roda. Linhas: {achadas:?}"
    );
}

#[test]
fn o_build_rs_ainda_embute_o_manifesto() {
    // Uma varredura que não acha o que guarda passa em silêncio. O caminho que
    // importa é o do `/MANIFESTINPUT`; se ele sair do `build.rs`, este teste
    // ficou para trás.
    assert!(
        o_build_rs().contains("/MANIFESTINPUT:"),
        "o `build.rs` não passa mais o manifesto ao linker: este teste guarda um caminho que \
         saiu do lugar"
    );
}
