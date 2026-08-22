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

    /// B-2: o nome que o usuario digitou nao serve. A recusa carrega o motivo
    /// proprio — quem digitou um nome com acento precisa ouvir "acento", e
    /// nao "nome invalido".
    #[error("nome de imagem recusado (B-2): {0}")]
    NomeRecusado(crate::nome::Recusa),

    /// C-2: o porteiro da receita recusou, **antes** de qualquer gravacao.
    /// Este erro nunca chega depois de o `grub.cfg` ter sido tocado.
    #[error("receita recusada (C-2): {0}")]
    ReceitaRecusada(crate::receita::RecusaDaReceita),

    /// O `estado.json` do `ARCABOOT` nao pode ser lido nem escrito sem
    /// adivinhacao.
    ///
    /// "Nao entendi o arquivo" nunca vira "nao ha job pendente": um
    /// dispositivo com job armado e estado corrompido continua armado, e
    /// tratar as duas coisas como iguais mandaria alguem reiniciar achando que
    /// nao ha nada esperando.
    #[error("o estado do job nao pode ser lido: {0}")]
    EstadoRecusado(crate::estado::RecusaDoEstado),

    /// A fonte de entropia do Windows recusou. Sem selo nao se arma (C-11):
    /// um job sem selo e um job cujo desfecho ninguem consegue reclamar.
    #[error(
        "o Windows nao entregou os bytes do selo (NTSTATUS {estado:#010x}). Sem selo nao ha como ligar o desfecho ao job, e o ARCA nao arma sem isso"
    )]
    EntropiaIndisponivel { estado: i32 },

    /// O `grub.cfg` do dispositivo nao pode ser desarmado sem adivinhacao.
    /// Como o de C-2, este erro chega **antes** da gravacao: um `grub.cfg`
    /// armado ainda boota, e um pela metade nao.
    #[error("o grub.cfg nao foi alterado: {0}")]
    GrubRecusado(crate::grub::RecusaDoGrub),

    /// C-3 na pratica: mandou-se apagar a marca de boot unico, e a releitura
    /// mostra que ela continua la. O sucesso do `bcdedit` nunca foi prova; a
    /// releitura e, e ela reprovou.
    #[error(
        "a marca de boot unico continua no firmware depois de mandada apagar, apontando para {entradas}. O bcdedit responde \"êxito\" sem ter feito nada em alguns casos, e e por isso que o ARCA confere com /enum. Rode `arca status` e confira o firmware antes de reiniciar"
    )]
    BootUnicoPersistente { entradas: String },

    /// O `bcdedit` respondeu, e a resposta nao tinha o que se foi buscar.
    ///
    /// Distinta de [`Erro::FerramentaRecusou`], que e a ferramenta dizendo
    /// nao. Aqui ela disse sim e devolveu algo que o parser nao reconheceu — e
    /// `crate::firmware::ler` nunca falha, entao isso chegaria como leitura
    /// vazia. Numa **exibicao**, leitura vazia e so uma tela vazia; numa
    /// releitura de C-3, seria o ARCA concluindo que a marca de boot unico
    /// sumiu porque nao conseguiu ver marca nenhuma.
    #[error(
        "o bcdedit respondeu ao /enum {alvo} sem trazer o gerenciador de firmware. Nao da para conferir se a marca de boot unico foi apagada, e o ARCA nao trata \"nao entendi a resposta\" como \"nao ha nada armado\". Rode `arca status` e confira o firmware antes de reiniciar"
    )]
    FirmwareIlegivel { alvo: &'static str },

    /// C-5: a ordem permanente de boot mudou durante o desarmar, que nao tem
    /// nada que ver com ela.
    #[error(
        "a ordem permanente de boot mudou durante o desarmar, de [{antes}] para [{depois}], e desarmar nunca deveria toca-la (C-5). Confira o firmware com `arca status`"
    )]
    OrdemPermanenteAlterada { antes: String, depois: String },

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
    /// Se este erro e "o arquivo nao esta la", e nao "nao consegui olhar".
    ///
    /// # Por que isto existe em vez de um `existe()` antes da leitura
    ///
    /// [`crate::portas::Arquivos::existe`] devolve `bool`, e um `bool` nao tem
    /// como dizer "nao sei": `Path::exists` transforma **qualquer** falha de
    /// I/O em `false`. Quem perguntasse antes de lê para separar "nao ha
    /// desfecho" de "nao consegui lê o desfecho" estaria fazendo a pergunta a
    /// quem ja confundiu as duas — um `arca-fim.txt` num volume com problema
    /// de leitura sairia como "o boot nao aconteceu", e quem lesse concluiria
    /// que o backup nunca rodou.
    ///
    /// A saida e nao perguntar: tenta-se a leitura, e o `ErrorKind` que volta
    /// diz qual dos dois casos e. E mais preciso, e nao ha janela entre a
    /// pergunta e a leitura.
    pub fn e_arquivo_ausente(&self) -> bool {
        matches!(
            self,
            Erro::Arquivo { origem, .. } if origem.kind() == std::io::ErrorKind::NotFound
        )
    }

    /// Codigo de saida do processo. `2` para uso incorreto e recusa de
    /// elevacao — o mesmo que o clap usa —, `1` para o resto.
    pub fn codigo_de_saida(&self) -> u8 {
        match self {
            // Nome recusado e uso incorreto, como o `clap` o entende: quem
            // chamou o ARCA de um script precisa distinguir "voce digitou um
            // nome invalido" de "alguma coisa falhou".
            Erro::ElevacaoRecusada | Erro::AindaNaoImplementado { .. } | Erro::NomeRecusado(_) => 2,
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
