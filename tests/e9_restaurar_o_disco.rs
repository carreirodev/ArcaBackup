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
use arca::desfecho::{self, Julgamento};
use arca::dispositivo::{self, Dispositivo};
use arca::estado::{Estado, Situacao};
use arca::firmware;
use arca::gpt;
use arca::imagens;
use arca::nome::Nome;
use arca::portas::{Arquivos, DiscoFisico, Discos};
use arca::receita::{Disco, Operacao, Pedido, Receita, Selo};
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

// ─────────────────────────────────────────────────────────────────────────────
// O marco em hardware, cumprido em 23/08/2026 as 11:31:55 do relogio do live.
//
// Os tres testes abaixo so foram possiveis depois dele: cada um corre contra um
// original que nao existia antes, e nenhum deles poderia ser escrito a partir
// do codigo. Correm contra as **capturas**, e nao contra o dispositivo, pelo
// motivo que a E8 registrou — o `arca-fim.txt` do `ARCAVAULT` sera truncado
// pela proxima operacao com este nome, porque toda receita comeca por
// `echo ARCA_SELO=… >` e o `>` trunca ao abrir.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn o_desfecho_da_restauracao_e_julgado_como_operacao_concluida() {
    // **As duas pontas do selo, e cada uma escrita de um lado do reinicio.**
    //
    // O `estado.json` saiu do Windows, do `arca restore`, antes de a maquina
    // desligar. O `arca-fim.txt` saiu do `bash` do live, do outro lado, e o
    // Windows que o le agora nem e o mesmo — ele veio de dentro da imagem.
    // Entre um e outro o disco inteiro foi apagado e reescrito.
    //
    // E o par so existe porque o estado mora no `ARCABOOT` (§4.1). O
    // `arca.log` do `%LOCALAPPDATA%`, que registraria o mesmo selo do lado
    // Windows, foi destruido por esta operacao: ele mora no `C:`.
    const DESFECHO: &str =
        include_str!("../recursos/capturas/arca-fim-restauracao-2026-08-22_Apps.txt");
    const ESTADO: &str =
        include_str!("../recursos/capturas/estado-restauracao-2026-08-22_Apps.json");

    let estado = Estado::de_json(ESTADO).expect("o estado.json do marco e legivel");
    assert_eq!(
        estado.comando,
        Operacao::Restauracao,
        "o estado do marco da E9 tem de ser de uma restauracao"
    );
    assert_eq!(
        estado.situacao,
        Situacao::Colhido,
        "o marco foi colhido em 23/08/2026 as 11:50:53, e o ADR-0008 diz que a marca fica"
    );

    let lido = desfecho::ler(DESFECHO);
    assert_eq!(
        lido.linhas_de_selo, 1,
        "o desfecho da restauracao tem de ter exatamente uma linha de selo: {DESFECHO:?}"
    );
    assert!(lido.fim, "o desfecho da restauracao perdeu o ARCA_FIM");
    assert!(
        lido.deu_certo && !lido.falhou,
        "o desfecho da restauracao nao e o do ramo de exito: {DESFECHO:?}"
    );

    assert_eq!(
        desfecho::julgar(&lido, &estado.selo),
        Julgamento::Concluida,
        "o desfecho e o estado do mesmo job nao se reconheceram"
    );

    // E o outro lado, que e o que o selo existe para fazer. O selo escolhido e
    // o do **backup** de 22/08 — o job anterior, real, cujo desfecho mora na
    // pasta ao lado. E o job fantasma mais plausivel que este dispositivo tem.
    let do_backup = Selo::novo("7d2d2f5153625b38").expect("selo valido");
    assert!(
        matches!(
            desfecho::julgar(&lido, &do_backup),
            Julgamento::JobFantasma { .. }
        ),
        "o desfecho da restauracao passou por desfecho do backup de 22/08"
    );
}

#[test]
fn a_receita_da_restauracao_escreve_o_marcador_que_o_original_traz() {
    // **O ciclo da transcricao fechando.** Ate aqui o `ARCA_RESTORE=` era
    // codigo novo, sem original: a E3 marcou o `arca-fim.txt` inteiro como
    // nunca-executado (P-16), a E7 fechou a metade do backup, e esta e a da
    // restauracao. O que este teste compara e a string que `src/receita.rs`
    // monta hoje com a que o `bash` do live gravou no disco.
    //
    // Um teste que so cobrasse `contains("ARCA_RESTORE")` na receita provaria
    // metade: provaria que o codigo diz o que pretende dizer, e nao que o que
    // saiu do outro lado foi aquilo.
    const DESFECHO: &str =
        include_str!("../recursos/capturas/arca-fim-restauracao-2026-08-22_Apps.txt");

    let pedido = Pedido {
        operacao: Operacao::Restauracao,
        nome: Nome::novo("2026-08-22_Apps").expect("o nome do marco passa por B-2"),
        disco: Disco::novo("nvme0n1").expect("disco valido"),
        selo: Selo::novo("ce04819cf0ee96f7").expect("o selo do marco"),
    };
    let receita = Receita::montar(&pedido).expect("a receita do marco continua valida");

    for linha in DESFECHO.lines().map(str::trim).filter(|l| !l.is_empty()) {
        assert!(
            receita.comando().contains(linha),
            "o original traz `{linha}`, e a receita de hoje nao a escreveria"
        );
    }

    // E a metade que importa mais: a receita **nao** escreveria o marcador da
    // outra operacao. Sao dois `echo` num `if/then/else`, e trocar o marcador
    // faria a colheita chamar o veredito da imagem de origem de verificacao
    // desta operacao — que e a primeira das tres diferencas do §6.3.
    assert!(
        !receita.comando().contains("ARCA_BACKUP"),
        "a receita de restauracao escreve o marcador do backup"
    );
}

#[test]
fn depois_da_restauracao_nenhuma_entrada_da_ordem_leva_ao_dispositivo() {
    // **O achado do marco, fixado contra as duas leituras que o produziram.**
    //
    // O §3.4 chamava de fundacao validada que a restauracao nao mexe na NVRAM,
    // e a evidencia era um par `nvram-antes`/`-depois` lido de **dentro do
    // mesmo boot** do live. Aquele par continua certo, e fala do `ocs-sr`.
    // Este fala do ciclo inteiro — armar, bootar, restaurar, religar — e diz
    // outra coisa: atravessando o reinicio, a ordem permanente muda.
    //
    // O ARCA nao a mudou. C-5 proibe, e tanto o armar quanto o desarme releem
    // (C-3); uma escrita dessas teria falhado alto. Ver o ADR-0012.
    const ANTES: &str = include_str!(
        "../recursos/capturas/bcdedit-enum-firmware-2026-08-23-antes-da-restauracao.txt"
    );
    const DEPOIS: &str =
        include_str!("../recursos/capturas/bcdedit-enum-firmware-2026-08-23-pos-restauracao.txt");

    // A letra vem das capturas, e nao do dispositivo de hoje: as duas trazem
    // `partition=R:`, e e sobre elas que este teste fala.
    let leva_ao_dispositivo = |leitura: &firmware::Leitura, identificador: &String| {
        leitura
            .entradas
            .iter()
            .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador))
            .and_then(|entrada| entrada.alvo.as_ref())
            .and_then(|alvo| alvo.letra())
            .is_some_and(|letra| letra.eq_ignore_ascii_case(&'R'))
    };

    let antes = firmware::ler(ANTES);
    let depois = firmware::ler(DEPOIS);
    assert!(
        antes.viu_o_gerenciador && depois.viu_o_gerenciador,
        "uma das duas leituras nao achou o gerenciador, e uma leitura vazia \
         e indistinguivel de uma ordem sem o dispositivo"
    );

    assert_eq!(
        antes
            .ordem_permanente
            .iter()
            .position(|id| leva_ao_dispositivo(&antes, id)),
        Some(0),
        "antes da restauracao o dispositivo estava em primeiro na ordem: [{}]",
        antes.ordem_permanente.join(", ")
    );
    assert!(
        !depois
            .ordem_permanente
            .iter()
            .any(|id| leva_ao_dispositivo(&depois, id)),
        "depois da restauracao a ordem ainda leva ao dispositivo: [{}]",
        depois.ordem_permanente.join(", ")
    );

    // E o que diz **para onde** ela voltou, que e o achado e nao a
    // consequencia: a leitura de depois e byte a byte a que a E2 capturou em
    // 22/08, antes de o ciclo de boot do primeiro backup acrescentar as duas
    // entradas. A restauracao devolveu o firmware ao que estava dentro da
    // imagem, junto com o disco.
    const DA_E2: &str = include_str!("../recursos/capturas/bcdedit-enum-firmware-pt.txt");
    assert_eq!(
        DEPOIS, DA_E2,
        "a leitura de depois da restauracao deixou de ser identica a de 22/08"
    );

    // A entrada `{687478f2}` `UEFI OS` sumiu **inteira**, e nao so da ordem.
    // Ela e a que o firmware criou durante o boot do backup, e o `bcdedit` a
    // lista como `Aplicativo de Firmware` — o que ele mostra para uma entrada
    // que nao e objeto do BCD. Foi por ela que a maquina bootou em 22/08.
    assert!(
        ANTES.contains("687478f2") && !DEPOIS.contains("687478f2"),
        "a entrada que o firmware criou nao e a que separa as duas leituras"
    );
}
