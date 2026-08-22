//! B-10 como propriedade verificavel da arquitetura.
//!
//! "Nunca apagar nada" nao e uma promessa guardada num comentario: e uma
//! propriedade do codigo, e este teste a cobra a cada build.
//!
//! O ARCA lê o dispositivo e escreve receita, estado e log. Nunca remove uma
//! imagem, nunca remove um residuo, nunca remove um `arca-fim.txt` — nem
//! quando parece obvio que aquilo e lixo. Um residuo se apaga a mao, depois
//! de olhar, porque so quem olhou sabe se aqueles 36 GB sao rastro de um
//! backup interrompido ou a unica copia que sobrou de alguma coisa.

use std::path::{Path, PathBuf};

/// As formas pelas quais uma exclusao entraria no codigo. Montadas em
/// pedacos para que a varredura nao encontre a si mesma.
fn marcas_de_exclusao() -> Vec<String> {
    vec![
        "remove_".to_string() + "file",
        "remove_".to_string() + "dir",
        "DeleteFile".to_string() + "W",
        "RemoveDirectory".to_string() + "W",
    ]
}

/// O que uma linha precisa mencionar para que uma exclusao seja aceita ali.
///
/// A unica exclusao legitima do ARCA e a do proprio arquivo temporario da
/// escrita atomica, quando a renomeacao falhou: aquele arquivo foi criado
/// naquela funcao, nao existia antes e nao interessa a ninguem. Qualquer
/// outra exclusao e violacao de B-10, e nao ha excecao a acrescentar aqui
/// sem que ela apareca no diff.
const UNICA_EXCLUSAO_LEGITIMA: &str = "temporario";

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

/// O codigo do arquivo sem os seus testes.
///
/// A convencao deste repositorio poe o `#[cfg(test)] mod testes` no fim de
/// cada arquivo, e teste que limpa o proprio diretorio temporario nao e o que
/// B-10 governa.
fn sem_os_testes(conteudo: &str) -> &str {
    match conteudo.find("#[cfg(test)]") {
        Some(inicio) => &conteudo[..inicio],
        None => conteudo,
    }
}

#[test]
fn nada_no_codigo_apaga_arquivo_ou_diretorio() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut arquivos = Vec::new();
    fontes(&raiz, &mut arquivos);

    assert!(!arquivos.is_empty(), "a varredura precisa achar fontes");

    let marcas = marcas_de_exclusao();
    let mut violacoes = Vec::new();

    for arquivo in &arquivos {
        let conteudo = std::fs::read_to_string(arquivo).expect("fonte legivel");

        for (numero, linha) in sem_os_testes(&conteudo).lines().enumerate() {
            let apaga = marcas.iter().any(|marca| linha.contains(marca.as_str()));
            if apaga && !linha.contains(UNICA_EXCLUSAO_LEGITIMA) {
                violacoes.push(format!(
                    "{}:{}: {}",
                    arquivo.display(),
                    numero + 1,
                    linha.trim()
                ));
            }
        }
    }

    assert!(
        violacoes.is_empty(),
        "B-10 violado — o ARCA nunca apaga nada:\n{}",
        violacoes.join("\n")
    );
}

#[test]
fn a_porta_do_sistema_de_arquivos_nao_oferece_exclusao() {
    // O dominio so fala com a porta. Sem metodo de exclusao no contrato,
    // nenhum comando consegue apagar nada nem por engano — e acrescentar um
    // seria uma mudanca visivel, discutida, e nao um `remove` perdido no meio
    // de uma funcao.
    let porta = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/portas/arquivos.rs");
    let conteudo = std::fs::read_to_string(&porta).expect("a porta existe");

    for verbo in ["apagar", "remover", "excluir", "delete", "descartar"] {
        let assinatura = format!("fn {verbo}");
        assert!(
            !conteudo.contains(&assinatura),
            "a porta declara `{assinatura}`, e B-10 diz que o ARCA nunca apaga nada"
        );
    }
}
