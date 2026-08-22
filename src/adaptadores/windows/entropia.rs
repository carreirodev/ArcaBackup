//! A porta da entropia, implementada por `BCryptGenRandom`.
//!
//! # Por que esta e nao uma dependencia
//!
//! O `Cargo.toml` do ARCA tem tres dependencias, e nenhuma delas sorteia
//! numero. O caminho de sempre seria acrescentar `rand`, que e uma linha —
//! e uma arvore. `BCryptGenRandom` ja esta no `windows-sys` que o projeto
//! usa desde a E0: bastou ligar a feature `Win32_Security_Cryptography`.
//! Nenhum crate novo, e a mesma familia de tudo que este diretorio faz.
//!
//! Ver `docs/adr/0006-o-selo-e-o-estado-sem-dependencia-nova.md`.
//!
//! # `BCRYPT_USE_SYSTEM_PREFERRED_RNG`, e nao um handle proprio
//!
//! A alternativa e abrir um algoritmo com `BCryptOpenAlgorithmProvider` e
//! fecha-lo depois. A flag dispensa as duas chamadas e o handle: o Windows usa
//! o gerador preferido do sistema. Menos codigo `unsafe`, e nada a vazar se
//! alguma coisa falhar no meio.

use crate::erro::{Erro, Resultado};
use crate::portas::Entropia;

#[derive(Debug, Clone, Copy, Default)]
pub struct EntropiaDoWindows;

impl Entropia for EntropiaDoWindows {
    fn preencher(&self, destino: &mut [u8]) -> Resultado<()> {
        use windows_sys::Win32::Security::Cryptography::{
            BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
        };

        // Pedir zero bytes nao e pedido: a API aceita e nao ha o que conferir.
        if destino.is_empty() {
            return Ok(());
        }

        // SEGURANCA: o ponteiro e o do proprio destino e o tamanho informado e
        // o dele. Com `BCRYPT_USE_SYSTEM_PREFERRED_RNG` o primeiro parametro e
        // ignorado, e por isso vai nulo.
        let estado = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                destino.as_mut_ptr(),
                destino.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };

        // O `NTSTATUS` e zero em exito e negativo em falha. Nao ha caminho de
        // sucesso parcial: ou preencheu tudo, ou nao escreveu nada.
        if estado != 0 {
            return Err(Erro::EntropiaIndisponivel { estado });
        }

        Ok(())
    }
}

#[cfg(all(test, windows))]
mod testes {
    use super::*;

    #[test]
    fn preenche_o_destino_inteiro() {
        // Um buffer grande o bastante para que "tudo zero" deixe de ser
        // coincidencia crivel: a chance de 256 bytes sairem zerados de um
        // gerador que funciona nao existe na pratica.
        let mut bytes = [0u8; 256];
        EntropiaDoWindows.preencher(&mut bytes).expect("o Windows responde");

        assert!(
            bytes.iter().any(|byte| *byte != 0),
            "o destino saiu zerado: o gerador nao escreveu"
        );
    }

    #[test]
    fn duas_chamadas_nao_devolvem_o_mesmo() {
        // E a unica propriedade de que o selo depende: nao repetir. Dois selos
        // iguais fariam dois jobs diferentes serem indistinguiveis, que e o
        // contrario do que C-11 quer.
        let mut primeiro = [0u8; 8];
        let mut segundo = [0u8; 8];

        EntropiaDoWindows.preencher(&mut primeiro).unwrap();
        EntropiaDoWindows.preencher(&mut segundo).unwrap();

        assert_ne!(primeiro, segundo);
    }

    #[test]
    fn pedir_nada_nao_e_erro() {
        assert!(EntropiaDoWindows.preencher(&mut []).is_ok());
    }
}
