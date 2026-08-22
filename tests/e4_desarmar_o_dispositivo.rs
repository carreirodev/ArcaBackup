//! A etapa E4 contra o dispositivo de verdade.
//!
//! Os testes de `src/grub.rs` provam que o desarmar devolve o inerte a partir
//! das copias preservadas em `recursos/capturas/`. Este arquivo prova a outra
//! metade: que aquelas copias **continuam sendo** o que esta no dispositivo, e
//! que o `grub.cfg` que a maquina boota hoje e de fato o inerte que a E4
//! reproduz.
//!
//! Sao coisas diferentes, e a segunda envelhece sozinha: uma atualizacao do
//! Clonezilla, um `arca prepare` da E10 ou um dispositivo novo mudam o
//! `grub.cfg`, e nenhum teste de fixture percebe.
//!
//! # Por que estes testes se pulam sozinhos
//!
//! Sem o dispositivo conectado nao ha `ARCABOOT`, e nao ha o que comparar.
//! Diferente da E2, aqui nao e questao de privilegio: ler o `R:\` e leitura de
//! arquivo comum, e o `grub.cfg` do `ARCABOOT` e escrivel pelo usuario —
//! medido em `examples/escrita_atomica_no_fat32.rs`. E questao de o SSD estar
//! ou nao na mesa.
//!
//! # Nenhum teste daqui escreve
//!
//! O `grub.cfg` e o arquivo de que a maquina depende para bootar, e um teste
//! nao e lugar de descobrir isso. Quem escreve nele e o `arca desarmar`, com
//! o dispositivo conectado e alguem olhando. Aqui so se lê e se compara.

#![cfg(windows)]

use arca::adaptadores::ArquivosDoSistema;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::dispositivo::{self, Dispositivo};
use arca::grub;
use arca::portas::Arquivos;
use std::path::PathBuf;

/// O `grub.cfg` inerte, como esta preservado no repositorio.
const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");

/// O dispositivo conectado, ou nada.
fn dispositivo() -> Option<Dispositivo> {
    match dispositivo::encontrar(&VolumesDoWindows) {
        Ok(dispositivo) => Some(dispositivo),
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

/// O caminho do `grub.cfg` do dispositivo conectado.
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

#[test]
fn o_grub_cfg_do_dispositivo_e_o_inerte_que_esta_no_repositorio() {
    // A copia em `recursos/capturas/` e evidencia, e evidencia que divergiu do
    // que ela documenta deixou de ser evidencia. Este teste e o que impede a
    // etapa inteira de estar provando alguma coisa sobre um arquivo que nao
    // existe mais.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema
        .ler_texto(&caminho)
        .expect("o grub.cfg do dispositivo e legivel");

    assert_eq!(
        corrente,
        INERTE,
        "o {} divergiu de recursos/capturas/grub-inerte-arcaboot.cfg. \
         Ou o dispositivo foi armado, ou o Clonezilla foi trocado — e nos dois \
         casos a captura precisa ser refeita antes de os testes da E4 valerem \
         alguma coisa",
        caminho.display()
    );
}

#[test]
fn o_dispositivo_esta_inerte_agora() {
    // O que o §6.3 do PRD pressupoe existir: um dispositivo em que se boota
    // por F12 e se usa o menu do Clonezilla. Se este teste falhar com o SSD
    // conectado, ha uma receita armada esperando um reinicio.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema.ler_texto(&caminho).expect("legivel");
    let desarmado = grub::desarmar(&corrente).expect("o grub.cfg do dispositivo desarma");

    assert!(
        !desarmado.havia_receita(),
        "ha receita armada em {}: {} bloco(s) do ARCA, set default devolvido: {}",
        caminho.display(),
        desarmado.blocos_removidos,
        desarmado.default_devolvido
    );
}

#[test]
fn desarmar_o_grub_cfg_do_dispositivo_nao_mudaria_um_byte() {
    // C-1 contra o arquivo de verdade, sem escrever nele. E a forma
    // verificavel do criterio de aceite da etapa: rodar o desarmar sobre o que
    // esta no dispositivo tem de sair identico ao que entrou.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema.ler_texto(&caminho).expect("legivel");
    let uma_vez = grub::desarmar(&corrente).expect("desarma");
    let duas_vezes = grub::desarmar(&uma_vez.texto).expect("desarma de novo");

    assert_eq!(
        uma_vez.texto, corrente,
        "a primeira passada mudaria o arquivo"
    );
    assert_eq!(
        duas_vezes.texto, corrente,
        "a segunda passada mudaria o arquivo"
    );
}

#[test]
fn as_copias_armadas_do_dispositivo_desarmam_para_o_inerte_corrente() {
    // As quatro copias que o dispositivo guarda ao lado do `grub.cfg`, lidas
    // de la e nao do repositorio. E a mesma prova de `src/grub.rs`, com o
    // oraculo vindo do disco em vez do `include_str!` — o que fecha a
    // possibilidade de as duas pontas terem sido copiadas juntas e erradas.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };
    let pasta = caminho.parent().expect("o grub.cfg tem pasta");
    let inerte = ArquivosDoSistema.ler_texto(&caminho).expect("legivel");

    let mut conferidas = 0;
    for copia in ["grub.cfg.teste01", "grub.cfg.teste02", "grub.cfg.backup02"] {
        let caminho_da_copia = pasta.join(copia);
        if !ArquivosDoSistema.existe(&caminho_da_copia) {
            eprintln!("pulado: {copia} nao esta no dispositivo");
            continue;
        }

        let armada = ArquivosDoSistema
            .ler_texto(&caminho_da_copia)
            .expect("a copia e legivel");
        let desarmada = grub::desarmar(&armada).expect("a copia desarma");

        assert_eq!(desarmada.texto, inerte, "`{copia}` nao voltou ao inerte");
        assert!(desarmada.havia_receita(), "`{copia}` nao estava armada");
        conferidas += 1;
    }

    assert!(
        conferidas > 0,
        "nenhuma copia armada foi conferida contra o dispositivo"
    );
}

#[test]
fn o_grub_cfg_que_o_clonezilla_entrega_desarma_para_o_inerte_deste_dispositivo() {
    // A resposta a "de onde vem o estado inerte", conferida contra o disco. O
    // `grub.cfg.original` e o que o Clonezilla instalou, com
    // `set default="0"` — que aponta por posicao, e a posicao muda quando o
    // bloco do ARCA entra. Desarmar o dele produz o inerte de hoje.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };
    let original = caminho.with_file_name("grub.cfg.original");
    if !ArquivosDoSistema.existe(&original) {
        eprintln!("pulado: grub.cfg.original nao esta no dispositivo");
        return;
    }

    let inerte = ArquivosDoSistema.ler_texto(&caminho).expect("legivel");
    let clonezilla = ArquivosDoSistema.ler_texto(&original).expect("legivel");
    let desarmado = grub::desarmar(&clonezilla).expect("desarma");

    assert_eq!(desarmado.texto, inerte);
    assert!(desarmado.default_devolvido);
    assert_eq!(desarmado.blocos_removidos, 0);
}

#[test]
fn o_arcaboot_e_fat32_e_e_nele_que_a_escrita_atomica_estreia() {
    // A E4 e a primeira etapa que escreve, e o primeiro arquivo que ela
    // escreve mora aqui. Que o volume seja FAT32 nao e detalhe: a escrita
    // atomica e temporario mais renomeacao, e em FAT32 nao ha jornal por
    // baixo. Medido em `examples/escrita_atomica_no_fat32.rs`.
    let Some(dispositivo) = dispositivo() else {
        return;
    };
    let Some(boot) = dispositivo.boot else {
        eprintln!("pulado: sem ARCABOOT");
        return;
    };

    assert!(
        boot.sistema_de_arquivos.eq_ignore_ascii_case("FAT32"),
        "o ARCABOOT deste dispositivo e {}, e a medicao da escrita atomica foi feita contra FAT32",
        boot.sistema_de_arquivos
    );
}
