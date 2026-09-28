//! Achar o dispositivo conectado.
//!
//! Pelo rotulo, sempre — nunca por letra, `sda` ou numero de serie (B-1,
//! S-3). O `ARCAVAULT` se chama assim em todo dispositivo ARCA, e e isso que
//! torna a receita reproduzivel e os dispositivos intercambiaveis (§4 do
//! PRD). O `ARCABOOT` pode levar um nome, `ARCA-<texto>`, desde 27/09/2026
//! (C-16, ADR-0027): a receita nao o cita, e sem nome dois dispositivos
//! saiam iguais no Explorer.
//!
//! A letra que sai daqui serve para montar caminho de arquivo do lado
//! Windows, e so. Ela muda de uma conexao para outra; o rotulo, nao.

use crate::erro::{Erro, Resultado};
use crate::portas::{Discos, Volume};
use std::path::PathBuf;

/// A particao NTFS onde moram as imagens e os logs.
pub const ARCAVAULT: &str = "ARCAVAULT";

/// A particao FAT32 de onde a maquina boota, com o Clonezilla e o estado do
/// job.
///
/// E o rotulo que o `arca prepare` grava, e o nome do papel da particao. O
/// rotulo real pode ser outro, `ARCA-<texto>` — quem decide se um rotulo e de
/// boot e [`e_rotulo_de_boot`], e nao a comparacao com esta constante.
pub const ARCABOOT: &str = "ARCABOOT";

/// O comeco do rotulo de um `ARCABOOT` com nome (C-16).
///
/// Com o hifen, e o hifen e o que separa: o prefixo `ARCA` sozinho casaria o
/// `ARCAVAULT`, e so o sistema de arquivos distinguiria as duas particoes. O
/// rotulo FAT32 tem 11 caracteres, e este prefixo ocupa 5 — o nome tem ate 6.
pub const PREFIXO_DO_NOME: &str = "ARCA-";

/// A pasta de logs do dispositivo, dentro do `ARCAVAULT` (§4 do PRD).
///
/// E onde a receita grava o `arca-fim.txt` de cada job — no `ARCAVAULT`, e
/// nao dentro da pasta da imagem, porque na restauracao a imagem e a origem:
/// escrever dentro dela seria escrever no que se esta lendo. Quem escreve e
/// a receita ([`crate::receita`]); quem a pula ao enumerar imagens e
/// [`crate::imagens`], porque ela nao e imagem nem residuo.
pub const ARCA_LOGS: &str = "ARCA-LOGS";

/// Onde o estado do job mora, dentro do `ARCABOOT` (§4 e §4.1 do PRD).
///
/// No dispositivo, e nunca no `C:`, porque e o `C:` que a restauracao
/// substitui: o que julga a restauracao nao pode morar no disco que ela troca.
/// O que ha **dentro** do arquivo — selo, comando, alvo — e assunto da etapa
/// E5; daqui ate la, saber se ele existe ja diz se ha job por colher.
pub const ESTADO_DO_JOB: &str = r"arca\estado.json";

/// O `grub.cfg` do dispositivo, dentro do `ARCABOOT` (§4 do PRD).
///
/// E o arquivo em que a receita e gravada a cada operacao, e e o unico arquivo
/// de que a maquina depende para bootar. Quem o devolve ao estado inerte e
/// [`crate::desarme`]; quem sabe as duas operacoes inversas sobre o texto dele
/// e [`crate::grub`].
pub const RECEITA_NO_GRUB: &str = r"boot\grub\grub.cfg";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispositivo {
    pub vault: Volume,

    /// Ausente e possivel, e nao e o mesmo que erro: um `arca list` so precisa
    /// ler imagens, e imagem mora no `ARCAVAULT`. Quem for **armar** e que
    /// nao pode seguir sem o `ARCABOOT`, porque e la que a receita e o estado
    /// do job moram (§4.1) — essa cobranca e das etapas que armam.
    ///
    /// # Nao esta provado que este `ARCABOOT` e do mesmo dispositivo
    ///
    /// A recusa de C-10 pega volume **a mais**, e nao volume orfao. Com
    /// dois dispositivos meio prontos conectados — um mostrando so o
    /// `ARCAVAULT`, o outro so o `ARCABOOT` — cada papel aparece uma vez, a
    /// contagem passa, e este campo traz a particao do dispositivo errado.
    /// Desde C-16 (27/09/2026) o volume errado pode ser tambem um pendrive
    /// qualquer rotulado `ARCA-...`.
    ///
    /// Para `arca list` isso e inofensivo, porque ele nao olha aqui. Para
    /// quem arma, nao seria: a receita e o `estado.json` iriam para um
    /// dispositivo e as imagens estariam no outro. Fechar isso exige saber em
    /// que disco fisico cada volume esta, que e o que
    /// [`crate::portas::Discos::discos_fisicos`] entrega na etapa E6 — antes,
    /// portanto, de a E7 armar qualquer coisa.
    pub boot: Option<Volume>,
}

impl Dispositivo {
    /// A raiz do `ARCAVAULT` como caminho do lado Windows, tipo `E:\`.
    pub fn raiz_do_vault(&self) -> Resultado<PathBuf> {
        raiz_de(&self.vault, ARCAVAULT)
    }

    /// A raiz do `ARCABOOT`, quando ele esta ai.
    pub fn raiz_do_boot(&self) -> Resultado<PathBuf> {
        match &self.boot {
            Some(volume) => raiz_de(volume, ARCABOOT),
            None => Err(Erro::ParticaoAusente { rotulo: ARCABOOT }),
        }
    }

    /// O caminho do `estado.json`, quando ha `ARCABOOT` para conte-lo.
    pub fn caminho_do_estado(&self) -> Resultado<PathBuf> {
        Ok(self.raiz_do_boot()?.join(ESTADO_DO_JOB))
    }

    /// O caminho do `grub.cfg`, quando ha `ARCABOOT` para conte-lo.
    pub fn caminho_do_grub(&self) -> Resultado<PathBuf> {
        Ok(self.raiz_do_boot()?.join(RECEITA_NO_GRUB))
    }
}

/// O dispositivo conectado, ou o motivo de nao haver um.
///
/// Dois `ARCAVAULT` sao recusa dura (C-10): a receita resolve o destino por
/// LABEL, e com o rotulo repetido nao ha o que escolher — o Clonezilla
/// montaria um dos dois, e nao ha como saber qual. Dois volumes de boot
/// tambem, com rotulos iguais ou diferentes (`ARCABOOT` e `ARCA-<texto>`,
/// C-16): a receita e o estado do job teriam dois lugares para ir.
pub fn encontrar(discos: &dyn Discos) -> Resultado<Dispositivo> {
    let volumes = discos.volumes()?;

    let vaults = com_rotulo(&volumes, ARCAVAULT);
    let boots: Vec<Volume> = volumes
        .iter()
        .filter(|volume| volume.rotulo.as_deref().is_some_and(e_rotulo_de_boot))
        .cloned()
        .collect();

    // A recusa **nomeia as letras**, e isso mudou na E10.
    //
    // Ela nasceu na E1, quando ter dois dispositivos ARCA na mesa exigia
    // comprar dois — e `Desconecte os demais` bastava, porque *"os demais"*
    // era um caso raro sobre coisas que alguém tinha posto ali de propósito.
    //
    // Desde a E10 **o ARCA faz o segundo**: um `arca prepare` bem-sucedido
    // deixa dois dispositivos conectados por definição, e a partir daí todo
    // comando cai aqui — inclusive o `arca status`, que é o que alguém rodaria
    // para entender o que está acontecendo. O caso deixou de ser raro, e a
    // mensagem passou a precisar dizer **quais**.
    //
    // Desde C-16 (27/09/2026) ela diz também o **nome** dos volumes de boot,
    // quando algum tem: as letras dizem onde estão, e só o nome diz qual
    // dispositivo é qual. Sem nome nenhum, a mensagem é a de antes.
    let volumes_de_boot = boots
        .iter()
        .any(|boot| boot.rotulo.as_deref().and_then(nome_do_boot).is_some())
        .then(|| letras_e_rotulos_de(&boots));

    for (rotulo, achados) in [(ARCAVAULT, &vaults), (ARCABOOT, &boots)] {
        if achados.len() > 1 {
            return Err(Erro::DispositivosDemais {
                rotulo,
                quantos: achados.len(),
                onde: letras_de(achados),
                volumes_de_boot: volumes_de_boot.clone(),
            });
        }
    }

    let Some(vault) = vaults.into_iter().next() else {
        return Err(Erro::DispositivoAusente);
    };

    Ok(Dispositivo {
        vault,
        boot: boots.into_iter().next(),
    })
}

/// Se um rotulo e o de um `ARCABOOT`: `ARCABOOT`, ou comecando com `ARCA-`
/// (C-16). Sem diferenciar caixa, como o resto do ARCA compara rotulo.
///
/// E o casamento que `encontrar`, a contagem de C-10 e o reconhecimento de
/// dispositivo no `arca prepare` usam — um so, para os tres nao divergirem.
///
/// O prefixo e comparado com `str::get`, e nunca com `&rotulo[..5]`: num
/// rotulo como `ARCAÉ` o `É` ocupa o quinto e o sexto byte, e num como `ARC`
/// o quinto byte nem existe. A fatia entraria em panico nos dois, e todo
/// volume desta maquina passa por aqui — um pendrive com acento no nome
/// derrubaria qualquer comando. `get` responde `None`, e `None` nao e boot.
pub fn e_rotulo_de_boot(rotulo: &str) -> bool {
    rotulo.eq_ignore_ascii_case(ARCABOOT)
        || rotulo
            .get(..PREFIXO_DO_NOME.len())
            .is_some_and(|comeco| comeco.eq_ignore_ascii_case(PREFIXO_DO_NOME))
}

/// O nome que o `ARCABOOT` leva, quando leva um (C-16): o rotulo inteiro, na
/// caixa em que o Windows o devolve, se ele tem a forma `ARCA-<texto>`.
///
/// Inteiro, e nao so o texto depois do hifen, porque e assim que o Explorer o
/// mostra — e e no Explorer que os dois SSDs iguais aparecem lado a lado.
/// `ARCABOOT`, em qualquer caixa, nao e nome: e o que todo dispositivo tem.
pub fn nome_do_boot(rotulo: &str) -> Option<&str> {
    (e_rotulo_de_boot(rotulo) && !rotulo.eq_ignore_ascii_case(ARCABOOT)).then_some(rotulo)
}

/// As letras e os rotulos de uma lista de volumes, como `R: ARCA-CASA, S:
/// ARCABOOT` — para a recusa de C-10 dizer qual dispositivo e qual.
fn letras_e_rotulos_de(volumes: &[Volume]) -> String {
    volumes
        .iter()
        .map(|volume| {
            let rotulo = volume.rotulo.as_deref().unwrap_or_default();
            match volume.letra {
                Some(letra) => format!("{letra}: {rotulo}"),
                None => format!("{rotulo} sem letra"),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Os volumes que carregam um rotulo, sem diferenciar caixa.
///
/// Sem diferenciar porque quem grava o rotulo e a ferramenta de formatacao do
/// usuario, e um `Arcavault` teimoso nao pode fazer o dispositivo desaparecer.
/// As letras de uma lista de volumes, como `D:, E:` — para a recusa poder
/// dizer **quais** desconectar.
///
/// Um volume sem letra aparece como `sem letra`, e não some da lista: ele
/// conta para a ambiguidade do mesmo jeito, e omiti-lo faria a mensagem dizer
/// "há 2" e mostrar um.
fn letras_de(volumes: &[Volume]) -> String {
    volumes
        .iter()
        .map(|volume| match volume.letra {
            Some(letra) => format!("{letra}:"),
            None => "sem letra".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn com_rotulo(volumes: &[Volume], rotulo: &str) -> Vec<Volume> {
    volumes
        .iter()
        .filter(|volume| {
            volume
                .rotulo
                .as_deref()
                .is_some_and(|seu| seu.eq_ignore_ascii_case(rotulo))
        })
        .cloned()
        .collect()
}

fn raiz_de(volume: &Volume, rotulo: &'static str) -> Resultado<PathBuf> {
    match volume.letra {
        Some(letra) => Ok(PathBuf::from(format!("{letra}:\\"))),
        // Sem letra o volume existe mas nao tem caminho: nada do lado Windows
        // consegue abrir um arquivo dentro dele.
        None => Err(Erro::VolumeSemLetra { rotulo }),
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{DiscosDeMentira, volume};
    use crate::portas::Volume;

    fn sem_rotulo() -> Volume {
        Volume {
            rotulo: None,
            ..volume("x", 'Z', 1000, 500)
        }
    }

    #[test]
    fn acha_o_dispositivo_pelos_rotulos() {
        let discos = DiscosDeMentira::com_dispositivo();
        let dispositivo = encontrar(&discos).expect("o dispositivo esta conectado");

        assert_eq!(dispositivo.vault.rotulo.as_deref(), Some(ARCAVAULT));
        assert_eq!(dispositivo.raiz_do_vault().unwrap(), PathBuf::from("E:\\"));
        assert_eq!(dispositivo.raiz_do_boot().unwrap(), PathBuf::from("R:\\"));
    }

    #[test]
    fn a_letra_nao_decide_nada_o_rotulo_decide() {
        // O mesmo dispositivo em outra letra continua sendo o mesmo
        // dispositivo. E o que S-3 quer dizer.
        let discos = DiscosDeMentira::com_volumes(vec![
            volume("Windows", 'C', 1000, 500),
            volume(ARCAVAULT, 'Z', 254_000_000_000, 176_400_000_000),
        ]);

        let dispositivo = encontrar(&discos).unwrap();
        assert_eq!(dispositivo.raiz_do_vault().unwrap(), PathBuf::from("Z:\\"));
    }

    #[test]
    fn o_rotulo_e_reconhecido_em_qualquer_caixa() {
        let discos = DiscosDeMentira::com_volumes(vec![volume("ArcaVault", 'E', 1000, 500)]);
        assert!(encontrar(&discos).is_ok());
    }

    #[test]
    fn sem_arcavault_nao_ha_dispositivo() {
        let discos =
            DiscosDeMentira::com_volumes(vec![volume("Windows", 'C', 1000, 500), sem_rotulo()]);

        assert!(matches!(
            encontrar(&discos).unwrap_err(),
            Erro::DispositivoAusente
        ));
    }

    #[test]
    fn dois_arcavault_sao_recusa_dura() {
        // C-10: e por LABEL que a receita resolve o destino. Com o rotulo
        // repetido nao ha como saber qual dos dois o Clonezilla montaria.
        let discos = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume(ARCAVAULT, 'F', 1000, 500),
        ]);

        let erro = encontrar(&discos).unwrap_err();
        match &erro {
            Erro::DispositivosDemais {
                rotulo,
                quantos,
                onde,
                ..
            } => {
                assert_eq!(*rotulo, ARCAVAULT);
                assert_eq!(*quantos, 2);

                // **As letras estao na mensagem desde a E10.** `Desconecte os
                // demais` sem dizer quais empurra a pergunta de volta para
                // quem nao tem como responde-la — e o caso deixou de ser raro
                // quando o `arca prepare` passou a criar o segundo
                // dispositivo.
                assert_eq!(onde, "E:, F:");
            }
            outro => panic!("esperava recusa por ambiguidade, veio {outro}"),
        }

        assert!(erro.to_string().contains("E:, F:"), "{erro}");
        assert!(
            erro.to_string().contains("acabou de preparar"),
            "a mensagem tem de nomear a causa mais provavel: {erro}"
        );
    }

    #[test]
    fn um_volume_sem_letra_nao_some_da_recusa() {
        // Ele conta para a ambiguidade do mesmo jeito, e omiti-lo faria a
        // mensagem dizer "ha 2" e mostrar um.
        let mut sem_letra = volume(ARCAVAULT, 'F', 1000, 500);
        sem_letra.letra = None;

        let discos =
            DiscosDeMentira::com_volumes(vec![volume(ARCAVAULT, 'E', 1000, 500), sem_letra]);

        let erro = encontrar(&discos).unwrap_err();
        assert!(erro.to_string().contains("E:, sem letra"), "{erro}");
    }

    #[test]
    fn dois_arcaboot_tambem_sao_recusa_dura() {
        let discos = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume(ARCABOOT, 'R', 1000, 500),
            volume(ARCABOOT, 'S', 1000, 500),
        ]);

        match encontrar(&discos).unwrap_err() {
            Erro::DispositivosDemais { rotulo, .. } => assert_eq!(rotulo, ARCABOOT),
            outro => panic!("esperava recusa por ambiguidade, veio {outro}"),
        }
    }

    #[test]
    fn o_arcaboot_renomeado_e_o_boot_do_dispositivo() {
        // C-16: quem tem dois dispositivos renomeia o `ARCABOOT` de cada um no
        // Explorer, e o renomeado continua sendo onde a receita e o estado do
        // job moram (ADR-0027).
        let discos = DiscosDeMentira::com_volumes(vec![
            volume("Windows", 'C', 1000, 500),
            volume(ARCAVAULT, 'E', 1000, 500),
            volume("ARCA-CASA", 'R', 1000, 500),
        ]);

        let dispositivo = encontrar(&discos).expect("o dispositivo esta conectado");
        assert_eq!(dispositivo.boot, Some(volume("ARCA-CASA", 'R', 1000, 500)));
        assert_eq!(
            dispositivo.caminho_do_grub().unwrap(),
            PathBuf::from(r"R:\boot\grub\grub.cfg")
        );
        assert_eq!(
            dispositivo.caminho_do_estado().unwrap(),
            PathBuf::from(r"R:\arca\estado.json")
        );
    }

    #[test]
    fn o_rotulo_de_boot_casa_arcaboot_ou_arca_hifen_em_qualquer_caixa() {
        // `ARCA-` sem texto e a forma, e conta (suposicao confirmada em
        // 28/09/2026): recusa-lo pediria uma mensagem nova para um caso sem
        // uso.
        for rotulo in ["ARCABOOT", "arcaboot", "ARCA-CASA", "arca-Casa", "ARCA-"] {
            let discos = DiscosDeMentira::com_volumes(vec![
                volume(ARCAVAULT, 'E', 1000, 500),
                volume(rotulo, 'R', 1000, 500),
            ]);

            let boot = encontrar(&discos).unwrap().boot;
            assert_eq!(
                boot.as_ref().and_then(|boot| boot.letra),
                Some('R'),
                "{rotulo}"
            );
            assert_eq!(boot.and_then(|boot| boot.rotulo).as_deref(), Some(rotulo));
        }
    }

    #[test]
    fn o_que_nao_tem_a_forma_fica_fora_do_arcaboot_sem_panico() {
        // O hifen e o que separa: sem ele, `ARCACASA` — e o proprio
        // `ARCAVAULT` — casariam o prefixo. E os dois ultimos sao os que
        // derrubariam uma fatia de bytes: `ARC` e mais curto que o prefixo, e
        // em `ARCAÉ` o `É` ocupa o quinto e o sexto byte, entao `&rotulo[..5]`
        // pararia entre os dois. `ÉRCA-X` nao derrubaria — o quinto byte ali e
        // fronteira —, e esta aqui porque e o exemplo que a AC 5 nomeia.
        for rotulo in ["ARCACASA", "ARC", "ÉRCA-X", "ARCAÉ"] {
            let discos = DiscosDeMentira::com_volumes(vec![
                volume(ARCAVAULT, 'E', 1000, 500),
                volume(rotulo, 'R', 1000, 500),
            ]);

            assert!(encontrar(&discos).unwrap().boot.is_none(), "{rotulo}");
        }
    }

    #[test]
    fn dois_volumes_de_boot_sao_recusa_dura_com_rotulos_iguais_ou_diferentes() {
        // C-10 conta os volumes de boot pela forma, e nao por um rotulo so:
        // com dois, a receita e o estado do job teriam dois lugares para ir.
        for (primeiro, segundo) in [
            ("ARCABOOT", "ARCA-CASA"),
            ("ARCA-CASA", "ARCA-ESCRIT"),
            ("ARCA-CASA", "ARCA-CASA"),
        ] {
            let discos = DiscosDeMentira::com_volumes(vec![
                volume(ARCAVAULT, 'E', 1000, 500),
                volume(primeiro, 'R', 1000, 500),
                volume(segundo, 'S', 1000, 500),
            ]);

            match encontrar(&discos).unwrap_err() {
                Erro::DispositivosDemais { quantos, .. } => {
                    assert_eq!(quantos, 2, "{primeiro} + {segundo}");
                }
                outro => panic!("{primeiro} + {segundo}: esperava C-10, veio {outro}"),
            }
        }
    }

    #[test]
    fn a_recusa_por_dois_volumes_de_boot_nomeados_diz_a_letra_e_o_rotulo_de_cada_um() {
        // "com o rotulo ARCABOOT" deixa de ser verdade quando os rotulos
        // diferem, e "e pelo rotulo que a receita resolve o destino" nunca foi
        // verdade para o boot: a receita so cita o `ARCAVAULT`. O que torna
        // dois volumes de boot ambiguos e que a receita e o estado do job sao
        // gravados neles (C-16, ADR-0027).
        let discos = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume("ARCA-CASA", 'R', 1000, 500),
            volume("ARCA-ESCRIT", 'S', 1000, 500),
        ]);

        let mensagem = encontrar(&discos).unwrap_err().to_string();
        for deve in [
            "R: ARCA-CASA, S: ARCA-ESCRIT",
            "estado do job",
            "receita",
            "Desconecte os demais",
            "Se voce acabou de preparar um dispositivo",
        ] {
            assert!(mensagem.contains(deve), "faltou `{deve}`: {mensagem}");
        }
        for nao_deve in [
            "com o rotulo ARCABOOT",
            "repetido",
            "e pelo rotulo que a receita resolve o destino",
        ] {
            assert!(
                !mensagem.contains(nao_deve),
                "sobrou `{nao_deve}`: {mensagem}"
            );
        }

        let um_sem_nome = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume(ARCABOOT, 'R', 1000, 500),
            volume("ARCA-CASA", 'S', 1000, 500),
        ]);
        let mensagem = encontrar(&um_sem_nome).unwrap_err().to_string();
        assert!(mensagem.contains("R: ARCABOOT, S: ARCA-CASA"), "{mensagem}");
    }

    #[test]
    fn a_recusa_por_dois_arcavault_nomeia_os_volumes_de_boot_quando_ha_nome() {
        // Com dois dispositivos inteiros na mesa, o `ARCAVAULT` e contado
        // primeiro, e e nesta recusa que o ARCA manda desconectar um. Os dois
        // `ARCAVAULT` sao iguais no Explorer; o nome do boot e o que diz qual
        // e qual.
        let discos = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume(ARCAVAULT, 'F', 1000, 500),
            volume("ARCA-CASA", 'R', 1000, 500),
            volume(ARCABOOT, 'S', 1000, 500),
        ]);

        let erro = encontrar(&discos).unwrap_err();
        match &erro {
            Erro::DispositivosDemais { rotulo, onde, .. } => {
                assert_eq!(*rotulo, ARCAVAULT);
                assert_eq!(onde, "E:, F:");
            }
            outro => panic!("esperava C-10, veio {outro}"),
        }
        let mensagem = erro.to_string();
        assert!(mensagem.contains("(E:, F:)"), "{mensagem}");
        assert!(mensagem.contains("R: ARCA-CASA, S: ARCABOOT"), "{mensagem}");
    }

    #[test]
    fn sem_volume_nomeado_as_recusas_de_c10_saem_como_antes() {
        // C-16 (28/09/2026) muda as duas recusas quando algum volume de boot
        // tem nome, e so entao. Sem nome, o texto e o que ja foi capturado em
        // hardware (README §5, 23/08/2026) — e a comparacao e com o literal,
        // e nao com um texto remontado pelo codigo que esta sendo testado.
        let dois_vaults = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume(ARCAVAULT, 'F', 1000, 500),
            volume(ARCABOOT, 'R', 1000, 500),
        ]);
        assert_eq!(
            encontrar(&dois_vaults).unwrap_err().to_string(),
            "ha 2 volumes com o rotulo ARCAVAULT conectados (E:, F:), e o ARCA opera um \
             dispositivo por vez: e pelo rotulo que a receita resolve o destino, e com ele \
             repetido nao ha o que escolher. Desconecte os demais e rode de novo. Se voce \
             acabou de preparar um dispositivo, sao os dois — o novo e o de antes"
        );

        let dois_boots = DiscosDeMentira::com_volumes(vec![
            volume(ARCAVAULT, 'E', 1000, 500),
            volume(ARCABOOT, 'R', 1000, 500),
            volume(ARCABOOT, 'S', 1000, 500),
        ]);
        assert_eq!(
            encontrar(&dois_boots).unwrap_err().to_string(),
            "ha 2 volumes com o rotulo ARCABOOT conectados (R:, S:), e o ARCA opera um \
             dispositivo por vez: e pelo rotulo que a receita resolve o destino, e com ele \
             repetido nao ha o que escolher. Desconecte os demais e rode de novo. Se voce \
             acabou de preparar um dispositivo, sao os dois — o novo e o de antes"
        );
    }

    #[test]
    fn sem_arcaboot_o_dispositivo_ainda_serve_para_listar() {
        let discos = DiscosDeMentira::com_volumes(vec![volume(ARCAVAULT, 'E', 1000, 500)]);
        let dispositivo = encontrar(&discos).unwrap();

        assert!(dispositivo.boot.is_none());
        assert!(dispositivo.raiz_do_vault().is_ok());
        assert!(matches!(
            dispositivo.raiz_do_boot().unwrap_err(),
            Erro::ParticaoAusente { .. }
        ));
    }

    #[test]
    fn o_estado_do_job_mora_no_arcaboot_e_so_la() {
        // §4.1: o que julga a restauracao nao pode morar no disco que ela
        // substitui. Sem `ARCABOOT` nao ha caminho nenhum — e isso nao e o
        // mesmo que nao haver job.
        let dispositivo = encontrar(&DiscosDeMentira::com_dispositivo()).unwrap();
        assert_eq!(
            dispositivo.caminho_do_estado().unwrap(),
            PathBuf::from(r"R:\arca\estado.json")
        );

        let sem_boot = encontrar(&DiscosDeMentira::com_volumes(vec![volume(
            ARCAVAULT, 'E', 1000, 500,
        )]))
        .unwrap();
        assert!(matches!(
            sem_boot.caminho_do_estado().unwrap_err(),
            Erro::ParticaoAusente { .. }
        ));
    }

    #[test]
    fn o_grub_cfg_mora_no_arcaboot_e_so_la() {
        // E o arquivo em que a receita e gravada, e o unico de que a maquina
        // depende para bootar. Sem `ARCABOOT` nao ha caminho nenhum — e e por
        // isso que desarmar exige a particao que `arca list` dispensa.
        let dispositivo = encontrar(&DiscosDeMentira::com_dispositivo()).unwrap();
        assert_eq!(
            dispositivo.caminho_do_grub().unwrap(),
            PathBuf::from(r"R:\boot\grub\grub.cfg")
        );

        let sem_boot = encontrar(&DiscosDeMentira::com_volumes(vec![volume(
            ARCAVAULT, 'E', 1000, 500,
        )]))
        .unwrap();
        assert!(matches!(
            sem_boot.caminho_do_grub().unwrap_err(),
            Erro::ParticaoAusente { .. }
        ));
    }

    #[test]
    fn volume_sem_letra_nao_tem_caminho() {
        let discos = DiscosDeMentira::com_volumes(vec![Volume {
            letra: None,
            ..volume(ARCAVAULT, 'E', 1000, 500)
        }]);

        let dispositivo = encontrar(&discos).unwrap();
        assert!(matches!(
            dispositivo.raiz_do_vault().unwrap_err(),
            Erro::VolumeSemLetra { .. }
        ));
    }
}
