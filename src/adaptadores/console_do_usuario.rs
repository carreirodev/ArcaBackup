//! A porta do console, implementada pelo `stdin` do processo.

use crate::erro::{Erro, Resultado};
use crate::portas::Console;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, Default)]
pub struct ConsoleDoUsuario;

impl Console for ConsoleDoUsuario {
    fn ler_linha(&self) -> Resultado<String> {
        let mut linha = String::new();

        // Zero bytes lidos e fim de entrada — `stdin` fechado, ou um `< NUL`.
        // Devolve linha vazia pelo motivo que a porta explica: nao digitar
        // nada nunca confirma nada, e um erro aqui so daria um segundo caminho
        // para a mesma recusa.
        std::io::stdin()
            .lock()
            .read_line(&mut linha)
            .map_err(|origem| Erro::Ferramenta {
                ferramenta: "console",
                origem,
            })?;

        // O `read_line` traz o `\r\n` do Enter junto. Podar aqui, e nao em
        // quem julga, deixa o julgamento comparando texto com texto.
        Ok(linha.trim_end_matches(['\r', '\n']).to_string())
    }
}
