//! A porta que particiona e formata o dispositivo (PR-5).
//!
//! Nasceu na etapa E10, quando P1 foi revisado e o ARCA passou a criar as duas
//! partições em vez de exigi-las prontas
//! ([ADR-0014](../../docs/adr/0014-o-arca-particiona-o-dispositivo.md)).
//!
//! # Por que uma porta nova, e não campos novos em [`crate::portas::Discos`]
//!
//! Porque as duas respondem perguntas diferentes, e uma delas **escreve**.
//! `Discos` é leitura de metadado e sete comandos dependem dela; acrescentar
//! `IsSystem`, `IsBoot`, o estilo da tabela e a lista de partições a
//! [`crate::portas::DiscoFisico`] carregaria em todos eles o que só o `arca
//! prepare` usa. E uma porta que **cria partição** pendurada no tipo que o
//! `arca backup` recebe é um convite a alguém chamar o que não devia.
//!
//! A regra do [`crate::portas`] é a de sempre: cada fronteira entra quando
//! alguma etapa precisa dela. Esta é a quinta.
//!
//! # S-1 continua valendo, e vale a pena dizer por quê
//!
//! Nenhuma assinatura daqui entrega handle de dispositivo, caminho de
//! dispositivo bruto nem deslocamento em setores. O que atravessa são
//! **tamanhos em bytes, rótulos e sistemas de arquivos** — e quem escreve a
//! tabela de partição é o próprio Windows, pelos mesmos cmdlets que o
//! Gerenciamento de Disco usa. É a mesma categoria do `chkdsk` de B-6 e do
//! `powercfg` de B-5, que a correção D5 do plano já delimitou: S-1 é sobre
//! **acesso raw ao dispositivo**, e não sobre pedir ao sistema que faça uma
//! operação dele.
//!
//! O que muda em relação às outras quatro portas é o **tamanho do estrago
//! quando o alvo está errado** — e é disso que as sete defesas de PR-5 tratam,
//! do lado de cá da fronteira, em [`crate::preparacao`].
//!
//! # B-10 não é furado, e a distinção é a mesma do `bootsequence`
//!
//! `src/desarme.rs` já defendeu que apagar o `bootsequence` não fura B-10,
//! porque B-10 fala do que o **usuário perderia** — imagem, resíduo, log — e a
//! marca de boot único é uma intenção que o próprio ARCA gravou.
//!
//! Aqui a distinção é outra e mais direta: **B-10 governa o que o ARCA faz com
//! o conteúdo de um dispositivo ARCA**, e `arca prepare` age sobre um disco que
//! ainda não é um. O que ele destrói foi nomeado pelo usuário, impresso na tela
//! antes (PR-4) e confirmado por escrito (S-2) — que é P1 revisado na letra.
//! `tests/b10_nada_e_apagado.rs` varre o código atrás de exclusão de **arquivo**
//! e não distingue os dois casos, daí valer deixar isto escrito.

use crate::erro::Resultado;

/// Uma partição que **já existe** no disco, para a tela de PR-4.
///
/// Quem vai perder dados tem de poder reconhecê-los antes: rótulo, sistema de
/// arquivos e tamanho, que é o que uma pessoa usa para dizer "ah, é aquele".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticaoExistente {
    pub numero: u32,
    pub letra: Option<char>,

    /// O rótulo do volume, quando há um formatado ali. `None` numa partição
    /// crua ou num sistema de arquivos que o Windows não monta.
    pub rotulo: Option<String>,

    /// `NTFS`, `FAT32`, ... `None` quando o Windows não reconhece.
    pub sistema_de_arquivos: Option<String>,

    pub tamanho_bytes: u64,
}

/// Um disco visto com os olhos do `arca prepare`.
///
/// Traz o que as sete defesas de PR-5 precisam, e nada além. Note que
/// `IsSystem` e `IsBoot` **não** estão em [`crate::portas::DiscoFisico`]: o
/// `Win32_DiskDrive` não os responde, e quem os responde é o `MSFT_Disk` — a
/// mesma fonte da régua de R-7 (ADR-0010).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoParaPreparar {
    pub indice: u32,

    /// O `FriendlyName` do `MSFT_Disk`. É o que a confirmação digitada de S-2
    /// pede, e por isso ele aparece na tela exatamente como o Windows o
    /// escreve.
    pub modelo: String,

    /// O `Model` do `Win32_DiskDrive`, que **não é o mesmo texto**. Nesta mesa
    /// o `MSFT_Disk` diz `JMicron Generic` e o WMI diz `JMicron Generic SCSI
    /// Disk Device`. Guardar os dois evita que a tela afirme um e a defesa
    /// julgue o outro sem que ninguém veja.
    pub modelo_no_wmi: Option<String>,

    /// O tamanho pelo `MSFT_Disk` — a régua boa (ADR-0010).
    pub tamanho_bytes: u64,

    /// `USB`, `NVMe`, `SATA`...
    pub barramento: String,

    /// O `MediaType` do `Win32_DiskDrive`, que é onde moram as palavras
    /// `External hard disk media` e `Removable Media` (§3.1, D10). É a
    /// primeira das sete defesas.
    pub tipo_de_midia: crate::portas::TipoDeMidia,

    /// `MBR`, `GPT` ou `RAW`. Informativo — o `arca prepare` reescreve a
    /// tabela de qualquer jeito.
    pub estilo_de_particao: String,

    /// O disco onde o Windows está. Recusa dura, sem opção de forçar.
    pub e_do_sistema: bool,

    /// O disco por onde a máquina bootou. Recusa dura.
    pub e_de_boot: bool,

    pub somente_leitura: bool,

    /// O que existe hoje no disco, e vai ser destruído (PR-4).
    pub particoes: Vec<ParticaoExistente>,
}

impl DiscoParaPreparar {
    /// As letras que este disco carrega hoje, em maiúscula.
    ///
    /// Serve para a defesa que compara o alvo com o `%SystemDrive%` — e para
    /// que a tela possa dizer `E:` em vez de "a partição 1".
    pub fn letras(&self) -> Vec<char> {
        self.particoes
            .iter()
            .filter_map(|particao| particao.letra)
            .map(|letra| letra.to_ascii_uppercase())
            .collect()
    }
}

/// O que ficar no disco depois: duas partições, com rótulo e sistema de
/// arquivos.
///
/// É a estrutura **transcrita** de
/// `recursos/capturas/medicao-gpt-2026-08-25.txt`, e não uma inventada — ver
/// [`crate::preparacao`] para o plano e o
/// [ADR-0025](../../docs/adr/0025-o-arca-particiona-em-gpt.md) para por que
/// GPT com duas partições de dados, e não MBR nem GPT+ESP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanoDeParticoes {
    pub indice_do_disco: u32,

    /// O tamanho do `ARCAVAULT`, a primeira partição, em bytes.
    pub vault_bytes: u64,

    /// O tamanho do `ARCABOOT`, a segunda, no fim do disco.
    pub boot_bytes: u64,
}

/// O que o particionamento deixou, **relido do disco** (C-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticoesFeitas {
    pub vault: ParticaoFeita,
    pub boot: ParticaoFeita,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticaoFeita {
    pub numero: u32,
    pub letra: char,
    pub rotulo: String,
    pub sistema_de_arquivos: String,

    /// O `GptType` que o Windows escreveu, como ele o devolve: entre chaves e
    /// em minúsculas.
    ///
    /// **É o mesmo nas duas** —
    /// [`crate::preparacao::TIPO_GPT_DADOS_BASICOS`] —, e essa é a diferença
    /// que mais muda em relação ao MBR, onde `7` e `12` distinguiam a
    /// `ARCAVAULT` da `ARCABOOT`. Conferi-lo continua valendo: ele descarta uma
    /// ESP, uma MSR ou o que o Windows tivesse criado por conta. O que ele
    /// deixou de fazer é dizer **qual é qual**, e quem faz isso é o rótulo, o
    /// sistema de arquivos e a ordem no disco — que já eram conferidos.
    ///
    /// Medido em 25/08/2026 nos dois dispositivos do marco: **o `GptType` sai
    /// pronto do `New-Partition`, e o `Format-Volume` não encosta nele.** É o
    /// contrário do MBR, em que o tipo era efeito colateral do formato. Ver o
    /// [ADR-0025](../../docs/adr/0025-o-arca-particiona-em-gpt.md).
    pub tipo_gpt: String,

    pub tamanho_bytes: u64,
    pub offset_bytes: u64,
    pub unidade_de_alocacao: u64,
    pub ativa: bool,
}

pub trait Particionador {
    /// Descreve um disco pelo índice do Windows, com o que as sete defesas
    /// precisam.
    ///
    /// `None` quando não há disco com aquele índice — e "não há" é uma
    /// resposta, não um erro: o usuário digitou um número que não existe, e a
    /// recusa que ele merece diz quais existem.
    fn descrever(&self, indice: u32) -> Resultado<Option<DiscoParaPreparar>>;

    /// Todos os discos, para a recusa nomear os índices que existem.
    fn enumerar(&self) -> Resultado<Vec<DiscoParaPreparar>>;

    /// **Apaga a tabela de partição, cria as duas e as formata.** O ponto sem
    /// volta do `arca prepare`.
    ///
    /// Devolve o que ficou, **relido do disco** — nunca o que se pediu. É C-3
    /// aplicado a outra ferramenta: o `New-Partition` e o `Format-Volume` não
    /// são o `bcdedit`, e a regra de não acreditar em código de saída já provou
    /// o seu valor três vezes neste projeto.
    fn particionar(&self, plano: &PlanoDeParticoes) -> Resultado<ParticoesFeitas>;
}
