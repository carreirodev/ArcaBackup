//! S-1 como propriedade verificavel da arquitetura.
//!
//! O requisito diz que o ARCA nunca abre o disco de origem em acesso raw de
//! escrita. Isso nao e uma promessa que se guarda num comentario: e uma
//! propriedade do codigo, e este teste a cobra a cada build varrendo `src/`
//! atras das formas pelas quais o acesso raw entraria.
//!
//! Chamar `powercfg` ou `chkdsk` nao e acesso raw — sao operacoes do proprio
//! Windows, pelas quais o Windows responde. Quem lê e escreve setor e o
//! Clonezilla, do outro lado do reinicio.

use std::path::{Path, PathBuf};

/// As marcas do acesso raw. Montadas em pedacos para que a varredura nao
/// encontre a si mesma quando alguem apontar o teste para o proprio arquivo.
fn marcas_proibidas() -> Vec<(String, &'static str)> {
    let barras = r"\\.\";
    vec![
        (
            format!("{barras}PhysicalDrive"),
            "abrir um disco fisico por caminho de dispositivo e exatamente o que S-1 proibe",
        ),
        (
            format!("{barras}Harddisk"),
            "caminho de dispositivo bruto: S-1",
        ),
        (
            "DeviceIo".to_string() + "Control",
            "conversa direta com o driver do disco: S-1",
        ),
        (
            "IOCTL_".to_string() + "DISK",
            "codigo de controle de disco: S-1",
        ),
        (
            "SetFilePointer".to_string() + "Ex",
            "posicionamento por deslocamento so faz sentido em acesso raw: S-1",
        ),
        (
            "FSCTL_".to_string() + "LOCK_VOLUME",
            "travar o volume para escrever por baixo do sistema de arquivos: S-1",
        ),
    ]
}

fn fontes(diretorio: &Path, encontrados: &mut Vec<PathBuf>) {
    let leitura = std::fs::read_dir(diretorio)
        .unwrap_or_else(|erro| panic!("nao consegui ler {}: {erro}", diretorio.display()));

    for entrada in leitura {
        let caminho = entrada.expect("entrada de diretorio").path();
        if caminho.is_dir() {
            fontes(&caminho, encontrados);
        } else if caminho.extension().is_some_and(|ext| ext == "rs") {
            encontrados.push(caminho);
        }
    }
}

#[test]
fn nenhuma_porta_abre_o_disco_em_modo_raw() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut arquivos = Vec::new();
    fontes(&raiz, &mut arquivos);

    assert!(!arquivos.is_empty(), "a varredura precisa achar fontes");

    let marcas = marcas_proibidas();
    let mut violacoes = Vec::new();

    for arquivo in &arquivos {
        let conteudo = std::fs::read_to_string(arquivo).expect("fonte legivel");
        for (numero, linha) in conteudo.lines().enumerate() {
            for (marca, porque) in &marcas {
                if linha.contains(marca.as_str()) {
                    violacoes.push(format!(
                        "{}:{}: `{marca}` — {porque}",
                        arquivo.display(),
                        numero + 1
                    ));
                }
            }
        }
    }

    assert!(
        violacoes.is_empty(),
        "S-1 violado:\n{}",
        violacoes.join("\n")
    );
}

#[test]
fn as_tres_fronteiras_perigosas_estao_atras_de_portas() {
    let portas = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/portas");

    for fronteira in ["firmware.rs", "discos.rs", "arquivos.rs"] {
        assert!(
            portas.join(fronteira).exists(),
            "a fronteira {fronteira} precisa de uma porta para ter teste sem hardware"
        );
    }
}
