//! A etapa E10 contra o que foi medido, e contra o hardware desta mesa.
//!
//! Três famílias de teste, e as três têm oráculo fora deste repositório:
//!
//! - **o `grub.cfg` do pacote**, que é o arquivo que o `arca prepare` vai
//!   instalar. Ele não é o do dispositivo desta mesa, e a diferença está
//!   medida;
//! - **a medição do particionamento**, feita à mão em 23/08/2026 antes de o
//!   código existir;
//! - **a criação da entrada de firmware**, medida no mesmo dia, com a entrada
//!   apagada no fim e o firmware voltando ao que era.
//!
//! Nenhum deles pode ser ajustado para passar: o alvo é sempre um arquivo que
//! outra ferramenta escreveu.

use arca::grub;
use arca::menuentry;
use arca::nome::Nome;
use arca::pacote;
use arca::preparacao;
use arca::receita::{Disco, Operacao, Pedido, Receita, Selo};

/// O `boot/grub/grub.cfg` de dentro do `clonezilla-live-3.3.3-15-amd64.zip`,
/// baixado do SourceForge em 23/08/2026 e extraído com o `bsdtar` do
/// `System32`.
const DO_PACOTE: &str = include_str!("../recursos/capturas/grub-clonezilla-do-pacote-3.3.3-15.cfg");

/// O `grub.cfg` que o Clonezilla entrega, preservado do **dispositivo desta
/// mesa** desde a E4.
const DO_DISPOSITIVO: &str = include_str!("../recursos/capturas/grub-clonezilla-original.cfg");

/// O `grub.cfg` inerte do dispositivo desta mesa — o oráculo da E4.
const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");

/// O `CHECKSUMS.TXT` que o mirror do projeto publica.
const CHECKSUMS: &str = include_str!("../recursos/capturas/clonezilla-checksums-2026-08-23.txt");

// ─────────────── o pacote é o que o ARCA vai instalar ───────────────

#[test]
fn o_sha256_compilado_e_o_que_o_projeto_publica() {
    // **O teste que PR-1 existe para tornar possível.** O número que o ARCA
    // carrega dentro de si tem de ser o que o Clonezilla publica — e o
    // `CHECKSUMS.TXT` aqui veio do mirror do projeto, servidor diferente do
    // que serve o arquivo.
    //
    // Ele não pode ser ajustado para passar: mudar a constante sem trocar a
    // captura quebra isto, e trocar a captura é trocar um arquivo que outra
    // gente escreveu.
    let linha = format!("{}  {}", pacote::SHA256, pacote::ARQUIVO);

    assert!(
        CHECKSUMS.contains(&linha),
        "o SHA256 compilado no ARCA nao e o que o projeto publica para o `{}`",
        pacote::ARQUIVO
    );
}

#[test]
fn o_sha256_esta_na_secao_certa_do_checksums() {
    // O `CHECKSUMS.TXT` traz **seis** algoritmos, e o SHA256 e o B3SUM tem os
    // mesmos 64 dígitos hexadecimais. Casar só a linha deixaria um B3SUM
    // passar por SHA256 — e o `certutil` não sabe B3, então a conferência
    // reprovaria todo download com uma mensagem sobre o arquivo estar errado.
    let secao_do_sha256 = CHECKSUMS
        .split("### ")
        .find(|bloco| bloco.starts_with("SHA256SUMS:"))
        .expect("o CHECKSUMS.TXT tem a secao SHA256SUMS");

    assert!(
        secao_do_sha256.contains(pacote::SHA256),
        "o numero compilado no ARCA nao esta na secao SHA256SUMS"
    );

    // E o B3SUM do mesmo arquivo, que tem o mesmo comprimento, é outro número.
    let b3 = CHECKSUMS
        .split("### ")
        .find(|bloco| bloco.starts_with("B3SUMS:"))
        .expect("o CHECKSUMS.TXT tem a secao B3SUMS");
    assert!(
        !b3.contains(pacote::SHA256),
        "o numero compilado casa com o B3SUM, e nao com o SHA256"
    );
}

// ─────────────── o `grub.cfg` do pacote não é o desta mesa ───────────────

#[test]
fn o_pacote_e_o_dispositivo_diferem_no_noeject_e_em_seis_segundos() {
    // **O achado da E10**, e ele explica de onde veio o dispositivo desta mesa.
    //
    // As duas únicas diferenças entre o `grub.cfg` do zip e o do dispositivo
    // são o `noeject` — treze vezes, uma por `menuentry` — e o carimbo de hora
    // do rodapé, que difere em **seis segundos**. Seis segundos é o
    // `ocs-live-dev` gerando o ISO e o zip na mesma execução: é a mesma build,
    // e o dispositivo veio do ISO.
    let do_pacote = sem_cr(DO_PACOTE);
    let do_dispositivo = sem_cr(DO_DISPOSITIVO);

    assert_eq!(
        do_pacote.lines().count(),
        do_dispositivo.lines().count(),
        "os dois tem de ter o mesmo numero de linhas"
    );

    assert_eq!(do_pacote.matches("noeject").count(), 13);
    assert_eq!(do_dispositivo.matches("noeject").count(), 0);

    // Tirado o `noeject`, sobra **uma** linha diferente: a do carimbo.
    let sem_noeject = do_pacote.replace("enforcing=0 noeject ", "enforcing=0 ");
    let divergentes: Vec<(&str, &str)> = sem_noeject
        .lines()
        .zip(do_dispositivo.lines())
        .filter(|(a, b)| a != b)
        .collect();

    assert_eq!(
        divergentes.len(),
        1,
        "sobrou mais do que o carimbo: {divergentes:?}"
    );
    assert!(
        divergentes[0].0.contains("Created at time")
            && divergentes[0].0.contains("04:11:28")
            && divergentes[0].1.contains("04:11:22"),
        "a unica diferenca que sobra tem de ser o carimbo: {:?}",
        divergentes[0]
    );
}

#[test]
fn o_pacote_traz_a_versao_que_o_arca_fixa() {
    // O `hostname=cl-<versao>` é o que liga o pacote à versão fixada, e ele
    // aparece na linha de comando de cada `menuentry`. Um pacote de outra
    // versão passaria pelo SHA256 apenas se as duas constantes mudassem
    // juntas; este teste é a terceira ponta.
    assert!(DO_PACOTE.contains(&format!("hostname=cl-{}", pacote::VERSAO)));
}

// ─────────────── o desarmar sobre o pacote (§4.4, ADR-0005) ───────────────

#[test]
fn o_grub_do_pacote_nao_esta_inerte_e_o_desarmar_o_deixa() {
    // **Uma peça nova encaixada numa peça antiga**, que é o padrão que este
    // projeto revisa a cada etapa. O `grub::desarmar` nasceu na E4 e nunca
    // tinha visto o `grub.cfg` do pacote — ele só conhecia o do dispositivo,
    // que já vinha com `set default="live-default"`.
    //
    // O pacote vem com `set default="0"`, que o ADR-0005 nomeia: `"0"` aponta
    // por **posição**, e a posição muda quando o bloco do ARCA entra antes do
    // `live-default`. Um dispositivo entregue assim ficaria armado no instante
    // em que o primeiro `arca backup` inserisse o bloco, **sem que ninguém
    // tocasse no `set default`**.
    assert!(
        DO_PACOTE.contains(r#"set default="0""#),
        "o pacote devia vir com o `set default` por posicao"
    );

    let desarmado = grub::desarmar(DO_PACOTE).expect("o grub.cfg do pacote se desarma");

    assert!(
        desarmado.default_devolvido,
        "o desarmar tinha de mexer no `set default` deste arquivo"
    );
    assert_eq!(
        desarmado.blocos_removidos, 0,
        "o pacote nao tem bloco do ARCA"
    );
    assert!(desarmado.texto.contains(r#"set default="live-default""#));
    assert!(!desarmado.texto.contains(r#"set default="0""#));
}

#[test]
fn o_desarmar_do_pacote_produz_o_inerte_desta_mesa_menos_o_noeject() {
    // O oráculo mais forte desta etapa: desarmar o `grub.cfg` do pacote produz
    // **exatamente** o `grub.cfg` inerte que está no dispositivo desta mesa, a
    // menos das duas diferenças de origem já medidas.
    //
    // Isso é o que prova que um dispositivo preparado pelo `arca prepare` fica
    // no mesmo estado que o dispositivo com que este projeto rodou os quatro
    // marcos em hardware.
    let desarmado = grub::desarmar(DO_PACOTE).expect("desarma");

    let saiu = sem_cr(&desarmado.texto)
        .replace("enforcing=0 noeject ", "enforcing=0 ")
        .replace("04:11:28", "04:11:22");

    assert_eq!(
        saiu,
        sem_cr(INERTE),
        "o dispositivo preparado nao sai igual ao desta mesa"
    );
}

#[test]
fn o_desarmar_do_pacote_e_idempotente() {
    // C-1 sai de graça por reconstruir do arquivo corrente (ADR-0005), e vale
    // aqui também: o `arca prepare` desarma uma vez, e um `arca backup` logo
    // depois desarma de novo como primeiro passo.
    let uma = grub::desarmar(DO_PACOTE).expect("desarma").texto;
    let duas = grub::desarmar(&uma).expect("desarma de novo");

    assert_eq!(duas.texto, uma);
    assert!(!duas.default_devolvido, "a segunda passada mexeu em algo");
}

#[test]
fn o_bloco_do_arca_deriva_do_live_toram_do_pacote() {
    // A outra peça antiga: `menuentry::derivar` (E7, ADR-0007) copia o
    // `menuentry --id live-toram` do arquivo corrente. Num dispositivo
    // preparado pelo `arca prepare`, o arquivo corrente é o do **pacote** — e
    // ele tem o `live-toram`, com o `noeject` junto.
    //
    // É exatamente o argumento do ADR-0007 visto pelo outro lado: derivar em
    // vez de transcrever é o que faz o `noeject` viajar sem que ninguém
    // precise saber que ele existe.
    let inerte = grub::desarmar(DO_PACOTE).expect("desarma").texto;

    // A receita de verdade, montada como o `arca backup` a monta — e não uma
    // string inventada: o que se quer provar é que o caminho inteiro funciona
    // sobre o `grub.cfg` do pacote.
    let receita = Receita::montar(&Pedido {
        operacao: Operacao::Backup,
        nome: Some(Nome::novo("2026-08-23_Novo").expect("nome valido por B-2")),
        disco: Some(Disco::novo("nvme0n1").expect("nome de disco valido")),
        selo: Selo::de_ensaio(),
    })
    .expect("a receita passa por C-2");

    let bloco = menuentry::derivar(&inerte, receita.parametros())
        .expect("o pacote tem o menuentry --id live-toram");

    assert!(bloco.contains("--id arca-backup"), "{bloco}");
    assert!(
        bloco.contains("toram=live,syslinux,EFI,boot,.disk,utils"),
        "{bloco}"
    );
    assert!(
        bloco.contains("noeject"),
        "o `noeject` do pacote tem de viajar para o bloco do ARCA:\n{bloco}"
    );
    assert!(
        bloco.contains("hostname=cl-3.3.3-15"),
        "a configuracao daquele dispositivo viaja junto:\n{bloco}"
    );
}

#[test]
fn o_noeject_cabe_no_orcamento_da_linha_do_kernel() {
    // §10.2.3: o `COMMAND_LINE_SIZE` do x86_64 é 2048, e estourá-lo faz o
    // kernel **truncar em silêncio** — o caso do §3.2, em que o Clonezilla
    // descarta a receita e abre o menu.
    //
    // O `menuentry` base deste dispositivo mede 471 bytes e a reserva é 512.
    // O `noeject ` do pacote acrescenta oito, e o teste guarda a folga que
    // sobra: um dia alguém vai acrescentar outro parâmetro.
    let inerte = grub::desarmar(DO_PACOTE).expect("desarma").texto;

    let base = inerte
        .lines()
        .find(|linha| linha.contains("$linux_cmd") && linha.contains("toram="))
        .expect("a linha do live-toram")
        .trim();

    const RESERVA: usize = 512;
    assert!(
        base.len() <= RESERVA,
        "o `menuentry` base do pacote ocupa {} dos {RESERVA} reservados",
        base.len()
    );
}

// ─────────────── o particionamento medido à mão ───────────────

/// A medição de 23/08/2026, feita **antes** de o código existir.
const MEDICAO: &str = include_str!("../recursos/capturas/medicao-particionamento-2026-08-23.txt");

/// O marco em hardware de 25/08/2026: as nove etapas que mediram o GPT.
///
/// É o oráculo do [ADR-0025](../docs/adr/0025-o-arca-particiona-em-gpt.md), e o
/// que estes testes fazem é impedir que as constantes do código andem sem que
/// alguém volte ao hardware.
const MEDICAO_GPT: &str = include_str!("../recursos/capturas/medicao-gpt-2026-08-25.txt");

#[test]
fn a_estrutura_que_o_arca_transcreve_e_a_que_foi_medida() {
    // O que a medição de 25/08/2026 registra do dispositivo que **bootou**. A
    // constante de tipo, a unidade de alocação e o tamanho do ARCABOOT são
    // constantes deste projeto porque são **medidas**, e este teste é o que as
    // liga ao arquivo.
    assert!(
        MEDICAO_GPT.contains(&format!(
            "GptType         : {}",
            preparacao::TIPO_GPT_DADOS_BASICOS
        )),
        "o GptType das duas particoes"
    );
    assert!(
        MEDICAO_GPT.contains(&format!(
            "AllocationUnitSize : {}",
            preparacao::UNIDADE_DE_ALOCACAO
        )),
        "a unidade de alocacao"
    );
    assert!(
        MEDICAO_GPT.contains(&format!("Size            : {}", preparacao::ARCABOOT_BYTES)),
        "o tamanho do ARCABOOT"
    );
    assert!(
        MEDICAO_GPT.contains("PartitionStyle    : GPT"),
        "o esquema e GPT, e nao MBR (ADR-0025)"
    );
}

#[test]
fn o_gpttype_nao_distingue_as_duas_particoes() {
    // **O achado que muda o critério da releitura.** Em MBR, `7` e `12`
    // diziam qual partição era qual. Em GPT as duas nascem com o mesmo tipo, e
    // quem diz qual é qual passa a ser o rótulo, o sistema de arquivos e a
    // ordem no disco.
    //
    // O teste conta as ocorrências na releitura do dispositivo que bootou: têm
    // de ser duas, e do mesmo GUID.
    let releitura = MEDICAO_GPT
        .split("### ETAPA 4 (SSD) — RELEITURA")
        .nth(1)
        .expect("a medicao tem a releitura do dispositivo que bootou");

    let iguais = releitura
        .matches(&format!(
            "GptType         : {}",
            preparacao::TIPO_GPT_DADOS_BASICOS
        ))
        .count();

    assert_eq!(
        iguais, 2,
        "as duas particoes tem de sair com o MESMO GptType:\n{releitura}"
    );
}

#[test]
fn o_gpttype_sai_do_new_partition_e_o_format_volume_nao_encosta_nele() {
    // **A medição que muda o desenho, e ela é o inverso do MBR.** Lá o
    // `New-Partition` criava as duas com `MbrType 6` e quem acertava para 7 e
    // 12 era o `Format-Volume` — o tipo era efeito colateral de outra
    // operação. Em GPT o tipo já sai pronto da criação.
    //
    // A releitura de PR-5 continua importando, e por outro motivo: ela é o que
    // pega uma ESP ou uma MSR no lugar do que se pediu.
    let antes = MEDICAO_GPT
        .split("### ETAPA 4 — GptType ANTES de formatar")
        .nth(1)
        .and_then(|resto| resto.split("### ETAPA 4 — Format-Volume 1").next())
        .expect("a medicao tem o GptType de antes de formatar");

    let depois = MEDICAO_GPT
        .split("### ETAPA 4 — GptType DEPOIS de formatar")
        .nth(1)
        .and_then(|resto| resto.split("### ETAPA 4 — Atribuindo letras").next())
        .expect("a medicao tem o GptType de depois de formatar");

    let alvo = format!("GptType         : {}", preparacao::TIPO_GPT_DADOS_BASICOS);

    assert_eq!(
        antes.matches(&alvo).count(),
        2,
        "as duas ja saem do New-Partition com o tipo final:\n{antes}"
    );
    assert_eq!(
        depois.matches(&alvo).count(),
        2,
        "e o Format-Volume nao encostou nele:\n{depois}"
    );
}

#[test]
fn a_msr_existe_e_e_por_isso_que_o_script_a_remove() {
    // Em MBR o `Initialize-Disk` deixa o disco vazio. Em GPT ele cria sozinho
    // uma Microsoft Reserved — medido nos **dois** dispositivos do marco, com
    // os mesmos três números. É o que justifica a linha do `Remove-Partition`
    // ser um passo, e não uma condicional (ADR-0025).
    assert!(
        MEDICAO_GPT.contains(&format!("GptType         : {}", preparacao::TIPO_GPT_MSR)),
        "a MSR que o Initialize-Disk cria"
    );
    assert!(
        MEDICAO_GPT.contains("Offset          : 17408"),
        "o offset da MSR"
    );
    assert_eq!(
        MEDICAO_GPT
            .matches(&format!("GptType         : {}", preparacao::TIPO_GPT_MSR))
            .count(),
        2,
        "a MSR apareceu nos dois dispositivos, e e isso que a torna comportamento e nao acidente"
    );
    assert!(
        MEDICAO_GPT.contains("particoes restantes: 0"),
        "e ela foi removida, deixando o disco vazio como o MBR deixava"
    );
}

#[test]
fn o_dispositivo_gpt_bootou_e_o_device_path_esta_lido_de_dentro_do_boot() {
    // **A Etapa 7, que é a que decide**, e a Etapa 8, que é a que prova. O
    // ADR-0014 dizia que a falha de um dispositivo GPT "só se descobre depois
    // de o Windows já ter sido apagado"; este arquivo é a medição que mostra
    // que não — e o `efibootmgr`, lido de dentro do live, é a evidência da
    // mesma qualidade que as seis leituras de NVRAM do ADR-0023.
    const DE_DENTRO_DO_BOOT: &str =
        include_str!("../recursos/capturas/efibootmgr-gpt-2026-08-25.txt");

    assert!(
        DE_DENTRO_DO_BOOT.contains("HD(2,GPT,9c86b84a-596f-47e6-b92a-cd5b84b4a1fe,"),
        "o device path em GPT, com o PARTUUID da particao no lugar da assinatura do disco"
    );
    assert!(
        DE_DENTRO_DO_BOOT.contains(r"\EFI\BOOT\BOOTX64.EFI"),
        "e o caminho do que o firmware carregou"
    );

    // O par de números do ADR-0023, e ele é o mesmo em GPT: bootou pela
    // entrada do dispositivo com o Windows à frente da ordem permanente. C-5
    // não custa nada aqui.
    assert!(
        DE_DENTRO_DO_BOOT.contains("BootCurrent: 0001"),
        "bootou pela segunda entrada"
    );
    assert!(
        DE_DENTRO_DO_BOOT.contains("BootOrder: 0000,0001"),
        "com o Windows a frente da ordem"
    );

    // E o PARTUUID do device path é o da ARCABOOT, e não de outra partição —
    // é o `blkid` do mesmo arquivo que fecha essa amarração.
    assert!(
        DE_DENTRO_DO_BOOT.contains(r#"LABEL="ARCABOOT""#),
        "o blkid nomeia a ARCABOOT"
    );
    let linha_da_arcaboot = DE_DENTRO_DO_BOOT
        .lines()
        .find(|linha| linha.contains(r#"LABEL="ARCABOOT""#))
        .expect("a linha do blkid da ARCABOOT");
    assert!(
        linha_da_arcaboot.contains("9c86b84a-596f-47e6-b92a-cd5b84b4a1fe"),
        "o GUID do device path e o PARTUUID da ARCABOOT: {linha_da_arcaboot}"
    );
}

#[test]
fn nenhuma_particao_sai_ativa() {
    // A captura da estrutura registra `IsActive: False` nas duas, e é isso que
    // confirma que o boot do dispositivo é **UEFI puro, e não BIOS**. A
    // medição do particionamento reproduziu o mesmo sem que ninguém pedisse.
    let releitura = MEDICAO
        .split("### RELEITURA — Get-Partition")
        .nth(1)
        .expect("a medicao tem a releitura");

    assert!(
        !releitura.contains("IsActive        : True"),
        "alguma particao saiu ativa:\n{releitura}"
    );
    assert_eq!(
        releitura.matches("IsActive        : False").count(),
        2,
        "as duas particoes tem de estar na releitura"
    );
}

#[test]
fn as_duas_reguas_do_adr_0010_aparecem_no_segundo_disco_tambem() {
    // O `MSFT_Disk` e o `Win32_DiskDrive` dão dois tamanhos para o **mesmo**
    // disco, e a diferença é o produto da geometria CHS legada truncado no
    // último cilindro (ADR-0010). Medido no `KINGSTON` na E9; medido de novo
    // aqui, no `JMicron`.
    //
    // O teste existe porque a régua errada é uma armadilha que já custou uma
    // etapa, e ela não é do NVMe — é do Windows.
    const MSFT: u64 = 480_103_981_056;
    const WIN32: u64 = 480_101_368_320;

    assert!(MEDICAO.contains(&MSFT.to_string()), "o Get-Disk");
    assert!(MEDICAO.contains(&WIN32.to_string()), "o Win32_DiskDrive");

    // E o número menor é exatamente o produto CHS: 58369 × 255 × 63 × 512.
    assert_eq!(WIN32, 58_369 * 255 * 63 * 512);
    assert_eq!(MSFT - WIN32, 2_612_736);
}

// ─────────────── a criação da entrada de firmware ───────────────

/// A medição de 23/08/2026: criar, apontar, tirar da ordem e apagar.
const CRIACAO: &str =
    include_str!("../recursos/capturas/medicao-criacao-de-entrada-parte2-2026-08-23.txt");

/// A primeira metade, que mediu o que o `/copy` faz com a ordem permanente.
const CRIACAO_1: &str =
    include_str!("../recursos/capturas/medicao-criacao-de-entrada-2026-08-23.txt");

#[test]
fn a_entrada_criada_sai_identica_a_que_ja_existia() {
    // **O que fecha o que a E7 deixou em aberto.** C-4 diz que armar não cria
    // entrada porque "criar uma do zero é código sem original — nenhuma
    // captura mostra a forma". Esta captura mostra.
    //
    // A entrada criada por `/copy {bootmgr}` mais dois `/set` é, campo a
    // campo, a entrada `ARCA` que esta máquina já tinha. As duas só divergem
    // no `identificador` e na `description`.
    let criada = bloco_do_bcdedit(CRIACAO, "### RELEITURA (C-3) — a entrada pronta");
    let existente = bloco_do_bcdedit(CRIACAO, "### A entrada ARCA que ja existia, para comparar");

    // Fora da comparação: o identificador (que é diferente por definição), a
    // descrição (que é o que se pediu no `/d`), e a linha `> bcdedit ...` que a
    // própria medição registrou — ela carrega o GUID e não é resposta do
    // `bcdedit`.
    let normalizar = |bloco: &str| -> Vec<String> {
        bloco
            .lines()
            .map(str::trim)
            .filter(|linha| {
                !linha.is_empty()
                    && !linha.starts_with("identificador")
                    && !linha.starts_with("description")
                    && !linha.starts_with("> bcdedit")
            })
            .map(str::to_string)
            .collect()
    };

    assert_eq!(
        normalizar(&criada),
        normalizar(&existente),
        "a entrada criada nao sai igual a que ja existia"
    );

    // E as duas apontam para o mesmo lugar.
    assert!(criada.contains("device                  partition=R:"));
    assert!(criada.contains(r"path                    \EFI\boot\bootx64.efi"));
}

#[test]
fn o_copy_poe_a_entrada_nova_na_ordem_permanente_sozinho() {
    // **O achado que ninguém tinha previsto**, e é o que faz o `arca prepare`
    // tirar a entrada da ordem: `bcdedit /copy` a acrescenta ao `displayorder`
    // sem que ninguém peça.
    //
    // Isso é exatamente o perigo que C-5 nomeia — o ARCA acrescentar um
    // caminho permanente para bootar no dispositivo.
    let antes = bloco_do_bcdedit(CRIACAO_1, "### ANTES — o gerenciador de firmware");
    let depois = bloco_do_bcdedit(CRIACAO_1, "### DEPOIS DE CRIAR — o gerenciador de firmware");

    assert_eq!(
        antes.matches("{").count() - antes.matches("{fwbootmgr}").count(),
        1,
        "antes havia so o {{bootmgr}} na ordem:\n{antes}"
    );
    assert!(
        depois.contains("f4057bd1"),
        "a entrada criada tinha de estar na ordem depois do /copy:\n{depois}"
    );
}

#[test]
fn o_remove_tira_da_ordem_e_a_entrada_sobrevive() {
    // O outro lado: `/set {fwbootmgr} displayorder {novo} /remove` tira a
    // entrada da ordem **sem apagar o objeto** — que é o que o boot único
    // precisa que continue existindo.
    //
    // E tirar não quebra nada: o `bootsequence` funciona sobre uma entrada que
    // não está no `displayorder`, medido na E7 e exercitado no marco de 22/08
    // (ADR-0007).
    const MEDICAO_DA_ORDEM: &str =
        include_str!("../recursos/capturas/medicao-letras-e-ordem-2026-08-23.txt");

    let depois = bloco_do_bcdedit(
        MEDICAO_DA_ORDEM,
        "### DEPOIS DE TIRAR — o gerenciador de firmware",
    );
    assert!(
        !depois.contains("f4057bd2"),
        "a entrada continua na ordem:\n{depois}"
    );

    let sobreviveu = bloco_do_bcdedit(
        MEDICAO_DA_ORDEM,
        "### DEPOIS DE TIRAR — a entrada sobreviveu?",
    );
    assert!(
        sobreviveu.contains("ARCA-MEDICAO-ORDEM"),
        "o objeto tinha de sobreviver ao /remove:\n{sobreviveu}"
    );

    // E a segunda passada não muda nada: `/remove` é idempotente.
    assert_eq!(
        MEDICAO_DA_ORDEM
            .matches("### TIRAR de novo — e idempotente?")
            .count(),
        1
    );
}

#[test]
fn o_firmware_voltou_ao_que_era_depois_da_medicao() {
    // A medição criou uma entrada de boot nesta máquina e a apagou. Este teste
    // é o que atesta que ela não sobrou — e é o mesmo cuidado que o ADR-0013
    // teve ao medir `/addfirst` e conferir a NVRAM byte a byte no fim.
    assert!(CRIACAO.contains("a entrada de medicao sumiu: True"));
    assert!(CRIACAO.contains("o displayorder tem so o {bootmgr}: True"));
}

#[test]
fn a_letra_nao_e_escolhida_e_o_motivo_esta_medido() {
    // `Set-Partition -NewDriveLetter C` responde *"The requested access path is
    // already in use"*. Escolher a letra é supor que ela está livre — e S-3 diz
    // que a letra não importa, o rótulo importa.
    //
    // E o `Add-PartitionAccessPath -AssignDriveLetter` **não é idempotente**:
    // a segunda passada recusa e **não muda nada**. É o caso do `bcdedit
    // /deletevalue` do ADR-0005: manda fazer, descarta o que a ferramenta
    // responde, e pergunta de novo.
    const MEDICAO_DA_ORDEM: &str =
        include_str!("../recursos/capturas/medicao-letras-e-ordem-2026-08-23.txt");

    assert!(
        MEDICAO_DA_ORDEM.contains("The requested access path is already in use"),
        "a medicao da letra escolhida"
    );
    assert!(
        MEDICAO_DA_ORDEM.contains("Cannot assign multiple drive letters to a partition"),
        "a medicao da segunda passada"
    );

    // E depois das duas passadas as letras continuam as mesmas.
    let final_ = MEDICAO_DA_ORDEM
        .split("### depois da segunda passada")
        .nth(1)
        .expect("a medicao tem a leitura final");
    assert!(final_.contains("DriveLetter     : E"));
    assert!(final_.contains("DriveLetter     : F"));
}

// ─────────────── o marco em hardware de 23/08/2026 ───────────────

/// A tela do primeiro `arca prepare`, rodado de verdade sobre o segundo
/// dispositivo desta mesa.
const MARCO: &str = include_str!("../recursos/capturas/arca-prepare-2026-08-23-marco.txt");

/// A segunda metade do marco: a entrada de firmware **criada do zero** pelo
/// ARCA, e o `--iso` de PR-2.
const MARCO_2: &str = include_str!("../recursos/capturas/arca-prepare-2026-08-23-com-iso.txt");

/// O firmware lido antes e depois da criação.
const FIRMWARE_DO_MARCO: &str =
    include_str!("../recursos/capturas/arca-prepare-2026-08-23-criacao-da-entrada.txt");

/// A tela do `arca prepare` rodado **de verdade** com o código GPT, em
/// 25/08/2026, sobre o KGSSE100 desta mesa.
///
/// É o segundo marco de execução deste projeto, e ele existe por uma distinção
/// que quase passou: o marco em GPT do mesmo dia foi feito **à mão**, com
/// PowerShell, e provou que a *estrutura* boota. Não provava que o *código que
/// a produz* funciona — as três capturas de execução do `arca prepare` eram
/// todas de 23/08, de quando o código era MBR.
const MARCO_GPT: &str = include_str!("../recursos/capturas/arca-prepare-2026-08-25-marco-gpt.txt");

#[test]
fn o_marco_em_gpt_rodou_pelo_codigo_e_nao_a_mao() {
    // O que a execução real escreveu na tela, e cada linha é uma coisa que só
    // o código faz — o `Remove-Partition` da MSR, o `GptType` relido, a
    // entrada de firmware criada do zero e tirada da ordem.
    assert!(
        MARCO_GPT
            .contains("Particionando ................... ok · GPT, 2 particoes de dados basicos"),
        "a linha do particionamento em GPT"
    );
    assert!(
        MARCO_GPT.contains(preparacao::TIPO_GPT_DADOS_BASICOS),
        "e o GptType relido do disco, na tela"
    );
    assert!(
        MARCO_GPT.contains("sem a MSR que o Windows cria"),
        "a MSR foi removida pelo codigo, e a tela diz"
    );
    assert!(
        MARCO_GPT.contains("nenhuma particao ativa, unidade 4096 (C-3)"),
        "a releitura de PR-5"
    );
    assert!(
        MARCO_GPT.contains("Entrada de firmware ............. criada · ARCA ·"),
        "a entrada nasceu do `/copy`, e a tela distingue `criada` de `reusada`"
    );
    assert!(
        MARCO_GPT.contains("a entrada saiu da ordem permanente"),
        "C-5: o `bcdedit /copy` a poe no displayorder e o ARCA a tira"
    );
    assert!(MARCO_GPT.contains("Dispositivo pronto."), "e terminou");

    // E o que ele **não** conseguiu conferir sozinho continua dito na tela, em
    // vez de ficar implícito: quem responde se o dispositivo boota é o boot.
    assert!(
        MARCO_GPT.contains("se este dispositivo boota mesmo"),
        "a tela nao promete o que o comando nao mediu (P-26)"
    );
}

#[test]
fn o_marco_produziu_a_estrutura_transcrita() {
    // O `arca prepare` rodou em hardware em 23/08/2026 e a releitura de PR-5
    // saiu com os dois tipos MBR da captura — o 7 e o 12 — e sem partição
    // ativa. É a transcrição do ADR-0014 provada pelo lado da execução, e não
    // só pelo do teste.
    //
    // **Esta captura é histórica, e o texto abaixo não é o que o comando diz
    // hoje.** Em 25/08/2026 o ADR-0025 trocou o esquema por GPT, e a linha
    // passou a falar de dados básicos e da MSR removida. O teste continua aqui
    // porque o que ele guarda é que o marco em MBR **aconteceu** — e é contra
    // ele que o marco em GPT se compara.
    assert!(
        MARCO.contains("Particionando ................... ok · MBR, 2 particoes · MbrType 7 e 12"),
        "a linha do particionamento do marco de 23/08, que e historia e nao a tela de hoje"
    );
    assert!(
        MARCO.contains("nenhuma particao ativa, unidade 4096 (C-3)"),
        "a releitura do marco"
    );
}

#[test]
fn o_marco_desarmou_o_grub_que_o_pacote_entrega() {
    // §4.4 e ADR-0005: o pacote vem com `set default="0"`, que **não** é o
    // estado inerte. O `arca prepare` desarma o que acabou de instalar, e a
    // tela do marco registra que isso aconteceu de verdade.
    assert!(
        MARCO.contains(r#"o `set default` do pacote era "0", e voltou para `live-default`"#),
        "o marco tinha de mostrar o desarmar do pacote"
    );
}

#[test]
fn a_criacao_da_entrada_rodou_pelo_arca_e_nao_so_a_mao() {
    // **O que a E7 mandou para cá, fechado.** C-4 recusava criar entrada de
    // firmware ao armar porque "criar uma do zero é código sem original", e
    // dizia que o lugar era o `arca prepare`.
    //
    // A medição de 23/08 deu o original do **artefato**; este marco deu o do
    // **código**: a entrada foi criada pelo ARCA, com o identificador achado
    // pela forma na resposta traduzida do `bcdedit`.
    //
    // O primeiro `prepare` do mesmo dia **reusou** a entrada que já existia
    // (C-4), e por isso este segundo existiu: sem apagar a antiga, o caminho
    // da criação não seria exercitado.
    assert!(
        MARCO_2.contains("Entrada de firmware ............. criada · ARCA · {f4057bd3-"),
        "a entrada tinha de ser criada, e nao reusada:\n{MARCO_2}"
    );
    assert!(
        MARCO
            .contains("Entrada de firmware ............. reusada e reapontada · ARCA · {f4057bd0-"),
        "e o primeiro prepare tinha de reusar a que existia"
    );
}

#[test]
fn a_entrada_criada_pelo_arca_saiu_da_ordem_permanente() {
    // **O achado que ninguém previu, resolvido em hardware.** `bcdedit /copy`
    // põe a entrada nova no `displayorder` sozinho — acrescentar um caminho
    // permanente para o dispositivo é o perigo que C-5 nomeia.
    //
    // A tela diz que ela saiu, e o `bcdedit` lido depois confirma: o
    // `displayorder` tem **só** o `{bootmgr}`.
    assert!(
        MARCO_2.contains("a entrada saiu da ordem permanente"),
        "a tela do marco:\n{MARCO_2}"
    );

    let depois = FIRMWARE_DO_MARCO
        .split("### DEPOIS — o gerenciador de firmware")
        .nth(1)
        .expect("o firmware lido depois do marco");

    assert!(
        depois.contains("displayorder            {bootmgr}"),
        "a ordem permanente depois do marco:\n{depois}"
    );
    assert!(
        !depois.contains("f4057bd3"),
        "a entrada criada continua na ordem permanente:\n{depois}"
    );
}

#[test]
fn a_entrada_que_o_arca_criou_e_igual_a_que_havia_antes() {
    // A entrada `{f4057bd3}` que o ARCA criou tem de ser, campo a campo, a
    // `{f4057bd0}` que esta máquina tinha desde antes do projeto — a menos do
    // identificador. Se divergisse, o dispositivo bootaria por uma entrada
    // diferente da que rodou os quatro marcos anteriores.
    let depois = FIRMWARE_DO_MARCO
        .split("### DEPOIS — as entradas de firmware")
        .nth(1)
        .expect("as entradas depois do marco");

    let da_arca = depois
        .split("Gerenciador de Inicialização do Windows")
        .find(|bloco| bloco.contains("description             ARCA"))
        .expect("a entrada ARCA criada pelo ARCA");

    for campo in [
        "device                  partition=F:",
        r"path                    \EFI\boot\bootx64.efi",
        "locale                  pt-BR",
        "inherit                 {globalsettings}",
        "flightsigning           Yes",
        "default                 {current}",
        "displayorder            {current}",
        "toolsdisplayorder       {memdiag}",
        "timeout                 30",
    ] {
        assert!(
            da_arca.contains(campo),
            "falta `{campo}` na entrada criada:\n{da_arca}"
        );
    }
}

#[test]
fn o_iso_local_de_pr_2_rodou_e_nao_baixou_nada() {
    // PR-2 existe para o caso em que a máquina que precisa preparar o
    // dispositivo é justamente a que está sem Windows — e sem rede. O marco
    // com `--iso` conferiu o mesmo SHA256 sem passar pelo `curl`.
    assert!(
        MARCO_2.contains("SHA256 conferido ................ ok · 00cee7700433 · de C:\\"),
        "o SHA256 tinha de vir do arquivo local:\n{MARCO_2}"
    );
    assert!(
        !MARCO_2.contains("Baixando Clonezilla"),
        "com --iso o comando nao pode baixar nada:\n{MARCO_2}"
    );

    // E o primeiro marco baixou, que é o outro caminho.
    assert!(MARCO.contains("Baixando Clonezilla ............. 3.3.3-15"));
}

#[test]
fn o_marco_avisou_que_ia_apagar_um_dispositivo_arca() {
    // O disco que o marco destruiu já tinha os dois rótulos — sobra da medição
    // à mão. A tela nomeou o que se perdia, que é PR-4 na letra, e este é o
    // único original que esse aviso tem.
    assert!(MARCO.contains("ESTE DISCO JA E UM DISPOSITIVO ARCA"));
    assert!(MARCO.contains(r#"1  NTFS    445,6 GB  "ARCAVAULT"                E:"#));
}

/// O `arca backup --dry-run` no dispositivo que a E10 acabou de criar.
const BACKUP_NO_NOVO: &str =
    include_str!("../recursos/capturas/arca-backup-num-dispositivo-novo-2026-08-23.txt");

#[test]
fn um_dispositivo_recem_preparado_nao_faz_o_primeiro_backup_pelo_arca() {
    // **O defeito que uma pergunta pegou, depois de a etapa estar escrita.**
    //
    // A tela do `arca prepare` terminava com `Primeiro backup: arca backup
    // <nome>` — e esse comando recusa. O nome que o Linux dá ao disco sai do
    // `blkdev.list` de dentro de uma imagem (§4.5), e um dispositivo que acabou
    // de nascer não tem imagem nenhuma.
    //
    // Esta captura é o original: rodado em 23/08/2026, com o dispositivo da E10
    // sozinho na mesa, em `--dry-run` para não tocar em nada.
    assert!(
        BACKUP_NO_NOVO.contains("Disco de origem ................. POR DETERMINAR"),
        "a captura devia mostrar o nome do disco por determinar"
    );
    assert!(
        BACKUP_NO_NOVO.contains("nenhuma imagem do dispositivo traz um `blkdev.list` legivel"),
        "e a razao, com todas as letras"
    );

    // E o ensaio marca a receita que imprime como **de exemplo**, em vez de
    // deixá-la passar por utilizável. Uma receita com um nome de disco chutado
    // é o que §4.5 existe para impedir.
    assert!(
        BACKUP_NO_NOVO.contains("DE EXEMPLO: o nome de verdade nao foi determinado"),
        "o ensaio tem de dizer que aquela receita nao serviria"
    );
}

#[test]
fn os_tres_comandos_que_armam_precisam_de_algo_que_o_dispositivo_novo_nao_tem() {
    // A consequência inteira, e ela é maior do que o `arca backup`: **nenhum
    // dos três comandos que armam funciona num dispositivo recém-nascido.**
    //
    // - `arca backup` precisa do nome do disco, que vem de uma imagem (§4.5);
    // - `arca restore` precisa de uma imagem para restaurar (R-1);
    // - `arca verify --completo` precisa de uma imagem para verificar (V-2).
    //
    // O teste é sobre o **código-fonte** porque é uma propriedade da
    // arquitetura, e não de uma execução: os três dependem de `imagens` ou de
    // `blkdev`, e quem acrescentar um quarto comando que arma vai encontrá-lo.
    for (comando, fonte) in [
        ("backup", include_str!("../src/comandos/backup.rs")),
        ("restore", include_str!("../src/comandos/restore.rs")),
        ("verify", include_str!("../src/comandos/verify.rs")),
    ] {
        assert!(
            fonte.contains("imagens::enumerar") || fonte.contains("blkdev"),
            "`arca {comando}` arma e nao depende de imagem nenhuma — se isso mudou, a tela do `arca prepare` precisa mudar junto"
        );
    }

    // **E o quarto, que a E12 acrescentou, é o único que não precisa de nada
    // disso — e é essa a razão de ele existir.** Ele não enumera imagens e não
    // consulta `blkdev`: ele *produz* o que os outros três leem.
    let sondar = include_str!("../src/comandos/sondar.rs");
    assert!(
        !sondar.contains("imagens::enumerar"),
        "`arca sondar` passou a depender de imagem — e ele existe exatamente para o dispositivo que nao tem nenhuma"
    );

    // E a tela do `arca prepare` manda para ele, em vez de prometer um backup
    // que recusaria — ou de mandar para o menu do Clonezilla, que era a
    // resposta anterior e que este app existe para não precisar.
    let prepare = include_str!("../src/comandos/prepare.rs");
    assert!(
        prepare.contains("ANTES DO PRIMEIRO BACKUP, RODE:  arca sondar"),
        "a tela do prepare precisa mandar sondar antes do primeiro backup"
    );
    assert!(
        !prepare.contains("no menu do Clonezilla, faca um backup"),
        "a tela do prepare voltou a mandar fazer o primeiro backup a mão"
    );
}

#[test]
fn o_marco_releu_o_disco_antes_de_apagar() {
    // O terceiro tempo de PR-4, exercitado: entre imprimir o plano e escrever
    // a tabela, o ARCA perguntou de novo ao Windows e comparou.
    assert!(
        MARCO.contains(
            "Conferido antes de escrever ..... ok · o disco 1 continua sendo `JMicron Generic` de 447,1 GB"
        ),
        "a conferencia do terceiro tempo:\n{MARCO}"
    );

    // E ela vem **depois** da pergunta e **antes** da confirmação digitada.
    let pergunta = MARCO.find("Podemos continuar?").expect("a pergunta");
    let conferencia = MARCO
        .find("Conferido antes de escrever")
        .expect("a conferencia");
    let confirmacao = MARCO
        .find("Digite o modelo do disco")
        .expect("a confirmacao");
    let particionamento = MARCO.find("Particionando ...").expect("o ponto sem volta");

    assert!(
        pergunta < conferencia && conferencia < confirmacao && confirmacao < particionamento,
        "a ordem dos quatro tempos de PR-4 mudou"
    );
}

// ─────────────────────────── auxiliares ───────────────────────────

/// O trecho de uma medição entre um título `###` e o próximo.
fn bloco_do_bcdedit(medicao: &str, titulo: &str) -> String {
    medicao
        .split(titulo)
        .nth(1)
        .unwrap_or_else(|| panic!("a medicao nao tem a secao `{titulo}`"))
        .split("\n###")
        .next()
        .unwrap_or_default()
        .to_string()
}

/// O texto sem `\r`, para comparar arquivos que passaram por ferramentas
/// diferentes.
fn sem_cr(texto: &str) -> String {
    texto.replace('\r', "")
}
