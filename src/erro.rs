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

    /// O pre-voo recusou a operacao **antes** de qualquer gravacao (B-3, B-4,
    /// C-6, C-10). Como o de C-2, este erro nunca chega depois de algo ter
    /// sido tocado.
    #[error("o pre-voo recusou: {0}")]
    PreVooRecusou(crate::prevoo::RecusaDoPreVoo),

    /// A restauracao foi recusada **antes** da confirmacao digitada, e portanto
    /// antes de qualquer escrita (R-1, R-2, R-3, R-7, L-2).
    ///
    /// Toda recusa desta familia acontece antes de o usuario digitar o nome da
    /// imagem: ninguem digita o nome inteiro de uma imagem para ouvir um nao
    /// depois — a mesma regra que a E7 aplicou ao disco de origem.
    #[error("a restauracao foi recusada: {0}")]
    RestauracaoRecusada(crate::comandos::restore::RecusaDaRestauracao),

    /// V-1 e V-2 recusam antes de conferir nada e antes de armar nada: a
    /// imagem nao existe, e residuo (L-2), ou o `MD5SUMS` dela nao serve.
    #[error("a verificacao foi recusada: {0}")]
    VerificacaoRecusada(crate::comandos::verify::RecusaDaVerificacao),

    /// V-1 conferiu e a imagem nao passou (S-5).
    ///
    /// **Nao e uma falha do comando** — ele fez exatamente o que se pediu, e a
    /// resposta e ruim. O erro existe pelo mesmo motivo que o `arca resultado`
    /// sai com codigo diferente de zero num desfecho ruim: quem chamou o ARCA
    /// de um script nao pode lê uma imagem reprovada como exito. A tela
    /// inteira ja foi impressa quando isto sobe, com cada arquivo que nao
    /// bateu.
    #[error(
        "a imagem `{nome}` NAO passou na conferencia: {quantos} arquivos nao bateram com o `MD5SUMS`. O detalhe de cada um esta na tela acima"
    )]
    ImagemReprovada { nome: String, quantos: usize },

    /// `arca prepare` recusou o disco **antes** de escrever qualquer coisa
    /// (PR-4, PR-5).
    ///
    /// Como a de C-2 e a do pre-voo, esta recusa nunca chega depois de algo ter
    /// sido tocado — e aqui isso vale mais do que em qualquer outro comando,
    /// porque o que vem depois apaga um disco inteiro.
    #[error("o `arca prepare` recusou o disco: {0}")]
    PreparacaoRecusada(crate::preparacao::RecusaDaPreparacao),

    /// O terceiro tempo de PR-4: o disco relido **não é** o do plano.
    ///
    /// O índice do Windows não é identidade — medido em 23/08/2026, quando o
    /// dispositivo desta mesa passou de disco 1 a disco 2 com um segundo SSD
    /// conectado. Entre imprimir o plano e escrever a tabela há uma pessoa
    /// lendo e digitando, e nesse intervalo cabe trocar um cabo.
    #[error(
        "o disco {indice} NAO e mais o que estava no plano (`{modelo}`), ou sumiu da enumeracao. Nada foi apagado. O indice do Windows muda quando se conecta ou desconecta um disco — rode `arca prepare --dispositivo <indice>` de novo e confira o modelo e o tamanho na tela antes de responder"
    )]
    DiscoMudouEntreOPlanoEOSim { indice: u32, modelo: String },

    /// O disco foi particionado e a releitura não mostra a estrutura pedida
    /// (PR-5, defesa 7).
    #[error("{0}")]
    ParticionamentoDivergiu(crate::preparacao::Divergencia),

    /// PR-1: o pacote do Clonezilla não passou.
    #[error("o pacote do Clonezilla foi recusado: {0}")]
    PacoteRecusado(crate::pacote::RecusaDoPacote),

    /// O `bcdedit /copy` respondeu e a resposta não trouxe **um**
    /// identificador.
    ///
    /// Zero é o `bcdedit` tendo recusado sem dizer; mais de um não diz qual
    /// vale, e escolher o errado apontaria o boot da máquina para outro lugar.
    #[error(
        "o `bcdedit /copy` respondeu com {quantos} identificadores, e o ARCA precisa de exatamente um para saber que entrada de boot acabou de criar. O ARCA acha o identificador pela FORMA — 36 caracteres entre chaves —, e nunca pelo texto, que vem traduzido. A resposta foi: {resposta}"
    )]
    EntradaNaoFoiCriada { quantos: usize, resposta: String },

    /// C-3 sobre o `path` da entrada nova.
    #[error(
        "a entrada de firmware {identificador} devia carregar `{esperado}` e a releitura mostra `{tem}`. O dispositivo esta particionado e com o Clonezilla dentro; o que falta e a entrada de boot apontar para o `.efi` certo"
    )]
    CaminhoDoEfiRecusado {
        identificador: String,
        esperado: String,
        tem: String,
    },

    /// C-3 sobre a `description` da entrada — a terceira das três escritas, e a
    /// que ficou sem conferência até 25/08/2026.
    ///
    /// # Por que ela merece o mesmo tratamento que o `device`
    ///
    /// Porque é o **mesmo comando**, e o C-6 é sobre o comando e não sobre o
    /// campo: medido num Kingston DataTraveler Max, o `bcdedit /set` responde
    /// *"A operação foi concluída com êxito"*, código 0, e não escreve. Não
    /// havia razão para supor que só o `device` sofre disso — havia só o
    /// hábito de conferir aquele.
    ///
    /// O que ela protege é C-4: a `description` é o que migra a entrada legada
    /// `Clonezilla` para `ARCA`, e é **a identidade de uma entrada de firmware
    /// neste projeto** — `Leitura::entrada_do_arca` procura por ela, e não por
    /// um GUID guardado, porque o identificador nomeia o slot da NVRAM e o
    /// firmware o reescreve (ADR-0025).
    #[error(
        "a entrada de firmware {identificador} devia se chamar `{esperado}` e a releitura mostra `{tem}`. O bcdedit respondeu sem escrever, que e o C-6 medido. O dispositivo esta particionado e com o Clonezilla dentro — rode `arca prepare` de novo no mesmo disco, que daqui em diante ele e idempotente"
    )]
    DescricaoDoFirmwareRecusada {
        identificador: String,
        esperado: String,
        tem: String,
    },

    /// A entrada continua na ordem permanente depois de mandada sair.
    ///
    /// `bcdedit /copy` a põe lá sozinho, e deixá-la é acrescentar um caminho
    /// permanente para bootar no dispositivo — o perigo que C-5 nomeia.
    #[error(
        "a entrada {identificador} devia ter saido da ordem permanente de boot e a releitura mostra [{ordem}]. Deixa-la ali acrescenta um caminho permanente para a maquina bootar no dispositivo, que e o que C-5 existe para impedir — e ligar a maquina com ele conectado passaria a abrir o menu do Clonezilla"
    )]
    EntradaContinuaNaOrdem {
        identificador: String,
        ordem: String,
    },

    /// A enumeracao de discos nao achou o disco onde o Windows mora, ou achou
    /// so o proprio dispositivo.
    ///
    /// Sem ele nao ha o que clonar, e nao ha como calcular o espaco de B-4.
    /// **Nao se supoe que e o disco 0**: numa maquina com dois discos isso
    /// daria a origem errada, e a origem errada e o que a receita nomeia.
    #[error(
        "a enumeracao de discos nao achou o disco onde o Windows esta, separado do dispositivo ARCA. Sem ele nao ha origem para o backup, e o ARCA nao supoe que seja o disco 0 — numa maquina com dois discos isso nomearia o disco errado na receita"
    )]
    OrigemDesconhecida,

    /// O `estado.json` do `ARCABOOT` nao pode ser lido nem escrito sem
    /// adivinhacao.
    ///
    /// "Nao entendi o arquivo" nunca vira "nao ha job pendente": um
    /// dispositivo com job armado e estado corrompido continua armado, e
    /// tratar as duas coisas como iguais mandaria alguem reiniciar achando que
    /// nao ha nada esperando.
    ///
    /// A mensagem diz o que fazer, e nao so o que houve. Um `estado.json`
    /// ilegivel e o caso em que o ARCA menos pode agir sozinho — ele nao sabe
    /// qual job esta armado — e ao mesmo tempo aquele em que **o dispositivo
    /// pode estar armado**. Sem a instrucao, quem lê fica com um diagnostico e
    /// nenhuma saida, e a saida existe: `arca desarmar` nao consulta estado
    /// nenhum (C-1) e por isso funciona justamente aqui.
    #[error(
        "o estado do job nao pode ser lido: {0}. Isto NAO quer dizer que nao ha job: o dispositivo pode estar armado, e o que dizia qual job era este se perdeu. Rode `arca status` para ver se ha boot unico armado, e `arca desarmar` para devolver o dispositivo ao estado inerte — ele nao consulta estado nenhum, e por isso funciona mesmo com este arquivo ilegivel. O ARCA nao apaga o arquivo (B-10)"
    )]
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

    /// Nao ha de onde derivar o `menuentry` do ARCA. Chega **antes** de
    /// qualquer gravacao, como o de C-2.
    #[error("o bloco do ARCA nao pode ser montado: {0}")]
    MenuentryRecusado(crate::menuentry::RecusaDoMenuentry),

    /// C-4 sem nada a migrar: nao ha entrada `ARCA` nem a legada `Clonezilla`.
    ///
    /// Criar uma entrada de firmware do zero e codigo sem original — nenhuma
    /// captura mostra a forma —, e o lugar disso e o `arca prepare` da E10.
    /// Armar nao e a hora de estrear a criacao de entrada de boot.
    #[error(
        "nao ha entrada de firmware chamada `ARCA` nem a legada `Clonezilla`, e sem uma delas nao ha por onde a maquina bootar no dispositivo sem F12. Criar uma entrada do zero e trabalho do `arca prepare`, que a etapa E10 entrega — armar nao cria entrada de boot"
    )]
    SemEntradaDeFirmware,

    /// C-4 com C-3: mandou-se renomear a entrada legada e a releitura mostra
    /// que ela continua com o nome antigo.
    #[error(
        "a entrada de firmware {identificador} devia ter sido migrada de `{de}` para `ARCA` e a releitura ainda mostra `{tem}`. O sucesso do bcdedit nunca e prova (C-3), e o ARCA nao arma sobre uma entrada que nao sabe se mexeu"
    )]
    EntradaNaoMigrou {
        identificador: String,
        de: String,
        tem: String,
    },

    /// C-6 na pratica, e pela primeira vez: o `bcdedit` respondeu "êxito" e
    /// manteve o valor antigo do `device`.
    ///
    /// E assim que a rejeicao silenciosa do §3.1 se revela — nao por etiqueta,
    /// que essas palavras nao saem do `bcdedit`, mas pela releitura de C-3.
    #[error(
        "a entrada de firmware {identificador} devia apontar para `{esperado}` e a releitura mostra `{tem}`. O bcdedit responde \"êxito\" e mantem o valor antigo quando o alvo e midia removivel (C-6, §3.1) — um dispositivo assim boota por F12, nunca por entrada de firmware"
    )]
    AlvoDoFirmwareRecusado {
        identificador: String,
        esperado: String,
        tem: String,
    },

    /// C-3 do lado de armar: mandou-se marcar o boot unico e a releitura nao
    /// mostra a marca, ou mostra uma apontando para outra entrada.
    ///
    /// O dispositivo fica com receita gravada e a maquina reinicia no Windows.
    /// O que este erro impede e o reinicio: um ARCA que reiniciasse aqui
    /// dispararia o reinicio sem saber se armou.
    #[error(
        "a marca de boot unico devia apontar para {identificador} e a releitura mostra {tem}. O grub.cfg ficou com a receita gravada e a maquina NAO foi reiniciada: rode `arca desarmar` para devolver o dispositivo ao estado inerte"
    )]
    BootUnicoNaoArmou { identificador: String, tem: String },

    /// A marca de boot unico pegou, e o identificador **deixou de nomear a
    /// entrada do ARCA**.
    ///
    /// # Por que isto e um erro proprio, e nao um caso do de cima
    ///
    /// Porque o de cima pergunta *"a marca aponta para o identificador que eu
    /// armei?"* e este pergunta *"o identificador que eu armei ainda e a
    /// entrada que eu queria?"* — e as duas perguntas se separaram por
    /// medicao, e nao por zelo.
    ///
    /// **Medido em 25/08/2026:** o
    /// `{31cc955f-a0ae-11f1-8a54-806e6f6e6963}` era `UEFI:CD/DVD Drive`, sem
    /// `device`, e depois de um boot o **mesmo GUID** era outra entrada, com
    /// `device partition=E:`. O identificador nomeia o *slot* `Boot####` da
    /// NVRAM, e o firmware reescreve os slots.
    ///
    /// Sem esta conferencia, um slot que trocasse de dono entre a leitura e a
    /// escrita faria o ARCA armar **outra entrada**, ver o proprio GUID no
    /// `bootsequence`, e relatar exito — com a maquina reiniciando para o
    /// lugar errado. E exatamente o modo de falha que o comentario de
    /// `marcar_o_boot_unico` ja nomeava, e contra o qual o GUID sozinho nao
    /// bastava.
    ///
    /// Que isso aconteca **dentro de uma sessao**, sem reinicio, nao foi
    /// medido — nas seis releituras do marco nada mudou. A defesa existe
    /// porque custa uma comparacao de texto e o modo de falha e o pior que
    /// este comando tem.
    #[error(
        "armei o boot unico em {identificador} e a releitura mostra que esse identificador e da entrada `{descricao}`, e nao do ARCA. O identificador do bcdedit nomeia o slot da NVRAM, e nao a entrada — o firmware pode te-los reescrito. O grub.cfg ficou com a receita gravada e a maquina NAO foi reiniciada: rode `arca desarmar` e confira o firmware com `arca status`"
    )]
    BootUnicoApontaParaOutra {
        identificador: String,
        descricao: String,
    },

    /// O nome do disco de origem nao foi determinado, e armar exige um.
    ///
    /// **Nao ha caminho de "digite voce"**, e isso e decisao e nao omissao:
    /// um nome de disco do Linux digitado do lado Windows nao tem contra o
    /// que ser conferido, e a receita que o nomeia e destrutiva na E9. O
    /// oraculo e um `blkdev.list` (§4.5).
    ///
    /// **A saida mudou na E12**, e a mensagem mudou com ela: ate a E11 ela
    /// mandava fazer o primeiro backup pelo menu do Clonezilla, que era o que
    /// existia — e que e exatamente aquilo que este app existe para nao
    /// precisar. `arca sondar` produz o mesmo arquivo num reinicio, sem imagem
    /// nenhuma. **O `porque` ja diz isso quando sondar resolve**, e por isso a
    /// frase fixa nao repete: cada recusa de [`crate::blkdev::SemNome`] diz a
    /// saida dela, e `ModeloAmbiguo` nao tem essa.
    #[error(
        "o nome que o Linux da ao disco de origem nao foi determinado, e a receita precisa dele: {porque}. O ARCA nao aceita esse nome digitado nem o deriva do indice do Windows — o indice do Windows nao e o do Linux, e um nome sem oraculo nomearia o disco errado numa receita (§4.5)"
    )]
    DiscoDeOrigemPorDeterminar { porque: String },

    /// S-2: a confirmacao digitada nao bate com o nome da imagem.
    #[error(
        "a confirmacao nao bate: era para digitar `{esperado}` e veio `{digitado}`. Nada foi armado"
    )]
    ConfirmacaoNaoBate { esperado: String, digitado: String },

    /// S-5: o backup terminou e alguma parte dele nao deu certo.
    ///
    /// Falha parcial e falha total. A saida do §5.4 ja foi impressa inteira
    /// quando este erro sobe — ele existe para o codigo de saida, e para que
    /// quem chamou o ARCA de um script nao leia um desfecho ruim como exito.
    #[error("{0}")]
    OperacaoNaoConcluida(String),

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

    /// C-13: mandou-se pôr o `{bootmgr}` no topo da ordem permanente, e a
    /// releitura mostra outra coisa em primeiro.
    ///
    /// E o modo de falha medido do `bcdedit` desde a E2 — responder "êxito"
    /// sem ter escrito —, e por isso quem responde e sempre o `/enum` (C-3).
    /// A consequencia de deixar passar e concreta: quem lesse a tela acharia
    /// que a maquina volta ao Windows, e ela continuaria bootando no
    /// dispositivo a cada reinicio.
    #[error(
        "mandei por o gerenciador do Windows no topo da ordem permanente de boot e a releitura mostra [{ordem}]. O bcdedit respondeu sem escrever, ou escreveu noutro lugar. Enquanto isso, ligar a maquina com o SSD conectado continua bootando nele (C-13)"
    )]
    OrdemNaoDevolvida { ordem: String },

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
    ///
    /// **A mensagem nomeia as letras desde a E10**, e a razao e que o caso
    /// deixou de ser raro: `arca prepare` cria um dispositivo, e um comando
    /// bem-sucedido deixa dois conectados por definicao. A partir dai todo
    /// comando cai aqui — inclusive o `arca status`, que e o que alguem
    /// rodaria para entender o que esta acontecendo.
    #[error(
        "ha {quantos} volumes com o rotulo {rotulo} conectados ({onde}), e o ARCA opera um dispositivo por vez: e pelo rotulo que a receita resolve o destino, e com ele repetido nao ha o que escolher. Desconecte os demais e rode de novo. Se voce acabou de preparar um dispositivo, sao os dois — o novo e o de antes"
    )]
    DispositivosDemais {
        rotulo: &'static str,
        quantos: usize,

        /// As letras dos volumes achados, para quem lê saber **quais**
        /// desconectar. `Desconecte os demais` sem dizer quais empurra a
        /// pergunta de volta para quem não tem como respondê-la.
        onde: String,
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
