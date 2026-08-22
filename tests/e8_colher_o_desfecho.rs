//! A etapa E8 contra o dispositivo de verdade.
//!
//! O julgamento do desfecho e testado em `src/desfecho.rs` contra texto, e a
//! colheita em `src/comandos/resultado.rs` contra duplos. O que este arquivo
//! prova e o que nenhum dos dois pode: que os arquivos de que aquelas
//! decisoes falam **continuam sendo** o que esta no dispositivo.
//!
//! # Nenhum teste daqui escreve
//!
//! Nem no `estado.json`, nem no `grub.cfg`, nem no `ARCAVAULT`. Quem colhe e
//! `arca resultado`, e colher desarma.

#![cfg(windows)]

use arca::adaptadores::ArquivosDoSistema;
use arca::adaptadores::windows::volumes::VolumesDoWindows;
use arca::desfecho::{self, Julgamento, NaoEDesfecho};
use arca::dispositivo::{self, Dispositivo};
use arca::estado;
use arca::imagens;
use arca::nome::Nome;
use arca::portas::Arquivos;
use arca::receita::{Operacao, Selo};
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

#[test]
fn o_unico_arca_fim_do_dispositivo_continua_sem_selo() {
    // P-16 fixado no hardware. O `arca-fim.txt` que existe neste dispositivo
    // veio do trabalho **manual** de validacao, e nao de receita nenhuma:
    // `ARCA_RESTORE=OK` e `ARCA_FIM`, sem `ARCA_SELO=`. E o arquivo que deu
    // origem a linha nova do §5.5 na etapa E5.
    //
    // Enquanto ele for a unica coisa que o mecanismo de desfecho produziu
    // neste dispositivo, P-16 continua aberta — e este teste e o que diz
    // quando ela fechar: a partir do primeiro `arca backup` colhido, havera um
    // segundo `arca-fim.txt`, esse sim com selo.
    let Some(raiz) = raiz_do_vault() else {
        return;
    };

    let caminho = raiz
        .join("ARCA-LOGS")
        .join("2026-08-21_WindowsCompleto")
        .join("arca-fim.txt");

    if !ArquivosDoSistema.existe(&caminho) {
        eprintln!("pulado: {} nao existe", caminho.display());
        return;
    }

    let texto = ArquivosDoSistema
        .ler_texto_alheio(&caminho)
        .expect("o arca-fim.txt e legivel");

    let lido = desfecho::ler(&texto);
    assert_eq!(lido.linhas_de_selo, 0, "ele ganhou um selo: {texto:?}");
    assert!(lido.fim, "ele perdeu o ARCA_FIM: {texto:?}");
    assert!(lido.deu_certo, "ele perdeu o ARCA_RESTORE=OK: {texto:?}");

    // E o julgamento que o §5.5 manda dar sobre ele: nao e "o selo nao bate",
    // porque nao ha selo a bater.
    let qualquer = Selo::novo("a3f1c9e07b2d4856").unwrap();
    assert_eq!(
        desfecho::julgar(&lido, &qualquer),
        Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo)
    );
}

#[test]
fn aquele_arca_fim_continua_inalcancavel_pelo_arca_de_hoje() {
    // A E3 decidiu que a pasta do log leva a **operacao** no nome, e por isso
    // o ARCA de hoje nunca vai olhar para `ARCA-LOGS\2026-08-21_WindowsCompleto\`.
    // O teste existe porque a afirmacao aparece em tres lugares do codigo e
    // ninguem a tinha conferido contra o disco.
    let Some(raiz) = raiz_do_vault() else {
        return;
    };

    let nome = Nome::novo("2026-08-21_WindowsCompleto").expect("nome valido");
    for operacao in [Operacao::Backup, Operacao::Restauracao] {
        let onde = estado::caminho_do_desfecho(&raiz, operacao, &nome);
        assert!(
            !ArquivosDoSistema.existe(&onde),
            "o ARCA de hoje alcanca {}, e o §5.5 supoe que nao",
            onde.display()
        );
    }
}

#[test]
fn nao_ha_job_pendente_neste_dispositivo() {
    // O estado normal, e o que a E8 pressupoe. Um `estado.json` sobrando aqui
    // seria um job armado que ninguem colheu — ou um teste que escreveu onde
    // nao devia.
    let Some(dispositivo) = dispositivo() else {
        return;
    };
    let Ok(caminho) = dispositivo.caminho_do_estado() else {
        eprintln!("pulado: sem ARCABOOT com letra");
        return;
    };

    if !ArquivosDoSistema.existe(&caminho) {
        return;
    }

    // Havendo estado, ele tem de ser legivel e dizer a que situacao pertence.
    // Um `estado.json` que este binario nao lê e pior do que nenhum: ele diria
    // que ha job pendente sem dizer qual.
    let lido = estado::ler(&ArquivosDoSistema, &caminho)
        .expect("o estado.json do dispositivo tem de ser legivel por este binario");

    eprintln!(
        "ha estado no dispositivo: {} `{}` · selo {} · {}",
        lido.comando.nome(),
        lido.nome,
        lido.selo,
        lido.situacao
    );
}

#[test]
fn as_imagens_do_dispositivo_tem_veredito_legivel() {
    // O veredito e metade da §5.4, e ele sai de `arca-check.log` que outra
    // coisa escreveu. Um leitor que deixasse de reconhecer aquelas duas formas
    // faria a colheita dizer "sem veredito" para uma imagem aprovada — que e
    // o modo de falha que o ADR-0003 escolheu de proposito, e por isso mesmo
    // e silencioso.
    let Some(raiz) = raiz_do_vault() else {
        return;
    };

    let pastas = imagens::enumerar(&ArquivosDoSistema, &raiz).expect("o ARCAVAULT e legivel");
    let imagens: Vec<_> = pastas.iter().filter(|pasta| pasta.e_imagem()).collect();

    if imagens.is_empty() {
        eprintln!("pulado: nao ha imagem no dispositivo");
        return;
    }

    for imagem in &imagens {
        eprintln!("{}: {:?}", imagem.nome, imagem.especie);
    }

    assert!(
        imagens.iter().any(|imagem| matches!(
            imagem.especie,
            imagens::Especie::Imagem {
                veredito: Some(imagens::Veredito::Aprovada)
            }
        )),
        "nenhuma das {} imagens deste dispositivo tem veredito de aprovacao, e as duas formas do \
         ADR-0003 estavam aqui em 22/08/2026",
        imagens.len()
    );
}
