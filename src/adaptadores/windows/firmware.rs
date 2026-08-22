//! A porta do firmware, implementada por `bcdedit`.
//!
//! A etapa E2 constroi o parser por valor e as regras de C-3, C-4 e C-6. Aqui
//! so mora a chamada: nenhuma interpretacao do que o `bcdedit` respondeu.

use crate::erro::{Erro, Resultado};
use crate::portas::Firmware;
use std::process::Command;

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
        let mut texto = String::from_utf8_lossy(&saida.stdout).into_owned();
        if !saida.stderr.is_empty() {
            texto.push_str(&String::from_utf8_lossy(&saida.stderr));
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
