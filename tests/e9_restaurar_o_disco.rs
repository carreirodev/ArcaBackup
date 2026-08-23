//! A etapa E9 contra o hardware desta mesa.
//!
//! O julgamento de R-7 e testado em `src/comandos/restore.rs` contra duplos, e
//! o leitor do `sgdisk` em `src/gpt.rs` contra o texto copiado da imagem. O
//! que este arquivo prova e o que nenhum dos dois pode: **que as duas pontas da
//! comparacao continuam saindo da mesma regua no hardware de verdade.**
//!
//! O achado que a etapa mediu, e que vale mais do que qualquer asserção daqui:
//! o **mesmo** disco tem dois tamanhos conforme quem responde.
//!
//! ```text
//! Get-Disk (MSFT_Disk) ........ 500.107.862.016 bytes = 976.773.168 setores
//! Win32_DiskDrive.Size ........ 500.105.249.280 bytes = 976.768.065 setores
//! nvme0n1-gpt.sgdisk na imagem  976.773.168 setores
//! diferenca ................... 2.612.736 bytes = 5.103 setores
//! ```
//!
//! `60801 x 255 x 63 x 512` da exatamente o numero do `Win32_DiskDrive` — o
//! produto da geometria CHS legada, truncado no ultimo cilindro inteiro. Com a
//! regua errada, este disco nao caberia em si mesmo.
//!
//! # Nenhum teste daqui escreve, e nenhum arma
//!
//! Sao leituras: o `ARCAVAULT`, o WMI e os arquivos de dentro das imagens.
//! Quem arma e `arca restore`, e armar e o ponto sem volta.

#![cfg(windows)]

use arca::adaptadores::ArquivosDoSistema;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::blkdev;
use arca::dispositivo::{self, Dispositivo};
use arca::gpt;
use arca::imagens;
use arca::portas::{Arquivos, DiscoFisico, Discos};
use std::path::PathBuf;

fn dispositivo() -> Option<Dispositivo> {
    match dispositivo::encontrar(&VolumesDoWindows) {
        Ok(dispositivo) => Some(dispositivo),
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

fn raiz_do_vault() -> Option<PathBuf> {
    match dispositivo()?.raiz_do_vault() {
        Ok(raiz) => Some(raiz),
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

fn discos() -> Option<Vec<DiscoFisico>> {
    match VolumesDoWindows.discos_fisicos() {
        Ok(discos) => Some(discos),
        Err(motivo) => {
            eprintln!("pulado: {motivo}");
            None
        }
    }
}

/// A origem de cada imagem do dispositivo, lida do `sgdisk` de dentro dela.
fn origens() -> Vec<(String, gpt::OrigemDaImagem)> {
    let Some(raiz) = raiz_do_vault() else {
        return Vec::new();
    };
    let Ok(pastas) = imagens::enumerar(&ArquivosDoSistema, &raiz) else {
        return Vec::new();
    };

    let mut achadas = Vec::new();
    for pasta in pastas.iter().filter(|pasta| pasta.e_imagem()) {
        let dentro = raiz.join(&pasta.nome);
        let Ok(disco) = ArquivosDoSistema.ler_texto_alheio(&dentro.join("disk")) else {
            continue;
        };
        let arquivo = gpt::arquivo_do_disco(disco.trim());
        let Ok(texto) = ArquivosDoSistema.ler_texto_alheio(&dentro.join(&arquivo)) else {
            continue;
        };
        if let Ok(origem) = gpt::ler(&arquivo, &texto) {
            achadas.push((pasta.nome.clone(), origem));
        }
    }
    achadas
}

#[test]
fn toda_imagem_deste_dispositivo_traz_a_medida_da_origem() {
    // R-7 so se responde se a imagem disser de que tamanho era o disco. Este
    // teste e o que garante que a fonte escolhida — o `<disco>-gpt.sgdisk` —
    // esta em **todas** as imagens que este dispositivo carrega, e nao so
    // naquela em que ela foi encontrada.
    let Some(raiz) = raiz_do_vault() else {
        return;
    };
    let Ok(pastas) = imagens::enumerar(&ArquivosDoSistema, &raiz) else {
        eprintln!("pulado: nao deu para enumerar o ARCAVAULT");
        return;
    };

    let quantas = pastas.iter().filter(|pasta| pasta.e_imagem()).count();
    if quantas == 0 {
        eprintln!("pulado: nao ha imagem no dispositivo");
        return;
    }

    assert_eq!(
        origens().len(),
        quantas,
        "toda imagem tem de trazer `disk` e `<disco>-gpt.sgdisk` legiveis"
    );
}

#[test]
fn o_msft_disk_bate_byte_a_byte_com_a_gpt_de_dentro_da_imagem() {
    // **O teste que a etapa inteira existe para escrever.** As duas pontas da
    // comparacao de R-7 — o destino, medido pelo Windows, e a origem, lida da
    // imagem — tem de sair da mesma regua. Este teste falha no dia em que
    // alguem trocar a fonte do destino de volta para o `Win32_DiskDrive`.
    let (Some(discos), origens) = (discos(), origens()) else {
        return;
    };
    if origens.is_empty() {
        eprintln!("pulado: nao ha imagem com medida");
        return;
    }

    let mut conferidos = 0;
    for (imagem, origem) in &origens {
        let Some(disco) = discos
            .iter()
            .find(|disco| blkdev::mesmo_modelo(&disco.modelo, &origem.modelo))
        else {
            eprintln!("pulado: `{}` nao esta nesta maquina", origem.modelo);
            continue;
        };
        let Some(medida) = disco.medida else {
            panic!(
                "o `MSFT_Disk` nao respondeu por `{}`, e sem ele R-7 nao se responde",
                disco.modelo
            );
        };

        assert_eq!(
            medida.bytes_por_setor, origem.bytes_por_setor,
            "o setor logico de `{}` mudou entre a imagem `{imagem}` e agora",
            disco.modelo
        );
        assert_eq!(
            medida.setores(),
            origem.setores,
            "o disco `{}` mediu {} setores agora e {} quando a imagem `{imagem}` foi feita — \
             as duas pontas de R-7 tem de sair da mesma regua",
            disco.modelo,
            medida.setores(),
            origem.setores
        );
        conferidos += 1;
    }

    assert!(
        conferidos > 0,
        "nenhum disco desta maquina casou com imagem nenhuma — o teste nao exercitou nada"
    );
}

#[test]
fn o_win32_diskdrive_nao_bate_e_a_diferenca_e_o_truncamento_chs() {
    // O outro lado da mesma medicao, e ele tem de continuar falso. Um dia em
    // que os dois numeros coincidirem nesta maquina, o teste acima passaria a
    // provar menos do que diz — e este avisa.
    let (Some(discos), origens) = (discos(), origens()) else {
        return;
    };

    for (imagem, origem) in &origens {
        let Some(disco) = discos
            .iter()
            .find(|disco| blkdev::mesmo_modelo(&disco.modelo, &origem.modelo))
        else {
            continue;
        };

        let pelo_win32 = disco.tamanho_bytes / origem.bytes_por_setor;
        if pelo_win32 == origem.setores {
            eprintln!(
                "nota: em `{}` as duas fontes coincidem (imagem `{imagem}`) — \
                 a armadilha da regua nao aparece neste disco",
                disco.modelo
            );
            continue;
        }

        assert!(
            pelo_win32 < origem.setores,
            "o `Win32_DiskDrive` so pode ficar ATRAS: ele trunca no ultimo cilindro CHS inteiro"
        );

        // Menos de um cilindro de 255x63 setores e a assinatura do
        // truncamento. Mais do que isso seria outra coisa, e ai a explicacao
        // deste projeto estaria errada.
        let atras = origem.setores - pelo_win32;
        assert!(
            atras < 255 * 63,
            "a diferenca de {atras} setores em `{}` e maior que um cilindro CHS ({}): \
             isso nao e truncamento de geometria, e o ADR-0010 esta explicando outra coisa",
            disco.modelo,
            255 * 63
        );
    }
}

#[test]
fn o_disco_desta_maquina_cabe_em_si_mesmo_pela_regua_certa() {
    // O caso mais simples da restauracao, e o que a regua errada quebraria:
    // restaurar a imagem de volta no disco de que ela veio.
    let (Some(discos), origens) = (discos(), origens()) else {
        return;
    };

    for (imagem, origem) in &origens {
        let Some(disco) = discos
            .iter()
            .find(|disco| blkdev::mesmo_modelo(&disco.modelo, &origem.modelo))
        else {
            continue;
        };
        let Some(medida) = disco.medida else {
            continue;
        };

        assert!(
            medida.setores() >= origem.setores,
            "a imagem `{imagem}` nao caberia no disco de que ela veio (`{}`)",
            disco.modelo
        );
    }
}

#[test]
fn o_disco_do_dispositivo_e_menor_que_a_origem_desta_mesa() {
    // Nao e detalhe de configuracao: e o que torna a recusa dura do
    // dispositivo **verificavel de dois jeitos** nesta mesa. Se alguem
    // trocasse a ordem das recusas em `escolher_o_destino`, o SSD externo
    // continuaria recusado — mas pela mensagem errada, falando de tamanho, e
    // quem lesse acharia que um SSD maior resolveria.
    let (Some(discos), Some(dispositivo), origens) = (discos(), dispositivo(), origens()) else {
        return;
    };
    let Some((_, origem)) = origens.first() else {
        eprintln!("pulado: nao ha imagem com medida");
        return;
    };

    let letras: Vec<char> = dispositivo
        .vault
        .letra
        .into_iter()
        .chain(dispositivo.boot.as_ref().and_then(|boot| boot.letra))
        .collect();

    let Some(o_dispositivo) = discos
        .iter()
        .find(|disco| letras.iter().any(|letra| disco.tem_a_letra(*letra)))
    else {
        eprintln!("pulado: nao deu para achar o disco do dispositivo");
        return;
    };
    let Some(medida) = o_dispositivo.medida else {
        eprintln!("pulado: o MSFT_Disk nao respondeu pelo dispositivo");
        return;
    };

    assert!(
        medida.setores() < origem.setores,
        "esta mesa deixou de ter o dispositivo menor que a origem, e o teste acima perdeu o \
         segundo caminho que ele documentava"
    );
}

#[test]
fn o_help_do_ocs_sr_continua_dizendo_que_o_destino_menor_e_recusado() {
    // P-17, fechada na E9 contra o original. O `ocs-sr-help.txt` e o help
    // **desta** versao, tirado desta maquina — e ele diz o contrario do que
    // R-7 supunha: o Clonezilla confere e **desiste**, em vez de corromper.
    //
    // A recusa do ARCA continua valendo, e por outro motivo: a do Clonezilla
    // acontece do outro lado do reinicio. Ver o ADR-0010.
    const HELP: &str = include_str!("../recursos/capturas/ocs-sr-help.txt");

    assert!(
        HELP.contains(
            "By default it will be checked and if the size is smaller than the source disk, quit"
        ),
        "a conferencia de tamanho do destino deixou de ser o padrao do ocs-sr"
    );
    assert!(
        HELP.contains("-icds, --ignore-chk-dsk-size-pt"),
        "`-icds` sumiu do help, e e ele quem desligaria a conferencia"
    );

    // E a outra metade, que a E9 achou lendo o help inteiro: na restauracao
    // tambem ha uma conferencia nativa ligada por padrao — mas ela e sobre a
    // **imagem**, e roda antes de gravar. Nao e um segundo juiz do resultado,
    // e P-6 continua aberta deste lado.
    assert!(
        HELP.contains("By default Clonezilla will check the image if restorable before restoring"),
        "`-scr` sumiu do help"
    );
}
