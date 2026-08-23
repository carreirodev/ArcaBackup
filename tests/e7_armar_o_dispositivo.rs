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
        disco: Some(Disco::novo("nvme0n1").expect("disco valido")),
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
fn o_dispositivo_a_frente_da_ordem_permanente_exige_o_dispositivo_inerte() {
    // **Este teste ja foi outro, e a troca esta no ADR-0009.**
    //
    // Ele cobrava que a entrada do ARCA estivesse **fora** do `displayorder`,
    // porque era essa configuracao que fazia a medicao do ADR-0007 significar
    // alguma coisa enquanto o boot unico nao tivesse rodado. O boot unico
    // rodou em 22/08/2026, e o que o prova nao e mais uma configuracao do
    // firmware de hoje: e a captura `nvram-live-2026-08-22.txt`, escrita
    // **durante** aquele boot — que o teste ao lado fixa.
    //
    // A premissa deixou de ser necessaria, e deixou de ser sustentavel na
    // mesma medicao: **o ciclo de boot poe a entrada de volta na ordem.** O
    // ARCA nao a poe (C-5, e ha releitura no armar e no desarme), mas depois
    // de um backup ela esta la. Cobrar o contrario seria uma suite vermelha
    // por uma condicao que o ARCA nao controla e que e agora o estado normal.
    //
    // O que sobra e a invariante que importa, e ela vale nos dois casos: um
    // dispositivo **a frente da ordem** e **armado** e um backup que roda no
    // proximo reinicio sem ninguem pedir. A janela existe de verdade — vai do
    // fim da receita ao `arca resultado`, e em 22/08 durou oito minutos.
    // **`a_frente` e o primeiro lugar, e nao "esta na lista".** A revisao pegou
    // isto: com o dispositivo em segundo, atras do Windows, quem boota e o
    // Windows — e um dispositivo armado ali e o estado **normal** da janela
    // entre o `arca backup` e o reinicio. Cobrar inercia naquele caso deixaria
    // a suite vermelha acusando um perigo que nao existe.
    //
    // E a pergunta e sobre **o dispositivo**, e nao sobre a entrada chamada
    // `ARCA`: esta maquina tem duas entradas em `partition=R:` desde o marco, e
    // foi pela que o firmware criou que ela bootou. Quem decide o boot e para
    // onde a entrada aponta.
    let Some(leitura) = gerenciador() else {
        return;
    };
    let Ok(texto) = Bcdedit.enumerar("firmware") else {
        eprintln!("pulado: o bcdedit recusou o /enum firmware");
        return;
    };
    let Some(dispositivo) = dispositivo() else {
        return;
    };
    let Some(boot) = dispositivo.boot.as_ref().and_then(|boot| boot.letra) else {
        eprintln!("pulado: o ARCABOOT nao tem letra para conferir");
        return;
    };

    let entradas = firmware::ler(&texto).entradas;
    let leva_ao_dispositivo = |identificador: &String| {
        entradas
            .iter()
            .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador))
            .and_then(|entrada| entrada.alvo.as_ref())
            .and_then(|alvo| alvo.letra())
            .is_some_and(|letra| letra.eq_ignore_ascii_case(&boot))
    };

    let posicao = leitura
        .ordem_permanente
        .iter()
        .position(leva_ao_dispositivo);

    if posicao != Some(0) {
        // A configuracao de ate 22/08 (fora da ordem) e a de um dispositivo
        // atras do Windows. Nas duas, quem leva a maquina ao dispositivo e o
        // boot unico — e ele e o que o proprio ARCA arma e desarma.
        return;
    }

    let Some(caminho) = caminho_do_grub() else {
        return;
    };
    let corrente = ArquivosDoSistema
        .ler_texto(&caminho)
        .expect("o grub.cfg do dispositivo e legivel");

    let desarmado = grub::desarmar(&corrente).expect("o grub.cfg do dispositivo desarma");
    assert!(
        !desarmado.havia_receita(),
        "uma entrada que leva ao ARCABOOT ({boot}:) esta em **primeiro** na ordem permanente [{}] \
         e o grub.cfg do dispositivo esta armado. O proximo reinicio — venha de onde vier — boota \
         no dispositivo e roda a receita, sem boot unico nenhum e sem ninguem pedir. Rode \
         `arca resultado` para colher, ou `arca desarmar`. Ver ADR-0009",
        leitura.ordem_permanente.join(", ")
    );

    assert!(
        !leitura.tem_boot_unico(),
        "uma entrada que leva ao ARCABOOT ({boot}:) esta em primeiro na ordem permanente [{}] e \
         ainda ha boot unico armado apontando para [{}]. Ver ADR-0009",
        leitura.ordem_permanente.join(", "),
        leitura.boot_unico.join(", ")
    );
}

#[test]
fn a_invariante_do_teste_acima_reprovaria_um_dispositivo_armado() {
    // O teste acima so reprova quando o dispositivo esta **a frente da ordem e
    // armado**, e o segundo termo depende de uma condicao que nenhum teste
    // pode montar sem armar a maquina de quem o roda. Sem isto, a asserção que
    // importa nunca teria sido vista disparar — e uma asserção que nunca
    // disparou e uma suposicao com sintaxe de teste.
    //
    // O caso dificil aqui e o **armado de verdade**, e ele existe: a
    // `teste-03` e a unica das quatro capturas com `set default="arca-backup"`,
    // e a unica que provavelmente rodou desatendida (ADR-0007). E o arquivo
    // exato que a invariante tem de reconhecer como perigoso.
    const ARMADA: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg");

    let armada = grub::desarmar(ARMADA).expect("a captura armada desarma");
    assert!(
        armada.havia_receita(),
        "a `teste-03` esta armada e a invariante nao a reconheceu: o teste acima passaria verde \
         sobre um dispositivo que roda a receita no proximo reinicio"
    );

    // E o outro lado, para que "havia receita" nao seja uma resposta que se dá
    // a qualquer arquivo: o inerte deste dispositivo nao dispara.
    let inerte = grub::desarmar(INERTE).expect("o inerte desarma");
    assert!(
        !inerte.havia_receita(),
        "o grub.cfg inerte foi lido como armado, e o teste acima reprovaria sempre"
    );
}

#[test]
fn a_captura_do_live_mostra_o_boot_unico_funcionando_sobre_uma_entrada_de_tras() {
    // **P-18 fechada, fixada onde ela foi medida.** A E7 fixou o ADR-0007
    // contra a configuracao do firmware; este fixa o ADR-0009 contra a
    // captura, que e o que de fato prova.
    //
    // O `efibootmgr` que o Clonezilla roda sozinho ao salvar uma imagem
    // escreveu, **durante** o boot de 22/08/2026: `BootCurrent: 0001` com
    // `BootOrder: 0000,0001`. A maquina bootou pela entrada `0001` estando a
    // `0000` a frente. Nenhuma ordem permanente explica isso — o
    // `bootsequence` explica, e e a metade de P-18 que so o hardware
    // respondia.
    //
    // A mesma leitura, feita em 21/08, tras `BootOrder: 0001,0000` — o
    // dispositivo a frente —, e e por isso que aquele backup nao provava nada.
    // A diferenca entre as duas e a diferenca entre uma coincidencia e uma
    // medicao.
    const LIVE: &str = include_str!("../recursos/capturas/nvram-live-2026-08-22.txt");

    let valor = |chave: &str| {
        LIVE.lines()
            .find_map(|linha| linha.strip_prefix(chave))
            .map(str::trim)
            .unwrap_or_else(|| panic!("a captura do live tem `{chave}`"))
    };

    let bootou_por = valor("BootCurrent:");
    let ordem: Vec<&str> = valor("BootOrder:").split(',').map(str::trim).collect();

    assert!(
        ordem.contains(&bootou_por),
        "a entrada que bootou ({bootou_por}) nem esta na ordem {ordem:?} — a captura mudou de \
         forma, e a leitura precisa ser refeita antes de valer como evidencia"
    );
    assert_ne!(
        ordem.first().copied(),
        Some(bootou_por),
        "na captura do live a entrada que bootou ({bootou_por}) e a primeira da ordem {ordem:?}. \
         Uma ordem com o dispositivo a frente explica o boot inteiro sem passar por boot unico, e \
         esta captura deixa de ser a prova de P-18 que o §3.1 diz que ela e"
    );

    // E o que ela carrega e o bootloader do ARCABOOT, e nao o do Windows: sem
    // isso, "bootou por uma entrada de tras" poderia ser qualquer entrada.
    let linha_da_entrada = LIVE
        .lines()
        .find(|linha| linha.starts_with(&format!("Boot{bootou_por}")))
        .expect("a captura descreve a entrada que bootou");
    assert!(
        linha_da_entrada
            .to_ascii_uppercase()
            .contains("BOOTX64.EFI"),
        "a entrada que bootou nao carrega o bootloader do ARCABOOT: {linha_da_entrada}"
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

/// O `grub.cfg` do **zip** do Clonezilla 3.3.3-15, que o `arca prepare`
/// instala desde a E10.
const DO_PACOTE: &str = include_str!("../recursos/capturas/grub-clonezilla-do-pacote-3.3.3-15.cfg");

#[test]
fn o_grub_cfg_do_dispositivo_continua_inerte_e_e_um_dos_conhecidos() {
    // A E4 ja cobra isto, e a E7 o cobra de novo por um motivo proprio: ela e
    // a primeira etapa que **escreve** neste arquivo. A copia em
    // `recursos/capturas/` e a unica que existe fora do dispositivo, e ela tem
    // de continuar sendo o que estava la antes da primeira gravacao.
    //
    // **Desde a E10 ha dois inertes legitimos**, e este teste aceita os dois —
    // ver o irmao dele em `tests/e4_desarmar_o_dispositivo.rs` para por que. O
    // que importa aqui e a mesma coisa de sempre: o arquivo **nao esta
    // armado**, e e um dos que este repositorio conhece.
    let Some(caminho) = caminho_do_grub() else {
        return;
    };

    let corrente = ArquivosDoSistema
        .ler_texto(&caminho)
        .expect("o grub.cfg do dispositivo e legivel");

    let do_pacote_inerte = arca::grub::desarmar(DO_PACOTE)
        .expect("o grub.cfg do pacote se desarma")
        .texto;

    let qual = if corrente == INERTE {
        "o do ISO, preparado a mao"
    } else if corrente == do_pacote_inerte {
        "o do zip, instalado pelo `arca prepare`"
    } else {
        // A mensagem **não** despeja os dois arquivos: são 11 KB cada, e um
        // `assert_eq!` aqui enche a tela de quem roda a suíte com algo que não
        // se lê. O que importa é o veredito e o que fazer.
        panic!(
            "o grub.cfg de {} nao e nenhum dos dois inertes conhecidos. Se foi um \
             `arca backup` que o armou, `arca desarmar` o devolve; se foi outra coisa, \
             a copia precisa ser refeita antes de continuar valendo como evidencia. \
             Para ver a diferenca: `diff` entre ele e \
             recursos/capturas/grub-inerte-arcaboot.cfg",
            caminho.display()
        )
    };

    eprintln!("  (o grub.cfg de {} e {qual})", caminho.display());
}

/// Os três comandos que armam avisam o que vai aparecer depois do reinício.
///
/// Nasceu de uma operação real desligada no meio, em 23/08/2026: o menu do
/// Clonezilla apareceu, quem estava na frente da tela viu que não era o
/// Windows e desligou. **Não havia defeito** — o `grub.cfg` tem
/// `set timeout="30"`, e o `set default` escolhe qual entrada boota sem tirar
/// a espera. O que faltava era a tela dizer isso.
///
/// O teste mora aqui, e não em cada comando, porque a invariante é sobre os
/// **três**: quem acrescentar um quarto comando que arma vai encontrá-lo.
#[test]
fn todo_comando_que_arma_avisa_o_que_vem_depois_do_reinicio() {
    let aviso = arca::armar::montar_o_que_vem_pela_frente();

    // O oráculo é o código-fonte de cada comando: o que se cobra é que os três
    // chamem a mesma função, e não que tenham textos parecidos. Duas cópias
    // divergiriam na primeira mudança, e a que divergisse passaria a prometer
    // uma espera que não é a que existe.
    for (comando, fonte) in [
        ("backup", include_str!("../src/comandos/backup.rs")),
        ("restore", include_str!("../src/comandos/restore.rs")),
        ("verify", include_str!("../src/comandos/verify.rs")),
    ] {
        assert!(
            fonte.contains("montar_o_que_vem_pela_frente()"),
            "`arca {comando}` arma e nao avisa o que vai aparecer na tela"
        );
    }

    assert!(aviso.contains("30 SEGUNDOS"));
    assert!(aviso.contains("NAO DESLIGUE"));

    // **E o `prepare` não arma, o que é a outra metade da invariante.**
    //
    // Ele entrou na E10 e é destrutivo, então a pergunta natural ao lê-lo é
    // *"por que ele não avisa o que vem depois do reinício?"* — a resposta é
    // que não há reinício: ele é o único comando destrutivo do ARCA que faz
    // tudo do lado Windows, com a tela na frente.
    //
    // Cobrar isso aqui é o que impede que um dia alguém o faça armar e o
    // deixe sem o aviso. As duas asserções falam da mesma coisa por lados
    // opostos: quem chama `armar::executar` tem de chamar o aviso, e quem não
    // chama nenhum dos dois continua não chamando.
    let prepare = include_str!("../src/comandos/prepare.rs");
    assert!(
        !prepare.contains("armar::executar"),
        "`arca prepare` passou a armar, e nao avisa o que vai aparecer na tela do outro lado do reinicio"
    );
    assert!(
        !prepare.contains("sistema.reiniciar()"),
        "`arca prepare` passou a reiniciar a maquina, e a tela dele nao diz o que vem depois"
    );
}
