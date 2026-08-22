//! A etapa E2 contra o `bcdedit` de verdade.
//!
//! As capturas provam que o parser lê o que o `bcdedit` escreveu; este arquivo
//! prova que o que chega ate ele ainda e o que o `bcdedit` escreve. Sao coisas
//! diferentes, e a segunda e a que envelhece sozinha: uma atualizacao do
//! Windows muda o formato, e nenhum teste de fixture percebe.
//!
//! # Por que estes testes se pulam sozinhos
//!
//! `bcdedit /enum` exige privilegio administrativo, e um binario de teste em
//! `tests/` **nao** carrega o manifesto `requireAdministrator` — o `build.rs`
//! so o aplica ao bin `arca`. Rodando sem elevacao, o `bcdedit` escreve
//! "Acesso negado" na saida padrao e sai com codigo 1, que o adaptador
//! converte em [`Erro::FerramentaRecusou`].
//!
//! Isso e diferente da E1, onde ler `E:\` e leitura de arquivo comum e os
//! testes rodam sem UAC. Aqui, sem elevacao eles dizem que pularam — e o que
//! os faz valer alguma coisa e rodar este binario a partir de um processo ja
//! elevado. O criterio de aceite da etapa e a execucao do `arca status`, nao
//! este arquivo.

#![cfg(windows)]

use arca::adaptadores::windows::firmware::Bcdedit;
use arca::erro::Erro;
use arca::firmware::{self, Leitura};
use arca::portas::Firmware;

/// A saida do `bcdedit`, ou nada quando falta privilegio para lê-la.
fn enumeracao() -> Option<String> {
    match Bcdedit.enumerar("firmware") {
        Ok(texto) => Some(texto),
        Err(Erro::FerramentaRecusou { .. }) => {
            eprintln!("pulado: `bcdedit /enum` precisa de elevacao, e este binario nao a tem");
            None
        }
        Err(outro) => panic!("o bcdedit falhou por outro motivo: {outro}"),
    }
}

fn leitura() -> Option<Leitura> {
    enumeracao().map(|texto| firmware::ler(&texto))
}

#[test]
fn o_texto_que_chega_do_bcdedit_nao_tem_caractere_perdido() {
    // Este e o teste que a etapa existe para ter. O `bcdedit` escreve na pagina
    // de codigo do console — 850 na janela que o UAC abre nesta maquina —, e
    // lê-lo como UTF-8 troca cada acento por `U+FFFD` sem levantar erro nenhum.
    // Medido em `examples/codificacao_do_bcdedit.rs`; corrigido em
    // `adaptadores::windows::texto`.
    let Some(texto) = enumeracao() else { return };

    let perdidos = texto.chars().filter(|c| *c == '\u{FFFD}').count();
    assert_eq!(
        perdidos, 0,
        "{perdidos} caractere(s) perdido(s) ao decodificar a saida do bcdedit"
    );
}

#[test]
fn a_enumeracao_desta_maquina_tem_entradas_de_boot() {
    let Some(leitura) = leitura() else { return };

    assert!(
        !leitura.entradas.is_empty(),
        "o bcdedit respondeu e o parser nao achou entrada nenhuma — \
         o formato mudou, ou a leitura quebrou"
    );

    // Toda entrada tem identificador entre chaves. E a unica coisa que o parser
    // acha por posicao, e e dela que tudo o mais depende.
    for entrada in &leitura.entradas {
        assert!(
            entrada.identificador.starts_with('{') && entrada.identificador.ends_with('}'),
            "identificador estranho: {:?}",
            entrada.identificador
        );
    }
}

#[test]
fn a_entrada_do_arca_existe_nesta_maquina_e_aponta_para_algum_lugar() {
    let Some(leitura) = leitura() else { return };

    let Some(achado) = leitura.entrada_do_arca() else {
        // Nao e falha: uma maquina sem entrada nenhuma e o estado antes da E7.
        // Mas nesta aqui ha uma, e dizer isso alto ajuda quem rodar noutra.
        eprintln!("esta maquina nao tem entrada ARCA nem Clonezilla no firmware");
        return;
    };

    eprintln!(
        "entrada {:?} · {} · {} · {:?} · {:?}",
        achado.procedencia,
        achado.descricao,
        achado.entrada.identificador,
        achado.entrada.alvo,
        achado.entrada.caminho
    );

    assert!(
        achado.entrada.alvo.is_some(),
        "a entrada existe e nao diz para onde bootar"
    );
    assert!(
        achado
            .entrada
            .caminho
            .as_deref()
            .is_some_and(|caminho| caminho.to_lowercase().ends_with(".efi")),
        "a entrada nao carrega um .efi: {:?}",
        achado.entrada.caminho
    );
}

#[test]
fn ler_o_firmware_duas_vezes_da_o_mesmo_resultado() {
    // A E2 so lê. Se a segunda leitura divergisse da primeira, alguma coisa
    // teria mudado o firmware entre as duas — e nada aqui tem permissao para
    // isso.
    let Some(primeira) = leitura() else { return };
    let segunda = leitura().expect("a segunda leitura tambem responde");

    assert_eq!(primeira, segunda);
}

#[test]
fn a_captura_do_repositorio_ainda_descreve_este_firmware() {
    // Nao compara texto: compara o que o parser extrai. O BCD desta maquina
    // muda — a entrada foi renomeada entre 20/08 e 22/08 —, e uma fixture que
    // exigisse igualdade byte a byte quebraria a cada mudanca sem indicar nada
    // de util.
    //
    // O que se cobra e que a **forma** continue a mesma: os campos que a
    // captura tem, a enumeracao de agora tambem tem.
    let Some(agora) = leitura() else { return };

    let capturado = firmware::ler(include_str!(
        "../recursos/capturas/bcdedit-enum-firmware-pt.txt"
    ));

    for entrada in &capturado.entradas {
        let Some(igual) = agora
            .entradas
            .iter()
            .find(|sua| sua.identificador == entrada.identificador)
        else {
            eprintln!(
                "a entrada {} sumiu do firmware desde a captura",
                entrada.identificador
            );
            continue;
        };

        assert_eq!(
            igual.alvo.is_some(),
            entrada.alvo.is_some(),
            "o campo `device` de {} mudou de existencia",
            entrada.identificador
        );
        assert_eq!(
            igual.caminho.is_some(),
            entrada.caminho.is_some(),
            "o campo `path` de {} mudou de existencia",
            entrada.identificador
        );
    }
}

#[test]
fn a_leitura_do_firmware_nao_arma_nada() {
    // C-5, do lado da leitura: `arca status` nao pode deixar a maquina com boot
    // unico pendente. Nada neste caminho escreve, e este teste e o que cobra
    // isso contra o firmware de verdade.
    let Some(leitura) = leitura() else { return };

    assert!(
        !leitura.tem_boot_unico(),
        "ha boot unico armado neste firmware: {:?}. \
         Se nao foi voce que armou, alguma coisa escreveu no firmware",
        leitura.boot_unico
    );
}
