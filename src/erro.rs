//! O erro do ARCA.
//!
//! Toda variante existe para produzir uma mensagem propria: nenhum desfecho
//! do ARCA e silencio (secao 5.5 do PRD). Erros de infraestrutura carregam a
//! operacao que falhou junto do motivo, porque "acesso negado" sozinho nao
//! diz a quem lê o que estava sendo tentado.

use std::path::PathBuf;

pub type Resultado<T> = std::result::Result<T, Erro>;

#[derive(Debug, thiserror::Error)]
pub enum Erro {
    /// O comando existe na superficie da linha de comando, mas a etapa que o
    /// entrega ainda nao foi construida. Nomear a etapa evita que isto seja
    /// confundido com uma falha.
    #[error("`arca {comando}` chega na etapa {etapa}; a fundacao ja esta de pe")]
    AindaNaoImplementado {
        comando: &'static str,
        etapa: &'static str,
    },

    /// O UAC foi recusado ou fechado. Nao e falha do ARCA, e uma decisao do
    /// usuario, e merece mensagem propria.
    #[error(
        "elevacao recusada: o ARCA escreve no grub.cfg e fala com o bcdedit, e nenhuma das duas coisas roda sem privilegio administrativo"
    )]
    ElevacaoRecusada,

    #[error("nao foi possivel relancar o ARCA com elevacao: {0}")]
    FalhaAoElevar(String),

    #[error("nao foi possivel descobrir o caminho do proprio executavel: {0}")]
    ExecutavelDesconhecido(std::io::Error),

    #[error("{operacao} falhou em {caminho}: {origem}")]
    Arquivo {
        operacao: &'static str,
        caminho: PathBuf,
        #[source]
        origem: std::io::Error,
    },

    #[error("{ferramenta} falhou: {origem}")]
    Ferramenta {
        ferramenta: &'static str,
        #[source]
        origem: std::io::Error,
    },

    /// A ferramenta rodou e recusou o pedido. Distinta de [`Erro::Ferramenta`],
    /// que e nao ter conseguido roda-la.
    ///
    /// Existe porque o `bcdedit` escreve "Acesso negado" na **saida padrao** e
    /// sai com codigo 1: quem so lesse o texto acharia que leu uma
    /// configuracao de boot vazia, e concluiria que nao ha entrada `ARCA`
    /// onde na verdade nao houve permissao para olhar.
    #[error("{ferramenta} recusou (codigo {codigo}): {saida}")]
    FerramentaRecusou {
        ferramenta: &'static str,
        codigo: i32,
        saida: String,
    },

    #[error("nao foi possivel saber se este processo esta elevado: {0}")]
    ElevacaoIndeterminada(String),

    /// Nenhum volume respondeu com o rotulo `ARCAVAULT`, e ha duas razoes
    /// possiveis para isso — a mensagem nomeia as duas, como C-12 exige do
    /// desfecho ausente. Um volume que existe mas nao responde a consulta
    /// (bloqueado pelo BitLocker, ainda montando) some da enumeracao do mesmo
    /// jeito que um dispositivo desconectado, e dizer so "conecte o
    /// dispositivo" mandaria o usuario conectar o que ja esta na mesa.
    #[error(
        "nenhum dispositivo ARCA conectado: nao ha volume que responda pelo rotulo ARCAVAULT. Ou o dispositivo nao esta conectado, ou o volume dele nao respondeu — um volume bloqueado pelo BitLocker ou ainda montando nao aparece"
    )]
    DispositivoAusente,

    /// C-10. Dois rotulos iguais tornam o destino ambiguo, e e por rotulo que
    /// a receita resolve o destino (S-3).
    #[error(
        "ha {quantos} volumes com o rotulo {rotulo} conectados, e o ARCA opera um dispositivo por vez: e pelo rotulo que a receita resolve o destino, e com ele repetido nao ha o que escolher. Desconecte os demais"
    )]
    DispositivosDemais {
        rotulo: &'static str,
        quantos: usize,
    },

    #[error(
        "o volume {rotulo} nao tem letra atribuida, e sem letra nao ha caminho por onde lê-lo. Atribua uma no Gerenciamento de Disco"
    )]
    VolumeSemLetra { rotulo: &'static str },

    #[error("o dispositivo conectado nao tem a particao {rotulo}")]
    ParticaoAusente { rotulo: &'static str },
}

impl Erro {
    /// Codigo de saida do processo. `2` para uso incorreto e recusa de
    /// elevacao — o mesmo que o clap usa —, `1` para o resto.
    pub fn codigo_de_saida(&self) -> u8 {
        match self {
            Erro::ElevacaoRecusada | Erro::AindaNaoImplementado { .. } => 2,
            _ => 1,
        }
    }
}

pub fn erro_de_arquivo(
    operacao: &'static str,
    caminho: impl Into<PathBuf>,
) -> impl FnOnce(std::io::Error) -> Erro {
    let caminho = caminho.into();
    move |origem| Erro::Arquivo {
        operacao,
        caminho,
        origem,
    }
}
