//! Devolver o `{bootmgr}` ao topo da ordem permanente (C-13, P-20).
//!
//! # Por que isto existe, e por que nao existia
//!
//! O ciclo de boot pelo dispositivo poe a entrada do ARCA na ordem permanente
//! — medido no marco da E7, e explicado no
//! [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md). O
//! ARCA nao a poe: C-5 proibe, e tanto o armar quanto o desarme releem para
//! conferir. Mas depois de um backup ela esta la, **em primeiro**, e a partir
//! dai ligar a maquina com o SSD conectado boota nele.
//!
//! O dispositivo esta inerte, entao isso para no menu do Clonezilla e espera
//! alguem. Nao ha risco; ha friccao, e ela e paga em **todo** boot ate alguem
//! desconectar o SSD. Nao era assim antes de o ARCA existir.
//!
//! O ADR-0009 decidiu **avisar em vez de consertar**, e registrou no mesmo dia
//! que a decisao tinha pedido de revisao. O
//! [ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md) a
//! supersede: colher devolve o `{bootmgr}` ao topo.
//!
//! # O que separa isto de violar C-5
//!
//! C-5 foi escrito contra o ARCA **acrescentar** um caminho permanente para o
//! dispositivo — "desfeito o job, a maquina continuaria com um caminho a
//! mais". Isto nao acrescenta caminho nenhum: poe o Windows na frente dos que
//! ja existem, e **nao remove nada**. Depois de armar, a ordem continua sendo
//! exatamente a que o ARCA encontrou; o que muda e quem esta em primeiro.
//!
//! C-5 continua valendo inteiro onde foi escrito para valer: no armar e no
//! desarme, que releem a ordem e falham se ela mudou.
//!
//! # `/addfirst`, e nao `/remove`
//!
//! Os dois foram medidos a mao em 23/08/2026, antes de virar codigo — como a
//! E7 fez com o `bootsequence`:
//!
//! ```text
//! /set {fwbootmgr} displayorder {ARCA}    /addfirst  → exit 0 · ARCA ao topo
//! /set {fwbootmgr} displayorder {bootmgr} /addfirst  → exit 0 · Windows ao topo, ARCA em segundo
//! /set {fwbootmgr} displayorder {bootmgr} /addfirst  → exit 0 · nada muda (idempotente)
//! /set {fwbootmgr} displayorder {ARCA}    /remove    → exit 0 · sai da ordem, objeto sobrevive
//! ```
//!
//! O `/remove` faria a ordem voltar **literalmente** ao que era antes de o ARCA
//! existir, e mesmo assim ficou de fora. A razao e o modo de falha: `/remove`
//! precisa acertar **quais** entradas tirar, e "quais levam ao dispositivo" e
//! uma pergunta que esta maquina ja respondeu errado uma vez — a revisao do
//! marco da E8 pegou a linha do `arca status` procurando pela entrada chamada
//! `ARCA` enquanto quem levava ao dispositivo era a `{687478f2}` `UEFI OS`,
//! que o firmware criou.
//!
//! `/addfirst {bootmgr}` nao faz essa pergunta. E um identificador fixo, que
//! nao depende de identificar coisa nenhuma, e o resultado vale para **todas**
//! as entradas do dispositivo de uma vez — inclusive as que o firmware ainda
//! nao criou. Uma escrita com um alvo constante tem menos como errar do que N
//! escritas com alvos deduzidos.
//!
//! # A escrita e incondicional, e a leitura de antes so serve para a mensagem
//!
//! O mesmo raciocinio de [`crate::desarme`] sobre o `deletevalue`: decidir
//! escrever a partir da leitura e confiar na leitura, e ela pode estar errada.
//! O `/addfirst` sobre uma ordem ja consertada foi medido — sai com codigo 0 e
//! nao muda nada —, entao a escrita incondicional custa uma chamada de
//! processo e nao custa mais nada.
//!
//! O que a leitura de antes responde e **o que dizer na tela**: se havia algo a
//! consertar, e o que estava na frente. Um `ok` sobre acao que nao aconteceu e
//! a mentira que este projeto ja contou duas vezes (§11).

use crate::erro::{Erro, Resultado};
use crate::firmware::{self, Leitura};
use crate::portas::Firmware;

const FWBOOTMGR: &str = "{fwbootmgr}";
const DISPLAYORDER: &str = "displayorder";
const ADDFIRST: &str = "/addfirst";

/// O alvo que se enumera, e **nao** e o `{fwbootmgr}`.
///
/// A primeira versao lia `{fwbootmgr}`, como [`crate::desarme`] faz, e a
/// execucao real mostrou o que os testes nao mostravam: aquele alvo devolve o
/// bloco do gerenciador **sozinho**, sem as entradas. A ordem vinha certa e
/// [`descrever`] nunca achava a descricao, entao a linha saia
/// `na frente de {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}` — um GUID, para quem
/// abriu a tela querendo saber se pode religar com o SSD na mesa.
///
/// `firmware` traz as duas coisas na mesma chamada, e nao custa uma leitura a
/// mais. Os duplos passavam porque modelam o `{fwbootmgr}` e as entradas no
/// mesmo lugar; o `bcdedit` nao os junta.
const FIRMWARE: &str = "firmware";

/// O gerenciador de inicializacao do Windows, pelo alias que o `bcdedit` usa.
///
/// E um alias, e nao um GUID: o `bcdedit` o aceita na escrita e o devolve na
/// leitura com estas mesmas letras, nas oito capturas deste projeto.
pub const BOOTMGR: &str = "{bootmgr}";

/// O que o conserto encontrou, e o que ele fez.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrdemDevolvida {
    /// O `{bootmgr}` ja era o primeiro. A escrita aconteceu e nao mudou nada.
    JaEstavaNaFrente,
    /// Outra entrada estava na frente, e agora nao esta.
    ///
    /// `estava_na_frente` e a **descricao** da entrada, e nao o GUID, pelo
    /// mesmo motivo da linha do `arca status`: quem lê a tela reconhece
    /// `ARCA`, e nao `{f4057bd0-…}`.
    Devolvida { estava_na_frente: String },
    /// Nao havia ordem permanente nenhuma, e agora ha uma com o Windows.
    ///
    /// Nunca observado nesta maquina. Existe porque um `{fwbootmgr}` sem
    /// `displayorder` e representavel, e chamar isso de "ja estava na frente"
    /// afirmaria que o Windows estava em primeiro quando nao havia primeiro.
    NaoHaviaOrdem,
}

impl OrdemDevolvida {
    /// Se a ordem precisou de conserto.
    pub fn houve_conserto(&self) -> bool {
        !matches!(self, OrdemDevolvida::JaEstavaNaFrente)
    }
}

/// Poe o `{bootmgr}` no topo da ordem permanente e confere com `/enum` (C-3).
///
/// O que o `bcdedit` responde e descartado. Os quatro comandos medidos em
/// 23/08/2026 respondem *"A operação foi concluída com êxito"* e saem com
/// codigo 0 — inclusive o que nao muda nada. Quem responde e a releitura.
pub fn devolver_o_windows(ferramenta: &dyn Firmware) -> Resultado<OrdemDevolvida> {
    let antes = firmware::ler(&ferramenta.enumerar(FIRMWARE)?);

    // "Nao entendi a resposta" nao pode virar "estava tudo bem". O parser
    // nunca falha por desenho, e texto irreconhecivel vira leitura vazia —
    // que tem `ordem_permanente` vazia, indistinguivel de uma ordem sem
    // problema. E o mesmo furo que a revisao do marco da E8 pegou na linha
    // `Ordem de boot` do `arca status`, e aqui ele seria pior: a conferencia
    // de C-3 logo abaixo compararia duas leituras vazias e passaria junto.
    if !antes.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    let estava_na_frente = antes.ordem_permanente.first().cloned();

    let _ = ferramenta.executar(&["/set", FWBOOTMGR, DISPLAYORDER, BOOTMGR, ADDFIRST]);

    let depois = firmware::ler(&ferramenta.enumerar(FIRMWARE)?);
    if !depois.viu_o_gerenciador {
        return Err(Erro::FirmwareIlegivel { alvo: FWBOOTMGR });
    }

    // C-3 sobre a pos-condicao que importa, e nao sobre "o comando deu certo".
    // Um `bcdedit` sem privilegio suficiente responde exito e nao escreve, e e
    // exatamente assim que a rejeicao silenciosa de C-6 chega.
    if !depois
        .ordem_permanente
        .first()
        .is_some_and(|id| id.eq_ignore_ascii_case(BOOTMGR))
    {
        return Err(Erro::OrdemNaoDevolvida {
            ordem: se_vazia(&depois.ordem_permanente),
        });
    }

    Ok(match estava_na_frente {
        Some(id) if id.eq_ignore_ascii_case(BOOTMGR) => OrdemDevolvida::JaEstavaNaFrente,
        Some(id) => OrdemDevolvida::Devolvida {
            estava_na_frente: descrever(&antes, &id),
        },
        None => OrdemDevolvida::NaoHaviaOrdem,
    })
}

/// A descricao da entrada, com o identificador atras dela.
///
/// Nao havendo descricao, sobra o identificador sozinho — que e feio e e
/// verdade. Inventar um nome para uma entrada que nao tem seria a unica coisa
/// pior.
fn descrever(leitura: &Leitura, identificador: &str) -> String {
    match leitura
        .entradas
        .iter()
        .find(|entrada| entrada.identificador.eq_ignore_ascii_case(identificador))
        .and_then(|entrada| entrada.descricao.as_deref())
        .filter(|descricao| !descricao.is_empty())
    {
        Some(descricao) => format!("{descricao} · {identificador}"),
        None => identificador.to_string(),
    }
}

fn se_vazia(ordem: &[String]) -> String {
    if ordem.is_empty() {
        "(vazia)".to_string()
    } else {
        ordem.join(", ")
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::FirmwareDeMentira;

    const ARCA: &str = "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";
    const UEFI_OS: &str = "{687478f2-9e87-11f1-8a47-806e6f6e6963}";

    fn firmware(ordem: &[&str]) -> FirmwareDeMentira {
        FirmwareDeMentira::novo().modelando_o_fwbootmgr(ordem)
    }

    /// O `{fwbootmgr}` como o `bcdedit` desta maquina o enumera.
    ///
    /// Existe porque os dois testes de C-3 abaixo precisam de um firmware que
    /// **nao** obedece, e o modelo do duplo obedece sempre. O primeiro
    /// rascunho deste arquivo montava um trecho a mao, sem o cabecalho da
    /// secao — e o parser o recusou como ilegivel, fazendo o teste passar
    /// pelo motivo errado. Um caso construido mais facil do que o real e a
    /// licao da revisao da E4.
    fn como_o_bcdedit_escreve(ordem: &[&str]) -> String {
        let mut saida = String::from(
            "\r\nGerenciador de Inicialização de Firmware\r\n\
             ----------------------------------------\r\n\
             identificador           {fwbootmgr}\r\n",
        );
        for (indice, valor) in ordem.iter().enumerate() {
            let campo = if indice == 0 { "displayorder" } else { "" };
            saida.push_str(&format!("{campo:<24}{valor}\r\n"));
        }
        saida.push_str("timeout                 1\r\n");
        saida
    }

    #[test]
    fn com_o_dispositivo_na_frente_a_ordem_e_devolvida() {
        let ferramenta = firmware(&[ARCA, BOOTMGR]);

        let devolvida = devolver_o_windows(&ferramenta).expect("o conserto roda");

        assert!(devolvida.houve_conserto());
        assert!(
            matches!(&devolvida, OrdemDevolvida::Devolvida { estava_na_frente } if estava_na_frente.contains(ARCA)),
            "a linha tem de nomear quem estava na frente: {devolvida:?}"
        );
        assert_eq!(
            ferramenta.ordem_permanente(),
            vec![BOOTMGR.to_string(), ARCA.to_string()],
            "o Windows tem de ter ido ao topo, e a entrada do ARCA tem de continuar la"
        );
    }

    #[test]
    fn a_linha_nomeia_a_entrada_que_estava_na_frente_e_nao_so_o_guid() {
        // **Este teste nasceu de um defeito que a execucao real pegou e os
        // testes nao pegavam.** A primeira versao lia `{fwbootmgr}`, que
        // devolve o bloco do gerenciador sem as entradas: a ordem vinha certa,
        // a descricao nunca era achada, e a tela dizia
        // `na frente de {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}`.
        //
        // Os duplos passavam porque modelavam os dois alvos com a mesma
        // resposta. O `bcdedit` nao os junta, e agora o duplo tambem nao.
        let entradas = "\r\nGerenciador de Inicialização do Windows\r\n\
                        ---------------------------------------\r\n\
                        identificador           {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n\
                        device                  partition=R:\r\n\
                        description             ARCA\r\n";
        let ferramenta = firmware(&[ARCA, BOOTMGR]).respondendo(FIRMWARE, entradas);

        let devolvida = devolver_o_windows(&ferramenta).expect("o conserto roda");

        let OrdemDevolvida::Devolvida { estava_na_frente } = &devolvida else {
            panic!("tinha de haver conserto: {devolvida:?}");
        };
        assert!(
            estava_na_frente.starts_with("ARCA · "),
            "a linha tem de comecar pelo nome que quem lê reconhece, e nao pelo GUID: {estava_na_frente}"
        );
        assert!(
            estava_na_frente.contains(ARCA),
            "o GUID fica atras do nome, para quem for conferir a mao: {estava_na_frente}"
        );
    }

    #[test]
    fn sem_descricao_sobra_o_identificador_e_nao_um_nome_inventado() {
        // O modelo sem entradas nenhuma e o caso: `{687478f2}` `UEFI OS` nao
        // tinha `description` em nenhuma captura ate 22/08. Um nome inventado
        // aqui mandaria alguem procurar uma entrada que nao existe com aquele
        // nome.
        let ferramenta = firmware(&[UEFI_OS, BOOTMGR]);

        let devolvida = devolver_o_windows(&ferramenta).expect("o conserto roda");

        assert_eq!(
            devolvida,
            OrdemDevolvida::Devolvida {
                estava_na_frente: UEFI_OS.to_string()
            }
        );
    }

    #[test]
    fn a_entrada_do_dispositivo_nao_e_removida_da_ordem() {
        // **`/addfirst` move, e nao remove** — medido em 23/08/2026. Isto e
        // deliberado e nao efeito colateral: remover exigiria acertar quais
        // entradas levam ao dispositivo, que e a pergunta que esta maquina ja
        // respondeu errado uma vez.
        let ferramenta = firmware(&[UEFI_OS, ARCA, BOOTMGR]);

        devolver_o_windows(&ferramenta).expect("o conserto roda");

        let ordem = ferramenta.ordem_permanente();
        assert_eq!(ordem.first().map(String::as_str), Some(BOOTMGR));
        assert_eq!(
            ordem.len(),
            3,
            "nenhuma entrada podia sair da ordem: {ordem:?}"
        );
        assert!(ordem.contains(&UEFI_OS.to_string()) && ordem.contains(&ARCA.to_string()));
    }

    #[test]
    fn com_o_windows_ja_na_frente_nao_ha_conserto_a_anunciar() {
        let ferramenta = firmware(&[BOOTMGR, ARCA]);

        let devolvida = devolver_o_windows(&ferramenta).expect("o conserto roda");

        assert_eq!(devolvida, OrdemDevolvida::JaEstavaNaFrente);
        assert!(!devolvida.houve_conserto());
    }

    #[test]
    fn a_escrita_acontece_mesmo_com_o_windows_ja_na_frente() {
        // O mesmo que `desarme` faz com o `deletevalue`: nao se pula a escrita
        // porque a leitura disse que nao precisa. Quem responde e a releitura,
        // e o caso normal tem de ser o caso exercitado.
        let ferramenta = firmware(&[BOOTMGR]);

        devolver_o_windows(&ferramenta).expect("o conserto roda");

        assert_eq!(
            ferramenta.executados(),
            vec![vec![
                "/set".to_string(),
                FWBOOTMGR.to_string(),
                DISPLAYORDER.to_string(),
                BOOTMGR.to_string(),
                ADDFIRST.to_string(),
            ]],
            "a escrita tem de acontecer uma vez, e com a forma medida a mao"
        );
    }

    #[test]
    fn rodar_duas_vezes_da_o_mesmo_resultado() {
        let ferramenta = firmware(&[ARCA, BOOTMGR]);

        let primeira = devolver_o_windows(&ferramenta).expect("o conserto roda");
        let segunda = devolver_o_windows(&ferramenta).expect("o conserto roda de novo");

        assert!(primeira.houve_conserto());
        assert_eq!(
            segunda,
            OrdemDevolvida::JaEstavaNaFrente,
            "a segunda passada nao pode achar nada a consertar"
        );
        assert_eq!(
            ferramenta.ordem_permanente(),
            vec![BOOTMGR.to_string(), ARCA.to_string()]
        );
    }

    #[test]
    fn um_firmware_que_nao_se_deixa_entender_e_falha_e_nao_alivio() {
        // O `{fwbootmgr}` faltando **enquanto** o resto sai legivel: e quando
        // `ordem_permanente` fica vazia e a vazia parece a resposta boa.
        let ferramenta = FirmwareDeMentira::novo().respondendo(FIRMWARE, "texto de outro formato");

        let erro = devolver_o_windows(&ferramenta).expect_err("tinha de recusar");

        assert!(
            matches!(erro, Erro::FirmwareIlegivel { .. }),
            "uma leitura que nao se entende virou afirmacao de que esta tudo bem: {erro:?}"
        );
    }

    #[test]
    fn um_bcdedit_que_responde_exito_sem_escrever_nao_passa_por_consertado() {
        // C-3 na letra. O duplo sem modelo aceita a escrita, responde e nao
        // muda nada — que e o modo de falha medido do `bcdedit` desde a E2, e
        // o mesmo pelo qual a rejeicao silenciosa de C-6 chega.
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, &como_o_bcdedit_escreve(&[ARCA, BOOTMGR]));

        let erro = devolver_o_windows(&ferramenta).expect_err("tinha de recusar");

        assert!(
            matches!(erro, Erro::OrdemNaoDevolvida { .. }),
            "um bcdedit que nao escreveu passou por conserto feito: {erro:?}"
        );
        assert!(
            erro.to_string().contains(ARCA),
            "a mensagem tem de mostrar a ordem que sobrou, para quem for consertar a mao: {erro}"
        );
    }

    #[test]
    fn um_bcdedit_que_recusa_a_escrita_tambem_nao_passa() {
        // O outro modo de falha, e o codigo trata os dois igual **de
        // proposito**: a escrita vai com `let _ =`, e quem responde e a
        // releitura. Um `Acesso negado` e um "êxito" mentiroso deixam o mesmo
        // rastro no firmware — nenhum —, e e esse rastro que decide.
        let ferramenta = FirmwareDeMentira::novo()
            .respondendo(FIRMWARE, &como_o_bcdedit_escreve(&[ARCA, BOOTMGR]))
            .recusando_o_executar(Erro::FerramentaRecusou {
                ferramenta: "bcdedit",
                codigo: 1,
                saida: "Acesso negado.".to_string(),
            });

        let erro = devolver_o_windows(&ferramenta).expect_err("tinha de recusar");

        assert!(
            matches!(erro, Erro::OrdemNaoDevolvida { .. }),
            "uma escrita recusada passou por conserto feito: {erro:?}"
        );
    }

    #[test]
    fn sem_ordem_nenhuma_o_conserto_nao_afirma_que_o_windows_ja_estava_la() {
        // Nunca observado nesta maquina, e representavel. Chamar isto de "ja
        // estava na frente" afirmaria que o Windows era o primeiro quando nao
        // havia primeiro — o mesmo tipo de afirmacao conveniente que o
        // ADR-0003 recusou para imagem sem veredito.
        let ferramenta = firmware(&[]);

        let devolvida = devolver_o_windows(&ferramenta).expect("o conserto roda");

        assert_eq!(devolvida, OrdemDevolvida::NaoHaviaOrdem);
        assert!(devolvida.houve_conserto());
        assert_eq!(ferramenta.ordem_permanente(), vec![BOOTMGR.to_string()]);
    }
}
