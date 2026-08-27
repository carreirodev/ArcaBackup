//! Ler a configuracao de boot do firmware.
//!
//! Este e o unico parser do sistema cuja leitura errada leva a maquina a
//! bootar no lugar errado com uma receita armada. Por isso ele e codigo puro
//! sobre texto, do lado de ca da porta [`crate::portas::Firmware`], e por isso
//! os seus testes rodam contra saidas que o `bcdedit` **escreveu de verdade**,
//! nos dois idiomas — ver `recursos/capturas/PROVENIENCIA.md`.
//!
//! # Por valor, e nao por nome de campo
//!
//! O `bcdedit` traduz duas coisas: o titulo de cada bloco e o nome do campo
//! `identificador`. Todo o resto — `device`, `path`, `description`,
//! `displayorder`, `bootsequence` — sai identico em portugues e em ingles
//! (§3.1 do PRD; as duas capturas do mesmo BCD provam as duas metades).
//!
//! Dai as duas regras deste parser:
//!
//! - o **titulo do bloco nunca decide nada**. Ele e traduzido, e alem disso
//!   nao distingue a entrada do ARCA da do Windows: as duas aparecem como
//!   `Windows Boot Manager`, porque a do ARCA nasceu de um `bcdedit /copy`;
//! - o **identificador se acha por posicao**, e nao pelo nome do campo: e
//!   sempre o primeiro par de cada bloco, e o seu valor e sempre um `{...}`.
//!
//! # Sobre C-6 e as palavras `Removable Media`
//!
//! A fundacao §3.1 diz que o `bcdedit` rejeita `Removable Media` em silencio,
//! respondendo "êxito" e mantendo o valor antigo, e a E2 herdou disso a tarefa
//! de "distinguir `External hard disk media` de `Removable Media`".
//!
//! **Essas palavras nao saem do `bcdedit`.** Procuradas no `bcdedit.exe` e nos
//! seus recursos nos dois idiomas: nao estao la. Sao valores de `MediaType` do
//! WMI (`Win32_DiskDrive`, em `cimwin32.dll`), que e outra ferramenta e outra
//! pergunta. Nenhum parser da saida do `bcdedit` pode produzi-las, hoje ou
//! nunca.
//!
//! O que o `bcdedit` de fato oferece e melhor do que uma etiqueta: perguntado
//! de novo, ele diz para onde a entrada aponta. A rejeicao silenciosa aparece
//! como um `device` que **nao mudou** depois da escrita — e e isso que
//! [`EntradaDeFirmware::aponta_para`] verifica. E a mesma logica de C-3: o
//! sucesso do `bcdedit` nunca e prova; o `/enum` seguinte e.
//!
//! O sinal antecipado, esse, vem do Windows: um volume que o `GetDriveType`
//! classifica como [`crate::portas::TipoDeMidia::Removivel`] e um pendrive, e
//! o `arca status` avisa antes de alguem tentar armar. Ver
//! `src/adaptadores/windows/volumes.rs` sobre por que a classificacao para por
//! ai.

use crate::erro::{Erro, Resultado};
use crate::portas::Firmware;

/// O nome da entrada de firmware do ARCA (C-4).
pub const ARCA: &str = "ARCA";

/// O nome da entrada legada desta maquina (§3.1 do PRD).
///
/// Procurar so por `ARCA` criaria uma entrada orfa ao lado desta, e a maquina
/// passaria a ter duas formas de bootar no Clonezilla — uma delas sem ninguem
/// olhando.
pub const LEGADA: &str = "Clonezilla";

/// Se esta descricao e a de uma entrada do ARCA — a propria ou a legada.
///
/// # E a identidade de uma entrada de firmware neste projeto, e ha medicao
///
/// **O identificador que o `bcdedit /enum firmware` devolve nao e identidade.**
/// Medido em 25/08/2026, no marco em GPT: o
/// `{31cc955f-a0ae-11f1-8a54-806e6f6e6963}` era `UEFI:CD/DVD Drive`, **sem
/// `device`**, antes de um boot; depois dele, o **mesmo GUID** era `ARCA GPT
/// TESTE`, com `device partition=E:`. Ele nomeia o *slot* `Boot####` da NVRAM,
/// e nao a entrada que esta nele — e o firmware reescreve os slots.
///
/// Este projeto ja fazia a coisa certa sem ter medido por que: [`chamada`]
/// procura a entrada **pela descricao**, e nao por um GUID guardado. Esta
/// funcao e a mesma regra exposta para quem precisa conferir o caminho
/// inverso — dado um identificador, ele ainda nomeia a entrada que se pensava?
///
/// A caixa nao diferencia pela mesma razao de [`Leitura::chamada`]: quem
/// digitou a descricao da entrada legada foi uma pessoa, uma vez.
///
/// Ver o [ADR-0025](../docs/adr/0025-o-arca-particiona-em-gpt.md).
pub fn e_descricao_do_arca(descricao: &str) -> bool {
    descricao.eq_ignore_ascii_case(ARCA) || descricao.eq_ignore_ascii_case(LEGADA)
}

/// O gerenciador de inicializacao do firmware, dono da ordem de boot.
///
/// `bcdedit /enum firmware` o mostra pelo apelido; com `/v`, pelo GUID. O ARCA
/// consulta sem `/v`, mas reconhecer os dois custa uma linha e evita que um
/// dia alguem acrescente a flag e o parser emudeca.
const FWBOOTMGR: [&str; 2] = ["{fwbootmgr}", "{a5a30fa2-3d06-4e9f-b5f4-a01df9d1fcba}"];

/// Os campos que o `bcdedit` **nao** traduz.
const DESCRIPTION: &str = "description";
const DEVICE: &str = "device";
const PATH: &str = "path";
const DISPLAYORDER: &str = "displayorder";
const BOOTSEQUENCE: &str = "bootsequence";

/// Para onde uma entrada de firmware aponta, como o `bcdedit` escreve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Alvo {
    /// `partition=R:` — o Windows tem uma letra para essa particao.
    ParticaoComLetra(char),

    /// `partition=\Device\HarddiskVolume1` — particao sem letra atribuida. E o
    /// que o `bcdedit` mostra da particao EFI do sistema.
    ParticaoSemLetra(String),

    /// `ramdisk=[...]...`, `locate=...` e o que mais aparecer. Guardado inteiro
    /// porque o ARCA nao tem o que fazer com isso alem de mostrar.
    Outro(String),
}

impl Alvo {
    /// Como o `bcdedit` escreveria este alvo de volta.
    ///
    /// E a forma que a E7 passa ao `bcdedit /set`, e e contra ela que a
    /// releitura de C-3 compara.
    pub fn como_bcdedit_escreve(&self) -> String {
        match self {
            Alvo::ParticaoComLetra(letra) => format!("partition={letra}:"),
            Alvo::ParticaoSemLetra(caminho) => format!("partition={caminho}"),
            Alvo::Outro(texto) => texto.clone(),
        }
    }

    /// A letra da particao, quando o `bcdedit` deu uma. Sempre em maiuscula,
    /// que e como o Windows nomeia volume.
    pub fn letra(&self) -> Option<char> {
        match self {
            Alvo::ParticaoComLetra(letra) => Some(*letra),
            _ => None,
        }
    }

    fn ler(valor: &str) -> Alvo {
        let Some(particao) = valor.strip_prefix("partition=") else {
            return Alvo::Outro(valor.to_string());
        };

        // `X:` e letra; qualquer outra coisa e caminho de dispositivo.
        let mut caracteres = particao.chars();
        match (caracteres.next(), caracteres.next(), caracteres.next()) {
            (Some(letra), Some(':'), None) if letra.is_ascii_alphabetic() => {
                Alvo::ParticaoComLetra(letra.to_ascii_uppercase())
            }
            _ => Alvo::ParticaoSemLetra(particao.to_string()),
        }
    }
}

/// Uma entrada de boot, do jeito que o `bcdedit` a descreve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaDeFirmware {
    /// O `{guid}` ou o apelido. Achado por posicao — e o primeiro par do bloco
    /// —, porque o nome deste campo e o unico que sai traduzido.
    pub identificador: String,

    /// O nome legivel. E por ele que C-4 acha a entrada do ARCA, e ele nao e
    /// traduzido.
    pub descricao: Option<String>,

    pub alvo: Option<Alvo>,

    /// O `path` — o `.efi` que a entrada carrega.
    pub caminho: Option<String>,
}

impl EntradaDeFirmware {
    /// Se esta entrada, **relida** do `bcdedit`, aponta para onde se pediu.
    ///
    /// E assim que C-6 se verifica, e nao por etiqueta nenhuma: quando o alvo e
    /// midia removivel, o `bcdedit` responde "êxito" e mantem o valor antigo.
    /// So a releitura revela — que e o que C-3 ja exige de toda escrita.
    pub fn aponta_para(&self, alvo: &Alvo) -> bool {
        self.alvo.as_ref() == Some(alvo)
    }
}

/// Como a entrada do ARCA foi encontrada (C-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Procedencia {
    /// Uma entrada chamada `ARCA`, que o proprio ARCA criou ou migrou.
    Propria,

    /// A entrada legada `Clonezilla`, feita a mao antes de o ARCA existir. E
    /// esta que a E7 migra, em vez de criar outra ao lado.
    Legada,
}

/// A entrada do ARCA, e por qual dos dois nomes ela apareceu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntradaDoArca<'a> {
    pub entrada: &'a EntradaDeFirmware,

    /// A descricao como esta escrita no firmware, e nao a que se procurou: a
    /// busca ignora caixa, e mostrar `Clonezilla` onde esta escrito
    /// `clonezilla` seria o ARCA relatando o que ele espera em vez do que ha.
    pub descricao: &'a str,

    pub procedencia: Procedencia,
}

/// O que um `bcdedit /enum firmware` diz.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Leitura {
    pub entradas: Vec<EntradaDeFirmware>,

    /// O `displayorder` do gerenciador de firmware: a ordem **permanente**,
    /// que C-5 proibe o ARCA de tocar.
    pub ordem_permanente: Vec<String>,

    /// O `bootsequence` do gerenciador de firmware: o boot unico armado. Vazio
    /// quando nao ha nenhum — que e o estado inerte de C-1.
    pub boot_unico: Vec<String>,

    /// Se o bloco do `{fwbootmgr}` apareceu na enumeracao.
    ///
    /// Existe porque [`ler`] nunca falha: texto que ele nao entende vira
    /// leitura vazia, e leitura vazia tem `boot_unico` vazio — que e
    /// indistinguivel de "nao ha boot unico armado". Para quem **exibe**, dá
    /// no mesmo. Para quem acabou de mandar apagar a marca e esta conferindo
    /// se ela sumiu (C-3), nao dá: "nao entendi a resposta" viraria "desarmou"
    /// e o boot unico continuaria armado. Ver [`crate::desarme`].
    pub viu_o_gerenciador: bool,

    /// O codigo com que o `bcdedit` saiu, quando saiu **diferente de zero e
    /// ainda assim listou** (C-15). `None` e o caso normal: codigo 0.
    ///
    /// Medido em 27/08/2026: com a entrada `ARCA` apontando para uma particao
    /// que o `arca prepare` tinha acabado de apagar, **todo** `/enum` desta
    /// maquina imprimia a listagem inteira e terminava com *"Foi especificado
    /// um dispositivo inexistente."*, codigo 1 — inclusive o `/enum {bootmgr}`
    /// e o de uma `UEFI:*` sem `device` nenhum. A recusa e sobre o estado da
    /// NVRAM, e nao sobre o que foi pedido; a listagem, essa, veio completa.
    ///
    /// Fica na leitura para quem escreve poder decidir com ele: o `arca
    /// prepare` nao **cria** entrada a partir de uma leitura assim (C-4), so
    /// reusa — e registra o codigo. Ver [`enumerar`].
    pub codigo_da_recusa: Option<i32>,
}

impl Leitura {
    /// A entrada do ARCA, procurada como C-4 manda: primeiro pelo nome
    /// proprio, e so entao pela legada.
    ///
    /// A ordem nao e detalhe. Uma maquina que ja tenha as duas — porque a
    /// migracao foi interrompida no meio — tem de ser levada a usar a do ARCA,
    /// nunca a legada; o contrario deixaria a migracao sem fim.
    pub fn entrada_do_arca(&self) -> Option<EntradaDoArca<'_>> {
        for (nome, procedencia) in [(ARCA, Procedencia::Propria), (LEGADA, Procedencia::Legada)] {
            if let Some((entrada, descricao)) = self.chamada(nome) {
                return Some(EntradaDoArca {
                    entrada,
                    descricao,
                    procedencia,
                });
            }
        }
        None
    }

    /// A entrada com esta descricao, sem diferenciar caixa, junto da descricao
    /// como ela esta escrita.
    ///
    /// Sem diferenciar porque quem digitou a descricao da entrada legada foi
    /// uma pessoa, numa linha de comando, uma vez — e um `clonezilla` teimoso
    /// nao pode fazer o ARCA criar uma entrada orfa ao lado da que ja existe.
    fn chamada(&self, procurada: &str) -> Option<(&EntradaDeFirmware, &str)> {
        self.entradas.iter().find_map(|entrada| {
            let sua = entrada.descricao.as_deref()?;
            sua.eq_ignore_ascii_case(procurada)
                .then_some((entrada, sua))
        })
    }

    /// Se ha boot unico armado no firmware.
    pub fn tem_boot_unico(&self) -> bool {
        !self.boot_unico.is_empty()
    }

    /// Cada identificador da ordem permanente junto da entrada que ele nomeia
    /// — `None` quando nao ha bloco com esse identificador na leitura.
    ///
    /// A resolucao e a mesma que a linha `Ordem de boot` do `arca status` fazia
    /// a mao, e ela mora aqui desde que o `arca prepare` passou a precisar da
    /// mesma pergunta sem ter um [`crate::dispositivo::Dispositivo`] em maos.
    pub fn ordem_resolvida(&self) -> impl Iterator<Item = (&str, Option<&EntradaDeFirmware>)> {
        self.ordem_permanente.iter().map(|identificador| {
            (
                identificador.as_str(),
                self.entradas
                    .iter()
                    .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador)),
            )
        })
    }

    /// A primeira entrada da ordem permanente que **nao diz para onde aponta**,
    /// entre as `ate` primeiras, pelo nome com que ela se apresenta.
    ///
    /// # Por que a ausencia de alvo nao e a mesma resposta que "nao leva"
    ///
    /// As entradas que o firmware acrescenta no POST — `UEFI:CD/DVD Drive`,
    /// `UEFI:Removable Device`, `UEFI:Network Device` — trazem so
    /// `identificador` e `description`: nem `device` nem `path`. Quem as
    /// resolve e o firmware, no proximo POST, pelo que estiver conectado; nao
    /// ha alvo escrito contra o que conferi-las. Lê isso como "nao leva ao
    /// dispositivo" e transformar um **nao sei** na resposta tranquilizadora,
    /// que e o erro que [`Leitura::viu_o_gerenciador`] existe para nao cometer
    /// no bloco vizinho. E C-14 e P-28, e o
    /// [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).
    ///
    /// **O `{bootmgr}` nao entra aqui, e e essa distincao que impede o aviso de
    /// sair em toda tela.** Ele aponta para `partition=\Device\
    /// HarddiskVolume1`: um alvo concreto que so nao da para conferir por
    /// letra. Uma entrada **com** `device` aponta para uma particao; uma **sem**
    /// `device` nao aponta para coisa nenhuma. Medido na captura de 24/08/2026.
    ///
    /// Um identificador da ordem **sem bloco nenhum** conta junto: a leitura
    /// que o deixou de fora tambem nao diz para onde ele aponta.
    pub fn primeira_sem_alvo(&self, ate: usize) -> Option<String> {
        self.ordem_resolvida()
            .take(ate)
            .find(|(_, entrada)| entrada.is_none_or(|entrada| entrada.alvo.is_none()))
            .map(|(identificador, entrada)| nome_na_ordem(identificador, entrada))
    }
}

/// Como uma entrada da ordem se apresenta numa tela: a descricao que ela tem,
/// e o identificador quando ela nao tem nenhuma — ou nem bloco tem.
///
/// Quem lê a tela reconhece `ARCA` e `UEFI:Removable Device`, e nao
/// `{6cc093dc-…}`; mas um GUID dito e melhor do que um silencio.
pub fn nome_na_ordem(identificador: &str, entrada: Option<&EntradaDeFirmware>) -> String {
    entrada
        .and_then(|entrada| entrada.descricao.as_deref())
        .unwrap_or(identificador)
        .to_string()
}

/// Lê a saida de um `bcdedit /enum`.
///
/// Nunca falha. Texto que nao e uma enumeracao — a recusa "Não há objetos
/// correspondentes", por exemplo — devolve uma leitura vazia, e quem chamou
/// decide o que uma leitura vazia significa. Levantar erro aqui obrigaria o
/// parser a distinguir recusa de repositorio vazio pelo texto, em dois
/// idiomas, que e exatamente o tipo de interpretacao que C-3 quer evitar.
pub fn ler(texto: &str) -> Leitura {
    let mut leitura = Leitura::default();

    for bloco in blocos(texto) {
        // O identificador e o primeiro par, qualquer que seja o nome traduzido
        // que o `bcdedit` lhe deu. Bloco cujo primeiro valor nao e um `{...}`
        // nao e entrada de boot.
        let Some(identificador) = bloco
            .first()
            .and_then(|(_, valores)| valores.first())
            .filter(|valor| valor.starts_with('{') && valor.ends_with('}'))
        else {
            continue;
        };

        let primeiro = |campo: &str| {
            bloco
                .iter()
                .find(|(nome, _)| nome == campo)
                .and_then(|(_, valores)| valores.first())
                .cloned()
        };
        let todos = |campo: &str| {
            bloco
                .iter()
                .find(|(nome, _)| nome == campo)
                .map(|(_, valores)| valores.clone())
                .unwrap_or_default()
        };

        // O gerenciador de firmware nao e uma entrada de boot: e quem guarda a
        // ordem permanente e a marca de boot unico.
        if FWBOOTMGR.contains(&identificador.as_str()) {
            leitura.ordem_permanente = todos(DISPLAYORDER);
            leitura.boot_unico = todos(BOOTSEQUENCE);
            leitura.viu_o_gerenciador = true;
            continue;
        }

        leitura.entradas.push(EntradaDeFirmware {
            identificador: identificador.clone(),
            descricao: primeiro(DESCRIPTION),
            alvo: primeiro(DEVICE).as_deref().map(Alvo::ler),
            caminho: primeiro(PATH),
        });
    }

    leitura
}

/// Enumera pelo `bcdedit` e le — e a recusa dele **nao apaga o que ele
/// listou** (C-15).
///
/// # O que foi medido em 27/08/2026, e por que isto existe
///
/// O `arca prepare` apagou um dispositivo ARCA existente (passo 5) e morreu
/// no passo 11, no primeiro `/enum`. Nao era o passo: **todo** `bcdedit
/// /enum` desta maquina — `{fwbootmgr}`, `firmware`, `{bootmgr}`, `all`, o de
/// uma `UEFI:*` sem `device` — passou a imprimir a listagem inteira e sair
/// com codigo 1, *"Foi especificado um dispositivo inexistente."*. A causa,
/// confirmada por releitura da NVRAM: a entrada `ARCA` apontava para o GUID
/// da `ARCABOOT` que o proprio comando tinha acabado de apagar. O `bcdedit
/// /set device` para a particao nova devolveu o codigo 0 a todos os `/enum`.
///
/// Ate ali, codigo diferente de zero era recusa, ponto — e a razao continua
/// valendo: sem privilegio o `bcdedit` escreve "Acesso negado" **na saida
/// padrao** e sai com 1, e quem lesse so o texto concluiria que nao ha
/// entrada `ARCA` onde nao houve permissao para olhar. As duas coisas cabem
/// juntas, e o discriminante ja existia: `viu_o_gerenciador`. Uma resposta
/// que traz o gerenciador (ou, num `/enum {guid}`, a entrada pedida) e uma
/// leitura, e o codigo vira informacao **a mais**, guardada em
/// [`Leitura::codigo_da_recusa`]; uma resposta que nao traz nada continua
/// sendo a recusa que era, com o texto do `bcdedit` inteiro.
///
/// # O que isto nao faz
///
/// Nao decide por ninguem se a leitura basta para **escrever**. E o
/// `arca prepare` que recusa criar entrada (`/copy`) sobre uma leitura com
/// codigo — C-4, uma entrada so — e ele decide isso **antes** do ponto sem
/// volta. Os outros so reusam o que leram, e reusar o que se leu e o que C-3
/// sempre mandou.
///
/// A forma medida da recusa e a mensagem **depois** da listagem completa, em
/// todos os alvos; a leitura nao tem como saber se um `bcdedit` futuro parar
/// no meio. E por isso que a guarda e o gerenciador, que sai primeiro, e nao
/// a contagem de entradas.
pub fn enumerar(ferramenta: &dyn Firmware, alvo: &str) -> Resultado<Leitura> {
    match ferramenta.enumerar(alvo) {
        Ok(texto) => Ok(ler(&texto)),
        Err(Erro::FerramentaRecusou {
            ferramenta: "bcdedit",
            codigo,
            saida,
        }) => {
            let mut leitura = ler(&saida);
            if !leitura.viu_o_gerenciador && leitura.entradas.is_empty() {
                return Err(Erro::FerramentaRecusou {
                    ferramenta: "bcdedit",
                    codigo,
                    saida,
                });
            }
            leitura.codigo_da_recusa = Some(codigo);
            Ok(leitura)
        }
        Err(outro) => Err(outro),
    }
}

/// Um bloco e a lista de pares campo/valores de uma entrada; o titulo
/// traduzido fica de fora, porque nao decide nada.
type Bloco = Vec<(String, Vec<String>)>;

/// Reparte a saida do `bcdedit` em blocos.
///
/// A forma e sempre a mesma: um titulo, uma linha de tracos do tamanho dele,
/// os pares, e uma linha em branco. E a linha de tracos que marca o comeco —
/// nao o titulo, que e traduzido, nem a linha em branco, que tambem abre o
/// arquivo.
///
/// Campo de varios valores continua nas linhas seguintes, so com o valor,
/// alinhado sob o primeiro. E o caso de `displayorder` e `inherit`.
fn blocos(texto: &str) -> Vec<Bloco> {
    let mut blocos: Vec<Bloco> = Vec::new();
    let mut atual: Option<Bloco> = None;

    for linha in texto.lines() {
        if so_tracos(linha) {
            if let Some(bloco) = atual.take() {
                blocos.push(bloco);
            }
            atual = Some(Vec::new());
            continue;
        }

        let Some(bloco) = atual.as_mut() else {
            continue;
        };

        if linha.trim().is_empty() {
            blocos.push(std::mem::take(bloco));
            atual = None;
            continue;
        }

        // Linha recuada e continuacao do campo anterior. Sem campo anterior ela
        // nao tem dono, e e descartada em vez de virar campo sem nome.
        if linha.starts_with([' ', '\t']) {
            if let Some((_, valores)) = bloco.last_mut() {
                valores.push(linha.trim().to_string());
            }
            continue;
        }

        // O nome do campo nunca tem espaco; o valor pode ter — uma
        // `description` e texto livre.
        let (nome, valor) = match linha.split_once(char::is_whitespace) {
            Some((nome, resto)) => (nome.to_string(), resto.trim().to_string()),
            None => (linha.trim().to_string(), String::new()),
        };
        let valores = if valor.is_empty() {
            Vec::new()
        } else {
            vec![valor]
        };
        bloco.push((nome, valores));
    }

    if let Some(bloco) = atual {
        blocos.push(bloco);
    }
    blocos
}

/// A linha de tracos que o `bcdedit` desenha sob cada titulo.
fn so_tracos(linha: &str) -> bool {
    let podado = linha.trim_end();
    !podado.is_empty() && podado.chars().all(|c| c == '-')
}

#[cfg(test)]
mod testes {
    use super::*;

    /// `bcdedit /enum firmware` desta maquina, em portugues.
    const PT: &str = include_str!("../recursos/capturas/bcdedit-enum-firmware-pt.txt");

    /// O **mesmo BCD, no mesmo instante**, em ingles. Ver
    /// `recursos/capturas/PROVENIENCIA.md`.
    const EN: &str = include_str!("../recursos/capturas/bcdedit-enum-firmware-en.txt");

    /// A captura de 20/08, quando a entrada ainda se chamava `Clonezilla`.
    const LEGADO: &str = include_str!("../recursos/capturas/bcdedit-enum-firmware-legado-pt.txt");

    /// A captura de 24/08, depois do religar limpo que fechou P-22: a mesma
    /// ordem com as tres `UEFI:*` que o firmware acrescentou no POST.
    const POS_RELIGAR: &str =
        include_str!("../recursos/capturas/bcdedit-enum-firmware-2026-08-24-pos-religar.txt");

    /// O identificador da entrada desta maquina, o mesmo nas tres capturas.
    const GUID: &str = "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";

    /// 27/08/2026: o `bcdedit /enum firmware` que listou tudo e saiu com
    /// codigo 1, com a entrada `ARCA` apontando para uma particao que o
    /// `arca prepare` tinha acabado de apagar (`device unknown`).
    const COM_RECUSA_NO_FIM: &str = include_str!(
        "../recursos/capturas/bcdedit-enum-firmware-2026-08-27-dispositivo-inexistente.txt"
    );

    /// O `/enum {fwbootmgr}` do mesmo instante — so o gerenciador, e a mesma
    /// recusa no fim.
    const FWBOOTMGR_COM_RECUSA_NO_FIM: &str = include_str!(
        "../recursos/capturas/bcdedit-enum-fwbootmgr-2026-08-27-dispositivo-inexistente.txt"
    );

    /// A entrada `ARCA` desta maquina em 27/08 — outra maquina de BCD, outro
    /// GUID.
    const GUID_DE_27_08: &str = "{8a1c6901-a179-11f1-be2c-cbfb5c43df57}";

    /// Um `bcdedit` que responde a **todo** `/enum` com a mesma saida e o
    /// mesmo codigo — a forma medida em 27/08/2026.
    struct BcdeditQueSaiCom {
        codigo: i32,
        saida: &'static str,
    }

    impl Firmware for BcdeditQueSaiCom {
        fn enumerar(&self, _alvo: &str) -> Resultado<String> {
            if self.codigo == 0 {
                return Ok(self.saida.to_string());
            }
            Err(Erro::FerramentaRecusou {
                ferramenta: "bcdedit",
                codigo: self.codigo,
                saida: self.saida.to_string(),
            })
        }

        fn executar(&self, _argumentos: &[&str]) -> Resultado<String> {
            Ok(String::new())
        }
    }

    // ─────────── C-15: a recusa do bcdedit nao apaga o que ele listou ───────────

    #[test]
    fn a_recusa_no_fim_nao_apaga_o_que_o_bcdedit_listou() {
        // **Medido em 27/08/2026.** A listagem e a de sempre — gerenciador,
        // cinco na ordem, seis entradas — e a ultima linha e a recusa. Ate
        // este dia o codigo 1 virava `FerramentaRecusou` e o ARCA inteiro
        // ficava sem ler o firmware, inclusive o comando que consertaria o
        // estado tres linhas depois.
        let leitura = enumerar(
            &BcdeditQueSaiCom {
                codigo: 1,
                saida: COM_RECUSA_NO_FIM,
            },
            "firmware",
        )
        .expect("listou, logo e leitura");

        assert_eq!(leitura.codigo_da_recusa, Some(1));
        assert!(leitura.viu_o_gerenciador);
        assert_eq!(
            leitura.ordem_permanente.len(),
            5,
            "{:?}",
            leitura.ordem_permanente
        );
        assert!(leitura.boot_unico.is_empty());
        assert_eq!(leitura.entradas.len(), 6, "{:#?}", leitura.entradas);

        // A entrada do ARCA esta la, e e assim que ela aparece pendurada: o
        // `bcdedit` escreve `device unknown` para uma particao que nao existe.
        let achada = leitura
            .entrada_do_arca()
            .expect("a entrada ARCA esta na listagem");
        assert_eq!(achada.entrada.identificador, GUID_DE_27_08);
        assert_eq!(
            achada.entrada.alvo,
            Some(Alvo::Outro("unknown".to_string())),
            "e a forma medida de uma entrada que aponta para o nada"
        );
        assert_eq!(
            achada.entrada.caminho.as_deref(),
            Some(r"\EFI\boot\bootx64.efi")
        );

        // A linha da recusa vem depois da ultima entrada, sem linha em branco
        // antes, e o parser a engole como um campo sem dono — ela nao vira
        // entrada nem estraga a que a precede.
        let ultima = leitura.entradas.last().unwrap();
        assert_eq!(ultima.descricao.as_deref(), Some("UEFI:Network Device"));
        assert!(ultima.alvo.is_none());
    }

    #[test]
    fn o_gerenciador_sozinho_com_a_recusa_no_fim_tambem_e_leitura() {
        // E o `/enum {fwbootmgr}` que o `prepare`, o `armar` e o `desarme` leem
        // primeiro — o alvo em que o ARCA morria.
        let leitura = enumerar(
            &BcdeditQueSaiCom {
                codigo: 1,
                saida: FWBOOTMGR_COM_RECUSA_NO_FIM,
            },
            "{fwbootmgr}",
        )
        .expect("o gerenciador veio inteiro");

        assert!(leitura.viu_o_gerenciador);
        assert_eq!(leitura.codigo_da_recusa, Some(1));
        assert_eq!(leitura.ordem_permanente.len(), 5);
        assert_eq!(leitura.ordem_permanente[0], "{bootmgr}");
        assert!(leitura.boot_unico.is_empty());
        assert!(leitura.entradas.is_empty());
    }

    #[test]
    fn acesso_negado_continua_sendo_recusa_e_com_o_texto_inteiro() {
        // A razao de o codigo ter sido recusa ate aqui nao foi embora: sem
        // privilegio o `bcdedit` escreve isto **na saida padrao** e sai com 1.
        // Ler isso como "nao ha entrada ARCA" criaria uma duplicata (C-4).
        const ACESSO_NEGADO: &str = "Não foi possível abrir o repositório de dados de configuração da inicialização.\r\nAcesso negado.\r\n";

        let erro = enumerar(
            &BcdeditQueSaiCom {
                codigo: 1,
                saida: ACESSO_NEGADO,
            },
            "firmware",
        )
        .expect_err("nao trouxe gerenciador nem entrada: e recusa");

        match erro {
            Erro::FerramentaRecusou {
                ferramenta,
                codigo,
                saida,
            } => {
                assert_eq!(ferramenta, "bcdedit");
                assert_eq!(codigo, 1);
                assert_eq!(saida, ACESSO_NEGADO, "o texto do bcdedit chega inteiro");
            }
            outro => panic!("esperava a recusa do bcdedit, veio {outro}"),
        }
    }

    #[test]
    fn a_saida_vazia_com_codigo_um_tambem_e_recusa() {
        // Um `bcdedit` que nao disse nada e saiu com 1 nao listou coisa
        // nenhuma — e o mesmo caso do acesso negado, sem o texto.
        let erro = enumerar(
            &BcdeditQueSaiCom {
                codigo: 1,
                saida: "",
            },
            "firmware",
        )
        .expect_err("nada listado e recusa");
        assert!(matches!(erro, Erro::FerramentaRecusou { codigo: 1, .. }));
    }

    #[test]
    fn a_releitura_de_uma_entrada_so_com_a_recusa_no_fim_e_leitura() {
        // O `/enum {guid}` da releitura de C-3 nao traz o gerenciador: traz a
        // entrada pedida. E a entrada, com a recusa colada no fim, continua
        // sendo a entrada — e o que o `prepare` relê depois dos tres `/set`.
        const SO_A_ENTRADA: &str = "\r\nGerenciador de Inicialização do Windows\r\n\
             ---------------------------------------\r\n\
             identificador           {8a1c6901-a179-11f1-be2c-cbfb5c43df57}\r\n\
             device                  partition=E:\r\n\
             path                    \\EFI\\boot\\bootx64.efi\r\n\
             description             ARCA\r\n\
             Foi especificado um dispositivo inexistente.\r\n";

        let leitura = enumerar(
            &BcdeditQueSaiCom {
                codigo: 1,
                saida: SO_A_ENTRADA,
            },
            GUID_DE_27_08,
        )
        .expect("a entrada veio");

        assert!(!leitura.viu_o_gerenciador);
        assert_eq!(leitura.codigo_da_recusa, Some(1));
        assert_eq!(leitura.entradas.len(), 1);
        assert_eq!(leitura.entradas[0].alvo, Some(Alvo::ParticaoComLetra('E')));
    }

    #[test]
    fn o_codigo_zero_nao_deixa_recusa_na_leitura() {
        let leitura = enumerar(
            &BcdeditQueSaiCom {
                codigo: 0,
                saida: PT,
            },
            "firmware",
        )
        .unwrap();

        assert_eq!(leitura.codigo_da_recusa, None);
        assert_eq!(leitura, ler(PT), "o caminho normal e o `ler` de sempre");
    }

    #[test]
    fn o_idioma_nao_muda_nada_do_que_o_parser_extrai() {
        // Este e o teste que fecha o risco nomeado pelo plano. As duas capturas
        // sao leituras do mesmo BCD com segundos de diferenca, uma em cada
        // idioma: qualquer dependencia de texto traduzido aparece aqui como
        // diferenca.
        assert_eq!(ler(PT), ler(EN));
    }

    #[test]
    fn a_entrada_do_arca_e_achada_pela_descricao() {
        let leitura = ler(PT);
        let achado = leitura.entrada_do_arca().expect("a entrada ARCA existe");

        assert_eq!(achado.procedencia, Procedencia::Propria);
        assert_eq!(achado.descricao, "ARCA");
        assert_eq!(achado.entrada.identificador, GUID);
        assert_eq!(achado.entrada.alvo, Some(Alvo::ParticaoComLetra('R')));
        assert_eq!(
            achado.entrada.caminho.as_deref(),
            Some(r"\EFI\boot\bootx64.efi")
        );
    }

    #[test]
    fn sem_entrada_arca_vale_a_legada_e_ela_e_a_mesma_entrada() {
        // C-4 inteiro num teste: a captura de 20/08 e a de hoje sao a mesma
        // entrada de firmware, com o mesmo GUID, antes e depois de ser
        // renomeada. Procurar so por `ARCA` na de 20/08 criaria uma entrada
        // orfa ao lado desta.
        let leitura = ler(LEGADO);
        let achado = leitura.entrada_do_arca().expect("a legada esta la");

        assert_eq!(achado.procedencia, Procedencia::Legada);
        assert_eq!(achado.descricao, "Clonezilla");
        assert_eq!(achado.entrada.identificador, GUID);
        assert_eq!(
            achado.entrada.identificador,
            ler(PT).entrada_do_arca().unwrap().entrada.identificador
        );
    }

    #[test]
    fn a_entrada_propria_ganha_da_legada_quando_as_duas_existem() {
        // Uma migracao interrompida no meio deixa as duas. Preferir a legada
        // deixaria a migracao sem fim, e a maquina com duas formas de bootar no
        // Clonezilla.
        let texto = concat!(
            "\r\nGerenciador de Inicialização do Windows\r\n",
            "---------------------------------------\r\n",
            "identificador           {11111111-1111-1111-1111-111111111111}\r\n",
            "description             Clonezilla\r\n",
            "\r\nGerenciador de Inicialização do Windows\r\n",
            "---------------------------------------\r\n",
            "identificador           {22222222-2222-2222-2222-222222222222}\r\n",
            "description             ARCA\r\n"
        );

        let leitura = ler(texto);
        let achado = leitura.entrada_do_arca().unwrap();

        assert_eq!(achado.procedencia, Procedencia::Propria);
        assert_eq!(
            achado.entrada.identificador,
            "{22222222-2222-2222-2222-222222222222}"
        );
    }

    #[test]
    fn a_descricao_e_reconhecida_em_qualquer_caixa_e_relatada_como_esta_escrita() {
        let texto = concat!(
            "\r\nWindows Boot Manager\r\n",
            "--------------------\r\n",
            "identifier              {33333333-3333-3333-3333-333333333333}\r\n",
            "description             clonezilla\r\n"
        );

        let leitura = ler(texto);
        let achado = leitura.entrada_do_arca().unwrap();

        assert_eq!(achado.procedencia, Procedencia::Legada);
        assert_eq!(achado.descricao, "clonezilla", "relatou o que espera achar");
    }

    #[test]
    fn o_gerenciador_de_firmware_nao_e_uma_entrada_de_boot() {
        // Ele nao carrega `.efi` nenhum: e quem guarda a ordem. Se virasse
        // entrada, `arca status` ofereceria bootar no proprio gerenciador.
        let leitura = ler(PT);

        assert!(
            !leitura
                .entradas
                .iter()
                .any(|entrada| FWBOOTMGR.contains(&entrada.identificador.as_str())),
            "o {{fwbootmgr}} entrou na lista de entradas"
        );
        assert_eq!(leitura.ordem_permanente, vec!["{bootmgr}"]);
    }

    #[test]
    fn a_ordem_permanente_de_varias_linhas_e_lida_inteira() {
        // A captura de 20/08 tem cinco entradas no `displayorder`, e as quatro
        // ultimas vem em linhas de continuacao, so com o valor. Ler so a
        // primeira daria a C-5 uma ordem permanente que nao existe.
        let leitura = ler(LEGADO);

        assert_eq!(leitura.ordem_permanente.len(), 5);
        assert_eq!(leitura.ordem_permanente[0], "{bootmgr}");
        assert_eq!(leitura.ordem_permanente[1], GUID);
    }

    #[test]
    fn sem_bootsequence_nao_ha_boot_unico_armado() {
        // Nenhuma das capturas tem job armado, e e assim que o firmware fica
        // depois do desarmar de C-1.
        for captura in [PT, EN, LEGADO] {
            assert!(!ler(captura).tem_boot_unico());
        }
    }

    #[test]
    fn com_bootsequence_o_boot_unico_aparece() {
        // Caso **construido**: nenhuma captura desta maquina tem job armado,
        // porque armar e a etapa E7 e a E2 nao escreve no firmware. O formato
        // aqui e o do `bootsequence` documentado pelo `bcdedit`, e a E7 o
        // confirma contra hardware ao armar pela primeira vez.
        let texto = concat!(
            "\r\nGerenciador de Inicialização de Firmware\r\n",
            "----------------------------------------\r\n",
            "identificador           {fwbootmgr}\r\n",
            "displayorder            {bootmgr}\r\n",
            "bootsequence            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n",
            "timeout                 1\r\n"
        );

        let leitura = ler(texto);
        assert!(leitura.tem_boot_unico());
        assert_eq!(leitura.boot_unico, vec![GUID]);
        // C-5: o boot unico nao mexeu na ordem permanente.
        assert_eq!(leitura.ordem_permanente, vec!["{bootmgr}"]);
    }

    #[test]
    fn a_particao_sem_letra_nao_vira_letra() {
        // O `bootmgr` do Windows mora na particao EFI, que nao tem letra. Um
        // parser que arrancasse uma letra dai apontaria o ARCA para o volume
        // errado.
        let bootmgr = ler(PT)
            .entradas
            .into_iter()
            .find(|entrada| entrada.descricao.as_deref() == Some("Windows Boot Manager"))
            .expect("o bootmgr esta na enumeracao");

        assert_eq!(
            bootmgr.alvo,
            Some(Alvo::ParticaoSemLetra(r"\Device\HarddiskVolume1".into()))
        );
        assert_eq!(bootmgr.alvo.unwrap().letra(), None);
    }

    #[test]
    fn o_alvo_volta_ao_texto_que_o_bcdedit_escreveu() {
        // A ida e a volta tem de fechar: e esta string que a E7 passa ao
        // `bcdedit /set`, e e contra ela que a releitura de C-3 compara.
        for captura in [PT, EN, LEGADO] {
            for entrada in ler(captura).entradas {
                let Some(alvo) = entrada.alvo else { continue };
                assert!(
                    captura.contains(&alvo.como_bcdedit_escreve()),
                    "{:?} nao esta na captura",
                    alvo.como_bcdedit_escreve()
                );
            }
        }
    }

    #[test]
    fn o_alvo_relido_e_o_que_prova_se_o_bcdedit_aceitou() {
        // C-6 na forma em que ele e verificavel. O `bcdedit` responde "êxito" e
        // mantem o valor antigo quando o alvo e midia removivel; so a releitura
        // revela.
        let entrada = ler(PT).entrada_do_arca().unwrap().entrada.clone();

        assert!(entrada.aponta_para(&Alvo::ParticaoComLetra('R')));
        assert!(!entrada.aponta_para(&Alvo::ParticaoComLetra('F')));
    }

    #[test]
    fn recusa_do_bcdedit_nao_vira_entrada() {
        // Texto que nao e enumeracao devolve leitura vazia, e quem chamou
        // decide o que isso significa. Nos dois idiomas, porque interpretar
        // frase e o que C-3 quer evitar.
        for recusa in [
            "Não há objetos correspondentes ou o repositório está vazio.\r\n",
            "There are no matching objects or the store is empty.\r\n",
            "",
        ] {
            let leitura = ler(recusa);
            assert!(leitura.entradas.is_empty());
            assert!(leitura.entrada_do_arca().is_none());
            assert!(!leitura.tem_boot_unico());

            // E o que separa "nao ha boot unico" de "nao consegui olhar". Para
            // quem exibe, dá no mesmo; para a releitura de C-3 do desarmar,
            // nao dá — ver `crate::desarme`.
            assert!(
                !leitura.viu_o_gerenciador,
                "uma recusa nao pode passar por leitura do {{fwbootmgr}}"
            );
        }
    }

    #[test]
    fn a_enumeracao_de_verdade_traz_o_gerenciador_de_firmware() {
        for captura in [PT, EN, LEGADO] {
            assert!(
                ler(captura).viu_o_gerenciador,
                "o {{fwbootmgr}} esta nas tres capturas"
            );
        }
    }

    #[test]
    fn o_titulo_traduzido_nao_e_confundido_com_campo() {
        // As duas capturas trazem titulos diferentes para os mesmos blocos. Se
        // o titulo entrasse como campo, a comparacao pt/en falharia — e e por
        // isso que a linha de tracos, e nao o titulo, marca o comeco do bloco.
        assert!(PT.contains("Gerenciador de Inicialização do Windows"));
        assert!(EN.contains("Windows Boot Manager\r\n---"));

        for entrada in ler(PT).entradas {
            assert!(entrada.identificador.starts_with('{'), "{entrada:?}");
        }
    }

    #[test]
    fn o_parser_aguenta_lf_sozinho() {
        // As capturas tem CRLF, que e o que o `bcdedit` escreve. Um checkout do
        // git com outra configuracao de fim de linha nao pode mudar o que o
        // parser entende.
        assert_eq!(ler(PT), ler(&PT.replace("\r\n", "\n")));
    }

    #[test]
    fn a_entrada_sem_descricao_nao_e_confundida_com_a_do_arca() {
        // Os blocos `Aplicativo de Firmware` da captura de 20/08 tem so
        // identificador e descricao; um deles se chama `UEFI:Removable
        // Device`, que nao e nem `ARCA` nem `Clonezilla`.
        let leitura = ler(LEGADO);

        let sem_alvo = leitura
            .entradas
            .iter()
            .filter(|entrada| entrada.alvo.is_none())
            .count();
        assert_eq!(sem_alvo, 3, "os tres aplicativos de firmware");

        assert_eq!(
            leitura.entrada_do_arca().unwrap().descricao,
            "Clonezilla",
            "`UEFI:Removable Device` tem a palavra, e nao e a entrada do ARCA"
        );
    }

    #[test]
    fn as_tres_entradas_do_firmware_estao_na_ordem_e_nao_dizem_para_onde_apontam() {
        // **A medicao de que P-28 vive** (ADR-0021), nas duas capturas em que
        // as tres aparecem — 20/08 e 24/08, com GUIDs diferentes. O que o
        // `bcdedit` imprime delas e so `identificador` e `description`: quem as
        // resolve e o firmware, no POST.
        for captura in [LEGADO, POS_RELIGAR] {
            let leitura = ler(captura);

            assert_eq!(leitura.ordem_permanente.len(), 5, "as duas tem cinco");
            assert_eq!(
                leitura.primeira_sem_alvo(usize::MAX).as_deref(),
                Some("UEFI:CD/DVD Drive"),
                "a primeira opaca da ordem"
            );

            // **A distincao de que depende nao virar ruido**: o `{bootmgr}` e o
            // segundo da ordem tem alvo, e por isso as duas primeiras posicoes
            // nao levantam duvida nenhuma.
            assert_eq!(leitura.primeira_sem_alvo(2), None);
        }
    }

    #[test]
    fn a_ordem_resolvida_devolve_o_identificador_de_quem_nao_tem_bloco() {
        // Um identificador que a leitura nao resolve tambem nao diz para onde
        // aponta, e a tela precisa chama-lo de alguma coisa.
        let orfa = POS_RELIGAR.replacen(
            "identificador           {6cc093dc-9ff9-11f1-8a4e-806e6f6e6963}",
            "identificador           {6cc093dc-0000-0000-0000-000000000000}",
            1,
        );
        assert_ne!(orfa, POS_RELIGAR, "a troca nao pegou");

        let leitura = ler(&orfa);
        assert_eq!(
            leitura.primeira_sem_alvo(usize::MAX).as_deref(),
            Some("UEFI:CD/DVD Drive"),
            "a primeira opaca continua sendo a primeira"
        );
        assert!(
            leitura
                .ordem_resolvida()
                .any(|(identificador, entrada)| entrada.is_none()
                    && nome_na_ordem(identificador, entrada)
                        == "{6cc093dc-9ff9-11f1-8a4e-806e6f6e6963}"),
            "a orfa tem de sair pelo identificador"
        );
    }
}
