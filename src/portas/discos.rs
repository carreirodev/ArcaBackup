//! A porta da enumeracao de discos.
//!
//! Le metadado, nunca conteudo. O dispositivo se acha pelos rotulos
//! `ARCABOOT` e `ARCAVAULT` — nunca por letra, `sda` ou numero de serie
//! (S-3) —, e a letra que aparece em [`Volume`] serve para montar caminho de
//! arquivo do lado Windows, jamais para enderecar destino de receita.

use crate::erro::Resultado;

/// Como o Windows classifica a midia. A distincao importa porque o `bcdedit`
/// **rejeita `Removable Media` em silencio** — responde "êxito" e mantem o
/// valor antigo (C-6). Um pendrive nunca serve de dispositivo ARCA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoDeMidia {
    /// `External hard disk media` — o que o `bcdedit` aceita.
    DiscoExterno,
    /// `Removable Media` — recusado por C-6.
    Removivel,
    DiscoFixo,
    Desconhecido,
}

/// Uma particao montada, vista do lado Windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Volume {
    /// O rotulo — `ARCABOOT` ou `ARCAVAULT` num dispositivo ARCA.
    pub rotulo: Option<String>,
    /// A letra atribuida pelo Windows, quando ha uma.
    pub letra: Option<char>,
    pub sistema_de_arquivos: String,
    pub total_bytes: u64,
    pub livre_bytes: u64,
    pub tipo_de_midia: TipoDeMidia,
}

/// O tamanho de um disco na regua que casa com a GPT de dentro de uma imagem
/// (R-7).
///
/// # Por que este campo existe ao lado de `tamanho_bytes`
///
/// Sao dois numeros do **mesmo** disco, e eles nao batem. Medido nesta maquina
/// em 23/08/2026, para o `KINGSTON SNV3S500G`:
///
/// | Fonte | Bytes |
/// |---|---|
/// | `MSFT_Disk.Size` — este campo | 500.107.862.016 |
/// | `Win32_DiskDrive.Size` — [`DiscoFisico::tamanho_bytes`] | 500.105.249.280 |
///
/// O segundo e `60801 x 255 x 63 x 512`: o produto da geometria CHS legada,
/// truncado no ultimo cilindro inteiro. Fica 5.103 setores atras, que e menos
/// de um cilindro (16.065) — a assinatura do truncamento. O primeiro bate
/// **byte a byte** com o `nvme0n1-gpt.sgdisk` que a imagem carrega.
///
/// R-7 compara o destino com a origem, e a origem so existe dentro da imagem
/// ([`crate::gpt`]). Medir uma ponta pela GPT e a outra pelo `Win32_DiskDrive`
/// faria o mesmo disco aparecer 2,6 MB menor do que ele proprio. Por isso a
/// medida certa entra como campo novo em vez de substituir o antigo: B-4
/// continua no `tamanho_bytes`, onde superestimar o em uso e o lado seguro de
/// "cabe uma imagem?", e R-7 usa este.
///
/// `None` quando o `MSFT_Disk` nao respondeu por aquele disco. **Nunca cai de
/// volta no `Win32_DiskDrive`**: "nao consegui medir" e uma resposta, e R-7
/// recusa em cima dela. Ver
/// `docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Medida {
    pub bytes: u64,

    /// O setor **logico** — o que o `MSFT_Disk` chama de `LogicalSectorSize`,
    /// e o primeiro numero do `Sector size (logical/physical)` do `sgdisk`. E
    /// a unidade em que a tabela de particao e escrita.
    pub bytes_por_setor: u64,
}

impl Medida {
    pub fn setores(&self) -> u64 {
        if self.bytes_por_setor == 0 {
            0
        } else {
            self.bytes / self.bytes_por_setor
        }
    }
}

/// Um disco fisico, pelo que o Windows sabe dele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoFisico {
    /// O indice que o Windows da ao disco. Serve para distinguir dois discos
    /// do mesmo modelo, e **nao** para derivar o nome que o Linux lhe da: o
    /// indice do Windows nao e o do Linux por construcao. Ver
    /// [`crate::blkdev`].
    pub indice: u32,

    pub modelo: String,
    pub tamanho_bytes: u64,

    /// Quanto do **disco** esta em uso — a base da regra de espaco de B-4.
    ///
    /// Contado como `tamanho do disco menos o livre nos volumes com letra`, e
    /// nao como a soma do que os volumes com letra usam. A diferenca importa
    /// e foi medida: o disco desta maquina tem quatro particoes e so o `C:`
    /// tem letra. As outras tres — ESP de 300 M, MSR de 16 M e recuperacao de
    /// 1 G — somam cerca de 1,3 GB que a soma por volume simplesmente ignora.
    /// Pior, o `Win32_DiskPartition` nem enxerga a MSR.
    ///
    /// Contando assim, tudo que nao esta livre num volume com letra conta como
    /// em uso. Isso **superestima** — a ESP nao esta cheia —, e superestimar e
    /// o lado seguro de uma regra que responde "cabe uma imagem?". E, o que
    /// vale mais: o nome do campo passa a ser verdade.
    pub em_uso_bytes: u64,

    /// Como o Windows classifica a midia deste disco.
    ///
    /// Aqui a distincao **existe de verdade**, ao contrario do que sai do
    /// `GetDriveType` em [`Volume::tipo_de_midia`]: o `MediaType` do WMI
    /// responde literalmente `External hard disk media` para o SSD externo e
    /// `Fixed hard disk media` para o interno. Sao as palavras da §3.1 do PRD,
    /// que o `bcdedit` nao produz (D10) — elas saem daqui. E o sinal
    /// antecipado de C-6, muito melhor do que o `DiscoFixo` que o
    /// `GetDriveType` devolve para os dois.
    pub tipo_de_midia: TipoDeMidia,

    /// O tamanho na regua que casa com a GPT da imagem (R-7). Ver [`Medida`]
    /// para por que ele nao substitui o [`DiscoFisico::tamanho_bytes`].
    pub medida: Option<Medida>,

    /// As letras dos volumes que moram neste disco.
    ///
    /// E o que fecha a pendencia de [`crate::dispositivo::Dispositivo::boot`]:
    /// C-10 recusa rotulo repetido, e nao rotulo orfao, e ate a E6 nada provava
    /// que o `ARCAVAULT` e o `ARCABOOT` encontrados eram do mesmo dispositivo
    /// fisico. Com as letras por disco, prova-se.
    pub letras: Vec<char>,
}

impl DiscoFisico {
    pub fn tem_a_letra(&self, letra: char) -> bool {
        self.letras
            .iter()
            .any(|sua| sua.eq_ignore_ascii_case(&letra))
    }
}

pub trait Discos {
    fn volumes(&self) -> Resultado<Vec<Volume>>;

    /// Os discos fisicos e o que mora em cada um.
    ///
    /// Custa uma consulta ao WMI, que leva cerca de dois segundos nesta
    /// maquina — por isso quem so precisa listar imagens nao chama.
    fn discos_fisicos(&self) -> Resultado<Vec<DiscoFisico>>;
}
