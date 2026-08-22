//! Achar o dispositivo conectado.
//!
//! Pelo rotulo, sempre — nunca por letra, `sda` ou numero de serie (B-1,
//! S-3). Os rotulos sao os mesmos em todo dispositivo ARCA, e e isso que
//! torna a receita reproduzivel e os dispositivos intercambiaveis (§4 do
//! PRD).
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
pub const ARCABOOT: &str = "ARCABOOT";

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
    /// A recusa de C-10 pega rotulo **repetido**, e nao rotulo orfao. Com
    /// dois dispositivos meio prontos conectados — um mostrando so o
    /// `ARCAVAULT`, o outro so o `ARCABOOT` — cada rotulo aparece uma vez, a
    /// contagem passa, e este campo traz a particao do dispositivo errado.
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
/// Dois `ARCAVAULT` ou dois `ARCABOOT` sao recusa dura (C-10): a receita
/// resolve o destino por LABEL, e com o rotulo repetido nao ha o que escolher
/// — o Clonezilla montaria um dos dois, e nao ha como saber qual.
pub fn encontrar(discos: &dyn Discos) -> Resultado<Dispositivo> {
    let volumes = discos.volumes()?;

    let vaults = com_rotulo(&volumes, ARCAVAULT);
    let boots = com_rotulo(&volumes, ARCABOOT);

    if vaults.len() > 1 {
        return Err(Erro::DispositivosDemais {
            rotulo: ARCAVAULT,
            quantos: vaults.len(),
        });
    }
    if boots.len() > 1 {
        return Err(Erro::DispositivosDemais {
            rotulo: ARCABOOT,
            quantos: boots.len(),
        });
    }

    let Some(vault) = vaults.into_iter().next() else {
        return Err(Erro::DispositivoAusente);
    };

    Ok(Dispositivo {
        vault,
        boot: boots.into_iter().next(),
    })
}

/// Os volumes que carregam um rotulo, sem diferenciar caixa.
///
/// Sem diferenciar porque quem grava o rotulo e a ferramenta de formatacao do
/// usuario, e um `Arcavault` teimoso nao pode fazer o dispositivo desaparecer.
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

        match encontrar(&discos).unwrap_err() {
            Erro::DispositivosDemais { rotulo, quantos } => {
                assert_eq!(rotulo, ARCAVAULT);
                assert_eq!(quantos, 2);
            }
            outro => panic!("esperava recusa por ambiguidade, veio {outro}"),
        }
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
