//! A porta do firmware, implementada por `bcdedit`.
//!
//! O parser por valor e as regras de C-3, C-4 e C-6 moram em
//! [`crate::firmware`]. Aqui so mora a chamada e a decodificacao dos bytes:
//! nenhuma interpretacao do que o `bcdedit` respondeu.
//!
//! # A decodificacao e parte da fronteira, nao detalhe dela
//!
//! O `bcdedit` nao escreve UTF-8. Ele escreve na pagina de codigo do console
//! de quem o chamou — 850 na janela que o UAC abre nesta maquina —, e ler
//! esses bytes como UTF-8 troca cada acento por `U+FFFD` sem erro nenhum. Foi
//! medido: `examples/codificacao_do_bcdedit.rs`.
//!
//! Que os campos que o parser lê sejam todos ASCII e sorte, nao desenho: a
//! `description` de uma entrada de firmware e texto livre, e e ela que o
//! `arca status` imprime.

use crate::erro::{Erro, Resultado};
use crate::portas::Firmware;
use std::process::Command;

use super::texto::{de_pagina_de_codigo, pagina_do_console};

#[derive(Debug, Clone, Copy, Default)]
pub struct Bcdedit;

impl Bcdedit {
    fn rodar(argumentos: &[&str]) -> Resultado<String> {
        let saida = Command::new("bcdedit")
            .args(argumentos)
            .output()
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "bcdedit",
                origem,
            })?;

        // O `bcdedit` escreve tanto em stdout quanto em stderr conforme o
        // subcomando; quem parseia recebe os dois, sem julgamento aqui.
        let pagina = pagina_do_console();
        let mut texto = de_pagina_de_codigo(&saida.stdout, pagina);
        if !saida.stderr.is_empty() {
            texto.push_str(&de_pagina_de_codigo(&saida.stderr, pagina));
        }

        // Um `bcdedit` que recusou nao pode virar texto vazio. Sem privilegio
        // ele escreve "Acesso negado" **na saida padrao** e sai com codigo 1:
        // quem lesse so o texto concluiria que nao ha entrada `ARCA` onde na
        // verdade nao houve permissao para olhar — e criaria uma duplicata.
        //
        // Isto nao contradiz C-3. C-3 diz que o **sucesso** do `bcdedit` nao e
        // prova, e continua nao sendo: quem escreve confere depois com
        // `enumerar`. A recusa, essa, e informacao de verdade.
        if !saida.status.success() {
            return Err(Erro::FerramentaRecusou {
                ferramenta: "bcdedit",
                codigo: saida.status.code().unwrap_or(-1),
                saida: texto.trim().to_string(),
            });
        }

        Ok(texto)
    }
}

impl Firmware for Bcdedit {
    fn enumerar(&self, alvo: &str) -> Resultado<String> {
        Self::rodar(&["/enum", alvo])
    }

    fn executar(&self, argumentos: &[&str]) -> Resultado<String> {
        Self::rodar(argumentos)
    }
}
