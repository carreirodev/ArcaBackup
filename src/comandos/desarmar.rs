//! `arca desarmar` — devolve o dispositivo ao estado inerte (C-1).
//!
//! # Por que isto e um comando, se o plano diz que desarmar nao e um
//!
//! O plano chama o desarmar de "o primeiro passo de todo comando", e continua
//! sendo: a E7 desarma antes de armar, e a E8 desarma depois de colher. Nada
//! disso muda. Ele ganha um comando proprio por dois motivos.
//!
//! O primeiro e o criterio de aceite da etapa: "rodar duas vezes seguidas da
//! o mesmo resultado". Sem comando, a unica forma de rodar duas vezes seria
//! por dentro de um comando que arma — e armar e a E7. A etapa que existe
//! para vir **antes** do armar nao pode depender dele para ser exercitada.
//!
//! O segundo e um caso de uso que o PRD ja descreve e nao atende. O §5.5
//! lista "sem `arca-fim.txt`, com job pendente — o boot nao aconteceu" como
//! desfecho possivel. Depois dele o dispositivo continua armado, e ate aqui
//! nao havia nada a rodar: `arca resultado` e a E8 e exige desfecho,
//! `arca backup` armaria de novo. Um comando que so desarma e a resposta a
//! "alguma coisa deu errado e eu quero o dispositivo de volta ao normal".
//!
//! # O caminho aparece na tela, e nao so o `ok`
//!
//! O §5.2 do PRD mostra `Desarmando receita anterior ..... ok`. Aqui sai
//! `Desarmando receita anterior ..... ok · R:\boot\grub\grub.cfg`, com o
//! caminho **na coluna do valor** — no rotulo ele estouraria a coluna e
//! desalinharia esta linha das que vem depois dela no diálogo do backup.
//!
//! O motivo e uma pendencia herdada, documentada em
//! [`crate::dispositivo::Dispositivo::boot`]: nada prova que o `ARCAVAULT` e o
//! `ARCABOOT` encontrados sao do mesmo dispositivo fisico. Fechar isso exige
//! `Discos::discos_fisicos`, que e da E6. Ate la, imprimir a letra e a unica
//! defesa que existe — com dois dispositivos na mesa, a letra errada aparece
//! na tela de quem esta olhando. Custa uma interpolacao.
//!
//! # Para a E5: o `estado.json` sobrevive a este comando, e tem de sobreviver
//!
//! Desarmar nao consulta estado nenhum (C-1) e nao escreve nele. Quando a E5
//! criar o `estado.json`, um `arca desarmar` seguido de um `arca status` vai
//! mostrar "Boot unico: nao armado" ao lado de "Estado no ARCABOOT: presente"
//! — o que lê como contradicao e nao e: o dispositivo esta inerte, e o job
//! continua registrado por colher.
//!
//! Quem resolve isso e a E8, que colhe o desfecho e ai sim encerra o job. O
//! que **nao** pode acontecer e este comando passar a apagar o `estado.json`
//! para a tela ficar bonita: seria consultar-e-decidir onde C-1 proibe, e
//! apagaria a unica coisa que liga um desfecho encontrado depois ao job que o
//! produziu (C-11).

use crate::app::Contexto;
use crate::desarme::{self, Desarme, MarcaDeBootUnico};
use crate::dispositivo::{self, Dispositivo};
use crate::erro::Resultado;
use crate::formato::linha;

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let caminho = dispositivo.caminho_do_grub()?;

    if contexto.dry_run {
        print!(
            "{}",
            montar_ensaio(&dispositivo, &caminho.to_string_lossy())
        );
        return Ok(());
    }

    let desarme = desarme::executar(contexto.arquivos, contexto.firmware, &caminho)?;

    contexto.registro.info(format!(
        "desarmado · {} · blocos removidos: {} · set default devolvido: {} · grub.cfg regravado: {} · boot unico: {}",
        desarme.caminho_do_grub.display(),
        desarme.blocos_removidos,
        desarme.default_devolvido,
        desarme.grub_regravado,
        match &desarme.boot_unico {
            MarcaDeBootUnico::NaoHavia => "nao havia".to_string(),
            MarcaDeBootUnico::Removida { entradas } => format!("removida de {}", entradas.join(", ")),
        }
    ));

    print!("{}", montar(&desarme));
    Ok(())
}

/// O que o desarmar tem a dizer, em texto.
pub fn montar(desarme: &Desarme) -> String {
    let mut saida = String::from("Desarmando o dispositivo\n");

    // O caminho vai na coluna do **valor**, e nao no rotulo. No rotulo ele
    // estoura a coluna 33 do §5.2 e desalinha esta linha das que vem depois
    // dela no diálogo do backup; no valor, o alinhamento fica e o caminho
    // continua a vista.
    saida.push_str(&linha(
        "Desarmando receita anterior",
        &format!("ok · {}", desarme.caminho_do_grub.display()),
    ));

    saida.push_str(&linha(
        "Marca de boot unico",
        &match &desarme.boot_unico {
            MarcaDeBootUnico::NaoHavia => "nao havia".to_string(),
            MarcaDeBootUnico::Removida { entradas } => {
                format!("removida · apontava para {}", entradas.join(", "))
            }
        },
    ));

    saida.push('\n');
    saida.push_str(&desfecho(desarme));

    saida
}

/// A frase final, que diz o que de fato havia — e nao so que havia algo.
///
/// A distincao apareceu na execucao real da etapa, e nao nos testes: com o
/// `grub.cfg` que o Clonezilla entrega, em que o `set default` aponta por
/// posicao e nao ha `menuentry` do ARCA nenhum, a saida dizia "havia receita
/// armada". Nao havia. Havia um `set default` que **armaria sozinho** na
/// proxima insercao, que e um problema diferente e merece ser nomeado, porque
/// quem lesse "receita armada" acharia que a maquina estava a um reinicio de
/// rodar um backup.
fn desfecho(desarme: &Desarme) -> String {
    let receita = match desarme.blocos_removidos {
        0 => None,
        1 => Some("Havia receita armada no grub.cfg, e ela foi tirada.".to_string()),
        varios => Some(format!(
            "Havia {varios} receitas armadas no grub.cfg, e elas foram tiradas."
        )),
    };

    let default = desarme.default_devolvido.then(|| {
        format!(
            "O `set default` do grub.cfg apontava para outra entrada e voltou para\n\
             `{}` — o menu normal do Clonezilla. E ele que faz o boot ser\n\
             desatendido: sem isso, a receita so apareceria no menu.",
            crate::grub::ID_INERTE
        )
    });

    match (receita, default) {
        (None, None) => {
            "Nao havia nada armado. O dispositivo ja estava inerte, e continua.\n".to_string()
        }
        (receita, default) => {
            let mut frases: Vec<String> = [receita, default].into_iter().flatten().collect();
            frases.push("O dispositivo boota no menu normal do Clonezilla.".to_string());
            format!("{}\n", frases.join("\n"))
        }
    }
}

/// O que o `--dry-run` diz. Nada e escrito, nem no `grub.cfg` nem no firmware.
fn montar_ensaio(dispositivo: &Dispositivo, caminho: &str) -> String {
    format!(
        "Ensaio (--dry-run): nada e gravado, nada e desarmado.\n\n\
         Dispositivo ARCA: {} ({})\n\
         O desarmar reescreveria {caminho} no estado inerte e mandaria o\n\
         bcdedit apagar a marca de boot unico do {{fwbootmgr}}.\n\n\
         Nada foi gravado.\n",
        dispositivo::ARCABOOT,
        match dispositivo.boot.as_ref().and_then(|boot| boot.letra) {
            Some(letra) => format!("{letra}:"),
            None => "sem letra".to_string(),
        },
    )
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{ArquivosEmMemoria, DiscosDeMentira, FirmwareDeMentira, RelogioParado};
    use crate::erro::Erro;
    use crate::registro::Registro;
    use std::path::PathBuf;

    const INERTE: &str = include_str!("../../recursos/capturas/grub-inerte-arcaboot.cfg");
    const ARMADA: &str = include_str!("../../recursos/capturas/grub-backup-arca-teste-03.cfg");

    const GRUB: &str = r"R:\boot\grub\grub.cfg";
    const ALVO: &str = "{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";

    fn fwbootmgr(boot_unico: Option<&str>) -> String {
        let sequencia = match boot_unico {
            Some(alvo) => format!("bootsequence            {alvo}\r\n"),
            None => String::new(),
        };
        format!(
            "\r\nGerenciador de Inicialização de Firmware\r\n\
             ----------------------------------------\r\n\
             identificador           {{fwbootmgr}}\r\n\
             displayorder            {{bootmgr}}\r\n\
             {sequencia}timeout                 1\r\n"
        )
    }

    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        registro: Registro,
    }

    impl Bancada {
        fn nova(arquivos: ArquivosEmMemoria, firmware: FirmwareDeMentira) -> Bancada {
            Bancada::com_discos(arquivos, firmware, DiscosDeMentira::com_dispositivo())
        }

        fn com_discos(
            arquivos: ArquivosEmMemoria,
            firmware: FirmwareDeMentira,
            discos: DiscosDeMentira,
        ) -> Bancada {
            Bancada {
                arquivos,
                discos,
                firmware,
                relogio: RelogioParado::em("2026-08-22T11:42:03"),
                registro: Registro::em(
                    std::env::temp_dir().join(format!(
                        "arca-desarmar-{}-{:?}",
                        std::process::id(),
                        std::thread::current().id()
                    )),
                    Box::new(RelogioDoSistema),
                ),
            }
        }

        fn contexto(&self, dry_run: bool) -> Contexto<'_> {
            Contexto {
                dry_run,
                registro: &self.registro,
                firmware: &self.firmware,
                discos: &self.discos,
                arquivos: &self.arquivos,
                relogio: &self.relogio,
            }
        }
    }

    impl Drop for Bancada {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.registro.caminho().parent().unwrap());
        }
    }

    fn desarme_de(caminho: &str, boot_unico: Option<Vec<&str>>) -> Desarme {
        Desarme {
            caminho_do_grub: PathBuf::from(caminho),
            blocos_removidos: 1,
            default_devolvido: true,
            grub_regravado: true,
            boot_unico: match boot_unico {
                Some(entradas) => MarcaDeBootUnico::Removida {
                    entradas: entradas.into_iter().map(str::to_string).collect(),
                },
                None => MarcaDeBootUnico::NaoHavia,
            },
        }
    }

    #[test]
    fn a_linha_do_paragrafo_5_2_leva_o_caminho_em_que_desarmou() {
        // A defesa barata contra desarmar o dispositivo errado, enquanto a E6
        // nao sabe dizer em que disco fisico cada volume esta. Com dois
        // dispositivos na mesa, a letra errada aparece na tela.
        let saida = montar(&desarme_de(GRUB, None));

        assert!(
            saida.contains(&linha(
                "Desarmando receita anterior",
                &format!("ok · {GRUB}")
            )),
            "{saida}"
        );

        // E o alinhamento do §5.2 tem de sobreviver ao caminho: o rotulo e o
        // mesmo do documento, e e o valor que cresce.
        assert!(
            saida.contains("Desarmando receita anterior ....."),
            "o caminho desalinhou a linha do §5.2:\n{saida}"
        );
    }

    #[test]
    fn o_boot_unico_removido_diz_para_onde_apontava() {
        let saida = montar(&desarme_de(GRUB, Some(vec![ALVO])));
        assert!(
            saida.contains(&format!("removida · apontava para {ALVO}")),
            "{saida}"
        );
    }

    #[test]
    fn sem_nada_armado_a_saida_diz_isso_em_vez_de_so_ok() {
        // Nenhum desfecho do ARCA e silencio (§5.5). "Ja estava inerte" e uma
        // informacao diferente de "desarmei", e quem roda o comando depois de
        // um boot que nao aconteceu precisa das duas separadas.
        let inerte = Desarme {
            blocos_removidos: 0,
            default_devolvido: false,
            grub_regravado: false,
            ..desarme_de(GRUB, None)
        };

        let saida = montar(&inerte);
        assert!(saida.contains("ja estava inerte"), "{saida}");
        assert!(!saida.contains("Havia receita armada"), "{saida}");
    }

    #[test]
    fn o_set_default_sozinho_nao_e_relatado_como_receita_armada() {
        // Achado da execucao real desta etapa, e nao dos testes. Com o
        // `grub.cfg` que o Clonezilla entrega — `set default="0"`, sem
        // `menuentry` do ARCA nenhum — a saida dizia "havia receita armada".
        // Nao havia: havia um `set default` que armaria sozinho na proxima
        // insercao. Sao coisas diferentes, e quem le "receita armada" acha que
        // a maquina esta a um reinicio de rodar um backup.
        let so_o_default = Desarme {
            blocos_removidos: 0,
            default_devolvido: true,
            ..desarme_de(GRUB, None)
        };

        let saida = montar(&so_o_default);
        assert!(
            !saida.contains("Havia receita armada"),
            "nao havia receita nenhuma:\n{saida}"
        );
        assert!(saida.contains("set default"), "{saida}");
        assert!(saida.contains("live-default"), "{saida}");
        assert!(saida.contains("menu normal do Clonezilla"), "{saida}");
    }

    #[test]
    fn a_receita_tirada_e_relatada_como_receita() {
        let saida = montar(&desarme_de(GRUB, None));
        assert!(
            saida.contains("Havia receita armada no grub.cfg"),
            "{saida}"
        );
        assert!(
            saida.contains("set default"),
            "as duas coisas foram desfeitas:\n{saida}"
        );
    }

    #[test]
    fn o_comando_desarma_de_verdade_e_e_idempotente() {
        // O criterio de aceite da etapa, pelo comando inteiro: duas vezes
        // seguidas, mesmo resultado.
        let bancada = Bancada::nova(
            ArquivosEmMemoria::novo().com(GRUB, ARMADA),
            FirmwareDeMentira::novo()
                .respondendo("{fwbootmgr}", &fwbootmgr(Some(ALVO)))
                .respondendo_depois("{fwbootmgr}", &fwbootmgr(None)),
        );

        executar(&bancada.contexto(false)).expect("primeira passada");
        let depois_da_primeira = bancada.arquivos.conteudo_de(GRUB);
        assert_eq!(depois_da_primeira.as_deref(), Some(INERTE));

        executar(&bancada.contexto(false)).expect("segunda passada");
        assert_eq!(bancada.arquivos.conteudo_de(GRUB), depois_da_primeira);
    }

    #[test]
    fn o_ensaio_nao_escreve_nem_fala_com_o_firmware() {
        // `--dry-run` e flag de primeira classe em todo comando que escreve, e
        // este escreve nas duas fronteiras.
        let bancada = Bancada::nova(
            ArquivosEmMemoria::novo().com(GRUB, ARMADA),
            FirmwareDeMentira::novo().respondendo("{fwbootmgr}", &fwbootmgr(Some(ALVO))),
        );

        executar(&bancada.contexto(true)).expect("o ensaio roda");

        assert_eq!(
            bancada.arquivos.conteudo_de(GRUB).as_deref(),
            Some(ARMADA),
            "o ensaio reescreveu o grub.cfg"
        );
        assert!(
            bancada.firmware.executados.borrow().is_empty(),
            "o ensaio mandou o bcdedit fazer alguma coisa"
        );
    }

    #[test]
    fn sem_arcaboot_o_comando_recusa_em_vez_de_dizer_que_desarmou() {
        // O `grub.cfg` mora no `ARCABOOT`. Sem ele nao ha o que desarmar, e
        // dizer "ok" seria mentir sobre um dispositivo que pode estar armado.
        let bancada = Bancada::com_discos(
            ArquivosEmMemoria::novo(),
            FirmwareDeMentira::novo().respondendo("{fwbootmgr}", &fwbootmgr(None)),
            DiscosDeMentira::com_volumes(vec![crate::duplos::volume(
                dispositivo::ARCAVAULT,
                'E',
                1000,
                500,
            )]),
        );

        assert!(matches!(
            executar(&bancada.contexto(false)).unwrap_err(),
            Erro::ParticaoAusente { .. }
        ));
    }

    #[test]
    fn sem_dispositivo_o_comando_recusa() {
        let bancada = Bancada::com_discos(
            ArquivosEmMemoria::novo(),
            FirmwareDeMentira::novo().respondendo("{fwbootmgr}", &fwbootmgr(None)),
            DiscosDeMentira::default(),
        );

        assert!(matches!(
            executar(&bancada.contexto(false)).unwrap_err(),
            Erro::DispositivoAusente
        ));
    }
}
