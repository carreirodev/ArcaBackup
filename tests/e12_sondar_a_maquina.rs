//! A etapa E12 — `arca sondar` — e os requisitos SD-1 a SD-6.
//!
//! # O que esta etapa fecha, em uma frase
//!
//! O §4.5 diz que o nome do disco no Linux sai do `blkdev.list` de dentro de
//! uma imagem. Um dispositivo recém-preparado **não tem imagem**, logo não tem
//! o nome, logo `arca backup` recusa — e `arca restore` e `arca verify
//! --completo` também. A sondagem dá uma **segunda fonte para o mesmo
//! arquivo**, e ela não depende de imagem nenhuma.
//!
//! # O que este arquivo prova, e o que ele não tem como provar
//!
//! Ele prova as propriedades que atravessam módulos: que a receita não chama
//! programa nenhum do Clonezilla, que a sondagem grava onde a colheita procura,
//! que as duas fontes do §4.5 têm precedência definida, e que a saída sempre
//! diz de onde o nome veio.
//!
//! O que ele **não** prova é que o `lsblk` daquele live system aceita aquelas
//! flags — isso é reconstrução (ver `FLAGS_DE_SONDAGEM` em `src/receita.rs`), e
//! quem responde é o hardware. O que o código garante é que o modo de falha
//! dessa reconstrução seja barato e visível: `ARCA_PROBE=FALHOU` numa tela, com
//! a mensagem do `lsblk` dentro do próprio `blkdev.list`.
//!
//! # Nenhum teste daqui arma, e nenhum reinicia
//!
//! São montagens de receita e leituras de duplos. Quem arma é `arca sondar`, e
//! armar é o ponto sem volta.

use arca::blkdev::{self, Fonte, Lista, Origem, SemNome};
use arca::duplos::{ArquivosEmMemoria, momento};
use arca::estado::Estado;
use arca::nome::Nome;
use arca::receita::{Disco, Operacao, Pedido, Receita, Selo};
use arca::sondagem;
use std::path::Path;

/// A raiz do `ARCAVAULT` como o Windows a vê.
const VAULT: &str = r"E:\";

/// O `blkdev.list` que a sondagem grava, com as colunas do cabeçalho
/// capturado das imagens deste dispositivo.
///
/// A linha do `ARCAVAULT` sai com `/home/partimag` no `MOUNTPOINT` — como já
/// sai nos `blkdev.list` capturados —, e isso é o **segundo sinal de graça** da
/// etapa: o próprio arquivo testemunha que foi escrito no repositório montado,
/// e não no tmpfs da RAM.
const DA_SONDAGEM: &str = concat!(
    "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
    "loop0     loop0       466.2M loop squashfs /run/live/rootfs/filesystem.squashfs \n",
    "sda       sda         238.5G disk                                               KGSSE100256\n",
    "sda1      |-sda1      236.9G part ntfs     /home/partimag                       \n",
    "sda2      `-sda2        1.6G part vfat                                          \n",
    "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
    "nvme0n1p1 |-nvme0n1p1   300M part vfat                                          \n",
);

/// O `blkdev.list` de dentro de uma imagem, copiado do dispositivo em
/// 22/08/2026.
const DA_IMAGEM: &str = concat!(
    "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
    "sda       sda         238.5G disk                                               KGSSE100256\n",
    "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
);

fn receita_da_sondagem() -> Receita {
    Receita::montar(&Pedido {
        operacao: Operacao::Sondagem,
        nome: None,
        disco: None,
        selo: Selo::de_ensaio(),
    })
    .expect("a receita da sondagem passa por C-2")
}

// ───────────────────── SD-1: a receita não faz backup ─────────────────────

#[test]
fn sd1_a_receita_nao_chama_programa_nenhum_do_clonezilla() {
    // **É o que faz desta a única etapa deste projeto com marco em hardware
    // cujo modo de falha não envolve gravação.** Sem `ocs-sr` não há
    // `savedisk` nem `restoredisk`; sem `ocs-chkimg` não há escrita dentro de
    // pasta de imagem. O pior caso é a máquina parar num menu.
    let comando = receita_da_sondagem().comando().to_string();

    for programa in ["ocs-sr", "ocs-chkimg", "savedisk", "restoredisk"] {
        assert!(
            !comando.contains(programa),
            "a sondagem chama `{programa}`: {comando}"
        );
    }

    // E tudo o que ela escreve mora dentro do `ARCAVAULT`, que o Clonezilla
    // monta em `/home/partimag`.
    for escrita in comando.split("; ").filter(|passo| passo.contains('>')) {
        assert!(
            escrita.contains("/home/partimag/"),
            "a sondagem escreve fora do ARCAVAULT: {escrita}"
        );
    }
}

// ────────── SD-2: a saída sai no formato que o §4.5 já sabe ler ──────────

#[test]
fn sd2_o_arquivo_da_sondagem_e_lido_pelo_parser_das_imagens() {
    // **O parser não muda**: `crate::blkdev` continua sendo o único lugar que
    // lê aquele formato. Um segundo leitor divergiria do primeiro na primeira
    // mudança, e a divergência apareceria como um comando achando o disco que
    // o outro não acha.
    let discos = blkdev::ler(DA_SONDAGEM);

    assert_eq!(discos.len(), 2, "os dois discos, sem as partições e o loop");
    assert_eq!(discos[1].nome, "nvme0n1");
    assert_eq!(discos[1].modelo, "KINGSTON SNV3S500G");

    // E é o mesmo parser que lê o de dentro da imagem, com o mesmo resultado.
    assert_eq!(
        blkdev::ler(DA_IMAGEM)[1],
        discos[1],
        "as duas fontes descrevem o mesmo disco do mesmo jeito"
    );
}

#[test]
fn sd2_as_colunas_da_receita_sao_as_do_cabecalho_capturado() {
    // **Reconstrução, e não transcrição.** Temos o resultado — o cabeçalho
    // acima, copiado do dispositivo — e não a linha de comando que o produziu:
    // ela mora nos scripts do Clonezilla, dentro do `filesystem.squashfs`.
    //
    // O teste confere a reconstrução contra o **original**, lendo as colunas do
    // próprio cabeçalho em vez de repeti-las: uma lista escrita à mão aqui
    // provaria que eu sei copiar a constante.
    //
    // **E a comparação é por igualdade, e não por `contains`.** A primeira
    // versão deste teste procurava `-o <colunas>` como substring, e uma coluna
    // **a mais** passava por ela — `-o A,B,C,D` contém `-o A,B,C`. Não é
    // hipotético: a falha forçada de 24/08/2026 acrescentou `FLAGQUENAOEXISTE`
    // ao fim da lista para fazer o `lsblk` recusar a receita, e esta asserção
    // **passou**.
    let cabecalho = DA_SONDAGEM.lines().next().expect("o cabeçalho está lá");
    let colunas: Vec<&str> = cabecalho.split_whitespace().collect();

    const MARCA: &str = "-o ";
    let comando = receita_da_sondagem().comando().to_string();
    let depois = &comando[comando.find(MARCA).expect("a receita tem `-o `") + MARCA.len()..];
    let na_receita = depois.split_whitespace().next().expect("há colunas");

    assert_eq!(
        na_receita,
        colunas.join(","),
        "as colunas do `lsblk` não reproduzem o cabeçalho de `{cabecalho}`"
    );
}

#[test]
fn sd2_uma_coluna_a_mais_nao_atravessa_o_teste_das_colunas() {
    // **O guarda do guarda**, e ele existe porque o buraco foi medido.
    //
    // O teste acima só vale se ele reprovar uma lista diferente da do
    // cabeçalho — e a versão dele que usava `contains` **não reprovava** uma
    // coluna a mais. Este teste exercita a mesma extração sobre uma receita
    // adulterada, para que a forma frouxa não volte por descuido.
    let comando = receita_da_sondagem()
        .comando()
        .replace(",MODEL ", ",MODEL,FLAGQUENAOEXISTE ");

    const MARCA: &str = "-o ";
    let depois = &comando[comando.find(MARCA).expect("a receita tem `-o `") + MARCA.len()..];
    let na_receita = depois.split_whitespace().next().expect("há colunas");

    let cabecalho = DA_SONDAGEM.lines().next().expect("o cabeçalho está lá");
    let colunas: Vec<&str> = cabecalho.split_whitespace().collect();

    assert_ne!(
        na_receita,
        colunas.join(","),
        "uma coluna a mais passou pela extração: o teste acima não guarda nada"
    );
    // E a forma frouxa, para mostrar o que ela deixava passar.
    assert!(
        comando.contains(&format!("-o {}", colunas.join(","))),
        "esta é a asserção que a falha forçada atravessou, e ela continua atravessando"
    );
}

// ──────────── SD-3: o `if`, e o que o `;` escreveria no lugar ────────────

#[test]
fn sd3_o_lsblk_roda_dentro_de_um_if_e_o_desfecho_segue_o_codigo_de_saida() {
    // **R-5, e a primeira forma escrita desta receita não o tinha.** O `;` não
    // olha código de saída: com o `lsblk` falhando — uma flag que esta versão
    // do util-linux não conheça basta —, o desfecho diria `OK` assim mesmo.
    //
    // O estrago é uma contradição dentro da mesma sessão: o `arca resultado`
    // diria que a sondagem concluiu, e a tela seguinte diria `Disco de origem
    // ... POR DETERMINAR`.
    //
    // Que o **bash** faz o que esta string diz está medido em
    // `recursos/ensaio-da-receita.sh`, que roda as duas formas lado a lado.
    let comando = receita_da_sondagem().comando().to_string();

    assert!(comando.contains("if lsblk "), "{comando}");
    assert!(comando.contains("then echo ARCA_PROBE=OK"), "{comando}");
    assert!(comando.contains("else echo ARCA_PROBE=FALHOU"), "{comando}");
}

#[test]
fn sd3_o_erro_do_lsblk_fica_no_dispositivo_em_vez_de_sumir() {
    // O que torna a reconstrução das flags aceitável: o modo de falha é
    // **barato e visível**. Com `2>&1` apontando para o próprio `blkdev.list`,
    // uma flag recusada deixa a mensagem do `lsblk` no dispositivo em vez de
    // sumir com o `poweroff`.
    //
    // E um arquivo com mensagem de erro não é lido como oráculo: o cabeçalho
    // não bate, e o parser devolve lista vazia — que é o que ele devolve para
    // tudo o que não entende.
    assert!(
        receita_da_sondagem().comando().contains("blkdev.list 2>&1"),
        "o erro do `lsblk` não vai para o arquivo"
    );
    assert!(
        blkdev::ler("lsblk: unknown column: FLAGQUENAOEXISTE\n").is_empty(),
        "uma mensagem de erro foi lida como se fosse tabela de discos"
    );
}

// ─────────── SD-4: onde ela grava, e o que acontece com a anterior ───────────

#[test]
fn sd4_os_dois_lados_do_reinicio_apontam_para_a_mesma_pasta() {
    // A receita escreve num caminho Linux; a colheita lê num caminho Windows.
    // Os dois saem da **mesma** função — `receita::pasta_do_log` —, e é isso
    // que garante que ninguém procure o desfecho onde ele não está.
    let comando = receita_da_sondagem().comando().to_string();

    assert!(
        comando.contains("/home/partimag/ARCA-LOGS/sondagem/blkdev.list"),
        "{comando}"
    );
    assert!(
        comando.contains("/home/partimag/ARCA-LOGS/sondagem/arca-fim.txt"),
        "{comando}"
    );

    assert_eq!(
        sondagem::caminho(Path::new(VAULT)),
        Path::new(r"E:\ARCA-LOGS\sondagem\blkdev.list")
    );
    assert_eq!(
        arca::estado::caminho_do_desfecho(Path::new(VAULT), Operacao::Sondagem, None),
        Path::new(r"E:\ARCA-LOGS\sondagem\arca-fim.txt")
    );
}

#[test]
fn sd4_a_pasta_da_sondagem_nao_colide_com_a_de_nenhuma_imagem() {
    // A pasta é fixa porque a sondagem não tem nome de imagem, e a de todas as
    // outras leva `-<nome>`. **Isto é o que impede a colisão**, e vale para
    // qualquer nome que B-2 aceite — inclusive um chamado `sondagem`.
    let homonima = Nome::novo("sondagem").expect("B-2 aceita este nome de imagem");

    for operacao in [
        Operacao::Backup,
        Operacao::Restauracao,
        Operacao::Verificacao,
    ] {
        assert_ne!(
            arca::receita::pasta_do_log(operacao, Some(&homonima)),
            arca::receita::pasta_do_log(Operacao::Sondagem, None),
            "uma imagem chamada `sondagem` colidiria com a pasta da sondagem em `{}`",
            operacao.nome()
        );
    }
}

#[test]
fn sd4_a_sondagem_nao_aparece_como_imagem_nem_como_residuo() {
    // `ARCA-LOGS` está **fora** da listagem de imagens, de propósito e com
    // teste desde a E1. Sem isso, `ARCA-LOGS\sondagem\` apareceria no `arca
    // list` como resíduo — não tem `MD5SUMS` —, e B-3 passaria a recusar o
    // nome `sondagem` para um backup.
    let arquivos = ArquivosEmMemoria::novo()
        .com(r"E:\ARCA-LOGS\sondagem\blkdev.list", DA_SONDAGEM)
        .com(r"E:\ARCA-LOGS\sondagem\arca-fim.txt", "ARCA_PROBE=OK\n")
        .com(r"E:\2026-08-22_Apps\MD5SUMS", "");

    let pastas = arca::imagens::enumerar(&arquivos, Path::new(VAULT)).expect("a listagem funciona");

    assert_eq!(pastas.len(), 1, "só a imagem aparece: {pastas:?}");
    assert_eq!(pastas[0].nome, "2026-08-22_Apps");

    // E quem quiser a sondagem vai buscá-la onde ela está.
    assert!(
        sondagem::ler(&arquivos, Path::new(VAULT)).is_some(),
        "a sondagem não é alcançável pelo módulo que a lê"
    );
}

#[test]
fn sd4_o_estado_de_uma_sondagem_da_a_volta_sem_nome_de_imagem() {
    // O `nome` do `estado.json` virou opcional, com a **string vazia** como
    // sentinela — o precedente é da E11, e o argumento é o mesmo: `Nome::novo`
    // recusa o vazio, então ele nunca foi um nome possível e não pode colidir.
    let json = r#"{
  "selo": "a3f1c9e07b2d4856",
  "comando": "sondagem",
  "nome": "",
  "disco": "",
  "armado_em": "2026-08-23T21:10:44-03:00",
  "situacao": "armado"
}
"#;

    let estado = Estado::de_json(json).expect("o estado de uma sondagem é legível");

    assert_eq!(estado.comando, Operacao::Sondagem);
    assert_eq!(estado.nome, None);
    assert_eq!(estado.disco, None);
    assert_eq!(estado.descricao(), "sondagem");
}

// ──────────── SD-5: a precedência entre a sondagem e as imagens ────────────

fn da_sondagem(texto: &str, quando: &str) -> Lista {
    Lista {
        fonte: Fonte::Sondagem {
            quando: Some(momento(quando)),
        },
        texto: texto.to_string(),
    }
}

fn da_imagem(imagem: &str, texto: &str) -> Lista {
    Lista {
        fonte: Fonte::Imagem(imagem.to_string()),
        texto: texto.to_string(),
    }
}

#[test]
fn sd5_num_dispositivo_sem_imagem_a_sondagem_responde_sozinha() {
    // **P-26 em um teste.** É este o caso que a etapa existe para resolver: o
    // dispositivo que o `arca prepare` acabou de criar, sem imagem nenhuma, em
    // que os três comandos que armam recusavam.
    let achado = blkdev::nome_do_disco(
        "KINGSTON SNV3S500G",
        &[da_sondagem(DA_SONDAGEM, "2026-08-23T21:14:07")],
    )
    .expect("a sondagem responde sem imagem nenhuma");

    assert_eq!(achado.disco.como_texto(), "nvme0n1");
    assert!(matches!(achado.origem, Origem::LidoDaSondagem { .. }));
}

#[test]
fn sd5_a_sondagem_ganha_das_imagens_e_a_divergencia_e_dita() {
    // A sondagem descreve a máquina de **agora**; a imagem descreve a de
    // quando o backup foi feito. Um disco trocado entre as duas muda o nome
    // que o Linux dá a ele, e a imagem passaria a nomear um disco que não está
    // mais lá.
    //
    // A sondagem ganha — e a divergência sai na tela, nunca resolvida em
    // silêncio.
    let na_imagem = DA_IMAGEM.replace("nvme0n1   nvme0n1", "nvme1n1   nvme1n1");
    let listas = vec![
        da_sondagem(DA_SONDAGEM, "2026-08-23T21:14:07"),
        da_imagem("2026-08-21_WindowsCompleto", &na_imagem),
    ];

    let achado = blkdev::nome_do_disco("KINGSTON SNV3S500G", &listas).unwrap();
    let dito = achado.to_string();

    assert_eq!(achado.disco.como_texto(), "nvme0n1");
    assert!(dito.contains("sondagem"), "{dito}");
    assert!(dito.contains("DIVERGE"), "{dito}");
    assert!(dito.contains("nvme1n1"), "{dito}");
    assert!(dito.contains("2026-08-21_WindowsCompleto"), "{dito}");
}

#[test]
fn sd5_a_saida_sempre_diz_de_onde_o_nome_veio() {
    // O padrão que a E3 estabeleceu e que o §4.5 exige: *uma receita destrutiva
    // que nomeie um disco sem dizer a origem do nome é pior do que não imprimir
    // nada*. Com duas fontes, a frase tem de distinguir **qual** respondeu.
    let pela_sondagem = blkdev::nome_do_disco(
        "KINGSTON SNV3S500G",
        &[da_sondagem(DA_SONDAGEM, "2026-08-23T21:14:07")],
    )
    .unwrap()
    .to_string();
    let pela_imagem = blkdev::nome_do_disco(
        "KINGSTON SNV3S500G",
        &[da_imagem("2026-08-21_WindowsCompleto", DA_IMAGEM)],
    )
    .unwrap()
    .to_string();

    assert!(
        pela_sondagem.contains("lido da sondagem"),
        "{pela_sondagem}"
    );
    assert!(pela_sondagem.contains("23/08 21:14"), "{pela_sondagem}");
    assert!(
        pela_imagem.contains("lido de 2026-08-21_WindowsCompleto"),
        "{pela_imagem}"
    );

    assert_ne!(
        pela_sondagem, pela_imagem,
        "as duas fontes se apresentam igual, e a tela não distingue qual respondeu"
    );
}

#[test]
fn sd5_a_recusa_sem_oraculo_manda_sondar_em_vez_do_menu_do_clonezilla() {
    // **A saída mudou, e é essa a diferença que a etapa compra.** Até a E11
    // quem caía aqui tinha de fazer o primeiro backup pelo menu do Clonezilla
    // (§6.4) — dois reinícios e cerca de quarenta minutos, e exatamente aquilo
    // que este app existe para não precisar.
    let recusa = blkdev::nome_do_disco("KINGSTON SNV3S500G", &[]).unwrap_err();

    assert_eq!(recusa, SemNome::SemOraculo);
    assert!(recusa.to_string().contains("arca sondar"), "{recusa}");
}

// ────────────── SD-6: o que se digita, e o que isso impede ──────────────

#[test]
fn sd6_a_sondagem_nao_pede_texto_por_extenso_e_nao_ha_o_que_pedir() {
    // **A decisão, e o argumento dela.** S-2 pede o *alvo* por extenso — o nome
    // da imagem que vai ser gravada, o modelo do disco que vai ser apagado —, e
    // existe para custar lê-lo. A sondagem não tem alvo: ela não apaga nada e
    // não escolhe nada, e o irreversível dela é **reiniciar a máquina**.
    //
    // Pedir a palavra `sondar` por extenso seria ruído: quem acabou de digitar
    // `arca sondar` a ecoaria sem ler nada, e uma confirmação que só ecoa o
    // comando ensina a digitar sem ler — o contrário do que S-2 compra.
    //
    // O que fica é a pergunta de uma tecla com o padrão no **não**, a mesma do
    // primeiro tempo de PR-4. E o que ela impede está dito na tela logo acima
    // dela: o reinício.
    let fonte = include_str!("../src/comandos/sondar.rs");

    assert!(
        fonte.contains("perguntar_se_pode"),
        "o `arca sondar` deixou de perguntar antes de reiniciar"
    );
    assert!(
        !fonte.contains("confirmacao::pedir"),
        "o `arca sondar` passou a pedir texto por extenso, e não há alvo a confirmar"
    );
    assert!(
        arca::confirmacao::e_sim("s") && !arca::confirmacao::e_sim(""),
        "o padrão da pergunta deixou de ser o não"
    );
}

#[test]
fn sd6_a_tela_diz_o_que_a_sondagem_faz_antes_de_perguntar() {
    // Uma confirmação sobre a qual a pessoa não leu nada não impede nada. O que
    // separa esta operação das outras três é o que ela **não** faz, e é isso
    // que decide se alguém aperta `s`.
    let saida = arca::comandos::sondar::montar_o_que_vai_acontecer();

    assert!(saida.contains("NAO FAZ BACKUP NEM RESTAURACAO"), "{saida}");
    assert!(saida.contains("reinicia a maquina"), "{saida}");
    assert!(saida.contains("se perde"), "o custo real: {saida}");
}

// ─────────────────── o que a etapa NÃO afirma ───────────────────

#[test]
fn a_tela_nao_estima_quanto_tempo_o_boot_do_clonezilla_leva() {
    // **O custo de um boot do Clonezilla isolado não está medido neste
    // repositório**: toda execução anterior tinha uma operação longa depois
    // dele — 39,7 GB gravados, uma restauração, um `ocs-chkimg` de 312 s —, e o
    // boot ficou embutido em cada total.
    //
    // Esta etapa é a primeira em que ele é quase tudo o que há, e por isso ela
    // mede. Até a medição existir, um número na tela seria palpite vestido de
    // medição — o padrão que o §3.5 do PRD conta ter custado caro cinco vezes.
    let saida = arca::comandos::sondar::montar_o_que_vai_acontecer();

    for palpite in ["minuto", "segundo", "demora", "leva "] {
        assert!(
            !saida.contains(palpite),
            "a tela promete tempo (`{palpite}`): {saida}"
        );
    }
}

#[test]
fn a_sondagem_e_a_unica_das_quatro_que_nao_nomeia_imagem_nem_disco() {
    // A matriz inteira, num lugar só. Os dois eixos são independentes — a
    // verificação **não** nomeia disco e **nomeia** imagem —, e é isso que
    // impede o `estado.json` de aceitar metade das combinações.
    let matriz = [
        (Operacao::Backup, true, true),
        (Operacao::Restauracao, true, true),
        (Operacao::Verificacao, true, false),
        (Operacao::Sondagem, false, false),
    ];

    for (operacao, nomeia_imagem, nomeia_disco) in matriz {
        assert_eq!(
            operacao.nomeia_imagem(),
            nomeia_imagem,
            "{}",
            operacao.nome()
        );
        assert_eq!(operacao.nomeia_disco(), nomeia_disco, "{}", operacao.nome());

        // E `Receita::montar` cobra as duas, nos dois sentidos.
        let montada = Receita::montar(&Pedido {
            operacao,
            nome: nomeia_imagem.then(|| Nome::novo("2026-08-22_Apps").unwrap()),
            disco: nomeia_disco.then(|| Disco::novo("nvme0n1").unwrap()),
            selo: Selo::de_ensaio(),
        });
        assert!(
            montada.is_ok(),
            "a combinação coerente de `{}` foi recusada",
            operacao.nome()
        );
    }
}
