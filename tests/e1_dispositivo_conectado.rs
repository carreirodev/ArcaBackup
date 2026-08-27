//! A etapa E1 contra o dispositivo de verdade.
//!
//! Os duplos provam que a regra esta certa; este arquivo prova que ela
//! encontra o hardware. Sao coisas diferentes, e a E1 e a primeira etapa que
//! precisa das duas: um `ARCAVAULT` que o `GetVolumeInformationW` devolvesse
//! com um NUL grudado no fim passaria em todo teste com duplo e nao acharia
//! dispositivo nenhum.
//!
//! Nada aqui escreve no dispositivo — nem log (§ "Ainda nao" da E1). E nada
//! aqui precisa de elevacao: ler `E:\` e leitura de arquivo comum. Quem exige
//! privilegio administrativo e o `arca.exe`, pelo manifesto, e este binario de
//! teste nao o carrega.
//!
//! Sem o dispositivo conectado, os testes passam dizendo que pularam. O
//! criterio de aceite da E1 e a execucao do `arca list`, nao este arquivo.

#![cfg(windows)]

use arca::adaptadores::ArquivosDoSistema;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::comandos::list;
use arca::dispositivo::{self, Dispositivo};
use arca::erro::Erro;
use arca::imagens::{self, Especie};
use arca::portas::Arquivos;

/// O dispositivo, ou nada quando ele nao esta plugado.
fn dispositivo_conectado() -> Option<Dispositivo> {
    match dispositivo::encontrar(&VolumesDoWindows) {
        Ok(dispositivo) => Some(dispositivo),
        Err(Erro::DispositivoAusente) => {
            eprintln!("pulado: nenhum dispositivo ARCA conectado");
            None
        }
        Err(outro) => panic!("a descoberta falhou por outro motivo: {outro}"),
    }
}

#[test]
fn o_dispositivo_e_achado_pelo_rotulo_e_tem_caminho() {
    let Some(dispositivo) = dispositivo_conectado() else {
        return;
    };

    assert_eq!(
        dispositivo.vault.rotulo.as_deref(),
        Some(dispositivo::ARCAVAULT)
    );
    assert_eq!(dispositivo.vault.sistema_de_arquivos, "NTFS");
    assert!(dispositivo.vault.total_bytes > 0);

    let raiz = dispositivo.raiz_do_vault().expect("o vault tem letra");
    assert!(
        ArquivosDoSistema.existe(&raiz),
        "{} nao existe",
        raiz.display()
    );
}

#[test]
fn o_que_a_listagem_chama_de_imagem_tem_md5sums_de_verdade() {
    let Some(dispositivo) = dispositivo_conectado() else {
        return;
    };
    let raiz = dispositivo.raiz_do_vault().unwrap();
    let pastas = imagens::enumerar(&ArquivosDoSistema, &raiz).expect("o vault e legivel");

    // **Um `ARCAVAULT` vazio deixou de ser impossível na E10.** Este teste
    // nasceu na E1, quando todo dispositivo desta mesa tinha imagens porque
    // fora preparado à mão com elas dentro — e `o dispositivo tem imagens` era
    // uma afirmação sobre o mundo, não sobre o código.
    //
    // Desde que o `arca prepare` cria dispositivos, um recém-nascido é vazio
    // por construção: ele nem consegue fazer a primeira imagem, porque o nome
    // do disco no Linux sai do `blkdev.list` de dentro de uma imagem (§4.5) e o
    // primeiro backup é pelo menu do Clonezilla (§6.4).
    //
    // O teste sai cedo **dizendo isso**. Um teste de hardware que sai calado é
    // indistinguível de um que passou, e o que ele prova — que o `MD5SUMS`
    // separa imagem de resíduo (B-3) — continua provado assim que houver o que
    // enumerar.
    if pastas.is_empty() {
        eprintln!(
            "  (o {} em {} esta VAZIO — nada a enumerar.\n   \
             E um estado legitimo desde a E10, e nao uma falha: `arca prepare` cria\n   \
             dispositivos sem imagem. Este teste nao conferiu nada.)",
            arca::dispositivo::ARCAVAULT,
            raiz.display()
        );
        return;
    }

    for pasta in &pastas {
        let caminho = raiz.join(&pasta.nome);
        assert!(ArquivosDoSistema.existe(&caminho), "{caminho:?} sumiu");

        let tem_md5sums = ArquivosDoSistema.existe(&caminho.join("MD5SUMS"));
        match pasta.especie {
            // B-3: o `MD5SUMS` e o que separa imagem de residuo, e este e o
            // unico teste que confere isso contra o sistema de arquivos.
            Especie::Imagem { .. } => assert!(tem_md5sums, "{} sem MD5SUMS", pasta.nome),
            Especie::Residuo => assert!(!tem_md5sums, "{} tem MD5SUMS", pasta.nome),
        }

        assert!(
            pasta.tamanho_bytes > 0 || matches!(pasta.especie, Especie::Residuo),
            "{} e imagem e mediu zero byte",
            pasta.nome
        );
    }
}

#[test]
fn as_pastas_de_servico_do_dispositivo_ficam_fora_da_listagem() {
    let Some(dispositivo) = dispositivo_conectado() else {
        return;
    };
    let raiz = dispositivo.raiz_do_vault().unwrap();
    let pastas = imagens::enumerar(&ArquivosDoSistema, &raiz).unwrap();

    // `ARCA-LOGS` esta no §4 do PRD e existe neste dispositivo. Se ela
    // aparecesse como residuo, a listagem estaria oferecendo ao usuario que
    // apagasse os desfechos de todos os backups.
    for servico in ["ARCA-LOGS", "System Volume Information", "$RECYCLE.BIN"] {
        assert!(
            !pastas.iter().any(|pasta| pasta.nome == servico),
            "{servico} apareceu na listagem"
        );
    }
}

#[test]
fn a_saida_tem_a_forma_do_paragrafo_5_4() {
    let Some(dispositivo) = dispositivo_conectado() else {
        return;
    };
    let raiz = dispositivo.raiz_do_vault().unwrap();
    let pastas = imagens::enumerar(&ArquivosDoSistema, &raiz).unwrap();

    let saida = list::montar(
        &pastas,
        dispositivo.vault.livre_bytes,
        list::Descricoes::Mostrar,
    );
    eprintln!("\n{saida}");

    let mut linhas = saida.lines();
    assert_eq!(
        linhas.next(),
        Some(if pastas.is_empty() {
            "Nenhuma imagem em ARCAVAULT."
        } else {
            "Imagens em ARCAVAULT:"
        })
    );

    for pasta in &pastas {
        let linha = linhas.next().expect("uma linha por pasta");
        assert!(linha.starts_with(&format!("  {}", pasta.nome)), "{linha}");
        assert_eq!(
            linha.matches(" · ").count(),
            2,
            "faltou separador em: {linha}"
        );

        // A descricao de L-3 e texto que alguem escreveu num bloco de notas, e
        // ocupa quantas linhas recuadas precisar. O que este teste afirma e a
        // forma da linha **da imagem**, e por isso as dela sao consumidas
        // aqui: a linha da imagem comeca com dois espacos e o nome.
        if pasta.descricao.is_some() {
            while linhas
                .clone()
                .next()
                .is_some_and(|proxima| proxima.starts_with("      "))
            {
                linhas.next();
            }
        }
    }

    assert_eq!(linhas.next(), Some(""));
    assert!(
        linhas.next().is_some_and(|fim| fim.ends_with(" GB livres")),
        "o espaco livre tem de fechar a listagem"
    );
    assert!(linhas.next().is_none(), "sobrou saida depois do rodape");
}

#[test]
fn a_leitura_do_dispositivo_nao_deixa_rastro() {
    // A E1 so lê. Nada de `arca-*.tmp`, nada de pasta nova, nada de log no
    // dispositivo — o registro do lado Windows mora em `%LOCALAPPDATA%`.
    let Some(dispositivo) = dispositivo_conectado() else {
        return;
    };
    let raiz = dispositivo.raiz_do_vault().unwrap();

    let antes = ArquivosDoSistema.listar(&raiz).unwrap();
    let _ = imagens::enumerar(&ArquivosDoSistema, &raiz).unwrap();
    let depois = ArquivosDoSistema.listar(&raiz).unwrap();

    let nomes = |entradas: &[arca::portas::Entrada]| -> Vec<String> {
        entradas.iter().map(|entrada| entrada.nome()).collect()
    };
    assert_eq!(nomes(&antes), nomes(&depois));
}
