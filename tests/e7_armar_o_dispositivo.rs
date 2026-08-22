//! A etapa E7 contra o hardware desta mesa.
//!
//! Os testes de `src/menuentry.rs` e `src/armar.rs` provam a derivacao e a
//! ordem das tres gravacoes contra capturas e duplos. Este arquivo prova a
//! outra metade — que aquelas capturas continuam descrevendo **esta** maquina
//! — e fixa os tres achados de medicao da etapa.
//!
//! # Nenhum teste daqui escreve
//!
//! Nem no `grub.cfg`, nem no firmware. Um teste que armasse deixaria a maquina
//! de quem o roda com boot unico pendente, e o proximo reinicio — venha de
//! onde vier — bootaria no dispositivo. Quem arma e `arca backup`, com alguem
//! olhando e depois de uma confirmacao digitada.
//!
//! O que **foi** medido escrevendo, a mao, em 22/08/2026, esta registrado no
//! ADR-0007: `bcdedit /set {fwbootmgr} bootsequence {f4057bd0-…}` sai com
//! codigo 0, a releitura mostra a marca, o `displayorder` nao muda, e o
//! `/deletevalue` seguinte a tira. Aqui isso vira asserção sobre a
//! **configuracao** que torna esse resultado significativo: a entrada do ARCA
//! estar de fora da ordem permanente.

#![cfg(windows)]

use arca::adaptadores::ArquivosDoSistema;
use arca::adaptadores::windows::firmware::Bcdedit;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::dispositivo::{self, Dispositivo};
use arca::firmware::{self, Procedencia};
use arca::grub;
use arca::menuentry;
use arca::nome::Nome;
use arca::portas::{Arquivos, Firmware};
use arca::receita::{Disco, Operacao, Pedido, Receita, Selo};
use std::path::PathBuf;

const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");

fn dispositivo() -> Option<Dispositivo> {
    match dispositivo::encontrar(&VolumesDoWindows) {
        Ok(dispositivo) => Some(dispositivo),
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

fn caminho_do_grub() -> Option<PathBuf> {
    let dispositivo = dispositivo()?;
    match dispositivo.caminho_do_grub() {
        Ok(caminho) if ArquivosDoSistema.existe(&caminho) => Some(caminho),
        Ok(caminho) => {
            eprintln!("pulado: {} nao existe", caminho.display());
            None
        }
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

/// A leitura do `{fwbootmgr}` desta maquina, ou nada.
///
/// Sem elevacao o `bcdedit` sai com codigo 1, e o adaptador transforma isso em
/// erro — a mesma razao pela qual os testes da E2 se pulam sozinhos.
fn gerenciador() -> Option<firmware::Leitura> {
    match Bcdedit.enumerar("{fwbootmgr}") {
        Ok(texto) => {
            let leitura = firmware::ler(&texto);
            if leitura.viu_o_gerenciador {
                Some(leitura)
            } else {
                eprintln!("pulado: o bcdedit respondeu sem o gerenciador de firmware");
                None
            }
        }
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

/// A receita de backup como o ARCA a monta hoje, com um selo de exemplo.
fn receita() -> Receita {
    Receita::montar(&Pedido {
        operacao: Operacao::Backup,
        nome: Nome::novo("2026-08-22_Apps").expect("nome valido"),
        disco: Disco::novo("nvme0n1").expect("disco valido"),
        selo: Selo::novo("a3f1c9e07b2d4856").expect("selo valido"),
    })
    .expect("a receita e valida por C-2")
}

#[test]
fn o_bloco_deriva_do_grub_cfg_que_esta_no_dispositivo_agora() {
    // A derivacao tem oraculo contra a captura `teste-02`, e isso esta em
    // `src/menuentry.rs`. O que este teste acrescenta e que o **modelo**
    // continua no dispositivo: um `arca prepare` da E10, um Clonezilla novo ou
    // outro dispositivo mudam o `grub.cfg`, e nenhum teste de fixture percebe.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema
        .ler_texto(&caminho)
        .expect("o grub.cfg do dispositivo e legivel");

    let bloco = menuentry::derivar(&corrente, receita().parametros())
        .expect("o grub.cfg do dispositivo tem de onde derivar o bloco do ARCA");

    // A configuracao **deste** hardware atravessa — e a razao inteira de
    // derivar em vez de transcrever.
    for herdado in ["hostname=cl-3.3.3-15", "nvme.poll_queues=1", "toram="] {
        assert!(
            bloco.contains(herdado),
            "o bloco derivado do dispositivo perdeu `{herdado}`"
        );
    }
}

#[test]
fn armar_e_desarmar_o_dispositivo_de_verdade_se_cancelam() {
    // A ida e a volta sobre o arquivo que a maquina boota, sem escrever nele.
    // Se algum dia o bloco derivado deixar de ser removivel, e aqui que
    // aparece — e antes de o `arca backup` grava-lo.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema
        .ler_texto(&caminho)
        .expect("o grub.cfg do dispositivo e legivel");

    let bloco = menuentry::derivar(&corrente, receita().parametros()).expect("deriva");
    let armado = grub::armar(&corrente, &bloco).expect("arma");

    assert_ne!(armado, corrente, "armar nao mudou nada");
    assert!(armado.contains("set default=\"arca-backup\""));

    let desarmado = grub::desarmar(&armado).expect("desarma");
    assert_eq!(
        desarmado.texto, corrente,
        "armar e desarmar nao devolveram o arquivo do dispositivo byte a byte"
    );
}

#[test]
fn a_entrada_do_arca_existe_nesta_maquina_e_e_a_propria() {
    // C-4 medido: a entrada desta maquina ja se chama `ARCA` — ela foi migrada
    // a mao em 22/08, e a captura de 20/08 preserva o estado anterior. O caso
    // "ha a legada `Clonezilla`" continua coberto por aquela captura, em
    // `src/armar.rs`.
    let Ok(texto) = Bcdedit.enumerar("firmware") else {
        eprintln!("pulado: o bcdedit recusou o /enum firmware");
        return;
    };

    let leitura = firmware::ler(&texto);
    let Some(achado) = leitura.entrada_do_arca() else {
        panic!(
            "nao ha entrada `ARCA` nem `Clonezilla` nesta maquina, e armar nao cria entrada de boot"
        );
    };

    assert_eq!(
        achado.procedencia,
        Procedencia::Propria,
        "a entrada desta maquina voltou a se chamar `{}`",
        achado.descricao
    );
    assert!(
        achado.entrada.alvo.is_some(),
        "a entrada existe e nao diz para onde ir"
    );
}

#[test]
fn a_entrada_do_arca_esta_fora_da_ordem_permanente() {
    // **O achado da etapa, fixado.** C-5 proibe o ARCA de tocar a ordem
    // permanente, e por isso o `bootsequence` tem de funcionar sobre uma
    // entrada que **nao esta nela** — o que nao estava medido em lugar nenhum.
    //
    // Medido em 22/08/2026: o `displayorder` do `{fwbootmgr}` desta maquina
    // tras so o `{bootmgr}`. Este teste e o que faz a medicao do ADR-0007
    // significar alguma coisa: se um dia a entrada do ARCA voltar para a
    // ordem, o resultado daquela medicao deixa de valer, e P-18 reabre.
    let Some(leitura) = gerenciador() else {
        return;
    };
    let Ok(texto) = Bcdedit.enumerar("firmware") else {
        eprintln!("pulado: o bcdedit recusou o /enum firmware");
        return;
    };
    let Some(achado) = firmware::ler(&texto).entrada_do_arca().map(|achado| {
        achado.entrada.identificador.clone()
    }) else {
        eprintln!("pulado: nao ha entrada do ARCA nesta maquina");
        return;
    };

    assert!(
        !leitura
            .ordem_permanente
            .iter()
            .any(|entrada| entrada.eq_ignore_ascii_case(&achado)),
        "a entrada do ARCA ({achado}) esta na ordem permanente [{}]. O ARCA nunca a poe la (C-5), \
         entao alguem a pos — e o boot unico deixa de ser o que faz a maquina bootar no dispositivo. \
         Ver ADR-0007 e P-18",
        leitura.ordem_permanente.join(", ")
    );
}

#[test]
fn nao_ha_boot_unico_pendente_nesta_maquina() {
    // O estado normal, e o que este arquivo inteiro pressupoe. Um
    // `bootsequence` sobrando aqui seria um job armado que ninguem colheu — ou
    // um teste que escreveu onde nao devia.
    let Some(leitura) = gerenciador() else {
        return;
    };

    assert!(
        !leitura.tem_boot_unico(),
        "ha boot unico armado apontando para [{}]. Rode `arca status` e depois `arca desarmar`",
        leitura.boot_unico.join(", ")
    );
}

#[test]
fn o_grub_cfg_do_dispositivo_continua_inerte_e_igual_a_captura() {
    // A E4 ja cobra isto, e a E7 o cobra de novo por um motivo proprio: ela e
    // a primeira etapa que **escreve** neste arquivo. A copia em
    // `recursos/capturas/` e a unica que existe fora do dispositivo, e ela tem
    // de continuar sendo o que estava la antes da primeira gravacao.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema
        .ler_texto(&caminho)
        .expect("o grub.cfg do dispositivo e legivel");

    assert_eq!(
        corrente, INERTE,
        "o grub.cfg do dispositivo divergiu da copia do repositorio. Se foi um `arca backup` que \
         o armou, `arca desarmar` o devolve; se foi outra coisa, a copia precisa ser refeita antes \
         de continuar valendo como evidencia"
    );
}
