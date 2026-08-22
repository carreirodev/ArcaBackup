//! `arca status` — diagnostico nao destrutivo (§8 do PRD).
//!
//! Diz quatro coisas e nao faz nenhuma: qual dispositivo esta conectado, que
//! imagens ele tem, qual e a entrada de firmware do ARCA e se ha job pendente.
//! Lê o dispositivo, lê o `bcdedit`, e nao escreve em lugar nenhum — nem no
//! `grub.cfg`, nem no firmware, nem no dispositivo.
//!
//! E o comando que se roda **antes** de armar, e o que se roda quando alguma
//! coisa nao esta como se esperava. Por isso ele nomeia o que esta errado em
//! vez de so descrever o que encontrou: uma entrada de firmware apontando para
//! o volume errado e a diferenca entre a maquina bootar no Clonezilla e bootar
//! no Windows com um job armado esperando.

use crate::app::Contexto;
use crate::dispositivo::{self, Dispositivo};
use crate::erro::Resultado;
use crate::firmware::{self, Alvo, Leitura, Procedencia};
use crate::formato::{linha, tamanho};
use crate::imagens::{self, Pasta};
use crate::portas::{TipoDeMidia, Volume};

use super::list;

/// O alvo que se pergunta ao `bcdedit`: as entradas de boot do firmware, que
/// sao as que apontam para o dispositivo.
const ALVO: &str = "firmware";

/// Se ha job por colher, pelo que o `ARCABOOT` mostra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoDoJob {
    Nenhum,

    /// Ha um `estado.json` gravado. O que ele diz — selo, comando, alvo — e a
    /// etapa E5 que lê; aqui basta que ele exista.
    Presente,

    /// Sem `ARCABOOT` nao ha onde olhar, e isso nao e o mesmo que nao haver
    /// job: e o ARCA nao ter conseguido perguntar.
    SemOndeOlhar,
}

/// Tudo que o `status` colheu, antes de virar texto.
pub struct Diagnostico<'a> {
    pub dispositivo: &'a Dispositivo,
    pub pastas: &'a [Pasta],
    pub firmware: &'a Leitura,
    pub estado_do_job: EstadoDoJob,
}

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let pastas = imagens::enumerar(contexto.arquivos, &dispositivo.raiz_do_vault()?)?;

    let firmware = firmware::ler(&contexto.firmware.enumerar(ALVO)?);

    let estado_do_job = match dispositivo.caminho_do_estado() {
        Ok(caminho) if contexto.arquivos.existe(&caminho) => EstadoDoJob::Presente,
        Ok(_) => EstadoDoJob::Nenhum,
        // So ha um motivo para nao haver caminho: nao ha `ARCABOOT`.
        Err(_) => EstadoDoJob::SemOndeOlhar,
    };

    contexto.registro.info(format!(
        "status · {} entrada(s) no firmware · entrada do ARCA: {} · boot unico: {} · estado: {estado_do_job:?}",
        firmware.entradas.len(),
        match firmware.entrada_do_arca() {
            Some(achado) => format!("{} ({:?})", achado.descricao, achado.procedencia),
            None => "nenhuma".to_string(),
        },
        if firmware.tem_boot_unico() { "armado" } else { "nao armado" },
    ));

    print!(
        "{}",
        montar(&Diagnostico {
            dispositivo: &dispositivo,
            pastas: &pastas,
            firmware: &firmware,
            estado_do_job,
        })
    );
    Ok(())
}

/// O diagnostico inteiro, em texto.
pub fn montar(diagnostico: &Diagnostico) -> String {
    let mut saida = String::new();

    saida.push_str(&secao_do_dispositivo(diagnostico.dispositivo));
    saida.push('\n');

    // Sem segunda formatacao das imagens: a saida do §5.4 e criterio de aceite
    // da E1, e duas versoes dela divergiriam na primeira mudanca.
    saida.push_str(&list::montar(
        diagnostico.pastas,
        diagnostico.dispositivo.vault.livre_bytes,
    ));
    saida.push('\n');

    saida.push_str(&secao_do_firmware(
        diagnostico.firmware,
        diagnostico.dispositivo,
    ));
    saida.push('\n');

    saida.push_str(&secao_do_job(
        diagnostico.firmware,
        diagnostico.estado_do_job,
    ));
    saida
}

/// As duas particoes, e o aviso de C-6 quando couber.
fn secao_do_dispositivo(dispositivo: &Dispositivo) -> String {
    let mut saida = String::from("Dispositivo ARCA\n");

    saida.push_str(&linha(
        dispositivo::ARCAVAULT,
        &descrever(&dispositivo.vault),
    ));
    saida.push_str(&linha(
        dispositivo::ARCABOOT,
        &match &dispositivo.boot {
            Some(boot) => descrever(boot),
            // Sem `ARCABOOT` da para listar imagens, e nao da para armar: a
            // receita e o estado do job moram nele (§4.1).
            None => "ausente — sem ele nao ha onde gravar receita nem estado".to_string(),
        },
    ));

    if dispositivo
        .boot
        .as_ref()
        .is_some_and(|boot| boot.tipo_de_midia == TipoDeMidia::Removivel)
    {
        saida.push_str(concat!(
            "\n  AVISO (C-6): o Windows classifica o ARCABOOT como midia removivel.\n",
            "  O bcdedit recusa esse alvo em silencio — responde \"exito\" e mantem o\n",
            "  valor antigo. Um dispositivo assim boota por F12, nunca por entrada de\n",
            "  firmware.\n"
        ));
    }

    saida
}

fn descrever(volume: &Volume) -> String {
    let letra = match volume.letra {
        Some(letra) => format!("{letra}:"),
        // Sem letra o volume existe e nao tem caminho. `arca list` recusa antes
        // de chegar aqui; o status tem de dizer por que.
        None => "sem letra".to_string(),
    };

    format!(
        "{letra} · {} · {}",
        volume.sistema_de_arquivos,
        tamanho(volume.total_bytes)
    )
}

/// A entrada do ARCA no firmware, e para onde ela aponta de verdade.
fn secao_do_firmware(leitura: &Leitura, dispositivo: &Dispositivo) -> String {
    let mut saida = String::from("Entrada de firmware\n");

    let Some(achado) = leitura.entrada_do_arca() else {
        saida.push_str(&linha(&format!("Entrada {}", firmware::ARCA), "nenhuma"));
        saida.push_str(&linha(&format!("Entrada {}", firmware::LEGADA), "nenhuma"));
        saida.push_str(&format!(
            "  Nao ha por onde bootar no dispositivo sem passar pelo F12. A etapa E7\n\
             \x20 cria a entrada; ate la, ha {} entrada(s) de boot no firmware.\n",
            leitura.entradas.len()
        ));
        return saida;
    };

    saida.push_str(&linha(
        "Descricao",
        &match achado.procedencia {
            Procedencia::Propria => achado.descricao.to_string(),
            // C-4: a legada nao e um problema, e a entrada certa com o nome
            // antigo. Quem a renomeia e a E7, ao armar.
            Procedencia::Legada => format!("{} · legada, a migrar (C-4)", achado.descricao),
        },
    ));
    saida.push_str(&linha("Identificador", &achado.entrada.identificador));

    saida.push_str(&linha(
        "Aponta para",
        &match &achado.entrada.alvo {
            Some(alvo) => format!(
                "{} · {}",
                alvo.como_bcdedit_escreve(),
                confere_com_o_arcaboot(alvo, dispositivo)
            ),
            None => "nada — a entrada existe e nao diz para onde ir".to_string(),
        },
    ));

    saida.push_str(&linha(
        "Carrega",
        achado
            .entrada
            .caminho
            .as_deref()
            .unwrap_or("nada — a entrada nao diz que .efi carregar"),
    ));

    saida
}

/// Se a entrada de firmware aponta para o `ARCABOOT` que esta na mesa.
///
/// Esta e a pergunta que o `status` existe para responder. A entrada guarda uma
/// letra, e letra muda de uma conexao para outra: uma entrada armada apontando
/// para a letra de ontem manda a maquina bootar em outra coisa — ou em nada.
fn confere_com_o_arcaboot(alvo: &Alvo, dispositivo: &Dispositivo) -> String {
    let Some(boot) = &dispositivo.boot else {
        return "sem ARCABOOT conectado para conferir".to_string();
    };

    match (alvo.letra(), boot.letra) {
        (Some(apontada), Some(atual)) if apontada.eq_ignore_ascii_case(&atual) => {
            "o ARCABOOT deste dispositivo".to_string()
        }
        (Some(_), Some(atual)) => format!("NAO e o ARCABOOT, que esta em {atual}:"),
        // Uma entrada por caminho de dispositivo (`\Device\HarddiskVolume1`)
        // nao da para conferir por letra, e inventar uma correspondencia aqui
        // seria pior do que admitir que nao se sabe.
        _ => "nao da para conferir por letra".to_string(),
    }
}

/// Se ha job por colher, pelos dois sinais que a E2 sabe ler.
fn secao_do_job(leitura: &Leitura, estado: EstadoDoJob) -> String {
    let mut saida = String::from("Job pendente\n");

    saida.push_str(&linha(
        "Boot unico",
        &if leitura.tem_boot_unico() {
            format!("ARMADO para {}", leitura.boot_unico.join(", "))
        } else {
            "nao armado".to_string()
        },
    ));

    saida.push_str(&linha(
        "Estado no ARCABOOT",
        match estado {
            EstadoDoJob::Nenhum => "nenhum",
            EstadoDoJob::Presente => "presente — o selo e o alvo, na etapa E5",
            EstadoDoJob::SemOndeOlhar => "sem ARCABOOT, nao da para olhar",
        },
    ));

    saida
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{momento, volume};
    use crate::imagens::{Especie, Veredito};

    const PT: &str = include_str!("../../recursos/capturas/bcdedit-enum-firmware-pt.txt");
    const LEGADO: &str =
        include_str!("../../recursos/capturas/bcdedit-enum-firmware-legado-pt.txt");

    fn dispositivo_conectado() -> Dispositivo {
        Dispositivo {
            vault: volume(
                dispositivo::ARCAVAULT,
                'E',
                254_000_000_000,
                196_400_000_000,
            ),
            boot: Some(Volume {
                sistema_de_arquivos: "FAT32".to_string(),
                ..volume(dispositivo::ARCABOOT, 'R', 1_700_000_000, 1_070_000_000)
            }),
        }
    }

    fn uma_imagem() -> Vec<Pasta> {
        vec![Pasta {
            nome: "2026-08-21_WindowsCompleto".to_string(),
            tamanho_bytes: 38_823_623_035,
            modificado_em: Some(momento("2026-08-21T12:56:31")),
            especie: Especie::Imagem {
                veredito: Some(Veredito::Aprovada),
            },
        }]
    }

    fn montar_com(dispositivo: &Dispositivo, texto_do_bcdedit: &str) -> String {
        let leitura = firmware::ler(texto_do_bcdedit);
        montar(&Diagnostico {
            dispositivo,
            pastas: &uma_imagem(),
            firmware: &leitura,
            estado_do_job: EstadoDoJob::Nenhum,
        })
    }

    #[test]
    fn o_status_responde_as_quatro_perguntas() {
        // O criterio de aceite da etapa, em texto: dispositivo, imagens,
        // entrada de firmware, job pendente.
        let saida = montar_com(&dispositivo_conectado(), PT);

        assert!(saida.contains("Dispositivo ARCA"), "faltou o dispositivo");
        assert!(
            saida.contains(&linha(dispositivo::ARCABOOT, "R: · FAT32 · 1,6 GB")),
            "faltou o ARCABOOT:\n{saida}"
        );
        assert!(
            saida.contains("2026-08-21_WindowsCompleto"),
            "faltaram as imagens"
        );
        assert!(
            saida.contains("{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"),
            "faltou a entrada de firmware"
        );
        assert!(saida.contains("Job pendente"), "faltou o job");
    }

    #[test]
    fn a_entrada_que_aponta_para_o_arcaboot_e_reconhecida() {
        let saida = montar_com(&dispositivo_conectado(), PT);
        assert!(
            saida.contains(&linha(
                "Aponta para",
                "partition=R: · o ARCABOOT deste dispositivo"
            )),
            "{saida}"
        );
    }

    #[test]
    fn a_entrada_que_aponta_para_outra_letra_e_denunciada() {
        // Letra muda de uma conexao para outra. Uma entrada armada apontando
        // para a letra de ontem manda a maquina bootar em outra coisa, e o
        // status existe para dizer isso antes de alguem armar.
        let dispositivo = Dispositivo {
            boot: Some(volume(
                dispositivo::ARCABOOT,
                'S',
                1_700_000_000,
                1_070_000_000,
            )),
            ..dispositivo_conectado()
        };

        let saida = montar_com(&dispositivo, PT);
        assert!(
            saida.contains("NAO e o ARCABOOT, que esta em S:"),
            "{saida}"
        );
    }

    #[test]
    fn a_entrada_legada_aparece_como_a_migrar() {
        // C-4: nao e um problema a resolver a mao, e a entrada certa com o nome
        // antigo. Quem a renomeia e a E7.
        let saida = montar_com(&dispositivo_conectado(), LEGADO);

        assert!(
            saida.contains(&linha("Descricao", "Clonezilla · legada, a migrar (C-4)")),
            "{saida}"
        );
        assert!(saida.contains("{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"));
    }

    #[test]
    fn sem_entrada_de_firmware_o_status_diz_o_que_falta() {
        let saida = montar_com(&dispositivo_conectado(), "");

        assert!(saida.contains(&linha("Entrada ARCA", "nenhuma")), "{saida}");
        assert!(
            saida.contains(&linha("Entrada Clonezilla", "nenhuma")),
            "{saida}"
        );
        assert!(saida.contains("F12"), "faltou dizer como bootar sem ela");
    }

    #[test]
    fn o_boot_unico_armado_aparece_em_maiuscula() {
        // Job armado e a diferenca entre a maquina reiniciar no Windows e
        // reiniciar no Clonezilla com uma receita esperando. Nao pode passar
        // despercebido no meio de uma listagem.
        let texto = concat!(
            "\r\nGerenciador de Inicialização de Firmware\r\n",
            "----------------------------------------\r\n",
            "identificador           {fwbootmgr}\r\n",
            "displayorder            {bootmgr}\r\n",
            "bootsequence            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}\r\n"
        );

        let saida = montar_com(&dispositivo_conectado(), texto);
        assert!(
            saida.contains(&linha(
                "Boot unico",
                "ARMADO para {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}"
            )),
            "{saida}"
        );
    }

    #[test]
    fn sem_boot_unico_o_firmware_esta_inerte() {
        let saida = montar_com(&dispositivo_conectado(), PT);
        assert!(
            saida.contains(&linha("Boot unico", "nao armado")),
            "{saida}"
        );
    }

    #[test]
    fn o_estado_do_job_tem_uma_linha_para_cada_caso() {
        let dispositivo = dispositivo_conectado();
        let leitura = firmware::ler(PT);

        let com = |estado| {
            montar(&Diagnostico {
                dispositivo: &dispositivo,
                pastas: &uma_imagem(),
                firmware: &leitura,
                estado_do_job: estado,
            })
        };

        assert!(com(EstadoDoJob::Nenhum).contains(&linha("Estado no ARCABOOT", "nenhum")));
        assert!(com(EstadoDoJob::Presente).contains("presente — o selo e o alvo, na etapa E5"));
        assert!(com(EstadoDoJob::SemOndeOlhar).contains("sem ARCABOOT, nao da para olhar"));
    }

    #[test]
    fn sem_arcaboot_o_status_diz_o_que_isso_impede() {
        // `arca list` funciona sem `ARCABOOT`, porque imagem mora no
        // `ARCAVAULT`. Armar, nao: a receita e o estado moram no `ARCABOOT`.
        let dispositivo = Dispositivo {
            boot: None,
            ..dispositivo_conectado()
        };

        let saida = montar_com(&dispositivo, PT);
        assert!(
            saida.contains("ausente — sem ele nao ha onde gravar"),
            "{saida}"
        );
        assert!(
            saida.contains("sem ARCABOOT conectado para conferir"),
            "{saida}"
        );
    }

    #[test]
    fn midia_removivel_leva_o_aviso_de_c6() {
        // O `bcdedit` recusa esse alvo respondendo "êxito". O aviso vem do
        // Windows, que ja sabe que aquilo e um pendrive, e chega antes de
        // alguem tentar armar.
        let dispositivo = Dispositivo {
            boot: Some(Volume {
                tipo_de_midia: TipoDeMidia::Removivel,
                ..volume(dispositivo::ARCABOOT, 'R', 1_700_000_000, 1_070_000_000)
            }),
            ..dispositivo_conectado()
        };

        let saida = montar_com(&dispositivo, PT);
        assert!(saida.contains("AVISO (C-6)"), "{saida}");
        assert!(saida.contains("F12"), "faltou dizer o que fazer");
    }

    #[test]
    fn o_dispositivo_normal_nao_leva_aviso_nenhum() {
        assert!(!montar_com(&dispositivo_conectado(), PT).contains("AVISO"));
    }

    #[test]
    fn a_listagem_de_imagens_e_a_mesma_do_arca_list() {
        let dispositivo = dispositivo_conectado();
        let pastas = uma_imagem();

        assert!(
            montar_com(&dispositivo, PT)
                .contains(&list::montar(&pastas, dispositivo.vault.livre_bytes))
        );
    }
}
