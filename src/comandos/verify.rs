//! `arca verify <nome>` — conferir uma imagem sem reiniciar (V-1), e
//! `--completo` para a verificacao armada (V-2).
//!
//! # Os dois comandos sao um comando so ate a escolha, e depois nao sao nada
//! parecidos
//!
//! Ate achar a imagem, os dois fazem o mesmo: localizar o dispositivo (B-1),
//! enumerar, recusar residuo (L-2). Dali em diante:
//!
//! - **V-1 lê.** Nao escreve, nao arma, nao reinicia, nao desarma. E um
//!   comando de consulta, como o `arca list` — e por isso C-1 nao se aplica:
//!   ele fala dos comandos que **armam**, e o mesmo raciocinio ja esta
//!   registrado em [`super::resultado`].
//! - **V-2 arma.** Desarma primeiro (C-1), pede a confirmacao digitada, arma,
//!   avisa C-9 e reinicia. E o mecanismo da E7 inteiro, com uma receita menor.
//!
//! # Por que `--completo` pede confirmacao se nao destroi nada
//!
//! S-2 fala de operacao destrutiva, e verificar nao destroi. O `arca backup`
//! tambem nao destroi — B-10 — e pede assim mesmo, e a razao vale igual aqui:
//! **a maquina vai reiniciar e desligar sozinha.** Quem digitou
//! `arca verify X --completo` sem ler o `--completo` esta a um Enter de perder
//! o que estiver aberto, e a confirmacao e o que separa as duas coisas.
//!
//! # O veredito de V-1 nao entra na listagem, e a tela diz isso quando importa
//!
//! A coluna `aprovada` do `arca list` sai do `arca-check.log`, que e o parecer
//! do `ocs-chkimg` — *"esta imagem e restauravel?"*. V-1 responde outra
//! pergunta: *"os bytes que estao aqui sao os que o Clonezilla gravou?"*.
//! Escrever uma reprovacao de V-1 naquele arquivo faria a listagem afirmar que
//! o `ocs-chkimg` reprovou, e ele nem rodou.
//!
//! Entao V-1 imprime e **registra no `arca.log`** — o registro do lado Windows,
//! que todo comando ja alimenta —, e nao toca no `arca-check.log`. Quando
//! reprova, a tela diz que aquela reprovacao nao vai aparecer no `arca list`,
//! porque quem lê precisa saber que a listagem vai continuar dizendo outra
//! coisa. Quando aprova, o aviso nao aparece: conselho que sai sempre vira
//! ruido, e a E10 ja pagou por essa licao no `arca resultado`.

use crate::app::Contexto;
use crate::armar;
use crate::desarme;
use crate::dispositivo::{self, Dispositivo};
use crate::erro::{Erro, Resultado};
use crate::formato::{dia_e_mes, gigabytes, linha, tamanho};
use crate::imagens::{self, Especie, Pasta, Veredito};
use crate::md5sums;
use crate::nome::Nome;
use crate::receita::{Operacao, Pedido, Receita, Selo};
use crate::verificacao::{self, Andamento, Conferencia, Plano};
use std::fmt;
use std::path::Path;

/// Por que nao ha o que verificar.
///
/// Toda variante acontece **antes** de qualquer escrita e antes da confirmacao
/// digitada — a mesma regra que o `arca restore` segue desde a E9: ninguem
/// digita o nome inteiro de uma imagem para ouvir um nao depois.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDaVerificacao {
    /// Nao ha pasta com esse nome no `ARCAVAULT`.
    NaoExiste { nome: String },

    /// A pasta existe e nao tem `MD5SUMS`: e residuo (L-2, B-3).
    EResiduo { nome: String },

    /// O `MD5SUMS` esta la e nao serve.
    Md5sumsRecusado(md5sums::RecusaDoMd5sums),
}

impl fmt::Display for RecusaDaVerificacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDaVerificacao::NaoExiste { nome } => write!(
                f,
                "nao ha imagem chamada `{nome}` no ARCAVAULT. Rode `arca list` para ver o que ha"
            ),
            RecusaDaVerificacao::EResiduo { nome } => write!(
                f,
                "`{nome}` nao e imagem, e residuo: nao ha `MD5SUMS` na pasta, o que e o rastro de um backup interrompido (L-2). Nao ha o que conferir, porque nao ha contra o que conferir"
            ),
            RecusaDaVerificacao::Md5sumsRecusado(recusa) => write!(f, "{recusa}"),
        }
    }
}

pub fn executar(contexto: &Contexto, nome_bruto: &str, completo: bool) -> Resultado<()> {
    // B-2 primeiro, e antes de tocar no dispositivo: um nome recusado nao
    // precisa de SSD conectado para ser recusado. E o mesmo comeco do
    // `arca backup`.
    let nome = Nome::novo(nome_bruto).map_err(Erro::NomeRecusado)?;

    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;

    let pasta = achar(&pastas, &nome).map_err(Erro::VerificacaoRecusada)?;

    if completo {
        return armada(contexto, &dispositivo, &nome, pasta);
    }
    aqui_mesmo(contexto, &dispositivo, &nome, pasta, &raiz_do_vault)
}

/// A imagem pedida, ou por que ela nao serve.
fn achar<'a>(pastas: &'a [Pasta], nome: &Nome) -> Result<&'a Pasta, RecusaDaVerificacao> {
    let Some(pasta) = pastas
        .iter()
        .find(|pasta| pasta.nome.eq_ignore_ascii_case(nome.como_texto()))
    else {
        return Err(RecusaDaVerificacao::NaoExiste {
            nome: nome.to_string(),
        });
    };

    // L-2: residuo nunca e oferecido para restaurar, e aqui nao ha o que
    // conferir — o `MD5SUMS` e justamente o que falta. E a recusa vem por
    // este caminho, e nao por "nao achei o MD5SUMS", porque residuo tem nome
    // e o nome diz o que aconteceu: um backup foi interrompido.
    if matches!(pasta.especie, Especie::Residuo) {
        return Err(RecusaDaVerificacao::EResiduo {
            nome: nome.to_string(),
        });
    }

    Ok(pasta)
}

/// V-1: confere aqui mesmo, lendo os arquivos.
fn aqui_mesmo(
    contexto: &Contexto,
    dispositivo: &Dispositivo,
    nome: &Nome,
    pasta: &Pasta,
    raiz_do_vault: &Path,
) -> Resultado<()> {
    let pasta_da_imagem = raiz_do_vault.join(&pasta.nome);
    let caminho_do_md5sums = pasta_da_imagem.join(md5sums::ARQUIVO);

    let texto = contexto.arquivos.ler_texto_alheio(&caminho_do_md5sums)?;
    let entradas = md5sums::ler(&texto).map_err(|recusa| {
        Erro::VerificacaoRecusada(RecusaDaVerificacao::Md5sumsRecusado(recusa))
    })?;

    // O plano mede antes de conferir: e o que permite a tela dizer quanto vai
    // demorar em vez de ficar tres minutos parada sem explicacao, e e de onde
    // sai a largura da coluna do andamento. Ver [`verificacao::Plano`].
    let plano = verificacao::planejar(contexto.arquivos, &pasta_da_imagem, &entradas)?;

    print!(
        "{}",
        montar_cabecalho(dispositivo, pasta, &plano, &caminho_do_md5sums)
    );

    // O `--dry-run` aqui nao tem o que ensaiar: este comando nao escreve nada
    // e nao arma nada. Ele diz o que faria e para — porque um `--dry-run` que
    // lê 39,7 GB seria a execucao inteira com outro nome.
    if contexto.dry_run {
        println!(
            "\nEnsaio: nada foi conferido. A conferencia lê os {} arquivos inteiros,\ne e a unica coisa que este comando faz.\n",
            plano.quantos()
        );
        return Ok(());
    }

    print!("{}", montar_o_aviso_da_espera(&plano));

    let conferencia = verificacao::conferir(
        contexto.arquivos,
        contexto.sistema,
        &pasta_da_imagem,
        &plano,
        &mut |andamento| print!("{}", montar_andamento(andamento)),
    )?;

    contexto.registro.info(format!(
        "verificado `{nome}` · {} de {} arquivos batem · {} lidos · veredito {:?}",
        conferencia.quantos() - conferencia.falhas().len(),
        conferencia.quantos(),
        tamanho(conferencia.bytes_lidos),
        conferencia.veredito()
    ));

    print!("{}", montar_o_veredito(&conferencia));

    // S-5, e o mesmo contrato do `arca resultado`: quem chamou o ARCA de um
    // script nao pode lê uma imagem reprovada como exito.
    if conferencia.veredito() == Veredito::Reprovada {
        return Err(Erro::ImagemReprovada {
            nome: nome.to_string(),
            quantos: conferencia.falhas().len(),
        });
    }
    Ok(())
}

/// O cabecalho de V-1, antes de a conferencia comecar.
pub fn montar_cabecalho(
    dispositivo: &Dispositivo,
    pasta: &Pasta,
    plano: &Plano,
    caminho_do_md5sums: &Path,
) -> String {
    let mut saida = String::new();

    saida.push_str(&format!(
        "\nDispositivo ARCA: {} ({}) · {} livres\n",
        dispositivo::ARCAVAULT,
        dispositivo
            .vault
            .letra
            .map_or("sem letra".to_string(), |letra| format!("{letra}:")),
        gigabytes(dispositivo.vault.livre_bytes)
    ));
    saida.push_str(&format!(
        "Imagem: {} · {} · {}\n\n",
        pasta.nome,
        dia_e_mes(pasta.modificado_em),
        tamanho(pasta.tamanho_bytes)
    ));

    saida.push_str(&linha(
        "MD5SUMS lido",
        &format!(
            "{} arquivos · {}",
            plano.quantos(),
            caminho_do_md5sums.to_string_lossy()
        ),
    ));
    // O tamanho da pasta e o que se vai lê nao sao o mesmo numero, e a
    // diferenca e exatamente `fora_do_md5sums`. Sao duas linhas de proposito:
    // a de cima e a imagem, e esta e o trabalho.
    saida.push_str(&linha("A conferir", &tamanho(plano.bytes_totais)));

    saida
}

/// O aviso de que a tela vai ficar parada, com quanto tempo.
///
/// A estimativa sai da taxa medida em 23/08/2026 — ver
/// [`verificacao::estimar`]. Ela nao precisa acertar o segundo; o que ela evita
/// e a tela prometer o tempo desta imagem para qualquer imagem, que era o que
/// a primeira versao fazia.
pub fn montar_o_aviso_da_espera(plano: &Plano) -> String {
    let segundos = verificacao::estimar(plano.bytes_totais).as_secs();

    format!(
        "\nConferindo {} arquivos · {}. Estimativa: {}.\nA tela vai andando um arquivo por vez — parada nao e travamento.\n\n",
        plano.quantos(),
        tamanho(plano.bytes_totais),
        duracao(segundos)
    )
}

/// `3 min 23 s`, `47 s`, `1 h 4 min`.
///
/// Sem casa decimal e sem `0 min`: e uma estimativa, e uma estimativa com
/// precisao falsa convida a ser cobrada.
fn duracao(segundos: u64) -> String {
    match segundos {
        0..=59 => format!("{segundos} s"),
        60..=3599 => match (segundos / 60, segundos % 60) {
            (minutos, 0) => format!("{minutos} min"),
            (minutos, resto) => format!("{minutos} min {resto} s"),
        },
        _ => match (segundos / 3600, (segundos % 3600) / 60) {
            (horas, 0) => format!("{horas} h"),
            (horas, minutos) => format!("{horas} h {minutos} min"),
        },
    }
}

/// Uma linha de andamento.
///
/// Existe porque a conferencia de 39,7 GB leva 3 min 23 s, e uma tela parada
/// durante tres minutos e indistinguivel de um comando travado.
///
/// # A coluna vem da lista, e nao de [`crate::formato::linha`]
///
/// Aquela funcao tem coluna fixa em 33, medida nas linhas do §5.2 do PRD, e
/// deixa o rotulo **estourar** quando nao cabe — o que esta certo para um
/// rotulo excepcional. Aqui o rotulo e um nome de arquivo do Clonezilla, e
/// `nvme0n1p3.ntfs-ptcl-img.zst.aa` tem trinta caracteres: com o contador na
/// frente, quatorze das trinta e nove linhas estouravam, e a coluna deixava de
/// existir justamente na parte da lista que demora.
///
/// Achado **rodando o comando de verdade**, com a suite verde — como na E6, na
/// E7, na E9 e na E10.
pub fn montar_andamento(andamento: &Andamento) -> String {
    let largura_do_contador = andamento.total.to_string().chars().count();
    let rotulo = format!(
        "[{:>largura_do_contador$}/{}] {}",
        andamento.numero, andamento.total, andamento.arquivo
    );

    // A coluna cabe no maior rotulo possivel desta lista, com dois pontos de
    // folga. O rotulo maximo e `[` + contador + `/` + total + `]` + espaco +
    // o maior nome, ou seja `2L + 4 + N`; a folga e o que impede a linha mais
    // longa de sair com um ponto so, que e o que a coluna fixa fazia em
    // quatorze das trinta e nove.
    const FOLGA: usize = 2;
    let coluna = largura_do_contador * 2 + 4 + andamento.largura_do_nome + FOLGA;
    let pontos = coluna.saturating_sub(rotulo.chars().count()).max(FOLGA);

    format!(
        "  {rotulo} {} {}\n",
        ".".repeat(pontos),
        andamento.conferido.achado
    )
}

/// O fecho de V-1: o que se conferiu e o veredito.
pub fn montar_o_veredito(conferencia: &Conferencia) -> String {
    let mut saida = String::from("\n");

    let falhas = conferencia.falhas();

    saida.push_str(&linha(
        "Conferidos",
        &format!(
            "{} de {} · {} lidos",
            conferencia.quantos() - falhas.len(),
            conferencia.quantos(),
            tamanho(conferencia.bytes_lidos)
        ),
    ));

    // A contagem sai sempre, e nunca como problema: e a hora em que cada
    // arquivo nasceu. Ver [`Conferencia::fora_do_md5sums`].
    if conferencia.fora_do_md5sums > 0 {
        saida.push_str(&linha(
            "Fora do MD5SUMS",
            &format!(
                "{} arquivos · normal — o proprio MD5SUMS e o que nasce depois dele",
                conferencia.fora_do_md5sums
            ),
        ));
    }

    saida.push_str(&linha(
        "Veredito",
        match conferencia.veredito() {
            Veredito::Aprovada => "APROVADA — os bytes sao os que o Clonezilla gravou",
            Veredito::Reprovada => "REPROVADA",
        },
    ));

    if !falhas.is_empty() {
        saida.push_str("\nO que nao bateu:\n");
        for falha in &falhas {
            saida.push_str(&format!("  {} · {}\n", falha.arquivo, falha.achado));
        }

        // O alcance desta reprovacao, dito onde ele importa. Ver o cabecalho
        // deste modulo: a coluna do `arca list` e o parecer do `ocs-chkimg`, e
        // esta conferencia respondeu outra pergunta.
        saida.push_str(concat!(
            "\n  ESTA REPROVACAO NAO APARECE NO `arca list`. A coluna de la e o veredito\n",
            "  do `ocs-chkimg`, que responde `esta imagem e restauravel?`, e ele nao\n",
            "  rodou agora — o que se conferiu aqui foi se os bytes no dispositivo sao\n",
            "  os que o Clonezilla gravou. A imagem vai continuar aparecendo com o\n",
            "  veredito do backup que a criou.\n",
            "\n",
            "  O ARCA nao apaga nada (B-10). Quem decide o que fazer com esta imagem e\n",
            "  voce, e um backup novo e o caminho normal.\n"
        ));
    } else {
        // O que V-1 **nao** respondeu, dito uma vez e sem alarme: a diferenca
        // entre as duas verificacoes e o motivo de V-2 existir ao lado.
        saida.push_str(concat!(
            "\n  Isto conferiu que os bytes nao mudaram desde o backup. NAO conferiu que a\n",
            "  imagem e restauravel — quem responde isso e o `ocs-chkimg`, e para isso ha\n",
            "  `arca verify <nome> --completo`, que custa um reinicio.\n"
        ));
    }

    saida
}

/// V-2: arma o boot unico que so roda o `ocs-chkimg` e desliga.
fn armada(
    contexto: &Contexto,
    dispositivo: &Dispositivo,
    nome: &Nome,
    pasta: &Pasta,
) -> Resultado<()> {
    let caminho_do_grub = dispositivo.caminho_do_grub()?;

    // C-1, incondicionalmente e como primeiro passo — este comando arma.
    let desarme = if contexto.dry_run {
        None
    } else {
        Some(desarme::executar(
            contexto.arquivos,
            contexto.firmware,
            &caminho_do_grub,
        )?)
    };

    // O que ja aconteceu, impresso **antes** de qualquer recusa poder cortar a
    // saida. E a armadilha que a revisao da E7 pegou no `arca backup` e a E9
    // cometeu de novo no `arca restore`: a recusa engolindo a noticia do
    // desarmar.
    print!(
        "\nDispositivo ARCA: {} ({}) · {} livres\n\n",
        dispositivo::ARCAVAULT,
        dispositivo
            .vault
            .letra
            .map_or("sem letra".to_string(), |letra| format!("{letra}:")),
        gigabytes(dispositivo.vault.livre_bytes)
    );
    print!(
        "{}",
        desarme::linha_do_desarme(desarme.as_ref(), &caminho_do_grub.to_string_lossy())
    );
    print!(
        "{}",
        linha(
            "Imagem",
            &format!(
                "{} · {} · {}",
                pasta.nome,
                dia_e_mes(pasta.modificado_em),
                tamanho(pasta.tamanho_bytes)
            )
        )
    );
    print!(
        "{}",
        linha(
            "Veredito de hoje",
            match &pasta.especie {
                Especie::Imagem {
                    veredito: Some(Veredito::Aprovada),
                } => "aprovada · o `ocs-chkimg` vai escrever o parecer novo ao lado",
                Especie::Imagem {
                    veredito: Some(Veredito::Reprovada),
                } => "REPROVADA · e uma reprovacao antiga nao e apagada por uma aprovacao nova",
                _ => "sem veredito · esta sera a primeira vez",
            }
        )
    );

    if contexto.dry_run {
        print!("{}", ensaio_da_receita(nome)?);
        return Ok(());
    }

    println!(
        concat!(
            "\nA verificacao completa reinicia a maquina, roda o `ocs-chkimg` e desliga.\n",
            "Ela NAO substitui a verificacao de todo backup (B-9), e nao destroi nada —\n",
            "mas a maquina desliga, e o que estiver aberto se perde.\n",
            "Na imagem de 39,7 GB desta mesa o `ocs-chkimg` levou 5 min 12 s.\n",
            "\n",
            "Sem reiniciar, `arca verify {}` confere os MD5SUMS em 3 min 23 s."
        ),
        nome
    );

    // S-2: texto digitado, nunca `s`. Nao porque destrua — nao destroi —, mas
    // porque a maquina vai desligar. Ver o cabecalho deste modulo.
    crate::confirmacao::pedir(contexto, "Digite o nome da imagem para confirmar", nome)?;

    let armado = armar::executar(
        contexto.arquivos,
        contexto.firmware,
        contexto.entropia,
        contexto.relogio,
        &armar::Pedir {
            dispositivo,
            operacao: Operacao::Verificacao,
            nome,
            // A verificacao nao nomeia disco: o `ocs-chkimg` opera sobre a
            // imagem. Quem cobra essa coerencia e `Receita::montar`.
            disco: None,
        },
    )?;

    contexto.registro.info(format!(
        "armada verificacao de `{nome}` · selo {} · desfecho em {}",
        armado.selo, armado.pasta_do_desfecho
    ));

    print!("{}", montar_o_armado(&armado));

    contexto.sistema.reiniciar().inspect_err(|_| {
        eprintln!(
            "\nO dispositivo FICOU ARMADO e a maquina nao reiniciou. O proximo reinicio,\n\
             seja qual for a causa, vai bootar no dispositivo e rodar a verificacao.\n\
             Para desfazer:  arca desarmar"
        );
    })
}

/// O que se imprime depois de armado, com o aviso de C-9 no fim.
///
/// As cinco linhas do meio sao as mesmas dos outros dois comandos que armam —
/// [`armar::montar_as_linhas`] —, e o que muda e o que vem depois. Aqui o
/// aviso e mais curto do que o do `arca restore`: nada esta sendo apagado, e a
/// janela do ADR-0009 leva a uma verificacao, e nao a uma restauracao.
pub fn montar_o_armado(armado: &armar::Armado) -> String {
    let mut saida = String::from("\n");
    saida.push_str(&armar::montar_as_linhas(armado));

    // O que se vê do outro lado do reinício é igual nos três comandos que
    // armam, e mora em [`armar::montar_o_que_vem_pela_frente`] desde a E11 —
    // ver lá por que ele existe, e foi **esta** operação que o produziu.
    saida.push_str(armar::montar_o_que_vem_pela_frente());

    saida.push_str(concat!(
        "\nAO TERMINAR: remova o SSD antes de religar.\n",
        "\nDepois de religar, `arca resultado` colhe o veredito.\n",
        "\nReiniciando...\n"
    ));

    saida
}

/// A receita inteira, so no `--dry-run`.
fn ensaio_da_receita(nome: &Nome) -> Resultado<String> {
    // O selo de verdade nasce ao armar. Este e de ensaio, e a saida o diz.
    let receita = Receita::montar(&Pedido {
        operacao: Operacao::Verificacao,
        nome: nome.clone(),
        disco: None,
        selo: Selo::de_ensaio(),
    })
    .map_err(Erro::ReceitaRecusada)?;

    Ok(format!(
        concat!(
            "\nEnsaio: nada foi armado e o dispositivo nao foi desarmado.\n",
            "\nA receita da verificacao, como iria para o grub.cfg:\n\n{}\n",
            "\nO selo acima e de ensaio — dezesseis zeros. O de verdade nasce ao armar.\n"
        ),
        receita.comando()
    ))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::momento;

    fn imagem(nome: &str, veredito: Option<Veredito>) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes: 42_604_877_207,
            modificado_em: Some(momento("2026-08-22T18:03:24")),
            especie: Especie::Imagem { veredito },
        }
    }

    fn residuo(nome: &str) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes: 1024,
            modificado_em: None,
            especie: Especie::Residuo,
        }
    }

    fn nome(bruto: &str) -> Nome {
        Nome::novo(bruto).expect("nome valido por B-2")
    }

    #[test]
    fn a_imagem_pedida_e_achada_sem_diferenciar_caixa() {
        // O sistema de arquivos do Windows nao diferencia, e a pasta esta la.
        let pastas = vec![imagem("2026-08-22_Apps", Some(Veredito::Aprovada))];
        assert_eq!(
            achar(&pastas, &nome("2026-08-22_apps")).unwrap().nome,
            "2026-08-22_Apps"
        );
    }

    #[test]
    fn residuo_e_recusado_por_ser_residuo_e_nao_por_falta_de_md5sums() {
        // L-2. A recusa nomeia o que aconteceu — um backup interrompido —, e
        // nao o sintoma. "Nao achei o MD5SUMS" mandaria alguem procurar um
        // arquivo; "e residuo" diz o que a pasta e.
        let pastas = vec![residuo("2026-08-23_Interrompido")];
        let recusa = achar(&pastas, &nome("2026-08-23_Interrompido")).unwrap_err();

        assert!(matches!(recusa, RecusaDaVerificacao::EResiduo { .. }));
        assert!(recusa.to_string().contains("residuo"));
        assert!(recusa.to_string().contains("interrompido"));
    }

    #[test]
    fn imagem_que_nao_existe_e_recusa_propria() {
        let pastas = vec![imagem("2026-08-22_Apps", None)];
        let recusa = achar(&pastas, &nome("2026-08-99_Nada")).unwrap_err();

        assert!(matches!(recusa, RecusaDaVerificacao::NaoExiste { .. }));
        assert!(recusa.to_string().contains("arca list"), "diz o que fazer");
    }

    #[test]
    fn a_tela_de_aprovada_diz_o_que_nao_foi_conferido() {
        // A diferenca entre V-1 e V-2 e o que justifica os dois existirem, e
        // ela precisa estar na tela de quem acabou de lê `APROVADA` — senao a
        // palavra promete mais do que a conferencia entregou.
        let conferencia = Conferencia {
            conferidos: vec![crate::verificacao::Conferido {
                arquivo: "disk".to_string(),
                achado: crate::verificacao::Achado::Bate,
                bytes: 8,
            }],
            bytes_lidos: 8,
            fora_do_md5sums: 4,
        };

        let saida = montar_o_veredito(&conferencia);
        assert!(saida.contains("APROVADA"));
        assert!(saida.contains("NAO conferiu que a"));
        assert!(saida.contains("--completo"));
        assert!(saida.contains("4 arquivos · normal"), "{saida}");
    }

    #[test]
    fn a_tela_de_reprovada_diz_que_o_arca_list_nao_vai_mudar() {
        // O alcance da reprovacao. Sem isto, quem reprovasse uma imagem aqui
        // rodaria `arca list`, veria `aprovada` e concluiria que uma das duas
        // telas esta errada — e as duas estao certas, sobre perguntas
        // diferentes.
        let conferencia = Conferencia {
            conferidos: vec![crate::verificacao::Conferido {
                arquivo: "nvme0n1p3.ntfs-ptcl-img.zst.aa".to_string(),
                achado: crate::verificacao::Achado::NaoBate {
                    esperado: crate::resumo::Resumo::novo(
                        crate::resumo::Algoritmo::Md5,
                        "db0f987cd2362b4e8c70817a08678210",
                    )
                    .unwrap(),
                    encontrado: crate::resumo::Resumo::novo(
                        crate::resumo::Algoritmo::Md5,
                        "00000000000000000000000000000000",
                    )
                    .unwrap(),
                },
                bytes: 4_096_000_000,
            }],
            bytes_lidos: 4_096_000_000,
            fora_do_md5sums: 0,
        };

        let saida = montar_o_veredito(&conferencia);
        assert!(saida.contains("REPROVADA"));
        assert!(saida.contains("NAO APARECE NO `arca list`"));
        assert!(saida.contains("nvme0n1p3.ntfs-ptcl-img.zst.aa"));
        assert!(saida.contains("B-10"), "o ARCA nao apaga a imagem ruim");
        assert!(
            !saida.contains("NAO conferiu que a"),
            "o conselho de aprovada nao aparece numa reprovacao"
        );
    }

    /// Uma linha de andamento com a largura da lista de verdade: o maior nome
    /// da `2026-08-22_Apps` e `nvme0n1p3.ntfs-ptcl-img.zst.aa`, com trinta.
    fn andamento_de(numero: usize, arquivo: &str) -> String {
        let conferido = crate::verificacao::Conferido {
            arquivo: arquivo.to_string(),
            achado: crate::verificacao::Achado::Bate,
            bytes: 8,
        };
        montar_andamento(&Andamento {
            numero,
            total: 39,
            arquivo,
            conferido: &conferido,
            largura_do_nome: 30,
            bytes_lidos: 8,
            bytes_totais: 100,
        })
    }

    /// Em que coluna o valor comeca, contada como o console a desenha.
    fn coluna_do_valor(linha: &str) -> usize {
        linha.chars().count() - linha.chars().rev().take_while(|c| *c != '.').count()
    }

    #[test]
    fn o_andamento_alinha_os_numeros_pela_largura_do_total() {
        // Sem isso o `[1/39]` e o `[39/39]` sairiam com larguras diferentes e
        // a coluna dancaria durante os tres minutos de espera.
        assert!(andamento_de(1, "disk").contains("[ 1/39]"));
        assert!(andamento_de(39, "parts").contains("[39/39]"));
    }

    #[test]
    fn o_nome_mais_longo_da_imagem_nao_estoura_a_coluna() {
        // **O defeito que a execucao real pegou com a suite verde.** Com a
        // coluna fixa de `formato::linha`, quatorze das trinta e nove linhas
        // estouravam e saiam com um ponto so — a coluna deixava de existir
        // justamente na parte da lista que demora tres minutos.
        let curto = andamento_de(5, "disk");
        let longo = andamento_de(24, "nvme0n1p3.ntfs-ptcl-img.zst.aa");

        assert_eq!(
            coluna_do_valor(&curto),
            coluna_do_valor(&longo),
            "as duas linhas tem de ter o valor na mesma coluna:\n{curto}{longo}"
        );
        assert!(
            longo.contains(".. ok"),
            "o nome mais longo ficou sem separador: {longo}"
        );
    }

    #[test]
    fn a_estimativa_da_espera_sai_em_tempo_legivel() {
        let plano = Plano {
            arquivos: Vec::new(),
            bytes_totais: 42_604_877_207,
            fora_do_md5sums: 0,
        };
        let aviso = montar_o_aviso_da_espera(&plano);

        assert!(aviso.contains("3 min 23 s"), "{aviso}");
        assert!(aviso.contains("39,7 GB"), "{aviso}");
        assert!(
            aviso.contains("parada nao e travamento"),
            "a tela fica tres minutos sem se mexer"
        );
    }

    #[test]
    fn a_duracao_nao_promete_precisao_que_nao_tem() {
        assert_eq!(duracao(0), "0 s");
        assert_eq!(duracao(47), "47 s");
        assert_eq!(duracao(60), "1 min");
        assert_eq!(duracao(203), "3 min 23 s");
        assert_eq!(duracao(3600), "1 h");
        assert_eq!(duracao(3840), "1 h 4 min");
    }

    #[test]
    fn o_ensaio_imprime_a_receita_e_diz_que_o_selo_e_de_mentira() {
        let saida = ensaio_da_receita(&nome("2026-08-22_Apps")).unwrap();

        assert!(saida.contains("ocs-chkimg"));
        assert!(saida.contains("ARCA_VERIFY=OK"));
        assert!(saida.contains("0000000000000000"));
        assert!(saida.contains("de ensaio"));
        assert!(
            !saida.contains("savedisk") && !saida.contains("restoredisk"),
            "a receita da verificacao nao toca em disco"
        );
    }
}
