//! Medicao: o `estado.json` no `ARCABOOT` de verdade, que e FAT32.
//!
//! A E4 aprendeu que **o primeiro uso real de uma porta e onde as surpresas
//! moram** — a escrita atomica nunca tinha rodado em producao, e so uma
//! medicao contra o FAT32 mostrou que a sequencia funcionava. A E5 estreia
//! duas coisas pelo mesmo caminho:
//!
//! - **`criar_diretorio`**, que nunca rodou em producao. `R:\arca\` nao existe
//!   num dispositivo preparado a mao, e o desta mesa nao o tinha.
//! - **`BCryptGenRandom`**, de onde sai o selo.
//!
//! Roda com o dispositivo conectado:
//!
//! ```text
//! cargo run --example estado_no_arcaboot
//! ```
//!
//! # O arquivo nao se chama `estado.json`, de proposito
//!
//! Escrever um `estado.json` de mentira no dispositivo criaria um job pendente
//! que nao existe, e `arca status` passaria a anunciar um backup armado que
//! ninguem armou. A medicao usa outro nome e o apaga no fim — e apagar aqui
//! nao fura B-10, que fala de imagem, residuo e log, e cujo teste varre `src/`
//! e nao `examples/`. Ainda assim: o unico arquivo apagado e o que esta funcao
//! acabou de criar.

#[cfg(windows)]
fn main() {
    use arca::adaptadores::windows::entropia::EntropiaDoWindows;
    use arca::adaptadores::{ArquivosDoSistema, RelogioDoSistema};
    use arca::dispositivo;
    use arca::estado::{Estado, MomentoDoArmar, Situacao, gerar_selo};
    use arca::nome::Nome;
    use arca::portas::Arquivos;
    use arca::receita::{Disco, Operacao};

    let arquivos = ArquivosDoSistema;

    let dispositivo =
        match dispositivo::encontrar(&arca::adaptadores::windows::volumes::VolumesDoWindows) {
            Ok(dispositivo) => dispositivo,
            Err(erro) => {
                println!("sem dispositivo conectado: {erro}");
                return;
            }
        };

    let raiz = match dispositivo.raiz_do_boot() {
        Ok(raiz) => raiz,
        Err(erro) => {
            println!("sem ARCABOOT: {erro}");
            return;
        }
    };

    let pasta = raiz.join("arca");
    let alvo = pasta.join("medicao-e5.json");

    println!("ARCABOOT em {}", raiz.display());
    println!(
        "  {} existia antes? {}",
        pasta.display(),
        arquivos.existe(&pasta)
    );

    // 1. `criar_diretorio` no FAT32, pela primeira vez em producao.
    arquivos
        .criar_diretorio(&pasta)
        .expect("criar_diretorio no FAT32");
    println!("  criado ........................ ok");

    // E de novo, porque gravar estado acontece a cada armar e a pasta ja vai
    // existir da segunda vez em diante. Um `criar_diretorio` que falhasse com
    // a pasta pronta quebraria todo armar exceto o primeiro.
    arquivos
        .criar_diretorio(&pasta)
        .expect("criar_diretorio e idempotente");
    println!("  criado de novo (idempotente) .. ok");

    // 2. O selo, da fonte de entropia do Windows.
    let selo = gerar_selo(&EntropiaDoWindows).expect("BCryptGenRandom responde");
    let outro = gerar_selo(&EntropiaDoWindows).expect("BCryptGenRandom responde de novo");
    println!("  selo .......................... {selo}");
    println!("  outro selo .................... {outro}");
    assert_ne!(selo, outro, "dois selos iguais: o gerador nao serve");

    // 3. Ida e volta pelo FAT32.
    let original = Estado {
        selo,
        comando: Operacao::Backup,
        nome: Some(Nome::novo("2026-08-22_Medicao").expect("nome valido")),
        disco: Some(Disco::novo("nvme0n1").expect("disco valido")),
        armado_em: MomentoDoArmar::agora(&RelogioDoSistema),
        situacao: Situacao::Armado,
    };

    let json = original.como_json().expect("os seis campos cabem");
    arquivos
        .escrever_atomico(&alvo, &json)
        .expect("escrita atomica no FAT32");
    println!("  gravado ....................... {}", alvo.display());

    let lido = arquivos.ler_texto(&alvo).expect("leitura");
    let volta = Estado::de_json(&lido).expect("o arquivo se lê de volta");
    assert_eq!(volta, original, "os cinco campos nao voltaram iguais");
    println!("  ida e volta ................... ok, os cinco campos");

    // O LF nao pode virar CRLF pelo caminho: quem lê o arquivo do outro lado
    // e este mesmo codigo, mas um `estado.json` com fim de linha trocado seria
    // um sinal de que a escrita nao e byte a byte.
    assert_eq!(lido, json, "o conteudo mudou entre gravar e lê");
    println!("  byte a byte ................... ok");

    // Nenhum temporario para tras, como a E4 mediu para o `grub.cfg`.
    let restos: Vec<String> = std::fs::read_dir(&pasta)
        .expect("listar a pasta")
        .filter_map(Result::ok)
        .map(|entrada| entrada.file_name().to_string_lossy().into_owned())
        .filter(|nome| nome.contains("arca-tmp"))
        .collect();
    println!(
        "  temporario para tras .......... {}",
        if restos.is_empty() {
            "nenhum".to_string()
        } else {
            format!("SOBROU {restos:?}")
        }
    );
    assert!(restos.is_empty());

    // O truncado tem de ser recusado, e nao lido pela metade — no arquivo de
    // verdade, e nao so na memoria.
    let cortado = &json[..json.len() / 2];
    arquivos
        .escrever_atomico(&alvo, cortado)
        .expect("gravar o pedaco");
    let relido = arquivos.ler_texto(&alvo).expect("leitura");
    match Estado::de_json(&relido) {
        Ok(_) => panic!("um estado.json cortado ao meio passou por bom"),
        Err(recusa) => println!("  truncado recusado ............. {recusa}"),
    }

    std::fs::remove_file(&alvo).expect("tirar o arquivo da medicao");
    println!("  arquivo da medicao removido ... ok");
    println!(
        "\nA pasta {} fica: e onde o estado do job vai morar (§4.1).",
        pasta.display()
    );
}

#[cfg(not(windows))]
fn main() {
    println!("esta medicao so faz sentido no Windows, com o dispositivo conectado");
}
