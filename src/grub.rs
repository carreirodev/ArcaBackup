//! As duas operacoes inversas sobre o `grub.cfg`: inserir a receita e tira-la.
//!
//! Codigo puro sobre texto. Nao abre arquivo, nao fala com o firmware, nao
//! decide nada sobre o dispositivo. Quem escreve e [`crate::desarme`].
//!
//! # O que armar muda, medido
//!
//! O `grub.cfg` inerte deste dispositivo e a captura
//! `grub-backup-arca-teste-03.cfg` diferem em **exatamente duas coisas**:
//!
//! ```text
//! -set default="live-default"
//! +set default="arca-backup"
//! +
//! +menuentry "ARCA - backup automatico" --id arca-backup {
//! +  search --set -f /live/vmlinuz
//! +  $linux_cmd /live/vmlinuz ... ocs_live_run="bash -c '...'" ...
//! +  $initrd_cmd /live/initrd.img
//! +}
//! ```
//!
//! Nada mais no arquivo muda — nem `timeout`, nem os outros `menuentry`, nem
//! uma linha de comentario. Desarmar e desfazer as duas, e e so isso.
//!
//! # O `set default` e o que faz o boot ser desatendido
//!
//! Este achado nao esta no PRD nem no plano, e passou tres etapas sem
//! aparecer. Inserir o `menuentry` do ARCA **nao arma nada**: ele vira mais
//! uma linha no menu, e a maquina continua esperando trinta segundos e
//! bootando no Clonezilla normal. Quem faz o boot ser desatendido e o
//! `set default` apontar para o id do ARCA. As tres capturas provam os dois
//! lados: `grub-backup-arca-teste-02.cfg` e
//! `grub-restauracao-arca-teste-02.cfg` tem o `menuentry` e **nao** tem o
//! `set default` — nesse estado a maquina nao executaria receita nenhuma.
//!
//! Dai a ordem de importancia do desarmar: devolver o `set default` e o que
//! torna o dispositivo inerte; tirar o bloco e higiene. As duas acontecem na
//! mesma escrita, mas so a primeira e o que separa "boota no menu" de "boota
//! e executa".
//!
//! # Por que `live-default` e nao `0`
//!
//! O `grub.cfg` que o Clonezilla entrega — `grub.cfg.original`, preservado em
//! `recursos/capturas/grub-clonezilla-original.cfg` — traz
//! `set default="0"`, e difere do inerte deste dispositivo **so nisso**.
//!
//! `"0"` aponta por **posicao**, e a posicao muda: o bloco do ARCA entra
//! **antes** do `live-default`, e passa a ser o indice 0. Um dispositivo com
//! `set default="0"` esta armado no instante em que o bloco e inserido, sem
//! que ninguem toque no `set default`. Nao e o estado inerte — e um estado
//! que parece inerte.
//!
//! `"live-default"` aponta pelo `--id` que o proprio Clonezilla da ao seu
//! menuentry padrao (esta no `grub.cfg.original`, nao foi ninguem que
//! inventou), e continua apontando para o mesmo lugar com ou sem bloco do
//! ARCA no meio. E por isso que [`desarmar`] devolve o `set default` para
//! `live-default` **qualquer que seja o valor que encontrou** — inclusive
//! `"0"` —, e nao apenas quando encontra o id do ARCA.

use std::fmt;

/// O `--id` do menuentry que o ARCA insere. E por ele que [`desarmar`] acha o
/// bloco a remover: e uma marca que o **proprio ARCA** escreve, e nao uma
/// heuristica sobre texto alheio.
pub const ID_DO_ARCA: &str = "arca-backup";

/// O `--id` do menuentry padrao do Clonezilla, para onde o `set default`
/// aponta no estado inerte.
///
/// Nao e escolha do ARCA: esta no `grub.cfg` que o Clonezilla entrega, em
/// `menuentry "Clonezilla live (VGA 800x600)" --id live-default`.
pub const ID_INERTE: &str = "live-default";

/// A diretiva que decide em que entrada o `grub` boota sozinho.
const SET_DEFAULT: &str = "set default";

/// O que impediu de armar ou desarmar.
///
/// Toda variante e uma recusa **antes** de gravar: um `grub.cfg` pela metade
/// e pior do que um `grub.cfg` armado, porque o armado ainda boota.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoGrub {
    /// Achou o `menuentry` do ARCA e nao achou a chave que o fecha. Remover
    /// ate o fim do arquivo truncaria o `grub.cfg`, e um `grub.cfg` truncado
    /// e uma maquina que nao boota.
    BlocoSemFechamento { linha: usize },

    /// Nao ha linha `set default` nenhuma. Sem ela o `grub` boota o indice 0,
    /// que e exatamente onde o bloco do ARCA entra — mas onde inserir a
    /// diretiva num arquivo que o ARCA nao escreveu e adivinhacao.
    SemSetDefault,

    /// Nao ha `menuentry` com o id inerte. Sem ele, apontar o `set default`
    /// para `live-default` mandaria o `grub` procurar uma entrada que nao
    /// existe.
    SemMenuentryInerte,

    /// Pediram para armar um `grub.cfg` que ja tem bloco do ARCA. Quem arma
    /// desarma antes (C-1); dois blocos com o mesmo `--id` seriam ambiguidade
    /// gravada em disco.
    JaArmado,
}

impl fmt::Display for RecusaDoGrub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoGrub::BlocoSemFechamento { linha } => write!(
                f,
                "o menuentry do ARCA comeca na linha {linha} do grub.cfg e nao tem a chave que o fecha. Remover ate o fim do arquivo deixaria o grub.cfg truncado, e nada foi gravado"
            ),
            RecusaDoGrub::SemSetDefault => write!(
                f,
                "o grub.cfg nao tem linha `{SET_DEFAULT}`, e sem ela o grub boota a primeira entrada do menu — que e justamente onde o bloco do ARCA entra. Onde inserir a diretiva num arquivo que o ARCA nao escreveu nao e coisa que se adivinhe"
            ),
            RecusaDoGrub::SemMenuentryInerte => write!(
                f,
                "o grub.cfg nao tem menuentry com `--id {ID_INERTE}`, e apontar o `{SET_DEFAULT}` para ele mandaria o grub procurar uma entrada que nao existe"
            ),
            RecusaDoGrub::JaArmado => write!(
                f,
                "o grub.cfg ja tem um menuentry `--id {ID_DO_ARCA}`. Quem arma desarma antes (C-1); dois blocos com o mesmo id seriam ambiguidade gravada em disco"
            ),
        }
    }
}

/// O `grub.cfg` desarmado, e o que precisou ser desfeito para chegar nele.
///
/// O que foi desfeito nao decide nada — desarmar acontece incondicionalmente
/// (C-1) —, mas vai para o registro e para a tela: quem roda o comando merece
/// saber se havia receita armada ou se o dispositivo ja estava inerte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Desarmado {
    pub texto: String,

    /// Quantos `menuentry` do ARCA foram tirados. Normalmente zero ou um;
    /// mais de um so aconteceria se alguma coisa tivesse armado duas vezes
    /// sem desarmar no meio, e o desarmar tira todos.
    pub blocos_removidos: usize,

    /// Se o `set default` apontava para outra coisa e precisou voltar.
    pub default_devolvido: bool,
}

impl Desarmado {
    /// Se havia alguma coisa a desfazer no `grub.cfg`.
    pub fn havia_receita(&self) -> bool {
        self.blocos_removidos > 0 || self.default_devolvido
    }
}

/// Tira do `grub.cfg` tudo que o ARCA pos nele.
///
/// Duas operacoes, na mesma passada: remove os `menuentry` com
/// `--id arca-backup` e aponta o `set default` para [`ID_INERTE`]. O que nao
/// for uma dessas duas coisas sai byte a byte como entrou — inclusive as
/// quebras de linha, que sao LF neste `grub.cfg` e nao podem virar CRLF por
/// causa de uma passagem por texto.
///
/// **Idempotente por construcao**: a segunda passada nao acha bloco nenhum e
/// encontra o `set default` ja no lugar, e devolve o mesmo texto. E o que C-1
/// cobra, e sai de graca por reconstruir do arquivo corrente em vez de
/// comparar com uma copia guardada.
pub fn desarmar(corrente: &str) -> Result<Desarmado, RecusaDoGrub> {
    let mut linhas: Vec<String> = corrente
        .split_inclusive('\n')
        .map(|linha| linha.to_string())
        .collect();

    let mut blocos_removidos = 0;
    while let Some(bloco) = achar_bloco(&linhas, ID_DO_ARCA)? {
        // Junto do bloco sai a linha em branco **de depois**, que e o que
        // `armar` insere — tirar a de depois e a inversa exata. Sem ela ali,
        // nao se tira nada: a linha em branco de **antes** separa o bloco do
        // que veio antes dele e nao foi o ARCA que a pos, e removeria-la
        // colaria duas entradas do Clonezilla uma na outra. Uma linha em
        // branco a mais e inofensiva; mexer no que o ARCA nao escreveu, nao.
        let inicio = bloco.inicio;
        let mut fim = bloco.fim + 1;

        if linhas.get(fim).is_some_and(|linha| e_vazia(linha)) {
            fim += 1;
        }

        linhas.drain(inicio..fim);
        blocos_removidos += 1;
    }

    let default_devolvido = apontar_default(&mut linhas, ID_INERTE)?;

    // Pos-condicao sobre o que **esta funcao** removeu, e nao zelo: tendo
    // tirado bloco, o `set default` do resultado tem de nomear um `menuentry`
    // que ainda existe. Sem isto, um bloco removido a mais deixaria o
    // `grub.cfg` apontando para uma entrada que acabou de sumir — e como o
    // `set default` ja estaria com o valor certo, `apontar_default` sairia
    // cedo sem conferir nada. E a mesma familia do defeito que `achar_bloco`
    // fecha, e as duas defesas custam pouco demais para se escolher uma.
    //
    // So quando houve remocao: um `grub.cfg` em que nada mudou continua sendo
    // o que era antes de o ARCA chegar, e recusa-lo seria o ARCA falhar por
    // uma coisa que nao fez, deixando um dispositivo talvez armado sem
    // desarmar.
    if blocos_removidos > 0 && achar_menuentry(&linhas, ID_INERTE).is_none() {
        return Err(RecusaDoGrub::SemMenuentryInerte);
    }

    Ok(Desarmado {
        texto: linhas.concat(),
        blocos_removidos,
        default_devolvido,
    })
}

/// Poe o `bloco` no `grub.cfg` inerte e aponta o boot para ele.
///
/// A E4 **nao chama esta funcao** — ela existe aqui, e agora, por um motivo
/// so: com as duas metades no mesmo lugar da para provar que uma desfaz a
/// outra contra as capturas reais, em vez de testar o desarmar contra um alvo
/// que o proprio teste inventou. Quem a chama e a E7.
///
/// O `bloco` vem pronto de quem chama, com o `menuentry` inteiro. Esta funcao
/// nao monta menuentry: as tres capturas mostram blocos **diferentes** entre
/// si — a `teste-03` perdeu o `hostname` e as blacklists de driver que a
/// `teste-02` tem —, e escolher entre eles e decidir o que o Clonezilla vai
/// receber de linha de comando. Isso e da E7, que arma de verdade; aqui o
/// bloco e dado, e o que se prova e que inserir e tirar se cancelam.
pub fn armar(inerte: &str, bloco: &str) -> Result<String, RecusaDoGrub> {
    let mut linhas: Vec<String> = inerte
        .split_inclusive('\n')
        .map(|linha| linha.to_string())
        .collect();

    if achar_bloco(&linhas, ID_DO_ARCA)?.is_some() {
        return Err(RecusaDoGrub::JaArmado);
    }

    let alvo = achar_menuentry(&linhas, ID_INERTE).ok_or(RecusaDoGrub::SemMenuentryInerte)?;

    // O terminador do arquivo, e nao um `\n` fixo: um `grub.cfg` em CRLF tem
    // de continuar em CRLF depois de armado.
    let terminador = terminador_de(&linhas);
    let mut inserido: Vec<String> = bloco
        .split_inclusive('\n')
        .map(|linha| linha.to_string())
        .collect();
    // Um bloco entregue sem quebra no fim colaria no que vem depois.
    if let Some(ultima) = inserido.last_mut() {
        if !ultima.ends_with('\n') {
            ultima.push_str(terminador);
        }
    }
    // Uma linha em branco separando o bloco do menuentry seguinte, como nas
    // quatro copias armadas. E ela que `desarmar` tira junto.
    inserido.push(terminador.to_string());

    linhas.splice(alvo..alvo, inserido);
    apontar_default(&mut linhas, ID_DO_ARCA)?;

    Ok(linhas.concat())
}

/// O `menuentry` do ARCA, do jeito que ele esta escrito num `grub.cfg` armado.
///
/// E o que permite provar a inversao contra o arquivo de verdade: tira-se o
/// bloco da captura, desarma-se a captura, arma-se de volta com o mesmo
/// bloco, e o resultado tem de ser a captura byte a byte.
pub fn bloco_do_arca(texto: &str) -> Option<String> {
    bloco_com_id(texto, ID_DO_ARCA)
}

/// O `menuentry` com este `--id`, inteiro, da linha de abertura ate a chave
/// que o fecha.
///
/// Existe para a E7: o bloco do ARCA nao se inventa, deriva-se de um
/// `menuentry` que ja esta no `grub.cfg` do dispositivo. Quem faz essa
/// derivacao e [`crate::menuentry`], e o que ela precisa daqui e o mesmo
/// achador de bloco que o desarmar usa — com as mesmas guardas, inclusive a
/// de parar no proximo abridor em vez de engolir dois blocos.
///
/// `None` tanto quando nao ha bloco com esse id quanto quando ha um sem
/// fechamento. Quem chama trata os dois como "nao ha de onde derivar": num
/// `grub.cfg` que o ARCA nao consegue entender, adivinhar e pior do que
/// recusar.
pub fn bloco_com_id(texto: &str, id: &str) -> Option<String> {
    let linhas: Vec<String> = texto
        .split_inclusive('\n')
        .map(|linha| linha.to_string())
        .collect();

    let faixa = achar_bloco(&linhas, id).ok().flatten()?;
    Some(linhas[faixa.inicio..=faixa.fim].concat())
}

/// Onde um `menuentry` comeca e termina, em indices de linha.
struct Faixa {
    inicio: usize,
    fim: usize,
}

/// A faixa do primeiro `menuentry` com este id, se houver.
///
/// O fim e a primeira linha seguinte que so tem `}`. E como o proprio
/// `grub.cfg` escreve — a chave de fechamento sempre sozinha, na coluna zero
/// — e nao contar chaves na linha do `$linux_cmd`, que tem setecentos
/// caracteres de parametros e onde uma chave dentro de uma string enganaria a
/// contagem.
///
/// # A varredura para no proximo bloco, e nao so no fim do arquivo
///
/// "Procurar o `}`" sozinho tem um modo de falha caro, e a revisao desta etapa
/// o encontrou. Num `menuentry` do ARCA que perdeu a chave de fechamento — uma
/// edicao a mao interrompida, um arquivo escrito pela metade —, o primeiro `}`
/// seguinte e o do **proximo** `menuentry`, e a remocao levaria os dois. Num
/// `grub.cfg` de verdade sempre ha um `}` adiante, entao a guarda de "nao ha
/// fechamento nenhum ate o fim" nunca dispararia: medido, o arquivo saiu
/// reduzido a uma linha, com o `menuentry --id live-default` junto.
///
/// Achar um abridor de bloco antes do fechamento e, portanto, o mesmo que nao
/// achar fechamento: recusa, e nada e gravado.
fn achar_bloco(linhas: &[String], id: &str) -> Result<Option<Faixa>, RecusaDoGrub> {
    let Some(inicio) = achar_menuentry(linhas, id) else {
        return Ok(None);
    };

    let sem_fechamento = RecusaDoGrub::BlocoSemFechamento { linha: inicio + 1 };

    for (indice, linha) in linhas.iter().enumerate().skip(inicio + 1) {
        if conteudo(&linha.to_string()) == "}" {
            return Ok(Some(Faixa {
                inicio,
                fim: indice,
            }));
        }
        if abre_bloco(linha) {
            return Err(sem_fechamento);
        }
    }

    Err(sem_fechamento)
}

/// Se a linha abre um bloco do `grub.cfg` — um `menuentry` ou um `submenu`.
///
/// Serve para [`achar_bloco`] saber que passou do fim do bloco que procurava,
/// mesmo sem ter visto a chave que o fecharia.
fn abre_bloco(linha: &str) -> bool {
    let conteudo = conteudo(linha);
    conteudo.starts_with("menuentry") || conteudo.starts_with("submenu")
}

/// O indice da linha `menuentry ... --id <id> {`, se houver.
fn achar_menuentry(linhas: &[String], id: &str) -> Option<usize> {
    linhas
        .iter()
        .position(|linha| id_do_menuentry(linha) == Some(id))
}

/// O `--id` de uma linha de `menuentry`, se ela for uma.
///
/// Lido por token, e nao por `contains`: um `--id arca-backup-antigo` contem
/// `arca-backup`, e remove-lo por engano tiraria do menu uma entrada que nao
/// e do ARCA. As duas formas que o `grub` aceita — `--id x` e `--id=x` — sao
/// reconhecidas, porque custa uma linha e a captura so mostra uma delas.
///
/// **Limite conhecido**: o titulo entre aspas nao e tratado como uma unidade,
/// entao um `menuentry "titulo com --id dentro" --id x` seria lido errado.
/// Nenhum dos vinte e poucos `menuentry` do `grub.cfg` do Clonezilla tem isso,
/// e o unico titulo que o ARCA escreve e o dele proprio. Escrever um parser de
/// aspas para cobrir um caso que nao existe traria mais formas de errar do que
/// resolveria.
fn id_do_menuentry(linha: &str) -> Option<&str> {
    let conteudo = conteudo(linha);
    if !conteudo.starts_with("menuentry") {
        return None;
    }

    let mut pedacos = conteudo.split_whitespace();
    while let Some(pedaco) = pedacos.next() {
        if let Some(id) = pedaco.strip_prefix("--id=") {
            return Some(id);
        }
        if pedaco == "--id" {
            return pedacos.next();
        }
    }
    None
}

/// Aponta o `set default` para este id, e diz se precisou mudar alguma coisa.
///
/// A conferencia de que o `menuentry` alvo existe acontece **so quando ha o
/// que escrever**. Um `grub.cfg` que ja aponta para onde se queria nao e
/// recusado por causa de um `menuentry` que falta: ele estava assim antes de o
/// ARCA chegar, e quebrar por causa disso seria o ARCA falhar por uma coisa
/// que nao fez. Escrever, isso sim, exige que o alvo exista — apontar o `grub`
/// para uma entrada inexistente e pior do que nao mexer.
fn apontar_default(linhas: &mut [String], id: &str) -> Result<bool, RecusaDoGrub> {
    // **Todas** as linhas `set default`, e nao so a primeira. O `grub` executa
    // o arquivo de cima a baixo e a ultima atribuicao vence: trocar so a
    // primeira, num arquivo que tivesse duas, deixaria a segunda mandando — e
    // o ARCA diria que desarmou um dispositivo que continua armado. Nao ha
    // `grub.cfg` assim nas capturas, e essa e exatamente a razao de nao se
    // poder contar com isso.
    let indices: Vec<usize> = linhas
        .iter()
        .enumerate()
        .filter(|(_, linha)| e_set_default(linha))
        .map(|(indice, _)| indice)
        .collect();

    if indices.is_empty() {
        return Err(RecusaDoGrub::SemSetDefault);
    }

    let desejada = format!("{SET_DEFAULT}=\"{id}\"");
    let a_mudar: Vec<usize> = indices
        .into_iter()
        .filter(|indice| conteudo(&linhas[*indice]) != desejada)
        .collect();

    if a_mudar.is_empty() {
        return Ok(false);
    }

    if achar_menuentry(linhas, id).is_none() {
        return Err(RecusaDoGrub::SemMenuentryInerte);
    }

    for indice in a_mudar {
        // O terminador desta linha, e nao um `\n` fixo — e a ultima linha de
        // um arquivo pode nao ter nenhum.
        let terminador = terminador_da(&linhas[indice]);
        linhas[indice] = format!("{desejada}{terminador}");
    }
    Ok(true)
}

/// Se a linha e uma diretiva `set default`.
///
/// O que vem depois de `set default` tem de ser `=` ou espaco: um
/// `starts_with` cru casaria com um hipotetico `set defaultfoo=1` e o
/// reescreveria como se fosse a diretiva.
fn e_set_default(linha: &str) -> bool {
    conteudo(linha)
        .strip_prefix(SET_DEFAULT)
        .is_some_and(|resto| resto.starts_with('=') || resto.starts_with(char::is_whitespace))
}

/// O conteudo de uma linha, sem o terminador e sem os espacos das pontas.
fn conteudo(linha: &str) -> &str {
    linha.trim()
}

fn e_vazia(linha: &str) -> bool {
    conteudo(linha).is_empty()
}

/// O terminador desta linha: `\r\n`, `\n`, ou nada se ela e a ultima e o
/// arquivo nao termina em quebra.
fn terminador_da(linha: &str) -> &'static str {
    if linha.ends_with("\r\n") {
        "\r\n"
    } else if linha.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

/// O terminador que o arquivo usa, pela primeira linha que tem um.
fn terminador_de(linhas: &[String]) -> &'static str {
    linhas
        .iter()
        .map(|linha| terminador_da(linha))
        .find(|terminador| !terminador.is_empty())
        .unwrap_or("\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O `grub.cfg` inerte deste dispositivo — `R:\boot\grub\grub.cfg`.
    ///
    /// E o alvo do desarmar, e o oraculo desta etapa inteira: nao um arquivo
    /// que o teste montou, mas o que esta no dispositivo agora.
    const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");

    /// O `grub.cfg` que o **Clonezilla** entrega, com `set default="0"`.
    const CLONEZILLA: &str = include_str!("../recursos/capturas/grub-clonezilla-original.cfg");

    /// As quatro copias armadas que existem no dispositivo. Tres rodaram em
    /// hardware (as receitas do ADR-0004); a quarta, `teste-01`, e o mesmo
    /// padrao numa quarta ocorrencia.
    const ARMADAS: [(&str, &str); 4] = [
        (
            "backup-teste-01",
            include_str!("../recursos/capturas/grub-backup-arca-teste-01.cfg"),
        ),
        (
            "backup-teste-02",
            include_str!("../recursos/capturas/grub-backup-arca-teste-02.cfg"),
        ),
        (
            "backup-teste-03",
            include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg"),
        ),
        (
            "restauracao-teste-02",
            include_str!("../recursos/capturas/grub-restauracao-arca-teste-02.cfg"),
        ),
    ];

    // ───────────────── o desarmar contra os arquivos de verdade ─────────────

    #[test]
    fn desarmar_uma_captura_devolve_o_inerte_byte_a_byte() {
        // O teste que fecha a etapa. O alvo nao e um arquivo que este teste
        // montou: e o `grub.cfg` que esta no dispositivo agora. Nenhuma das
        // quatro copias armadas pode sair diferente dele.
        for (nome, armada) in ARMADAS {
            let desarmada = desarmar(armada).expect("a captura desarma");
            assert_eq!(
                desarmada.texto, INERTE,
                "desarmar `{nome}` nao devolveu o grub.cfg inerte"
            );
        }
    }

    #[test]
    fn desarmar_o_inerte_nao_muda_nada() {
        // C-1 exige que rodar duas vezes seguidas de o mesmo resultado, e a
        // segunda passada e sempre esta.
        let desarmada = desarmar(INERTE).expect("o inerte desarma");

        assert_eq!(desarmada.texto, INERTE);
        assert_eq!(desarmada.blocos_removidos, 0);
        assert!(!desarmada.default_devolvido);
        assert!(!desarmada.havia_receita());
    }

    #[test]
    fn desarmar_duas_vezes_seguidas_da_o_mesmo_resultado() {
        // C-1 na letra, sobre cada arquivo armado que existe.
        for (nome, armada) in ARMADAS {
            let primeira = desarmar(armada).expect("primeira passada");
            let segunda = desarmar(&primeira.texto).expect("segunda passada");

            assert_eq!(primeira.texto, segunda.texto, "`{nome}` mudou na segunda");
            assert!(primeira.havia_receita(), "`{nome}` estava armada");
            assert!(
                !segunda.havia_receita(),
                "`{nome}` continuava armada depois de desarmada"
            );
        }
    }

    #[test]
    fn o_que_desarmar_muda_sao_duas_coisas_e_nao_mais() {
        // O achado que muda a etapa, cobrado como teste: entre um `grub.cfg`
        // armado e o inerte ha exatamente duas diferencas — o `set default` e
        // o bloco. Se um dia o desarmar passar a mexer em `timeout`, num
        // comentario ou noutro `menuentry`, este teste denuncia.
        let armada = ARMADAS[2].1;
        let diferentes: Vec<&str> = armada
            .lines()
            .filter(|linha| !INERTE.lines().any(|inerte| inerte == *linha))
            .collect();

        assert!(
            diferentes.iter().all(|linha| linha
                .trim()
                .starts_with(&format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\""))
                || linha.contains(ID_DO_ARCA)
                || linha.trim() == "search --set -f /live/vmlinuz"
                || linha.trim().starts_with("$linux_cmd")
                || linha.trim().starts_with("$initrd_cmd")),
            "linha inesperada fora do bloco do ARCA: {diferentes:?}"
        );
    }

    #[test]
    fn o_set_default_e_devolvido_mesmo_sem_bloco_para_remover() {
        // As capturas `teste-02` tem o bloco e **nao** tem o `set default`
        // apontando para ele. O caso simetrico — `set default` apontando para
        // o ARCA sem bloco nenhum — e uma maquina que reinicia e para num
        // menuentry inexistente. Desarmar cobre os dois.
        let so_o_default = INERTE.replace(
            &format!("{SET_DEFAULT}=\"{ID_INERTE}\""),
            &format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\""),
        );

        let desarmada = desarmar(&so_o_default).expect("desarma");
        assert_eq!(desarmada.texto, INERTE);
        assert_eq!(desarmada.blocos_removidos, 0);
        assert!(desarmada.default_devolvido);
    }

    #[test]
    fn o_bloco_e_removido_mesmo_com_o_default_ja_no_lugar() {
        // E o estado das duas capturas `teste-02`, byte a byte como estao no
        // dispositivo: bloco presente, `set default` ja em `live-default`.
        let (_, teste_02) = ARMADAS[1];
        assert!(teste_02.contains(&format!("{SET_DEFAULT}=\"{ID_INERTE}\"")));
        assert!(teste_02.contains(&format!("--id {ID_DO_ARCA}")));

        let desarmada = desarmar(teste_02).expect("desarma");
        assert_eq!(desarmada.blocos_removidos, 1);
        assert!(!desarmada.default_devolvido, "o default ja estava certo");
        assert_eq!(desarmada.texto, INERTE);
    }

    #[test]
    fn o_grub_cfg_do_clonezilla_vira_o_inerte_deste_dispositivo() {
        // A resposta a "de onde vem o estado inerte". O que o Clonezilla
        // entrega difere do inerte deste dispositivo **so** no `set default`,
        // que ele traz como `"0"` — por posicao. Desarmar o dele produz o
        // nosso, byte a byte, e essa e a prova de que o inerte nao e um
        // arquivo guardado em lugar nenhum: e o que sai de aplicar a regra.
        assert!(CLONEZILLA.contains(&format!("{SET_DEFAULT}=\"0\"")));

        let desarmada = desarmar(CLONEZILLA).expect("desarma");
        assert_eq!(desarmada.texto, INERTE);
        assert!(desarmada.default_devolvido);
        assert_eq!(desarmada.blocos_removidos, 0);
    }

    #[test]
    fn o_default_por_posicao_e_devolvido_porque_a_posicao_muda() {
        // `set default="0"` aponta para a primeira entrada, e o bloco do ARCA
        // entra **antes** do `live-default`: com `"0"`, inserir o bloco arma
        // sozinho. Um dispositivo assim nao esta inerte, esta parecendo
        // inerte — e e por isso que o desarmar nao deixa `"0"` passar.
        let armado_com_zero = armar(CLONEZILLA, &bloco_de_exemplo()).expect("arma");
        let posicao_do_arca = armado_com_zero
            .lines()
            .position(|linha| id_do_menuentry(linha) == Some(ID_DO_ARCA));
        let posicao_do_inerte = armado_com_zero
            .lines()
            .position(|linha| id_do_menuentry(linha) == Some(ID_INERTE));

        assert!(
            posicao_do_arca < posicao_do_inerte,
            "o bloco do ARCA entra antes do menuentry padrao, e por isso `0` o alcancaria"
        );
    }

    // ───────────────── armar e desarmar se cancelam ─────────────────

    #[test]
    fn armar_o_bloco_da_copia_armada_reproduz_a_copia_byte_a_byte() {
        // O teste que so e possivel com as duas metades no mesmo lugar, e o
        // motivo de `armar` existir na E4. O oraculo nao e um alvo inventado:
        // e o arquivo que saiu do dispositivo. Tira-se o bloco da copia,
        // desarma-se, arma-se de volta com o mesmo bloco — e tem de sair a
        // copia, byte a byte.
        let armadas = de_fato_armadas();
        assert_eq!(
            armadas.len(),
            1,
            "das quatro copias, so a `teste-03` tem o `{SET_DEFAULT}` apontando para o ARCA"
        );

        for (nome, armada) in armadas {
            let bloco = bloco_do_arca(armada).expect("a copia tem bloco do ARCA");
            let inerte = desarmar(armada).expect("desarma").texto;
            let rearmada = armar(&inerte, &bloco).expect("arma");

            assert_eq!(rearmada, armada, "a ida e a volta nao fecharam em `{nome}`");
        }
    }

    #[test]
    fn as_outras_copias_estao_meio_armadas_e_a_diferenca_e_so_o_set_default() {
        // O achado desta etapa, virado teste. Tres das quatro copias tem o
        // `menuentry` do ARCA e o `{SET_DEFAULT}` ainda em `live-default`.
        // Nesse estado a maquina espera trinta segundos e boota no menu
        // normal do Clonezilla — **nao executa receita nenhuma**. Rearmadas,
        // elas diferem da copia numa linha so, e e essa linha que separa
        // "aparece no menu" de "roda sozinho".
        //
        // Por que elas estao assim e pergunta fechada por falta de evidencia
        // (ADR-0005), e nao muda nada aqui: nas duas explicacoes possiveis o
        // `{SET_DEFAULT}` faz parte do que se arma, logo faz parte do que se
        // desarma.
        let meio_armadas: Vec<_> = ARMADAS
            .into_iter()
            .filter(|(_, texto)| !aponta_para_o_arca(texto))
            .collect();
        assert_eq!(meio_armadas.len(), 3);

        for (nome, meio_armada) in meio_armadas {
            let bloco = bloco_do_arca(meio_armada).expect("tem bloco do ARCA");
            let inerte = desarmar(meio_armada).expect("desarma").texto;
            let rearmada = armar(&inerte, &bloco).expect("arma");

            let diferencas: Vec<(&str, &str)> = rearmada
                .lines()
                .zip(meio_armada.lines())
                .filter(|(rearmada, copia)| rearmada != copia)
                .collect();

            assert_eq!(
                diferencas.len(),
                1,
                "`{nome}` divergiu em mais de uma linha: {diferencas:?}"
            );
            assert_eq!(
                diferencas[0].0.trim(),
                format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\""),
                "`{nome}`"
            );
            assert_eq!(
                diferencas[0].1.trim(),
                format!("{SET_DEFAULT}=\"{ID_INERTE}\""),
                "`{nome}`"
            );
        }
    }

    #[test]
    fn armar_e_depois_desarmar_devolve_o_inerte() {
        let armada = armar(INERTE, &bloco_de_exemplo()).expect("arma");
        assert_ne!(armada, INERTE, "armar tem de mudar alguma coisa");

        let desarmada = desarmar(&armada).expect("desarma");
        assert_eq!(desarmada.texto, INERTE);
    }

    #[test]
    fn armar_troca_o_set_default_e_poe_o_bloco_antes_do_menuentry_padrao() {
        let armada = armar(INERTE, &bloco_de_exemplo()).expect("arma");

        assert!(armada.contains(&format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\"")));
        assert!(!armada.contains(&format!("{SET_DEFAULT}=\"{ID_INERTE}\"")));

        let arca = armada.find(&format!("--id {ID_DO_ARCA}")).unwrap();
        let inerte = armada.find(&format!("--id {ID_INERTE}")).unwrap();
        assert!(arca < inerte, "o bloco do ARCA tem de vir antes");
    }

    #[test]
    fn armar_sobre_um_grub_cfg_ja_armado_e_recusado() {
        // Quem arma desarma antes (C-1). Dois blocos com o mesmo `--id` seriam
        // ambiguidade gravada num arquivo de que a maquina depende para
        // bootar.
        let armada = armar(INERTE, &bloco_de_exemplo()).expect("arma");
        assert_eq!(
            armar(&armada, &bloco_de_exemplo()).unwrap_err(),
            RecusaDoGrub::JaArmado
        );
    }

    // ───────────────── o que o desarmar recusa ─────────────────

    #[test]
    fn bloco_sem_chave_de_fechamento_e_recusa_e_nao_truncamento() {
        // O modo de falha mais caro que este modulo poderia ter: remover ate
        // o fim do arquivo. Um `grub.cfg` truncado e uma maquina que nao
        // boota, e um `grub.cfg` armado ainda boota — entao recusar e melhor.
        let quebrada = format!(
            "set default=\"{ID_INERTE}\"\nmenuentry \"x\" --id {ID_DO_ARCA} {{\n  search\nmenuentry \"y\" --id {ID_INERTE} {{\n"
        );

        match desarmar(&quebrada).unwrap_err() {
            RecusaDoGrub::BlocoSemFechamento { linha } => assert_eq!(linha, 2),
            outra => panic!("esperava recusa por falta de fechamento, veio {outra}"),
        }
    }

    #[test]
    fn bloco_sem_fechamento_nao_leva_o_menuentry_seguinte_junto() {
        // O defeito que a revisao desta etapa achou, e que o teste acima nao
        // pegava: naquele caso nao havia `}` nenhum ate o fim, e a guarda
        // disparava. Num `grub.cfg` de verdade **sempre** ha um `}` adiante —
        // o do proximo `menuentry`. Medido antes da correcao: o arquivo saiu
        // reduzido a uma linha, com o `menuentry --id live-default` removido
        // junto, e o `set default` apontando para uma entrada que sumiu. E
        // esse arquivo iria para o dispositivo.
        let quebrada = format!(
            "set default=\"{ID_INERTE}\"\n\
             menuentry \"ARCA\" --id {ID_DO_ARCA} {{\n\
             \x20 search --set -f /live/vmlinuz\n\
             menuentry \"Clonezilla live\" --id {ID_INERTE} {{\n\
             \x20 linux /live/vmlinuz\n\
             }}\n"
        );

        assert!(
            quebrada.contains("}"),
            "o caso so vale se houver um `}}` adiante, que e o do proximo menuentry"
        );
        match desarmar(&quebrada).unwrap_err() {
            RecusaDoGrub::BlocoSemFechamento { linha } => assert_eq!(linha, 2),
            outra => panic!("esperava recusa por falta de fechamento, veio {outra}"),
        }
    }

    #[test]
    fn um_submenu_tambem_interrompe_a_procura_pelo_fechamento() {
        // O `grub.cfg` do Clonezilla tem um `submenu 'Other modes...' {` com
        // menuentries dentro. Ele abre bloco como um `menuentry`, e ignora-lo
        // deixaria a mesma porta aberta.
        let quebrada = format!(
            "set default=\"{ID_INERTE}\"\n\
             menuentry \"ARCA\" --id {ID_DO_ARCA} {{\n\
             submenu 'Other modes' {{\n\
             \x20 menuentry \"x\" --id {ID_INERTE} {{\n\
             \x20 }}\n\
             }}\n"
        );

        assert!(matches!(
            desarmar(&quebrada).unwrap_err(),
            RecusaDoGrub::BlocoSemFechamento { .. }
        ));
    }

    #[test]
    fn a_linha_em_branco_de_antes_do_bloco_nao_e_removida() {
        // Ela separa o bloco do que veio antes dele, e nao foi o ARCA que a
        // pos: `armar` insere o bloco **seguido** de uma linha em branco.
        // Remove-la colaria duas entradas do Clonezilla uma na outra — e o
        // modulo promete que o que nao for o `set default` nem o bloco sai
        // byte a byte como entrou.
        let bloco = bloco_de_exemplo();
        let colado = format!(
            "set default=\"{ID_INERTE}\"\n\
             menuentry \"antes\" --id antes {{\n\
             }}\n\
             \n\
             {bloco}menuentry \"Clonezilla\" --id {ID_INERTE} {{\n\
             }}\n"
        );

        let desarmada = desarmar(&colado).expect("desarma");
        assert_eq!(desarmada.blocos_removidos, 1);
        assert_eq!(
            desarmada.texto,
            format!(
                "set default=\"{ID_INERTE}\"\n\
                 menuentry \"antes\" --id antes {{\n\
                 }}\n\
                 \n\
                 menuentry \"Clonezilla\" --id {ID_INERTE} {{\n\
                 }}\n"
            ),
            "a linha em branco de antes do bloco foi embora"
        );
    }

    #[test]
    fn remover_o_bloco_nao_pode_deixar_o_default_apontando_para_o_nada() {
        // A pos-condicao, cobrada sobre um caso que a correcao de
        // `achar_bloco` ja impede — e e de proposito: sao duas defesas contra
        // o mesmo estrago, e um `grub.cfg` que boota vale as duas.
        let so_o_arca =
            format!("set default=\"{ID_INERTE}\"\nmenuentry \"ARCA\" --id {ID_DO_ARCA} {{\n}}\n");

        assert_eq!(
            desarmar(&so_o_arca).unwrap_err(),
            RecusaDoGrub::SemMenuentryInerte,
            "tirar o bloco deixaria o grub.cfg sem a entrada que o `set default` nomeia"
        );
    }

    #[test]
    fn grub_cfg_sem_set_default_e_recusado_em_vez_de_adivinhado() {
        let sem = format!("# comentario\nmenuentry \"x\" --id {ID_INERTE} {{\n}}\n");
        assert_eq!(
            desarmar(&sem).unwrap_err(),
            RecusaDoGrub::SemSetDefault,
            "sem `{SET_DEFAULT}` nao ha onde apontar, e inventar onde inserir e adivinhar"
        );
    }

    #[test]
    fn armar_sem_o_menuentry_padrao_e_recusado() {
        let sem = format!("{SET_DEFAULT}=\"0\"\nmenuentry \"x\" --id outro {{\n}}\n");
        assert_eq!(
            armar(&sem, &bloco_de_exemplo()).unwrap_err(),
            RecusaDoGrub::SemMenuentryInerte
        );
    }

    // ───────────────── o que o texto exige ─────────────────

    #[test]
    fn o_id_e_lido_por_token_e_nao_por_pedaco_de_texto() {
        // `--id arca-backup-antigo` contem `arca-backup`. Remove-lo tiraria do
        // menu uma entrada que nao e do ARCA.
        assert_eq!(
            id_do_menuentry("menuentry \"x\" --id arca-backup {"),
            Some(ID_DO_ARCA)
        );
        assert_eq!(
            id_do_menuentry("menuentry \"x\" --id=arca-backup {"),
            Some(ID_DO_ARCA)
        );
        assert_eq!(
            id_do_menuentry("menuentry \"x\" --id arca-backup-antigo {"),
            Some("arca-backup-antigo")
        );
        assert_eq!(id_do_menuentry("  set default=\"x\""), None);
        assert_eq!(id_do_menuentry("menuentry \"sem id\" {"), None);
    }

    #[test]
    fn o_menuentry_parecido_nao_e_removido() {
        // `--id arca-backup-antigo` contem `arca-backup`. Tira-lo seria o
        // desarmar mexendo numa entrada de menu que nao e do ARCA.
        let vizinho = bloco_de_exemplo().replace(ID_DO_ARCA, &format!("{ID_DO_ARCA}-antigo"));
        let alvo = format!("menuentry \"Clonezilla live (VGA 800x600)\" --id {ID_INERTE} {{");
        assert!(INERTE.contains(&alvo), "o menuentry padrao esta no inerte");

        let com_vizinho = INERTE.replace(&alvo, &format!("{vizinho}\n{alvo}"));
        let desarmada = desarmar(&com_vizinho).expect("desarma");

        assert_eq!(desarmada.blocos_removidos, 0);
        assert_eq!(
            desarmada.texto, com_vizinho,
            "mexeu no arquivo sem precisar"
        );
        assert!(
            desarmada
                .texto
                .contains(&format!("--id {ID_DO_ARCA}-antigo"))
        );
    }

    #[test]
    fn desarmar_sem_o_menuentry_padrao_recusa_em_vez_de_apontar_para_o_nada() {
        // Devolver o `set default` para uma entrada que nao existe mandaria o
        // grub procurar o que nao ha. Recusar e melhor: nada e gravado, e o
        // dispositivo continua com o que tinha.
        let sem_o_padrao =
            format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\"\nmenuentry \"x\" --id outro {{\n}}\n");
        assert_eq!(
            desarmar(&sem_o_padrao).unwrap_err(),
            RecusaDoGrub::SemMenuentryInerte
        );
    }

    #[test]
    fn sem_nada_a_mudar_o_menuentry_que_falta_nao_vira_recusa() {
        // Um `grub.cfg` que ja aponta para onde se queria estava assim antes
        // de o ARCA chegar. Recusa-lo seria o ARCA falhar por uma coisa que
        // nao fez — e o dispositivo ja esta inerte.
        let ja_inerte = format!("{SET_DEFAULT}=\"{ID_INERTE}\"\n# so isto\n");
        let desarmada = desarmar(&ja_inerte).expect("nao ha o que fazer");

        assert_eq!(desarmada.texto, ja_inerte);
        assert!(!desarmada.havia_receita());
    }

    #[test]
    fn todas_as_linhas_set_default_sao_devolvidas_e_nao_so_a_primeira() {
        // O `grub` executa o arquivo de cima a baixo, e a ultima atribuicao
        // vence. Trocar so a primeira, num `grub.cfg` que tivesse duas,
        // deixaria a segunda mandando — e o ARCA diria que desarmou um
        // dispositivo que continua armado. Nao ha `grub.cfg` assim nas
        // capturas, e e por isso mesmo que nao se pode contar com isso.
        let com_duas = INERTE.replacen(
            &format!("{SET_DEFAULT}=\"{ID_INERTE}\""),
            &format!("{SET_DEFAULT}=\"{ID_INERTE}\"\n{SET_DEFAULT}=\"{ID_DO_ARCA}\""),
            1,
        );

        let desarmada = desarmar(&com_duas).expect("desarma");
        assert!(desarmada.default_devolvido);
        assert!(
            !desarmada
                .texto
                .contains(&format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\"")),
            "sobrou um `{SET_DEFAULT}` apontando para o ARCA"
        );
        assert_eq!(
            desarmada.texto.matches(SET_DEFAULT).count(),
            2,
            "as duas linhas continuam existindo, so apontando para o lugar certo"
        );
    }

    #[test]
    fn uma_diretiva_parecida_nao_e_confundida_com_set_default() {
        assert!(e_set_default("set default=\"live-default\""));
        assert!(e_set_default("  set default  =  0  "));
        assert!(!e_set_default("set defaultfoo=1"));
        assert!(!e_set_default("set def=1"));
        assert!(!e_set_default("# set default=\"x\""));
    }

    #[test]
    fn o_arquivo_em_crlf_continua_em_crlf() {
        // O `grub.cfg` deste dispositivo e LF, e o `.gitattributes` o mantem
        // assim. Um checkout com outra configuracao, ou um dispositivo
        // preparado noutra maquina, nao pode fazer o desarmar reescrever o
        // arquivo inteiro com outras quebras de linha.
        let armada_crlf = ARMADAS[2].1.replace('\n', "\r\n");
        let inerte_crlf = INERTE.replace('\n', "\r\n");

        let desarmada = desarmar(&armada_crlf).expect("desarma");
        assert_eq!(desarmada.texto, inerte_crlf);
        assert!(!desarmada.texto.contains("\n\n"), "sobrou LF sozinho");
    }

    #[test]
    fn o_resto_do_arquivo_sai_como_entrou() {
        // Tudo que nao e o `set default` nem o bloco do ARCA e territorio do
        // Clonezilla: `timeout`, `efitextmode`, os outros `menuentry`, os
        // comentarios. O desarmar nao pode encostar em nada disso.
        let desarmada = desarmar(ARMADAS[2].1).expect("desarma");

        for linha in ["set timeout=\"30\"", "efitextmode 0", "insmod play"] {
            assert!(desarmada.texto.contains(linha), "sumiu `{linha}`");
        }
        assert_eq!(
            desarmada.texto.matches("menuentry ").count(),
            INERTE.matches("menuentry ").count(),
            "o numero de menuentry mudou"
        );
    }

    #[test]
    fn dois_blocos_do_arca_saem_os_dois() {
        // Nao deveria acontecer — quem arma desarma antes —, mas se
        // acontecesse, deixar um para tras seria deixar o dispositivo com uma
        // receita velha no menu.
        let bloco = bloco_de_exemplo();
        let uma_vez = armar(INERTE, &bloco).expect("arma");
        let duas_vezes = uma_vez.replace(&bloco, &format!("{bloco}\n{bloco}"));

        let desarmada = desarmar(&duas_vezes).expect("desarma");
        assert_eq!(desarmada.blocos_removidos, 2);
        assert!(!desarmada.texto.contains(ID_DO_ARCA));
    }

    #[test]
    fn o_bloco_extraido_e_o_bloco_que_esta_na_captura() {
        let bloco = bloco_do_arca(ARMADAS[2].1).expect("ha bloco");

        assert!(bloco.starts_with("menuentry "));
        assert!(bloco.contains(&format!("--id {ID_DO_ARCA}")));
        assert!(bloco.trim_end().ends_with('}'));
        assert!(ARMADAS[2].1.contains(&bloco), "o bloco saiu da captura");
        assert!(bloco_do_arca(INERTE).is_none(), "o inerte nao tem bloco");
    }

    /// Se o `set default` desta copia aponta para o ARCA — isto e, se ela
    /// estava armada de verdade, e nao so com o bloco no menu.
    fn aponta_para_o_arca(texto: &str) -> bool {
        texto.contains(&format!("{SET_DEFAULT}=\"{ID_DO_ARCA}\""))
    }

    /// As copias que estavam de fato armadas: bloco **e** `set default`.
    fn de_fato_armadas() -> Vec<(&'static str, &'static str)> {
        ARMADAS
            .into_iter()
            .filter(|(_, texto)| aponta_para_o_arca(texto))
            .collect()
    }

    /// Um bloco cru, para os testes que nao precisam de um bloco de verdade.
    ///
    /// **Nao** e a forma que a E7 vai inserir: escolher o menuentry que vai
    /// para o dispositivo e decidir que linha de comando o kernel recebe, e
    /// isso e da E7. Aqui o bloco e dado, e o que se prova e a inversao.
    fn bloco_de_exemplo() -> String {
        format!(
            "menuentry \"ARCA - backup automatico\" --id {ID_DO_ARCA} {{\n  search --set -f /live/vmlinuz\n  $linux_cmd /live/vmlinuz locales=en_US.UTF-8\n  $initrd_cmd /live/initrd.img\n}}\n"
        )
    }
}
