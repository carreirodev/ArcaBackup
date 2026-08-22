//! S-6 como propriedade verificavel da arquitetura.
//!
//! O requisito diz que o ARCA nunca compara uma data escrita pelo Windows com
//! outra escrita pelo Linux para decidir se um desfecho pertence a um job.
//! Isso ja foi um comentario, e o comentario nao impediu: uma trava construida
//! sobre comparacao de datas reprovou um backup perfeito neste projeto
//! (§4.3, ADR-0001). O Clonezilla lê o RTC — hora local do Windows — como se
//! fosse UTC e roda 3 h adiantado, permanentemente (P-7).
//!
//! Ha data no ARCA, e ela e legitima: `arca list` mostra `21/08` ao lado de
//! cada imagem, e `crate::portas::Entrada` carrega o `modificado_em` do
//! sistema de arquivos. O que este teste cobra nao e ausencia de data — e que
//! **o codigo que decide nao alcance nenhuma**.
//!
//! Duas frentes, e a segunda e a que importa mais:
//!
//! 1. `MomentoDoArmar` guarda texto e nao deriva ordenacao. Nao ha o que
//!    subtrair nem o que comparar, e quem quisesse violar S-6 precisaria
//!    primeiro parsear a string de volta, de proposito, num `let` que
//!    apareceria no diff.
//! 2. `src/desfecho.rs` — o modulo que julga a quem um `arca-fim.txt`
//!    pertence — nao menciona tempo em forma nenhuma. Nao e disciplina: e que
//!    o tipo nao esta la para ser usado.

use std::path::{Path, PathBuf};

fn fonte(caminho: &str) -> String {
    let alvo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(caminho);
    std::fs::read_to_string(&alvo)
        .unwrap_or_else(|erro| panic!("nao consegui lê {}: {erro}", alvo.display()))
}

/// O codigo do arquivo sem os seus testes, como em `b10_nada_e_apagado.rs`.
///
/// A convencao do repositorio poe o `#[cfg(test)] mod testes` no fim de cada
/// arquivo, e um teste que constroi um momento a partir de um relogio parado
/// nao e o que S-6 governa.
fn sem_os_testes(conteudo: &str) -> String {
    match conteudo.find("#[cfg(test)]") {
        Some(inicio) => conteudo[..inicio].to_string(),
        None => conteudo.to_string(),
    }
}

/// Os `derive` que estao logo acima de uma declaracao.
fn derives_de(fonte: &str, declaracao: &str) -> String {
    let posicao = fonte
        .find(declaracao)
        .unwrap_or_else(|| panic!("`{declaracao}` nao existe mais: este teste ficou para tras"));

    // Trinta caracteres a mais que o maior `derive` deste repositorio, o que
    // basta para pegar o bloco inteiro sem alcancar a declaracao anterior.
    let inicio = posicao.saturating_sub(120);
    fonte[inicio..posicao].to_string()
}

#[test]
fn o_momento_do_armar_nao_pode_ser_ordenado() {
    // Derivar `Ord` aqui e uma linha, e depois dela `armado_em < outra_coisa`
    // compila. Com `PartialEq` nao ha problema — comparar dois momentos
    // escritos pelo **mesmo** relogio nao e o que S-6 proibe, e o teste de ida
    // e volta do `estado.json` precisa disso.
    let fonte = sem_os_testes(&fonte("src/estado.rs"));
    let derives = derives_de(&fonte, "pub struct MomentoDoArmar");

    for proibido in ["PartialOrd", "Ord"] {
        assert!(
            !derives.contains(proibido),
            "`MomentoDoArmar` deriva `{proibido}`, e ai `armado_em < qualquer coisa` compila. \
             S-6 proibe que o tempo decida (§4.3, ADR-0001). O bloco encontrado foi:\n{derives}"
        );
    }
}

#[test]
fn nada_no_estado_devolve_um_tempo_comparavel() {
    // O tipo guarda texto justamente para que nao haja saida. Um acessor
    // devolvendo `DateTime` desfaria a defesa inteira sem que nenhum `derive`
    // mudasse — e e assim que uma protecao morre: por conveniencia.
    let fonte = sem_os_testes(&fonte("src/estado.rs"));

    for assinatura in ["-> DateTime", "-> chrono::", "-> SystemTime", "-> Option<DateTime"] {
        assert!(
            !fonte.contains(assinatura),
            "`src/estado.rs` tem uma funcao `{assinatura}`: o momento do armar voltou a ser \
             comparavel, e S-6 depende de ele nao ser"
        );
    }
}

#[test]
fn o_modulo_que_julga_o_desfecho_nao_alcanca_o_tempo() {
    // A garantia forte. `desfecho.rs` decide se um `arca-fim.txt` pertence ao
    // job pendente — e o unico lugar do sistema onde a comparacao errada
    // reprovaria um backup perfeito. Ele nao importa `chrono`, nao lê
    // `modificado_em` e nao conhece `SystemTime`: o que liga um job ao seu
    // desfecho e o selo, e nao ha alternativa a mao.
    let fonte = sem_os_testes(&fonte("src/desfecho.rs"));

    for marca in [
        "chrono",
        "DateTime",
        "SystemTime",
        "modificado_em",
        "Instant",
        "Local",
    ] {
        assert!(
            !fonte.contains(marca),
            "`src/desfecho.rs` menciona `{marca}`. Quem julga a quem um desfecho pertence nao \
             pode alcancar o tempo: o Clonezilla roda 3 h adiantado, de forma permanente (P-7), \
             e uma trava construida sobre datas ja reprovou um backup perfeito"
        );
    }

    // E o selo tem de estar la, senao este teste passaria num arquivo vazio.
    assert!(
        fonte.contains("Selo"),
        "`src/desfecho.rs` deixou de falar em selo: e ele que faz o julgamento (C-11)"
    );
}

#[test]
fn o_teste_aponta_para_arquivos_que_existem() {
    // Uma varredura que nao acha o alvo passa em silencio, que e a pior forma
    // de um teste de arquitetura falhar.
    for caminho in ["src/estado.rs", "src/desfecho.rs"] {
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(caminho).exists(),
            "{caminho} nao existe: este teste esta guardando um arquivo que saiu do lugar"
        );
    }
}
