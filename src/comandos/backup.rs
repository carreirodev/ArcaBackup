//! `arca backup <nome>` — na etapa E3, so o ensaio.
//!
//! Com `--dry-run`, monta as duas receitas e as imprime inteiras. Sem ele,
//! continua dizendo que armar e a etapa E7: **armar nao e desta etapa**, e um
//! comando que armasse aqui pularia o desarmar (E4) e o selo (E5), que sao
//! justamente o que o plano poe antes do primeiro reinicio.
//!
//! # Por que o ensaio imprime tambem a receita de restauracao
//!
//! A E3 cobre R-4 e R-5, e a restauracao so ganha comando na E9. Sem
//! aparecer aqui, a unica receita destrutiva do sistema ficaria seis etapas
//! sem ninguem poder olhar para ela. Ela sai marcada como previa: e o que a
//! E9 armaria, e nao o que este comando faria.

use crate::app::Contexto;
use crate::dispositivo::{self, Dispositivo};
use crate::erro::{Erro, Resultado};
use crate::formato::{gigabytes, linha};
use crate::nome::Nome;
use crate::receita::{Disco, Operacao, Pedido, Receita, Selo};

/// O disco de origem, enquanto a E6 nao souber descobri-lo.
///
/// `nvme0n1` e o disco interno desta maquina, nomeado nas tres receitas que
/// rodaram em hardware. Quem o descobrira e
/// [`crate::portas::Discos::discos_fisicos`], que hoje devolve vazio (B-4,
/// E6). Ate la o ensaio o traz fixo, **e a saida diz que ele e suposto** —
/// uma receita destrutiva que nomeasse um disco sem avisar de onde ele veio
/// seria pior do que nao imprimir nada.
const DISCO_SUPOSTO: &str = "nvme0n1";

/// O que o ensaio tem para mostrar, antes de virar texto.
///
/// O disco nao tem campo dizendo se foi descoberto ou suposto porque **hoje
/// ele e sempre suposto**: `Discos::discos_fisicos` devolve vazio ate a E6.
/// Um campo com um valor so seria uma distincao que o codigo ainda nao sabe
/// fazer; a E6 o acrescenta quando ela existir.
pub struct Ensaio<'a> {
    pub dispositivo: &'a Dispositivo,
    pub nome: &'a Nome,
    pub disco: &'a Disco,
    pub backup: &'a Receita,
    pub restauracao: &'a Receita,
}

pub fn executar(contexto: &Contexto, nome_bruto: &str) -> Resultado<()> {
    // B-2 primeiro, e antes de tocar no dispositivo: um nome recusado nao
    // precisa de SSD conectado para ser recusado.
    let nome = Nome::novo(nome_bruto).map_err(Erro::NomeRecusado)?;

    if !contexto.dry_run {
        return Err(Erro::AindaNaoImplementado {
            comando: "backup",
            etapa: "E7",
        });
    }

    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let disco = Disco::novo(DISCO_SUPOSTO).map_err(Erro::ReceitaRecusada)?;

    // O selo de verdade nasce ao armar, na E5. Este e de ensaio, e a saida o
    // diz — ver [`Selo::de_ensaio`].
    let selo = Selo::de_ensaio();

    let montar_para = |operacao| {
        Receita::montar(&Pedido {
            operacao,
            nome: nome.clone(),
            disco: disco.clone(),
            selo: selo.clone(),
        })
        .map_err(Erro::ReceitaRecusada)
    };

    let backup = montar_para(Operacao::Backup)?;
    let restauracao = montar_para(Operacao::Restauracao)?;

    contexto.registro.info(format!(
        "ensaio de backup `{nome}` · disco {disco} (suposto) · receita de {} caracteres · validada por C-2",
        backup.comando().chars().count()
    ));

    print!(
        "{}",
        montar(&Ensaio {
            dispositivo: &dispositivo,
            nome: &nome,
            disco: &disco,
            backup: &backup,
            restauracao: &restauracao,
        })
    );
    Ok(())
}

/// O ensaio inteiro, em texto.
pub fn montar(ensaio: &Ensaio) -> String {
    let mut saida = String::new();

    saida.push_str("Ensaio (--dry-run): nada e gravado, nada e armado.\n\n");

    saida.push_str(&format!(
        "Dispositivo ARCA: {} ({}) · {} livres\n",
        dispositivo::ARCAVAULT,
        match ensaio.dispositivo.vault.letra {
            Some(letra) => format!("{letra}:"),
            None => "sem letra".to_string(),
        },
        gigabytes(ensaio.dispositivo.vault.livre_bytes)
    ));
    saida.push_str(&format!("Imagem: {}\n", ensaio.nome));
    saida.push_str(&format!(
        "Disco de origem: {} · suposto: quem o descobre e a etapa E6\n\n",
        ensaio.disco
    ));

    saida.push_str(&linha("Nome validado (B-2)", "ok"));
    saida.push_str(&linha("Receita validada (C-2)", "ok"));
    saida.push('\n');

    saida.push_str(&secao(
        "Receita de backup — e esta que a etapa E7 armaria",
        ensaio.backup,
    ));
    saida.push('\n');
    saida.push_str(&secao(
        "Receita de restauracao — previa; quem a arma e a etapa E9",
        ensaio.restauracao,
    ));

    saida.push_str(concat!(
        "\nO selo acima e de ensaio (so zeros). O de verdade nasce ao armar, na\n",
        "etapa E5, e e ele que liga o job ao desfecho.\n",
        "\nNada foi gravado. Armar e a etapa E7.\n"
    ));

    saida
}

/// Uma receita: o que o Clonezilla executa, e como ela entra no `grub.cfg`.
///
/// As duas formas, e nao so uma. A primeira e o que se lê para conferir se a
/// operacao esta certa; a segunda e o que de fato vai para o disco, com o
/// aninhamento de aspas que C-2 existe para proteger. Conferir a receita numa
/// forma e gravar a outra foi o que deixou o `--dry-run` mentir uma vez.
fn secao(titulo: &str, receita: &Receita) -> String {
    format!(
        "{titulo}\n\n  O que o Clonezilla executa:\n\n    {}\n\n  Como entra na linha do grub.cfg:\n\n    {}\n",
        receita.comando(),
        receita.parametros_do_grub()
    )
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::adaptadores::RelogioDoSistema;
    use crate::duplos::{ArquivosEmMemoria, DiscosDeMentira, FirmwareDeMentira, RelogioParado};
    use crate::portas::Volume;
    use crate::registro::Registro;

    fn dispositivo_conectado() -> Dispositivo {
        dispositivo::encontrar(&DiscosDeMentira::com_dispositivo()).unwrap()
    }

    fn receita(operacao: Operacao) -> Receita {
        Receita::montar(&Pedido {
            operacao,
            nome: Nome::novo("2026-08-22_Apps").unwrap(),
            disco: Disco::novo(DISCO_SUPOSTO).unwrap(),
            selo: Selo::de_ensaio(),
        })
        .unwrap()
    }

    fn ensaio_montado() -> String {
        let dispositivo = dispositivo_conectado();
        let nome = Nome::novo("2026-08-22_Apps").unwrap();
        let disco = Disco::novo(DISCO_SUPOSTO).unwrap();
        let backup = receita(Operacao::Backup);
        let restauracao = receita(Operacao::Restauracao);

        montar(&Ensaio {
            dispositivo: &dispositivo,
            nome: &nome,
            disco: &disco,
            backup: &backup,
            restauracao: &restauracao,
        })
    }

    #[test]
    fn o_ensaio_nomeia_a_imagem_e_o_dispositivo() {
        // Quem lê precisa saber sobre o que a receita fala antes de ler a
        // receita — nao depois, achando o nome no meio de uma linha de 700
        // caracteres.
        let saida = ensaio_montado();
        assert!(saida.contains("Imagem: 2026-08-22_Apps"), "{saida}");
        assert!(saida.contains("ARCAVAULT (E:)"), "{saida}");
    }

    #[test]
    fn o_ensaio_imprime_as_duas_receitas_inteiras() {
        // O criterio de aceite da etapa. "Inteiras" quer dizer que o que sai
        // impresso e a string que seria gravada, e nao um resumo dela.
        let saida = ensaio_montado();

        for operacao in [Operacao::Backup, Operacao::Restauracao] {
            let esperada = receita(operacao);
            assert!(
                saida.contains(esperada.comando()),
                "faltou a receita de {} inteira:\n{saida}",
                operacao.nome()
            );
            assert!(
                saida.contains(&esperada.parametros_do_grub()),
                "faltou a linha do grub.cfg da {}:\n{saida}",
                operacao.nome()
            );
        }
    }

    #[test]
    fn o_ensaio_diz_que_e_ensaio_e_que_nada_foi_gravado() {
        let saida = ensaio_montado();
        assert!(saida.contains("--dry-run"), "{saida}");
        assert!(saida.contains("Nada foi gravado"), "{saida}");
    }

    #[test]
    fn o_ensaio_nao_deixa_o_disco_suposto_passar_por_descoberto() {
        // Uma receita destrutiva que nomeasse um disco sem dizer de onde ele
        // veio e pior do que nao imprimir nada.
        assert!(ensaio_montado().contains("suposto"));
    }

    #[test]
    fn o_ensaio_avisa_que_o_selo_nao_e_de_verdade() {
        let saida = ensaio_montado();
        assert!(saida.contains("de ensaio"), "{saida}");
        assert!(saida.contains("ARCA_SELO=0000000000000000"), "{saida}");
    }

    #[test]
    fn o_ensaio_diz_qual_etapa_arma_cada_receita() {
        // A de restauracao aparece aqui porque a E3 a cobre e a E9 e quem a
        // arma. Sem essa marca, ela leria como algo que este comando faria.
        let saida = ensaio_montado();
        assert!(saida.contains("etapa E7 armaria"), "{saida}");
        assert!(saida.contains("quem a arma e a etapa E9"), "{saida}");
    }

    // ───────────────────────── o comando inteiro ─────────────────────────

    struct Bancada {
        arquivos: ArquivosEmMemoria,
        discos: DiscosDeMentira,
        firmware: FirmwareDeMentira,
        relogio: RelogioParado,
        registro: Registro,
    }

    impl Bancada {
        fn nova(discos: DiscosDeMentira) -> Bancada {
            Bancada {
                arquivos: ArquivosEmMemoria::novo(),
                discos,
                firmware: FirmwareDeMentira::novo(),
                relogio: RelogioParado::em("2026-08-22T11:42:03"),
                registro: Registro::em(
                    std::env::temp_dir().join(format!(
                        "arca-backup-{}-{:?}",
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

    #[test]
    fn sem_dry_run_o_backup_continua_dizendo_que_armar_e_a_e7() {
        // Armar e a E7 porque antes dela vem o desarmar (E4) e o selo (E5).
        // Um backup que armasse aqui deixaria a maquina com boot unico
        // pendente e nenhuma forma de cancelar.
        let bancada = Bancada::nova(DiscosDeMentira::com_dispositivo());
        let erro = executar(&bancada.contexto(false), "2026-08-22_Apps").unwrap_err();

        match erro {
            Erro::AindaNaoImplementado { etapa, comando } => {
                assert_eq!(etapa, "E7");
                assert_eq!(comando, "backup");
            }
            outro => panic!("esperava a etapa nomeada, veio {outro}"),
        }
    }

    #[test]
    fn o_ensaio_nao_escreve_nada_em_lugar_nenhum() {
        // "Nao toca em nada" e criterio de aceite, e nao promessa: o sistema
        // de arquivos de mentira comeca vazio e tem de terminar vazio.
        let bancada = Bancada::nova(DiscosDeMentira::com_dispositivo());
        executar(&bancada.contexto(true), "2026-08-22_Apps").expect("o ensaio roda");

        for caminho in [
            r"R:\boot\grub\grub.cfg",
            r"R:\arca\estado.json",
            r"E:\2026-08-22_Apps",
            r"E:\ARCA-LOGS",
        ] {
            assert!(
                bancada.arquivos.conteudo_de(caminho).is_none(),
                "o ensaio escreveu em {caminho}"
            );
        }

        // E nem falou com o firmware: armar boot unico e a E7.
        assert!(
            bancada.firmware.executados.borrow().is_empty(),
            "o ensaio mandou o bcdedit fazer alguma coisa"
        );
    }

    #[test]
    fn o_nome_e_recusado_antes_de_o_dispositivo_ser_procurado() {
        // B-2 nao precisa de SSD conectado. Recusar o nome primeiro poupa
        // quem digitou errado de ouvir "conecte o dispositivo".
        let bancada = Bancada::nova(DiscosDeMentira::default());
        let erro = executar(&bancada.contexto(true), "meu backup").unwrap_err();

        assert!(
            matches!(erro, Erro::NomeRecusado(_)),
            "esperava a recusa do nome, veio {erro}"
        );
    }

    #[test]
    fn o_nome_e_recusado_tambem_sem_dry_run() {
        // Senao um nome invalido sairia como "chega na etapa E7", e quem
        // digitou nunca saberia que o nome era o problema.
        let bancada = Bancada::nova(DiscosDeMentira::com_dispositivo());
        let erro = executar(&bancada.contexto(false), "backup;poweroff").unwrap_err();

        assert!(
            matches!(erro, Erro::NomeRecusado(_)),
            "esperava a recusa do nome, veio {erro}"
        );
    }

    #[test]
    fn sem_dispositivo_o_ensaio_recusa_em_vez_de_inventar_um() {
        let bancada = Bancada::nova(DiscosDeMentira::com_volumes(vec![Volume {
            rotulo: Some("Windows".to_string()),
            ..crate::duplos::volume("Windows", 'C', 1000, 500)
        }]));

        assert!(matches!(
            executar(&bancada.contexto(true), "2026-08-22_Apps").unwrap_err(),
            Erro::DispositivoAusente
        ));
    }
}
