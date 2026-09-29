//! Duas coisas que só o build sabe fazer: o manifesto de elevação e o carimbo
//! de qual commit este binário é.
//!
//! # O manifesto `requireAdministrator` (PRD 10.4)
//!
//! Com o manifesto no executavel, e o proprio Windows quem eleva e quem
//! repassa a linha de comando — o caminho mais curto para C-7, porque nao
//! passa por nenhuma serializacao nossa. A reelevacao explicita do modulo
//! `adaptadores::windows::privilegios` continua existindo para o caso de o
//! binario rodar sem o manifesto em vigor.
//!
//! # O carimbo do commit (24/08/2026)
//!
//! O `arca.exe` mora em dois lugares — o `target\release\` do `C:` e o
//! `arca\arca.exe` do `ARCABOOT` —, e §4.1 quer exatamente isso: quem julga uma
//! restauração não pode morar no disco que ela substitui. A consequência é que
//! **um dispositivo preparado hoje carrega o ARCA de hoje**, e continua
//! carregando depois de o ARCA mudar (ver `comandos::prepare::instalar_o_arca`).
//!
//! Até 24/08/2026 não havia como perguntar a um `arca.exe` de que versão ele
//! era: `--version` respondia o `0.1.0` do `Cargo.toml`, igual em todo build.
//! Descobrir que o binário do `ARCABOOT` estava três consertos atrás exigiu
//! procurar strings dentro do executável — o que ninguém vai fazer na hora em
//! que importa, que é na frente de um disco apagado.
//!
//! O carimbo traz o commit, a data dele, e **se a árvore de trabalho estava
//! limpa**. Um binário compilado de árvore suja não corresponde a commit nenhum:
//! o hash mente por omissão, porque diz de onde o código *partiu* e não o que
//! ele *é*. Dizer `arvore suja` é a mesma escolha de `NaoSeSabe` em C-14 —
//! deixar de afirmar em vez de afirmar o tranquilizador.
//!
//! Sem `git` na máquina, ou fora de um clone, o carimbo diz `sem git` e a
//! compilação segue. O carimbo nunca falha um build: um carimbo ausente é
//! informação a menos, e derrubar a compilação por causa dele seria pior do que
//! o problema que ele resolve.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=recursos/arca.manifest");
    println!("cargo:rerun-if-changed=build.rs");

    // O carimbo vem antes do manifesto porque o manifesto sai cedo fora do
    // Windows, e um binário sem carimbo é pior do que um binário sem manifesto:
    // o segundo falha na cara de quem roda, o primeiro mente calado.
    carimbar_a_versao();
    embutir_o_manifesto();
}

// ---------------------------------------------------------------- manifesto

fn embutir_o_manifesto() {
    let alvo = std::env::var("TARGET").unwrap_or_default();
    if !alvo.contains("windows-msvc") {
        return;
    }

    // O caminho vem de quando o script roda, e não de quando ele compilou. O
    // cargo deu às pastas do script o mesmo hash em cópias do repositório
    // guardadas em lugares diferentes, e com a `target/` compartilhada uma
    // cópia roda o script que outra compilou. Com `env!` aqui, o linker recebia
    // o manifesto dessa outra cópia (medido em 29/09/2026, na WPC-82). A
    // pergunta nasceu em 28/09/2026, quando uma worktree já apagada fez o link
    // reprovar com `LNK1327`. Isto não cobre o caso em que o cargo nem roda o
    // script e reaproveita a saída que a outra cópia deixou: ali nenhum código
    // deste arquivo roda, e o `LNK1327` pode voltar.
    let raiz = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("o cargo define CARGO_MANIFEST_DIR para todo script de build que roda");
    let manifesto = std::path::Path::new(&raiz).join("recursos/arca.manifest");
    println!("cargo:rustc-link-arg-bin=arca=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-bin=arca=/MANIFESTINPUT:{}",
        manifesto.display()
    );
    println!(
        "cargo:rustc-link-arg-bin=arca=/MANIFESTUAC:level='requireAdministrator' uiAccess='false'"
    );
}

// ------------------------------------------------------------------ carimbo

fn carimbar_a_versao() {
    // Um `git commit` que não toca fonte nenhum — o caso comum de commitar
    // documentação — deixaria o carimbo apontando para o commit anterior.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");

    // E uma edição que ainda não passou pelo git deixaria o carimbo dizendo
    // árvore limpa. Até 29/09/2026 este comentário supunha que o cargo já
    // rodava o script quando um fonte mudava, e ele não rodava: qualquer
    // `rerun-if-changed` desliga o padrão de olhar o pacote inteiro, e o `main`
    // declara dois antes de chegar aqui. A WPC-92 mediu um `arca.exe` com o
    // `src/main.rs` editado saindo com carimbo de árvore limpa. `src` é uma
    // pasta, e o cargo confere a pasta inteira; `Cargo.toml` e `Cargo.lock`
    // decidem quais dependências entram no binário.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let pacote = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "?".into());

    println!("cargo:rustc-env=ARCA_VERSAO={}", montar(&pacote));
}

/// `0.1.0 (cd38384 2026-08-24)`, ou com `, arvore suja` quando for o caso.
fn montar(pacote: &str) -> String {
    let Some(commit) = git(&["rev-parse", "--short", "HEAD"]) else {
        return format!("{pacote} (sem git)");
    };

    let data = git(&["log", "-1", "--format=%cd", "--date=short"]).unwrap_or_else(|| "?".into());

    // `--porcelain` sai vazio quando não há nada a commitar. Arquivos não
    // rastreados contam: um fonte novo que ainda não entrou no `git` muda o que
    // o binário faz tanto quanto um fonte editado.
    let sujo = match git(&["status", "--porcelain"]) {
        Some(saida) => !saida.is_empty(),
        // Se o `status` não respondeu mas o `rev-parse` respondeu, não dá para
        // saber — e não saber é o caso em que este projeto não afirma.
        None => return format!("{pacote} ({commit} {data}, arvore desconhecida)"),
    };

    if sujo {
        format!("{pacote} ({commit} {data}, arvore suja)")
    } else {
        format!("{pacote} ({commit} {data})")
    }
}

/// O `git` com estes argumentos, ou `None` se ele não existir, não for um clone,
/// ou sair com erro.
fn git(args: &[&str]) -> Option<String> {
    let saida = Command::new("git").args(args).output().ok()?;
    if !saida.status.success() {
        return None;
    }
    Some(String::from_utf8(saida.stdout).ok()?.trim().to_string())
}
