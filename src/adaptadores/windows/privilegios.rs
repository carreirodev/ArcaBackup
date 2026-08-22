//! Elevacao por UAC, repassando os argumentos intactos (C-7).
//!
//! O manifesto embutido (ver `build.rs`) faz o Windows elevar antes de o
//! programa comecar, e nesse caminho quem repassa a linha de comando e o
//! proprio sistema. Este adaptador cobre o caso em que o manifesto nao
//! vigora: relanca com o verbo `runas`, espera e propaga o codigo de saida.
//!
//! Os argumentos que chegam aqui sao os **originais**, colhidos de
//! `std::env::args`. Reconstrui-los a partir do que o `clap` entendeu seria
//! recriar a armadilha que apagou o `--dry-run`.

use crate::adaptadores::windows::linha_de_comando;
use crate::adaptadores::windows::texto::para_utf16;
use crate::erro::{Erro, Resultado};
use crate::portas::Privilegios;

use std::io;
use windows_sys::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE};
use windows_sys::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
};
use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetExitCodeProcess, INFINITE, OpenProcessToken, WaitForSingleObject,
};
use windows_sys::Win32::UI::Shell::{
    SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

#[derive(Debug, Clone, Copy, Default)]
pub struct PrivilegiosDoWindows;

impl Privilegios for PrivilegiosDoWindows {
    fn elevado(&self) -> Resultado<bool> {
        token_elevado()
            .ok_or_else(|| Erro::ElevacaoIndeterminada(io::Error::last_os_error().to_string()))
    }

    fn relancar_elevado(&self, argumentos: &[String]) -> Resultado<i32> {
        let executavel = std::env::current_exe().map_err(Erro::ExecutavelDesconhecido)?;

        let verbo = para_utf16("runas");
        let arquivo = para_utf16(&executavel.to_string_lossy());
        let parametros = para_utf16(&linha_de_comando::montar_parametros(argumentos));

        // O processo elevado e criado pelo servico AppInfo, e nao por este
        // processo: sem isto ele nasce em `%SystemRoot%\System32` e todo
        // caminho relativo que o usuario digitou — `--iso clonezilla.zip` —
        // passa a apontar para o lugar errado. Os argumentos chegariam
        // intactos e ainda assim nao resolveriam.
        let diretorio = std::env::current_dir()
            .ok()
            .map(|atual| para_utf16(&atual.to_string_lossy()));

        // O `ShellExecuteExW` quer COM inicializado. Um erro aqui costuma ser
        // "ja inicializado noutro modelo", que nao impede a chamada.
        // SEGURANCA: chamada sem ponteiros nossos.
        unsafe {
            let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
        }

        let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
        info.lpVerb = verbo.as_ptr();
        info.lpFile = arquivo.as_ptr();
        info.lpParameters = parametros.as_ptr();
        info.lpDirectory = diretorio
            .as_ref()
            .map_or(std::ptr::null(), |largo| largo.as_ptr());
        info.nShow = SW_SHOWNORMAL;

        // SEGURANCA: os quatro vetores UTF-16 vivem ate o fim desta funcao, que
        // e depois de `WaitForSingleObject` — o `ShellExecuteExW` so lê os
        // ponteiros durante a chamada, e o processo filho ja tem sua copia.
        let disparou = unsafe { ShellExecuteExW(&mut info) };

        if disparou == 0 {
            let falha = io::Error::last_os_error();
            return match falha.raw_os_error() {
                Some(codigo) if codigo == ERROR_CANCELLED as i32 => Err(Erro::ElevacaoRecusada),
                _ => Err(Erro::FalhaAoElevar(falha.to_string())),
            };
        }

        let processo = info.hProcess;
        if processo.is_null() {
            return Err(Erro::FalhaAoElevar(
                "o Windows nao devolveu handle do processo elevado".to_string(),
            ));
        }

        // SEGURANCA: `processo` veio do proprio `ShellExecuteExW` e so e
        // fechado ao final desta funcao.
        let codigo = unsafe {
            WaitForSingleObject(processo, INFINITE);
            let mut codigo: u32 = 0;
            let leu = GetExitCodeProcess(processo, &mut codigo);
            CloseHandle(processo);
            if leu == 0 { None } else { Some(codigo) }
        };

        match codigo {
            Some(codigo) => Ok(codigo as i32),
            None => Err(Erro::FalhaAoElevar(
                "o processo elevado terminou sem codigo de saida legivel".to_string(),
            )),
        }
    }
}

/// Consulta `TokenElevation` no token do processo. `None` quando a consulta
/// em si falha, para que quem chama nao confunda "nao elevado" com "nao sei".
fn token_elevado() -> Option<bool> {
    let mut token: HANDLE = std::ptr::null_mut();

    // SEGURANCA: `token` e uma variavel da pilha; o handle e fechado abaixo.
    let abriu = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
    if abriu == 0 {
        return None;
    }

    let mut elevacao = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut devolvido: u32 = 0;

    // SEGURANCA: o tamanho passado e o da propria struct.
    let consultou = unsafe {
        let consultou = GetTokenInformation(
            token,
            TokenElevation,
            (&raw mut elevacao).cast(),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut devolvido,
        );
        CloseHandle(token);
        consultou
    };

    if consultou == 0 {
        None
    } else {
        Some(elevacao.TokenIsElevated != 0)
    }
}
