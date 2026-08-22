//! Embute o manifesto `requireAdministrator` no binario (PRD 10.4).
//!
//! Com o manifesto no executavel, e o proprio Windows quem eleva e quem
//! repassa a linha de comando — o caminho mais curto para C-7, porque nao
//! passa por nenhuma serializacao nossa. A reelevacao explicita do modulo
//! `adaptadores::windows::privilegios` continua existindo para o caso de o
//! binario rodar sem o manifesto em vigor.

fn main() {
    println!("cargo:rerun-if-changed=recursos/arca.manifest");
    println!("cargo:rerun-if-changed=build.rs");

    let alvo = std::env::var("TARGET").unwrap_or_default();
    if !alvo.contains("windows-msvc") {
        return;
    }

    let manifesto = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("recursos/arca.manifest");
    println!("cargo:rustc-link-arg-bin=arca=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bin=arca=/MANIFESTINPUT:{}",
        manifesto.display()
    );
    println!(
        "cargo:rustc-link-arg-bin=arca=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'"
    );
}
