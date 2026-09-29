//! O que só o build sabe fazer: o manifesto de elevação, o ícone e o carimbo de
//! qual commit este binário é.
//!
//! # O manifesto `requireAdministrator` (PRD 10.4)
//!
//! Com o manifesto no executavel, e o proprio Windows quem eleva e quem
//! repassa a linha de comando — o caminho mais curto para C-7, porque nao
//! passa por nenhuma serializacao nossa. A reelevacao explicita do modulo
//! `adaptadores::windows::privilegios` continua existindo para o caso de o
//! binario rodar sem o manifesto em vigor.
//!
//! # O ícone (29/09/2026, WPC-89)
//!
//! O linker só aceita um ícone dentro de um recurso compilado, o `.res`. O
//! caminho de sempre é o `rc.exe` do Windows SDK, chamado por um crate que o
//! procure (`winresource`, `embed-resource`), porque ele não está no `PATH`
//! fora do prompt de desenvolvedor. Ficou de fora pela razão do ADR-0006: um
//! `.res` que só leva ícone são dois tipos de registro, e escrevê-lo aqui custa
//! menos do que uma dependência de build e a caça a um executável do SDK.
//!
//! Ao contrário do carimbo, o ícone derruba o build quando falta ou vem
//! quebrado. O `docs/Icon.ico` é versionado como o manifesto; se ele some, o
//! erro é do repositório, e um aviso deixaria sair um `arca.exe` sem ícone que
//! ninguém lê.
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
    println!("cargo:rerun-if-changed=docs/Icon.ico");
    println!("cargo:rerun-if-changed=build.rs");

    // O carimbo vem antes do manifesto porque o manifesto sai cedo fora do
    // Windows, e um binário sem carimbo é pior do que um binário sem manifesto:
    // o segundo falha na cara de quem roda, o primeiro mente calado.
    carimbar_a_versao();
    embutir_o_manifesto();
    embutir_o_icone();
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

// -------------------------------------------------------------------- ícone

fn embutir_o_icone() {
    let alvo = std::env::var("TARGET").unwrap_or_default();
    if !alvo.contains("windows-msvc") {
        return;
    }

    // Os dois caminhos vêm de quando o script roda, pela mesma razão do
    // manifesto (WPC-82).
    let raiz = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("o cargo define CARGO_MANIFEST_DIR para todo script de build que roda");
    let saida = std::env::var_os("OUT_DIR")
        .expect("o cargo define OUT_DIR para todo script de build que roda");

    let icone = std::path::Path::new(&raiz).join("docs/Icon.ico");
    let ico = std::fs::read(&icone)
        .unwrap_or_else(|erro| panic!("não consegui ler {}: {erro}", icone.display()));
    let res = montar_o_res(&ico)
        .unwrap_or_else(|motivo| panic!("{} não serve de ícone: {motivo}", icone.display()));

    let destino = std::path::Path::new(&saida).join("icone.res");
    std::fs::write(&destino, res)
        .unwrap_or_else(|erro| panic!("não consegui escrever {}: {erro}", destino.display()));

    // O `link.exe` aceita um `.res` como entrada e o converte sozinho.
    println!("cargo:rustc-link-arg-bin=arca={}", destino.display());
}

/// O `.res` que o `rc.exe` compilaria de `1 ICON "docs/Icon.ico"`: um
/// `RT_ICON` por imagem do `.ico`, com ids de 1 em diante, e um
/// `RT_GROUP_ICON` de id 1 que os lista. O grupo é o que o Explorer mostra.
fn montar_o_res(ico: &[u8]) -> Result<Vec<u8>, String> {
    const RT_ICON: u16 = 3;
    const RT_GROUP_ICON: u16 = 14;

    let u16_em = |i: usize| ico.get(i..i + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_em = |i: usize| {
        ico.get(i..i + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };

    if u16_em(0) != Some(0) || u16_em(2) != Some(1) {
        return Err("o cabeçalho não é o de um .ico".into());
    }
    let quantas = u16_em(4)
        .filter(|&n| n > 0)
        .ok_or("o .ico não traz imagem nenhuma")?;

    let mut res = Vec::new();
    // Todo `.res` de 32 bits abre com um registro vazio: é por ele que o
    // linker o distingue do formato de 16 bits.
    anexar_registro(&mut res, 0, 0, &[]);

    let mut grupo = Vec::new();
    grupo.extend_from_slice(&ico[..6]);
    for id in 1..=quantas {
        let entrada = 6 + 16 * usize::from(id - 1);
        let (Some(tamanho), Some(inicio)) = (u32_em(entrada + 8), u32_em(entrada + 12)) else {
            return Err(format!("a entrada {id} passa do fim do arquivo"));
        };
        let imagem = usize::try_from(inicio)
            .ok()
            .zip(usize::try_from(tamanho).ok())
            .and_then(|(inicio, tamanho)| ico.get(inicio..inicio.checked_add(tamanho)?))
            .ok_or_else(|| format!("a imagem {id} passa do fim do arquivo"))?;
        anexar_registro(&mut res, RT_ICON, id, imagem);

        // Largura, altura, cores, reservado, planos e bits por pixel vão como
        // estão no `.ico`. Copiar em vez de recalcular preserva o 0 que quer
        // dizer 256 na imagem grande.
        grupo.extend_from_slice(&ico[entrada..entrada + 8]);
        grupo.extend_from_slice(&tamanho.to_le_bytes());
        grupo.extend_from_slice(&id.to_le_bytes());
    }
    anexar_registro(&mut res, RT_GROUP_ICON, 1, &grupo);

    Ok(res)
}

/// Um registro do `.res`: cabeçalho de 32 bytes, com tipo e nome numéricos, e
/// os dados completados até múltiplo de 4.
fn anexar_registro(res: &mut Vec<u8>, tipo: u16, id: u16, dados: &[u8]) {
    let tamanho = u32::try_from(dados.len()).expect("um recurso de ícone cabe em 4 GB");
    res.extend_from_slice(&tamanho.to_le_bytes());
    res.extend_from_slice(&32u32.to_le_bytes());
    res.extend_from_slice(&[0xFF, 0xFF]);
    res.extend_from_slice(&tipo.to_le_bytes());
    res.extend_from_slice(&[0xFF, 0xFF]);
    res.extend_from_slice(&id.to_le_bytes());
    // Versão dos dados, flags de memória, idioma, versão e características. O
    // Win32 ignora as flags, e o idioma neutro faz o Windows achar o ícone em
    // qualquer idioma de interface.
    res.extend_from_slice(&[0; 16]);
    res.extend_from_slice(dados);
    res.resize(res.len().next_multiple_of(4), 0);
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
