//! Desarmar: devolver o `grub.cfg` ao estado inerte e limpar a marca de boot
//! unico (C-1).
//!
//! Duas metades, uma em cada fronteira. A do `grub.cfg` e escrita de arquivo,
//! e o texto vem de [`crate::grub`], que e puro. A da marca de boot unico e
//! uma conversa com o `bcdedit`, e o que ele responde nao serve de prova.
//!
//! **Incondicional, e sem consultar estado nenhum.** Nao ha `if` perguntando
//! se ha job pendente, nem leitura de `estado.json`, nem decisao tomada a
//! partir do que o firmware diz. Desarmar acontece; o que se lê depois e para
//! conferir se aconteceu, nunca para decidir se deveria acontecer.
//!
//! # C-1 e C-3 nao brigam, valem em momentos diferentes
//!
//! C-1 proibe consultar estado **antes de decidir**: desarmar nao pergunta.
//! C-3 exige conferir com `/enum` **depois de escrever**: o sucesso do
//! `bcdedit` nunca e prova. As duas valem, e aqui as duas aparecem — a
//! primeira na ausencia de condicional, a segunda na releitura do fim.
//!
//! # O `bcdedit` chama de erro nao ter o que apagar
//!
//! Medido em 22/08/2026, nesta maquina, com o `{fwbootmgr}` sem
//! `bootsequence`:
//!
//! ```text
//! > bcdedit /deletevalue {fwbootmgr} bootsequence
//! Erro ao tentar excluir o elemento de dados especificado.
//! Elemento não encontrado.
//! (codigo de saida 1)
//! ```
//!
//! O `/enum` antes e depois sai identico: ele **nao muda nada** e ainda assim
//! sai com codigo diferente de zero. Isso importa porque
//! [`crate::adaptadores::windows::firmware::Bcdedit`] transforma codigo ≠ 0
//! em erro — e com razao, porque e assim que "Acesso negado" chega. Um
//! desarmar que propagasse esse erro **falharia justamente no caso normal**,
//! que e o dispositivo ja estar inerte, e a segunda das duas passadas que C-1
//! exige nunca passaria.
//!
//! A saida nao e ler o texto da recusa para distinguir "nao havia nada" de
//! "nao pude olhar" — sao frases, em dois idiomas, e interpretar frase e o que
//! C-3 quer evitar. A saida e **nao acreditar no `bcdedit` em nenhum dos dois
//! sentidos**: manda apagar, ignora o que ele responde, e pergunta de novo. Se
//! a marca sumiu — ou nunca esteve la —, desarmou. Se continua, e falha. E se
//! foi falta de privilegio, a releitura falha junto, porque `bcdedit /enum`
//! sem privilegio tambem sai com codigo 1.
//!
//! # Apagar o `bootsequence` nao viola B-10
//!
//! B-10 diz que o ARCA nunca apaga nada, e fala de **imagem, residuo e log** —
//! do que o usuario perderia. A marca de boot unico nao e nada disso: e uma
//! intencao que o proprio ARCA gravou, e desfaze-la e o que C-1 manda fazer.
//! `tests/b10_nada_e_apagado.rs` varre o codigo atras de exclusao de
//! **arquivo**, e nao distingue os dois casos — dai valer deixar isto escrito
//! onde alguem va procurar.

use crate::erro::{Erro, Resultado};
use crate::firmware;
use crate::grub;
use crate::portas::{Arquivos, Firmware};
use std::path::{Path, PathBuf};

/// O objeto do `bcdedit` que guarda a ordem de boot e a marca de boot unico.
const FWBOOTMGR: &str = "{fwbootmgr}";

/// O elemento que carrega o boot unico armado.
const BOOTSEQUENCE: &str = "bootsequence";

/// O que aconteceu com a marca de boot unico.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarcaDeBootUnico {
    /// Nao havia nenhuma. E o estado normal, e e o que a segunda passada de
    /// C-1 encontra sempre.
    NaoHavia,

    /// Havia, e a releitura confirma que sumiu.
    Removida { entradas: Vec<String> },
}

/// O que o desarmar fez, para o registro e para a tela.
///
/// Nada aqui decide coisa alguma: desarmar ja aconteceu. Serve para quem roda
/// o comando saber se havia receita armada ou se o dispositivo ja estava
/// inerte — e para o caminho aparecer na tela, que e a unica defesa barata
/// contra desarmar o dispositivo errado enquanto a E6 nao sabe dizer em que
/// disco fisico cada volume esta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Desarme {
    pub caminho_do_grub: PathBuf,
    pub blocos_removidos: usize,
    pub default_devolvido: bool,

    /// Se o `grub.cfg` precisou ser regravado.
    ///
    /// A operacao e incondicional; a **gravacao** e consequencia dela. Um
    /// `grub.cfg` que ja estava inerte sai da transformacao byte a byte igual
    /// ao que entrou, e regrava-lo nao mudaria nada — so abriria de novo a
    /// janela em que um desligamento pega o arquivo entre o temporario e a
    /// renomeacao, num arquivo de que a maquina depende para bootar.
    pub grub_regravado: bool,

    pub boot_unico: MarcaDeBootUnico,
}

impl Desarme {
    /// Se havia alguma coisa armada — no `grub.cfg` ou no firmware.
    pub fn havia_job(&self) -> bool {
        self.blocos_removidos > 0
            || self.default_devolvido
            || self.boot_unico != MarcaDeBootUnico::NaoHavia
    }
}

/// Desarma: `grub.cfg` primeiro, marca de boot unico depois.
///
/// # Por que nesta ordem
///
/// Uma das duas metades pode falhar depois de a outra ter passado, e as duas
/// ordens deixam estados diferentes:
///
/// - `grub.cfg` inerte com a marca ainda no firmware: a maquina reinicia no
///   dispositivo e **para no menu normal do Clonezilla**. E o §6.3 do PRD,
///   sem nada de mais.
/// - `grub.cfg` armado com a marca ja limpa: a maquina reinicia no Windows,
///   e o dispositivo fica com uma receita esperando quem bootar nele por F12.
///
/// O primeiro se resolve olhando uma tela; o segundo fica guardado. Dai o
/// `grub.cfg` primeiro.
///
/// A leitura do firmware acontece **antes** de qualquer escrita, de proposito:
/// sem privilegio administrativo ela falha ali, e nada chegou a ser gravado.
pub fn executar(
    arquivos: &dyn Arquivos,
    ferramenta: &dyn Firmware,
    caminho_do_grub: &Path,
) -> Resultado<Desarme> {
    // Falha cedo, antes de escrever: sem privilegio, o `bcdedit` recusa aqui.
    let antes = firmware::ler(&ferramenta.enumerar(FWBOOTMGR)?);

    let corrente = arquivos.ler_texto(caminho_do_grub)?;
    let desarmado = grub::desarmar(&corrente).map_err(Erro::GrubRecusado)?;

    let grub_regravado = desarmado.texto != corrente;
    if grub_regravado {
        arquivos.escrever_atomico(caminho_do_grub, &desarmado.texto)?;
    }

    let boot_unico = limpar_a_marca(ferramenta, &antes)?;

    Ok(Desarme {
        caminho_do_grub: caminho_do_grub.to_path_buf(),
        blocos_removidos: desarmado.blocos_removidos,
        default_devolvido: desarmado.default_devolvido,
        grub_regravado,
        boot_unico,
    })
}

/// Manda apagar o `bootsequence` e confere com `/enum` que ele sumiu (C-3).
///
/// O que o `bcdedit` responde e descartado — nos dois sentidos. "êxito" nao
/// prova que apagou, e a recusa nao prova que ha problema: sem
/// `bootsequence`, apagar o `bootsequence` **e** uma recusa. Quem responde e
/// a releitura.
fn limpar_a_marca(
    ferramenta: &dyn Firmware,
    antes: &firmware::Leitura,
) -> Resultado<MarcaDeBootUnico> {
    // Sem `if antes.tem_boot_unico()`, e nao por descuido. A leitura de
    // `antes` existe para conferir C-5 depois e para falhar cedo sem
    // privilegio — **nunca** para decidir se o `deletevalue` acontece. Pular a
    // escrita quando a leitura diz que nao ha marca pareceria uma otimizacao
    // obvia e seria exatamente o que C-1 proibe: um desarmar que so desarma
    // quando ja acha que precisa e um desarmar que confia na leitura, e a
    // leitura pode estar errada. Ele custa uma chamada de processo.
    let _ = ferramenta.executar(&["/deletevalue", FWBOOTMGR, BOOTSEQUENCE]);

    let depois = firmware::ler(&ferramenta.enumerar(FWBOOTMGR)?);

    // "Nao entendi a resposta" nao pode virar "a marca sumiu". O parser nunca
    // falha por desenho — texto que ele nao reconhece vira leitura vazia, e
    // leitura vazia tem `boot_unico` vazio, que e indistinguivel de estar
    // inerte. Um `bcdedit` que saisse zero com a saida noutro formato faria o
    // ARCA dizer "nao havia" com o boot unico ainda armado, e o proximo
    // reinicio rodaria a receita velha — exatamente o que C-3 existe para
    // impedir. A comparacao de C-5 logo abaixo ficaria vazia junto, comparando
    // duas listas vazias.
    if !depois.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    if depois.tem_boot_unico() {
        return Err(Erro::BootUnicoPersistente {
            entradas: depois.boot_unico.join(", "),
        });
    }

    // C-5: o boot unico nunca altera a ordem permanente, e o desarmar tambem
    // nao pode. Conferir custa nada — a leitura ja aconteceu — e e a unica
    // chance de pegar um `bcdedit` que tenha mexido no que nao devia.
    if depois.ordem_permanente != antes.ordem_permanente {
        return Err(Erro::OrdemPermanenteAlterada {
            antes: antes.ordem_permanente.join(", "),
            depois: depois.ordem_permanente.join(", "),
        });
    }

    Ok(if antes.tem_boot_unico() {
        MarcaDeBootUnico::Removida {
            entradas: antes.boot_unico.clone(),
        }
    } else {
        MarcaDeBootUnico::NaoHavia
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{ArquivosEmMemoria, FirmwareDeMentira};
    use crate::grub::{ID_DO_ARCA, ID_INERTE};

    const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");
    const ARMADA: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg");

    const GRUB: &str = r"R:\boot\grub\grub.cfg";
    const ALVO: &str = "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";

    /// Um `{fwbootmgr}` como o `bcdedit` o enumera, com ou sem boot unico.
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

    /// A recusa real do `bcdedit` quando nao ha `bootsequence` para apagar.
    ///
    /// Medida em 22/08/2026: codigo de saida 1, e nada muda. E este o caso
    /// normal, e e ele que um desarmar ingenuo transformaria em falha.
    fn recusa_do_deletevalue() -> Erro {
        Erro::FerramentaRecusou {
            ferramenta: "bcdedit",
            codigo: 1,
            saida:
                "Erro ao tentar excluir o elemento de dados especificado.\nElemento não encontrado."
                    .to_string(),
        }
    }

    #[test]
    fn desarmar_devolve_o_grub_cfg_ao_inerte_e_grava() {
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, ARMADA);
        let ferramenta = FirmwareDeMentira::novo().respondendo(FWBOOTMGR, &fwbootmgr(None));

        let desarme = executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("desarma");

        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));
        assert!(desarme.grub_regravado);
        assert_eq!(desarme.blocos_removidos, 1);
        assert!(desarme.default_devolvido);
        assert!(desarme.havia_job());
    }

    #[test]
    fn duas_passadas_seguidas_dao_o_mesmo_resultado() {
        // C-1 na letra, atravessando as duas fronteiras: o arquivo e o
        // `bcdedit`. E a segunda passada e a que um desarmar ingenuo
        // quebraria, porque e nela que o `deletevalue` recusa.
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, ARMADA);
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &fwbootmgr(Some(ALVO)))
            .respondendo_depois(FWBOOTMGR, &fwbootmgr(None))
            .recusando_o_executar(recusa_do_deletevalue());

        let primeira = executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("primeira");
        let depois_da_primeira = arquivos.conteudo_de(GRUB);

        let segunda = executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("segunda");

        assert_eq!(arquivos.conteudo_de(GRUB), depois_da_primeira);
        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));

        assert!(primeira.havia_job(), "a primeira achou o que desarmar");
        assert!(!segunda.havia_job(), "a segunda achou tudo ja inerte");
        assert!(!segunda.grub_regravado, "regravou um arquivo identico");
        assert_eq!(segunda.boot_unico, MarcaDeBootUnico::NaoHavia);
    }

    #[test]
    fn o_bcdedit_recusando_por_nao_haver_o_que_apagar_nao_e_falha() {
        // O achado que muda o desenho. Sem `bootsequence`, o `bcdedit`
        // responde "Elemento não encontrado" e sai com codigo 1 — sem mudar
        // nada. Propagar isso faria o desarmar falhar no caso normal.
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, INERTE);
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &fwbootmgr(None))
            .recusando_o_executar(recusa_do_deletevalue());

        let desarme = executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("nao e falha");

        assert_eq!(desarme.boot_unico, MarcaDeBootUnico::NaoHavia);
        assert!(!desarme.havia_job());
    }

    #[test]
    fn a_marca_de_boot_unico_e_mandada_apagar_pelo_bcdedit() {
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, INERTE);
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &fwbootmgr(Some(ALVO)))
            .respondendo_depois(FWBOOTMGR, &fwbootmgr(None));

        let desarme = executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("desarma");

        assert_eq!(
            desarme.boot_unico,
            MarcaDeBootUnico::Removida {
                entradas: vec![ALVO.to_string()]
            }
        );
        assert_eq!(
            *ferramenta.executados.borrow(),
            vec![vec![
                "/deletevalue".to_string(),
                FWBOOTMGR.to_string(),
                BOOTSEQUENCE.to_string()
            ]]
        );
    }

    #[test]
    fn a_marca_que_sobrevive_a_escrita_e_falha() {
        // C-3: o sucesso do `bcdedit` nunca e prova. Um `bcdedit` que
        // respondesse "êxito" e mantivesse a marca deixaria a maquina com boot
        // unico pendente e o ARCA dizendo que desarmou.
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, INERTE);
        let ferramenta = FirmwareDeMentira::novo().respondendo(FWBOOTMGR, &fwbootmgr(Some(ALVO)));

        match executar(&arquivos, &ferramenta, Path::new(GRUB)).unwrap_err() {
            Erro::BootUnicoPersistente { entradas } => assert_eq!(entradas, ALVO),
            outro => panic!("esperava a marca persistente, veio {outro}"),
        }
    }

    #[test]
    fn uma_releitura_que_o_parser_nao_entende_e_falha_e_nao_desarmou() {
        // O parser nunca falha por desenho: texto irreconhecivel vira leitura
        // vazia, e leitura vazia tem `boot_unico` vazio — indistinguivel de
        // estar inerte. Sem esta guarda, um `bcdedit` que saisse zero com
        // outra saida faria o ARCA dizer "nao havia" com a marca ainda
        // armada, e o proximo reinicio rodaria a receita velha.
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, INERTE);
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &fwbootmgr(Some(ALVO)))
            .respondendo_depois(FWBOOTMGR, "alguma coisa que nao e uma enumeracao\r\n");

        match executar(&arquivos, &ferramenta, Path::new(GRUB)).unwrap_err() {
            Erro::FirmwareIlegivel { alvo } => assert_eq!(alvo, FWBOOTMGR),
            outro => panic!("esperava a releitura ilegivel, veio {outro}"),
        }
    }

    #[test]
    fn mexer_na_ordem_permanente_e_falha() {
        // C-5: o desarmar nao tem nada que ver com a ordem permanente, e a
        // releitura ja esta na mao — conferir custa nada.
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, INERTE);
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FWBOOTMGR, &fwbootmgr(Some(ALVO)))
            .respondendo_depois(
                FWBOOTMGR,
                &fwbootmgr(None).replace("{bootmgr}", "{outra-coisa}"),
            );

        match executar(&arquivos, &ferramenta, Path::new(GRUB)).unwrap_err() {
            Erro::OrdemPermanenteAlterada { antes, depois } => {
                assert_eq!(antes, "{bootmgr}");
                assert_eq!(depois, "{outra-coisa}");
            }
            outro => panic!("esperava a ordem permanente alterada, veio {outro}"),
        }
    }

    #[test]
    fn sem_privilegio_nada_e_gravado() {
        // A leitura do firmware vem antes de qualquer escrita justamente para
        // isto: um `bcdedit` que recusa por falta de privilegio nao pode
        // deixar o `grub.cfg` mexido pela metade.
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, ARMADA);
        let ferramenta = FirmwareDeMentira::novo().recusando_o_enumerar(Erro::FerramentaRecusou {
            ferramenta: "bcdedit",
            codigo: 1,
            saida: "Acesso negado.".to_string(),
        });

        assert!(executar(&arquivos, &ferramenta, Path::new(GRUB)).is_err());
        assert_eq!(
            arquivos.conteudo_de(GRUB).as_deref(),
            Some(ARMADA),
            "o grub.cfg foi tocado apesar da recusa"
        );
        assert!(ferramenta.executados.borrow().is_empty());
    }

    #[test]
    fn o_grub_cfg_que_nao_da_para_entender_e_recusado_sem_ser_gravado() {
        // Um `grub.cfg` truncado e uma maquina que nao boota; um armado ainda
        // boota. Na duvida, nao gravar.
        let quebrado = format!(
            "set default=\"{ID_INERTE}\"\nmenuentry \"x\" --id {ID_DO_ARCA} {{\n  sem fechamento\n"
        );
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, &quebrado);
        let ferramenta = FirmwareDeMentira::novo().respondendo(FWBOOTMGR, &fwbootmgr(None));

        assert!(matches!(
            executar(&arquivos, &ferramenta, Path::new(GRUB)).unwrap_err(),
            Erro::GrubRecusado(_)
        ));
        assert_eq!(
            arquivos.conteudo_de(GRUB).as_deref(),
            Some(quebrado.as_str())
        );
        assert!(
            ferramenta.executados.borrow().is_empty(),
            "mexeu no firmware com o grub.cfg recusado"
        );
    }

    #[test]
    fn desarmar_nao_consulta_estado_nenhum() {
        // C-1 na letra: "sem consultar estado nenhum". Nao basta que o
        // `estado.json` nao **mude** — o desarmar nao pode nem olhar para
        // ele, porque olhar so faria sentido para decidir, e nao ha decisao a
        // tomar. O selo e o `estado.json` sao da E5, e nada aqui os alcanca.
        let estado = r"R:\arca\estado.json";
        let selo = r#"{"selo":"a3f1c9e07b2d4856"}"#;

        let arquivos = ArquivosEmMemoria::novo()
            .com(GRUB, ARMADA)
            .com(estado, selo);
        let ferramenta = FirmwareDeMentira::novo().respondendo(FWBOOTMGR, &fwbootmgr(None));

        executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("desarma");

        assert!(
            !arquivos.foi_consultado(estado),
            "o desarmar olhou para o estado do job"
        );
        assert!(
            arquivos.foi_consultado(GRUB),
            "o desarmar precisa lê o grub.cfg que vai reescrever"
        );
        assert_eq!(arquivos.conteudo_de(estado).as_deref(), Some(selo));
    }

    #[test]
    fn o_desarme_desarma_um_grub_cfg_que_o_arca_nunca_viu() {
        // Um dispositivo armado a mao, ou por uma versao antiga do ARCA, ou
        // com o `set default` do Clonezilla puro. Desarmar nao pressupoe que
        // quem armou foi este binario — e nem poderia, porque nao consulta
        // estado.
        let clonezilla = include_str!("../recursos/capturas/grub-clonezilla-original.cfg");
        let arquivos = ArquivosEmMemoria::novo().com(GRUB, clonezilla);
        let ferramenta = FirmwareDeMentira::novo().respondendo(FWBOOTMGR, &fwbootmgr(None));

        let desarme = executar(&arquivos, &ferramenta, Path::new(GRUB)).expect("desarma");

        assert_eq!(arquivos.conteudo_de(GRUB).as_deref(), Some(INERTE));
        assert!(desarme.default_devolvido, "o `set default=0` foi devolvido");
        assert_eq!(desarme.blocos_removidos, 0);
    }
}
