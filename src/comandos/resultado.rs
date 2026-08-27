//! `arca resultado` — colher o desfecho, desarmar e imprimir a §5.4 (S-4,
//! S-5, C-12, D8).
//!
//! # Fiacao, e de proposito
//!
//! Quase nada aqui e novo, e isso e o desenho: a E5 construiu o julgamento
//! (`crate::desfecho`), a E3 construiu o leitor do veredito
//! (`crate::imagens::interpretar_veredito`), a E4 construiu o desarmar, e a E1
//! construiu a listagem. A E8 e quem os liga na ordem certa. Reescrever
//! qualquer um deles aqui daria duas versoes da mesma regra, e duas versoes da
//! mesma regra divergem na primeira mudanca — foi por isso que o `arca status`
//! ja reusava `list::montar` em vez de formatar as imagens de novo.
//!
//! # Colher encerra o job, e isso fecha uma contradicao que a E5 deixou
//!
//! Depois de um `arca desarmar`, o `arca status` mostrava "Boot unico: nao
//! armado" ao lado de um job pendente. Nao era contradicao — o dispositivo
//! estava inerte e o job continuava registrado —, mas era um par que ninguem
//! fechava. Quem fecha e este comando: colher marca o `estado.json` como
//! [`Situacao::Colhido`], e o `arca status` passa a dizer "ultimo job,
//! colhido" (ver [`crate::estado::Situacao`] para por que marcar e nao
//! apagar).
//!
//! # O que encerra o job, e o que nao encerra
//!
//! Encerra quando o ARCA **chegou a um veredito sobre este job**: achou o
//! `arca-fim.txt` e o julgou (qualquer das cinco linhas do §5.5), ou nao achou
//! arquivo nenhum — que e o C-12 na letra, "o boot nao aconteceu, ou o
//! Clonezilla abriu menu". As duas sao respostas.
//!
//! **Nao** encerra quando o arquivo esta la e nao se deixou lê. "Nao consegui
//! olhar" nao e veredito, e essa e exatamente a distincao que a revisao da E5
//! pagou caro para existir: transformar "nao consegui olhar" em "nao ha nada
//! la" faria um backup bem-sucedido virar um job encerrado como se nunca
//! tivesse rodado. Nesse caso o job fica pendente e a colheita pode ser
//! tentada de novo.
//!
//! # S-5: falha parcial e falha total
//!
//! O desfecho e o veredito sao independentes — o `CONTEXT.md` diz isso, e a
//! §5.4 os mostra em linhas diferentes. Um `ARCA_BACKUP=OK` com a imagem
//! reprovada **nao** e sucesso, e nenhum dos dois pode esconder o outro: a
//! saida imprime os dois sempre, e o codigo de saida do processo segue o pior
//! deles.

use crate::app::Contexto;
use crate::desarme::{self, Desarme};
use crate::desfecho::{Encontrado, Julgamento};
use crate::dispositivo::{self, Dispositivo};
use crate::erro::{Erro, Resultado};
use crate::estado::{self, Estado, Situacao};
use crate::formato::{dia_e_mes, linha, tamanho};
use crate::imagens::{self, Pasta, Veredito};
use crate::ordem::{self, OrdemDevolvida};
use crate::portas::Arquivos;
use crate::receita::Operacao;
use std::path::Path;

use super::list;

/// O que aconteceu com o job ao fim da colheita.
///
/// # A linha da tela nunca afirma mais do que houve
///
/// A saida da §5.4 tem uma linha `Job ..... encerrado`, e ela nao pode dizer
/// isso antes de o `estado.json` ter sido gravado — seria a mesma mentira que
/// o `--dry-run` deste projeto ja contou uma vez (§11), um `ok` sobre uma acao
/// que nao aconteceu.
///
/// Mas o inverso tambem e ruim, e a revisao desta etapa o nomeou: gravar
/// primeiro e **so entao** imprimir faz o relatorio inteiro se perder quando a
/// gravacao falha — o `ARCABOOT` cheio, protegido, ou o dispositivo removido
/// entre o desarmar e a gravacao. O ARCA teria lido o desfecho do backup e o
/// jogado fora.
///
/// Este tipo resolve os dois: grava-se antes de imprimir, e o que a gravacao
/// respondeu vai para a linha. O relatorio sai sempre, e a linha e verdade nos
/// tres casos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encerramento {
    /// Havia veredito, e o `estado.json` foi marcado como colhido.
    Encerrado,

    /// Nao havia veredito: o `arca-fim.txt` esta la e nao se deixou lê. O job
    /// continua pendente **de proposito**.
    ContinuaPendente,

    /// Havia veredito e a gravacao falhou. O job continua pendente por
    /// acidente, e a diferenca importa: aqui ha o que consertar.
    NaoDeuParaEncerrar { motivo: String },
}

/// O job colhido, e tudo que se soube dele.
pub struct Colheita<'a> {
    pub estado: &'a Estado,
    pub desfecho: &'a Encontrado,

    /// A pasta da imagem no `ARCAVAULT`, quando ela existe. Um backup cujo
    /// boot nao aconteceu nao tem pasta nenhuma.
    pub pasta: Option<&'a Pasta>,

    /// Os discos que a sondagem viu, quando o job era uma sondagem.
    ///
    /// `None` nas outras tres operacoes **e** quando a sondagem nao deixou
    /// arquivo — e a tela distingue as duas, porque na sondagem a segunda e
    /// falha e nas outras nem pergunta se faz.
    pub sondagem: Option<&'a [crate::blkdev::DiscoDaImagem]>,

    pub desarme: &'a Desarme,
    pub encerramento: &'a Encerramento,

    /// O conserto de C-13, que e independente de tudo acima.
    ///
    /// Vem em linha propria na tela pelo motivo que a E8 registrou: misturar
    /// "colhi" com "arrumei" tira de quem lê a informacao de qual das duas
    /// aconteceu.
    pub ordem: &'a OrdemDevolvida,

    pub pastas: &'a [Pasta],
    pub livre_bytes: u64,
}

/// A linha `Ordem de boot`, comum aos tres caminhos do `arca resultado`.
///
/// O rotulo e o mesmo que o `arca status` usa desde o ADR-0009, e de
/// proposito: e a mesma coisa, e quem viu o aviso la tem de reconhecer o
/// conserto aqui.
fn linha_da_ordem(ordem: &OrdemDevolvida) -> String {
    linha(
        "Ordem de boot",
        &match ordem {
            OrdemDevolvida::JaEstavaNaFrente => "ok · o Windows ja era o primeiro".to_string(),
            OrdemDevolvida::Devolvida { estava_na_frente } => {
                format!("devolvida · o Windows voltou ao topo, na frente de {estava_na_frente}")
            }
            OrdemDevolvida::NaoHaviaOrdem => {
                "devolvida · nao havia ordem nenhuma, e agora ela leva ao Windows".to_string()
            }
        },
    )
}

/// O que muda para quem liga a maquina, e so quando houve conserto.
///
/// Dizer isto sempre treinaria quem lê a pular a linha, e ela e a resposta a
/// um incomodo concreto: ate a E9 a maquina passava a bootar no dispositivo a
/// cada religada depois de um backup, e nao era assim antes de o ARCA existir.
///
/// **E so isso.** A versao anterior explicava o mecanismo — que o ciclo de boot
/// tinha posto a entrada na frente, que nenhuma entrada foi removida, e o
/// numero da regra. Quem acabou de colher um job nao precisa disso para decidir
/// nada; precisa saber se pode religar a maquina. O mecanismo esta no
/// [ADR-0013](../../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md).
fn conselho_da_ordem(ordem: &OrdemDevolvida) -> String {
    if !ordem.houve_conserto() {
        return String::new();
    }
    "\n  Daqui em diante a maquina liga no Windows, com o SSD conectado ou sem ele.\n".to_string()
}

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let caminho_do_estado = dispositivo.caminho_do_estado()?;

    // O estado primeiro, e **antes de desarmar**: sem job nao ha o que colher,
    // e desarmar um dispositivo inerte para depois dizer "nada a colher" seria
    // agir antes de ter o que dizer. C-1 nao obriga aqui — ele fala de comandos
    // que **armam**, e este nao arma.
    let estado_do_job = match estado::ler(contexto.arquivos, &caminho_do_estado) {
        Ok(estado) => estado,
        Err(erro) if erro.e_arquivo_ausente() => return nada_a_colher(contexto, &dispositivo),
        // "Nao consegui entender" nunca vira "nao ha job": o dispositivo pode
        // estar armado, e quem lê precisa saber disso antes de reiniciar.
        Err(erro) => return Err(erro),
    };

    if estado_do_job.situacao == Situacao::Colhido {
        return ja_colhido(contexto, &dispositivo, &estado_do_job);
    }

    let onde = estado::caminho_do_desfecho(
        &raiz_do_vault,
        estado_do_job.comando,
        estado_do_job.nome.as_ref(),
    );
    let desfecho = ler_o_desfecho(contexto.arquivos, &onde, &estado_do_job);

    // Desarmar acontece **depois** de lê o desfecho e antes de encerrar o
    // job. Depois de lê porque o `arca-fim.txt` mora no `ARCAVAULT` e nada no
    // desarmar o toca — a ordem so importa para o caso em que o desarmar
    // falha, e ali e melhor ja ter o desfecho em maos do que perde-lo.
    let desarme = desarme::executar(
        contexto.arquivos,
        contexto.firmware,
        &dispositivo.caminho_do_grub()?,
    )?;

    // C-13, e **depois** do desarme de propósito. O desarme relê a ordem
    // permanente para conferir que ele próprio não a tocou (C-5); consertá-la
    // antes faria aquela conferência correr sobre um valor que este comando
    // acabou de mudar, e a checagem que existe para pegar um `bcdedit` que
    // mexeu no que não devia passaria a comparar duas coisas minhas.
    let ordem = ordem::devolver_o_windows(contexto.firmware)?;

    // A listagem **antes** de encerrar o job, e nao depois. Se a enumeracao do
    // `ARCAVAULT` falhar — um setor ruim, o dispositivo removido no meio —, o
    // erro sobe com o job ainda pendente, e a colheita pode ser tentada de
    // novo. Na ordem inversa, o job sairia encerrado e ninguem teria visto o
    // desfecho: o ARCA teria lido o resultado do backup e o perdido.
    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;

    // Sem nome de imagem nao ha pasta a procurar, e nao e falta: a sondagem
    // nao opera sobre imagem nenhuma. Procurar assim mesmo faria o `find`
    // comparar contra a string vazia — e `Nome` nunca e vazia, entao daria
    // `None` por acaso em vez de por decisao.
    let pasta = estado_do_job.nome.as_ref().and_then(|nome| {
        pastas
            .iter()
            .find(|pasta| pasta.nome.eq_ignore_ascii_case(nome.como_texto()))
    });

    // O job so se encerra quando houve veredito sobre ele. Ver o cabecalho e
    // [`Encerramento`] — a gravacao acontece antes de imprimir, e o que ela
    // respondeu vai para a linha, em vez de decidir se ha linha.
    let encerramento = if encerra_o_job(&desfecho) {
        match estado::gravar(
            contexto.arquivos,
            &caminho_do_estado,
            &Estado {
                situacao: Situacao::Colhido,
                ..estado_do_job.clone()
            },
        ) {
            Ok(()) => Encerramento::Encerrado,
            Err(erro) => Encerramento::NaoDeuParaEncerrar {
                motivo: erro.to_string(),
            },
        }
    } else {
        Encerramento::ContinuaPendente
    };

    contexto.registro.info(format!(
        "colhido {} · selo {} · desfecho: {desfecho} · veredito: {} · job {encerramento:?}",
        estado_do_job.descricao(),
        estado_do_job.selo,
        descrever_veredito(pasta),
    ));

    // O que a sondagem produziu, lido **depois** de o job ser encerrado: ele e
    // para a tela, e uma leitura que falhe nao pode desfazer uma colheita que
    // ja aconteceu. `None` quando o job nao e sondagem — as outras tres nao
    // escrevem naquele lugar.
    let sondagem = (estado_do_job.comando == Operacao::Sondagem)
        .then(|| crate::sondagem::ler(contexto.arquivos, &raiz_do_vault))
        .flatten()
        .map(|lista| crate::blkdev::ler(&lista.texto));

    print!(
        "{}",
        montar(&Colheita {
            estado: &estado_do_job,
            desfecho: &desfecho,
            pasta,
            sondagem: sondagem.as_deref(),
            desarme: &desarme,
            ordem: &ordem,
            encerramento: &encerramento,
            pastas: &pastas,
            livre_bytes: dispositivo.vault.livre_bytes,
        })
    );

    // S-5, e por ultimo: a saida ja foi impressa inteira. Estes erros existem
    // para o codigo de saida — quem chamou o ARCA de um script nao pode lê um
    // desfecho ruim como exito.
    //
    // A falha ao encerrar vem **antes** da de S-5, e nao por gravidade: ela e a
    // unica das duas que pede uma acao agora. Um desfecho ruim ja esta dito na
    // tela e nao muda mais; um job que ficou pendente por acidente vai
    // aparecer no proximo `arca status` como se houvesse algo esperando.
    if let Encerramento::NaoDeuParaEncerrar { motivo } = &encerramento {
        return Err(Erro::OperacaoNaoConcluida(format!(
            "o desfecho foi lido e esta acima, e o job NAO pode ser encerrado: {motivo}. Ele continua marcado como armado no dispositivo, e um `arca resultado` rodado de novo o relê"
        )));
    }

    match julgar_o_conjunto(&desfecho, pasta, estado_do_job.comando) {
        Some(porque) => Err(Erro::OperacaoNaoConcluida(porque)),
        None => Ok(()),
    }
}

/// Lê o que ha no lugar do desfecho e julga pelo selo (C-11, C-12).
///
/// Copia deliberada do que `arca status` ja faz — e nao um `pub` compartilhado
/// — porque as duas leituras respondem perguntas diferentes: la e diagnostico,
/// aqui e colheita. O que **e** compartilhado e o que decide:
/// [`crate::desfecho::julgar`].
fn ler_o_desfecho(arquivos: &dyn Arquivos, onde: &Path, estado: &Estado) -> Encontrado {
    // `ler_texto_alheio` porque quem escreveu foi o `echo` de um bash do outro
    // lado do reinicio: um byte solto nao pode fazer o desfecho sumir.
    match arquivos.ler_texto_alheio(onde) {
        Ok(texto) => Encontrado::Arquivo(crate::desfecho::julgar(
            &crate::desfecho::ler(&texto),
            &estado.selo,
        )),
        Err(erro) if erro.e_arquivo_ausente() => Encontrado::SemArquivo,
        Err(erro) => Encontrado::NaoDeuParaLer {
            motivo: erro.to_string(),
        },
    }
}

/// Se este desfecho e um veredito sobre o job, ou so a impossibilidade de
/// olhar.
fn encerra_o_job(desfecho: &Encontrado) -> bool {
    match desfecho {
        Encontrado::Arquivo(_) | Encontrado::SemArquivo => true,
        Encontrado::NaoDeuParaLer { .. } => false,
    }
}

/// S-5: o pior entre o desfecho e o veredito, ou `None` quando os dois
/// prestam.
///
/// A ordem e a mesma que o resto do projeto usa: **toda forma de nao ter dado
/// certo antes de toda forma de ter dado**. Um `ARCA_BACKUP=OK` com imagem
/// reprovada cai na segunda metade, e e exatamente o caso que S-5 nomeia.
///
/// # A restauracao para na primeira metade, e a E9 achou isso relendo esta
///
/// Ate a E9 esta funcao nao conhecia a operacao, e a segunda metade valia para
/// as duas. Numa restauracao ela estava **errada**, e do jeito mais caro: um
/// `ARCA_RESTORE=OK` cuja pasta nao tem `arca-check.log` saia reprovado por
/// "a imagem esta sem veredito", e uma restauracao bem-sucedida era relatada
/// como falha.
///
/// A confusao e de sujeito. No backup a pasta e **o que a operacao produziu**,
/// e o veredito dela e o segundo sinal que S-5 manda nao esconder. Na
/// restauracao a pasta e **a imagem de origem**: ela ja existia antes, o
/// veredito dela e do backup que a criou, e ele nao diz nada sobre a
/// restauracao ter dado certo. Julgar a operacao por ele e julgar uma coisa
/// pelo parecer de outra.
///
/// O que sobra na restauracao e o desfecho, sozinho — e P-6 aplicado a ela e a
/// razao de o [`conselho`] dizer isso na tela em vez de deixar por conta de
/// quem lê.
///
/// Achada relendo **esta funcao** procurando o que a restauracao muda nela, e
/// nao lendo o codigo novo procurando defeitos. E a mesma defesa que funcionou
/// na E4 e na E7.
/// Se o veredito da pasta e um parecer **sobre esta operacao**.
///
/// # A pergunta certa nao e "esta operacao mexe numa imagem?"
///
/// A E9 aprendeu isso na restauracao, e a E12 quase a desaprendeu escrevendo
/// "as que produzem imagem": pela producao, a **verificacao** ficaria de fora,
/// e ela e justamente uma das que devem entrar. Ela nao produz imagem nenhuma
/// e o `arca-check.log` que se lê e o `ocs-chkimg` **daquela** execucao.
///
/// | Operacao | A pasta e | Julga? |
/// |---|---|---|
/// | backup | o que a operacao acabou de gravar | **sim** |
/// | verificacao | a imagem que o `ocs-chkimg` acabou de julgar | **sim** |
/// | restauracao | a imagem de **origem**, de dias antes | nao |
/// | sondagem | nao ha pasta: ela nao opera sobre imagem | nao |
fn o_veredito_fala_desta_operacao(operacao: Operacao) -> bool {
    match operacao {
        Operacao::Backup | Operacao::Verificacao => true,
        Operacao::Restauracao | Operacao::Sondagem => false,
    }
}

fn julgar_o_conjunto(
    desfecho: &Encontrado,
    pasta: Option<&Pasta>,
    operacao: Operacao,
) -> Option<String> {
    if !matches!(desfecho, Encontrado::Arquivo(Julgamento::Concluida)) {
        return Some(format!("a operacao nao foi concluida: {desfecho}"));
    }

    if !o_veredito_fala_desta_operacao(operacao) {
        return None;
    }

    match pasta.map(|pasta| &pasta.especie) {
        Some(imagens::Especie::Imagem {
            veredito: Some(Veredito::Aprovada),
        }) => None,
        Some(imagens::Especie::Imagem {
            veredito: Some(Veredito::Reprovada),
        }) => Some(
            "o desfecho diz OK e a imagem foi REPROVADA pelo ocs-chkimg. Falha parcial e falha total (S-5): esta imagem nao serve para restaurar"
                .to_string(),
        ),
        Some(imagens::Especie::Imagem { veredito: None }) => Some(
            "o desfecho diz OK e a imagem esta sem veredito: o `arca-check.log` nao esta la, ou nao diz nada reconhecivel. Imagem nao verificada e suposicao, e o ARCA nao a apresenta como aprovada"
                .to_string(),
        ),
        Some(imagens::Especie::Residuo) => Some(
            "o desfecho diz OK e a pasta nao tem `MD5SUMS`: e residuo, e nao imagem. Os dois sinais discordam, e o que manda e o disco"
                .to_string(),
        ),
        None => Some(
            "o desfecho diz OK e nao ha pasta com esse nome no ARCAVAULT. O que a receita disse ter gravado nao esta la"
                .to_string(),
        ),
    }
}

/// `backup` → `Backup`. ASCII por construção: [`Operacao::nome`] devolve
/// `backup` ou `restauracao`, e nao ha caractere multibyte a partir.
fn com_inicial_maiuscula(texto: &str) -> String {
    let mut letras = texto.chars();
    match letras.next() {
        Some(primeira) => primeira.to_uppercase().collect::<String>() + letras.as_str(),
        None => String::new(),
    }
}

fn descrever_veredito(pasta: Option<&Pasta>) -> String {
    match pasta.map(|pasta| &pasta.especie) {
        Some(imagens::Especie::Imagem { veredito }) => match veredito {
            Some(Veredito::Aprovada) => "APROVADA".to_string(),
            Some(Veredito::Reprovada) => "REPROVADA".to_string(),
            None => "sem veredito".to_string(),
        },
        Some(imagens::Especie::Residuo) => "residuo".to_string(),
        None => "nao ha pasta".to_string(),
    }
}

/// A §5.4 inteira.
pub fn montar(colheita: &Colheita) -> String {
    let mut saida = String::new();

    // O cabecalho nomeia a operacao e a imagem, como no §5.4. Sem imagem — a
    // sondagem — sobra a operacao sozinha, que e o que ela e.
    saida.push_str(&format!(
        "{}{}\n",
        com_inicial_maiuscula(colheita.estado.comando.nome()),
        match &colheita.estado.nome {
            Some(nome) => format!(" {nome}"),
            None => String::new(),
        }
    ));

    match colheita.pasta {
        Some(pasta) => saida.push_str(&format!(
            "  {} · {}\n",
            dia_e_mes(pasta.modificado_em),
            tamanho(pasta.tamanho_bytes)
        )),
        // Duas ausencias diferentes, e dize-las igual seria dizer que faltou
        // alguma coisa numa operacao a que nao falta nada: um backup sem pasta
        // e um problema — o que a receita disse ter gravado nao esta la —, e
        // uma sondagem sem pasta e o normal, porque ela nao grava imagem.
        None if colheita.estado.comando.nomeia_imagem() => {
            saida.push_str("  nao ha pasta com este nome no ARCAVAULT\n")
        }
        None => saida.push_str("  nao opera sobre imagem nenhuma\n"),
    }

    // As duas linhas que S-5 exige lado a lado. Nenhuma esconde a outra: um
    // desfecho OK com veredito REPROVADA aparece assim mesmo, e e por isso
    // que sao duas linhas e nao uma conclusao.
    saida.push_str(&format!("  Desfecho: {}\n", colheita.desfecho));

    // A segunda linha muda de **rotulo** conforme a operacao, e nao so de
    // valor. `Verificacao:` numa restauracao seria a mesma confusao de sujeito
    // que [`julgar_o_conjunto`] cometia: quem lesse concluiria que a
    // restauracao foi verificada, e o que esta ali e o parecer do backup que
    // criou a imagem de origem, dias antes e sobre outra coisa.
    saida.push_str(&match colheita.estado.comando {
        Operacao::Backup => format!("  Verificacao: {}\n", descrever_veredito(colheita.pasta)),
        Operacao::Restauracao => format!(
            "  Imagem de origem: {} — veredito do backup que a criou, e nao desta operacao\n",
            descrever_veredito(colheita.pasta)
        ),
        // Numa verificacao armada os dois sinais tem a **mesma fonte**: o
        // codigo de saida do `ocs-chkimg` decide o `ARCA_VERIFY=` do desfecho
        // e o `ARCA_VEREDITO=` do log, no mesmo `if`. Num backup eles sao
        // independentes (§4.3, S-5) e aqui nao sao, e a linha diz isso — quem
        // lê duas linhas concordando merece saber se sao duas testemunhas ou
        // uma so falando duas vezes. E o mesmo cuidado do conselho do §6.3.
        Operacao::Verificacao => format!(
            "  Veredito: {} — mesmo sinal do desfecho acima, e nao um segundo\n",
            descrever_veredito(colheita.pasta)
        ),
        // A sondagem nao tem segunda linha sobre imagem, porque nao ha imagem.
        // O que ela produziu e o que a linha diz: um nome de disco, que os
        // outros comandos vao lê. Imprimir `Verificacao: nao ha pasta` aqui
        // seria a mesma confusao de sujeito que [`julgar_o_conjunto`] cometia
        // na restauracao ate a E9.
        Operacao::Sondagem => format!(
            "  Discos vistos: {}\n",
            match colheita.sondagem {
                Some(discos) if !discos.is_empty() => discos
                    .iter()
                    .map(|disco| format!("{} ({})", disco.nome, disco.modelo))
                    .collect::<Vec<_>>()
                    .join(", "),
                Some(_) => "nenhum — o `blkdev.list` esta la e nao traz disco nenhum".to_string(),
                None => "nao ha `blkdev.list` no lugar onde a sondagem o grava".to_string(),
            }
        ),
    });

    saida.push_str(&format!("  Selo: {}\n", colheita.estado.selo));

    saida.push('\n');
    saida.push_str(&linha(
        "Desarmando SSD",
        &format!("ok · {}", colheita.desarme.caminho_do_grub.display()),
    ));
    saida.push_str(&linha(
        "Job",
        &match colheita.encerramento {
            Encerramento::Encerrado => "encerrado · o desfecho foi lido e dito".to_string(),
            Encerramento::ContinuaPendente => {
                "CONTINUA PENDENTE · nao deu para lê o desfecho, e isso nao e um veredito"
                    .to_string()
            }
            Encerramento::NaoDeuParaEncerrar { motivo } => {
                format!("NAO FOI POSSIVEL ENCERRAR · {motivo}")
            }
        },
    ));
    saida.push_str(&linha_da_ordem(colheita.ordem));

    saida.push('\n');
    saida.push_str(&list::montar(colheita.pastas, colheita.livre_bytes));

    saida.push_str(&conselho(colheita));
    saida.push_str(&conselho_da_ordem(colheita.ordem));
    saida
}

/// O que fazer a seguir, quando ha o que fazer.
///
/// Uma linha que so diz o que aconteceu empurra o problema de volta para quem
/// nao sabe resolve-lo — e a mesma regra que os avisos do pre-voo seguem.
/// O nome da pasta da imagem, para as frases que falam dela.
///
/// Os dois ramos que a usam so sao alcancaveis por operacao que nomeia imagem
/// — o `Concluida` sem veredito e o `Truncado` —, e o vazio aqui e a rede que
/// impede uma frase falando de `` ` ` `` se um dia deixarem de ser.
fn nome_da_pasta(colheita: &Colheita) -> String {
    colheita
        .estado
        .nome
        .as_ref()
        .map(|nome| nome.to_string())
        .unwrap_or_default()
}

/// O que a sondagem entrega, dito onde alguem lê depois de ela rodar.
///
/// Ela nao produz imagem e nao tem veredito: o que ela produz e o nome que o
/// **Linux** da ao disco desta maquina, e o valor disso e um comando que
/// recusava passar a funcionar. Sem esta linha, quem colhesse uma sondagem
/// leria `ARCA_PROBE=OK` e nao saberia o que fazer com aquilo.
fn conselho_da_sondagem(colheita: &Colheita) -> String {
    match colheita.sondagem {
        Some(discos) if !discos.is_empty() => concat!(
            "\n  A lista de discos ficou em ARCA-LOGS\\sondagem\\blkdev.list, e o primeiro\n",
            "  backup ja pode ser feito. Para ver o que ele vai nomear, sem armar nada:\n",
            "    arca backup <nome> --dry-run\n"
        )
        .to_string(),

        // O desfecho diz `OK` e nao ha lista: as duas afirmacoes nao cabem
        // juntas, e quem lê precisa saber que a segunda vale. E o caso que o
        // `if` da receita existe para tornar raro — com o `lsblk` falhando o
        // desfecho diria `FALHOU` —, e este ramo cobre o que sobra dele.
        _ => concat!(
            "\n  O DESFECHO DIZ QUE A SONDAGEM CONCLUIU, E NAO HA LISTA DE DISCOS.\n",
            "  O `blkdev.list` devia estar em ARCA-LOGS\\sondagem\\ e nao esta la, ou nao\n",
            "  traz disco nenhum. O nome do disco continua por determinar, e `arca backup`\n",
            "  vai continuar recusando — olhe o arquivo antes de sondar de novo: se o\n",
            "  `lsblk` recusou alguma flag, a mensagem dele esta dentro dele.\n"
        )
        .to_string(),
    }
}

fn conselho(colheita: &Colheita) -> String {
    // A falha ao encerrar fala primeiro, e nao por gravidade: e a unica que
    // pede uma acao **agora**. O desfecho ja esta dito nas linhas acima e nao
    // muda mais; um job que ficou pendente por acidente vai reaparecer no
    // proximo `arca status` como se houvesse alguma coisa esperando.
    if let Encerramento::NaoDeuParaEncerrar { motivo } = colheita.encerramento {
        return format!(
            "\n  O DESFECHO ACIMA FOI LIDO, e o job NAO pôde ser encerrado:\n\
             \x20 {motivo}\n\
             \x20 O `estado.json` continua dizendo `armado`, e o proximo `arca status` vai\n\
             \x20 mostrar este job como pendente. O dispositivo **ja foi desarmado** e nao\n\
             \x20 vai bootar sozinho. Resolvido o problema de escrita no ARCABOOT, rode\n\
             \x20 `arca resultado` de novo: o selo continua gravado, e o desfecho continua\n\
             \x20 sendo deste job.\n"
        );
    }

    match colheita.desfecho {
        // A restauracao tem conselho proprio no ramo do exito, e o backup nao
        // tem nenhum: la o exito e exito, e ha dois sinais independentes
        // dizendo isso. Aqui ha um so.
        Encontrado::Arquivo(Julgamento::Concluida)
            if colheita.estado.comando == Operacao::Restauracao =>
        {
            conselho_da_restauracao(colheita)
        }

        // A sondagem concluida tem conselho proprio, e ele e o que ela existe
        // para dizer: o oraculo do §4.5 passou a existir, e o comando que
        // recusava agora funciona.
        Encontrado::Arquivo(Julgamento::Concluida)
            if colheita.estado.comando == Operacao::Sondagem =>
        {
            conselho_da_sondagem(colheita)
        }

        Encontrado::Arquivo(Julgamento::Concluida) => match colheita.pasta.map(|p| &p.especie) {
            Some(imagens::Especie::Imagem {
                veredito: Some(Veredito::Aprovada),
            }) => String::new(),
            _ => format!(
                "\n  FALHA PARCIAL E FALHA TOTAL (S-5). O Clonezilla disse que terminou, e a\n\
                 \x20 imagem nao esta aprovada. O ARCA nao apaga nada (B-10): a pasta\n\
                 \x20 `{}` continua no ARCAVAULT para quem quiser olhar.\n\
                 \x20 Para gravar de novo, use outro nome — o ARCA nunca escreve por cima.\n",
                nome_da_pasta(colheita)
            ),
        },
        Encontrado::Arquivo(Julgamento::Truncado) => format!(
            "\n  A pasta `{}` e RESIDUO: o desligamento pegou a operacao no meio, e nao\n\
             \x20 ha imagem inteira ali. Ela nao aparece para restaurar (L-2), e o ARCA\n\
             \x20 nao a apaga (B-10) — apague a mao depois de olhar.\n",
            nome_da_pasta(colheita)
        ),
        Encontrado::SemArquivo => concat!(
            "\n  O BOOT NAO ACONTECEU, ou o Clonezilla abriu o menu em vez de executar a\n",
            "  receita (C-12). As duas causas deixam o mesmo rastro, que e nenhum.\n",
            "  O dispositivo ja foi desarmado acima. Antes de tentar de novo, confira\n",
            "  com `arca status` para onde a entrada de firmware aponta.\n"
        )
        .to_string(),
        Encontrado::NaoDeuParaLer { .. } => concat!(
            "\n  O JOB CONTINUA PENDENTE. O `arca-fim.txt` esta la e nao se deixou lê, e\n",
            "  isso NAO e o mesmo que o boot nao ter acontecido: a operacao pode ter\n",
            "  terminado bem. Resolvido o problema de leitura, rode `arca resultado` de\n",
            "  novo — o selo continua gravado e o desfecho continua sendo deste job.\n"
        )
        .to_string(),
        // **A sondagem aponta o arquivo pelo nome, e as outras três não podem.**
        //
        // Nelas o log é do `ocs-sr` ou do `ocs-chkimg`, e o que está lá é
        // centenas de linhas de progresso — dizer "olhe a pasta" é o melhor que
        // se pode fazer. Na sondagem o arquivo tem **uma linha**, e ela é a
        // mensagem do `lsblk` dizendo o que ele recusou: mandar procurar na
        // pasta seria esconder a resposta a um `cd` de distância.
        //
        // Medido na falha forçada de 24/08/2026: `lsblk: unknown column:
        // FLAGQUENAOEXISTE`, quarenta bytes, no `blkdev.list`.
        Encontrado::Arquivo(Julgamento::Falhou)
            if colheita.estado.comando == Operacao::Sondagem =>
        {
            concat!(
                "\n  O `lsblk` FALHOU E DISSE POR QUÊ. A mensagem dele esta em\n",
                "  ARCA-LOGS\\sondagem\\blkdev.list — o `2>&1` da receita aponta para la, e\n",
                "  por isso ela sobreviveu ao desligamento.\n",
                "\n",
                "  As flags do `lsblk` sao RECONSTRUCAO a partir do formato do arquivo, e\n",
                "  nao ha captura da linha de comando que o Clonezilla usa (§10.2.5). Uma\n",
                "  coluna que esta versao do util-linux nao conheca cai exatamente aqui.\n",
                "\n",
                "  O nome do disco continua por determinar, e `arca backup` vai continuar\n",
                "  recusando ate uma sondagem concluir.\n"
            )
            .to_string()
        }

        Encontrado::Arquivo(Julgamento::Falhou) => format!(
            "\n  O CLONEZILLA FALHOU E DISSE. O log da operacao esta em\n\
             \x20 ARCA-LOGS\\{}\\, junto do proprio `arca-fim.txt`.\n",
            crate::desfecho::pasta_do_job(colheita.estado.comando, colheita.estado.nome.as_ref())
        ),
        Encontrado::Arquivo(Julgamento::JobFantasma { .. })
        | Encontrado::Arquivo(Julgamento::NaoPertenceAoArca(_)) => concat!(
            "\n  O ARQUIVO ENCONTRADO NAO E O DESFECHO DESTE JOB, e o ARCA nao acredita\n",
            "  nele (C-11). Ele continua onde estava — o ARCA nao apaga nada (B-10).\n"
        )
        .to_string(),
    }
}

/// O que dizer depois de uma restauracao que o Clonezilla deu por concluida.
///
/// Quatro linhas, e nenhuma delas e "deu tudo certo".
///
/// **A tela mandava religar, e quem a lê ja religou.** Ate 27/08/2026 o texto
/// fechava com "o juiz que falta e o Windows subir: religue e confira" — um
/// pedido impossivel de atender, porque atende-lo e a condicao para ver a
/// tela. [`crate::comandos::restore::montar_o_armado`] manda remover o SSD,
/// religar, e **so entao** reconectar para colher; o `arca.log` destruido do
/// paragrafo abaixo e a prova disso dentro do proprio texto. Quem lê esta
/// tela esta dentro do Windows que voltou da imagem, e o que falta a ele nao
/// e religar: e abrir o que restaurou.
///
/// **P-6 dói mais deste lado**, e por isso a linha do exito e morna. No backup
/// ha dois sinais independentes sobre o codigo de saida: a conferencia nativa
/// que o Clonezilla faz por padrao — e que `-scs` desligaria, razao de ele
/// ficar de fora (ADR-0004) — e o `ocs-chkimg` explicito de B-9. Na
/// restauracao ha uma conferencia parecida, e ela e sobre **outra pergunta**:
/// `-scr` desligaria a checagem de que a imagem e restauravel, e ela roda
/// **antes** de gravar. Nenhuma delas olha o resultado da gravacao. Se o
/// `ocs-sr` devolver zero ao falhar, o `if/then/else` de R-5 escreve `OK`
/// sobre uma restauracao quebrada, e o unico juiz que sobra e o Windows subir
/// — que e o que ja aconteceu, e e o que a tela diz.
///
/// **O `arca.log` deste lado foi destruido pela propria operacao.** Ele mora em
/// `%LOCALAPPDATA%\ARCA`, no `C:`, que e o que a restauracao substitui. E uma
/// consequencia de §4.1 que so agora tem dente: o registro do lado Windows de
/// que o job foi armado nao existe mais, e o que sobrevive e o `estado.json`
/// do `ARCABOOT` — que e exatamente o que §4.1 existe para garantir. O
/// `arca.log` que estiver la agora e o de **dentro da imagem**, e as linhas
/// dele sao de outro tempo.
///
/// **A janela do [ADR-0009] fechou aqui, e ela era destrutiva.** O desarmar ja
/// aconteceu — a linha acima diz isso —, e a partir de agora um reinicio com o
/// SSD conectado para no menu do Clonezilla em vez de restaurar de novo. Esta
/// e a unica das tres razoes que sobreviveu **como frase** na tela, e o motivo
/// e a hora: o SSD esta conectado neste instante porque o `arca restore`
/// mandou reconecta-lo para colher, depois de avisar em caixa alta que religar
/// com ele plugado RESTAURARIA DE NOVO. Fechar esse medo e o unico dos tres
/// porques que muda o que a pessoa faz nos proximos minutos.
///
/// **E so isso.** O resto — a comparacao com os dois sinais do backup, o
/// caminho do `%LOCALAPPDATA%`, o `estado.json` que sobreviveu, o numero do
/// ADR — e registro de projeto: mora aqui e nos ADRs, e quem colhe uma
/// restauracao nao o pediu. Mesmo corte de [`conselho_da_sondagem`] e
/// [`linha_da_ordem`], e pelo mesmo motivo.
///
/// [ADR-0009]: ../../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md
fn conselho_da_restauracao(colheita: &Colheita) -> String {
    let mut saida = String::from(
        "\n  A RESTAURACAO TERMINOU, e este Windows ja e o que voltou da imagem. Confira\n\
         \x20 seus programas e arquivos.\n",
    );

    saida.push_str(&format!(
        "\x20 O log do Clonezilla esta em\n\
         \x20 ARCA-LOGS\\{}\\arca-restore.log, no ARCAVAULT.\n",
        crate::desfecho::pasta_do_job(colheita.estado.comando, colheita.estado.nome.as_ref())
    ));

    saida.push_str(concat!(
        "\x20 O `arca.log` do C: veio de dentro da imagem, e nao registra esta operacao.\n",
        "  O SSD ja foi desarmado: religar com ele conectado nao restaura mais nada.\n",
    ));

    saida
}

/// Nao ha `estado.json`: nao ha nada a colher (§5.5, ultima linha).
///
/// **Nao desarma**, e isso e deliberado. Sem job nao houve armar, e desarmar
/// aqui misturaria "colhi" com "arrumei". Quem quer desarmar sem colher tem
/// `arca desarmar`, que e o comando que a E4 criou exatamente para isso.
fn nada_a_colher(contexto: &Contexto, dispositivo: &Dispositivo) -> Resultado<()> {
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;

    // C-13 acontece **mesmo sem job**, e a razao é que ele não fala de job: a
    // ordem permanente é estado da NVRAM, e ela está suja ou não está,
    // independentemente de alguém ter armado alguma coisa. É a diferença para
    // o desarmar do parágrafo acima — aquele desfaz uma intenção do ARCA, e
    // sem job não houve intenção nenhuma.
    let ordem = ordem::devolver_o_windows(contexto.firmware)?;

    contexto.registro.info("resultado · nao ha job a colher");

    print!(
        "{}",
        montar_sem_job(&pastas, dispositivo.vault.livre_bytes, &ordem)
    );
    Ok(())
}

pub fn montar_sem_job(pastas: &[Pasta], livre_bytes: u64, ordem: &OrdemDevolvida) -> String {
    let mut saida = String::from("Nao ha job a colher.\n\n");
    saida.push_str(&linha_da_ordem(ordem));
    saida.push('\n');
    saida.push_str(&list::montar(pastas, livre_bytes));
    saida.push_str(concat!(
        "\n  Nao ha `estado.json` no ARCABOOT, entao nenhum job foi armado por este\n",
        "  dispositivo — ou o ultimo ja foi colhido e o arquivo foi levado junto de\n",
        "  uma preparacao. Nada foi desarmado: para isso ha `arca desarmar`.\n"
    ));
    saida.push_str(&conselho_da_ordem(ordem));
    saida
}

/// O job ja foi colhido antes. Nao se colhe duas vezes.
fn ja_colhido(contexto: &Contexto, dispositivo: &Dispositivo, estado: &Estado) -> Resultado<()> {
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;

    // C-13 também aqui, e este é o caso que o pedido de P-20 mais usa: colher
    // duas vezes é o que alguém faz quando quer conferir. O ADR-0008 decidiu
    // que colher não se repete, e isso continua valendo para o job — mas a
    // ordem de boot não é o job, e recusar-se a arrumá-la porque o desfecho já
    // foi lido seria deixar a máquina bootando no dispositivo por um motivo
    // que não tem nada que ver.
    let ordem = ordem::devolver_o_windows(contexto.firmware)?;

    contexto.registro.info(format!(
        "resultado · o job {} ja estava colhido",
        estado.descricao()
    ));

    print!(
        "{}",
        montar_ja_colhido(estado, &pastas, dispositivo.vault.livre_bytes, &ordem)
    );
    Ok(())
}

pub fn montar_ja_colhido(
    estado: &Estado,
    pastas: &[Pasta],
    livre_bytes: u64,
    ordem: &OrdemDevolvida,
) -> String {
    let mut saida = String::from("Nao ha job a colher.\n\n");
    saida.push_str(&linha(
        "Ultimo job",
        &format!("{} · ja colhido", estado.descricao()),
    ));
    saida.push_str(&linha("Selo", estado.selo.como_texto()));
    saida.push_str(&linha(
        "Armado em",
        &format!("{} · informativo, nunca comparado", estado.armado_em),
    ));
    saida.push_str(&linha_da_ordem(ordem));

    saida.push('\n');
    saida.push_str(&list::montar(pastas, livre_bytes));
    saida.push_str(&conselho_da_ordem(ordem));
    saida.push_str(concat!(
        "\n  O desfecho deste job ja foi lido e dito. Colher duas vezes nao muda nada,\n",
        "  e o ARCA nao o desarma de novo por isso: para desarmar ha `arca desarmar`.\n"
    ));
    saida
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::desarme::MarcaDeBootUnico;
    use crate::desfecho::NaoEDesfecho;
    use crate::duplos::momento;
    use crate::estado::MomentoDoArmar;
    use crate::imagens::Especie;
    use crate::nome::Nome;
    use crate::receita::{Disco, Operacao, Selo};
    use std::path::PathBuf;

    const DO_JOB: &str = "a3f1c9e07b2d4856";

    /// A ordem que já estava certa, que é o caso em que C-13 não tem o que
    /// dizer.
    ///
    /// É o padrão dos testes desta seção de propósito: eles falam do desfecho
    /// e do job, e um conselho de cinco linhas sobre a ordem de boot no meio
    /// da saída faria as asserções deles passarem por coincidência de
    /// substring. Quem exercita o outro caso são os dois testes ao lado.
    fn ordem() -> OrdemDevolvida {
        OrdemDevolvida::JaEstavaNaFrente
    }

    fn estado(situacao: Situacao) -> Estado {
        Estado {
            selo: Selo::novo(DO_JOB).unwrap(),
            comando: Operacao::Backup,
            nome: Some(Nome::novo("2026-08-22_Apps").unwrap()),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            armado_em: MomentoDoArmar::agora(&crate::duplos::RelogioParado::em(
                "2026-08-22T19:14:03",
            )),
            situacao,
        }
    }

    fn imagem(nome: &str, veredito: Option<Veredito>) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes: 38_823_623_035,
            modificado_em: Some(momento("2026-08-22T09:14:02")),
            especie: Especie::Imagem { veredito },
        }
    }

    fn desarme() -> Desarme {
        Desarme {
            caminho_do_grub: PathBuf::from(r"R:\boot\grub\grub.cfg"),
            blocos_removidos: 1,
            default_devolvido: true,
            grub_regravado: true,
            boot_unico: MarcaDeBootUnico::Removida {
                entradas: vec!["{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string()],
            },
        }
    }

    fn colher(desfecho: Encontrado, veredito: Option<Veredito>) -> String {
        let encerramento = if encerra_o_job(&desfecho) {
            Encerramento::Encerrado
        } else {
            Encerramento::ContinuaPendente
        };
        colher_com(desfecho, veredito, encerramento)
    }

    fn colher_com(
        desfecho: Encontrado,
        veredito: Option<Veredito>,
        encerramento: Encerramento,
    ) -> String {
        colher_com_ordem(desfecho, veredito, encerramento, ordem())
    }

    fn colher_com_ordem(
        desfecho: Encontrado,
        veredito: Option<Veredito>,
        encerramento: Encerramento,
        ordem: OrdemDevolvida,
    ) -> String {
        let estado = estado(Situacao::Armado);
        let pastas = vec![
            imagem("2026-08-21_WindowsCompleto", Some(Veredito::Aprovada)),
            imagem("2026-08-22_Apps", veredito),
        ];
        let desarme = desarme();
        let pasta = pastas.iter().find(|p| p.nome == "2026-08-22_Apps");

        montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta,
            sondagem: None,
            desarme: &desarme,
            ordem: &ordem,
            encerramento: &encerramento,
            pastas: &pastas,
            livre_bytes: 176_312_811_520,
        })
    }

    #[test]
    fn a_ordem_devolvida_sai_em_linha_propria_e_nomeia_quem_estava_na_frente() {
        // A E8 registrou que misturar "colhi" com "arrumei" tira de quem lê a
        // informacao de qual das duas aconteceu. Sao duas linhas, e esta diz o
        // que estava na frente — o `arca status` ja nomeia a entrada desde a
        // revisao do marco da E8, e o conserto nao pode dizer menos.
        let saida = colher_com_ordem(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
            Encerramento::Encerrado,
            OrdemDevolvida::Devolvida {
                estava_na_frente: "ARCA · {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}".to_string(),
            },
        );

        assert!(saida.contains("Ordem de boot"), "{saida}");
        assert!(
            saida.contains("devolvida · o Windows voltou ao topo"),
            "{saida}"
        );
        assert!(saida.contains("ARCA · {f4057bd0-"), "{saida}");

        // E a linha do desarmar continua sendo a do desarmar. As duas acoes
        // aconteceram, e nenhuma pode ser lida como a outra.
        assert!(saida.contains("Desarmando SSD"), "{saida}");

        // O conselho so aparece quando houve conserto, e ele responde a
        // pergunta que fez C-13 existir: e agora, ligando com o SSD na mesa?
        assert!(
            saida.contains("liga no Windows, com o SSD conectado ou sem ele"),
            "{saida}"
        );
    }

    #[test]
    fn sem_conserto_a_saida_nao_promete_conserto_nenhum() {
        // Um `ok` sobre acao que nao aconteceu e a mentira que este projeto ja
        // contou duas vezes (§11), e a E4 pegou a versao dela no desarmar —
        // "Havia receita armada" quando nao havia. A linha existe sempre; o
        // que ela **diz** muda.
        let saida = colher(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        assert!(
            saida.contains("ok · o Windows ja era o primeiro"),
            "{saida}"
        );
        assert!(
            !saida.contains("voltou ao topo"),
            "a saida afirma um conserto que nao houve: {saida}"
        );
        assert!(
            !saida.contains("liga no Windows, com o SSD conectado"),
            "o conselho do conserto saiu sem conserto: {saida}"
        );
    }

    #[test]
    fn a_ordem_e_consertada_tambem_quando_nao_ha_job() {
        // C-13 nao fala de job: a ordem permanente esta suja ou nao esta,
        // tenha alguem armado alguma coisa ou nao. E o caminho por onde o
        // conserto chega a quem rodou `arca resultado` so para conferir.
        let pastas = vec![imagem("2026-08-22_Apps", Some(Veredito::Aprovada))];
        let devolvida = OrdemDevolvida::Devolvida {
            estava_na_frente: "ARCA · {f4057bd0}".to_string(),
        };

        let sem_job = montar_sem_job(&pastas, 176_312_811_520, &devolvida);
        let ja_colhido = montar_ja_colhido(
            &estado(Situacao::Colhido),
            &pastas,
            176_312_811_520,
            &devolvida,
        );

        for saida in [&sem_job, &ja_colhido] {
            assert!(saida.contains("Ordem de boot"), "{saida}");
            assert!(saida.contains("o Windows voltou ao topo"), "{saida}");
            // **O conselho, e nao so a linha.** A primeira versao deste teste
            // cobrava so a linha, e com isso deixou passar um
            // `montar_ja_colhido` que tinha a linha e **nao** tinha o
            // conselho. Quem rodou o comando de verdade viu; o teste nao.
            assert!(
                saida.contains("liga no Windows, com o SSD conectado ou sem ele"),
                "a saida conserta e nao diz o que isso significa: {saida}"
            );
        }

        // E nenhum dos dois passa a dizer que desarmou: continuam sendo os
        // caminhos que **nao** desarmam, e o texto que diz isso fica.
        assert!(sem_job.contains("Nada foi desarmado"), "{sem_job}");
        assert!(ja_colhido.contains("nao o desarma de novo"), "{ja_colhido}");
    }

    #[test]
    fn a_saida_tem_as_quatro_partes_do_paragrafo_5_4() {
        let saida = colher(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        // 1. o desfecho do job, 2. o veredito da imagem,
        // 3. o `Desarmando SSD`, 4. a listagem.
        assert!(saida.contains("Backup 2026-08-22_Apps"));
        assert!(saida.contains("Desfecho: concluida"));
        assert!(saida.contains("Verificacao: APROVADA"));
        assert!(saida.contains("Desarmando SSD"));
        assert!(saida.contains("Imagens em ARCAVAULT:"));
        assert!(saida.contains("164 GB livres"));
    }

    #[test]
    fn desfecho_ok_com_imagem_reprovada_mostra_os_dois_sem_um_esconder_o_outro() {
        // S-5 na letra. O caso construido e o **dificil**: os dois sinais
        // discordam, e o desenho inteiro do §5.5 e do ADR-0003 existe porque
        // eles sao independentes.
        let saida = colher(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Reprovada),
        );

        assert!(saida.contains("Desfecho: concluida"), "{saida}");
        assert!(saida.contains("Verificacao: REPROVADA"), "{saida}");
        assert!(saida.contains("FALHA PARCIAL E FALHA TOTAL"), "{saida}");
    }

    /// A mesma colheita, com o job sendo uma **restauracao**.
    fn colher_restauracao(desfecho: Encontrado, veredito: Option<Veredito>) -> String {
        let estado = Estado {
            comando: Operacao::Restauracao,
            ..estado(Situacao::Armado)
        };
        let pastas = vec![imagem("2026-08-22_Apps", veredito)];
        let desarme = desarme();
        let pasta = pastas.first();

        montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta,
            sondagem: None,
            desarme: &desarme,
            ordem: &ordem(),
            encerramento: &Encerramento::Encerrado,
            pastas: &pastas,
            livre_bytes: 176_312_811_520,
        })
    }

    #[test]
    fn a_colheita_de_uma_restauracao_nao_chama_a_imagem_de_origem_de_verificacao() {
        let saida = colher_restauracao(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        assert!(saida.contains("Restauracao 2026-08-22_Apps"), "{saida}");
        assert!(saida.contains("Desfecho: concluida"), "{saida}");
        assert!(
            saida.contains("Imagem de origem: APROVADA — veredito do backup que a criou"),
            "a linha tem de dizer de quem e o veredito:\n{saida}"
        );
        assert!(
            !saida.contains("Verificacao:"),
            "nao ha verificacao numa restauracao — a imagem e a origem:\n{saida}"
        );
    }

    /// A mesma colheita, com o job sendo uma **verificacao armada** (V-2, E11).
    fn colher_verificacao(desfecho: Encontrado, veredito: Option<Veredito>) -> String {
        let estado = Estado {
            comando: Operacao::Verificacao,
            // O `ocs-chkimg` opera sobre a imagem, e nao sobre disco nenhum.
            disco: None,
            ..estado(Situacao::Armado)
        };
        let pastas = vec![imagem("2026-08-22_Apps", veredito)];
        let desarme = desarme();
        let pasta = pastas.first();

        montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta,
            sondagem: None,
            desarme: &desarme,
            ordem: &ordem(),
            encerramento: &Encerramento::Encerrado,
            pastas: &pastas,
            livre_bytes: 176_312_811_520,
        })
    }

    #[test]
    fn a_colheita_de_uma_verificacao_diz_que_os_dois_sinais_sao_um_so() {
        // A diferenca que separa esta tela das outras duas. Num backup o
        // desfecho e o veredito sao **independentes** (§4.3, S-5): um vem do
        // `savedisk` e o outro do `ocs-chkimg`. Aqui os dois saem do **mesmo**
        // `if`, sobre o mesmo codigo de saida — e quem lê duas linhas
        // concordando merece saber se sao duas testemunhas ou uma so falando
        // duas vezes.
        let saida = colher_verificacao(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        assert!(saida.contains("Verificacao 2026-08-22_Apps"), "{saida}");
        assert!(saida.contains("Desfecho: concluida"), "{saida}");
        assert!(
            saida.contains("Veredito: APROVADA — mesmo sinal do desfecho acima, e nao um segundo"),
            "a linha tem de dizer que os dois sinais tem a mesma fonte:\n{saida}"
        );
        assert!(
            !saida.contains("Imagem de origem:"),
            "esse rotulo e da restauracao, onde a imagem e a origem:\n{saida}"
        );
    }

    #[test]
    fn uma_verificacao_reprovada_sai_como_falha() {
        // S-5 vale igual: uma imagem reprovada pelo `ocs-chkimg` e falha, e o
        // comando sai com codigo diferente de zero depois de imprimir a tela.
        let saida = colher_verificacao(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Reprovada),
        );

        assert!(saida.contains("Veredito: REPROVADA"), "{saida}");
        assert!(saida.contains("FALHA PARCIAL E FALHA TOTAL"), "{saida}");
    }

    #[test]
    fn uma_verificacao_nao_ganha_o_conselho_da_restauracao() {
        // As linhas do §6.3 falam de coisas que so acontecem numa restauracao:
        // o `arca.log` que veio de dentro da imagem, o Windows que acabou de
        // voltar, o SSD que ainda esta conectado. Numa verificacao nenhuma
        // delas e verdade, e um conselho que sai onde nao vale e ruido que
        // ensina quem lê a pular o texto.
        let saida = colher_verificacao(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        assert!(!saida.contains("A RESTAURACAO TERMINOU"), "{saida}");
        // A sentinela da ausencia mudou junto com o texto: ate 27/08/2026 era
        // `DESTRUIDO`, do paragrafo do `%LOCALAPPDATA%` que saiu da tela.
        assert!(!saida.contains("veio de dentro da imagem"), "{saida}");
    }

    #[test]
    fn uma_restauracao_concluida_sem_veredito_nao_sai_como_falha_parcial() {
        // O defeito de sujeito, agora pelo lado da tela. Ate a E9 esta saida
        // trazia `FALHA PARCIAL E FALHA TOTAL (S-5)` numa restauracao que deu
        // certo, porque a imagem de origem nao tinha `arca-check.log`.
        let saida = colher_restauracao(Encontrado::Arquivo(Julgamento::Concluida), None);

        assert!(!saida.contains("FALHA PARCIAL"), "{saida}");
        assert!(saida.contains("A RESTAURACAO TERMINOU"), "{saida}");
    }

    #[test]
    fn a_restauracao_concluida_diz_o_que_conferir_e_onde_o_log_ficou() {
        let saida = colher_restauracao(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        // **Quem lê esta tela ja religou**, e a versao anterior mandava
        // religar. Ver [`super::conselho_da_restauracao`]: o `arca restore`
        // manda reconectar o SSD so para colher, entao o Windows subir e
        // pre-requisito para o comando rodar, e nao tarefa pendente.
        assert!(
            saida.contains("este Windows ja e o que voltou da imagem"),
            "{saida}"
        );
        assert!(saida.contains("Confira\n  seus programas"), "{saida}");
        // O log do Clonezilla desta operacao, que sobreviveu no ARCAVAULT.
        assert!(
            saida.contains(r"ARCA-LOGS\restauracao-2026-08-22_Apps\arca-restore.log"),
            "{saida}"
        );
        // §4.1 com dente, agora numa linha: o `arca.log` de la e de outro tempo.
        assert!(
            saida.contains("`arca.log` do C: veio de dentro da imagem"),
            "{saida}"
        );
        // E a janela do ADR-0009, que o desarmar acima fechou — a resposta ao
        // aviso em caixa alta que o `arca restore` deu antes de reiniciar.
        assert!(
            saida.contains("religar com ele conectado nao restaura mais"),
            "{saida}"
        );
    }

    #[test]
    fn o_conselho_da_restauracao_nao_explica_o_projeto() {
        // Irmao de `o_conselho_nao_explica_o_projeto`, e a quarta tela deste
        // projeto a perder a explicacao. Aqui eram tres paragrafos: a
        // comparacao com os dois sinais do backup (`ocs-chkimg`, B-9, P-6), o
        // caminho do `%LOCALAPPDATA%` com o `estado.json` de §4.1, e o
        // mecanismo da janela do ADR-0009.
        //
        // Quem acabou de restaurar quer saber **o que conferir**, **onde o log
        // ficou** e **se pode religar com o SSD plugado**. Por que nao ha
        // segundo juiz do resultado e registro de projeto: mora no `///` de
        // [`super::conselho_da_restauracao`] e nos ADRs.
        let saida = colher_restauracao(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        for vazamento in [
            "(P-6)",
            "§4.1",
            "%LOCALAPPDATA%",
            "ocs-chkimg",
            "ocs-sr",
            "B-9",
            "ADR-",
            "estado.json",
            "UM sinal so",
        ] {
            assert!(
                !saida.contains(vazamento),
                "o conselho vazou `{vazamento}`, que e registro de projeto:\n{saida}"
            );
        }

        // E o pedido impossivel que saiu em 27/08/2026: quem lê ja religou.
        assert!(
            !saida.contains("religue e confira"),
            "a tela voltou a mandar religar quem ja religou:\n{saida}"
        );
    }

    #[test]
    fn uma_restauracao_que_falhou_continua_apontando_o_log() {
        let saida = colher_restauracao(
            Encontrado::Arquivo(Julgamento::Falhou),
            Some(Veredito::Aprovada),
        );

        assert!(saida.contains("O CLONEZILLA FALHOU E DISSE"), "{saida}");
        assert!(!saida.contains("A RESTAURACAO TERMINOU"), "{saida}");
    }

    #[test]
    fn o_conjunto_e_julgado_pelo_pior_dos_dois() {
        let aprovada = imagem("2026-08-22_Apps", Some(Veredito::Aprovada));
        let reprovada = imagem("2026-08-22_Apps", Some(Veredito::Reprovada));
        let sem_veredito = imagem("2026-08-22_Apps", None);
        let residuo = Pasta {
            especie: Especie::Residuo,
            ..aprovada.clone()
        };
        let concluida = Encontrado::Arquivo(Julgamento::Concluida);

        assert!(julgar_o_conjunto(&concluida, Some(&aprovada), Operacao::Backup).is_none());

        // Cada um destes e uma forma de nao ter dado certo, e nenhuma delas
        // pode sair como exito.
        for (pasta, o_que) in [
            (Some(&reprovada), "reprovada"),
            (Some(&sem_veredito), "sem veredito"),
            (Some(&residuo), "residuo"),
            (None, "sem pasta"),
        ] {
            assert!(
                julgar_o_conjunto(&concluida, pasta, Operacao::Backup).is_some(),
                "desfecho OK com imagem {o_que} passou por exito"
            );
        }

        // E o desfecho ruim reprova qualquer que seja o veredito.
        for desfecho in [
            Encontrado::Arquivo(Julgamento::Falhou),
            Encontrado::Arquivo(Julgamento::Truncado),
            Encontrado::Arquivo(Julgamento::JobFantasma {
                encontrado: Selo::novo("7e02b4d1af963c85").unwrap(),
            }),
            Encontrado::Arquivo(Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo)),
            Encontrado::SemArquivo,
            Encontrado::NaoDeuParaLer {
                motivo: "x".to_string(),
            },
        ] {
            assert!(
                julgar_o_conjunto(&desfecho, Some(&aprovada), Operacao::Backup).is_some(),
                "`{desfecho}` com imagem aprovada passou por exito"
            );
        }
    }

    #[test]
    fn numa_restauracao_o_veredito_da_imagem_de_origem_nao_reprova_a_operacao() {
        // O defeito que a E9 achou relendo `julgar_o_conjunto` procurando o
        // que a restauracao muda nela. A pasta e a imagem **de origem**: o
        // veredito dela e do backup que a criou, e nao diz nada sobre esta
        // operacao ter dado certo.
        //
        // O caso caro e o `sem veredito`: uma imagem trazida de outro
        // dispositivo, ou verificada por `arca verify` em vez de por B-9, nao
        // tem `arca-check.log` — e ate a E9 uma restauracao bem-sucedida a
        // partir dela saia relatada como falha.
        let concluida = Encontrado::Arquivo(Julgamento::Concluida);

        for (pasta, o_que) in [
            (
                Some(imagem("2026-08-22_Apps", Some(Veredito::Aprovada))),
                "aprovada",
            ),
            (
                Some(imagem("2026-08-22_Apps", Some(Veredito::Reprovada))),
                "reprovada",
            ),
            (Some(imagem("2026-08-22_Apps", None)), "sem veredito"),
            (None, "sem pasta"),
        ] {
            assert!(
                julgar_o_conjunto(&concluida, pasta.as_ref(), Operacao::Restauracao).is_none(),
                "uma restauracao concluida com a imagem de origem {o_que} tem de passar"
            );
        }

        // E o desfecho continua mandando: a restauracao nao ganhou passe
        // livre, ela perdeu o segundo juiz — que era de outra pergunta.
        let origem = imagem("2026-08-22_Apps", Some(Veredito::Aprovada));
        assert!(
            julgar_o_conjunto(
                &Encontrado::Arquivo(Julgamento::Falhou),
                Some(&origem),
                Operacao::Restauracao
            )
            .is_some(),
            "`ARCA_RESTORE=FALHOU` tem de reprovar"
        );
        assert!(
            julgar_o_conjunto(&Encontrado::SemArquivo, None, Operacao::Restauracao).is_some(),
            "ausencia de desfecho e falha, nunca silencio (C-12)"
        );
    }

    #[test]
    fn numa_sondagem_a_falta_de_pasta_nao_reprova_a_operacao() {
        // **A mutação que a falsificação pegou faltando.** Trocar a linha da
        // sondagem em `o_veredito_fala_desta_operacao` por `true` passava por
        // toda a suíte — e o estrago é grande: **toda sondagem bem-sucedida
        // sairia com código de erro**, porque não há pasta de imagem para
        // julgar e o ramo `None` diz *"o que a receita disse ter gravado não
        // está lá"*.
        //
        // É a mesma confusão de sujeito que a E9 achou na restauração, na
        // terceira operação em que ela aparece: a sondagem não produz imagem
        // nenhuma, e exigir uma dela é julgar uma coisa pelo parecer de outra.
        let concluida = Encontrado::Arquivo(Julgamento::Concluida);

        assert!(
            julgar_o_conjunto(&concluida, None, Operacao::Sondagem).is_none(),
            "uma sondagem concluida saiu como falha por nao haver pasta de imagem"
        );

        // E o desfecho continua mandando, como em toda operação.
        for desfecho in [
            Encontrado::Arquivo(Julgamento::Falhou),
            Encontrado::Arquivo(Julgamento::Truncado),
            Encontrado::SemArquivo,
        ] {
            assert!(
                julgar_o_conjunto(&desfecho, None, Operacao::Sondagem).is_some(),
                "`{desfecho}` numa sondagem tem de reprovar"
            );
        }
    }

    #[test]
    fn a_pergunta_certa_nao_e_se_a_operacao_mexe_numa_imagem() {
        // A tabela inteira num lugar só, e ela existe porque a formulação
        // errada é **plausível**: "as que produzem imagem" deixaria a
        // verificação de fora, e ela é justamente uma das que devem entrar — o
        // `arca-check.log` que ela lê é o `ocs-chkimg` **daquela** execução.
        assert!(o_veredito_fala_desta_operacao(Operacao::Backup));
        assert!(o_veredito_fala_desta_operacao(Operacao::Verificacao));
        assert!(!o_veredito_fala_desta_operacao(Operacao::Restauracao));
        assert!(!o_veredito_fala_desta_operacao(Operacao::Sondagem));
    }

    #[test]
    fn a_colheita_de_uma_sondagem_diz_os_discos_e_manda_seguir() {
        // O que a sondagem entrega não é um veredito sobre imagem: é o nome
        // que o **Linux** dá ao disco desta máquina. Sem esta linha, quem
        // colhesse leria `ARCA_PROBE=OK` e não saberia o que fazer com aquilo.
        let discos = crate::blkdev::ler(concat!(
            "KNAME     NAME          SIZE TYPE FSTYPE MOUNTPOINT MODEL\n",
            "nvme0n1   nvme0n1     465.8G disk                   KINGSTON SNV3S500G\n",
        ));
        let estado = Estado {
            comando: Operacao::Sondagem,
            nome: None,
            disco: None,
            ..estado(Situacao::Armado)
        };
        let desarme = desarme();
        let desfecho = Encontrado::Arquivo(Julgamento::Concluida);

        let saida = montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta: None,
            sondagem: Some(&discos),
            desarme: &desarme,
            ordem: &OrdemDevolvida::JaEstavaNaFrente,
            encerramento: &Encerramento::Encerrado,
            pastas: &[],
            livre_bytes: 176_000_000_000,
        });

        // O cabeçalho não inventa nome de imagem nenhum.
        assert!(saida.starts_with("Sondagem\n"), "{saida}");
        assert!(
            !saida.contains("nao ha pasta com este nome"),
            "a ausência de imagem virou queixa: {saida}"
        );
        assert!(saida.contains("nao opera sobre imagem nenhuma"), "{saida}");

        // O que ela viu, e o que fazer com isso.
        assert!(
            saida.contains("Discos vistos: nvme0n1 (KINGSTON SNV3S500G)"),
            "{saida}"
        );
        // Onde o arquivo ficou, e qual e o proximo comando. **Nao** por que ele
        // existe: isso e o §4.5, e o §4.5 nao e assunto de quem acabou de
        // colher uma sondagem. Ver `o_conselho_nao_explica_o_projeto`.
        assert!(saida.contains(r"ARCA-LOGS\sondagem\blkdev.list"), "{saida}");
        assert!(saida.contains("arca backup <nome> --dry-run"), "{saida}");
    }

    #[test]
    fn o_conselho_nao_explica_o_projeto() {
        // **A terceira tela deste projeto a perder a explicação, e as três
        // saíram pelo mesmo motivo.** As outras duas são do `arca prepare`: o
        // parágrafo que justificava o esquema de partição, e o que explicava
        // por que sondar.
        //
        // Quem acabou de colher um job quer saber **o que ficou onde** e **qual
        // é o próximo comando**. O oráculo do §4.5, o mecanismo do ciclo de
        // boot e o número da regra que foi respeitada são registro de projeto —
        // moram nos ADRs, e quem lê a tela não os pediu.
        let discos = crate::blkdev::ler(concat!(
            "KNAME     NAME          SIZE TYPE FSTYPE MOUNTPOINT MODEL\n",
            "nvme0n1   nvme0n1     465.8G disk                   KINGSTON SNV3S500G\n",
        ));
        let estado = Estado {
            comando: Operacao::Sondagem,
            nome: None,
            disco: None,
            ..estado(Situacao::Armado)
        };
        let desarme = desarme();
        let desfecho = Encontrado::Arquivo(Julgamento::Concluida);

        // A ordem **consertada**, que é o caso em que o conselho da ordem
        // aparece: o ciclo de boot tinha posto o `ARCA` na frente.
        let saida = montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta: None,
            sondagem: Some(&discos),
            desarme: &desarme,
            ordem: &OrdemDevolvida::Devolvida {
                estava_na_frente: "ARCA".to_string(),
            },
            encerramento: &Encerramento::Encerrado,
            pastas: &[],
            livre_bytes: 176_000_000_000,
        });

        for vazamento in [
            "§4.5",
            "ORACULO",
            "(C-13)",
            "ADR-",
            "boot unico nenhum",
            "ciclo de boot",
        ] {
            assert!(
                !saida.contains(vazamento),
                "o conselho vazou `{vazamento}`, que e registro de projeto:\n{saida}"
            );
        }

        // E o que ele **tem** de dizer continua lá.
        assert!(saida.contains("blkdev.list"), "onde ficou: {saida}");
        assert!(saida.contains("arca backup"), "o proximo comando: {saida}");
        assert!(
            saida.contains("liga no Windows"),
            "o que muda para quem religa a maquina: {saida}"
        );
    }

    #[test]
    fn uma_sondagem_que_falhou_aponta_o_arquivo_que_tem_a_mensagem() {
        // **A primeira tela de `FALHOU` deste projeto**, rodada em hardware em
        // 24/08/2026 com uma coluna inventada no `lsblk`. O conselho genérico
        // dizia *"o log da operação está em ARCA-LOGS\sondagem\"* — e ali há
        // **um** arquivo com **uma** linha, que é a resposta. Mandar procurar
        // na pasta seria esconder a resposta a um `cd` de distância.
        //
        // Nas outras três o log tem centenas de linhas de progresso, e "olhe a
        // pasta" é o melhor que se pode dizer. Por isso a sondagem tem conselho
        // próprio, e não uma frase que serve mal às quatro.
        let estado = Estado {
            comando: Operacao::Sondagem,
            nome: None,
            disco: None,
            ..estado(Situacao::Armado)
        };
        let desarme = desarme();
        let desfecho = Encontrado::Arquivo(Julgamento::Falhou);

        let saida = montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta: None,
            sondagem: Some(&[]),
            desarme: &desarme,
            ordem: &OrdemDevolvida::JaEstavaNaFrente,
            encerramento: &Encerramento::Encerrado,
            pastas: &[],
            livre_bytes: 176_000_000_000,
        });

        assert!(saida.contains(r"ARCA-LOGS\sondagem\blkdev.list"), "{saida}");
        assert!(saida.contains("2>&1"), "e por que ela sobreviveu: {saida}");
        assert!(
            saida.contains("RECONSTRUCAO"),
            "e a causa provável: {saida}"
        );
        assert!(
            !saida.contains("O CLONEZILLA FALHOU"),
            "quem falhou foi o `lsblk`, e a tela tem de dizer isso: {saida}"
        );
    }

    #[test]
    fn uma_sondagem_que_concluiu_sem_lista_diz_que_as_duas_coisas_nao_cabem_juntas() {
        // O desfecho diz `OK` e não há `blkdev.list`: duas afirmações que não
        // cabem juntas, e quem lê precisa saber que a segunda vale. É o caso
        // que o `if` da receita torna raro — com o `lsblk` falhando o desfecho
        // diria `FALHOU` —, e este ramo cobre o que sobra dele.
        let estado = Estado {
            comando: Operacao::Sondagem,
            nome: None,
            disco: None,
            ..estado(Situacao::Armado)
        };
        let desarme = desarme();
        let desfecho = Encontrado::Arquivo(Julgamento::Concluida);

        let saida = montar(&Colheita {
            estado: &estado,
            desfecho: &desfecho,
            pasta: None,
            sondagem: None,
            desarme: &desarme,
            ordem: &OrdemDevolvida::JaEstavaNaFrente,
            encerramento: &Encerramento::Encerrado,
            pastas: &[],
            livre_bytes: 176_000_000_000,
        });

        assert!(saida.contains("nao ha `blkdev.list`"), "{saida}");
        assert!(saida.contains("NAO HA LISTA DE DISCOS"), "{saida}");
        assert!(
            saida.contains("a mensagem dele esta dentro dele"),
            "a tela tem de dizer onde olhar: {saida}"
        );
    }

    #[test]
    fn nao_ter_conseguido_lê_nao_encerra_o_job() {
        // A distincao que a revisao da E5 pagou caro. Um `arca-fim.txt`
        // ilegivel nao e veredito nenhum: o backup pode ter terminado bem, e o
        // que falhou foi olhar. Encerrar o job aqui perderia o selo que liga o
        // desfecho ao job, e a colheita nao poderia ser tentada de novo.
        assert!(!encerra_o_job(&Encontrado::NaoDeuParaLer {
            motivo: "acesso negado".to_string()
        }));

        // As duas que **sao** veredito, inclusive a ausencia de arquivo — que
        // e o C-12 na letra: o boot nao aconteceu, e isso e uma resposta.
        assert!(encerra_o_job(&Encontrado::SemArquivo));
        assert!(encerra_o_job(&Encontrado::Arquivo(Julgamento::Concluida)));
        assert!(encerra_o_job(&Encontrado::Arquivo(Julgamento::Falhou)));
        assert!(encerra_o_job(&Encontrado::Arquivo(Julgamento::Truncado)));
    }

    #[test]
    fn o_relatorio_sai_inteiro_mesmo_quando_o_job_nao_pode_ser_encerrado() {
        // Achado da revisao desta etapa. Gravar antes de imprimir e o certo —
        // uma linha `Job: encerrado` impressa antes da gravacao seria um `ok`
        // sobre uma acao que nao aconteceu (§11) —, mas gravar com `?` fazia o
        // relatorio inteiro se perder quando a escrita falhava. O ARCA teria
        // lido o desfecho do backup e o jogado fora.
        //
        // As duas propriedades cabem juntas: grava-se antes, e o que a
        // gravacao respondeu vai para a linha.
        let saida = colher_com(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
            Encerramento::NaoDeuParaEncerrar {
                motivo: "escrever em R:\\arca\\estado.json falhou: acesso negado".to_string(),
            },
        );

        // O relatorio inteiro continua la.
        assert!(saida.contains("Desfecho: concluida"), "{saida}");
        assert!(saida.contains("Verificacao: APROVADA"), "{saida}");
        assert!(saida.contains("Imagens em ARCAVAULT:"), "{saida}");

        // E a linha nao afirma o que nao houve.
        assert!(saida.contains("NAO FOI POSSIVEL ENCERRAR"), "{saida}");
        assert!(saida.contains("acesso negado"), "{saida}");
        assert!(
            !saida.contains("Job ..... encerrado")
                && !saida.contains("· o desfecho foi lido e dito"),
            "a linha do job disse `encerrado` sem ter encerrado:\n{saida}"
        );
    }

    #[test]
    fn a_falha_ao_encerrar_e_distinta_de_nao_haver_veredito() {
        // As duas deixam o job pendente, e pedem coisas diferentes: uma tem o
        // que consertar agora, a outra e o desenho funcionando.
        let por_acidente = colher_com(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
            Encerramento::NaoDeuParaEncerrar {
                motivo: "x".to_string(),
            },
        );
        let de_proposito = colher(
            Encontrado::NaoDeuParaLer {
                motivo: "acesso negado".to_string(),
            },
            Some(Veredito::Aprovada),
        );

        assert!(
            por_acidente.contains("NAO pôde ser encerrado"),
            "{por_acidente}"
        );
        assert!(
            de_proposito.contains("O JOB CONTINUA PENDENTE"),
            "{de_proposito}"
        );
        assert!(
            !de_proposito.contains("NAO FOI POSSIVEL ENCERRAR"),
            "as duas saidas se confundiram:\n{de_proposito}"
        );
    }

    #[test]
    fn a_saida_diz_quando_o_job_continua_pendente() {
        let saida = colher(
            Encontrado::NaoDeuParaLer {
                motivo: "acesso negado".to_string(),
            },
            Some(Veredito::Aprovada),
        );

        assert!(saida.contains("CONTINUA PENDENTE"), "{saida}");
        assert!(
            saida.contains("NAO e o mesmo que o boot nao ter acontecido"),
            "{saida}"
        );
    }

    #[test]
    fn sem_arca_fim_a_saida_nomeia_as_duas_causas() {
        // C-12: ausencia de desfecho e falha, nunca silencio, e as duas causas
        // possiveis sao nomeadas.
        let saida = colher(Encontrado::SemArquivo, None);

        assert!(saida.contains("O BOOT NAO ACONTECEU"), "{saida}");
        assert!(saida.contains("abriu o menu"), "{saida}");
        assert!(saida.contains("Job"), "{saida}");
        assert!(saida.contains("encerrado"), "{saida}");
    }

    #[test]
    fn o_truncado_diz_que_a_pasta_e_residuo_e_que_o_arca_nao_a_apaga() {
        let saida = colher(Encontrado::Arquivo(Julgamento::Truncado), None);
        assert!(saida.contains("RESIDUO"), "{saida}");
        assert!(saida.contains("nao a apaga (B-10)"), "{saida}");
    }

    #[test]
    fn a_listagem_e_a_mesma_do_list_e_nao_uma_segunda_versao() {
        // Duas versoes da mesma listagem divergem na primeira mudanca. O
        // `arca status` ja reusa `list::montar`; a colheita reusa tambem.
        let pastas = vec![imagem("2026-08-22_Apps", Some(Veredito::Aprovada))];
        let saida = colher(
            Encontrado::Arquivo(Julgamento::Concluida),
            Some(Veredito::Aprovada),
        );

        assert!(
            saida.contains(
                list::montar(&pastas, 176_312_811_520)
                    .lines()
                    .next()
                    .unwrap()
            )
        );
    }

    #[test]
    fn sem_job_a_saida_diz_que_nada_foi_desarmado() {
        let pastas = vec![imagem("2026-08-22_Apps", Some(Veredito::Aprovada))];
        let saida = montar_sem_job(&pastas, 176_312_811_520, &ordem());

        assert!(saida.contains("Nao ha job a colher."));
        assert!(saida.contains("Nada foi desarmado"), "{saida}");
        assert!(saida.contains("Imagens em ARCAVAULT:"));
    }

    #[test]
    fn um_job_ja_colhido_nao_e_colhido_de_novo() {
        // O que fecha a contradicao da E5: depois de colhido, o job deixa de
        // ser pendente e o `arca status` para de mostra-lo como tal.
        let estado = estado(Situacao::Colhido);
        let pastas = vec![imagem("2026-08-22_Apps", Some(Veredito::Aprovada))];
        let saida = montar_ja_colhido(&estado, &pastas, 176_312_811_520, &ordem());

        assert!(saida.contains("ja colhido"), "{saida}");
        assert!(saida.contains("Colher duas vezes nao muda nada"), "{saida}");
    }
}
