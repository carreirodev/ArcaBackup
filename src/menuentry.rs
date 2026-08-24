//! De onde sai o `menuentry` que o ARCA insere no `grub.cfg`.
//!
//! A E4 deixou esta decisao escrita e sem tomar: [`crate::grub::armar`]
//! **recebe** o bloco pronto e nao o monta, porque as copias armadas do
//! dispositivo divergem entre si e escolher entre elas e decidir a linha de
//! comando que o kernel recebe. Este modulo e a escolha.
//!
//! # O bloco se deriva, e nao se transcreve
//!
//! E a mesma decisao do
//! [ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)
//! aplicada ao armar. O `grub.cfg` carrega a configuracao **daquele**
//! dispositivo e **daquela** versao do Clonezilla: `hostname=cl-3.3.3-15`, as
//! blacklists de driver, `scsi_mod.use_blk_mq=0`, `nvme.poll_queues=1`.
//! Escrever por cima um bloco fixo descartaria tudo isso em silencio, e o
//! modo de falha e o Clonezilla nao subir na maquina de quem trocar de
//! dispositivo.
//!
//! # O modelo e o `live-toram`, e nao o `live-default`
//!
//! Medido nesta etapa, e nao suposto. A captura
//! `grub-backup-arca-teste-02.cfg` e o `menuentry --id live-toram` do proprio
//! `grub.cfg` inerte com **exatamente cinco** substituicoes — as cinco
//! parametros que §10.2.1 do PRD lista. Nada mais muda, nem uma virgula:
//!
//! ```text
//! locales=                        -> locales=en_US.UTF-8
//! keyboard-layouts=               -> keyboard-layouts=NONE
//! (nada)                          -> ocs_repository="dev:///LABEL=ARCAVAULT"
//! ocs_live_run="ocs-live-general" -> ocs_live_run="bash -c '<a receita>'"
//! ocs_live_batch="no"             -> ocs_live_batch="yes"
//! ```
//!
//! Isso responde uma pergunta que o §10.2.1 respondia errado. Ele lista o
//! `toram` entre "o resto da linha, que e do `menuentry` base do Clonezilla"
//! — e o `live-default` **nao tem** `toram`. Quem tem e o `live-toram`, onde
//! o `toram=live,syslinux,EFI,boot,.disk,utils` esta exatamente na posicao em
//! que as capturas armadas o mostram, logo depois do `vga=788`. O `toram`
//! nunca foi acrescentado por ninguem: ele veio junto do modelo.
//!
//! E o modelo certo, e nao so por casar: e o `toram` que evita acoplar o live
//! system ao dispositivo que ele vai remontar como `/home/partimag`, que e a
//! decisao registrada no §10.3 e no
//! [ADR-0002](../docs/adr/0002-receita-como-string-no-grub.md).
//!
//! # O oraculo, e o unico byte que nao bate
//!
//! `derivar` aplicada ao `live-toram` do `grub.cfg` inerte, com os parametros
//! da `teste-02`, tem de produzir o bloco da `teste-02` **byte a byte**. Ha
//! teste, e ele nao pode ser ajustado para passar, porque o alvo e o arquivo
//! que rodou em hardware.
//!
//! Ele bate em tudo menos num byte: a `teste-02` tem **dois** espacos entre
//! `locales=en_US.UTF-8` e `keyboard-layouts=NONE`. E a impressao digital de
//! uma edicao a mao — quem trocou `locales=` por `locales=en_US.UTF-8 `
//! deixou o espaco que ja separava os dois. O ARCA escreve um espaco so, e o
//! teste nomeia essa diferenca em vez de copia-la: reproduzir um artefato de
//! edicao seria confundir o que rodou com o que se quis. Qualquer **outra**
//! divergencia reprova.
//!
//! # A `teste-03` nao nasceu assim, e isso tambem e evidencia
//!
//! Das quatro copias armadas, a `teste-03` e a unica com
//! `set default="arca-backup"` — a unica que, pelo ADR-0005, teria rodado
//! desatendida. E ela perdeu **nove** coisas que o modelo tem:
//! `hostname=cl-3.3.3-15`, `ocs_live_extra_param=""`, as tres
//! `*.blacklist=yes`, `vmwgfx.enable_fbdev=1`, `ocs_1_cpu_udev`,
//! `scsi_mod.use_blk_mq=0` e `nvme.poll_queues=1`.
//!
//! A ultima e a que dói: a unica copia que provavelmente rodou desatendida e
//! a que perdeu o parametro de NVMe, numa maquina cujo disco de origem e
//! NVMe. Isso nao e argumento para transcrever a `teste-03` — e o contrario.
//! Ela mostra o que acontece quando o bloco e montado a mao a partir de
//! memoria em vez de derivado do arquivo, e ha teste fixando o achado.

use crate::grub::{self, ID_DO_ARCA};
use crate::receita::Parametro;
use std::fmt;

/// O `--id` do `menuentry` de que o bloco do ARCA e derivado.
///
/// Nao e o `live-default`, que e para onde o `set default` volta no estado
/// inerte. Sao papeis diferentes: o `live-default` e o **alvo** do desarmar,
/// o `live-toram` e o **modelo** do armar. Ver o cabecalho deste modulo.
pub const ID_DO_MODELO: &str = "live-toram";

/// O titulo do `menuentry` do ARCA, como as quatro copias armadas o escrevem.
///
/// Transcrito, e constante de proposito: a captura de **restauracao** traz
/// este mesmo titulo. Ele nunca nomeou a operacao, e inventar um
/// "ARCA - restauracao automatica" seria acrescentar uma diferenca que nunca
/// rodou. Quem decide o que executa e o `--id`, para onde o `set default`
/// aponta; o titulo so apareceria num menu que o boot desatendido nao chega a
/// mostrar.
///
/// **A E9 chegou e nao mexeu nisto**, que era a tentacao que o ADR-0007 tinha
/// nomeado com um ano de antecedencia — ou melhor, com duas etapas. O
/// `arca restore` arma o mesmo bloco, com o mesmo titulo, e o que muda entre
/// as duas operacoes esta inteiro dentro do `ocs_live_run`.
pub const TITULO: &str = "ARCA - backup automatico";

/// A diretiva do `grub.cfg` que carrega a linha de comando do kernel.
const LINUX_CMD: &str = "$linux_cmd";

/// O parametro antes do qual entra o que o modelo nao tem.
///
/// E onde a `teste-02` pos o `ocs_repository`, e o unico ponto de insercao
/// medido. Ancorar num parametro que a receita sempre escreve — em vez de no
/// fim da linha — mantem a derivacao presa ao que se observou.
const ANCORA: &str = "ocs_live_run";

/// Por que nao ha de onde derivar o bloco.
///
/// Toda variante e uma recusa **antes** de gravar, como as de
/// [`crate::grub::RecusaDoGrub`]: um `grub.cfg` que o ARCA nao entende e um
/// `grub.cfg` que ele nao mexe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoMenuentry {
    /// Nao ha `menuentry --id live-toram`, ou ha um sem a chave que o fecha.
    ///
    /// Um `grub.cfg` de outra versao do Clonezilla pode nomear as entradas de
    /// outro jeito. Montar um bloco do zero seria escrever a linha de comando
    /// do kernel por conta propria, para um hardware que este binario nao
    /// conhece — e o modo de falha e a maquina nao bootar.
    SemModelo,

    /// O bloco existe e nao tem linha `$linux_cmd`. Nao ha onde pôr a receita.
    ModeloSemLinhaDeComando,

    /// Um parametro precisa entrar e o modelo nao tem a ancora.
    SemAncora { parametro: &'static str },
}

impl fmt::Display for RecusaDoMenuentry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoMenuentry::SemModelo => write!(
                f,
                "o grub.cfg do dispositivo nao tem `menuentry --id {ID_DO_MODELO}` inteiro, e e dele que o bloco do ARCA e derivado. O ARCA nao monta a linha de comando do kernel por conta propria: ela carrega a configuracao deste dispositivo e desta versao do Clonezilla, e escrever uma de memoria e como uma maquina deixa de bootar"
            ),
            RecusaDoMenuentry::ModeloSemLinhaDeComando => write!(
                f,
                "o `menuentry --id {ID_DO_MODELO}` do grub.cfg nao tem linha `{LINUX_CMD}`, e e nela que a receita entra"
            ),
            RecusaDoMenuentry::SemAncora { parametro } => write!(
                f,
                "o parametro `{parametro}` precisa entrar na linha do kernel e o modelo nao tem `{ANCORA}=`, que e o unico ponto de insercao medido nas capturas"
            ),
        }
    }
}

/// Deriva o bloco do ARCA do `grub.cfg` que esta no dispositivo.
///
/// `corrente` e o `grub.cfg` **inerte** — o mesmo texto que
/// [`crate::grub::armar`] vai receber. Deriva-se do arquivo que se vai
/// escrever, e nao de uma copia guardada, pelo motivo do ADR-0005: a
/// configuracao de hardware mora ali.
pub fn derivar(corrente: &str, parametros: &[Parametro]) -> Result<String, RecusaDoMenuentry> {
    let modelo = grub::bloco_com_id(corrente, ID_DO_MODELO).ok_or(RecusaDoMenuentry::SemModelo)?;

    let mut linhas: Vec<String> = modelo
        .split_inclusive('\n')
        .map(|linha| linha.to_string())
        .collect();

    // A primeira linha e reescrita inteira, e nao remendada: o modelo traz
    // `--hotkey=r` e um titulo entre aspas, e mexer nisso por substituicao
    // exigiria um parser de aspas para um caso em que o resultado ja e
    // conhecido. A `teste-02` escreve exatamente esta linha.
    let recuo = recuo_de(&linhas[0]);
    let terminador = terminador_da(&linhas[0]);
    linhas[0] = format!("{recuo}menuentry \"{TITULO}\" --id {ID_DO_ARCA} {{{terminador}");

    let alvo = linhas
        .iter()
        .position(|linha| linha.trim_start().starts_with(LINUX_CMD))
        .ok_or(RecusaDoMenuentry::ModeloSemLinhaDeComando)?;

    let recuo = recuo_de(&linhas[alvo]);
    let terminador = terminador_da(&linhas[alvo]);
    let comando = substituir(linhas[alvo].trim(), parametros)?;
    linhas[alvo] = format!("{recuo}{comando}{terminador}");

    Ok(linhas.concat())
}

/// Aplica os parametros a linha `$linux_cmd`, por token.
///
/// Por token, e nunca por `replace` de texto: um `locales=` cru aparece
/// tambem dentro de `keyboard-layouts=`? Nao — mas `nomodeset` contem `mode`,
/// `vga=788` contem `ga=`, e a linha tem vinte e poucos parametros escritos
/// por outra pessoa. Casar o nome inteiro seguido de `=` e a diferenca entre
/// substituir um parametro e corromper outro.
fn substituir(linha: &str, parametros: &[Parametro]) -> Result<String, RecusaDoMenuentry> {
    let mut tokens: Vec<String> = linha.split(' ').map(|token| token.to_string()).collect();

    for parametro in parametros {
        let prefixo = format!("{}=", parametro.nome);
        let escrito = parametro.to_string();

        if let Some(posicao) = tokens.iter().position(|token| token.starts_with(&prefixo)) {
            tokens[posicao] = escrito;
            continue;
        }

        // O que o modelo nao tem entra **antes da ancora**, que e onde a
        // `teste-02` pos o `ocs_repository`. Sem ancora nao se adivinha
        // posicao: a ordem dos parametros do kernel nao muda o que eles
        // significam, mas o oraculo desta etapa e um arquivo, e um arquivo
        // tem ordem.
        let ancora = tokens
            .iter()
            .position(|token| token.starts_with(&format!("{ANCORA}=")))
            .ok_or(RecusaDoMenuentry::SemAncora {
                parametro: parametro.nome,
            })?;
        tokens.insert(ancora, escrito);
    }

    Ok(tokens.join(" "))
}

/// Os espacos com que a linha comeca, para que o bloco derivado saia com o
/// recuo do arquivo de onde veio.
fn recuo_de(linha: &str) -> &str {
    let sem_recuo = linha.trim_start();
    &linha[..linha.len() - sem_recuo.len()]
}

/// O terminador desta linha: `\r\n`, `\n`, ou nada.
fn terminador_da(linha: &str) -> &'static str {
    if linha.ends_with("\r\n") {
        "\r\n"
    } else if linha.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nome::Nome;
    use crate::receita::{Disco, Operacao, Pedido, Receita, Selo};

    const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");
    const TESTE_02: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-02.cfg");
    const TESTE_03: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg");

    /// A receita que a `teste-02` executou, tirada do proprio arquivo.
    ///
    /// Extraida da captura, e nao repetida a mao: uma string repetida a mao
    /// prova que eu sei copiar; o arquivo prova o que o hardware executou.
    fn receita_da_teste_02() -> String {
        let bloco = grub::bloco_do_arca(TESTE_02).expect("a teste-02 tem bloco do ARCA");
        let linha = bloco
            .lines()
            .find(|linha| linha.trim_start().starts_with(LINUX_CMD))
            .expect("o bloco tem $linux_cmd");

        let inicio = linha.find("ocs_live_run=\"bash -c '").expect("tem receita");
        let resto = &linha[inicio + "ocs_live_run=\"".len()..];
        let fim = resto.find("'\"").expect("a receita fecha");
        resto[..fim + 1].to_string()
    }

    /// Os cinco parametros da `teste-02`, do jeito que o `Parametro` os
    /// escreve.
    fn parametros_da_teste_02() -> Vec<Parametro> {
        vec![
            Parametro {
                nome: "locales",
                valor: "en_US.UTF-8".to_string(),
                entre_aspas: false,
            },
            Parametro {
                nome: "keyboard-layouts",
                valor: "NONE".to_string(),
                entre_aspas: false,
            },
            Parametro {
                nome: "ocs_repository",
                valor: "dev:///LABEL=ARCAVAULT".to_string(),
                entre_aspas: true,
            },
            Parametro {
                nome: "ocs_live_run",
                valor: receita_da_teste_02(),
                entre_aspas: true,
            },
            Parametro {
                nome: "ocs_live_batch",
                valor: "yes".to_string(),
                entre_aspas: true,
            },
        ]
    }

    /// O unico byte em que a derivacao e a `teste-02` divergem, nomeado.
    ///
    /// A captura tem **dois** espacos onde a derivacao poe um. E rastro de
    /// edicao a mao: `locales=` virou `locales=en_US.UTF-8 ` e o espaco que
    /// ja separava os dois parametros ficou. Normalizar aqui, num ponto so e
    /// com nome, e diferente de afrouxar a comparacao — qualquer outra
    /// divergencia continua reprovando.
    fn sem_o_espaco_duplo(texto: &str) -> String {
        texto.replace("en_US.UTF-8  keyboard-layouts", "en_US.UTF-8 keyboard-layouts")
    }

    #[test]
    fn derivar_do_inerte_produz_o_bloco_que_rodou_em_hardware() {
        // O oraculo desta etapa. O alvo nao e um bloco que este teste montou:
        // e o `menuentry` que estava no `grub.cfg` do dispositivo quando a
        // maquina bootou nele e gravou `ARCA-TESTE-02` sozinha.
        let derivado = derivar(INERTE, &parametros_da_teste_02()).expect("deriva");
        let que_rodou = grub::bloco_do_arca(TESTE_02).expect("a teste-02 tem bloco");

        assert_eq!(derivado, sem_o_espaco_duplo(&que_rodou));
    }

    #[test]
    fn a_unica_divergencia_com_a_captura_e_o_espaco_duplo() {
        // O teste acima normaliza um ponto, e este cobra que seja **so** um.
        // Sem ele, alguem poderia alargar `sem_o_espaco_duplo` ate a
        // comparacao passar a nao provar nada.
        let que_rodou = grub::bloco_do_arca(TESTE_02).expect("a teste-02 tem bloco");
        let derivado = derivar(INERTE, &parametros_da_teste_02()).expect("deriva");

        assert_ne!(derivado, que_rodou, "a captura tem o espaco duplo");
        assert_eq!(
            que_rodou.matches("  ").count() - derivado.matches("  ").count(),
            1,
            "ha mais de um lugar em que os espacos divergem"
        );
    }

    #[test]
    fn o_modelo_e_o_live_toram_e_o_live_default_nao_serve() {
        // O achado da etapa, fixado: derivar do `live-default` produziria um
        // bloco **sem** `toram`, e as quatro copias armadas o tem. Se algum
        // dia alguem trocar a constante, este teste diz por que nao.
        let derivado = derivar(INERTE, &parametros_da_teste_02()).expect("deriva");
        assert!(
            derivado.contains("toram=live,syslinux,EFI,boot,.disk,utils"),
            "o bloco derivado perdeu o toram"
        );

        let default = grub::bloco_com_id(INERTE, crate::grub::ID_INERTE).expect("existe");
        assert!(
            !default.contains("toram="),
            "o live-default nao tem toram, e e por isso que ele nao e o modelo"
        );
    }

    #[test]
    fn a_configuracao_de_hardware_do_dispositivo_atravessa_a_derivacao() {
        // A razao de derivar em vez de transcrever. Estes valores sao **deste**
        // dispositivo e **desta** versao do Clonezilla, e um bloco fixo os
        // descartaria em silencio.
        let derivado = derivar(INERTE, &parametros_da_teste_02()).expect("deriva");

        for herdado in [
            "hostname=cl-3.3.3-15",
            "i915.blacklist=yes",
            "radeonhd.blacklist=yes",
            "nouveau.blacklist=yes",
            "vmwgfx.enable_fbdev=1",
            "ocs_1_cpu_udev",
            "scsi_mod.use_blk_mq=0",
            "nvme.poll_queues=1",
        ] {
            assert!(
                derivado.contains(herdado),
                "a derivacao perdeu `{herdado}`, que veio do grub.cfg do dispositivo"
            );
        }
    }

    #[test]
    fn a_teste_03_perdeu_nove_coisas_que_a_derivacao_preserva() {
        // A copia que provavelmente rodou desatendida — a unica com
        // `set default="arca-backup"` — e a que perdeu mais. A ultima da lista
        // e a que dói: `nvme.poll_queues=1` sumiu numa maquina cujo disco de
        // origem e NVMe.
        //
        // Isto nao e argumento para transcrever a `teste-03`: e o que mostra o
        // que acontece quando o bloco e escrito de memoria em vez de derivado.
        let da_teste_03 = grub::bloco_do_arca(TESTE_03).expect("a teste-03 tem bloco");
        let derivado = derivar(INERTE, &parametros_da_teste_02()).expect("deriva");

        let perdidos = [
            "hostname=cl-3.3.3-15",
            "ocs_live_extra_param=\"\"",
            "i915.blacklist=yes",
            "radeonhd.blacklist=yes",
            "nouveau.blacklist=yes",
            "vmwgfx.enable_fbdev=1",
            "ocs_1_cpu_udev",
            "scsi_mod.use_blk_mq=0",
            "nvme.poll_queues=1",
        ];
        assert_eq!(perdidos.len(), 9);

        for perdido in perdidos {
            assert!(
                !da_teste_03.contains(perdido),
                "a teste-03 tem `{perdido}`, e o achado dizia que nao"
            );
            assert!(
                derivado.contains(perdido),
                "a derivacao tambem perdeu `{perdido}`"
            );
        }
    }

    #[test]
    fn armar_com_o_bloco_derivado_e_desarmar_devolvem_o_inerte() {
        // As duas metades fecham: o que este modulo monta, o desarmar da E4
        // desfaz. A ida e a volta contra o arquivo do dispositivo, e nao
        // contra um alvo inventado.
        let bloco = derivar(INERTE, &parametros_da_teste_02()).expect("deriva");
        let armado = grub::armar(INERTE, &bloco).expect("arma");

        assert!(armado.contains(&format!("set default=\"{ID_DO_ARCA}\"")));
        assert_eq!(grub::bloco_do_arca(&armado).as_deref(), Some(bloco.as_str()));

        let desarmado = grub::desarmar(&armado).expect("desarma");
        assert_eq!(desarmado.texto, INERTE);
    }

    #[test]
    fn a_receita_de_hoje_cabe_no_bloco_derivado() {
        // A receita da E3 e maior do que a que rodou: ela tras o `arca-fim.txt`,
        // o selo, o `ARCA_FIM` e o `if/then/else`. O que se cobra aqui e que a
        // derivacao a aceite inteira e a ponha onde a `teste-02` a punha.
        let receita = Receita::montar(&Pedido {
            operacao: Operacao::Backup,
            nome: Some(Nome::novo("2026-08-22_Apps").unwrap()),
            disco: Some(Disco::novo("nvme0n1").unwrap()),
            selo: Selo::novo("a3f1c9e07b2d4856").unwrap(),
        })
        .expect("monta");

        let bloco = derivar(INERTE, receita.parametros()).expect("deriva");

        assert!(bloco.contains(&format!("ocs_live_run=\"bash -c '{}'\"", receita.comando())));
        assert!(bloco.contains("ARCA_SELO=a3f1c9e07b2d4856"));
        // A ancora: o `ocs_repository` entra antes do `ocs_live_run`, como na
        // captura.
        let linha = bloco
            .lines()
            .find(|linha| linha.trim_start().starts_with(LINUX_CMD))
            .unwrap();
        assert!(linha.find("ocs_repository=").unwrap() < linha.find("ocs_live_run=").unwrap());
    }

    #[test]
    fn um_grub_cfg_sem_o_modelo_e_recusado_em_vez_de_inventado() {
        let sem_modelo = INERTE.replace(&format!("--id {ID_DO_MODELO}"), "--id outra-coisa");
        assert_eq!(
            derivar(&sem_modelo, &parametros_da_teste_02()),
            Err(RecusaDoMenuentry::SemModelo)
        );
    }

    #[test]
    fn um_modelo_sem_linha_de_comando_e_recusado() {
        let mutilado = INERTE.replace(
            "menuentry --hotkey=r \"Clonezilla live (VGA 800x600 & To RAM)\" --id live-toram {\n  search --set -f /live/vmlinuz\n  $linux_cmd",
            "menuentry --hotkey=r \"Clonezilla live (VGA 800x600 & To RAM)\" --id live-toram {\n  search --set -f /live/vmlinuz\n  nada_de_kernel",
        );
        assert_eq!(
            derivar(&mutilado, &parametros_da_teste_02()),
            Err(RecusaDoMenuentry::ModeloSemLinhaDeComando)
        );
    }

    #[test]
    fn a_substituicao_e_por_token_e_nao_por_texto_solto() {
        // O perigo real: `vga=788` contem `ga=`, `nomodeset` contem `mode`, e a
        // linha tem vinte e poucos parametros que outra pessoa escreveu. Um
        // parametro chamado `ga` nao pode encostar no `vga`.
        let linha = "$linux_cmd /live/vmlinuz vga=788 nomodeset ocs_live_run=\"x\" quiet";
        let saida = substituir(
            linha,
            &[Parametro {
                nome: "ga",
                valor: "9".to_string(),
                entre_aspas: false,
            }],
        )
        .expect("substitui");

        assert!(saida.contains("vga=788"), "o vga foi corrompido: {saida}");
        assert!(saida.contains("ga=9 ocs_live_run"), "o novo nao entrou na ancora: {saida}");
    }

    #[test]
    fn sem_ancora_o_parametro_novo_nao_e_chutado_para_o_fim() {
        let linha = "$linux_cmd /live/vmlinuz vga=788 quiet";
        assert_eq!(
            substituir(
                linha,
                &[Parametro {
                    nome: "ocs_repository",
                    valor: "x".to_string(),
                    entre_aspas: true,
                }],
            ),
            Err(RecusaDoMenuentry::SemAncora {
                parametro: "ocs_repository"
            })
        );
    }

    #[test]
    fn o_bloco_derivado_sai_com_o_terminador_do_arquivo() {
        // Um `grub.cfg` em CRLF tem de continuar em CRLF depois de armado. O
        // deste dispositivo e LF, e por isso o caso CRLF so aparece aqui.
        let em_crlf = INERTE.replace('\n', "\r\n");
        let bloco = derivar(&em_crlf, &parametros_da_teste_02()).expect("deriva");

        assert!(bloco.ends_with("}\r\n"));
        assert!(!bloco.contains("\n\r"), "quebra de linha misturada: {bloco:?}");
    }
}
