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
fn o_arca_fim_de_21_08_continua_sem_selo() {
    // O `arca-fim.txt` de 21/08 veio do trabalho **manual** de validacao, e
    // nao de receita nenhuma: `ARCA_RESTORE=OK` e `ARCA_FIM`, sem
    // `ARCA_SELO=`. E o arquivo que deu origem a linha nova do §5.5 na E5.
    //
    // **Este teste se chamava "o unico", e ele deixou de ser o unico em
    // 22/08/2026.** O comentario antigo previa o que aconteceria quando P-16
    // fechasse — "a partir do primeiro `arca backup` colhido havera um segundo
    // `arca-fim.txt`, esse sim com selo" — e e exatamente o que ha. O de 21/08
    // continua aqui e continua sem selo; o do marco esta no teste seguinte.
    // Os dois lado a lado sao a diferenca entre o que uma pessoa escreveu e o
    // que a receita escreve.
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
fn o_desfecho_do_marco_e_julgado_como_operacao_concluida() {
    // **P-16 fechada, fixada.** O primeiro `arca-fim.txt` que uma receita do
    // ARCA escreveu, colhido em 22/08/2026 as 21:14:49. Tres linhas, e cada
    // uma e um pedaco de codigo que nunca tinha rodado — o selo, o desfecho e
    // o `ARCA_FIM`.
    //
    // O teste corre contra a **captura**, e nao contra o dispositivo, de
    // proposito: o arquivo no `ARCAVAULT` sera truncado pelo proximo
    // `arca backup 2026-08-22_Apps`, porque toda receita comeca por
    // `echo ARCA_SELO=… >` e o `>` trunca ao abrir. O original preservado e o
    // que continua provando depois disso.
    //
    // E o que ele prova nao e so a forma do arquivo: e que o julgamento da E5
    // — escrito inteiro contra texto inventado e duplos — classifica o
    // primeiro original de verdade no ramo certo.
    const DO_MARCO: &str = include_str!("../recursos/capturas/arca-fim-2026-08-22_Apps.txt");

    // O selo que o `estado.json` do job trazia, e que o `arca.log` registrou
    // ao armar as 20:53:48. Conferido a olho contra a primeira linha do
    // arquivo antes de qualquer conclusao sobre o marco — que e o que a etapa
    // pedia, e o que este teste passa a fazer sozinho.
    let selo_do_job = Selo::novo("7d2d2f5153625b38").expect("selo valido");

    let lido = desfecho::ler(DO_MARCO);
    assert_eq!(
        lido.linhas_de_selo, 1,
        "o desfecho do marco tem de ter exatamente uma linha de selo: {DO_MARCO:?}"
    );
    assert!(lido.fim, "o desfecho do marco perdeu o ARCA_FIM: {DO_MARCO:?}");
    assert!(
        lido.deu_certo,
        "o desfecho do marco perdeu o ARCA_BACKUP=OK: {DO_MARCO:?}"
    );

    assert_eq!(
        desfecho::julgar(&lido, &selo_do_job),
        Julgamento::Concluida,
        "o primeiro desfecho real do ARCA nao foi julgado como concluido"
    );

    // E o outro lado, que e o que o selo existe para fazer: o mesmo arquivo,
    // cobrado por outro job, e job fantasma — e nao "concluida".
    let de_outro = Selo::novo("a3f1c9e07b2d4856").expect("selo valido");
    assert!(
        matches!(
            desfecho::julgar(&lido, &de_outro),
            Julgamento::JobFantasma { .. }
        ),
        "o desfecho do marco foi aceito por um job que nao e o dele"
    );
}

#[test]
fn o_arca_check_log_do_marco_traz_as_duas_formas_do_adr_0003() {
    // O ADR-0003 achou o `arca-check.log` em **duas** formas no dispositivo: o
    // marcador `ARCA_VEREDITO=` da `2026-08-21_WindowsCompleto`, que veio de
    // um script de validacao manual, e a saida crua do `ocs-chkimg` da
    // `ARCA-TESTE-03`, que e o que a receita produzia. Ele decidiu ler as duas
    // e previu que a E3, ao acrescentar a linha, faria o marcador virar a
    // forma preferida.
    //
    // **O marco mostrou que a receita produz as duas no mesmo arquivo**, e e
    // uma coisa que nenhuma das duas capturas antigas mostrava: a saida crua
    // inteira, com os escapes de terminal do partclone, e o
    // `ARCA_VEREDITO=APROVADA` acrescentado no fim. O `ARCA_VEREDITO=` deixou
    // de ser codigo sem original.
    const DO_MARCO: &str = include_str!("../recursos/capturas/arca-check-2026-08-22_Apps.log");

    assert_eq!(
        imagens::interpretar_veredito(DO_MARCO),
        Some(imagens::Veredito::Aprovada),
        "o veredito escrito pela receita nao foi lido como aprovacao"
    );

    // As duas formas estao ali, e e por isso que este arquivo prova mais do
    // que qualquer das duas capturas antigas.
    assert!(
        DO_MARCO.contains("ARCA_VEREDITO=APROVADA"),
        "o marcador que a receita acrescenta sumiu do log"
    );
    assert!(
        DO_MARCO
            .to_lowercase()
            .contains("were checked and are restorable"),
        "o resumo cru do ocs-chkimg sumiu do log"
    );

    // E nao ha sinal de reprovacao — se houvesse, o ADR-0003 manda reprovar,
    // e este teste estaria afirmando o contrario do que o arquivo diz.
    assert!(
        !DO_MARCO.to_lowercase().contains("not restorable"),
        "o log do marco tem sinal de reprovacao, e a imagem foi dada como aprovada"
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
        let onde = estado::caminho_do_desfecho(&raiz, operacao, Some(&nome));
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
        "ha estado no dispositivo: {} · selo {} · {}",
        lido.descricao(),
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
