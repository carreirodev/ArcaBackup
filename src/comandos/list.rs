//! `arca list` — as imagens do dispositivo conectado (§5.4, L-1, L-2, D6).
//!
//! Lê e nada mais. Nao escreve no dispositivo, nem log: na etapa E1 o
//! `ARCAVAULT` e so lido. O registro local em `%LOCALAPPDATA%` continua
//! valendo, porque ele nao mora no dispositivo.

use crate::app::Contexto;
use crate::dispositivo;
use crate::erro::Resultado;
use crate::formato;
use crate::imagens::{self, Especie, Pasta, Veredito};

/// Espacos entre a coluna do nome e a da data, como no §5.4.
const SEPARACAO: usize = 3;

pub fn executar(contexto: &Contexto) -> Resultado<()> {
    let dispositivo = dispositivo::encontrar(contexto.discos)?;
    let raiz = dispositivo.raiz_do_vault()?;

    let pastas = imagens::enumerar(contexto.arquivos, &raiz)?;

    let imagens = pastas.iter().filter(|pasta| pasta.e_imagem()).count();
    contexto.registro.info(format!(
        "list em {} · {imagens} imagem(ns) · {} residuo(s)",
        raiz.display(),
        pastas.len() - imagens
    ));

    print!("{}", montar(&pastas, dispositivo.vault.livre_bytes));
    Ok(())
}

/// A saida do §5.4: uma linha por pasta e o espaco livre no fim.
pub fn montar(pastas: &[Pasta], livre_bytes: u64) -> String {
    let mut saida = String::new();

    if pastas.is_empty() {
        saida.push_str(&format!("Nenhuma imagem em {}.\n", dispositivo::ARCAVAULT));
    } else {
        saida.push_str(&format!("Imagens em {}:\n", dispositivo::ARCAVAULT));

        let coluna = pastas
            .iter()
            .map(|pasta| pasta.nome.chars().count())
            .max()
            .unwrap_or(0)
            + SEPARACAO;

        for pasta in pastas {
            // O preenchimento e contado a mao: `{:<n$}` conta bytes, e um
            // nome com acento sairia desalinhado.
            let recuo = " ".repeat(coluna - pasta.nome.chars().count());
            saida.push_str(&format!(
                "  {}{recuo}{} · {} · {}\n",
                pasta.nome,
                formato::dia_e_mes(pasta.modificado_em),
                formato::tamanho(pasta.tamanho_bytes),
                parecer(&pasta.especie),
            ));
        }
    }

    saida.push_str(&format!("\n{} livres\n", formato::gigabytes(livre_bytes)));
    saida
}

/// A ultima coluna: o veredito de uma imagem, ou a palavra que diz que aquilo
/// nao e uma.
fn parecer(especie: &Especie) -> &'static str {
    match especie {
        Especie::Imagem {
            veredito: Some(Veredito::Aprovada),
        } => "aprovada",
        Especie::Imagem {
            veredito: Some(Veredito::Reprovada),
        } => "reprovada",
        // Imagem nao verificada e suposicao: dizer isso e melhor do que
        // deixar a coluna vazia.
        Especie::Imagem { veredito: None } => "sem veredito",
        Especie::Residuo => "residuo",
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::momento;

    fn imagem(nome: &str, dia: &str, tamanho_bytes: u64, veredito: Option<Veredito>) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes,
            modificado_em: Some(momento(dia)),
            especie: Especie::Imagem { veredito },
        }
    }

    fn residuo(nome: &str, dia: &str, tamanho_bytes: u64) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes,
            modificado_em: Some(momento(dia)),
            especie: Especie::Residuo,
        }
    }

    #[test]
    fn a_saida_e_a_do_paragrafo_5_4_do_prd() {
        let pastas = vec![
            imagem(
                "2026-08-21_WindowsCompleto",
                "2026-08-21T12:56:31",
                38_823_623_035,
                Some(Veredito::Aprovada),
            ),
            imagem(
                "2026-08-22_Apps",
                "2026-08-22T09:14:02",
                38_823_623_035,
                Some(Veredito::Aprovada),
            ),
        ];

        assert_eq!(
            montar(&pastas, 196_400_000_000),
            "Imagens em ARCAVAULT:\n\
             \x20 2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada\n\
             \x20 2026-08-22_Apps              22/08 · 36,2 GB · aprovada\n\
             \n\
             183 GB livres\n"
        );
    }

    #[test]
    fn residuo_aparece_marcado_como_residuo() {
        // L-2: pasta sem MD5SUMS aparece como residuo, nunca como imagem.
        let pastas = vec![residuo(
            "2026-08-22_Interrompido",
            "2026-08-22T03:11:00",
            512,
        )];

        assert!(montar(&pastas, 0).contains("2026-08-22_Interrompido   22/08 · 512 B · residuo"));
    }

    #[test]
    fn imagem_sem_check_log_diz_que_nao_ha_veredito() {
        let pastas = vec![imagem("2026-08-22_Apps", "2026-08-22T09:14:02", 1024, None)];
        assert!(montar(&pastas, 0).contains("· sem veredito"));
    }

    #[test]
    fn imagem_reprovada_aparece_reprovada() {
        let pastas = vec![imagem(
            "2026-08-22_Apps",
            "2026-08-22T09:14:02",
            1024,
            Some(Veredito::Reprovada),
        )];
        assert!(montar(&pastas, 0).contains("· reprovada"));
    }

    #[test]
    fn vault_vazio_diz_que_esta_vazio_e_ainda_mostra_o_espaco() {
        // Nenhuma linha e silencio (§5.5).
        assert_eq!(
            montar(&[], 196_400_000_000),
            "Nenhuma imagem em ARCAVAULT.\n\n183 GB livres\n"
        );
    }

    #[test]
    fn a_coluna_acompanha_o_nome_mais_longo() {
        let pastas = vec![
            imagem("curto", "2026-08-22T09:14:02", 1024, None),
            imagem(
                "um_nome_bem_mais_longo_que_o_outro",
                "2026-08-22T09:14:02",
                1024,
                None,
            ),
        ];

        let saida = montar(&pastas, 0);
        let colunas: Vec<usize> = saida
            .lines()
            .filter(|linha| linha.contains("22/08"))
            .map(|linha| linha.find("22/08").unwrap())
            .collect();

        assert_eq!(colunas.len(), 2);
        assert_eq!(colunas[0], colunas[1], "as datas tem de ficar alinhadas");
    }
}
