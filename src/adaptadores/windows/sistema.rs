//! A porta das operacoes do sistema, implementada por registro e `chkdsk`.
//!
//! Duas fronteiras diferentes atras da mesma porta, e cada uma pelo caminho
//! que nao depende de idioma:
//!
//! - **B-5 lê o registro**, e nao o `powercfg`. Um `REG_DWORD` nao tem
//!   traducao; a saida do `powercfg /a` tem, e ela nem separa "desativada" de
//!   "indisponivel".
//! - **B-6 roda o `chkdsk` e olha o codigo de saida**, nunca o texto. Medido
//!   nesta maquina: `chkdsk C: /scan` elevado sai com **codigo 0** em 16,3 s,
//!   e o texto vem em **CP850** mesmo chamado de um console em UTF-8 — o mesmo
//!   caso do `bcdedit` da E2, e `de_pagina_de_codigo` resolve.

use crate::erro::{Erro, Resultado};
use crate::portas::{SaidaDeFerramenta, Sistema};
use std::process::Command;

use super::texto::{de_pagina_de_codigo, pagina_do_console, para_utf16};

/// Onde a Inicializacao Rapida mora, como numero.
const CHAVE_DA_ENERGIA: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Power";

/// O valor. Diferente de zero e Inicializacao Rapida ligada.
const VALOR_DA_INICIALIZACAO_RAPIDA: &str = "HiberbootEnabled";

#[derive(Debug, Clone, Copy, Default)]
pub struct SistemaDoWindows;

impl Sistema for SistemaDoWindows {
    fn inicializacao_rapida(&self) -> Resultado<Option<u32>> {
        ler_dword(CHAVE_DA_ENERGIA, VALOR_DA_INICIALIZACAO_RAPIDA)
    }

    fn conferir_volume(&self, letra: char) -> Resultado<SaidaDeFerramenta> {
        // `/scan`, e nunca `/f`: roda com o volume montado e nao escreve nada.
        let volume = format!("{letra}:");
        let saida = Command::new("chkdsk")
            .args([volume.as_str(), "/scan"])
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "chkdsk",
                origem,
            })?;

        let pagina = pagina_do_console();
        let mut texto = de_pagina_de_codigo(&saida.stdout, pagina);
        if !saida.stderr.is_empty() {
            texto.push_str(&de_pagina_de_codigo(&saida.stderr, pagina));
        }

        // Codigo diferente de zero **nao** vira erro aqui, ao contrario do que
        // o adaptador do `bcdedit` faz. E deliberado: o `chkdsk` usa o codigo
        // de saida para dizer o que achou no disco — 1 e "havia erro e foi
        // corrigido", 2 e "nao deu para conferir", 3 e "acesso negado". Todos
        // sao **resposta**, e quem os interpreta e o pre-voo, que tem teste.
        // Transforma-los em erro aqui faria o pre-voo inteiro parar por causa
        // de um disco que acusou alguma coisa — que e justamente o caso em que
        // B-6 quer falar com o usuario.
        Ok(SaidaDeFerramenta {
            codigo: saida.status.code().unwrap_or(-1),
            texto,
        })
    }
}

/// Um `REG_DWORD` de `HKEY_LOCAL_MACHINE`, ou `None` quando ele nao esta la.
///
/// `None` e ausencia de verdade. Quem lê decide o que fazer com "o registro
/// nao diz"; o que nao pode acontecer e isso virar "esta desativada".
fn ler_dword(subchave: &str, valor: &str) -> Resultado<Option<u32>> {
    use windows_sys::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows_sys::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RegGetValueW,
    };

    let subchave_larga = para_utf16(subchave);
    let valor_largo = para_utf16(valor);

    let mut dados: u32 = 0;
    let mut tamanho: u32 = std::mem::size_of::<u32>() as u32;

    // SEGURANCA: as duas cadeias terminam em NUL e vivem ate o fim da chamada;
    // o ponteiro de dados aponta para uma variavel desta pilha, e o tamanho
    // informado e o dela. `RRF_RT_REG_DWORD` faz a API recusar o valor se ele
    // nao for um DWORD, em vez de reinterpretar bytes.
    let estado = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            subchave_larga.as_ptr(),
            valor_largo.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&raw mut dados).cast(),
            &mut tamanho,
        )
    };

    if estado == ERROR_SUCCESS {
        return Ok(Some(dados));
    }
    if estado == ERROR_FILE_NOT_FOUND {
        // Nem a chave nem o valor existem. E resposta, e nao falha.
        return Ok(None);
    }

    Err(Erro::Ferramenta {
        ferramenta: "registro",
        origem: std::io::Error::from_raw_os_error(estado as i32),
    })
}

#[cfg(all(test, windows))]
mod testes {
    use super::*;

    #[test]
    fn a_inicializacao_rapida_desta_maquina_responde() {
        // Nao se cobra o **valor** — ele muda de maquina para maquina e o
        // usuario pode altera-lo. Cobra-se que a leitura funcione e devolva um
        // numero, que e o que separa esta implementacao de uma que interpreta
        // frase traduzida.
        let lida = SistemaDoWindows
            .inicializacao_rapida()
            .expect("o registro responde");

        assert!(
            lida.is_some(),
            "o valor `{VALOR_DA_INICIALIZACAO_RAPIDA}` nao esta no registro desta maquina"
        );
    }

    #[test]
    fn um_valor_que_nao_existe_e_none_e_nao_erro() {
        // A distincao que o `Option` carrega: "o registro nao diz" nao pode
        // virar erro nem, pior, virar zero.
        let ausente = ler_dword(CHAVE_DA_ENERGIA, "ArcaValorQueNaoExiste").expect("nao e erro");
        assert_eq!(ausente, None);
    }

    #[test]
    fn uma_chave_que_nao_existe_tambem_e_none() {
        let ausente = ler_dword(r"SOFTWARE\ArcaChaveQueNaoExiste", "x").expect("nao e erro");
        assert_eq!(ausente, None);
    }
}
