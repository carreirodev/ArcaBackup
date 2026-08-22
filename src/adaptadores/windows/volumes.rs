//! A porta da enumeracao de discos, implementada sobre a API de volumes do
//! Windows.
//!
//! # Por que so metadado, e por caminho
//!
//! Tudo aqui e pergunta feita a uma **raiz de volume** — `E:\` — pelas
//! funcoes que o Windows oferece para isso. Nenhuma abre handle de
//! dispositivo, nenhuma fala com o driver do disco, nenhuma tem deslocamento
//! em setores. E o que S-1 exige, e e por isso que a distincao entre disco
//! externo e disco interno **nao sai daqui**: quem a faz e o
//! `IOCTL_STORAGE_QUERY_PROPERTY`, que precisaria do handle que S-1 proibe.
//!
//! Isso nao deixa C-6 descoberto. A palavra final sobre `Removable Media` e
//! do proprio `bcdedit`, lida pelo parser da etapa E2 — que e o unico lugar
//! onde ela vale, porque e o `bcdedit` quem rejeita.

use crate::erro::Resultado;
use crate::portas::{DiscoFisico, Discos, TipoDeMidia, Volume};

use super::texto::{de_utf16, para_utf16};

/// O maior nome de volume que o Windows devolve, mais o NUL.
const TAMANHO_DO_NOME: usize = 261;

#[derive(Debug, Clone, Copy, Default)]
pub struct VolumesDoWindows;

impl Discos for VolumesDoWindows {
    fn volumes(&self) -> Resultado<Vec<Volume>> {
        let _silencio = SemDialogoDeMidiaAusente::ligar();

        Ok(letras_montadas().filter_map(ler_volume).collect())
    }

    fn discos_fisicos(&self) -> Resultado<Vec<DiscoFisico>> {
        // Pelo WMI, e nao pela API de volumes deste modulo: a distincao entre
        // disco externo e interno, e o mapeamento de volume para disco fisico,
        // nao saem daqui — sairiam do `IOCTL_STORAGE_QUERY_PROPERTY`, que
        // precisa do handle que S-1 proibe. Ver
        // [`crate::adaptadores::windows::wmi`].
        super::wmi::ler(&super::wmi::consultar()?)
    }
}

/// As letras que o Windows tem montadas agora.
fn letras_montadas() -> impl Iterator<Item = char> {
    use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;

    let mascara = unsafe { GetLogicalDrives() };

    (0..26u32)
        .filter(move |posicao| mascara & (1 << posicao) != 0)
        .map(|posicao| (b'A' + posicao as u8) as char)
}

/// O que se sabe do volume nessa letra, ou nada quando ele nao responde.
///
/// Um volume que nao responde e um leitor sem midia dentro, e nao um erro: o
/// `arca list` nao pode falhar porque ha um leitor de cartao vazio na
/// maquina.
fn ler_volume(letra: char) -> Option<Volume> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW,
    };

    let raiz = para_utf16(&format!("{letra}:\\"));

    // SEGURANCA: `raiz` termina em NUL e vive ate o fim da funcao.
    let tipo = unsafe { GetDriveTypeW(raiz.as_ptr()) };

    let mut nome = [0u16; TAMANHO_DO_NOME];
    let mut sistema_de_arquivos = [0u16; TAMANHO_DO_NOME];

    // SEGURANCA: os dois buffers sao da pilha desta funcao e os tamanhos
    // informados sao os deles; os ponteiros nulos sao os parametros de saida
    // opcionais que nao interessam aqui.
    let leu = unsafe {
        GetVolumeInformationW(
            raiz.as_ptr(),
            nome.as_mut_ptr(),
            nome.len() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            sistema_de_arquivos.as_mut_ptr(),
            sistema_de_arquivos.len() as u32,
        )
    };
    if leu == 0 {
        return None;
    }

    let mut livre_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;

    // SEGURANCA: os ponteiros de saida apontam para variaveis desta pilha.
    let mediu = unsafe {
        GetDiskFreeSpaceExW(
            raiz.as_ptr(),
            &mut livre_bytes,
            &mut total_bytes,
            std::ptr::null_mut(),
        )
    };
    if mediu == 0 {
        return None;
    }

    let rotulo = de_utf16(&nome);

    Some(Volume {
        // Volume sem rotulo tem nome vazio, e vazio nao e rotulo.
        rotulo: (!rotulo.is_empty()).then_some(rotulo),
        letra: Some(letra),
        sistema_de_arquivos: de_utf16(&sistema_de_arquivos),
        total_bytes,
        livre_bytes,
        tipo_de_midia: tipo_de_midia(tipo),
    })
}

/// O que o `GetDriveType` sabe dizer.
///
/// Nunca devolve [`TipoDeMidia::DiscoExterno`]: essa e a classificacao do
/// `bcdedit`, e sai do parser da E2. Um SSD externo por USB aparece aqui como
/// [`TipoDeMidia::DiscoFixo`], que e o que o Windows de fato responde sobre
/// ele.
fn tipo_de_midia(tipo: u32) -> TipoDeMidia {
    use windows_sys::Win32::System::WindowsProgramming::{DRIVE_FIXED, DRIVE_REMOVABLE};

    match tipo {
        DRIVE_REMOVABLE => TipoDeMidia::Removivel,
        DRIVE_FIXED => TipoDeMidia::DiscoFixo,
        _ => TipoDeMidia::Desconhecido,
    }
}

/// Cala o dialogo "Insira um disco" enquanto a enumeracao corre.
///
/// Sem isto, perguntar pelo rotulo de um leitor de cartao vazio abre uma
/// janela modal — e o `arca list` roda elevado, em janela propria, onde essa
/// caixa ficaria esperando um clique que ninguem deu. O modo anterior volta
/// no `Drop`, para nao deixar o processo alterado depois da enumeracao.
struct SemDialogoDeMidiaAusente {
    anterior: u32,
}

impl SemDialogoDeMidiaAusente {
    fn ligar() -> SemDialogoDeMidiaAusente {
        use windows_sys::Win32::System::Diagnostics::Debug::{
            SEM_FAILCRITICALERRORS, SetThreadErrorMode,
        };

        let mut anterior = 0u32;
        // SEGURANCA: o ponteiro de saida aponta para uma variavel desta pilha.
        unsafe { SetThreadErrorMode(SEM_FAILCRITICALERRORS, &mut anterior) };
        SemDialogoDeMidiaAusente { anterior }
    }
}

impl Drop for SemDialogoDeMidiaAusente {
    fn drop(&mut self) {
        use windows_sys::Win32::System::Diagnostics::Debug::SetThreadErrorMode;

        unsafe { SetThreadErrorMode(self.anterior, std::ptr::null_mut()) };
    }
}

#[cfg(all(test, windows))]
mod testes {
    use super::*;

    #[test]
    fn as_letras_montadas_incluem_o_disco_do_sistema() {
        let letras: Vec<char> = letras_montadas().collect();
        assert!(letras.contains(&'C'), "veio {letras:?}");
    }

    #[test]
    fn a_enumeracao_devolve_o_volume_do_sistema_com_tamanho() {
        // Sem hardware ARCA nenhum: o que este teste cobra e que a chamada a
        // API do Windows esta certa, e para isso o `C:` basta.
        let volumes = VolumesDoWindows.volumes().unwrap();

        let sistema = volumes
            .iter()
            .find(|volume| volume.letra == Some('C'))
            .expect("o volume do sistema tem de aparecer");

        assert!(sistema.total_bytes > 0);
        assert!(sistema.livre_bytes <= sistema.total_bytes);
        assert!(
            !sistema.sistema_de_arquivos.is_empty(),
            "o sistema de arquivos veio vazio"
        );
    }

    #[test]
    fn nenhum_rotulo_vem_com_cauda_de_nul() {
        for volume in VolumesDoWindows.volumes().unwrap() {
            if let Some(rotulo) = &volume.rotulo {
                assert!(!rotulo.contains('\0'), "rotulo sujo: {rotulo:?}");
                assert_eq!(rotulo.trim_end(), rotulo, "rotulo com sobra: {rotulo:?}");
            }
        }
    }
}
