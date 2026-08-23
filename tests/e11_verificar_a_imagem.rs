//! A etapa E11 contra o hardware desta mesa.
//!
//! O parser do `MD5SUMS` e testado em `src/md5sums.rs` contra a captura, o
//! leitor do `certutil` em `src/resumo.rs` contra a resposta transcrita, e o
//! julgamento em `src/verificacao.rs` contra duplos. O que este arquivo prova e
//! o que nenhum dos tres pode: **que as duas pontas da comparacao continuam
//! sendo as mesmas no dispositivo de verdade** — o `MD5SUMS` que o Linux
//! escreveu em 22/08, e o `certutil` do Windows lendo os mesmos bytes hoje.
//!
//! # O que a etapa mediu, e vale mais do que qualquer assercao daqui
//!
//! ```text
//! V-1  os 39 MD5 do MD5SUMS, 39,7 GB ....... 202,6 s a mao · 199,4 s e 202,8 s pelo comando
//! V-2  o ocs-chkimg, pelos mtimes de 22/08 . 312 s, mais um reinicio
//! ```
//!
//! **V-1 dizia "em segundos", e sao tres minutos e meio.** O requisito estava
//! errado e a etapa o corrigiu — ver `PRD/PRD-ARCA-v5_1.md`, §9.5.
//!
//! E o `MD5SUMS` **cobre os arquivos da imagem**, e nao so os metadados: a
//! ordem dele nao e alfabetica pura, e os catorze `nvme0n1p*` ficam no meio.
//! Quem olhar as primeiras e as ultimas linhas conclui o contrario, e V-1
//! inteiro ficaria construido sobre isso.
//!
//! # Nenhum teste daqui escreve, e nenhum arma
//!
//! Sao leituras: o `ARCAVAULT`, os `MD5SUMS` e o `certutil` sobre arquivos que
//! ja estao la. Quem arma e `arca verify --completo`, e armar e o ponto sem
//! volta.

#![cfg(windows)]

use arca::adaptadores::ArquivosDoSistema;
use arca::adaptadores::windows::sistema::SistemaDoWindows;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::dispositivo::{self, Dispositivo};
use arca::imagens::{self, Especie, Veredito};
use arca::md5sums;
use arca::portas::{Arquivos, Sistema};
use arca::resumo::{self, Algoritmo};
use arca::verificacao;
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

/// As imagens de verdade do dispositivo, sem os residuos.
/// As imagens do dispositivo desta mesa, ou `None` quando não há o que
/// conferir.
///
/// # Dois casos diferentes chegam aqui como `None`, e desde a E10 o segundo é
/// normal
///
/// **Não há dispositivo conectado** era o único caso quando estes testes
/// nasceram: sem `ARCAVAULT`, não há imagem, e o teste sai sem falar.
///
/// **Há dispositivo e ele está vazio** passou a existir na E10, quando o `arca
/// prepare` começou a criar dispositivos — e um dispositivo recém-nascido não
/// tem imagem nenhuma, por construção. Ele nem consegue fazer a primeira: o
/// nome do disco no Linux sai do `blkdev.list` de dentro de uma imagem (§4.5), e
/// o primeiro backup precisa ser feito pelo menu do Clonezilla (§6.4).
///
/// Os dois saem cedo, e **os dois dizem por quê** — um teste de hardware que
/// sai calado é indistinguível de um que passou, e a diferença entre "não testei"
/// e "testei e passou" é a mesma que este projeto persegue em toda parte.
///
/// O que **não** se perde saindo cedo: `src/md5sums.rs` fixa os catorze
/// `nvme0n1p*` da captura e roda sem hardware nenhum. O que estes testes
/// acrescentam é a confirmação contra as imagens **deste** dispositivo, e ela
/// só existe quando há imagens.
fn imagens_do_dispositivo() -> Option<Vec<(String, PathBuf)>> {
    let Some(raiz) = raiz_do_vault() else {
        eprintln!(
            "  (sem dispositivo ARCA conectado — este teste nao conferiu nada, e nao e falha)"
        );
        return None;
    };

    let pastas = imagens::enumerar(&ArquivosDoSistema, &raiz).ok()?;
    let imagens: Vec<(String, PathBuf)> = pastas
        .into_iter()
        .filter(|pasta| matches!(pasta.especie, Especie::Imagem { .. }))
        .map(|pasta| {
            let caminho = raiz.join(&pasta.nome);
            (pasta.nome, caminho)
        })
        .collect();

    if imagens.is_empty() {
        eprintln!(
            "  (o dispositivo em {} esta VAZIO — nenhuma imagem a conferir.\n   \
             E um estado legitimo desde a E10: `arca prepare` cria dispositivos sem\n   \
             imagem, e o primeiro backup deles e pelo menu do Clonezilla (§4.5, §6.4).\n   \
             Este teste nao conferiu nada, e nao e falha.)",
            raiz.display()
        );
        return None;
    }

    Some(imagens)
}

#[test]
fn todo_md5sums_deste_dispositivo_e_lido_pelo_parser() {
    // O parser foi escrito contra **uma** captura. As outras duas imagens do
    // dispositivo sao do mesmo Clonezilla e de dias diferentes, e sao o que
    // separa "o parser lê aquele arquivo" de "o parser lê o que esta
    // ferramenta escreve".
    let Some(imagens) = imagens_do_dispositivo() else {
        return;
    };
    // A lista nunca chega vazia aqui — `imagens_do_dispositivo` já sai cedo
    // nesse caso, e diz por quê. O `debug_assert` guarda a invariante sem
    // transformar um dispositivo recém-preparado em suíte vermelha.
    debug_assert!(!imagens.is_empty(), "a lista devia ter saido cedo");

    for (nome, caminho) in imagens {
        let arquivo = caminho.join(md5sums::ARQUIVO);
        let texto = ArquivosDoSistema
            .ler_texto_alheio(&arquivo)
            .unwrap_or_else(|erro| panic!("`{nome}`: {erro}"));

        let entradas = md5sums::ler(&texto)
            .unwrap_or_else(|recusa| panic!("`{nome}` tem MD5SUMS ilegivel: {recusa}"));

        assert!(
            !entradas.is_empty(),
            "`{nome}` tem MD5SUMS sem linha nenhuma"
        );
    }
}

#[test]
fn todo_md5sums_deste_dispositivo_cobre_os_arquivos_da_imagem() {
    // **O achado que quase passou batido.** Olhando so as primeiras e as
    // ultimas linhas do `MD5SUMS`, os `nvme0n1p*` nao aparecem — a ordem nao e
    // alfabetica pura, e eles ficam no meio. Quem concluisse dali que o
    // `MD5SUMS` cobre so os metadados construiria V-1 sobre isso, e o comando
    // aprovaria uma imagem tendo lido 2 KB de 39,7 GB.
    let Some(imagens) = imagens_do_dispositivo() else {
        return;
    };

    for (nome, caminho) in imagens {
        let texto = ArquivosDoSistema
            .ler_texto_alheio(&caminho.join(md5sums::ARQUIVO))
            .expect("MD5SUMS legivel");
        let entradas = md5sums::ler(&texto).expect("MD5SUMS valido");

        let de_imagem = entradas
            .iter()
            .filter(|entrada| entrada.arquivo.contains("-ptcl-img."))
            .count();

        assert!(
            de_imagem > 0,
            "`{nome}`: o MD5SUMS nao lista nenhum arquivo de particao, so metadados"
        );
    }
}

#[test]
fn nenhum_md5sums_deste_dispositivo_aponta_para_arquivo_ausente() {
    // Um arquivo listado e ausente e falha de verdade — imagem incompleta. O
    // inverso nao e: quatro arquivos da pasta ficam **de fora** do `MD5SUMS`
    // por construcao, e o teste seguinte cobra isso.
    let Some(imagens) = imagens_do_dispositivo() else {
        return;
    };

    for (nome, caminho) in imagens {
        let texto = ArquivosDoSistema
            .ler_texto_alheio(&caminho.join(md5sums::ARQUIVO))
            .expect("MD5SUMS legivel");
        let entradas = md5sums::ler(&texto).expect("MD5SUMS valido");

        for entrada in &entradas {
            assert!(
                ArquivosDoSistema.existe(&caminho.join(&entrada.arquivo)),
                "`{nome}`: o MD5SUMS lista `{}` e ele nao esta na pasta",
                entrada.arquivo
            );
        }
    }
}

#[test]
fn a_pasta_da_imagem_tem_arquivos_que_o_md5sums_nao_lista_e_isso_e_normal() {
    // Medido na `2026-08-22_Apps` em 23/08/2026: sao quatro, e cada um tem
    // hora. O `MD5SUMS`, o `clonezilla-img` e o `Info-img-id.txt` levam o
    // **mesmo mtime** — 18:00:49, o fim do `savedisk` —, e o `arca-check.log`
    // e de 18:06:02, cinco minutos depois, escrito pelo `ocs-chkimg` de B-9.
    //
    // Nao e falta: e a hora em que cada um nasceu. Chamar isso de falha
    // reprovaria toda imagem que o Clonezilla ja fez.
    let Some(imagens) = imagens_do_dispositivo() else {
        return;
    };

    for (nome, caminho) in imagens {
        let texto = ArquivosDoSistema
            .ler_texto_alheio(&caminho.join(md5sums::ARQUIVO))
            .expect("MD5SUMS legivel");
        let entradas = md5sums::ler(&texto).expect("MD5SUMS valido");

        let plano = verificacao::planejar(&ArquivosDoSistema, &caminho, &entradas)
            .expect("a pasta da imagem se deixa listar");

        assert!(
            plano.fora_do_md5sums > 0,
            "`{nome}`: o proprio MD5SUMS ja devia estar fora da lista dele"
        );
    }
}

#[test]
fn o_certutil_desta_maquina_confirma_o_md5_que_o_clonezilla_registrou() {
    // **As duas pontas da comparacao, no hardware.** De um lado o `md5sum` do
    // Linux, que escreveu o `MD5SUMS` em 22/08; do outro o `certutil` do
    // Windows, lendo os mesmos bytes hoje. Duas ferramentas, dois sistemas
    // operacionais, um numero.
    //
    // Confere o **menor** arquivo de cada imagem: o teste roda no `cargo test`
    // e nao pode levar tres minutos por imagem. Quem confere a imagem inteira
    // e `arca verify`, e ele foi rodado de verdade — 202,8 s, 39 de 39.
    let Some(imagens) = imagens_do_dispositivo() else {
        return;
    };

    for (nome, caminho) in imagens {
        let texto = ArquivosDoSistema
            .ler_texto_alheio(&caminho.join(md5sums::ARQUIVO))
            .expect("MD5SUMS legivel");
        let entradas = md5sums::ler(&texto).expect("MD5SUMS valido");

        let plano = verificacao::planejar(&ArquivosDoSistema, &caminho, &entradas)
            .expect("a pasta se deixa listar");

        let menor = plano
            .arquivos
            .iter()
            .filter(|(_, bytes)| *bytes > 0)
            .min_by_key(|(_, bytes)| *bytes)
            .map(|(entrada, _)| entrada)
            .expect("a imagem tem ao menos um arquivo com bytes");

        let saida = SistemaDoWindows
            .resumir(&caminho.join(&menor.arquivo), Algoritmo::Md5)
            .expect("o certutil responde");
        let achado = resumo::do_certutil(&saida, Algoritmo::Md5)
            .unwrap_or_else(|recusa| panic!("`{nome}`/`{}`: {recusa}", menor.arquivo));

        assert_eq!(
            achado, menor.soma,
            "`{nome}`: o `{}` nao soma o que o MD5SUMS registra",
            menor.arquivo
        );
    }
}

#[test]
fn a_verificacao_de_uma_imagem_intacta_aprova() {
    // O caminho inteiro de V-1, de ponta a ponta, na imagem mais leve do
    // dispositivo — planejar, conferir, julgar. E a mesma funcao que o
    // `arca verify` chama.
    let Some(imagens) = imagens_do_dispositivo() else {
        return;
    };

    // A mais leve, para o `cargo test` nao levar minutos. As tres desta mesa
    // passam dos 30 GB, e por isso a conferencia inteira e do comando, e nao
    // daqui — ver `o_certutil_desta_maquina_confirma...`.
    let Some((nome, caminho)) = imagens
        .into_iter()
        .min_by_key(|(_, caminho)| tamanho_da_pasta(caminho))
    else {
        return;
    };

    let texto = ArquivosDoSistema
        .ler_texto_alheio(&caminho.join(md5sums::ARQUIVO))
        .expect("MD5SUMS legivel");
    let entradas = md5sums::ler(&texto).expect("MD5SUMS valido");

    // So os metadados: os arquivos de particao sao gigabytes, e o que este
    // teste prova e que a fiacao esta certa. O tamanho e assunto do comando.
    let leves: Vec<md5sums::Entrada> = entradas
        .into_iter()
        .filter(|entrada| !entrada.arquivo.contains("-ptcl-img."))
        .collect();

    let plano =
        verificacao::planejar(&ArquivosDoSistema, &caminho, &leves).expect("a pasta se lista");
    let conferencia = verificacao::conferir(
        &ArquivosDoSistema,
        &SistemaDoWindows,
        &caminho,
        &plano,
        &mut |_| {},
    )
    .expect("a conferencia roda");

    assert_eq!(
        conferencia.veredito(),
        Veredito::Aprovada,
        "`{nome}`: os metadados nao bateram — {:?}",
        conferencia.falhas()
    );
    assert_eq!(conferencia.quantos(), leves.len());
}

#[test]
fn o_certutil_recusa_arquivo_que_nao_existe_em_vez_de_inventar_hash() {
    // O modo de falha que o leitor de `crate::resumo` existe para pegar: a
    // resposta de erro do `certutil` tambem tem linhas, e nenhuma delas e
    // hash. Medido: `exit=-2147024894`, duas linhas de frase traduzida.
    let saida = SistemaDoWindows
        .resumir(
            std::path::Path::new(r"D:\ARCA-ARQUIVO-QUE-NAO-EXISTE-9182734.bin"),
            Algoritmo::Md5,
        )
        .expect("o certutil roda mesmo com o arquivo ausente");

    assert_ne!(saida.codigo, 0, "o certutil devia recusar");
    assert!(
        resumo::do_certutil(&saida, Algoritmo::Md5).is_err(),
        "um arquivo ausente nao pode produzir resumo"
    );
}

/// Quanto a pasta ocupa, so para escolher a menor.
fn tamanho_da_pasta(caminho: &std::path::Path) -> u64 {
    ArquivosDoSistema
        .listar(caminho)
        .map(|entradas| entradas.iter().map(|item| item.tamanho_bytes).sum())
        .unwrap_or(u64::MAX)
}

/// O `>>` da receita **não** preservou o `arca-check.log`, e o achado fica
/// fixado aqui.
///
/// A E11 decidiu acrescentar em vez de truncar, e escreveu que o resultado
/// seriam **duas** marcas no mesmo arquivo — o caso que o ADR-0003 previu em
/// 22/08. O marco de 23/08 desmentiu: o log saiu com uma execução do
/// `ocs-chkimg`, e o do backup sumiu.
///
/// Este teste compara as duas capturas e cobra que a **diferença** seja a que
/// foi medida. Ele existe para que P-25 não vire folclore: quem for fechá-la
/// precisa saber exatamente o que foi observado, e o oráculo são os dois
/// arquivos que o Clonezilla escreveu.
#[test]
fn o_marco_mediu_que_a_verificacao_substituiu_o_arca_check_log() {
    const ANTES: &str = include_str!("../recursos/capturas/arca-check-2026-08-22_Apps.log");
    const DEPOIS: &str =
        include_str!("../recursos/capturas/arca-check-2026-08-22_Apps-pos-verificacao.log");

    // Toda execução do `ocs-chkimg` abre com esta sequência de escapes. Duas
    // execuções no mesmo arquivo dariam duas; é o que separa append de
    // truncamento sem depender de tamanho.
    const ABERTURA: &str = "\u{1b})0\u{1b}[1;24r";
    let aberturas = |texto: &str| texto.matches(ABERTURA).count();

    assert_eq!(aberturas(ANTES), 1, "a captura do backup tem uma execucao");
    assert_eq!(
        aberturas(DEPOIS),
        1,
        "se um dia isto der 2, o `>>` passou a acrescentar e P-25 mudou de resposta"
    );

    // A marca aparece uma vez em cada, e não duas no segundo.
    assert_eq!(ANTES.matches("ARCA_VEREDITO=").count(), 1);
    assert_eq!(
        DEPOIS.matches("ARCA_VEREDITO=").count(),
        1,
        "duas marcas seria o que a E11 previu e nao aconteceu"
    );

    // E o de depois não contém o de antes: não é append.
    assert!(
        !DEPOIS.starts_with(ANTES),
        "se passar a comecar com o log antigo, o `>>` funcionou e P-25 fecha"
    );
}

/// O desfecho do backup sobreviveu à verificação, e é isso que a pasta própria
/// existe para garantir.
///
/// A decisão de a verificação ser uma [`arca::receita::Operacao`] própria saiu
/// deste risco: toda receita começa truncando o seu `arca-fim.txt` com um `>`,
/// e uma pasta compartilhada faria a verificação apagar o desfecho de um backup
/// ainda não colhido. Em 23/08/2026 as duas operações escreveram, e o teste
/// cobra que os dois desfechos estejam lá — com selos diferentes.
#[test]
fn a_verificacao_nao_encostou_no_desfecho_do_backup() {
    let Some(raiz) = raiz_do_vault() else { return };

    let ler = |pasta: &str| -> Option<String> {
        let caminho = raiz.join("ARCA-LOGS").join(pasta).join("arca-fim.txt");
        ArquivosDoSistema.ler_texto_alheio(&caminho).ok()
    };

    let (Some(backup), Some(verificacao)) = (
        ler("backup-2026-08-22_Apps"),
        ler("verificacao-2026-08-22_Apps"),
    ) else {
        eprintln!("pulado: os dois desfechos de `2026-08-22_Apps` nao estao no dispositivo");
        return;
    };

    assert!(backup.contains("ARCA_BACKUP=OK"), "{backup}");
    assert!(verificacao.contains("ARCA_VERIFY=OK"), "{verificacao}");

    // Selos diferentes: são dois jobs, e cada um tem o seu. Iguais seria o
    // sintoma de um deles ter sido escrito por cima do outro.
    let selo_de = |texto: &str| {
        texto
            .lines()
            .next()
            .and_then(|linha| linha.trim().strip_prefix("ARCA_SELO=").map(str::to_string))
    };
    assert_ne!(selo_de(&backup), selo_de(&verificacao));
    assert_eq!(selo_de(&backup).as_deref(), Some("7d2d2f5153625b38"));
}
