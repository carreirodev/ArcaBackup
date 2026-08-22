//! Armar: gravar o estado, pôr a receita no `grub.cfg` e marcar o boot unico
//! (C-3, C-4, C-5, C-6, C-11). **E o ponto sem volta.**
//!
//! O inverso de [`crate::desarme`], e escrito depois dele de proposito: a
//! primeira regra do plano e que so se arma o que ja se sabe desarmar.
//!
//! # A ordem das tres gravacoes, e por que e esta
//!
//! Sao tres escritas — `estado.json`, `grub.cfg`, `bootsequence` — e qualquer
//! uma pode falhar com as anteriores ja feitas. As seis ordens possiveis
//! deixam estados diferentes, e so uma delas nao tem estado ruim:
//!
//! 1. **`estado.json` primeiro.** E o unico lugar onde fica escrito **qual**
//!    job foi armado — o selo, o nome, o disco. Falhando aqui, nada foi
//!    armado. A ordem inversa e que e cara: uma receita armada sem estado
//!    gravado faria a maquina reiniciar, rodar o backup e escrever um
//!    `arca-fim.txt` com um selo que **ninguem anotou**, e a colheita da E8
//!    nao teria com o que casar. O desfecho existiria e seria inalcancavel.
//!
//! 2. **`grub.cfg` depois.** Feito isto, o dispositivo esta armado no sentido
//!    que importa a quem bootar nele por F12 — e so nesse. A maquina continua
//!    bootando no Windows, porque nada no firmware mudou ainda.
//!
//! 3. **`bootsequence` por ultimo.** E a unica das tres que muda o que
//!    acontece no proximo reinicio **sem ninguem pedir**. Deixa-la para o fim
//!    e o que garante que o reinicio so vira automatico depois de tudo de que
//!    ele depende ja estar em disco.
//!
//! Os dois estados intermediarios sao nomeaveis e reversiveis:
//!
//! - estado gravado, `grub.cfg` inerte: `arca status` mostra job pendente com
//!   boot unico nao armado, o dispositivo esta inerte e nada roda. O proximo
//!   `arca backup` sobrescreve o estado.
//! - estado e `grub.cfg` armados, sem marca: a maquina reinicia no Windows e
//!   o dispositivo fica com uma receita esperando quem bootar nele por F12.
//!   E o mesmo estado que o [`crate::desarme`] nomeia na direcao inversa, e
//!   `arca desarmar` o resolve.
//!
//! O que **nao** existe em nenhuma ordem intermediaria e a marca de boot
//! unico apontando para um `grub.cfg` que ainda nao tem receita — que seria
//! um reinicio para o menu do Clonezilla, sem aviso.
//!
//! # O reinicio nao mora aqui, e isso importa
//!
//! Esta funcao arma e **confere**; quem reinicia e o comando, depois de
//! receber um [`Armado`] em maos. Um ARCA que reiniciasse antes da releitura
//! de C-3 dispararia o reinicio sem saber se armou — e o `bcdedit` responde
//! "êxito" sem ter feito nada quando o alvo e midia removivel.
//!
//! # C-3 aparece tres vezes, e nao por zelo
//!
//! Cada escrita no `bcdedit` e seguida de um `/enum` que a confirma: a
//! migracao da descricao (C-4), o alvo da entrada (C-6) e a marca de boot
//! unico. O sucesso do `bcdedit` nunca e prova em nenhuma das tres.

use crate::desfecho;
use crate::dispositivo::Dispositivo;
use crate::erro::{Erro, Resultado};
use crate::estado::{self, Estado, MomentoDoArmar};
use crate::firmware::{self, Alvo, EntradaDeFirmware, Procedencia};
use crate::grub;
use crate::menuentry;
use crate::nome::Nome;
use crate::portas::{Arquivos, Entropia, Firmware, Relogio};
use crate::receita::{Disco, Operacao, Pedido, Receita, Selo};
use std::path::PathBuf;

/// O objeto do `bcdedit` que guarda a ordem de boot e a marca de boot unico.
const FWBOOTMGR: &str = "{fwbootmgr}";

/// O elemento que carrega o boot unico.
const BOOTSEQUENCE: &str = "bootsequence";

/// O alvo do `/enum` que traz as entradas de boot do firmware.
const FIRMWARE: &str = "firmware";

/// O que aconteceu com a entrada de firmware (C-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entrada {
    /// Ja se chamava `ARCA`. Nada a migrar.
    JaEraDoArca,

    /// Chamava-se `Clonezilla` e foi **renomeada**, nunca duplicada. O parser
    /// da E2 distingue as duas por [`Procedencia`], e criar outra ao lado
    /// deixaria a maquina com duas formas de bootar no Clonezilla — uma delas
    /// sem ninguem olhando.
    MigradaDaLegada { de: String },
}

/// O que armar fez, para a tela e para o registro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armado {
    pub caminho_do_estado: PathBuf,
    pub caminho_do_grub: PathBuf,

    /// O selo deste job. E ele que a colheita da E8 vai cobrar do
    /// `arca-fim.txt`.
    pub selo: Selo,

    pub entrada: Entrada,

    /// O identificador da entrada de firmware para onde o boot unico aponta.
    pub identificador: String,

    /// Para onde a entrada aponta, **relido** depois de escrever (C-3, C-6).
    pub alvo: Alvo,

    /// Onde o desfecho deste job vai aparecer, do lado Windows.
    pub caminho_do_desfecho: PathBuf,

    /// A pasta do log, como a receita a nomeia dos dois lados do reinicio.
    pub pasta_do_desfecho: String,
}

/// O pedido de armar, ja com tudo julgado pelo pre-voo.
pub struct Pedir<'a> {
    pub dispositivo: &'a Dispositivo,
    pub operacao: Operacao,
    pub nome: &'a Nome,
    pub disco: &'a Disco,
}

/// Arma o dispositivo e confere que armou.
///
/// Nao desarma antes: quem chama ja desarmou como primeiro passo (C-1), e um
/// desarmar aqui seria o segundo. O `grub::armar` recusa um `grub.cfg` que ja
/// tenha bloco do ARCA, que e a rede embaixo dessa suposicao.
pub fn executar(
    arquivos: &dyn Arquivos,
    ferramenta: &dyn Firmware,
    entropia: &dyn Entropia,
    relogio: &dyn Relogio,
    pedido: &Pedir,
) -> Resultado<Armado> {
    let caminho_do_grub = pedido.dispositivo.caminho_do_grub()?;
    let caminho_do_estado = pedido.dispositivo.caminho_do_estado()?;
    let raiz_do_vault = pedido.dispositivo.raiz_do_vault()?;

    // O `ARCABOOT` tem letra — `caminho_do_grub` acabou de exigi-la —, e e
    // para ela que a entrada de firmware tem de apontar.
    let letra_do_boot = pedido
        .dispositivo
        .boot
        .as_ref()
        .and_then(|boot| boot.letra)
        .ok_or(Erro::VolumeSemLetra {
            rotulo: crate::dispositivo::ARCABOOT,
        })?;

    // Em maiuscula, e nao por estetica: [`Alvo::ler`] normaliza para maiuscula
    // o que vem do `bcdedit`, e a comparacao de [`EntradaDeFirmware::aponta_para`]
    // e por igualdade de `Alvo`. Um `r` minusculo vindo da enumeracao de
    // volumes nunca casaria com o `R` relido, e o ARCA diria que o `bcdedit`
    // recusou o alvo em silencio quando ele o aceitou. As duas pontas sao
    // maiusculas hoje — a enumeracao monta as letras de `b'A'` —, e e
    // exatamente por isso que depender disso sem dizer seria caro de descobrir.
    let alvo_desejado = Alvo::ParticaoComLetra(letra_do_boot.to_ascii_uppercase());

    // Antes de qualquer escrita: sem privilegio, o `bcdedit` recusa aqui e
    // nada foi tocado. E e daqui que sai a ordem permanente que C-5 protege.
    //
    // Ela e lida **antes das tres escritas no firmware**, e nao so antes da
    // ultima. As duas primeiras — a descricao de C-4 e o `device` de C-6 —
    // nao deveriam mexer na ordem de boot, e "nao deveria" e exatamente o que
    // C-3 manda desconfiar: a comparacao do fim pega qualquer uma das tres que
    // tenha mexido.
    let gerenciador_antes = firmware::ler(&ferramenta.enumerar(FWBOOTMGR)?);
    if !gerenciador_antes.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    // C-4 e C-6, nesta ordem, antes das tres gravacoes. Nenhuma das duas arma
    // coisa alguma — renomear uma descricao e acertar para onde uma entrada
    // aponta nao mudam o que a maquina faz no proximo reinicio —, e falhar
    // aqui deixa o dispositivo como estava.
    let (identificador, entrada) = migrar_a_entrada(ferramenta)?;
    let alvo = apontar_para_o_arcaboot(ferramenta, &identificador, &alvo_desejado)?;

    // A receita e montada com o selo dentro, e ja sai validada por C-2.
    let selo = estado::gerar_selo(entropia)?;
    let receita = Receita::montar(&Pedido {
        operacao: pedido.operacao,
        nome: pedido.nome.clone(),
        disco: pedido.disco.clone(),
        selo: selo.clone(),
    })
    .map_err(Erro::ReceitaRecusada)?;

    // O bloco deriva do `grub.cfg` que esta no dispositivo, e nao de uma copia
    // guardada: e ali que mora a configuracao daquele hardware.
    let inerte = arquivos.ler_texto(&caminho_do_grub)?;
    let bloco =
        menuentry::derivar(&inerte, receita.parametros()).map_err(Erro::MenuentryRecusado)?;
    let armado = grub::armar(&inerte, &bloco).map_err(Erro::GrubRecusado)?;

    // ---- as tres gravacoes, na ordem que o cabecalho deste modulo explica ----

    // 1. Quem e este job.
    let estado_do_job = Estado {
        selo: selo.clone(),
        comando: pedido.operacao,
        nome: pedido.nome.clone(),
        disco: pedido.disco.clone(),
        armado_em: MomentoDoArmar::agora(relogio),
        situacao: estado::Situacao::Armado,
    };
    estado::gravar(arquivos, &caminho_do_estado, &estado_do_job)?;

    // 2. O que a maquina executa se bootar no dispositivo.
    arquivos.escrever_atomico(&caminho_do_grub, &armado)?;

    // 3. O que faz a maquina bootar no dispositivo sozinha.
    marcar_o_boot_unico(ferramenta, &identificador, &gerenciador_antes)?;

    Ok(Armado {
        caminho_do_estado,
        caminho_do_grub,
        selo,
        entrada,
        identificador,
        alvo,
        caminho_do_desfecho: estado::caminho_do_desfecho(
            &raiz_do_vault,
            pedido.operacao,
            pedido.nome,
        ),
        pasta_do_desfecho: desfecho::pasta_do_job(pedido.operacao, pedido.nome),
    })
}

/// C-4: acha a entrada do ARCA e, sendo a legada, renomeia-a.
///
/// Renomeia, e nao cria outra. A entrada desta maquina ja tem o `.efi` certo,
/// o `device` certo e um GUID que o firmware conhece; o que ela nao tinha era
/// o nome. Criar uma segunda deixaria duas entradas apontando para o mesmo
/// lugar, e a ordem de boot passaria a ter uma que o ARCA nao gerencia.
///
/// **Nao havendo entrada nenhuma, recusa.** Criar uma do zero e codigo sem
/// original — nao ha captura de onde transcrever a forma —, e o lugar disso e
/// o `arca prepare` da E10, que prepara dispositivo. Armar nao e a hora de
/// estrear a criacao de entrada de firmware.
fn migrar_a_entrada(ferramenta: &dyn Firmware) -> Resultado<(String, Entrada)> {
    let leitura = firmware::ler(&ferramenta.enumerar(FIRMWARE)?);
    let achado = leitura
        .entrada_do_arca()
        .ok_or(Erro::SemEntradaDeFirmware)?;

    let identificador = achado.entrada.identificador.clone();
    if achado.procedencia == Procedencia::Propria {
        return Ok((identificador, Entrada::JaEraDoArca));
    }

    let de = achado.descricao.to_string();
    let _ = ferramenta.executar(&["/set", &identificador, "description", firmware::ARCA]);

    // C-3: o sucesso do `bcdedit` nao e prova. E aqui a releitura tem um
    // segundo emprego — ela e a **unica** forma de a migracao ser idempotente:
    // rodada de novo, a entrada ja se chama `ARCA` e o caminho acima nem e
    // tomado.
    let depois = releitura(ferramenta, &identificador)?;
    if !depois
        .descricao
        .as_deref()
        .is_some_and(|nome| nome.eq_ignore_ascii_case(firmware::ARCA))
    {
        return Err(Erro::EntradaNaoMigrou {
            identificador,
            de,
            tem: depois.descricao.unwrap_or_else(|| "nada".to_string()),
        });
    }

    Ok((identificador, Entrada::MigradaDaLegada { de }))
}

/// C-6: garante que a entrada aponta para o `ARCABOOT` que esta na mesa.
///
/// # Escreve sempre, e confere sempre
///
/// Comparar antes e so escrever quando difere pareceria economia e nao e. A
/// releitura teria de acontecer de qualquer jeito, e e **ela** que revela a
/// rejeicao silenciosa do §3.1: com o alvo em midia removivel, o `bcdedit`
/// responde "êxito" e mantem o valor antigo. Uma escrita que nao muda nada
/// custa uma chamada de processo; um `if` que a pula em troca de nada custa um
/// caminho a menos exercitado justamente no caso normal — que e o mesmo
/// raciocinio que [`crate::desarme`] faz sobre o `deletevalue`.
///
/// Esta e a primeira vez que C-6 e exercitado de verdade. Ate aqui, o
/// `Removable Media` do §3.1 era relatado por duas leituras — o `MediaType` do
/// WMI, no pre-voo — e nunca pela escrita que ele descreve.
fn apontar_para_o_arcaboot(
    ferramenta: &dyn Firmware,
    identificador: &str,
    desejado: &Alvo,
) -> Resultado<Alvo> {
    let escrito = desejado.como_bcdedit_escreve();
    let _ = ferramenta.executar(&["/set", identificador, "device", &escrito]);

    let depois = releitura(ferramenta, identificador)?;
    if !depois.aponta_para(desejado) {
        return Err(Erro::AlvoDoFirmwareRecusado {
            identificador: identificador.to_string(),
            esperado: escrito,
            tem: depois
                .alvo
                .as_ref()
                .map(Alvo::como_bcdedit_escreve)
                .unwrap_or_else(|| "nada".to_string()),
        });
    }

    Ok(desejado.clone())
}

/// Marca o boot unico e confere que ele pegou, sem tocar na ordem permanente.
///
/// # A entrada do ARCA nao entra na ordem de boot, e isso e C-5
///
/// Medido em 22/08/2026, nesta maquina: o `displayorder` do `{fwbootmgr}` tras
/// **so** o `{bootmgr}`, e a entrada do ARCA nao esta nele. O `bcdedit` aceita
/// `bootsequence` para uma entrada de fora da ordem — set, releitura, e la
/// esta ela. Pôr a entrada na ordem para "garantir" que o boot funcione seria
/// exatamente o que C-5 proibe, e e permanente: desfeito o job, a maquina
/// continuaria com um caminho a mais para bootar no dispositivo.
///
/// A comparacao com `antes` e o que pega um `bcdedit` que mexeu no que nao
/// devia. Ela custa nada: a leitura ja aconteceu.
fn marcar_o_boot_unico(
    ferramenta: &dyn Firmware,
    identificador: &str,
    antes: &firmware::Leitura,
) -> Resultado<()> {
    let _ = ferramenta.executar(&["/set", FWBOOTMGR, BOOTSEQUENCE, identificador]);

    let depois = firmware::ler(&ferramenta.enumerar(FWBOOTMGR)?);

    // O parser nunca falha por desenho: texto irreconhecivel vira leitura
    // vazia, e leitura vazia tem `boot_unico` vazio. Aqui isso seria pior do
    // que no desarmar — la o ARCA diria "nao havia" com a marca armada; aqui
    // ele diria "nao armou" com a marca armada, e o reinicio nao aconteceria
    // com o dispositivo pronto para bootar. Nas duas pontas, a saida e a
    // mesma: nao entendi nao vira resposta.
    if !depois.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    // C-5 antes do resto: uma ordem permanente alterada e falha mesmo que o
    // boot unico tenha pegado. Conferir depois deixaria o ARCA relatar exito
    // sobre uma maquina cuja configuracao de boot ele mudou para sempre.
    if depois.ordem_permanente != antes.ordem_permanente {
        return Err(Erro::OrdemPermanenteAlterada {
            antes: antes.ordem_permanente.join(", "),
            depois: depois.ordem_permanente.join(", "),
        });
    }

    // A marca tem de existir **e** apontar para esta entrada. "Ha algum boot
    // unico" nao basta: um `bootsequence` deixado por outra coisa faria o ARCA
    // dizer que armou enquanto a maquina bootaria noutro lugar.
    if !depois
        .boot_unico
        .iter()
        .any(|entrada| entrada.eq_ignore_ascii_case(identificador))
    {
        return Err(Erro::BootUnicoNaoArmou {
            identificador: identificador.to_string(),
            tem: if depois.boot_unico.is_empty() {
                "nenhuma marca".to_string()
            } else {
                depois.boot_unico.join(", ")
            },
        });
    }

    Ok(())
}

/// Relê uma entrada especifica depois de escrever nela (C-3).
fn releitura(ferramenta: &dyn Firmware, identificador: &str) -> Resultado<EntradaDeFirmware> {
    firmware::ler(&ferramenta.enumerar(FIRMWARE)?)
        .entradas
        .into_iter()
        .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador))
        .ok_or(Erro::FirmwareIlegivel { alvo: FIRMWARE })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::dispositivo;
    use crate::duplos::{
        ArquivosEmMemoria, DiscosDeMentira, EntropiaDeMentira, FirmwareDeMentira, RelogioParado,
    };
    use crate::grub::ID_DO_ARCA;

    const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");
    const PT: &str = include_str!("../recursos/capturas/bcdedit-enum-firmware-pt.txt");
    const LEGADO: &str = include_str!("../recursos/capturas/bcdedit-enum-firmware-legado-pt.txt");

    const GRUB: &str = r"R:\boot\grub\grub.cfg";
    const ESTADO: &str = r"R:\arca\estado.json";
    const GUID: &str = "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";

    /// Os oito bytes de que o selo de teste nasce.
    const BYTES: [u8; 8] = [0xa3, 0xf1, 0xc9, 0xe0, 0x7b, 0x2d, 0x48, 0x56];
    const SELO: &str = "a3f1c9e07b2d4856";

    /// Um `{fwbootmgr}` como o `bcdedit` desta maquina o enumera.
    ///
    /// A ordem permanente e a **medida**: so o `{bootmgr}`. A entrada do ARCA
    /// nao esta nela, e e sobre essa configuracao que o boot unico tem de
    /// funcionar (C-5).
    fn fwbootmgr(boot_unico: Option<&str>) -> String {
        let sequencia = match boot_unico {
            Some(alvo) => format!("bootsequence            {alvo}\r\n"),
            None => String::new(),
        };
        format!(
            "\r\nGerenciador de Inicialização de Firmware\r\n\
             ----------------------------------------\r\n\
             identificador           {FWBOOTMGR}\r\n\
             displayorder            {{bootmgr}}\r\n\
             {sequencia}timeout                 1\r\n"
        )
    }

    fn dispositivo() -> Dispositivo {
        dispositivo::encontrar(&DiscosDeMentira::com_dispositivo()).unwrap()
    }

    fn nome() -> Nome {
        Nome::novo("2026-08-22_Apps").unwrap()
    }

    fn disco() -> Disco {
        Disco::novo("nvme0n1").unwrap()
    }

    /// O firmware desta maquina: entrada `ARCA`, sem boot unico.
    fn firmware_desta_maquina() -> FirmwareDeMentira {
        FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, PT)
            .respondendo(FWBOOTMGR, &fwbootmgr(None))
            .respondendo_depois(FWBOOTMGR, &fwbootmgr(Some(GUID)))
    }

    fn arquivos() -> ArquivosEmMemoria {
        ArquivosEmMemoria::novo().com(GRUB, INERTE)
    }

    fn armar_com(
        arquivos: &ArquivosEmMemoria,
        ferramenta: &FirmwareDeMentira,
    ) -> Resultado<Armado> {
        let dispositivo = dispositivo();
        let nome = nome();
        let disco = disco();
        executar(
            arquivos,
            ferramenta,
            &EntropiaDeMentira::com(&BYTES),
            &RelogioParado::em("2026-08-22T19:14:03"),
            &Pedir {
                dispositivo: &dispositivo,
                operacao: Operacao::Backup,
                nome: &nome,
                disco: &disco,
            },
        )
    }

    #[test]
    fn armar_grava_as_tres_coisas_e_confere_o_boot_unico() {
        let arquivos = arquivos();
        let ferramenta = firmware_desta_maquina();

        let armado = armar_com(&arquivos, &ferramenta).expect("arma");

        // 1. O estado, com o selo que a colheita vai cobrar.
        let estado = arquivos.conteudo_de(ESTADO).expect("estado gravado");
        assert!(estado.contains(&format!("\"selo\": \"{SELO}\"")));
        assert!(estado.contains("\"situacao\": \"armado\""));
        assert_eq!(armado.selo.como_texto(), SELO);

        // 2. O `grub.cfg`, com o bloco e o `set default` apontando para ele.
        let grub = arquivos.conteudo_de(GRUB).expect("grub gravado");
        assert!(grub.contains(&format!("set default=\"{ID_DO_ARCA}\"")));
        assert!(grub.contains(&format!("ARCA_SELO={SELO}")));
        // A configuracao deste dispositivo atravessou (ver `crate::menuentry`).
        assert!(grub.contains("nvme.poll_queues=1"));

        // 3. A marca de boot unico, mandada e conferida — e nada alem disso
        //    escrito no firmware. A entrada ja se chamava `ARCA`, entao nao ha
        //    migracao; o alvo e escrito sempre, pelo motivo em
        //    `apontar_para_o_arcaboot`.
        assert_eq!(
            *ferramenta.executados.borrow(),
            vec![
                vec!["/set", GUID, "device", "partition=R:"],
                vec!["/set", FWBOOTMGR, BOOTSEQUENCE, GUID],
            ]
            .into_iter()
            .map(|argumentos| argumentos.into_iter().map(String::from).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
            "as escritas no bcdedit, na ordem"
        );
        assert_eq!(armado.entrada, Entrada::JaEraDoArca);
        assert_eq!(armado.alvo, Alvo::ParticaoComLetra('R'));
    }

    #[test]
    fn o_grub_cfg_armado_desarma_de_volta_para_o_inerte() {
        // O que a E7 escreve, a E4 desfaz. Se algum dia o bloco derivado
        // deixar de ser removivel, e aqui que aparece.
        let arquivos = arquivos();
        armar_com(&arquivos, &firmware_desta_maquina()).expect("arma");

        let armado = arquivos.conteudo_de(GRUB).unwrap();
        assert_eq!(grub::desarmar(&armado).expect("desarma").texto, INERTE);
    }

    #[test]
    fn a_entrada_legada_e_renomeada_e_nao_duplicada() {
        // C-4. A captura de 20/08 e a unica evidencia real do caso "nao ha
        // entrada ARCA, ha a legada Clonezilla", e o alvo e o mesmo GUID.
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, LEGADO)
            .respondendo_depois(FIRMWARE, PT)
            .respondendo(FWBOOTMGR, &fwbootmgr(None))
            .respondendo_depois(FWBOOTMGR, &fwbootmgr(Some(GUID)));

        let armado = armar_com(&arquivos, &ferramenta).expect("arma");

        assert_eq!(
            armado.entrada,
            Entrada::MigradaDaLegada {
                de: "Clonezilla".to_string()
            }
        );
        assert_eq!(armado.identificador, GUID);

        let escritas = ferramenta.executados.borrow();
        assert!(
            escritas.contains(&vec![
                "/set".to_string(),
                GUID.to_string(),
                "description".to_string(),
                "ARCA".to_string(),
            ]),
            "a descricao nao foi migrada: {escritas:?}"
        );
        assert!(
            !escritas.iter().any(|argumentos| argumentos.contains(&"/create".to_string())),
            "criou uma entrada em vez de migrar: {escritas:?}"
        );
    }

    #[test]
    fn uma_migracao_que_o_bcdedit_recusa_nao_passa_por_migrada() {
        // C-4 com C-3. O `bcdedit` recusa a escrita — sem privilegio, por
        // exemplo — e a releitura mostra a entrada ainda chamada `Clonezilla`.
        // O ARCA nao arma sobre uma entrada que nao sabe se mexeu.
        //
        // **Este teste so passou a provar alguma coisa depois da revisao desta
        // etapa.** O duplo tinha dois construtores que pareciam compor e nao
        // compunham: com o `{fwbootmgr}` modelado, a recusa injetada nunca
        // disparava, e um teste escrito assim passava verde sem exercitar
        // caminho nenhum.
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, LEGADO)
            .modelando_o_fwbootmgr(&["{bootmgr}"])
            .recusando_o_executar(Erro::FerramentaRecusou {
                ferramenta: "bcdedit",
                codigo: 1,
                saida: "Acesso negado.".to_string(),
            });

        match armar_com(&arquivos, &ferramenta).unwrap_err() {
            Erro::EntradaNaoMigrou { de, tem, .. } => {
                assert_eq!(de, "Clonezilla");
                assert_eq!(tem, "Clonezilla");
            }
            outro => panic!("esperava a migracao que nao pegou, veio {outro}"),
        }

        assert!(arquivos.conteudo_de(ESTADO).is_none(), "gravou estado");
        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));
    }

    #[test]
    fn sem_entrada_de_firmware_o_arca_recusa_em_vez_de_criar_uma() {
        // Criar uma entrada do zero e codigo sem original, e o lugar disso e o
        // `arca prepare` da E10. Armar nao e a hora de estrear.
        let arquivos = arquivos();
        let sem_entrada = format!(
            "\r\nGerenciador de Inicialização de Firmware\r\n\
             ----------------------------------------\r\n\
             identificador           {FWBOOTMGR}\r\n\
             displayorder            {{bootmgr}}\r\n\
             timeout                 1\r\n"
        );
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, &sem_entrada)
            .respondendo(FWBOOTMGR, &fwbootmgr(None));

        assert!(matches!(
            armar_com(&arquivos, &ferramenta).unwrap_err(),
            Erro::SemEntradaDeFirmware
        ));
        assert!(arquivos.conteudo_de(ESTADO).is_none(), "gravou estado");
        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));
    }

    #[test]
    fn a_rejeicao_silenciosa_do_bcdedit_e_pega_pela_releitura() {
        // C-6 e §3.1: com o alvo em midia removivel o `bcdedit` responde
        // "êxito" e **mantem o valor antigo**. Nao ha etiqueta a lê — quem
        // revela e o `device` que nao mudou. E a primeira vez que este
        // caminho e exercitado no projeto.
        let arquivos = arquivos();
        let apontando_para_outro = PT.replace("partition=R:", "partition=Z:");
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, &apontando_para_outro)
            .respondendo(FWBOOTMGR, &fwbootmgr(None));

        match armar_com(&arquivos, &ferramenta).unwrap_err() {
            Erro::AlvoDoFirmwareRecusado { esperado, tem, .. } => {
                assert_eq!(esperado, "partition=R:");
                assert_eq!(tem, "partition=Z:");
            }
            outro => panic!("esperava o alvo recusado, veio {outro}"),
        }

        assert!(arquivos.conteudo_de(ESTADO).is_none(), "gravou estado");
        assert_eq!(
            arquivos.conteudo_de(GRUB).as_deref(),
            Some(INERTE),
            "armou o grub.cfg com a entrada apontando para o lugar errado"
        );
    }

    #[test]
    fn a_marca_que_nao_pega_e_falha_com_o_grub_ja_armado() {
        // O terceiro dos tres estados intermediarios, e o que o cabecalho
        // deste modulo descreve: estado e receita gravados, marca nao. A
        // maquina reinicia no Windows, e `arca desarmar` resolve. O que **nao**
        // pode acontecer e o comando seguir para o reinicio.
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, PT)
            .respondendo(FWBOOTMGR, &fwbootmgr(None));

        match armar_com(&arquivos, &ferramenta).unwrap_err() {
            Erro::BootUnicoNaoArmou { identificador, tem } => {
                assert_eq!(identificador, GUID);
                assert_eq!(tem, "nenhuma marca");
            }
            outro => panic!("esperava a marca que nao pegou, veio {outro}"),
        }

        assert!(arquivos.conteudo_de(ESTADO).is_some(), "o job foi registrado");
        assert!(
            arquivos.conteudo_de(GRUB).unwrap().contains(ID_DO_ARCA),
            "o grub.cfg ficou armado, que e o estado nomeado"
        );
    }

    #[test]
    fn uma_marca_apontando_para_outra_entrada_nao_passa_por_armada() {
        // "Ha algum boot unico" nao basta. Um `bootsequence` deixado por outra
        // coisa faria o ARCA dizer que armou enquanto a maquina bootaria em
        // outro lugar — e o reinicio viria logo depois.
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, PT)
            .respondendo(FWBOOTMGR, &fwbootmgr(None))
            .respondendo_depois(FWBOOTMGR, &fwbootmgr(Some("{outra-entrada}")));

        match armar_com(&arquivos, &ferramenta).unwrap_err() {
            Erro::BootUnicoNaoArmou { tem, .. } => assert_eq!(tem, "{outra-entrada}"),
            outro => panic!("esperava a marca de outra entrada, veio {outro}"),
        }
    }

    #[test]
    fn mexer_na_ordem_permanente_e_falha_mesmo_com_o_boot_unico_armado() {
        // C-5. A conferencia vem **antes** da do boot unico de proposito: uma
        // ordem permanente alterada e falha ainda que a marca tenha pegado, e
        // relatar exito ali seria o ARCA dizer que fez o certo depois de mudar
        // para sempre a configuracao de boot da maquina.
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, PT)
            .respondendo(FWBOOTMGR, &fwbootmgr(None))
            .respondendo_depois(
                FWBOOTMGR,
                &fwbootmgr(Some(GUID)).replace("{bootmgr}", "{bootmgr}\r\n                        {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"),
            );

        match armar_com(&arquivos, &ferramenta).unwrap_err() {
            Erro::OrdemPermanenteAlterada { antes, depois } => {
                assert_eq!(antes, "{bootmgr}");
                assert!(depois.contains("f4057bd0"), "veio {depois}");
            }
            outro => panic!("esperava a ordem permanente alterada, veio {outro}"),
        }
    }

    #[test]
    fn uma_releitura_ilegivel_nao_vira_armado() {
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, PT)
            .respondendo(FWBOOTMGR, &fwbootmgr(None))
            .respondendo_depois(FWBOOTMGR, "isto nao e uma enumeracao\r\n");

        assert!(matches!(
            armar_com(&arquivos, &ferramenta).unwrap_err(),
            Erro::FirmwareIlegivel { alvo: FWBOOTMGR }
        ));
    }

    #[test]
    fn sem_privilegio_nada_e_gravado() {
        let arquivos = arquivos();
        let ferramenta = FirmwareDeMentira::novo().recusando_o_enumerar(Erro::FerramentaRecusou {
            ferramenta: "bcdedit",
            codigo: 1,
            saida: "Acesso negado.".to_string(),
        });

        assert!(armar_com(&arquivos, &ferramenta).is_err());
        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));
        assert!(arquivos.conteudo_de(ESTADO).is_none());
        assert!(ferramenta.executados.borrow().is_empty());
    }

    #[test]
    fn armar_um_grub_cfg_ja_armado_e_recusado() {
        // C-1 diz que quem arma desarma antes. Se isso falhar em algum
        // chamador futuro, o `grub::armar` recusa em vez de deixar dois blocos
        // com o mesmo `--id` gravados em disco.
        let armada = include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg");
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, armada);

        assert!(matches!(
            armar_com(&arquivos, &firmware_desta_maquina()).unwrap_err(),
            Erro::GrubRecusado(grub::RecusaDoGrub::JaArmado)
        ));
        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(armada));
    }

    #[test]
    fn sem_selo_nao_se_arma() {
        // C-11: um job sem selo e um job cujo desfecho ninguem consegue
        // reclamar. A recusa acontece **antes** das tres gravacoes.
        let arquivos = arquivos();
        let dispositivo = dispositivo();
        let nome = nome();
        let disco = disco();

        let erro = executar(
            &arquivos,
            &firmware_desta_maquina(),
            &EntropiaDeMentira::recusando(),
            &RelogioParado::em("2026-08-22T19:14:03"),
            &Pedir {
                dispositivo: &dispositivo,
                operacao: Operacao::Backup,
                nome: &nome,
                disco: &disco,
            },
        )
        .unwrap_err();

        assert!(matches!(erro, Erro::EntropiaIndisponivel { .. }));
        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));
        assert!(arquivos.conteudo_de(ESTADO).is_none());
    }

    #[test]
    fn o_estado_gravado_e_relido_pelo_leitor_da_e5() {
        // A ida e a volta do `estado.json` atravessando o que a E7 escreve de
        // verdade — e nao um `Estado` montado por teste. E o unico jeito de o
        // campo novo da E8 nao passar despercebido aqui.
        let arquivos = arquivos();
        let armado = armar_com(&arquivos, &firmware_desta_maquina()).expect("arma");

        let lido = estado::ler(&arquivos, &armado.caminho_do_estado).expect("relê");
        assert_eq!(lido.selo, armado.selo);
        assert_eq!(lido.comando, Operacao::Backup);
        assert_eq!(lido.nome.como_texto(), "2026-08-22_Apps");
        assert_eq!(lido.disco.como_texto(), "nvme0n1");
        assert_eq!(lido.situacao, estado::Situacao::Armado);
    }
}
