//! Diagnostico: a escrita atomica se comporta em FAT32 como se comporta em
//! NTFS?
//!
//! A duvida e da etapa E4, e ela e cara. `escrever_atomico` esta no contrato
//! da porta desde a E0 e **nunca rodou em producao**: o unico teste dele
//! escreve no `TEMP`, que nesta maquina e NTFS. A E4 e a primeira etapa que
//! escreve, e o primeiro arquivo que ela escreve e o `grub.cfg` do
//! `ARCABOOT` — FAT32, e o arquivo de que depende a maquina bootar.
//!
//! A escrita atomica e temporario mais renomeacao. No Windows quem renomeia e
//! o `MoveFileEx` com `MOVEFILE_REPLACE_EXISTING`, e a documentacao dele fala
//! de NTFS ao descrever a atomicidade. Em FAT32 nao ha jornal: a substituicao
//! e uma sequencia de operacoes de diretorio, e nao uma transacao. Isso muda o
//! que se pode prometer, e a promessa esta escrita no contrato da porta.
//!
//! Este exemplo mede em vez de supor. Ele **nao toca no `grub.cfg`**: escreve
//! num arquivo proprio, ao lado, e confere byte a byte.
//!
//! # O que ele mediu, em 22/08/2026, contra o `ARCABOOT` desta maquina
//!
//! | Pergunta | Resposta |
//! |---|---|
//! | Renomear por cima de arquivo existente | Funciona, sem erro |
//! | `sync_all` num arquivo FAT32 | Funciona |
//! | Conteudo byte a byte, com LF | Preservado — nada converte quebra de linha |
//! | Temporario deixado para tras | Nenhum |
//! | Nome longo (`grub.cfg.arca-tmp`) em FAT32 | Aceito |
//!
//! A conclusao **nao** e que a escrita virou transacional em FAT32. E que a
//! sequencia funciona e nao deixa resto. A janela em que um desligamento
//! deixaria o arquivo antigo no lugar continua existindo — e e por isso que a
//! E4 grava o `grub.cfg` **desarmado**, que e o estado seguro: interrompida
//! no meio, a maquina continua com o que havia antes, e o que havia antes ou
//! ja era o inerte, ou era um armado que a proxima passada desarma de novo.
//!
//! Roda sem privilegio administrativo: o `ARCABOOT` e escrivel pelo usuario.
//!
//! ```text
//! cargo run --example escrita_atomica_no_fat32
//! ```

#[cfg(windows)]
use arca::adaptadores::ArquivosDoSistema;
#[cfg(windows)]
use arca::dispositivo;
#[cfg(windows)]
use arca::portas::Arquivos;

/// Fora do Windows nao ha `ARCABOOT`, nem `GetDriveType`, nem `MoveFileEx`.
/// O `main.rs` diz por que o projeto continua compilando em outra plataforma:
/// mantem honesto o que e portatil e o que nao e — e um exemplo que quebrasse
/// o `cargo check --all-targets` la desfaria isso.
#[cfg(not(windows))]
fn main() {
    eprintln!("esta medicao e sobre o FAT32 do ARCABOOT, e so faz sentido no Windows");
}

#[cfg(windows)]
fn main() {
    let discos = arca::adaptadores::windows::volumes::VolumesDoWindows;
    let dispositivo = match dispositivo::encontrar(&discos) {
        Ok(dispositivo) => dispositivo,
        Err(erro) => {
            eprintln!("sem dispositivo para medir: {erro}");
            return;
        }
    };

    let raiz = match dispositivo.raiz_do_boot() {
        Ok(raiz) => raiz,
        Err(erro) => {
            eprintln!("sem ARCABOOT para medir: {erro}");
            return;
        }
    };

    let boot = dispositivo.boot.as_ref().expect("ha ARCABOOT");
    println!(
        "ARCABOOT em {} · {} · {} bytes livres",
        raiz.display(),
        boot.sistema_de_arquivos,
        boot.livre_bytes
    );
    if !boot.sistema_de_arquivos.eq_ignore_ascii_case("FAT32") {
        println!(
            "AVISO: este volume e {}, e a pergunta desta medicao e sobre FAT32",
            boot.sistema_de_arquivos
        );
    }

    // Ao lado do `grub.cfg`, para medir no mesmo diretorio e no mesmo volume
    // em que a E4 vai escrever — e com outro nome, porque esta medicao nao
    // tem nada a ver com o arquivo de que a maquina depende para bootar.
    let alvo = raiz.join(r"boot\grub\arca-medicao-fat32.cfg");
    println!("medindo em {}\n", alvo.display());

    // Conteudo com LF, como o `grub.cfg` de verdade: quebra de linha
    // convertida seria um arquivo diferente byte a byte.
    let primeiro = "set default=\"live-default\"\nset timeout=\"30\"\n";
    let segundo = "set default=\"arca-backup\"\nset timeout=\"30\"\nmenuentry {\n}\n";

    let arquivos = ArquivosDoSistema;

    match arquivos.escrever_atomico(&alvo, primeiro) {
        Ok(()) => println!("1. escrita num caminho novo ......... ok"),
        Err(erro) => {
            eprintln!("1. escrita num caminho novo ......... FALHOU: {erro}");
            return;
        }
    }

    conferir(
        &arquivos,
        &alvo,
        primeiro,
        "2. conteudo da primeira escrita",
    );

    // A pergunta que motivou a medicao: em FAT32, renomear por cima de um
    // arquivo que ja existe se comporta como em NTFS?
    match arquivos.escrever_atomico(&alvo, segundo) {
        Ok(()) => println!("3. renomeacao por cima do existente . ok"),
        Err(erro) => {
            eprintln!("3. renomeacao por cima do existente . FALHOU: {erro}");
            return;
        }
    }

    conferir(&arquivos, &alvo, segundo, "4. conteudo depois da segunda");

    // O temporario nao pode ficar para tras num dispositivo que a maquina
    // boota: o `grub` lê o diretorio inteiro.
    let temporario = raiz.join(r"boot\grub\arca-medicao-fat32.cfg.arca-tmp");
    println!(
        "5. temporario deixado para tras ..... {}",
        if arquivos.existe(&temporario) {
            "SOBROU — e ele fica num diretorio que o grub lê"
        } else {
            "nenhum"
        }
    );

    // Tamanho em bytes, e nao em caracteres: e o tamanho que o `grub.cfg`
    // tera, e e por ele que a E4 compara com o inerte conhecido.
    match std::fs::metadata(&alvo) {
        Ok(metadados) => println!(
            "6. tamanho em disco ................ {} bytes (escritos {})",
            metadados.len(),
            segundo.len()
        ),
        Err(erro) => eprintln!("6. tamanho em disco ................ nao deu para saber: {erro}"),
    }

    println!(
        "\nApague o arquivo de medicao a mao:\n  del {}",
        alvo.display()
    );
}

#[cfg(windows)]
fn conferir(arquivos: &ArquivosDoSistema, alvo: &std::path::Path, esperado: &str, rotulo: &str) {
    match arquivos.ler_texto(alvo) {
        Ok(lido) if lido == esperado => println!("{rotulo} ..... igual byte a byte"),
        Ok(lido) => {
            println!("{rotulo} ..... DIFERENTE\n   esperado: {esperado:?}\n   lido:     {lido:?}")
        }
        Err(erro) => println!("{rotulo} ..... nao deu para ler: {erro}"),
    }
}
