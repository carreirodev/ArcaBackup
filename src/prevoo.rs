//! O pre-voo: tudo que o §5.2 mostra **antes** da confirmacao digitada.
//!
//! Cobre B-2, B-3, B-4, B-5 e B-6. Termina antes de armar, de proposito:
//! confirmar e armar sao a E7, e um pre-voo que armasse aqui pularia o que o
//! plano poe entre os dois.
//!
//! # O que este modulo julga, e o que ele so relata
//!
//! Tres coisas **recusam**: nome invalido (B-2), nome ja usado inclusive por
//! residuo (B-3), e espaco abaixo do minimo (B-4). Duas coisas so **relatam**:
//! a Inicializacao Rapida (B-5) e o `chkdsk` (B-6).
//!
//! A divisao nao e arbitraria. As tres primeiras dizem que a operacao **nao
//! pode** acontecer como pedida; as duas ultimas dizem que ela pode acontecer
//! e o usuario devia fazer alguma coisa antes. B-5 e B-6 usam a palavra
//! "oferecer", e aqui oferecer e **dizer o comando e o que ele custa** — o
//! ARCA nao roda nenhum dos dois. O §5.2 mostra as duas como linha de status,
//! sem pergunta, e e assim que elas saem.
//!
//! Isso vale ser dito porque parece uma omissao e nao e: `powercfg /h off`
//! desliga a **hibernacao inteira**, e nao so a Inicializacao Rapida. Quem
//! aceitasse a oferta perderia o "Hibernar" do menu Iniciar. Rodar isso por
//! conta propria seria o ARCA mexer em mais coisa do que anunciou.

use crate::blkdev::{NomeDoDisco, SemNome};
use crate::dispositivo::Dispositivo;
use crate::espaco::{self, Estimativa, Veredito};
use crate::formato::{gigabytes, linha, tamanho};
use crate::imagens::Pasta;
use crate::nome::Nome;
use crate::portas::{DiscoFisico, SaidaDeFerramenta, TipoDeMidia};
use std::fmt;

/// O comando que B-5 manda oferecer, com o que ele custa junto.
pub const DESLIGAR_INICIALIZACAO_RAPIDA: &str = "powercfg /h off";

/// A Inicializacao Rapida, pelo que o registro respondeu (B-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InicializacaoRapida {
    Desativada,
    Ativada,
    /// O registro nao traz o valor. **Nao e "desativada"**: ausencia de prova
    /// nunca vira o desfecho conveniente, do mesmo jeito que imagem sem
    /// veredito nao vira aprovada (ADR-0003).
    NaoSeSabe,
}

impl InicializacaoRapida {
    /// A partir do `HiberbootEnabled`. Diferente de zero e ligada.
    pub fn do_registro(valor: Option<u32>) -> InicializacaoRapida {
        match valor {
            Some(0) => InicializacaoRapida::Desativada,
            Some(_) => InicializacaoRapida::Ativada,
            None => InicializacaoRapida::NaoSeSabe,
        }
    }
}

/// O que o `chkdsk /scan` respondeu (B-6).
///
/// Julgado pelo **codigo de saida**, e nunca pelo texto: o `chkdsk` desta
/// maquina responde em portugues, e interpretar frase traduzida e o erro que a
/// E2 nomeou.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chkdsk {
    /// Codigo 0: sem problema no sistema de arquivos.
    Limpo,
    /// Codigo diferente de zero. O texto vai junto porque ele e o unico lugar
    /// onde esta o que o `chkdsk` achou.
    Acusou { codigo: i32, resumo: String },
}

impl Chkdsk {
    pub fn da_saida(saida: &SaidaDeFerramenta) -> Chkdsk {
        if saida.codigo == 0 {
            return Chkdsk::Limpo;
        }
        Chkdsk::Acusou {
            codigo: saida.codigo,
            // Poucas linhas: o `chkdsk` desta maquina imprime mais de cem, e
            // quase todas sao barra de progresso. Despejar tudo esconderia o
            // resto do dialogo do §5.2.
            resumo: saida.resumo(3),
        }
    }
}

/// Por que o pre-voo recusou. Uma variante por motivo, como toda recusa deste
/// projeto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoPreVoo {
    /// B-3: a pasta ja existe, e o ARCA nunca escreve por cima.
    NomeJaUsado { nome: String, e_residuo: bool },

    /// B-4: nao cabe.
    SemEspaco(Estimativa),

    /// Os dois rotulos nao estao no mesmo disco fisico (C-10, e a pendencia
    /// que a E4 deixou para esta etapa).
    DispositivoPartido {
        vault: char,
        boot: char,
    },

    /// O `ARCABOOT` esta em midia que o `bcdedit` recusa em silencio (C-6).
    MidiaRemovivel,
}

impl fmt::Display for RecusaDoPreVoo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoPreVoo::NomeJaUsado { nome, e_residuo: false } => write!(
                f,
                "ja ha uma imagem chamada `{nome}` no dispositivo, e o ARCA nunca escreve por cima de uma (B-3). Escolha outro nome"
            ),
            RecusaDoPreVoo::NomeJaUsado { nome, e_residuo: true } => write!(
                f,
                "ja ha uma pasta chamada `{nome}` no dispositivo, sem `MD5SUMS`: e residuo de um backup interrompido (B-3). Gravar por cima destruiria o que sobrou dele sem que ninguem tivesse olhado. Escolha outro nome, ou apague a pasta a mao — o ARCA nunca apaga nada (B-10)"
            ),
            RecusaDoPreVoo::SemEspaco(estimativa) => write!(
                f,
                "nao ha espaco no ARCAVAULT (B-4): a imagem deve ocupar cerca de {}, e ha {} livres. A estimativa e o maior entre a maior imagem do dispositivo x1,3 ({}) e o disco em uso x0,45 ({})",
                tamanho(estimativa.minimo),
                tamanho(estimativa.livre),
                tamanho(estimativa.pela_maior_imagem),
                tamanho(estimativa.pelo_em_uso)
            ),
            RecusaDoPreVoo::DispositivoPartido { vault, boot } => write!(
                f,
                "o ARCAVAULT ({vault}:) e o ARCABOOT ({boot}:) estao em discos fisicos diferentes: sao dois dispositivos meio prontos, e nao um. A receita e o estado iriam para um e as imagens estariam no outro. Desconecte o que nao for o dispositivo ARCA"
            ),
            RecusaDoPreVoo::MidiaRemovivel => write!(
                f,
                "o Windows classifica o dispositivo como midia removivel, e o bcdedit recusa esse alvo em silencio — responde \"exito\" e mantem o valor antigo (C-6). Um dispositivo assim boota por F12, nunca por entrada de firmware"
            ),
        }
    }
}

/// O disco de origem, e o que se sabe do nome que o Linux lhe da.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoDeOrigem {
    Descoberto(NomeDoDisco),
    /// Nao foi possivel determinar. **Nao e falha do pre-voo** — e uma
    /// resposta, e ela e herdada pela E7, que e quem arma.
    PorDeterminar(SemNome),
}

/// A primeira metade do dialogo: o que ja aconteceu quando ela e impressa.
///
/// # Por que o dialogo sai em duas metades
///
/// Porque o desarmar de C-1 acontece **antes** de o pre-voo julgar, e uma
/// recusa corta o resto. Imprimir tudo no fim faria o caso "nome ja usado num
/// dispositivo armado" sair como so a recusa — e o job armado teria sumido em
/// silencio. Ver o comentario em [`crate::comandos::backup::executar`].
///
/// O que esta aqui e o que ja e fato: o dispositivo, a origem, a estimativa e
/// o desarmar. O que vem depois — B-5, B-6, o disco — so faz sentido se o
/// julgamento passar.
pub struct Cabecalho<'a> {
    pub dispositivo: &'a Dispositivo,
    pub nome: &'a Nome,
    pub origem: &'a DiscoFisico,
    pub espaco: Estimativa,

    /// O que o desarmar de C-1 fez, e `None` no `--dry-run`, em que ele nao
    /// aconteceu.
    ///
    /// A primeira linha do §5.2 diz "Desarmando receita anterior ..... ok", e
    /// ela so pode dizer isso quando de fato desarmou. No ensaio ela diz outra
    /// coisa — imprimir "ok" sobre uma acao que nao aconteceu e o modo de
    /// falha que o `--dry-run` deste projeto ja teve uma vez (§11).
    pub desarme: Option<&'a crate::desarme::Desarme>,

    /// O caminho do `grub.cfg` que o desarmar toca, para a primeira linha.
    pub caminho_do_grub: &'a str,
}

/// A segunda metade: o que so se colhe depois de o julgamento passar.
pub struct PreVoo<'a> {
    pub disco: &'a DiscoDeOrigem,
    pub inicializacao_rapida: InicializacaoRapida,
    pub chkdsk: Chkdsk,
}

/// As recusas de B-3, B-4, C-6 e C-10, na ordem em que valem a pena.
///
/// A ordem e do mais barato de corrigir para o mais caro. Um nome ja usado se
/// resolve digitando outro; um dispositivo partido exige desconectar hardware.
/// Quem digitou um nome repetido nao devia ter de ouvir sobre disco fisico
/// primeiro.
pub fn julgar(
    nome: &Nome,
    pastas: &[Pasta],
    espaco: &Estimativa,
    dispositivo: &Dispositivo,
    discos: &[DiscoFisico],
) -> Result<(), RecusaDoPreVoo> {
    // B-3: recusar nome cuja pasta ja exista, **mesmo sem `MD5SUMS`**. Um
    // residuo e rastro de backup interrompido, e escrever por cima destruiria
    // o que sobrou dele sem que ninguem tivesse olhado.
    if let Some(pasta) = pastas
        .iter()
        .find(|pasta| pasta.nome.eq_ignore_ascii_case(nome.como_texto()))
    {
        return Err(RecusaDoPreVoo::NomeJaUsado {
            nome: pasta.nome.clone(),
            e_residuo: !pasta.e_imagem(),
        });
    }

    if espaco.veredito == Veredito::Insuficiente {
        return Err(RecusaDoPreVoo::SemEspaco(*espaco));
    }

    // C-6: o sinal antecipado agora e o `MediaType` do WMI, e nao o
    // `GetDriveType` — que classifica este mesmo SSD externo como disco fixo e
    // nao distingue nada (§3.1, D10).
    let do_dispositivo = dispositivo
        .boot
        .as_ref()
        .and_then(|boot| boot.letra)
        .and_then(|letra| discos.iter().find(|disco| disco.tem_a_letra(letra)));

    if do_dispositivo.is_some_and(|disco| disco.tipo_de_midia == TipoDeMidia::Removivel) {
        return Err(RecusaDoPreVoo::MidiaRemovivel);
    }

    // A pendencia que a E4 deixou nomeada para esta etapa: C-10 recusa rotulo
    // **repetido**, e nao rotulo orfao. Com dois dispositivos meio prontos na
    // mesa — um so com o `ARCAVAULT`, o outro so com o `ARCABOOT` — cada
    // rotulo aparece uma vez, a contagem passa, e a receita iria para um
    // dispositivo com as imagens no outro.
    if let (Some(vault), Some(boot)) = (
        dispositivo.vault.letra,
        dispositivo.boot.as_ref().and_then(|boot| boot.letra),
    ) {
        let disco_do_vault = discos.iter().find(|disco| disco.tem_a_letra(vault));
        let disco_do_boot = discos.iter().find(|disco| disco.tem_a_letra(boot));

        // So recusa quando a enumeracao **encontrou os dois** e eles diferem.
        // Nao achar um deles nao e prova de nada, e transformar "nao consegui
        // olhar" em "sao dispositivos diferentes" e o padrao que o ADR-0005
        // nomeou.
        if let (Some(a), Some(b)) = (disco_do_vault, disco_do_boot)
            && a.indice != b.indice
        {
            return Err(RecusaDoPreVoo::DispositivoPartido { vault, boot });
        }
    }

    Ok(())
}

/// A primeira metade do §5.2, impressa antes de o pre-voo julgar.
pub fn montar_cabecalho(cabecalho: &Cabecalho) -> String {
    let mut saida = String::new();

    saida.push_str(&format!(
        "Dispositivo ARCA: {} ({}) · {} livres\n",
        crate::dispositivo::ARCAVAULT,
        match cabecalho.dispositivo.vault.letra {
            Some(letra) => format!("{letra}:"),
            None => "sem letra".to_string(),
        },
        gigabytes(cabecalho.dispositivo.vault.livre_bytes)
    ));

    saida.push_str(&format!(
        "Origem: {} · {} · {} em uso\n",
        cabecalho.origem.modelo,
        tamanho(cabecalho.origem.tamanho_bytes),
        tamanho(cabecalho.origem.em_uso_bytes)
    ));
    saida.push_str(&format!("Imagem estimada: {}\n", cabecalho.espaco));
    saida.push_str(&format!("Imagem: {}\n\n", cabecalho.nome));

    // Esta linha e a razao de o cabecalho existir separado: o desarmar ja
    // aconteceu quando ela e impressa, e uma recusa do julgamento nao pode
    // engolir a noticia de que ele aconteceu.
    saida.push_str(&linha(
        "Desarmando receita anterior",
        &match cabecalho.desarme {
            Some(desarme) if desarme.havia_job() => {
                format!("ok · havia receita armada · {}", cabecalho.caminho_do_grub)
            }
            Some(_) => format!("ok · ja estava inerte · {}", cabecalho.caminho_do_grub),
            // No ensaio nada foi desarmado, e a linha nao pode dizer "ok".
            None => format!("nao, e ensaio · {}", cabecalho.caminho_do_grub),
        },
    ));

    saida
}

/// A segunda metade do §5.2, ate a linha antes da confirmacao.
pub fn montar_o_resto(prevoo: &PreVoo) -> String {
    let mut saida = String::new();

    saida.push_str(&linha(
        "Inicializacao rapida",
        &match prevoo.inicializacao_rapida {
            InicializacaoRapida::Desativada => "desativada   ok".to_string(),
            InicializacaoRapida::Ativada => "ATIVADA      atencao".to_string(),
            InicializacaoRapida::NaoSeSabe => "o registro nao diz   atencao".to_string(),
        },
    ));
    saida.push_str(&linha(
        "chkdsk /scan",
        &match &prevoo.chkdsk {
            Chkdsk::Limpo => "limpo        ok".to_string(),
            Chkdsk::Acusou { codigo, .. } => format!("ACUSOU (codigo {codigo})   atencao"),
        },
    ));
    saida.push_str(&linha("Nome disponivel", "ok"));
    saida.push_str(&linha(
        "Disco de origem",
        &match prevoo.disco {
            DiscoDeOrigem::Descoberto(nome) => nome.to_string(),
            DiscoDeOrigem::PorDeterminar(_) => "POR DETERMINAR".to_string(),
        },
    ));

    saida.push_str(&avisos(prevoo));

    // "Nada foi gravado" seria falso: o desarmar de C-1 grava, quando ha o que
    // desarmar. O que **nao** aconteceu e armar, e e isso que a frase diz.
    saida.push_str(concat!(
        "\nPre-voo concluido, e o dispositivo esta inerte. Nenhuma receita foi armada\n",
        "e nenhum boot unico foi marcado: quem confirma e arma e a etapa E7.\n"
    ));

    saida
}

/// O que precisa de mais de uma linha, depois da lista.
///
/// Cada aviso diz **o comando e o que ele custa**. Um aviso que so diz "isto
/// esta errado" empurra o problema de volta para quem nao sabe resolve-lo.
fn avisos(prevoo: &PreVoo) -> String {
    let mut saida = String::new();

    match prevoo.inicializacao_rapida {
        InicializacaoRapida::Ativada => {
            saida.push_str(&format!(
                "\n  INICIALIZACAO RAPIDA ATIVADA (B-5). Com ela ligada o Windows nao\n\
                 \x20 desliga de verdade: ele hiberna o kernel, e o sistema de arquivos do\n\
                 \x20 `C:` fica com estado pendente que o Clonezilla veria como sujo.\n\
                 \x20 Para desligar:  {DESLIGAR_INICIALIZACAO_RAPIDA}\n\
                 \x20 O ARCA nao roda esse comando. E ele desliga a hibernacao INTEIRA, e\n\
                 \x20 nao so a Inicializacao Rapida: o \"Hibernar\" some do menu Iniciar.\n"
            ));
        }
        InicializacaoRapida::NaoSeSabe => {
            saida.push_str(concat!(
                "\n  O registro nao traz o `HiberbootEnabled` (B-5), e o ARCA nao supoe que\n",
                "  isso queira dizer desativada. Confira antes de armar: com a\n",
                "  Inicializacao Rapida ligada, o `C:` fica com estado pendente.\n"
            ));
        }
        InicializacaoRapida::Desativada => {}
    }

    if let Chkdsk::Acusou { codigo, resumo } = &prevoo.chkdsk {
        saida.push_str(&format!(
            "\n  O chkdsk /scan saiu com codigo {codigo} (B-6):\n\
             \x20   {resumo}\n\
             \x20 Para agendar a correcao no proximo boot:  chkdsk C: /f\n\
             \x20 O ARCA nao roda esse comando: ele exige reiniciar, e reiniciar e o\n\
             \x20 que este pre-voo esta preparando.\n"
        ));
    }

    if let DiscoDeOrigem::PorDeterminar(porque) = prevoo.disco {
        saida.push_str(&format!(
            "\n  O NOME DO DISCO DE ORIGEM NAO FOI DETERMINADO.\n\
             \x20 {porque}\n\
             \x20 A receita nomeia o disco pelo nome que o **Linux** lhe da, e o Windows\n\
             \x20 nao conhece esse nome. O ARCA o lê do `blkdev.list` de uma imagem ja\n\
             \x20 existente; ele nao deriva o nome de indice nem de tipo de barramento,\n\
             \x20 porque o indice do Windows nao e o do Linux.\n"
        ));
    }

    saida
}

/// A estimativa de espaco a partir das pastas do dispositivo e do disco de
/// origem.
pub fn estimar(pastas: &[Pasta], origem: &DiscoFisico, livre: u64) -> Estimativa {
    // So imagem conta como "maior imagem": um residuo e uma gravacao pela
    // metade, e o tamanho dele nao diz nada sobre o tamanho de uma imagem
    // inteira — contaria para menos, que e o lado caro.
    let maior_imagem = pastas
        .iter()
        .filter(|pasta| pasta.e_imagem())
        .map(|pasta| pasta.tamanho_bytes)
        .max()
        .unwrap_or(0);

    espaco::avaliar(maior_imagem, origem.em_uso_bytes, livre)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::blkdev::Origem;
    use crate::duplos::volume;
    use crate::imagens::Especie;
    use crate::portas::Volume;
    use crate::receita::Disco;

    const MAIOR_IMAGEM: u64 = 38_823_813_652;
    const EM_USO: u64 = 112_973_562_368;
    const LIVRE: u64 = 176_291_147_776;

    fn disco(indice: u32, modelo: &str, letras: &[char], midia: TipoDeMidia) -> DiscoFisico {
        DiscoFisico {
            indice,
            modelo: modelo.to_string(),
            tamanho_bytes: 500_105_249_280,
            em_uso_bytes: EM_USO,
            tipo_de_midia: midia,
            letras: letras.to_vec(),
        }
    }

    /// Os dois discos desta maquina.
    fn discos() -> Vec<DiscoFisico> {
        vec![
            disco(0, "KINGSTON SNV3S500G", &['C'], TipoDeMidia::DiscoFixo),
            disco(
                1,
                "KGSSE100 256 SCSI Disk Device",
                &['E', 'R'],
                TipoDeMidia::DiscoExterno,
            ),
        ]
    }

    fn dispositivo() -> Dispositivo {
        Dispositivo {
            vault: volume(
                crate::dispositivo::ARCAVAULT,
                'E',
                254_379_290_624,
                LIVRE,
            ),
            boot: Some(Volume {
                sistema_de_arquivos: "FAT32".to_string(),
                ..volume(crate::dispositivo::ARCABOOT, 'R', 1_673_527_296, 1_101_361_152)
            }),
        }
    }

    fn imagem(nome: &str, tamanho_bytes: u64) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes,
            modificado_em: None,
            especie: Especie::Imagem { veredito: None },
        }
    }

    fn residuo(nome: &str) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes: 12_000,
            modificado_em: None,
            especie: Especie::Residuo,
        }
    }

    fn pastas() -> Vec<Pasta> {
        vec![
            imagem("2026-08-21_WindowsCompleto", MAIOR_IMAGEM),
            imagem("ARCA-TESTE-03", 35_282_371_427),
        ]
    }

    fn nome(bruto: &str) -> Nome {
        Nome::novo(bruto).expect("nome valido no teste")
    }

    fn julgar_com(bruto: &str, pastas: &[Pasta]) -> Result<(), RecusaDoPreVoo> {
        let estimativa = estimar(pastas, &discos()[0], LIVRE);
        julgar(&nome(bruto), pastas, &estimativa, &dispositivo(), &discos())
    }

    // ────────────────────────── B-3 ──────────────────────────

    #[test]
    fn nome_novo_passa() {
        assert!(julgar_com("2026-08-22_Apps", &pastas()).is_ok());
    }

    #[test]
    fn nome_de_imagem_existente_e_recusado() {
        match julgar_com("2026-08-21_WindowsCompleto", &pastas()).unwrap_err() {
            RecusaDoPreVoo::NomeJaUsado { nome, e_residuo } => {
                assert_eq!(nome, "2026-08-21_WindowsCompleto");
                assert!(!e_residuo);
            }
            outro => panic!("esperava a recusa por nome, veio {outro}"),
        }
    }

    #[test]
    fn nome_de_residuo_e_recusado_e_a_mensagem_e_outra() {
        // B-3 na letra: "mesmo sem `MD5SUMS`". E a mensagem tem de dizer que o
        // usuario apaga a mao, porque o ARCA nunca apaga nada (B-10).
        let mut com_residuo = pastas();
        com_residuo.push(residuo("2026-08-22_Interrompido"));

        match julgar_com("2026-08-22_Interrompido", &com_residuo).unwrap_err() {
            RecusaDoPreVoo::NomeJaUsado { e_residuo, .. } => {
                assert!(e_residuo);
            }
            outro => panic!("esperava a recusa por residuo, veio {outro}"),
        }

        let mensagem = RecusaDoPreVoo::NomeJaUsado {
            nome: "X".to_string(),
            e_residuo: true,
        }
        .to_string();
        assert!(mensagem.contains("residuo"), "{mensagem}");
        assert!(mensagem.contains("a mao"), "{mensagem}");
    }

    #[test]
    fn o_nome_ja_usado_e_reconhecido_em_qualquer_caixa() {
        // No NTFS `Apps` e `APPS` sao a mesma pasta. Recusar so o nome exato
        // deixaria o Clonezilla gravar por cima da imagem existente.
        let com = vec![imagem("2026-08-22_Apps", 100)];
        assert!(julgar_com("2026-08-22_APPS", &com).is_err());
    }

    // ────────────────────────── B-4 ──────────────────────────

    #[test]
    fn o_espaco_deste_dispositivo_basta() {
        let estimativa = estimar(&pastas(), &discos()[0], LIVRE);
        assert_eq!(estimativa.veredito, Veredito::Suficiente);
        assert_eq!(estimativa.minimo, 50_838_103_065);
    }

    #[test]
    fn sem_espaco_o_pre_voo_recusa_e_a_mensagem_mostra_as_duas_parcelas() {
        let apertado = estimar(&pastas(), &discos()[0], 1_000);
        let recusa = julgar(
            &nome("2026-08-22_Apps"),
            &pastas(),
            &apertado,
            &dispositivo(),
            &discos(),
        )
        .unwrap_err();

        let mensagem = recusa.to_string();
        assert!(mensagem.contains("B-4"), "{mensagem}");
        assert!(mensagem.contains("x1,3"), "{mensagem}");
        assert!(mensagem.contains("x0,45"), "{mensagem}");
    }

    #[test]
    fn o_residuo_nao_conta_como_maior_imagem() {
        // Um residuo e uma gravacao pela metade; o tamanho dele nao diz nada
        // sobre o de uma imagem inteira, e contaria **para menos** — que e o
        // lado caro da regra.
        let so_residuo = vec![residuo("2026-08-22_Interrompido")];
        let estimativa = estimar(&so_residuo, &discos()[0], LIVRE);

        assert_eq!(estimativa.pela_maior_imagem, 0);
        assert_eq!(estimativa.minimo, estimativa.pelo_em_uso);
    }

    // ─────────── C-10: a pendencia que a E4 deixou para a E6 ───────────

    #[test]
    fn os_dois_rotulos_no_mesmo_disco_passam() {
        // O caso desta mesa: `E:` e `R:` no disco 1.
        assert!(julgar_com("2026-08-22_Apps", &pastas()).is_ok());
    }

    #[test]
    fn dois_dispositivos_meio_prontos_sao_recusados() {
        // A pendencia inteira, em um teste. Cada rotulo aparece uma vez, a
        // contagem de C-10 passa, e mesmo assim sao dois dispositivos: a
        // receita e o estado iriam para um e as imagens estariam no outro.
        let partidos = vec![
            disco(0, "INTERNO", &['C'], TipoDeMidia::DiscoFixo),
            disco(1, "PRIMEIRO", &['E'], TipoDeMidia::DiscoExterno),
            disco(2, "SEGUNDO", &['R'], TipoDeMidia::DiscoExterno),
        ];

        let estimativa = estimar(&pastas(), &partidos[0], LIVRE);
        match julgar(
            &nome("2026-08-22_Apps"),
            &pastas(),
            &estimativa,
            &dispositivo(),
            &partidos,
        )
        .unwrap_err()
        {
            RecusaDoPreVoo::DispositivoPartido { vault, boot } => {
                assert_eq!((vault, boot), ('E', 'R'));
            }
            outro => panic!("esperava o dispositivo partido, veio {outro}"),
        }
    }

    #[test]
    fn nao_achar_o_disco_de_um_volume_nao_e_prova_de_que_sao_diferentes() {
        // "Nao consegui olhar" nunca vira "sao dispositivos diferentes" — o
        // padrao que o ADR-0005 nomeou. Com a enumeracao vazia, o pre-voo
        // segue: a recusa exige ter visto os dois.
        let estimativa = estimar(&pastas(), &discos()[0], LIVRE);
        assert!(
            julgar(
                &nome("2026-08-22_Apps"),
                &pastas(),
                &estimativa,
                &dispositivo(),
                &[]
            )
            .is_ok()
        );
    }

    // ────────────────────────── C-6 ──────────────────────────

    #[test]
    fn midia_removivel_e_recusa_e_o_sinal_vem_do_wmi() {
        // O `GetDriveType` classifica o SSD externo desta mesa como disco
        // **fixo** e nao distingue nada. O `MediaType` do WMI responde
        // `Removable Media` para um pendrive — e sao as palavras da §3.1 que o
        // `bcdedit` nao produz (D10).
        let com_pendrive = vec![
            disco(0, "INTERNO", &['C'], TipoDeMidia::DiscoFixo),
            disco(1, "PENDRIVE", &['E', 'R'], TipoDeMidia::Removivel),
        ];

        let estimativa = estimar(&pastas(), &com_pendrive[0], LIVRE);
        assert!(matches!(
            julgar(
                &nome("2026-08-22_Apps"),
                &pastas(),
                &estimativa,
                &dispositivo(),
                &com_pendrive
            ),
            Err(RecusaDoPreVoo::MidiaRemovivel)
        ));
    }

    // ────────────────── B-5, B-6 e a saida do §5.2 ──────────────────

    fn descoberto() -> DiscoDeOrigem {
        DiscoDeOrigem::Descoberto(NomeDoDisco {
            disco: Disco::novo("nvme0n1").unwrap(),
            origem: Origem::LidoDaImagem {
                imagem: "2026-08-21_WindowsCompleto".to_string(),
                modelo: "KINGSTON SNV3S500G".to_string(),
            },
        })
    }

    /// Um desarme como o que a E4 produz: nada havia, nada foi regravado.
    fn ja_estava_inerte() -> crate::desarme::Desarme {
        crate::desarme::Desarme {
            caminho_do_grub: std::path::PathBuf::from(r"R:\boot\grub\grub.cfg"),
            blocos_removidos: 0,
            default_devolvido: false,
            grub_regravado: false,
            boot_unico: crate::desarme::MarcaDeBootUnico::NaoHavia,
        }
    }

    fn montar_com(
        inicializacao_rapida: InicializacaoRapida,
        chkdsk: Chkdsk,
        disco: DiscoDeOrigem,
    ) -> String {
        montar_com_desarme(inicializacao_rapida, chkdsk, disco, Some(ja_estava_inerte()))
    }

    /// O dialogo inteiro, as duas metades emendadas — que e o que o comando
    /// imprime quando o julgamento passa.
    fn montar_com_desarme(
        inicializacao_rapida: InicializacaoRapida,
        chkdsk: Chkdsk,
        disco: DiscoDeOrigem,
        desarme: Option<crate::desarme::Desarme>,
    ) -> String {
        let dispositivo = dispositivo();
        let discos = discos();
        let nome = nome("2026-08-22_Apps");
        let pastas = pastas();

        let cabecalho = montar_cabecalho(&Cabecalho {
            dispositivo: &dispositivo,
            nome: &nome,
            origem: &discos[0],
            espaco: estimar(&pastas, &discos[0], LIVRE),
            desarme: desarme.as_ref(),
            caminho_do_grub: r"R:\boot\grub\grub.cfg",
        });

        let resto = montar_o_resto(&PreVoo {
            disco: &disco,
            inicializacao_rapida,
            chkdsk,
        });

        format!("{cabecalho}{resto}")
    }

    fn saida_normal() -> String {
        montar_com(
            InicializacaoRapida::Desativada,
            Chkdsk::Limpo,
            descoberto(),
        )
    }

    #[test]
    fn o_dialogo_traz_as_linhas_do_paragrafo_5_2() {
        let saida = saida_normal();

        assert!(saida.contains("Dispositivo ARCA: ARCAVAULT (E:)"), "{saida}");
        assert!(saida.contains("Origem: KINGSTON SNV3S500G"), "{saida}");
        assert!(saida.contains("Imagem estimada:"), "{saida}");
        assert!(
            saida.contains("Desarmando receita anterior ....."),
            "{saida}"
        );
        assert!(
            saida.contains(&linha("Inicializacao rapida", "desativada   ok")),
            "{saida}"
        );
        assert!(
            saida.contains(&linha("chkdsk /scan", "limpo        ok")),
            "{saida}"
        );
        assert!(saida.contains(&linha("Nome disponivel", "ok")), "{saida}");
    }

    #[test]
    fn a_primeira_linha_so_diz_ok_quando_desarmou_de_verdade() {
        // A primeira versao desta etapa imprimia "Desarmando receita anterior
        // ..... ok" sem desarmar nada. Nao era detalhe de saida: um
        // dispositivo armado com receita velha sairia daqui com "pre-voo
        // concluido, pronto para a E7" e continuaria armado. C-1 manda
        // desarmar incondicionalmente, e a linha tem de ser verdade.
        let inerte = montar_com_desarme(
            InicializacaoRapida::Desativada,
            Chkdsk::Limpo,
            descoberto(),
            Some(ja_estava_inerte()),
        );
        assert!(inerte.contains("ok · ja estava inerte"), "{inerte}");

        let havia = montar_com_desarme(
            InicializacaoRapida::Desativada,
            Chkdsk::Limpo,
            descoberto(),
            Some(crate::desarme::Desarme {
                blocos_removidos: 1,
                default_devolvido: true,
                grub_regravado: true,
                ..ja_estava_inerte()
            }),
        );
        assert!(havia.contains("ok · havia receita armada"), "{havia}");
    }

    #[test]
    fn no_ensaio_a_linha_do_desarmar_nao_diz_ok() {
        // O `--dry-run` deste projeto ja virou execucao real uma vez (§11), e
        // a defesa nao e so nao agir: e nao **dizer** que agiu. Um "ok" sobre
        // uma acao que nao aconteceu e a mesma mentira, do outro lado.
        let ensaio = montar_com_desarme(
            InicializacaoRapida::Desativada,
            Chkdsk::Limpo,
            descoberto(),
            None,
        );

        assert!(ensaio.contains("nao, e ensaio"), "{ensaio}");
        assert!(
            !ensaio.contains("Desarmando receita anterior ..... ok"),
            "o ensaio disse que desarmou:\n{ensaio}"
        );
    }

    #[test]
    fn o_dialogo_termina_antes_de_armar() {
        // O criterio de aceite da etapa. Um pre-voo que armasse aqui pularia o
        // que o plano poe entre o pre-voo e o armar.
        let saida = saida_normal();

        assert!(saida.contains("quem confirma e arma e a etapa E7"), "{saida}");
        assert!(saida.contains("Nenhuma receita foi armada"), "{saida}");

        // E a frase nao pode dizer "nada foi gravado": o desarmar de C-1 grava
        // quando ha o que desarmar. O que nao aconteceu foi **armar**.
        assert!(
            !saida.contains("nada foi gravado"),
            "a frase final promete mais do que e verdade:\n{saida}"
        );
        assert!(
            !saida.contains("Digite o nome"),
            "o pre-voo pediu confirmacao, e isso e a E7:\n{saida}"
        );
        assert!(
            !saida.contains("Reiniciando"),
            "o pre-voo falou em reiniciar:\n{saida}"
        );
    }

    #[test]
    fn a_inicializacao_rapida_ativada_diz_o_comando_e_o_que_ele_custa() {
        // "Oferecer" sem dizer que `powercfg /h off` desliga a hibernacao
        // INTEIRA seria o ARCA mexer em mais coisa do que anunciou. Quem
        // aceitasse perderia o "Hibernar" do menu Iniciar.
        let saida = montar_com(InicializacaoRapida::Ativada, Chkdsk::Limpo, descoberto());

        assert!(saida.contains("ATIVADA"), "{saida}");
        assert!(saida.contains(DESLIGAR_INICIALIZACAO_RAPIDA), "{saida}");
        assert!(saida.contains("hibernacao INTEIRA"), "{saida}");
        assert!(saida.contains("Hibernar"), "{saida}");
        assert!(
            saida.contains("O ARCA nao roda esse comando"),
            "quem roda e o usuario:\n{saida}"
        );
    }

    #[test]
    fn o_registro_que_nao_diz_nao_vira_desativada() {
        // Ausencia de prova nunca vira o desfecho conveniente — o mesmo que o
        // ADR-0003 decidiu para imagem sem veredito.
        assert_eq!(
            InicializacaoRapida::do_registro(None),
            InicializacaoRapida::NaoSeSabe
        );

        let saida = montar_com(InicializacaoRapida::NaoSeSabe, Chkdsk::Limpo, descoberto());
        assert!(saida.contains("o registro nao diz"), "{saida}");
        assert!(saida.contains("nao supoe"), "{saida}");
    }

    #[test]
    fn a_inicializacao_rapida_e_lida_do_numero_e_nao_de_frase() {
        assert_eq!(
            InicializacaoRapida::do_registro(Some(0)),
            InicializacaoRapida::Desativada
        );
        assert_eq!(
            InicializacaoRapida::do_registro(Some(1)),
            InicializacaoRapida::Ativada
        );
        // Qualquer coisa diferente de zero e ligada, e nao so o 1.
        assert_eq!(
            InicializacaoRapida::do_registro(Some(2)),
            InicializacaoRapida::Ativada
        );
    }

    #[test]
    fn o_chkdsk_e_julgado_pelo_codigo_de_saida_e_nunca_pelo_texto() {
        // O `chkdsk` desta maquina responde em portugues. Interpretar frase
        // traduzida e o erro que a E2 nomeou e que o parser do `bcdedit` foi
        // construido para evitar.
        let em_portugues = SaidaDeFerramenta {
            codigo: 0,
            texto: "Nao ha problemas no sistema de arquivos.".to_string(),
        };
        assert_eq!(Chkdsk::da_saida(&em_portugues), Chkdsk::Limpo);

        // O mesmo texto de sucesso com codigo diferente de zero: o codigo
        // ganha. Um `chkdsk` que dissesse "sem problemas" e saisse com 2 nao
        // conseguiu conferir.
        let mentiroso = SaidaDeFerramenta {
            codigo: 2,
            texto: "Nao ha problemas no sistema de arquivos.".to_string(),
        };
        assert!(matches!(
            Chkdsk::da_saida(&mentiroso),
            Chkdsk::Acusou { codigo: 2, .. }
        ));
    }

    #[test]
    fn o_chkdsk_que_acusou_diz_o_comando_para_agendar() {
        let acusou = Chkdsk::da_saida(&SaidaDeFerramenta {
            codigo: 1,
            texto: "Erros encontrados e corrigidos.\n\nOutra linha.\n".to_string(),
        });

        let saida = montar_com(InicializacaoRapida::Desativada, acusou, descoberto());
        assert!(saida.contains("codigo 1"), "{saida}");
        assert!(saida.contains("chkdsk C: /f"), "{saida}");
        assert!(saida.contains("O ARCA nao roda esse comando"), "{saida}");
    }

    #[test]
    fn o_resumo_do_chkdsk_nao_despeja_cem_linhas_de_progresso() {
        // O `chkdsk` desta maquina imprime mais de cem linhas, quase todas
        // barra de progresso. Despejar tudo esconderia o resto do §5.2.
        let muitas: String = (0..200).map(|n| format!("linha {n}\n")).collect();
        let saida = SaidaDeFerramenta {
            codigo: 1,
            texto: muitas,
        };

        assert_eq!(saida.resumo(3), "linha 0 · linha 1 · linha 2");
    }

    // ────────────── o disco de origem, e de onde o nome veio ──────────────

    #[test]
    fn o_disco_descoberto_sai_dizendo_de_onde_veio() {
        // O padrao da E3: uma receita destrutiva que nomeie um disco sem dizer
        // a origem do nome e pior do que nao imprimir nada.
        let saida = saida_normal();

        assert!(saida.contains("nvme0n1"), "{saida}");
        assert!(saida.contains("blkdev.list"), "{saida}");
        assert!(
            saida.contains("2026-08-21_WindowsCompleto"),
            "faltou dizer de que imagem:\n{saida}"
        );
    }

    #[test]
    fn o_disco_por_determinar_e_dito_e_nao_chutado() {
        // "Nao da para saber com seguranca ainda" e uma resposta — desde que
        // escrita. O que nao pode e o ARCA inventar uma derivacao e a
        // documentar como descoberta.
        let saida = montar_com(
            InicializacaoRapida::Desativada,
            Chkdsk::Limpo,
            DiscoDeOrigem::PorDeterminar(SemNome::SemOraculo),
        );

        assert!(saida.contains("POR DETERMINAR"), "{saida}");
        assert!(saida.contains("blkdev.list"), "{saida}");
        assert!(
            saida.contains("nao deriva o nome de indice"),
            "faltou dizer por que nao se chuta:\n{saida}"
        );
        assert!(
            !saida.contains("nvme0n1"),
            "o pre-voo chutou um nome de disco:\n{saida}"
        );
    }

    #[test]
    fn o_em_uso_da_origem_e_o_do_disco_e_nao_o_do_volume_com_letra() {
        // O disco desta maquina tem quatro particoes e so o `C:` tem letra. A
        // linha "Origem" mostra o que a regra de B-4 usa, e as duas tem de ser
        // o mesmo numero.
        let saida = saida_normal();
        assert!(saida.contains(&tamanho(EM_USO)), "{saida}");
    }

    #[test]
    fn cada_recusa_do_pre_voo_tem_mensagem_propria() {
        let todas = [
            RecusaDoPreVoo::NomeJaUsado {
                nome: "X".to_string(),
                e_residuo: false,
            },
            RecusaDoPreVoo::NomeJaUsado {
                nome: "X".to_string(),
                e_residuo: true,
            },
            RecusaDoPreVoo::SemEspaco(estimar(&pastas(), &discos()[0], 0)),
            RecusaDoPreVoo::DispositivoPartido {
                vault: 'E',
                boot: 'R',
            },
            RecusaDoPreVoo::MidiaRemovivel,
        ];

        for recusa in todas {
            assert!(
                recusa.to_string().chars().count() > 40,
                "{recusa:?} sem mensagem propria"
            );
        }
    }
}
