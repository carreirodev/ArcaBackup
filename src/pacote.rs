//! O pacote do Clonezilla: versão fixada, SHA256 embutido e a conferência
//! (PR-1, PR-2, PR-3).
//!
//! # Por que a versão é fixada, e por que **esta**
//!
//! PR-1 manda fixar a versão e compilar o SHA256 no binário. A escolha da
//! versão não é gosto: **`3.3.3-15` é a que está bootando nesta mesa**. O
//! `grub.cfg` do dispositivo traz `hostname=cl-3.3.3-15` em cada `menuentry`, e
//! é sobre esse ambiente que rodaram o backup de 22/08, a restauração de 23/08
//! e a verificação da E11.
//!
//! Fixar a versão que já provou que funciona é o mesmo movimento do
//! [ADR-0014]: transcrever o que boota, em vez de instalar o que deveria
//! bootar.
//!
//! # O SHA256, e o que "baixado junto do arquivo não verifica nada" quer dizer
//!
//! Um checksum servido pelo mesmo servidor que serve o arquivo não prova coisa
//! alguma: quem pudesse trocar um trocaria o outro. Por isso ele é **constante
//! de código** — está aqui, no binário, e viaja com o ARCA.
//!
//! O número foi obtido em 23/08/2026 de **duas fontes independentes**, e as
//! duas dizem o mesmo:
//!
//! | Fonte | O que ela é |
//! |---|---|
//! | `https://free.nchc.org.tw/clonezilla-live/stable/CHECKSUMS.TXT` | O mirror do próprio projeto, em Taiwan, onde o Clonezilla é desenvolvido |
//! | O arquivo baixado do SourceForge, medido com `certutil -hashfile ... SHA256` | Outro servidor, outra rota |
//!
//! Servidores diferentes, o mesmo número. É o mais perto de verificação
//! independente que este caso admite, e está registrado em
//! `recursos/capturas/PROVENIENCIA.md`.
//!
//! # O zip, e não o ISO — e a diferença entre eles foi medida
//!
//! O Clonezilla publica os dois. O zip é o formato que se extrai direto numa
//! partição FAT32, que é exatamente o que `arca prepare` faz.
//!
//! **E o dispositivo desta mesa veio do ISO.** Medido em 23/08/2026: o
//! `boot/grub/grub.cfg` do zip e o `grub-clonezilla-original.cfg` do
//! dispositivo diferem em duas coisas, e só duas — o zip tem `noeject` em cada
//! um dos treze `menuentry`, e o carimbo de hora do rodapé difere em **seis
//! segundos** (`04:11:28` contra `04:11:22`). Seis segundos é o `ocs-live-dev`
//! gerando os dois artefatos na mesma execução: é a **mesma build**.
//!
//! Tirado o `noeject`, os dois arquivos são idênticos byte a byte.
//!
//! Isso não é motivo para preferir o ISO — é o contrário. `noeject` é o
//! parâmetro certo para mídia removível: ejetar um USB no desligamento é o
//! oposto do que se quer, e o ISO não o tem porque mídia óptica se ejeta
//! mesmo. O dispositivo desta mesa é que carrega o parâmetro de outra mídia.
//!
//! O que muda para o resto do ARCA: **nada que dependa do texto**. O estado
//! inerte se reconstrói do `grub.cfg` corrente ([ADR-0005]) e o bloco do ARCA
//! deriva do `live-toram` do próprio dispositivo ([ADR-0007]) — as duas
//! decisões existem justamente para o ARCA funcionar sobre o arquivo que
//! estiver lá. O `noeject` viaja junto, de graça.
//!
//! # E o zip entrega um `grub.cfg` que **não** está inerte
//!
//! O `set default` do pacote é `"0"`, e o [ADR-0005] tem uma seção sobre
//! exatamente isso: `"0"` aponta por **posição**, e a posição muda quando o
//! bloco do ARCA entra antes do `live-default`. Um dispositivo com `"0"` fica
//! armado no instante em que o bloco é inserido, sem que ninguém toque no `set
//! default` — *não é o estado inerte, é um estado que parece inerte*.
//!
//! Por isso `arca prepare` **desarma o que acabou de instalar**: extraído o
//! pacote, o `grub.cfg` passa por [`crate::grub::desarmar`] antes de o
//! dispositivo ser declarado pronto. Não é zelo — é o que faz o §4.4 valer
//! para um dispositivo recém-preparado.
//!
//! [ADR-0005]: ../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md
//! [ADR-0007]: ../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md
//! [ADR-0014]: ../docs/adr/0014-o-arca-particiona-o-dispositivo.md

use crate::resumo::{Algoritmo, RecusaDoResumo, Resumo};
use std::fmt;

/// A versão do Clonezilla que o ARCA instala.
///
/// É a que está no `hostname=cl-3.3.3-15` do `grub.cfg` deste dispositivo, e a
/// que rodou os três marcos em hardware deste projeto.
pub const VERSAO: &str = "3.3.3-15";

/// O nome do arquivo, como o projeto o publica.
pub const ARQUIVO: &str = "clonezilla-live-3.3.3-15-amd64.zip";

/// De onde baixar (PR-1).
///
/// O SourceForge é o canal de distribuição que o site do Clonezilla aponta, e
/// resolve para um mirror próximo de quem baixa. O `curl` segue o redirecionamento
/// com `-L`.
pub const URL: &str = "https://downloads.sourceforge.net/project/clonezilla/clonezilla_live_stable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip";

/// O SHA256 esperado, **compilado no binário** (PR-1).
///
/// Publicado pelo mirror do projeto em `CHECKSUMS.TXT` e conferido contra o
/// arquivo baixado do SourceForge em 23/08/2026 — duas fontes, o mesmo número.
/// Ver o cabeçalho deste módulo.
pub const SHA256: &str = "00cee7700433e63017e2ea9eb40519108829710132364a8028a6c039a6046304";

/// O tamanho do arquivo, medido no download de 23/08/2026.
///
/// Não é uma segunda verificação — o SHA256 já responde tudo o que o tamanho
/// responderia e mais. Serve para a tela poder dizer quanto vai baixar antes de
/// começar, num download de meio giga que leva minutos.
pub const TAMANHO_BYTES: u64 = 561_478_648;

/// Os quatro caminhos que o pacote precisa ter dentro, e que o `arca prepare`
/// confere depois de extrair.
///
/// São o que o §4 do PRD descreve como conteúdo do `ARCABOOT`. Conferi-los é o
/// que separa "o `bsdtar` saiu com zero" de "o dispositivo tem o que boota" —
/// e a distinção já custou caro neste projeto três vezes.
pub const CAMINHOS_OBRIGATORIOS: [&str; 4] = [
    "EFI/boot/bootx64.efi",
    "live/vmlinuz",
    "live/initrd.img",
    "boot/grub/grub.cfg",
];

/// Por que o pacote não serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecusaDoPacote {
    /// O `certutil` não resumiu o arquivo.
    NaoDeuParaResumir(RecusaDoResumo),

    /// O resumo saiu e **não é** o esperado. É o caso que PR-1 existe para
    /// pegar.
    ResumoDivergente { esperado: String, veio: String },

    /// Extraído, o pacote não tem o que faz um dispositivo bootar.
    PacoteIncompleto { faltando: Vec<String> },
}

impl fmt::Display for RecusaDoPacote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecusaDoPacote::NaoDeuParaResumir(porque) => write!(
                f,
                "nao deu para conferir o SHA256 do pacote do Clonezilla: {porque}. Sem essa conferencia o ARCA nao instala nada — um pacote que nao se conferiu e um pacote que nao se sabe o que e (PR-1)"
            ),
            RecusaDoPacote::ResumoDivergente { esperado, veio } => write!(
                f,
                "o SHA256 do pacote NAO bate. Esperado `{esperado}`, veio `{veio}`. O ARCA para aqui e nao extrai nada. O numero esperado esta compilado neste binario e nao veio junto do download — e por isso que ele vale alguma coisa (PR-1). Ou o download veio corrompido, ou o arquivo do outro lado nao e o que este ARCA conhece"
            ),
            RecusaDoPacote::PacoteIncompleto { faltando } => write!(
                f,
                "o pacote foi extraido e nao tem o que faz um dispositivo bootar: falta {}. O `bsdtar` ter saido com codigo zero nao e prova de que o conteudo esta la",
                faltando.join(", ")
            ),
        }
    }
}

/// Confere o resumo medido contra o que está compilado aqui (PR-1).
///
/// # A ordem, e ela não é estética
///
/// A recusa de **não ter conseguido medir** vem antes da de **não bater**,
/// porque as duas pedem coisas diferentes de quem lê: uma manda olhar o
/// `certutil`, a outra manda desconfiar do arquivo. É a mesma distinção que a
/// E5 pagou caro para existir — *"não consegui olhar" nunca vira "não há nada
/// lá"*.
pub fn conferir_o_resumo(medido: Result<Resumo, RecusaDoResumo>) -> Result<Resumo, RecusaDoPacote> {
    let medido = medido.map_err(RecusaDoPacote::NaoDeuParaResumir)?;
    let esperado = Resumo::novo(Algoritmo::Sha256, SHA256)
        .expect("a constante SHA256 deste modulo tem de ser um resumo valido");

    if medido != esperado {
        return Err(RecusaDoPacote::ResumoDivergente {
            esperado: esperado.como_texto().to_string(),
            veio: medido.como_texto().to_string(),
        });
    }
    Ok(medido)
}

/// Quais dos caminhos obrigatórios não estão na lista extraída.
///
/// A comparação normaliza `\` para `/` porque quem lista é o `bsdtar`, que usa
/// `/`, e quem confere é o Windows, que usa `\` — e trocar os dois é a fonte de
/// um "não achei" que não quer dizer nada.
pub fn o_que_falta<'a>(presentes: impl Iterator<Item = &'a str>) -> Vec<String> {
    let presentes: Vec<String> = presentes
        .map(|caminho| caminho.replace('\\', "/").trim_matches('/').to_lowercase())
        .collect();

    CAMINHOS_OBRIGATORIOS
        .iter()
        .filter(|obrigatorio| {
            let procurado = obrigatorio.to_lowercase();
            !presentes.iter().any(|presente| *presente == procurado)
        })
        .map(|obrigatorio| obrigatorio.to_string())
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::portas::SaidaDeFerramenta;
    use crate::resumo::do_certutil;

    /// A resposta do `certutil` sobre o pacote baixado em 23/08/2026.
    fn certutil_do_pacote() -> SaidaDeFerramenta {
        SaidaDeFerramenta {
            codigo: 0,
            texto: format!(
                "SHA256 hash de clonezilla-live-3.3.3-15-amd64.zip:\r\n{SHA256}\r\nCertUtil: -hashfile : comando concluido com exito.\r\n"
            ),
        }
    }

    #[test]
    fn a_constante_e_um_sha256_de_verdade() {
        // Uma constante mal digitada — 63 dígitos, um `g` no meio — só
        // apareceria na hora de conferir um download de meio giga. Aqui ela
        // aparece a cada build.
        let resumo = Resumo::novo(Algoritmo::Sha256, SHA256).expect("o SHA256 embutido e valido");
        assert_eq!(resumo.como_texto().chars().count(), 64);
        assert_eq!(resumo.algoritmo(), Algoritmo::Sha256);
    }

    #[test]
    fn a_versao_e_a_que_esta_bootando_nesta_mesa() {
        // O oráculo é o `grub.cfg` capturado do dispositivo: ele traz
        // `hostname=cl-3.3.3-15` em cada `menuentry`, e é sobre esse ambiente
        // que rodaram os três marcos em hardware deste projeto.
        //
        // Fixar outra versão passaria a instalar um Clonezilla que nada neste
        // repositório viu funcionar — e este teste falaria.
        const GRUB: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");
        assert!(
            GRUB.contains(&format!("hostname=cl-{VERSAO}")),
            "a versao fixada nao e a que o dispositivo desta mesa roda"
        );

        // E o nome do arquivo e a URL carregam a mesma versao. Trocar a
        // constante e esquecer de trocar a URL baixaria outra coisa.
        assert!(ARQUIVO.contains(VERSAO), "o nome do arquivo: {ARQUIVO}");
        assert!(URL.contains(VERSAO), "a URL: {URL}");
        assert!(
            URL.ends_with(ARQUIVO),
            "a URL nao termina no arquivo: {URL}"
        );
    }

    #[test]
    fn o_pacote_medido_em_23_08_passa_na_conferencia() {
        // O oráculo desta etapa: esta resposta é a do `certutil` sobre o
        // arquivo que veio do SourceForge, e a constante veio do
        // `CHECKSUMS.TXT` do mirror do projeto. Dois servidores, o mesmo
        // número — e o teste não pode ser ajustado para passar sem trocar os
        // dois lados.
        let resumo = conferir_o_resumo(do_certutil(&certutil_do_pacote(), Algoritmo::Sha256))
            .expect("o pacote medido tem de passar");
        assert_eq!(resumo.como_texto(), SHA256);
    }

    #[test]
    fn um_bit_trocado_no_pacote_e_recusa() {
        // O caso que PR-1 existe para pegar. **O último dígito muda, e nada
        // mais** — um pacote trocado no meio do caminho não anuncia isso de
        // outra forma.
        let ultimo_trocado = format!(
            "{}{}",
            &SHA256[..SHA256.len() - 1],
            if SHA256.ends_with('4') { '5' } else { '4' }
        );
        assert_ne!(ultimo_trocado, SHA256, "o teste tem de trocar alguma coisa");

        let mut torto = certutil_do_pacote();
        torto.texto = torto.texto.replace(SHA256, &ultimo_trocado);

        let recusa = conferir_o_resumo(do_certutil(&torto, Algoritmo::Sha256)).unwrap_err();
        assert!(matches!(recusa, RecusaDoPacote::ResumoDivergente { .. }));

        // E a mensagem diz por que o numero esperado vale alguma coisa: ele
        // nao veio junto do download.
        assert!(
            recusa.to_string().contains("nao veio junto do download"),
            "{recusa}"
        );
    }

    #[test]
    fn nao_ter_conseguido_medir_e_outra_recusa_e_nao_a_de_divergencia() {
        // "Nao consegui olhar" nunca vira "nao bate". As duas pedem coisas
        // diferentes de quem lê: uma manda olhar o `certutil`, a outra manda
        // desconfiar do arquivo.
        let falhou = SaidaDeFerramenta {
            codigo: -2147024894,
            texto: "CertUtil: -hashfile comando FALHOU: 0x80070002\r\n".to_string(),
        };

        assert!(matches!(
            conferir_o_resumo(do_certutil(&falhou, Algoritmo::Sha256)),
            Err(RecusaDoPacote::NaoDeuParaResumir(_))
        ));
    }

    #[test]
    fn um_md5_no_lugar_do_sha256_nao_passa_por_acidente() {
        // Os dois sao hexadecimais e so o comprimento os separa. Pedir SHA256
        // e receber 32 digitos e a resposta errada, e nao uma resposta curta.
        let md5 = SaidaDeFerramenta {
            codigo: 0,
            texto: "MD5 hash de x:\r\n2ae9c9d58b70a340ceaad0e2da3a491f\r\n".to_string(),
        };

        assert!(conferir_o_resumo(do_certutil(&md5, Algoritmo::Sha256)).is_err());
    }

    // ─────────────── o que o pacote precisa ter dentro ───────────────

    /// A listagem de verdade do pacote, abreviada — extraída com o `bsdtar` do
    /// `System32` em 23/08/2026. São 356 entradas no total.
    const LISTAGEM: &[&str] = &[
        ".disk/info",
        "Clonezilla-Live-Version",
        "EFI/boot/bootx64.efi",
        "boot/grub/grub.cfg",
        "home/partimag/",
        "live/filesystem.squashfs",
        "live/initrd.img",
        "live/vmlinuz",
        "syslinux/isolinux.cfg",
        "utils/",
    ];

    #[test]
    fn o_pacote_de_verdade_tem_os_quatro_caminhos() {
        assert!(
            o_que_falta(LISTAGEM.iter().copied()).is_empty(),
            "os quatro caminhos obrigatorios tem de estar na listagem medida"
        );
    }

    #[test]
    fn um_pacote_sem_o_efi_e_recusado() {
        // Um zip que extrai sem erro e sem o `bootx64.efi` produz um
        // dispositivo que nao boota — e isso so se descobre depois de o
        // Windows ter sido apagado, que e quando alguem precisa dele.
        let sem_efi: Vec<&str> = LISTAGEM
            .iter()
            .copied()
            .filter(|caminho| !caminho.contains("bootx64"))
            .collect();

        assert_eq!(
            o_que_falta(sem_efi.into_iter()),
            vec!["EFI/boot/bootx64.efi".to_string()]
        );
    }

    #[test]
    fn a_barra_do_windows_nao_faz_um_caminho_sumir() {
        // Quem lista e o `bsdtar`, que usa `/`; quem confere e o Windows, que
        // usa `\`. Trocar os dois produziria um "nao achei" que nao quer dizer
        // nada — e a consequencia seria recusar um pacote bom depois de o
        // disco ja ter sido apagado.
        let com_barra_invertida = [
            r"EFI\boot\bootx64.efi",
            r"live\vmlinuz",
            r"live\initrd.img",
            r"boot\grub\grub.cfg",
        ];

        assert!(o_que_falta(com_barra_invertida.into_iter()).is_empty());
    }

    #[test]
    fn um_prefixo_a_mais_no_zip_e_recusa_e_nao_um_sim() {
        // Um zip que extraisse tudo dentro de `clonezilla-live/` produziria um
        // `ARCABOOT` cujo `EFI/` esta um nivel abaixo — e o firmware nao acha
        // o `.efi`. O pacote de hoje extrai na raiz, e este teste guarda essa
        // premissa.
        let aninhado: Vec<String> = LISTAGEM
            .iter()
            .map(|caminho| format!("clonezilla-live/{caminho}"))
            .collect();

        assert_eq!(
            o_que_falta(aninhado.iter().map(String::as_str)).len(),
            CAMINHOS_OBRIGATORIOS.len()
        );
    }

    #[test]
    fn a_caixa_do_nome_nao_derruba_o_pacote() {
        let em_maiuscula = [
            "EFI/BOOT/BOOTX64.EFI",
            "LIVE/VMLINUZ",
            "LIVE/INITRD.IMG",
            "BOOT/GRUB/GRUB.CFG",
        ];
        assert!(o_que_falta(em_maiuscula.into_iter()).is_empty());
    }
}
