//! As sete defesas de PR-5 e o plano de partições — código puro (§7.1).
//!
//! Fica separado de [`crate::comandos::prepare`] pela razão de sempre: o
//! comando fala com o mundo, e o julgamento tem de ter teste sem hardware. E
//! aqui isso vale mais do que em qualquer outro lugar do ARCA, porque **o que
//! este módulo decide é qual disco vai ser apagado**.
//!
//! # A objeção que virou esta lista
//!
//! O [ADR-0014] registra a objeção levantada e superada quando P1 foi
//! revisado: *"o perigo não é particionar, é acertar em qual disco"*. Ela não
//! some com a decisão — vira as sete defesas, e nenhuma é opcional.
//!
//! O precedente que a sustenta é concreto: a revisão da E9 achou que R-8 tinha
//! um contorno por acidente de modelo, e a receita sairia `restoredisk <imagem>
//! sda` com o `sda` sendo o próprio dispositivo. **Identificar disco é onde
//! este código já errou.**
//!
//! # E este comando roda num mundo sem as defesas dos outros
//!
//! `arca prepare` é **o único comando do ARCA que não se localiza pelos
//! rótulos**. B-1 acha o dispositivo pelo `ARCAVAULT`, S-3 endereça por LABEL,
//! C-10 recusa rótulo repetido — e no disco que este comando vai preparar
//! nenhum deles existe. Não há o que C-10 recuse quando não há rótulo nenhum na
//! mesa.
//!
//! Daí as defesas serem dele sozinho, e daí elas serem sete.
//!
//! [ADR-0014]: ../docs/adr/0014-o-arca-particiona-o-dispositivo.md

use crate::formato::tamanho;
use crate::portas::TipoDeMidia;
use crate::portas::particionador::{DiscoParaPreparar, ParticoesFeitas, PlanoDeParticoes};
use std::fmt;

/// O rótulo da partição das imagens.
pub const ARCAVAULT: &str = crate::dispositivo::ARCAVAULT;

/// O rótulo da partição de boot.
pub const ARCABOOT: &str = crate::dispositivo::ARCABOOT;

/// O tamanho do `ARCABOOT`, **transcrito** da captura.
///
/// `1.677.721.600` bytes são exatamente 1600 MiB, e é o tamanho da partição
/// `R:` do dispositivo desta mesa — medido em 23/08/2026 e preservado em
/// `recursos/capturas/estrutura-de-particoes-do-dispositivo-2026-08-23.txt`.
///
/// PR-5 pede "FAT32 de ≥ 1 GB", e este número é maior do que o mínimo por um
/// motivo que não é folga: **é o que está bootando**. O pacote do Clonezilla
/// 3.3.3-15 ocupa 561 MB comprimido e cerca de 580 MB extraído, então 1,5 GiB
/// deixa espaço para uma versão maior sem que ninguém precise repartir o disco.
pub const ARCABOOT_BYTES: u64 = 1_677_721_600;

/// O menor `ARCABOOT` que PR-5 aceita.
///
/// Existe para a recusa de disco pequeno demais ter um número, e não para ser
/// usado: o plano sempre pede [`ARCABOOT_BYTES`].
pub const ARCABOOT_MINIMO_BYTES: u64 = 1_073_741_824;

/// O menor `ARCAVAULT` que faz sentido.
///
/// Não é uma regra do PRD — é a constatação de que um `ARCAVAULT` que não cabe
/// uma imagem não é um dispositivo ARCA. A menor imagem deste projeto tem
/// 32,9 GB; 16 GiB é metade disso, e recusar abaixo disso evita preparar um
/// pen drive de 8 GB que nunca vai servir para nada.
pub const ARCAVAULT_MINIMO_BYTES: u64 = 16 * 1024 * 1024 * 1024;

/// Por que este disco não pode ser preparado.
///
/// Cada variante é uma das sete defesas, e **nenhuma tem opção de forçar**. A
/// diferença entre elas e as recusas do resto do ARCA é o que está do outro
/// lado: aqui o modo de falha apaga o disco de alguém, e nenhuma confirmação
/// digitada compra isso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDaPreparacao {
    /// Não há disco com o índice pedido.
    DiscoDesconhecido { indice: u32, existentes: Vec<u32> },

    /// Defesa 1: o `MediaType` diz que é disco fixo, ou não diz nada.
    ///
    /// **Desconhecido também recusa**, e isso não é excesso: supor que um disco
    /// que não se sabe classificar é externo é o erro que faria a defesa passar
    /// batido — o mesmo raciocínio de
    /// [`crate::adaptadores::windows::wmi`], onde mídia irreconhecível nunca
    /// vira `DiscoFixo` por padrão.
    MidiaNaoRemovivel { modelo: String, tipo: TipoDeMidia },

    /// Defesa 2: é o disco onde o Windows mora, ou por onde a máquina bootou.
    DiscoDoSistema {
        modelo: String,
        e_do_sistema: bool,
        e_de_boot: bool,
    },

    /// Defesa 2, pelo outro lado: o disco carrega a letra do `%SystemDrive%`.
    ///
    /// O `IsSystem` e o `IsBoot` do `MSFT_Disk` respondem sobre **este** boot.
    /// A letra do `%SystemDrive%` responde sobre onde o Windows que está
    /// rodando mora. Numa máquina com dois Windows as duas perguntas divergem,
    /// e as duas recusam.
    CarregaOSystemDrive { modelo: String, letra: char },

    /// O disco é somente-leitura.
    SomenteLeitura { modelo: String },

    /// O disco é pequeno demais para as duas partições.
    DiscoPequenoDemais {
        modelo: String,
        tem: u64,
        precisa: u64,
    },

    /// Nenhum disco desta máquina passa pelas defesas, e não há o que oferecer.
    ///
    /// É a recusa do menu, e ela conta os recusados em vez de calar: uma tela
    /// que diz "nenhum disco" para quem está vendo dois discos na mesa parece
    /// defeito do ARCA. A tela lista cada um com o seu motivo antes desta
    /// mensagem — ver [`crate::comandos::prepare::montar_o_menu`].
    NadaAOferecer { recusados: usize },

    /// O que se digitou no menu não é o número de nenhum candidato.
    ///
    /// Uma tentativa, e não um laço — a mesma regra da confirmação e da lista
    /// do `arca restore`. Quem errou repete o comando, que até aqui não apagou
    /// nada. **E o Enter vazio cai aqui**, de propósito: o menu não tem padrão,
    /// porque um padrão seria o ARCA escolhendo o disco a apagar.
    EscolhaInvalida { digitado: String, quantas: usize },
}

impl RecusaDaPreparacao {
    /// O motivo em **uma linha**, para a lista do menu.
    ///
    /// # Por que não serve o `Display`
    ///
    /// Porque as duas respondem a perguntas diferentes. O `Display` é a
    /// mensagem de um comando que morreu por causa desta recusa: ele diz o
    /// motivo, diz que não há como forçar e diz o que fazer — e é longo de
    /// propósito, porque é a única coisa na tela.
    ///
    /// Na lista há três ou quatro discos, e cada um traz o seu motivo debaixo
    /// do nome. Três parágrafos de recusa afogariam a lista que eles anotam, e
    /// **a lista é o serviço** — quem está lendo veio escolher um disco, e não
    /// estudar por que os outros não servem. Quem quiser o texto inteiro o
    /// alcança apontando aquele disco com `--dispositivo`, e aí ele é a única
    /// coisa na tela outra vez.
    pub fn resumo(&self) -> String {
        match self {
            RecusaDaPreparacao::MidiaNaoRemovivel { tipo, .. } => match tipo {
                TipoDeMidia::Desconhecido => {
                    "o Windows nao soube classificar esta midia, e \"nao sei\" recusa junto com \"fixo\" (PR-5)".to_string()
                }
                _ => "e disco FIXO, e o `arca prepare` so prepara midia externa ou removivel (PR-5)"
                    .to_string(),
            },
            RecusaDaPreparacao::DiscoDoSistema {
                e_do_sistema,
                e_de_boot,
                ..
            } => format!(
                "e {} desta maquina (PR-5)",
                match (e_do_sistema, e_de_boot) {
                    (true, true) => "o disco do sistema E o disco de boot",
                    (true, false) => "o disco do sistema",
                    _ => "o disco de boot",
                }
            ),
            RecusaDaPreparacao::CarregaOSystemDrive { letra, .. } => format!(
                "carrega o volume {letra}:, que e onde este Windows mora (PR-5)"
            ),
            RecusaDaPreparacao::SomenteLeitura { .. } => {
                "esta marcado como somente-leitura, e nao ha o que particionar nele".to_string()
            }
            RecusaDaPreparacao::DiscoPequenoDemais { tem, precisa, .. } => format!(
                "tem {} e o ARCA precisa de pelo menos {}",
                tamanho(*tem),
                tamanho(*precisa)
            ),

            // As duas que nao falam de um disco da lista. Elas nunca chegam
            // aqui pelo caminho do menu — e devolver o texto inteiro e a
            // resposta honesta caso alguem as traga.
            outra => outra.to_string(),
        }
    }
}

impl fmt::Display for RecusaDaPreparacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDaPreparacao::DiscoDesconhecido { indice, existentes } => write!(
                f,
                "nao ha disco de indice {indice} nesta maquina. Os indices que existem sao: {}. O ARCA nao escolhe um disco para apagar — rode `arca prepare` sem `--dispositivo` para ver os discos desta maquina e escolher pelo numero",
                if existentes.is_empty() {
                    "nenhum".to_string()
                } else {
                    existentes
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
            RecusaDaPreparacao::MidiaNaoRemovivel { modelo, tipo } => write!(
                f,
                "o disco `{modelo}` e {}, e o `arca prepare` so prepara midia externa ou removivel. **Nao ha como forcar**: o modo de falha desta recusa apaga o Windows de alguem, e nenhuma confirmacao digitada compra isso (PR-5). {}",
                match tipo {
                    TipoDeMidia::DiscoFixo => "um disco FIXO".to_string(),
                    TipoDeMidia::Desconhecido =>
                        "de um tipo que o Windows nao soube classificar".to_string(),
                    outro => format!("do tipo {outro:?}"),
                },
                match tipo {
                    TipoDeMidia::Desconhecido =>
                        "E \"nao sei\" recusa junto com \"fixo\", de proposito: supor que o desconhecido e externo faria esta defesa passar batido justamente no caso em que ela mais importa",
                    _ =>
                        "O `MediaType` do WMI e quem responde isto, e ele diz `External hard disk media` para um SSD externo e `Fixed hard disk media` para o interno",
                }
            ),
            RecusaDaPreparacao::DiscoDoSistema {
                modelo,
                e_do_sistema,
                e_de_boot,
            } => write!(
                f,
                "o disco `{modelo}` e {} desta maquina. Prepara-lo apagaria o Windows que esta executando este comando. **Nao ha como forcar** (PR-5)",
                match (e_do_sistema, e_de_boot) {
                    (true, true) => "o disco do sistema E o disco de boot",
                    (true, false) => "o disco do sistema",
                    _ => "o disco de boot",
                }
            ),
            RecusaDaPreparacao::CarregaOSystemDrive { modelo, letra } => write!(
                f,
                "o disco `{modelo}` carrega o volume {letra}:, que e onde o Windows que esta rodando mora (`%SystemDrive%`). **Nao ha como forcar** (PR-5). Isto e uma segunda pergunta, e nao a mesma do `IsSystem`: aquele fala do boot corrente, e esta fala de onde este Windows esta"
            ),
            RecusaDaPreparacao::SomenteLeitura { modelo } => write!(
                f,
                "o disco `{modelo}` esta marcado como somente-leitura, e nao ha o que particionar nele. Tire a protecao antes"
            ),
            RecusaDaPreparacao::DiscoPequenoDemais {
                modelo,
                tem,
                precisa,
            } => write!(
                f,
                "o disco `{modelo}` tem {} e o ARCA precisa de pelo menos {}: sao {} para o {ARCABOOT} — onde o Clonezilla mora — e o resto para o {ARCAVAULT}, que guarda as imagens. Um dispositivo que nao cabe uma imagem nao serve para nada",
                tamanho(*tem),
                tamanho(*precisa),
                tamanho(ARCABOOT_BYTES)
            ),
            RecusaDaPreparacao::NadaAOferecer { recusados } => write!(
                f,
                "nenhum dos {recusados} disco(s) desta maquina pode ser preparado, e cada motivo esta na lista acima. O `arca prepare` so prepara midia externa ou removivel, e nunca o disco do sistema (PR-5). Conecte o disco que voce quer preparar e rode de novo"
            ),
            RecusaDaPreparacao::EscolhaInvalida { digitado, quantas } => write!(
                f,
                "`{digitado}` nao e um dos numeros da lista (1 a {quantas}). Nada foi tocado — rode o comando de novo. **O Enter vazio tambem nao escolhe**: o menu nao tem padrao, porque um padrao seria o ARCA escolhendo o disco a apagar"
            ),
        }
    }
}

/// O disco julgado e o plano que sai dele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preparacao {
    pub disco: DiscoParaPreparar,
    pub plano: PlanoDeParticoes,
}

/// As sete defesas, na ordem em que cada uma impede o que a seguinte não
/// impediria.
///
/// # A ordem, e o que cada posição protege
///
/// 1. **O disco existir**, porque nada abaixo se responde sem ele.
/// 2. **O disco do sistema**, antes de tudo o mais. É a recusa cuja mensagem
///    tem de aparecer inteira: se ela viesse depois da de tamanho, alguém leria
///    "o disco é pequeno demais" sobre o disco do próprio Windows e trocaria
///    por um maior. É a lição de R-8, que a E9 aprendeu ao pôr a recusa do
///    dispositivo antes da de tamanho.
/// 3. **O `%SystemDrive%`**, que é a mesma defesa por outro canal de
///    identidade — e são dois canais porque o vão entre eles é onde este
///    projeto já errou (a revisão da E9).
/// 4. **A mídia**, que é C-6 aplicado ao alvo.
/// 5. **Somente-leitura**, que impediria o resto de qualquer jeito.
/// 6. **O tamanho**, por último entre as recusas, porque é a única que se
///    resolve trocando de disco.
///
/// Quem passa por todas ganha um [`Preparacao`] com o plano — e o plano ainda
/// não escreveu nada. O ponto sem volta é a confirmação digitada, depois disto.
pub fn julgar(
    indice: u32,
    disco: Option<&DiscoParaPreparar>,
    existentes: &[u32],
    letra_do_sistema: Option<char>,
) -> Result<Preparacao, RecusaDaPreparacao> {
    let Some(disco) = disco else {
        return Err(RecusaDaPreparacao::DiscoDesconhecido {
            indice,
            existentes: existentes.to_vec(),
        });
    };

    // 2. O disco do sistema, e a mensagem tem de ser esta.
    if disco.e_do_sistema || disco.e_de_boot {
        return Err(RecusaDaPreparacao::DiscoDoSistema {
            modelo: disco.modelo.clone(),
            e_do_sistema: disco.e_do_sistema,
            e_de_boot: disco.e_de_boot,
        });
    }

    // 3. O mesmo perigo pelo outro canal. O `IsSystem` fala do boot corrente; a
    //    letra fala de onde este Windows mora. Numa maquina com dois Windows as
    //    duas divergem, e nenhuma das duas sozinha cobre a outra.
    if let Some(letra) = letra_do_sistema
        && disco
            .letras()
            .iter()
            .any(|sua| sua.eq_ignore_ascii_case(&letra))
    {
        return Err(RecusaDaPreparacao::CarregaOSystemDrive {
            modelo: disco.modelo.clone(),
            letra: letra.to_ascii_uppercase(),
        });
    }

    // 4. C-6 aplicado ao alvo. Desconhecido recusa junto com fixo.
    if !matches!(
        disco.tipo_de_midia,
        TipoDeMidia::DiscoExterno | TipoDeMidia::Removivel
    ) {
        return Err(RecusaDaPreparacao::MidiaNaoRemovivel {
            modelo: disco.modelo.clone(),
            tipo: disco.tipo_de_midia,
        });
    }

    if disco.somente_leitura {
        return Err(RecusaDaPreparacao::SomenteLeitura {
            modelo: disco.modelo.clone(),
        });
    }

    // 6. O tamanho, por ultimo.
    let minimo = ARCABOOT_BYTES + ARCAVAULT_MINIMO_BYTES;
    if disco.tamanho_bytes < minimo {
        return Err(RecusaDaPreparacao::DiscoPequenoDemais {
            modelo: disco.modelo.clone(),
            tem: disco.tamanho_bytes,
            precisa: minimo,
        });
    }

    Ok(Preparacao {
        disco: disco.clone(),
        plano: PlanoDeParticoes {
            indice_do_disco: disco.indice,
            // O `vault_bytes` e uma **estimativa** aqui: quem sabe quanto sobra
            // de verdade e o Windows, depois de escrever a tabela, e o
            // adaptador o recalcula pelo espaco livre real. O numero desta
            // struct existe para a tela poder dizer o tamanho antes de agir, e
            // por isso ele desconta o mesmo 1 MiB de alinhamento que o
            // `New-Partition` reserva — medido em 23/08/2026 no disco desta
            // mesa: `LargestFreeExtent` saiu 2.973.696 bytes abaixo do tamanho
            // do disco.
            vault_bytes: disco
                .tamanho_bytes
                .saturating_sub(ARCABOOT_BYTES)
                .saturating_sub(ALINHAMENTO_BYTES),
            boot_bytes: ARCABOOT_BYTES,
        },
    })
}

/// O que o `New-Partition` reserva antes da primeira partição e depois da
/// última.
///
/// Medido em 23/08/2026, no `JMicron Generic` de 480.103.981.056 bytes: depois
/// do `Initialize-Disk -PartitionStyle MBR`, o `LargestFreeExtent` respondeu
/// 480.101.007.360 — **2.973.696 bytes a menos**. A primeira partição saiu no
/// offset 1.048.576, que é o 1 MiB de alinhamento que o Windows usa desde o
/// Vista; o resto é reserva de fim.
///
/// O número entra só na **estimativa** que a tela imprime. Quem manda no
/// tamanho de verdade é o espaço livre que o Windows reporta na hora, e o
/// adaptador o lê em vez de calcular.
const ALINHAMENTO_BYTES: u64 = 2_973_696;

/// Os discos desta máquina repartidos entre o que se pode preparar e o que
/// não, cada recusa com o motivo que [`julgar`] deu.
///
/// # Por que ela não mede nem consulta nada de novo
///
/// O `arca prepare` **já enumera todos os discos** antes de qualquer coisa —
/// ele precisa da lista para a recusa de índice inexistente poder dizer quais
/// existem. E `julgar` já devolve, por disco, um veredito tipado. A oferta é
/// rodar o julgamento em cada um em vez de num só: nenhuma consulta nova,
/// nenhuma medição nova, a mesma informação que já estava na mão.
///
/// # É a `Oferta` do `arca restore`, e o nome é o mesmo de propósito
///
/// [`crate::comandos::restore::Oferta`] responde *"o que dá para restaurar"*
/// separando imagem de resíduo; esta responde *"o que dá para preparar"*
/// separando candidato de recusado. É o mesmo conceito — **o que se pode
/// escolher, e o que aparece dito mas sem número** —, e dois nomes para ele
/// fariam a segunda tela divergir da primeira na primeira mudança.
///
/// A doutrina que vem junto está escrita em
/// [`crate::comandos::restore::montar_a_lista`], e vale inteira aqui: os
/// recusados continuam ditos, embaixo e **sem número**, porque omiti-los faria
/// a lista parecer incompleta para quem sabe que há outro disco na mesa — e o
/// pior caso é justamente a defesa 1, que recusa o disco externo que o Windows
/// não soube classificar. Escondido, o motivo vira ausência; listado sem
/// número, ele vira uma frase.
pub struct Oferta<'a> {
    /// Só disco preparável, e na ordem em que a lista os numera.
    pub candidatos: Vec<&'a DiscoParaPreparar>,

    /// Os recusados, que aparecem nomeados e **nunca numerados**.
    pub recusados: Vec<(&'a DiscoParaPreparar, RecusaDaPreparacao)>,
}

impl<'a> Oferta<'a> {
    pub fn de(discos: &'a [DiscoParaPreparar], letra_do_sistema: Option<char>) -> Oferta<'a> {
        let existentes: Vec<u32> = discos.iter().map(|disco| disco.indice).collect();

        let mut oferta = Oferta {
            candidatos: Vec::new(),
            recusados: Vec::new(),
        };

        for disco in discos {
            match julgar(disco.indice, Some(disco), &existentes, letra_do_sistema) {
                Ok(_) => oferta.candidatos.push(disco),
                Err(porque) => oferta.recusados.push((disco, porque)),
            }
        }

        oferta
    }

    /// O **índice do Windows** do candidato de número `1..=n`, ou `None`.
    ///
    /// # Por que ela devolve um índice, e não o disco
    ///
    /// Porque o número escolhido no menu **não vira alvo direto**. Ele resolve
    /// para o índice que o `--dispositivo` teria dado, e daí em diante o
    /// comando percorre o caminho que já existia: julgar de novo, imprimir o
    /// plano, perguntar, **reler o disco** ([`e_o_mesmo_disco`]) e pedir o
    /// modelo digitado (S-2). O menu troca só a descoberta do número; o portão
    /// continua sendo o modelo.
    ///
    /// Devolver o `&DiscoParaPreparar` aqui seria entregar ao passo seguinte um
    /// retrato tirado antes da pergunta — que é exatamente o que a releitura de
    /// PR-4 existe para não fazer.
    ///
    /// **Com um candidato só, `1` continua sendo obrigatório.** Não há padrão,
    /// e o Enter vazio cai fora por não ser número — que é a mesma porta por
    /// onde caem `0`, `-1` e `dois`.
    pub fn escolher_pelo_numero(&self, digitado: &str) -> Option<u32> {
        // `parse::<usize>` recusa `-1` de graca, e `1..=n` fecha o resto. O `0`
        // cai fora por baixo, e e o erro mais provavel de quem conta de zero —
        // e nesta mesa o disco 0 e o do Windows.
        //
        // **`+1` ele aceita**, e vale dizer por que isso nao e furo: o
        // `usize::from_str` le o sinal `+`, entao `+1` chega aqui como o numero
        // 1 e escolhe o item 1 — o mesmo que `1` escolheria. Nao ha caminho por
        // onde um sinal mude *qual* disco sai escolhido. (O comentario gemeo em
        // `restore::escolher_pelo_indice` afirma que `+2` e recusado; medido em
        // 25/08/2026, nao e.)
        //
        // E o julgamento e o mesmo do `restore` pela mesma razao: e ele que
        // decide qual disco vai ser apagado a partir de um numero digitado.
        digitado
            .trim()
            .parse::<usize>()
            .ok()
            .filter(|numero| *numero >= 1 && *numero <= self.candidatos.len())
            .map(|numero| self.candidatos[numero - 1].indice)
    }
}

/// A conferência do terceiro tempo de PR-4: o disco relido é o mesmo do plano?
///
/// # Isto não é zelo, e a medição que o motiva é desta sessão
///
/// **O índice do Windows não é identidade.** Em 23/08/2026 o dispositivo ARCA
/// desta mesa era o disco **1**; horas depois, com um segundo SSD conectado,
/// ele passou a ser o disco **2** — e o `ARCAVAULT`, que sempre aparecera em
/// `E:`, veio em `D:`. Os índices mudam entre conexões.
///
/// Entre imprimir o plano e escrever a tabela há uma pessoa lendo a tela e
/// digitando. Nesse intervalo cabe desconectar um cabo, e o disco 1 passa a ser
/// outro. Reler e comparar **modelo e tamanho** é o que impede que o `sim`
/// dado sobre um disco seja executado sobre outro.
///
/// É a mesma família de C-3 — não acreditar no que se pediu, perguntar de novo
/// —, aplicada ao intervalo em que o ARCA não estava olhando.
pub fn e_o_mesmo_disco(planejado: &DiscoParaPreparar, relido: &DiscoParaPreparar) -> bool {
    planejado.indice == relido.indice
        && planejado.modelo == relido.modelo
        && planejado.tamanho_bytes == relido.tamanho_bytes
        && planejado.modelo_no_wmi == relido.modelo_no_wmi
}

/// A releitura de PR-5 depois de escrever: saiu o que se pediu?
///
/// Confere **a estrutura transcrita**, e não só "deu certo": os dois rótulos, os
/// dois sistemas de arquivos, os dois tipos MBR da captura, e que **nenhuma das
/// duas está ativa**.
///
/// O `IsActive` merece a conferência: a captura registra `False` nas duas, e é
/// isso que confirma que o boot é UEFI puro e não BIOS. Uma partição ativa não
/// impediria o dispositivo de bootar por UEFI, mas seria uma divergência da
/// estrutura medida — e este comando transcreve, não improvisa (ADR-0014).
pub fn conferir_o_que_saiu(feitas: &ParticoesFeitas) -> Result<(), Divergencia> {
    let mut problemas = Vec::new();

    for (particao, rotulo, sistema, tipo_mbr) in [
        (&feitas.vault, ARCAVAULT, "NTFS", TIPO_MBR_IFS),
        (&feitas.boot, ARCABOOT, "FAT32", TIPO_MBR_FAT32_LBA),
    ] {
        if particao.rotulo != rotulo {
            problemas.push(format!(
                "a particao {} devia se chamar `{rotulo}` e se chama `{}`",
                particao.numero, particao.rotulo
            ));
        }
        if !particao.sistema_de_arquivos.eq_ignore_ascii_case(sistema) {
            problemas.push(format!(
                "a particao `{rotulo}` devia ser {sistema} e e {}",
                particao.sistema_de_arquivos
            ));
        }
        if particao.tipo_mbr != tipo_mbr {
            problemas.push(format!(
                "a particao `{rotulo}` devia ter MbrType {tipo_mbr} e tem {}",
                particao.tipo_mbr
            ));
        }
        if particao.ativa {
            problemas.push(format!(
                "a particao `{rotulo}` saiu ATIVA, e a estrutura medida nao tem nenhuma ativa — o boot do dispositivo e UEFI puro"
            ));
        }
        if particao.unidade_de_alocacao != UNIDADE_DE_ALOCACAO {
            problemas.push(format!(
                "a particao `{rotulo}` saiu com unidade de alocacao {} e a medida e {UNIDADE_DE_ALOCACAO}",
                particao.unidade_de_alocacao
            ));
        }
    }

    // A ordem no disco importa: o `ARCAVAULT` vem primeiro e o `ARCABOOT` no
    // fim, como a captura registra. Trocá-las nao impediria o boot, e seria
    // outra estrutura — e o que se transcreve e esta.
    if feitas.vault.offset_bytes >= feitas.boot.offset_bytes {
        problemas.push(format!(
            "o `{ARCAVAULT}` devia vir antes do `{ARCABOOT}` no disco, e os offsets dizem o contrario ({} e {})",
            feitas.vault.offset_bytes, feitas.boot.offset_bytes
        ));
    }

    if problemas.is_empty() {
        Ok(())
    } else {
        Err(Divergencia { problemas })
    }
}

/// `MbrType 7` — IFS, que é como o Windows marca NTFS numa tabela MBR.
pub const TIPO_MBR_IFS: u32 = 7;

/// `MbrType 12` — FAT32 com endereçamento LBA (0x0C).
pub const TIPO_MBR_FAT32_LBA: u32 = 12;

/// A unidade de alocação das duas partições, transcrita da captura.
pub const UNIDADE_DE_ALOCACAO: u64 = 4096;

/// O disco não saiu como o plano pedia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergencia {
    pub problemas: Vec<String>,
}

impl fmt::Display for Divergencia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "o disco foi particionado e a releitura nao mostra a estrutura que se pediu: {}. O disco JA FOI APAGADO — o que quer que estivesse nele nao esta mais",
            self.problemas.join("; ")
        )
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::portas::particionador::{ParticaoExistente, ParticaoFeita};

    /// O disco 1 desta mesa em 23/08/2026 — o segundo dispositivo, medido
    /// antes de ser destruído de propósito.
    ///
    /// Números de verdade, e não redondos: um teste que passe com `500_000_000`
    /// e falhe com o tamanho real não está testando nada.
    fn o_ssd_da_mesa() -> DiscoParaPreparar {
        DiscoParaPreparar {
            indice: 1,
            modelo: "JMicron Generic".to_string(),
            modelo_no_wmi: Some("JMicron Generic SCSI Disk Device".to_string()),
            tamanho_bytes: 480_103_981_056,
            barramento: "USB".to_string(),
            tipo_de_midia: TipoDeMidia::DiscoExterno,
            estilo_de_particao: "MBR".to_string(),
            e_do_sistema: false,
            e_de_boot: false,
            somente_leitura: false,
            particoes: vec![ParticaoExistente {
                numero: 1,
                letra: Some('E'),
                rotulo: Some("Dell Beta Apps NO IA WSL".to_string()),
                sistema_de_arquivos: Some("NTFS".to_string()),
                tamanho_bytes: 480_099_958_784,
            }],
        }
    }

    /// O `KINGSTON SNV3S500G`, que é o disco do Windows desta máquina.
    fn o_disco_do_windows() -> DiscoParaPreparar {
        DiscoParaPreparar {
            indice: 0,
            modelo: "KINGSTON SNV3S500G".to_string(),
            modelo_no_wmi: Some("KINGSTON SNV3S500G".to_string()),
            tamanho_bytes: 500_107_862_016,
            barramento: "NVMe".to_string(),
            tipo_de_midia: TipoDeMidia::DiscoFixo,
            estilo_de_particao: "GPT".to_string(),
            e_do_sistema: true,
            e_de_boot: true,
            somente_leitura: false,
            particoes: vec![ParticaoExistente {
                numero: 3,
                letra: Some('C'),
                rotulo: Some("Windows".to_string()),
                sistema_de_arquivos: Some("NTFS".to_string()),
                tamanho_bytes: 498_701_697_024,
            }],
        }
    }

    fn julgar_o_da_mesa(disco: &DiscoParaPreparar) -> Result<Preparacao, RecusaDaPreparacao> {
        julgar(disco.indice, Some(disco), &[0, 1, 2], Some('C'))
    }

    // ─────────────────── o caminho normal ───────────────────

    #[test]
    fn o_ssd_desta_mesa_passa_pelas_sete_defesas() {
        let disco = o_ssd_da_mesa();
        let preparacao = julgar_o_da_mesa(&disco).expect("o segundo dispositivo tem de passar");

        assert_eq!(preparacao.plano.indice_do_disco, 1);
        assert_eq!(preparacao.plano.boot_bytes, 1_677_721_600);

        // O `ARCAVAULT` fica com o resto, e o resto e o disco menos o
        // `ARCABOOT` menos o alinhamento medido.
        assert_eq!(
            preparacao.plano.vault_bytes,
            480_103_981_056 - 1_677_721_600 - 2_973_696
        );
    }

    #[test]
    fn as_duas_particoes_cabem_no_disco() {
        let preparacao = julgar_o_da_mesa(&o_ssd_da_mesa()).unwrap();
        let somadas = preparacao.plano.vault_bytes + preparacao.plano.boot_bytes;

        assert!(
            somadas <= 480_103_981_056,
            "o plano pede {somadas} num disco de 480.103.981.056"
        );
    }

    // ─────────────────── as recusas duras ───────────────────

    #[test]
    fn o_disco_do_windows_e_recusa_dura() {
        let disco = o_disco_do_windows();
        let recusa = julgar(0, Some(&disco), &[0, 1, 2], Some('C')).unwrap_err();

        assert!(matches!(
            recusa,
            RecusaDaPreparacao::DiscoDoSistema {
                e_do_sistema: true,
                e_de_boot: true,
                ..
            }
        ));

        // A mensagem tem de dizer que nao ha como forcar. Uma recusa que
        // parecesse contornavel convidaria a procurar a flag que a contorna.
        assert!(
            recusa.to_string().contains("Nao ha como forcar"),
            "{recusa}"
        );
    }

    #[test]
    fn a_recusa_do_disco_do_sistema_vem_antes_da_de_tamanho() {
        // A licao de R-8, aplicada aqui. O disco do Windows desta maquina
        // tambem passaria pela defesa de tamanho; se a ordem mudasse, alguem
        // leria "o disco e pequeno demais" sobre o disco do proprio Windows e
        // trocaria por um maior.
        let mut disco = o_disco_do_windows();
        disco.tamanho_bytes = 1024; // pequeno demais, e do sistema

        assert!(matches!(
            julgar(0, Some(&disco), &[0], Some('C')),
            Err(RecusaDaPreparacao::DiscoDoSistema { .. })
        ));
    }

    #[test]
    fn o_disco_que_carrega_o_system_drive_e_recusado_mesmo_sem_issystem() {
        // O outro canal de identidade. Um disco que o `MSFT_Disk` nao marcou
        // como do sistema — porque a maquina bootou de outro — e que carrega o
        // `C:` deste Windows tem de ser recusado do mesmo jeito.
        //
        // Dois canais porque o vao entre eles e onde este projeto ja errou: a
        // revisao da E9 achou R-8 com contorno por acidente de modelo,
        // recusando por letra o que a receita nomeava por nome do Linux.
        let mut disco = o_disco_do_windows();
        disco.e_do_sistema = false;
        disco.e_de_boot = false;
        disco.tipo_de_midia = TipoDeMidia::DiscoExterno;

        assert_eq!(
            julgar(0, Some(&disco), &[0, 1], Some('C')),
            Err(RecusaDaPreparacao::CarregaOSystemDrive {
                modelo: "KINGSTON SNV3S500G".to_string(),
                letra: 'C',
            })
        );
    }

    #[test]
    fn disco_fixo_e_recusa_e_nao_ha_flag_que_libere() {
        let mut disco = o_ssd_da_mesa();
        disco.tipo_de_midia = TipoDeMidia::DiscoFixo;

        let recusa = julgar_o_da_mesa(&disco).unwrap_err();
        assert!(matches!(
            recusa,
            RecusaDaPreparacao::MidiaNaoRemovivel { .. }
        ));
        assert!(
            recusa.to_string().contains("Nao ha como forcar"),
            "{recusa}"
        );
    }

    #[test]
    fn midia_desconhecida_recusa_junto_com_fixo() {
        // Supor que o desconhecido e externo faria a defesa passar batido
        // justamente no caso em que ela mais importa. E o mesmo raciocinio do
        // leitor do WMI, que nunca transforma `Desconhecido` em `DiscoFixo`.
        let mut disco = o_ssd_da_mesa();
        disco.tipo_de_midia = TipoDeMidia::Desconhecido;

        let recusa = julgar_o_da_mesa(&disco).unwrap_err();
        assert!(matches!(
            recusa,
            RecusaDaPreparacao::MidiaNaoRemovivel {
                tipo: TipoDeMidia::Desconhecido,
                ..
            }
        ));
        assert!(
            recusa.to_string().contains("nao sei"),
            "a mensagem tem de dizer por que o desconhecido recusa: {recusa}"
        );
    }

    #[test]
    fn midia_removivel_de_verdade_passa() {
        // Um pen drive e `Removable Media`, e PR-5 aceita os dois. O que o
        // `bcdedit` rejeita em silencio (C-6) e outra pergunta, e ela e do
        // armar — aqui o que se decide e se da para particionar.
        let mut disco = o_ssd_da_mesa();
        disco.tipo_de_midia = TipoDeMidia::Removivel;

        assert!(julgar_o_da_mesa(&disco).is_ok());
    }

    #[test]
    fn disco_pequeno_demais_e_recusado_com_o_numero_na_mensagem() {
        let mut disco = o_ssd_da_mesa();
        disco.tamanho_bytes = 8 * 1024 * 1024 * 1024; // um pen drive de 8 GB

        let recusa = julgar_o_da_mesa(&disco).unwrap_err();
        match &recusa {
            RecusaDaPreparacao::DiscoPequenoDemais { tem, precisa, .. } => {
                assert_eq!(*tem, 8 * 1024 * 1024 * 1024);
                assert_eq!(*precisa, ARCABOOT_BYTES + ARCAVAULT_MINIMO_BYTES);
            }
            outro => panic!("esperava recusa por tamanho, veio {outro:?}"),
        }
        assert!(recusa.to_string().contains("8,0 GB"), "{recusa}");
    }

    #[test]
    fn disco_somente_leitura_e_recusado() {
        let mut disco = o_ssd_da_mesa();
        disco.somente_leitura = true;

        assert!(matches!(
            julgar_o_da_mesa(&disco),
            Err(RecusaDaPreparacao::SomenteLeitura { .. })
        ));
    }

    #[test]
    fn indice_que_nao_existe_nomeia_os_que_existem() {
        let recusa = julgar(9, None, &[0, 1, 2], Some('C')).unwrap_err();

        assert_eq!(
            recusa,
            RecusaDaPreparacao::DiscoDesconhecido {
                indice: 9,
                existentes: vec![0, 1, 2]
            }
        );
        assert!(recusa.to_string().contains("0, 1, 2"), "{recusa}");
    }

    #[test]
    fn sem_letra_do_sistema_a_defesa_do_system_drive_nao_dispara_nem_engole() {
        // "Nao consegui descobrir onde o Windows mora" nao pode virar "este
        // disco nao e o dele" — mas tambem nao pode recusar tudo. As outras
        // seis defesas continuam valendo, e o `IsSystem` do MSFT_Disk cobre o
        // caso principal.
        let disco = o_ssd_da_mesa();
        assert!(julgar(1, Some(&disco), &[0, 1], None).is_ok());

        let do_windows = o_disco_do_windows();
        assert!(julgar(0, Some(&do_windows), &[0, 1], None).is_err());
    }

    // ─────────────────── o terceiro tempo de PR-4 ───────────────────

    #[test]
    fn o_disco_relido_tem_de_ser_o_mesmo_do_plano() {
        let planejado = o_ssd_da_mesa();
        assert!(e_o_mesmo_disco(&planejado, &planejado.clone()));
    }

    #[test]
    fn trocar_o_cabo_entre_o_plano_e_o_sim_e_pego_pela_conferencia() {
        // **Medido nesta sessao**: o dispositivo ARCA desta mesa era o disco 1
        // em 23/08 e virou o disco 2 horas depois, com um segundo SSD
        // conectado. O indice do Windows nao e identidade.
        //
        // Entre imprimir o plano e escrever a tabela ha uma pessoa lendo e
        // digitando. Nesse intervalo cabe desconectar um cabo — e o disco 1
        // passa a ser outro disco, com o `sim` ja dado.
        let planejado = o_ssd_da_mesa();

        let mut outro = planejado.clone();
        outro.modelo = "KGSSE100 256".to_string();
        outro.tamanho_bytes = 256_060_514_304;
        assert!(!e_o_mesmo_disco(&planejado, &outro));

        // E um disco do mesmo modelo com outro tamanho tambem nao passa.
        let mut gemeo_maior = planejado.clone();
        gemeo_maior.tamanho_bytes += 1;
        assert!(!e_o_mesmo_disco(&planejado, &gemeo_maior));

        // Nem um do mesmo tamanho com outro modelo.
        let mut mesmo_tamanho = planejado.clone();
        mesmo_tamanho.modelo = "OUTRO".to_string();
        assert!(!e_o_mesmo_disco(&planejado, &mesmo_tamanho));
    }

    // ─────────────────── a oferta do menu, e o que ela não deduz ───────────────────

    /// Os três discos desta mesa em 23/08/2026: o disco do Windows e dois
    /// externos.
    fn os_discos_da_mesa() -> Vec<DiscoParaPreparar> {
        let mut o_outro_externo = o_ssd_da_mesa();
        o_outro_externo.indice = 2;
        o_outro_externo.modelo = "KGSSE100 256".to_string();
        o_outro_externo.tamanho_bytes = 256_060_514_304;

        vec![o_disco_do_windows(), o_ssd_da_mesa(), o_outro_externo]
    }

    #[test]
    fn nenhum_disco_desaparece_da_tela() {
        // A invariante que sustenta a decisao de listar os recusados: **todo
        // disco esta de um dos dois lados**. Um disco que caisse fora dos dois
        // sumiria da tela, e sumir e o modo de falha que a lista do
        // `arca restore` ja pagou uma vez para aprender — quem sabe que ha
        // outro disco na mesa conclui que o ARCA nao o enxerga, e vai procurar
        // como forcar.
        let discos = os_discos_da_mesa();
        let oferta = Oferta::de(&discos, Some('C'));

        assert_eq!(
            oferta.candidatos.len() + oferta.recusados.len(),
            discos.len()
        );
    }

    #[test]
    fn o_disco_do_windows_aparece_recusado_e_nunca_como_candidato() {
        let discos = os_discos_da_mesa();
        let oferta = Oferta::de(&discos, Some('C'));

        assert_eq!(
            oferta
                .candidatos
                .iter()
                .map(|d| d.indice)
                .collect::<Vec<_>>(),
            vec![1, 2],
            "os dois externos sao os candidatos"
        );

        let (recusado, porque) = &oferta.recusados[0];
        assert_eq!(recusado.indice, 0);
        assert!(
            matches!(porque, RecusaDaPreparacao::DiscoDoSistema { .. }),
            "{porque}"
        );

        // E o motivo curto diz por que, sem despejar o paragrafo inteiro da
        // recusa terminal em cima da lista.
        let resumo = porque.resumo();
        assert!(resumo.contains("disco do sistema"), "{resumo}");
        assert!(
            resumo.len() < porque.to_string().len(),
            "o resumo nao e mais curto do que o Display: {resumo}"
        );
    }

    #[test]
    fn o_numero_do_menu_nao_e_o_indice_do_windows() {
        // **A coincidencia que ensina errado.** Nesta mesa o `[1]` e o disco 1
        // e os dois batem; tire o disco 1 da mesa e o `[1]` passa a ser o
        // disco 2. Quem tivesse aprendido a ler o numero da esquerda como
        // indice apontaria para o disco errado — e por isso a tela imprime os
        // dois, e por isso este teste existe.
        let discos = vec![os_discos_da_mesa()[2].clone()];
        let oferta = Oferta::de(&discos, Some('C'));

        assert_eq!(oferta.escolher_pelo_numero("1"), Some(2));
    }

    #[test]
    fn com_um_candidato_so_o_enter_vazio_nao_escolhe_nada() {
        // O §6.1 escreve como principio: *"obrigatorio, mesmo havendo um
        // candidato so"*. Um menu de um item que se aceita com Enter e
        // exatamente o ARCA escolhendo o que apagar — que e o que P1 revisado
        // proibe, com menu ou sem menu.
        let discos = vec![o_ssd_da_mesa()];
        let oferta = Oferta::de(&discos, Some('C'));

        assert_eq!(oferta.candidatos.len(), 1);
        assert_eq!(oferta.escolher_pelo_numero(""), None);
        assert_eq!(oferta.escolher_pelo_numero("  "), None);
        assert_eq!(oferta.escolher_pelo_numero("\r\n"), None);

        // E o `1` digitado escolhe, que e o outro lado da mesma regra: nao ha
        // padrao, e ha uma escolha possivel.
        assert_eq!(oferta.escolher_pelo_numero("1"), Some(1));
    }

    #[test]
    fn o_que_nao_e_numero_da_lista_nao_escolhe() {
        // Lista de permissao, como B-2 e pelo mesmo motivo: o que nao esta aqui
        // e **nao**. O `0` e o erro mais provavel de quem conta de zero — e
        // nesta mesa o disco 0 e o do Windows, o que torna aceita-lo a pior
        // ideia possivel.
        let discos = os_discos_da_mesa();
        let oferta = Oferta::de(&discos, Some('C'));

        for digitado in ["0", "-1", "3", "99", "um", "1.0", "1 2", "s", "sim"] {
            assert_eq!(
                oferta.escolher_pelo_numero(digitado),
                None,
                "`{digitado}` escolheu alguma coisa"
            );
        }

        // `+1` **e aceito**, e esta aqui escrito porque o comentario gemeo do
        // `arca restore` diz o contrario: o `usize::from_str` le o sinal, entao
        // `+1` e o numero 1. O que importa e que ele escolha o mesmo item que
        // `1` escolheria — um sinal nao muda *qual* disco sai da lista.
        assert_eq!(
            oferta.escolher_pelo_numero("+1"),
            oferta.escolher_pelo_numero("1")
        );
    }

    #[test]
    fn o_enter_deixa_para_tras_e_nao_atrapalha() {
        let discos = os_discos_da_mesa();
        let oferta = Oferta::de(&discos, Some('C'));

        assert_eq!(oferta.escolher_pelo_numero("2\r\n"), Some(2));
        assert_eq!(oferta.escolher_pelo_numero("  1  "), Some(1));
    }

    #[test]
    fn sem_candidato_nenhum_nao_ha_o_que_escolher() {
        // Uma maquina com um disco so, o do Windows. Nada a oferecer, e
        // nenhum numero escolhe coisa alguma.
        let discos = vec![o_disco_do_windows()];
        let oferta = Oferta::de(&discos, Some('C'));

        assert!(oferta.candidatos.is_empty());
        assert_eq!(oferta.recusados.len(), 1);
        assert_eq!(oferta.escolher_pelo_numero("1"), None);
    }

    #[test]
    fn a_oferta_julga_cada_disco_pelas_mesmas_sete_defesas() {
        // O menu **nao** e um segundo julgamento com regras proprias — ele roda
        // o `julgar` em cada disco. Duas listas de defesas divergiriam na
        // primeira mudanca, e a que ficasse para tras ofereceria um disco que a
        // outra recusa.
        //
        // O oraculo aqui e o proprio `julgar`: o que ele aceita e candidato, o
        // que ele recusa e recusado, disco a disco.
        let discos = os_discos_da_mesa();
        let oferta = Oferta::de(&discos, Some('C'));
        let existentes: Vec<u32> = discos.iter().map(|disco| disco.indice).collect();

        for disco in &discos {
            let veredito = julgar(disco.indice, Some(disco), &existentes, Some('C'));
            let na_oferta = oferta
                .candidatos
                .iter()
                .any(|candidato| candidato.indice == disco.indice);

            assert_eq!(
                veredito.is_ok(),
                na_oferta,
                "o disco {} caiu do lado errado da oferta",
                disco.indice
            );
        }
    }

    #[test]
    fn a_letra_do_sistema_tira_o_disco_da_lista_em_vez_de_deixa_lo_passar() {
        // A defesa 3 vale dentro do menu como vale fora dele. Um externo que
        // carregue o `%SystemDrive%` — a maquina com dois Windows — sai da
        // lista com o motivo dito, e nao entra como candidato.
        let discos = vec![o_ssd_da_mesa()];

        let oferta = Oferta::de(&discos, Some('E'));
        assert!(
            oferta.candidatos.is_empty(),
            "o disco do E: virou candidato"
        );
        assert!(matches!(
            oferta.recusados[0].1,
            RecusaDaPreparacao::CarregaOSystemDrive { .. }
        ));

        // E sem a letra do sistema o mesmo disco passa — a defesa nao engole
        // nem inventa.
        assert_eq!(Oferta::de(&discos, None).candidatos.len(), 1);
    }

    // ─────────────────── a releitura depois de escrever ───────────────────

    /// O que a medição de 23/08/2026 leu do disco depois de particionar.
    fn o_que_saiu() -> ParticoesFeitas {
        ParticoesFeitas {
            vault: ParticaoFeita {
                numero: 1,
                letra: 'E',
                rotulo: ARCAVAULT.to_string(),
                sistema_de_arquivos: "NTFS".to_string(),
                tipo_mbr: 7,
                tamanho_bytes: 478_423_285_760,
                offset_bytes: 1_048_576,
                unidade_de_alocacao: 4096,
                ativa: false,
            },
            boot: ParticaoFeita {
                numero: 2,
                letra: 'F',
                rotulo: ARCABOOT.to_string(),
                sistema_de_arquivos: "FAT32".to_string(),
                tipo_mbr: 12,
                tamanho_bytes: 1_677_721_600,
                offset_bytes: 478_424_334_336,
                unidade_de_alocacao: 4096,
                ativa: false,
            },
        }
    }

    #[test]
    fn a_estrutura_medida_em_hardware_passa_na_conferencia() {
        // O oraculo desta etapa: estes numeros sao os que o Windows respondeu
        // em 23/08/2026, depois de o particionamento rodar a mao no segundo
        // dispositivo. O teste nao pode ser ajustado para passar.
        assert_eq!(conferir_o_que_saiu(&o_que_saiu()), Ok(()));
    }

    #[test]
    fn os_tipos_mbr_sao_os_da_captura_e_nao_os_que_o_new_partition_cria() {
        // **Medido**: o `New-Partition` cria as duas com `MbrType 6`, e quem
        // acerta para 7 e 12 e o `Format-Volume`. O tipo e efeito colateral de
        // outra operacao, e por isso a releitura importa: nada no caminho o
        // pede diretamente.
        let mut cru = o_que_saiu();
        cru.vault.tipo_mbr = 6;
        cru.boot.tipo_mbr = 6;

        let divergencia = conferir_o_que_saiu(&cru).unwrap_err();
        assert_eq!(divergencia.problemas.len(), 2);
        assert!(divergencia.to_string().contains("MbrType 7"));
        assert!(divergencia.to_string().contains("MbrType 12"));
    }

    #[test]
    fn uma_particao_ativa_e_divergencia() {
        // A captura registra `IsActive: False` nas duas, e e isso que confirma
        // que o boot do dispositivo e UEFI puro e nao BIOS. Uma ativa nao
        // impediria o boot — seria outra estrutura, e este comando transcreve.
        let mut com_ativa = o_que_saiu();
        com_ativa.boot.ativa = true;

        let divergencia = conferir_o_que_saiu(&com_ativa).unwrap_err();
        assert!(divergencia.to_string().contains("ATIVA"), "{divergencia}");
    }

    #[test]
    fn os_rotulos_trocados_sao_divergencia() {
        let mut trocados = o_que_saiu();
        std::mem::swap(&mut trocados.vault.rotulo, &mut trocados.boot.rotulo);

        assert!(conferir_o_que_saiu(&trocados).is_err());
    }

    #[test]
    fn o_arcaboot_antes_do_arcavault_e_divergencia() {
        // A captura poe o `ARCAVAULT` no comeco e o `ARCABOOT` no fim. Trocados
        // de lugar o dispositivo provavelmente bootaria igual — e seria outra
        // estrutura, e o ADR-0014 decidiu transcrever a que boota aqui.
        let mut invertidas = o_que_saiu();
        std::mem::swap(
            &mut invertidas.vault.offset_bytes,
            &mut invertidas.boot.offset_bytes,
        );

        let divergencia = conferir_o_que_saiu(&invertidas).unwrap_err();
        assert!(
            divergencia.to_string().contains("antes do"),
            "{divergencia}"
        );
    }

    #[test]
    fn a_unidade_de_alocacao_e_conferida_nas_duas() {
        let mut outra = o_que_saiu();
        outra.vault.unidade_de_alocacao = 512;

        assert!(
            conferir_o_que_saiu(&outra)
                .unwrap_err()
                .to_string()
                .contains("unidade de alocacao 512")
        );
    }

    #[test]
    fn a_divergencia_diz_que_o_disco_ja_foi_apagado() {
        // A pior mensagem possivel aqui seria uma que parecesse uma recusa
        // preventiva. Quem lê isto precisa saber que o disco JA foi.
        let mut errada = o_que_saiu();
        errada.vault.rotulo = "OUTRA COISA".to_string();

        assert!(
            conferir_o_que_saiu(&errada)
                .unwrap_err()
                .to_string()
                .contains("JA FOI APAGADO")
        );
    }

    #[test]
    fn um_sistema_de_arquivos_em_outra_caixa_nao_e_divergencia() {
        // O Windows responde `NTFS` e `FAT32`; um dia pode responder `ntfs`. A
        // caixa nao muda o que o sistema de arquivos e, e reprovar por isso
        // faria o comando falhar depois de ter apagado o disco — que e o pior
        // momento possivel para uma falha cosmetica.
        let mut minusculo = o_que_saiu();
        minusculo.vault.sistema_de_arquivos = "ntfs".to_string();
        minusculo.boot.sistema_de_arquivos = "fat32".to_string();

        assert_eq!(conferir_o_que_saiu(&minusculo), Ok(()));
    }
}
