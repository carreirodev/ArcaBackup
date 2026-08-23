//! `arca status` — diagnostico nao destrutivo (§8 do PRD).
//!
//! Diz quatro coisas e nao faz nenhuma: qual dispositivo esta conectado, que
//! imagens ele tem, qual e a entrada de firmware do ARCA e se ha job pendente.
//! Lê o dispositivo, lê o `bcdedit`, e nao escreve em lugar nenhum — nem no
//! `grub.cfg`, nem no firmware, nem no dispositivo.
//!
//! E o comando que se roda **antes** de armar, e o que se roda quando alguma
//! coisa nao esta como se esperava. Por isso ele nomeia o que esta errado em
//! vez de so descrever o que encontrou: uma entrada de firmware apontando para
//! o volume errado e a diferenca entre a maquina bootar no Clonezilla e bootar
//! no Windows com um job armado esperando.

use crate::app::Contexto;
use crate::desfecho::{self, Encontrado};
use crate::dispositivo::{self, Dispositivo};
use crate::erro::Resultado;
use crate::estado::{self, Estado};
use crate::firmware::{self, Alvo, Leitura, Procedencia};
use crate::formato::{linha, tamanho};
use crate::imagens::{self, Pasta};
use crate::portas::{Arquivos, TipoDeMidia, Volume};

use super::list;

/// O alvo que se pergunta ao `bcdedit`: as entradas de boot do firmware, que
/// sao as que apontam para o dispositivo.
const ALVO: &str = "firmware";

/// Se ha job por colher, pelo que o `ARCABOOT` mostra.
///
/// Ate a etapa E4 isto era so "o arquivo existe". A E5 passou a **lê o
/// conteudo**, porque um `estado.json` que existe e nao se lê nao e o mesmo
/// que nao haver job — e a diferenca decide se alguem vai reiniciar achando
/// que nao ha nada esperando.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstadoDoJob {
    Nenhum,

    /// Ha job armado, e o que ele diz. Junto vai o que se encontrou no lugar
    /// onde o desfecho dele apareceria — julgado pelo selo (C-11, §5.5).
    Pendente {
        estado: Estado,
        desfecho: Encontrado,
    },

    /// Ha `estado.json` e o job **ja foi colhido**. Nao ha nada esperando.
    ///
    /// Acrescentado na etapa E8, e e o que fecha o par que a E5 deixou aberto:
    /// depois de um `arca desarmar`, esta secao mostrava "Boot unico: nao
    /// armado" ao lado de um job pendente, e ninguem encerrava o job. Quem
    /// encerra e o `arca resultado`, ao colher; o arquivo continua no
    /// dispositivo porque ele e o unico registro que liga um selo a um nome
    /// (ver [`crate::estado::Situacao`]).
    Colhido { estado: Estado },

    /// O `estado.json` esta la e nao da para entender. **Nao e "nao ha job"**:
    /// o dispositivo pode estar armado, e o que dizia qual job era este se
    /// perdeu.
    ///
    /// Carrega o motivo em texto porque ha duas origens diferentes com o mesmo
    /// significado para quem lê — o arquivo nao se deixou abrir, ou abriu e
    /// nao se deixou entender — e nenhuma das duas pode virar ausencia.
    Ilegivel { motivo: String },

    /// Nao ha caminho para o `estado.json`, e isso nao e o mesmo que nao haver
    /// job: e o ARCA nao ter conseguido perguntar.
    ///
    /// Carrega o motivo porque ha **dois**, e eles pedem reacoes diferentes:
    /// nao existir `ARCABOOT`, e existir sem letra atribuida
    /// (`Erro::VolumeSemLetra`). No segundo o dispositivo esta na mesa e pode
    /// ter job armado — dizer "sem ARCABOOT" ali mandaria alguem procurar um
    /// dispositivo que ja esta conectado.
    SemOndeOlhar { motivo: String },
}

/// Tudo que o `status` colheu, antes de virar texto.
pub struct Diagnostico<'a> {
    pub dispositivo: &'a Dispositivo,
    pub pastas: &'a [Pasta],
    pub firmware: &'a Leitura,
    pub estado_do_job: EstadoDoJob,
}

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz_do_vault = dispositivo.raiz_do_vault()?;
    let pastas = imagens::enumerar(contexto.arquivos, &raiz_do_vault)?;

    let firmware = firmware::ler(&contexto.firmware.enumerar(ALVO)?);

    let estado_do_job = ler_o_job(contexto.arquivos, &dispositivo, &raiz_do_vault);

    contexto.registro.info(format!(
        "status · {} entrada(s) no firmware · entrada do ARCA: {} · boot unico: {} · job: {}",
        firmware.entradas.len(),
        match firmware.entrada_do_arca() {
            Some(achado) => format!("{} ({:?})", achado.descricao, achado.procedencia),
            None => "nenhuma".to_string(),
        },
        if firmware.tem_boot_unico() { "armado" } else { "nao armado" },
        match &estado_do_job {
            EstadoDoJob::Nenhum => "nenhum".to_string(),
            EstadoDoJob::Pendente { estado, desfecho } => format!(
                "{} `{}` · selo {} · desfecho: {desfecho}",
                estado.comando.nome(),
                estado.nome,
                estado.selo
            ),
            EstadoDoJob::Colhido { estado } => format!(
                "{} `{}` · selo {} · ja colhido",
                estado.comando.nome(),
                estado.nome,
                estado.selo
            ),
            EstadoDoJob::Ilegivel { motivo } => format!("estado ilegivel: {motivo}"),
            EstadoDoJob::SemOndeOlhar { motivo } => format!("sem onde olhar: {motivo}"),
        },
    ));

    print!(
        "{}",
        montar(&Diagnostico {
            dispositivo: &dispositivo,
            pastas: &pastas,
            firmware: &firmware,
            estado_do_job,
        })
    );
    Ok(())
}

/// O job pendente, pelo `estado.json`, e o que ha no lugar do desfecho dele.
///
/// Nao ha decisao nenhuma sendo tomada aqui: `arca status` diagnostica. Quem
/// colhe o desfecho, desarma e imprime a §5.4 e a etapa E8. O que este comando
/// entrega e a resposta a "o que ha no dispositivo agora", com o selo ja
/// julgado — e e por isso que ele e o comando que se roda antes de armar.
fn ler_o_job(
    arquivos: &dyn Arquivos,
    dispositivo: &Dispositivo,
    raiz_do_vault: &std::path::Path,
) -> EstadoDoJob {
    // Nao ha **um** motivo para nao haver caminho, ha dois: nao existir
    // `ARCABOOT`, e existir sem letra atribuida (`Erro::VolumeSemLetra`). Sao
    // situacoes diferentes para quem esta olhando — na segunda o dispositivo
    // esta na mesa e pode ter job armado —, e por isso o motivo vai junto em
    // vez de virar uma frase so.
    let caminho = match dispositivo.caminho_do_estado() {
        Ok(caminho) => caminho,
        Err(erro) => {
            return EstadoDoJob::SemOndeOlhar {
                motivo: erro.to_string(),
            };
        }
    };

    // Sem `existe()` antes, e nao por descuido. Um `bool` nao tem como dizer
    // "nao sei", e `Path::exists` transforma qualquer falha de I/O em `false`:
    // perguntar antes de lê seria fazer a pergunta a quem ja confundiu "nao
    // esta la" com "nao consegui olhar". Lê-se, e o erro diz qual dos dois foi
    // — ver [`Erro::e_arquivo_ausente`].
    let estado = match estado::ler(arquivos, &caminho) {
        Ok(estado) => estado,
        Err(erro) if erro.e_arquivo_ausente() => return EstadoDoJob::Nenhum,
        // Tudo o mais e o arquivo estar la sem se deixar entender — recusado
        // pelo leitor, ou nao lido por problema de disco ou permissao. **Nunca**
        // vira "nao ha job": um dispositivo com job armado e estado corrompido
        // continua armado.
        Err(erro) => {
            return EstadoDoJob::Ilegivel {
                motivo: erro.to_string(),
            };
        }
    };

    // Um job colhido nao tem desfecho a procurar: ele ja foi lido e dito. Ir
    // olhar de novo faria o `arca status` reabrir uma pergunta que o
    // `arca resultado` fechou — e, pior, um `arca-fim.txt` que a proxima
    // operacao truncasse apareceria aqui como "o boot nao aconteceu" para um
    // job que aconteceu.
    if estado.situacao == crate::estado::Situacao::Colhido {
        return EstadoDoJob::Colhido { estado };
    }

    let onde = estado::caminho_do_desfecho(raiz_do_vault, estado.comando, &estado.nome);

    // Pelo mesmo caminho: "nao ha desfecho" quer dizer que o boot nao
    // aconteceu (C-12), e "nao consegui lê" nao diz nada sobre o boot.
    // Confundi-las faria um backup bem-sucedido com o arquivo ilegivel sair
    // como boot que nunca ocorreu — o padrao que o ADR-0005 nomeou no firmware.
    //
    // `ler_texto_alheio` porque quem escreveu foi o `echo` de um bash do outro
    // lado do reinicio, e nao o ARCA: um byte solto nao pode fazer o desfecho
    // inteiro sumir.
    let desfecho = match arquivos.ler_texto_alheio(&onde) {
        Ok(texto) => Encontrado::Arquivo(desfecho::julgar(&desfecho::ler(&texto), &estado.selo)),
        Err(erro) if erro.e_arquivo_ausente() => Encontrado::SemArquivo,
        Err(erro) => Encontrado::NaoDeuParaLer {
            motivo: erro.to_string(),
        },
    };

    EstadoDoJob::Pendente { estado, desfecho }
}

/// O diagnostico inteiro, em texto.
/// A linha `Disco alvo`, que nem todo job tem.
///
/// A verificacao armada da E11 nao nomeia disco nenhum — o `ocs-chkimg` opera
/// sobre a imagem —, e o campo vem vazio do `estado.json`. Uma linha em branco
/// ali faria quem lê procurar o que se perdeu; a linha diz o que aconteceu, que
/// e nao haver disco a nomear.
fn disco_alvo(estado: &Estado) -> String {
    match &estado.disco {
        Some(disco) => disco.to_string(),
        None => format!(
            "nenhum · `{}` lê a imagem, e nao um disco",
            estado.comando.nome()
        ),
    }
}

pub fn montar(diagnostico: &Diagnostico) -> String {
    let mut saida = String::new();

    saida.push_str(&secao_do_dispositivo(diagnostico.dispositivo));
    saida.push('\n');

    // Sem segunda formatacao das imagens: a saida do §5.4 e criterio de aceite
    // da E1, e duas versoes dela divergiriam na primeira mudanca.
    saida.push_str(&list::montar(
        diagnostico.pastas,
        diagnostico.dispositivo.vault.livre_bytes,
    ));
    saida.push('\n');

    saida.push_str(&secao_do_firmware(
        diagnostico.firmware,
        diagnostico.dispositivo,
    ));
    saida.push('\n');

    saida.push_str(&secao_do_job(
        diagnostico.firmware,
        &diagnostico.estado_do_job,
    ));
    saida
}

/// As duas particoes, e o aviso de C-6 quando couber.
fn secao_do_dispositivo(dispositivo: &Dispositivo) -> String {
    let mut saida = String::from("Dispositivo ARCA\n");

    saida.push_str(&linha(
        dispositivo::ARCAVAULT,
        &descrever(&dispositivo.vault),
    ));
    saida.push_str(&linha(
        dispositivo::ARCABOOT,
        &match &dispositivo.boot {
            Some(boot) => descrever(boot),
            // Sem `ARCABOOT` da para listar imagens, e nao da para armar: a
            // receita e o estado do job moram nele (§4.1).
            None => "ausente — sem ele nao ha onde gravar receita nem estado".to_string(),
        },
    ));

    if dispositivo
        .boot
        .as_ref()
        .is_some_and(|boot| boot.tipo_de_midia == TipoDeMidia::Removivel)
    {
        saida.push_str(concat!(
            "\n  AVISO (C-6): o Windows classifica o ARCABOOT como midia removivel.\n",
            "  O bcdedit recusa esse alvo em silencio — responde \"exito\" e mantem o\n",
            "  valor antigo. Um dispositivo assim boota por F12, nunca por entrada de\n",
            "  firmware.\n"
        ));
    }

    saida
}

fn descrever(volume: &Volume) -> String {
    let letra = match volume.letra {
        Some(letra) => format!("{letra}:"),
        // Sem letra o volume existe e nao tem caminho. `arca list` recusa antes
        // de chegar aqui; o status tem de dizer por que.
        None => "sem letra".to_string(),
    };

    format!(
        "{letra} · {} · {}",
        volume.sistema_de_arquivos,
        tamanho(volume.total_bytes)
    )
}

/// A entrada do ARCA no firmware, e para onde ela aponta de verdade.
fn secao_do_firmware(leitura: &Leitura, dispositivo: &Dispositivo) -> String {
    let mut saida = String::from("Entrada de firmware\n");

    let Some(achado) = leitura.entrada_do_arca() else {
        saida.push_str(&linha(&format!("Entrada {}", firmware::ARCA), "nenhuma"));
        saida.push_str(&linha(&format!("Entrada {}", firmware::LEGADA), "nenhuma"));
        // Esta frase dizia "a etapa E7 cria a entrada", e a E7 chegou fazendo
        // o contrario: ela **recusa** em vez de criar, porque criar uma
        // entrada de firmware do zero e codigo sem original (C-4). Quem cria e
        // o `arca prepare` da E10. Uma linha de diagnostico que promete uma
        // saida que nao existe e pior do que uma que so descreve.
        saida.push_str(&format!(
            "  Nao ha por onde bootar no dispositivo sem passar pelo F12, e `arca backup`\n\
             \x20 recusa sem uma delas — armar migra a entrada que existe, e nao cria\n\
             \x20 entrada de boot (C-4). Quem prepara um dispositivo do zero e o\n\
             \x20 `arca prepare`, que a etapa E10 entrega. Ha {} entrada(s) de boot no\n\
             \x20 firmware desta maquina.\n",
            leitura.entradas.len()
        ));
        return saida;
    };

    saida.push_str(&linha(
        "Descricao",
        &match achado.procedencia {
            Procedencia::Propria => achado.descricao.to_string(),
            // C-4: a legada nao e um problema, e a entrada certa com o nome
            // antigo. Quem a renomeia e a E7, ao armar.
            Procedencia::Legada => format!("{} · legada, a migrar (C-4)", achado.descricao),
        },
    ));
    saida.push_str(&linha("Identificador", &achado.entrada.identificador));

    saida.push_str(&linha(
        "Aponta para",
        &match &achado.entrada.alvo {
            Some(alvo) => format!(
                "{} · {}",
                alvo.como_bcdedit_escreve(),
                confere_com_o_arcaboot(alvo, dispositivo)
            ),
            None => "nada — a entrada existe e nao diz para onde ir".to_string(),
        },
    ));

    saida.push_str(&linha(
        "Carrega",
        achado
            .entrada
            .caminho
            .as_deref()
            .unwrap_or("nada — a entrada nao diz que .efi carregar"),
    ));

    saida.push_str(&secao_da_ordem_de_boot(leitura, dispositivo));

    saida
}

/// Se esta entrada leva ao `ARCABOOT` que esta na mesa.
///
/// A versao de `sim ou nao` do [`confere_com_o_arcaboot`], e ela responde
/// **nao** para tudo que nao dá para conferir. Uma entrada por caminho de
/// dispositivo — o `{bootmgr}` e uma — nao tem letra, e supor que ela alcanca o
/// dispositivo faria o aviso abaixo disparar sempre, que e o mesmo que nao
/// avisar.
pub fn alcanca_o_arcaboot(entrada: &firmware::EntradaDeFirmware, dispositivo: &Dispositivo) -> bool {
    let (Some(alvo), Some(boot)) = (&entrada.alvo, &dispositivo.boot) else {
        return false;
    };
    match (alvo.letra(), boot.letra) {
        (Some(apontada), Some(atual)) => apontada.eq_ignore_ascii_case(&atual),
        _ => false,
    }
}

/// Onde o dispositivo esta na ordem permanente, e por quantas entradas.
///
/// # Por que isto e uma funcao publica, e nao um `if` repetido
///
/// A E9 precisa da mesma pergunta que a E8 ja fazia: antes de reiniciar para
/// uma **restauracao**, o aviso de que a maquina boota no dispositivo sozinha
/// deixa de ser chateacao e passa a ser "religar apaga o disco de novo". Duas
/// versoes da mesma regra divergem na primeira mudanca — foi por isso que o
/// `arca resultado` reusa `list::montar` em vez de formatar de novo, e e por
/// isso que a regra que decide o aviso mora aqui, num lugar so.
///
/// O que ela **nao** faz e o texto: `arca status` diz uma coisa e
/// `arca restore` diz outra, porque os dois estao em momentos diferentes. O
/// que se compartilha e o julgamento, e nao a frase.
pub struct LugarNaOrdem {
    /// A posicao da **primeira** entrada da ordem que leva ao `ARCABOOT` desta
    /// mesa. `None` quando nenhuma leva.
    pub posicao: Option<usize>,

    /// Quantas entradas da ordem levam a ele. Desde o marco de 22/08 sao
    /// **duas** nesta maquina — a `{f4057bd0}` do ARCA e a `{687478f2}`
    /// `UEFI OS` que o firmware criou —, e e por isso que a pergunta e sobre o
    /// alvo e nunca sobre o nome.
    pub quantas: usize,
}

impl LugarNaOrdem {
    /// Se todo reinicio boota no dispositivo. Falso tambem quando ele nao esta
    /// na ordem — que e a resposta certa e a tranquilizadora ao mesmo tempo.
    pub fn em_primeiro(&self) -> bool {
        self.posicao == Some(0)
    }
}

/// Percorre a ordem permanente resolvendo cada identificador na entrada que ele
/// nomeia, e pergunta se o alvo e o `ARCABOOT` que esta na mesa.
///
/// **Quem chama tem de ter conferido `viu_o_gerenciador` antes.** Uma leitura
/// que nao se deixou entender produz ordem vazia, e ordem vazia sai daqui como
/// "o dispositivo esta fora da ordem" — a resposta tranquilizadora. Ver a
/// guarda em [`secao_da_ordem_de_boot`].
pub fn lugar_do_dispositivo(leitura: &Leitura, dispositivo: &Dispositivo) -> LugarNaOrdem {
    let leva_ao_dispositivo = |identificador: &String| {
        leitura
            .entradas
            .iter()
            .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador))
            .is_some_and(|entrada| alcanca_o_arcaboot(entrada, dispositivo))
    };

    LugarNaOrdem {
        posicao: leitura.ordem_permanente.iter().position(leva_ao_dispositivo),
        quantas: leitura
            .ordem_permanente
            .iter()
            .filter(|id| leva_ao_dispositivo(id))
            .count(),
    }
}

/// Por onde a maquina boota quando ninguem pede nada, e o que isso muda no
/// proximo reinicio.
///
/// **Le, e nunca escreve.** C-5 proibe o ARCA de mexer na ordem permanente, e
/// a proibicao nao tem clausula para o caso de ele achar que esta arrumando. O
/// que o ARCA pode fazer e dizer o que leu.
///
/// A linha existe desde o
/// [ADR-0009](../../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md),
/// e o que a motivou foi uma medicao: **o ciclo de boot pelo dispositivo poe a
/// entrada de volta na ordem**, e depois de um backup ela costuma estar em
/// primeiro. O dado ja vinha sendo lido em todo comando — o `{fwbootmgr}` sai
/// inteiro do `bcdedit` — e nao estava sendo dito a ninguem.
///
/// # A pergunta e sobre o dispositivo, e nao sobre a entrada do ARCA
///
/// A primeira versao desta funcao procurava **a entrada chamada `ARCA`** na
/// ordem, e a revisao pegou o furo com a captura desta propria maquina: o
/// `bcdedit-enum-firmware-2026-08-22-pos-marco.txt` tem **duas** entradas em
/// `partition=R:` — a `{f4057bd0}` do ARCA e a `{687478f2}` `UEFI OS` que o
/// firmware criou —, e foi pela segunda que a maquina bootou (`Boot0001* UEFI
/// OS`, no `nvram-live-2026-08-22.txt`). Com a `{687478f2}` em primeiro e a do
/// ARCA em segundo, aquela versao diria `2o de 3 · o Windows vem antes` e
/// engoliria o aviso, enquanto todo reinicio com o SSD conectado continuaria
/// bootando no dispositivo.
///
/// O que decide o boot e **para onde a entrada aponta**, e nao como ela se
/// chama — que e a mesma licao de C-4 e de C-6, aplicada a ordem.
fn secao_da_ordem_de_boot(leitura: &Leitura, dispositivo: &Dispositivo) -> String {
    // Sem o bloco do `{fwbootmgr}`, `ordem_permanente` vem vazia — e vazia e
    // indistinguivel de "o dispositivo esta fora da ordem", que e a resposta
    // tranquilizadora. `firmware::ler` nunca falha, e e para exatamente isto
    // que `viu_o_gerenciador` existe: "nao entendi a resposta" nao pode virar
    // uma afirmacao de seguranca. Mesma guarda que `armar` e `desarme` fazem.
    if !leitura.viu_o_gerenciador {
        return linha(
            "Ordem de boot",
            "nao foi possivel ler o {fwbootmgr} — nada a afirmar sobre ela",
        );
    }

    let total = leitura.ordem_permanente.len();
    let LugarNaOrdem { posicao, quantas } = lugar_do_dispositivo(leitura, dispositivo);

    let Some(posicao) = posicao else {
        return linha(
            "Ordem de boot",
            &format!("{total} entrada(s), nenhuma para o dispositivo · so o boot unico leva a ele"),
        );
    };

    // O que vem antes decide. Se ha alguma coisa a frente da primeira entrada
    // do dispositivo, e ela que boota — e o aviso nao se aplica.
    if posicao > 0 {
        let antes = &leitura.ordem_permanente[0];
        let nome = leitura
            .entradas
            .iter()
            .find(|entrada| entrada.identificador.eq_ignore_ascii_case(antes))
            .and_then(|entrada| entrada.descricao.as_deref())
            .unwrap_or(antes.as_str());
        return linha(
            "Ordem de boot",
            &format!(
                "dispositivo em {}o de {total} · `{nome}` vem antes",
                posicao + 1
            ),
        );
    }

    let quais = if quantas > 1 {
        format!(" · {quantas} entradas levam a ele")
    } else {
        String::new()
    };
    let mut saida = linha(
        "Ordem de boot",
        &format!("dispositivo em 1o de {total} · todo reinicio boota nele{quais}"),
    );
    saida.push_str(concat!(
        "\n  Enquanto o SSD estiver conectado, a maquina boota nele sem boot unico\n",
        "  nenhum. Inerte, ele para no menu do Clonezilla e espera alguem;\n",
        "  armado, a receita roda. O ARCA nao pos a entrada ai e nao a tira —\n",
        "  mexer na ordem permanente e o que C-5 proibe. Quem a pos foi o proprio\n",
        "  ciclo de boot pelo dispositivo (ADR-0009). Remover o SSD antes de\n",
        "  religar resolve, e e o que o aviso de C-9 ja pedia.\n"
    ));
    saida
}

/// Se a entrada de firmware aponta para o `ARCABOOT` que esta na mesa.
///
/// Esta e a pergunta que o `status` existe para responder. A entrada guarda uma
/// letra, e letra muda de uma conexao para outra: uma entrada armada apontando
/// para a letra de ontem manda a maquina bootar em outra coisa — ou em nada.
fn confere_com_o_arcaboot(alvo: &Alvo, dispositivo: &Dispositivo) -> String {
    let Some(boot) = &dispositivo.boot else {
        return "sem ARCABOOT conectado para conferir".to_string();
    };

    match (alvo.letra(), boot.letra) {
        (Some(apontada), Some(atual)) if apontada.eq_ignore_ascii_case(&atual) => {
            "o ARCABOOT deste dispositivo".to_string()
        }
        (Some(_), Some(atual)) => format!("NAO e o ARCABOOT, que esta em {atual}:"),
        // Uma entrada por caminho de dispositivo (`\Device\HarddiskVolume1`)
        // nao da para conferir por letra, e inventar uma correspondencia aqui
        // seria pior do que admitir que nao se sabe.
        _ => "nao da para conferir por letra".to_string(),
    }
}

/// Se ha job por colher, pelos dois sinais independentes que existem: a marca
/// de boot unico no firmware e o `estado.json` do `ARCABOOT`.
///
/// # Os dois podem discordar, e discordar nao e contradicao
///
/// Um `arca desarmar` limpa a marca de boot unico e **nao toca no
/// `estado.json`** — desarmar nao consulta estado nenhum (C-1) e nao escreve
/// nele. Depois dele esta secao mostra "Boot unico: nao armado" ao lado de um
/// job pendente, e isso e exatamente o que aconteceu: o dispositivo esta
/// inerte, e o job continua registrado por colher. Quem encerra o job e o
/// `arca resultado`, ao colher o desfecho.
///
/// # O titulo varia com o estado, e a revisao explicou por que
///
/// Ele era `Job pendente` fixo. Com a linha nova da E8, um job **colhido**
/// saia sob esse titulo — "Job pendente / Estado no ARCABOOT: ja colhido,
/// nada esperando" —, que e uma versao menor exatamente da contradicao que a
/// E8 existia para fechar. Uma peca nova encaixada numa peca antiga que
/// ninguem releu ao encaixar, pela quarta vez neste projeto.
fn secao_do_job(leitura: &Leitura, estado: &EstadoDoJob) -> String {
    let titulo = match estado {
        EstadoDoJob::Pendente { .. } => "Job pendente",
        EstadoDoJob::Colhido { .. } => "Ultimo job, ja colhido",
        // Sem estado legivel, o titulo nao pode afirmar nem uma coisa nem
        // outra: o que se sabe e que se foi olhar.
        EstadoDoJob::Nenhum | EstadoDoJob::Ilegivel { .. } | EstadoDoJob::SemOndeOlhar { .. } => {
            "Job"
        }
    };
    let mut saida = format!("{titulo}\n");

    saida.push_str(&linha(
        "Boot unico",
        &if leitura.tem_boot_unico() {
            format!("ARMADO para {}", leitura.boot_unico.join(", "))
        } else {
            "nao armado".to_string()
        },
    ));

    saida.push_str(&linha(
        "Estado no ARCABOOT",
        &match estado {
            EstadoDoJob::Nenhum => "nenhum".to_string(),
            EstadoDoJob::Pendente { estado, .. } => {
                format!("{} `{}` · POR COLHER", estado.comando.nome(), estado.nome)
            }
            EstadoDoJob::Colhido { estado } => format!(
                "{} `{}` · ja colhido, nada esperando",
                estado.comando.nome(),
                estado.nome
            ),
            EstadoDoJob::Ilegivel { .. } => "presente e ILEGIVEL".to_string(),
            // O motivo vai na linha, e nao uma frase fixa: com o `ARCABOOT`
            // sem letra, dizer "sem ARCABOOT" mandaria alguem procurar um
            // dispositivo que ja esta conectado.
            EstadoDoJob::SemOndeOlhar { motivo } => format!("nao da para olhar — {motivo}"),
        },
    ));

    match estado {
        EstadoDoJob::Pendente { estado, desfecho } => {
            // O selo aparece inteiro: e ele que a mensagem de job fantasma vai
            // nomear, e sem os dois lados a vista ninguem confere nada.
            saida.push_str(&linha("Selo", estado.selo.como_texto()));
            saida.push_str(&linha("Disco alvo", &disco_alvo(estado)));
            saida.push_str(&linha(
                "Armado em",
                &format!("{} · informativo, nunca comparado", estado.armado_em),
            ));
            saida.push_str(&linha(
                "Pasta do desfecho",
                &desfecho::pasta_do_job(estado.comando, &estado.nome),
            ));
            saida.push_str(&linha("Desfecho", &desfecho.to_string()));
        }
        EstadoDoJob::Colhido { estado } => {
            saida.push_str(&linha("Selo", estado.selo.como_texto()));
            saida.push_str(&linha("Disco alvo", &disco_alvo(estado)));
            saida.push_str(&linha(
                "Armado em",
                &format!("{} · informativo, nunca comparado", estado.armado_em),
            ));
            saida.push_str(concat!(
                "\n  Este job ja foi colhido: o `arca resultado` leu o desfecho dele e disse\n",
                "  o que era. O `estado.json` fica no dispositivo de proposito — e o unico\n",
                "  registro que liga este selo a este nome, e o ARCA nao apaga nada (B-10).\n"
            ));
        }
        EstadoDoJob::Ilegivel { motivo } => {
            saida.push_str(&format!(
                "\n  O `estado.json` esta no dispositivo e nao da para entender:\n\
                 \x20 {motivo}\n\
                 \x20 Isto nao e o mesmo que nao haver job. Se o boot unico acima estiver\n\
                 \x20 armado, ha uma receita esperando e nao se sabe qual — rode\n\
                 \x20 `arca desarmar` antes de reiniciar.\n"
            ));
        }
        EstadoDoJob::Nenhum | EstadoDoJob::SemOndeOlhar { .. } => {}
    }

    saida
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{
        ArquivosEmMemoria, ArquivosQueRecusam, RelogioParado, momento, volume,
    };
    use crate::estado::{MomentoDoArmar, Situacao};
    use crate::imagens::{Especie, Veredito};
    use crate::nome::Nome;
    use crate::receita::{Disco, Operacao, Selo};

    const PT: &str = include_str!("../../recursos/capturas/bcdedit-enum-firmware-pt.txt");
    const LEGADO: &str =
        include_str!("../../recursos/capturas/bcdedit-enum-firmware-legado-pt.txt");
    /// O firmware desta maquina **depois** do primeiro backup do ARCA, com a
    /// entrada de volta na ordem permanente e em primeiro (ADR-0009).
    const POS_MARCO: &str =
        include_str!("../../recursos/capturas/bcdedit-enum-firmware-2026-08-22-pos-marco.txt");

    const DO_JOB: &str = "a3f1c9e07b2d4856";
    const DE_OUTRO: &str = "7e02b4d1af963c85";
    const ESTADO: &str = r"R:\arca\estado.json";

    fn estado_gravado() -> Estado {
        Estado {
            selo: Selo::novo(DO_JOB).unwrap(),
            comando: Operacao::Backup,
            nome: Nome::novo("2026-08-22_Apps").unwrap(),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            armado_em: MomentoDoArmar::agora(&RelogioParado::em("2026-08-22T18:14:03")),
            situacao: Situacao::Armado,
        }
    }

    fn job_pendente(desfecho: Encontrado) -> EstadoDoJob {
        EstadoDoJob::Pendente {
            estado: estado_gravado(),
            desfecho,
        }
    }

    /// O diagnostico montado com um estado de job qualquer, e o resto fixo.
    fn com_estado(estado_do_job: EstadoDoJob) -> String {
        montar(&Diagnostico {
            dispositivo: &dispositivo_conectado(),
            pastas: &uma_imagem(),
            firmware: &firmware::ler(PT),
            estado_do_job,
        })
    }

    fn dispositivo_conectado() -> Dispositivo {
        Dispositivo {
            vault: volume(
                dispositivo::ARCAVAULT,
                'E',
                254_000_000_000,
                196_400_000_000,
            ),
            boot: Some(Volume {
                sistema_de_arquivos: "FAT32".to_string(),
                ..volume(dispositivo::ARCABOOT, 'R', 1_700_000_000, 1_070_000_000)
            }),
        }
    }

    fn uma_imagem() -> Vec<Pasta> {
        vec![Pasta {
            nome: "2026-08-21_WindowsCompleto".to_string(),
            tamanho_bytes: 38_823_623_035,
            modificado_em: Some(momento("2026-08-21T12:56:31")),
            especie: Especie::Imagem {
                veredito: Some(Veredito::Aprovada),
            },
        }]
    }

    fn montar_com(dispositivo: &Dispositivo, texto_do_bcdedit: &str) -> String {
        let leitura = firmware::ler(texto_do_bcdedit);
        montar(&Diagnostico {
            dispositivo,
            pastas: &uma_imagem(),
            firmware: &leitura,
            estado_do_job: EstadoDoJob::Nenhum,
        })
    }

    #[test]
    fn o_status_responde_as_quatro_perguntas() {
        // O criterio de aceite da etapa, em texto: dispositivo, imagens,
        // entrada de firmware, job pendente.
        let saida = montar_com(&dispositivo_conectado(), PT);

        assert!(saida.contains("Dispositivo ARCA"), "faltou o dispositivo");
        assert!(
            saida.contains(&linha(dispositivo::ARCABOOT, "R: · FAT32 · 1,6 GB")),
            "faltou o ARCABOOT:\n{saida}"
        );
        assert!(
            saida.contains("2026-08-21_WindowsCompleto"),
            "faltaram as imagens"
        );
        assert!(
            saida.contains("{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"),
            "faltou a entrada de firmware"
        );
        // Sem `estado.json`, o titulo e `Job` — nao `Job pendente`, que
        // afirmaria haver um, nem `Ultimo job`, que afirmaria ter havido.
        assert!(saida.contains("Boot unico"), "faltou o job:\n{saida}");
        assert!(
            !saida.contains("Job pendente"),
            "disse que ha job pendente sem estado nenhum:\n{saida}"
        );
    }

    #[test]
    fn a_entrada_que_aponta_para_o_arcaboot_e_reconhecida() {
        let saida = montar_com(&dispositivo_conectado(), PT);
        assert!(
            saida.contains(&linha(
                "Aponta para",
                "partition=R: · o ARCABOOT deste dispositivo"
            )),
            "{saida}"
        );
    }

    #[test]
    fn a_entrada_que_aponta_para_outra_letra_e_denunciada() {
        // Letra muda de uma conexao para outra. Uma entrada armada apontando
        // para a letra de ontem manda a maquina bootar em outra coisa, e o
        // status existe para dizer isso antes de alguem armar.
        let dispositivo = Dispositivo {
            boot: Some(volume(
                dispositivo::ARCABOOT,
                'S',
                1_700_000_000,
                1_070_000_000,
            )),
            ..dispositivo_conectado()
        };

        let saida = montar_com(&dispositivo, PT);
        assert!(
            saida.contains("NAO e o ARCABOOT, que esta em S:"),
            "{saida}"
        );
    }

    #[test]
    fn a_entrada_legada_aparece_como_a_migrar() {
        // C-4: nao e um problema a resolver a mao, e a entrada certa com o nome
        // antigo. Quem a renomeia e a E7.
        let saida = montar_com(&dispositivo_conectado(), LEGADO);

        assert!(
            saida.contains(&linha("Descricao", "Clonezilla · legada, a migrar (C-4)")),
            "{saida}"
        );
        assert!(saida.contains("{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"));
    }

    #[test]
    fn sem_entrada_de_firmware_o_status_diz_o_que_falta() {
        let saida = montar_com(&dispositivo_conectado(), "");

        assert!(saida.contains(&linha("Entrada ARCA", "nenhuma")), "{saida}");
        assert!(
            saida.contains(&linha("Entrada Clonezilla", "nenhuma")),
            "{saida}"
        );
        assert!(saida.contains("F12"), "faltou dizer como bootar sem ela");
    }

    #[test]
    fn o_boot_unico_armado_aparece_em_maiuscula() {
        // Job armado e a diferenca entre a maquina reiniciar no Windows e
        // reiniciar no Clonezilla com uma receita esperando. Nao pode passar
        // despercebido no meio de uma listagem.
        let texto = concat!(
            "\r\nGerenciador de Inicialização de Firmware\r\n",
            "----------------------------------------\r\n",
            "identificador           {fwbootmgr}\r\n",
            "displayorder            {bootmgr}\r\n",
            "bootsequence            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n"
        );

        let saida = montar_com(&dispositivo_conectado(), texto);
        assert!(
            saida.contains(&linha(
                "Boot unico",
                "ARMADO para {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"
            )),
            "{saida}"
        );
    }

    #[test]
    fn sem_boot_unico_o_firmware_esta_inerte() {
        let saida = montar_com(&dispositivo_conectado(), PT);
        assert!(
            saida.contains(&linha("Boot unico", "nao armado")),
            "{saida}"
        );
    }

    #[test]
    fn sem_entrada_do_dispositivo_na_ordem_so_o_boot_unico_leva_a_ele() {
        // A configuracao desta maquina **antes** do marco: `displayorder` com
        // so o `{bootmgr}`. E o caso em que nao ha nada a avisar.
        let saida = montar_com(&dispositivo_conectado(), PT);
        assert!(
            saida.contains(&linha(
                "Ordem de boot",
                "1 entrada(s), nenhuma para o dispositivo · so o boot unico leva a ele"
            )),
            "{saida}"
        );
        assert!(
            !saida.contains("todo reinicio boota nele"),
            "avisou de um perigo que nao ha:\n{saida}"
        );
    }

    #[test]
    fn o_dispositivo_em_primeiro_na_ordem_avisa_que_todo_reinicio_boota_nele() {
        // **O caso que motivou a linha, e ele nao e construido: e a captura do
        // firmware desta maquina depois do primeiro backup** (ADR-0009). Um
        // caso montado a mao aqui provaria que sei montar `displayorder`.
        //
        // Ela tras **duas** entradas em `partition=R:` — a `{f4057bd0}` do
        // ARCA e a `{687478f2}` `UEFI OS` que o firmware criou —, e a linha
        // diz isso: as duas levam ao dispositivo.
        let saida = montar_com(&dispositivo_conectado(), POS_MARCO);
        assert!(
            saida.contains(&linha(
                "Ordem de boot",
                "dispositivo em 1o de 3 · todo reinicio boota nele · 2 entradas levam a ele"
            )),
            "{saida}"
        );
        // O aviso diz as duas metades: o que acontece inerte e o que acontece
        // armado. Sem a segunda, ele parece um inconveniente.
        assert!(saida.contains("menu do Clonezilla"), "{saida}");
        assert!(saida.contains("a receita roda"), "{saida}");
        // E diz que o ARCA nao o causou nem o conserta — senao a leitura
        // natural e que ele deveria consertar, que e o que C-5 proibe.
        assert!(saida.contains("C-5"), "{saida}");
    }

    #[test]
    fn o_dispositivo_atras_do_windows_na_ordem_nao_dispara_o_aviso() {
        // A ordem que o `efibootmgr` mediu **durante** o boot do marco:
        // `BootOrder: 0000,0001`, o Windows a frente e o dispositivo atras.
        // Bootar dali exige boot unico — e e o que prova P-18 —, entao nao ha
        // perigo a anunciar.
        //
        // As **duas** entradas do dispositivo vao para tras, e nao so a do
        // ARCA: deixar a `{687478f2}` na frente seria montar o caso facil, e o
        // teste passaria pelo motivo errado.
        let trocado = POS_MARCO.replacen(
            concat!(
                "displayorder            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n",
                "                        {bootmgr}\r\n",
                "                        {687478f2-9e87-11f1-8a47-806e6f6e6963}"
            ),
            concat!(
                "displayorder            {bootmgr}\r\n",
                "                        {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n",
                "                        {687478f2-9e87-11f1-8a47-806e6f6e6963}"
            ),
            1,
        );
        assert_ne!(
            trocado, POS_MARCO,
            "a troca nao pegou; a captura mudou de forma"
        );

        let saida = montar_com(&dispositivo_conectado(), &trocado);
        assert!(
            saida.contains(&linha(
                "Ordem de boot",
                "dispositivo em 2o de 3 · `Windows Boot Manager` vem antes"
            )),
            "{saida}"
        );
        assert!(
            !saida.contains("todo reinicio boota nele"),
            "avisou de um perigo que nao ha:\n{saida}"
        );
    }

    #[test]
    fn a_entrada_que_o_firmware_criou_sozinho_a_frente_tambem_dispara_o_aviso() {
        // **O furo que a revisao pegou, fixado.** A primeira versao desta
        // secao procurava a entrada chamada `ARCA` na ordem; a captura desta
        // maquina tem uma **segunda** entrada para o mesmo `partition=R:`, a
        // `{687478f2}` `UEFI OS`, criada pelo firmware — e e por ela que o
        // `nvram-live-2026-08-22.txt` mostra a maquina tendo bootado.
        //
        // Com a `{687478f2}` em primeiro e a do ARCA depois do Windows, aquela
        // versao diria "o Windows vem antes" e engoliria o aviso, enquanto todo
        // reinicio com o SSD conectado continuaria bootando no dispositivo. O
        // que decide o boot e para onde a entrada aponta, e nao como se chama.
        let so_a_do_firmware = POS_MARCO.replacen(
            concat!(
                "displayorder            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n",
                "                        {bootmgr}\r\n",
                "                        {687478f2-9e87-11f1-8a47-806e6f6e6963}"
            ),
            concat!(
                "displayorder            {687478f2-9e87-11f1-8a47-806e6f6e6963}\r\n",
                "                        {bootmgr}\r\n",
                "                        {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"
            ),
            1,
        );
        assert_ne!(
            so_a_do_firmware, POS_MARCO,
            "a troca nao pegou; a captura mudou de forma"
        );

        let saida = montar_com(&dispositivo_conectado(), &so_a_do_firmware);
        assert!(
            saida.contains("dispositivo em 1o de 3 · todo reinicio boota nele"),
            "a entrada que o firmware criou esta em primeiro e o aviso nao saiu:\n{saida}"
        );
    }

    #[test]
    fn o_bcdedit_que_nao_se_deixou_ler_nao_vira_afirmacao_de_seguranca() {
        // `firmware::ler` nunca falha: texto que ele nao entende vira leitura
        // vazia, e `ordem_permanente` vazia e **indistinguivel** de "o
        // dispositivo esta fora da ordem" — que e a resposta tranquilizadora.
        //
        // O caso dificil e o que **quase** se deixa ler: os blocos das entradas
        // saem certos, e so o do `{fwbootmgr}` falta. Ali `entrada_do_arca()`
        // responde normalmente, e so `viu_o_gerenciador` separa uma coisa da
        // outra. E o mesmo motivo pelo qual `armar` e `desarme` guardam nessa
        // flag, e esta secao nao guardava.
        let sem_gerenciador = POS_MARCO.replacen("{fwbootmgr}", "{outra-coisa}", 1);
        assert_ne!(
            sem_gerenciador, POS_MARCO,
            "a troca nao pegou; a captura mudou de forma"
        );

        let leitura = firmware::ler(&sem_gerenciador);
        assert!(
            !leitura.viu_o_gerenciador,
            "o caso construido nao e o que se queria: o gerenciador ainda foi visto"
        );
        assert!(
            leitura.entrada_do_arca().is_some(),
            "o caso construido nao e o dificil: a entrada do ARCA tambem sumiu"
        );

        let saida = montar_com(&dispositivo_conectado(), &sem_gerenciador);
        assert!(
            saida.contains(&linha(
                "Ordem de boot",
                "nao foi possivel ler o {fwbootmgr} — nada a afirmar sobre ela"
            )),
            "{saida}"
        );
        assert!(
            !saida.contains("so o boot unico leva a ele"),
            "'nao entendi a resposta' virou uma afirmacao de seguranca:\n{saida}"
        );
    }

    #[test]
    fn o_estado_do_job_tem_uma_linha_para_cada_caso() {
        assert!(com_estado(EstadoDoJob::Nenhum).contains(&linha("Estado no ARCABOOT", "nenhum")));
        assert!(
            com_estado(EstadoDoJob::SemOndeOlhar {
                motivo: "o dispositivo conectado nao tem a particao ARCABOOT".to_string()
            })
            .contains("nao da para olhar — o dispositivo conectado nao tem a particao ARCABOOT")
        );
        assert!(
            com_estado(EstadoDoJob::Ilegivel {
                motivo: "o arquivo termina no meio".to_string()
            })
            .contains("presente e ILEGIVEL")
        );
        assert!(
            com_estado(job_pendente(Encontrado::SemArquivo)).contains(&linha(
                "Estado no ARCABOOT",
                "backup `2026-08-22_Apps` · POR COLHER"
            ))
        );
        // A E8 acrescentou a sexta linha, e ela e a que fecha o par que a E5
        // deixou aberto: um job colhido nao e um job esperando.
        assert!(
            com_estado(EstadoDoJob::Colhido {
                estado: Estado {
                    situacao: Situacao::Colhido,
                    ..estado_gravado()
                }
            })
            .contains(&linha(
                "Estado no ARCABOOT",
                "backup `2026-08-22_Apps` · ja colhido, nada esperando"
            ))
        );
    }

    #[test]
    fn um_job_colhido_nao_aparece_como_pendente() {
        // A contradicao que a E5 nomeou e nao fechou: depois de desarmar, o
        // status mostrava "Boot unico: nao armado" ao lado de um job pendente.
        // Colhido o job, as duas linhas passam a dizer a mesma coisa.
        let saida = com_estado(EstadoDoJob::Colhido {
            estado: Estado {
                situacao: Situacao::Colhido,
                ..estado_gravado()
            },
        });

        assert!(saida.contains("ja colhido"), "{saida}");
        assert!(saida.contains(&linha("Selo", DO_JOB)), "{saida}");
        assert!(
            !saida.contains("Desfecho"),
            "o status foi procurar desfecho de um job ja colhido:\n{saida}"
        );
    }

    #[test]
    fn o_status_nao_procura_desfecho_de_um_job_ja_colhido() {
        // Ir olhar de novo reabriria uma pergunta que o `arca resultado`
        // fechou — e um `arca-fim.txt` truncado pela operacao seguinte
        // apareceria aqui como "o boot nao aconteceu" para um job que
        // aconteceu.
        let estado = Estado {
            situacao: Situacao::Colhido,
            ..estado_gravado()
        };
        let arquivos = ArquivosEmMemoria::novo()
            .com(ESTADO, &estado.como_json().unwrap())
            .com(
                r"E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt",
                "lixo que nao e desfecho",
            );

        let dispositivo =
            crate::dispositivo::encontrar(&crate::duplos::DiscosDeMentira::com_dispositivo())
                .unwrap();

        let colhido = ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\"));

        assert!(matches!(colhido, EstadoDoJob::Colhido { .. }));
        assert!(
            !arquivos.foi_consultado(r"E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt"),
            "foi procurar o desfecho de um job que ja tinha sido colhido"
        );
    }

    // ─────────────────────── o job pendente da E5 ───────────────────────

    #[test]
    fn um_job_de_verificacao_diz_que_nao_ha_disco_em_vez_de_deixar_em_branco() {
        // A E11 trouxe uma operacao que nao nomeia disco: o `ocs-chkimg` opera
        // sobre a imagem. Uma linha `Disco alvo .....` seguida de nada faria
        // quem lê procurar o que se perdeu — e o que aconteceu foi nao haver
        // disco a nomear, que e outra coisa.
        let saida = com_estado(EstadoDoJob::Pendente {
            estado: crate::estado::Estado {
                comando: crate::receita::Operacao::Verificacao,
                disco: None,
                ..estado_gravado()
            },
            desfecho: Encontrado::SemArquivo,
        });

        assert!(
            saida.contains("Disco alvo ...................... nenhum · `verificacao` lê a imagem"),
            "{saida}"
        );
        assert!(
            saida.contains("verificacao-2026-08-22_Apps"),
            "a pasta do desfecho tem de levar a operacao:\n{saida}"
        );
    }

    #[test]
    fn o_job_pendente_mostra_o_selo_o_alvo_e_o_momento() {
        // O `estado.json` deixou de ser "existe ou nao existe" na E5. O selo
        // aparece inteiro porque e ele que a mensagem de job fantasma nomeia:
        // sem os dois lados a vista, ninguem confere nada.
        let saida = com_estado(job_pendente(Encontrado::SemArquivo));

        assert!(saida.contains(&linha("Selo", DO_JOB)), "{saida}");
        assert!(saida.contains(&linha("Disco alvo", "nvme0n1")), "{saida}");
        assert!(saida.contains("Pasta do desfecho"), "{saida}");
        assert!(saida.contains("backup-2026-08-22_Apps"), "{saida}");
    }

    #[test]
    fn o_momento_do_armar_sai_dizendo_que_nao_decide_nada() {
        // S-6 na tela. Quem lê uma data ao lado de um job pendente vai
        // compara-la com a data de uma imagem mais cedo ou mais tarde, e o
        // deslocamento de 3 h do Clonezilla (P-7) faria a conta dar errado.
        let saida = com_estado(job_pendente(Encontrado::SemArquivo));
        assert!(saida.contains("Armado em"), "{saida}");
        assert!(saida.contains("informativo, nunca comparado"), "{saida}");
    }

    #[test]
    fn o_selo_divergente_aparece_como_job_fantasma_na_tela() {
        // O criterio de aceite da etapa, pelo comando que o expoe.
        let saida = com_estado(job_pendente(Encontrado::Arquivo(
            crate::desfecho::Julgamento::JobFantasma {
                encontrado: Selo::novo(DE_OUTRO).unwrap(),
            },
        )));

        assert!(saida.contains("job fantasma"), "{saida}");
        assert!(saida.contains(DE_OUTRO), "{saida}");
    }

    #[test]
    fn o_estado_ilegivel_manda_desarmar_em_vez_de_dizer_que_nao_ha_job() {
        // "Nao entendi o arquivo" nao pode virar "nao ha nada esperando": o
        // dispositivo pode estar armado e ninguem saber com o que.
        let saida = com_estado(EstadoDoJob::Ilegivel {
            motivo: "o arquivo termina no meio".to_string(),
        });

        assert!(saida.contains("nao e o mesmo que nao haver job"), "{saida}");
        assert!(saida.contains("arca desarmar"), "{saida}");
    }

    #[test]
    fn boot_unico_limpo_ao_lado_de_job_pendente_nao_e_contradicao() {
        // O que se ve depois de um `arca desarmar`: o dispositivo esta inerte
        // e o job continua registrado por colher, porque desarmar nao toca no
        // `estado.json` (C-1). As duas linhas aparecem, e cada uma diz o que e.
        let saida = com_estado(job_pendente(Encontrado::SemArquivo));

        assert!(saida.contains(&linha("Boot unico", "nao armado")), "{saida}");
        assert!(saida.contains("backup `2026-08-22_Apps`"), "{saida}");
    }

    // ──────────────────── o comando inteiro, lendo o disco ────────────────────

    #[test]
    fn o_comando_lê_o_conteudo_do_estado_e_julga_o_desfecho() {
        // A E4 so perguntava se o arquivo existia. A E5 lê o que ele diz e
        // procura o desfecho no caminho que a **propria receita** escreveria.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc")
            .com(ESTADO, &estado_gravado().como_json().unwrap())
            .com(
                r"E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt",
                &format!("ARCA_SELO={DE_OUTRO}\nARCA_BACKUP=OK\nARCA_FIM\n"),
            );

        let dispositivo = dispositivo_conectado();
        let job = ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\"));

        match job {
            EstadoDoJob::Pendente { estado, desfecho } => {
                assert_eq!(estado.selo.como_texto(), DO_JOB);
                assert_eq!(
                    desfecho,
                    Encontrado::Arquivo(crate::desfecho::Julgamento::JobFantasma {
                        encontrado: Selo::novo(DE_OUTRO).unwrap()
                    })
                );
            }
            outro => panic!("esperava job pendente, veio {outro:?}"),
        }
    }

    #[test]
    fn sem_arca_fim_no_lugar_do_desfecho_o_comando_nomeia_as_duas_causas() {
        // C-12: ausencia de desfecho e falha, nunca silencio.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc")
            .com(ESTADO, &estado_gravado().como_json().unwrap());

        let dispositivo = dispositivo_conectado();
        match ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\")) {
            EstadoDoJob::Pendente { desfecho, .. } => {
                assert_eq!(desfecho, Encontrado::SemArquivo);
                assert!(desfecho.to_string().contains("boot nao aconteceu"));
            }
            outro => panic!("esperava job pendente, veio {outro:?}"),
        }
    }

    #[test]
    fn o_estado_truncado_no_disco_chega_como_ilegivel_e_nao_como_ausencia() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc")
            .com(ESTADO, "{\n  \"selo\": \"a3f1c9e0");

        let dispositivo = dispositivo_conectado();
        assert!(matches!(
            ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\")),
            EstadoDoJob::Ilegivel { .. }
        ));
    }

    #[test]
    fn sem_estado_json_nao_ha_job_e_nada_e_procurado() {
        let arquivos = ArquivosEmMemoria::novo();
        let dispositivo = dispositivo_conectado();

        assert_eq!(
            ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\")),
            EstadoDoJob::Nenhum
        );
    }

    // ───── "nao consegui olhar" nunca vira "nao ha nada" (ADR-0005) ─────

    #[test]
    fn o_estado_que_nao_se_deixa_lê_nao_vira_ausencia_de_job() {
        // A revisao desta etapa achou isto: a versao anterior perguntava
        // `arquivos.existe()` antes de lê, e `Path::exists` transforma
        // **qualquer** falha de I/O em `false`. Um `estado.json` presente num
        // volume com problema de leitura sairia como "Estado no ARCABOOT:
        // nenhum", e alguem reiniciaria achando que nao ha nada esperando.
        //
        // A defesa que eu tinha escrito estava construida sobre a funcao que
        // ja confundia os dois casos — o padrao de sempre: peca nova encaixada
        // em peca antiga que ninguem releu.
        let arquivos = ArquivosQueRecusam::com(
            ESTADO,
            std::io::ErrorKind::PermissionDenied,
            "acesso negado",
        );
        let dispositivo = dispositivo_conectado();

        match ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\")) {
            EstadoDoJob::Ilegivel { motivo } => assert!(motivo.contains("acesso negado")),
            outro => panic!("um estado ilegivel virou {outro:?}"),
        }
    }

    #[test]
    fn o_desfecho_que_nao_se_deixa_lê_nao_vira_boot_que_nao_aconteceu() {
        // Mesmo mecanismo, do outro lado: um `arca-fim.txt` presente e
        // ilegivel saindo como "o boot nao aconteceu" faria alguem concluir
        // que o backup nunca rodou — quando ele pode ter terminado bem.
        let desfecho_em = r"E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt";
        let arquivos = ArquivosQueRecusam::com(
            desfecho_em,
            std::io::ErrorKind::PermissionDenied,
            "acesso negado",
        )
        .com_arquivo(ESTADO, &estado_gravado().como_json().unwrap());

        let dispositivo = dispositivo_conectado();
        match ler_o_job(&arquivos, &dispositivo, std::path::Path::new(r"E:\")) {
            EstadoDoJob::Pendente { desfecho, .. } => {
                assert!(
                    matches!(desfecho, Encontrado::NaoDeuParaLer { .. }),
                    "um desfecho ilegivel virou {desfecho:?}"
                );
            }
            outro => panic!("esperava job pendente, veio {outro:?}"),
        }
    }

    #[test]
    fn arcaboot_sem_letra_nao_e_relatado_como_arcaboot_ausente() {
        // Terceiro achado da revisao. `caminho_do_estado` falha por **dois**
        // motivos, e o codigo dizia que so havia um. Com o `ARCABOOT` na mesa
        // e sem letra, "sem ARCABOOT" mandaria alguem procurar um dispositivo
        // que ja esta conectado.
        let dispositivo = Dispositivo {
            boot: Some(Volume {
                letra: None,
                ..volume(dispositivo::ARCABOOT, 'R', 1_700_000_000, 1_070_000_000)
            }),
            ..dispositivo_conectado()
        };

        match ler_o_job(
            &ArquivosEmMemoria::novo(),
            &dispositivo,
            std::path::Path::new(r"E:\"),
        ) {
            EstadoDoJob::SemOndeOlhar { motivo } => {
                assert!(
                    motivo.contains("letra"),
                    "o motivo nao diz que o problema e a letra: {motivo}"
                );
                assert!(
                    !motivo.contains("nao tem a particao"),
                    "a particao esta la; o motivo mente: {motivo}"
                );
            }
            outro => panic!("esperava sem onde olhar, veio {outro:?}"),
        }
    }

    #[test]
    fn sem_arcaboot_o_status_diz_o_que_isso_impede() {
        // `arca list` funciona sem `ARCABOOT`, porque imagem mora no
        // `ARCAVAULT`. Armar, nao: a receita e o estado moram no `ARCABOOT`.
        let dispositivo = Dispositivo {
            boot: None,
            ..dispositivo_conectado()
        };

        let saida = montar_com(&dispositivo, PT);
        assert!(
            saida.contains("ausente — sem ele nao ha onde gravar"),
            "{saida}"
        );
        assert!(
            saida.contains("sem ARCABOOT conectado para conferir"),
            "{saida}"
        );
    }

    #[test]
    fn midia_removivel_leva_o_aviso_de_c6() {
        // O `bcdedit` recusa esse alvo respondendo "êxito". O aviso vem do
        // Windows, que ja sabe que aquilo e um pendrive, e chega antes de
        // alguem tentar armar.
        let dispositivo = Dispositivo {
            boot: Some(Volume {
                tipo_de_midia: TipoDeMidia::Removivel,
                ..volume(dispositivo::ARCABOOT, 'R', 1_700_000_000, 1_070_000_000)
            }),
            ..dispositivo_conectado()
        };

        let saida = montar_com(&dispositivo, PT);
        assert!(saida.contains("AVISO (C-6)"), "{saida}");
        assert!(saida.contains("F12"), "faltou dizer o que fazer");
    }

    #[test]
    fn o_dispositivo_normal_nao_leva_aviso_nenhum() {
        assert!(!montar_com(&dispositivo_conectado(), PT).contains("AVISO"));
    }

    #[test]
    fn a_listagem_de_imagens_e_a_mesma_do_arca_list() {
        let dispositivo = dispositivo_conectado();
        let pastas = uma_imagem();

        assert!(
            montar_com(&dispositivo, PT)
                .contains(&list::montar(&pastas, dispositivo.vault.livre_bytes))
        );
    }
}
