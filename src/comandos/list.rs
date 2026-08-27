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

/// O recuo da linha da descricao (L-3), sob a linha da imagem.
///
/// Fixo, e nao alinhado com a coluna do nome: a coluna acompanha o nome mais
/// longo do dispositivo, e a descricao comecaria num lugar diferente a cada
/// listagem — inclusive na mesma imagem, depois de a vizinha ser renomeada.
const RECUO_DA_DESCRICAO: &str = "      ";

/// Onde a linha da descricao quebra, recuo incluido.
///
/// A descricao chega de [`crate::imagens::interpretar_descricao`] como **uma
/// frase so**, de ate 300 caracteres — quem decide onde ela quebra e esta
/// tela, e nunca o arquivo que alguem digitou num bloco de notas. Sobram 70
/// caracteres por linha depois do recuo, e uma descricao no limite ocupa
/// cinco.
pub const LARGURA: usize = 76;

/// Se a listagem mostra a descricao de cada pasta (L-3).
///
/// Um enum, e nao um `bool`, porque esta funcao tem quatro chamadores e tres
/// deles dizem "nao": o `arca resultado` e o `arca status` reusam a listagem
/// em vez de formatar as imagens de novo, e a descricao nao e diagnostico —
/// e do `arca list`. Num `montar(&pastas, livre, false)` nao ha como ver o
/// que e falso.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Descricoes {
    Mostrar,
    Omitir,
}

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

    print!(
        "{}",
        montar(&pastas, dispositivo.vault.livre_bytes, Descricoes::Mostrar)
    );
    Ok(())
}

/// A saida do §5.4: uma linha por pasta, a descricao de quem tiver uma, e o
/// espaco livre no fim.
pub fn montar(pastas: &[Pasta], livre_bytes: u64, descricoes: Descricoes) -> String {
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

            if descricoes == Descricoes::Mostrar {
                if let Some(descricao) = &pasta.descricao {
                    for linha in quebrar(descricao) {
                        saida.push_str(&format!("{RECUO_DA_DESCRICAO}{linha}\n"));
                    }
                }
            }
        }
    }

    saida.push_str(&format!("\n{} livres\n", formato::gigabytes(livre_bytes)));
    saida
}

/// A frase da descricao repartida em linhas que cabem em [`LARGURA`].
///
/// Conta caracteres e nao bytes, pela mesma razao que a coluna do nome ja
/// contava: uma descricao em portugues tem mais bytes do que caracteres, e
/// medir por byte quebraria a linha cedo demais — em cima de um acento.
///
/// Uma palavra maior que a largura fica sozinha e passa do limite, de
/// proposito: parti-la esconderia o que ela e, e o unico jeito de aparecer
/// uma dessas e alguem ter digitado um caminho ou uma URL — justamente o que
/// se quer poder copiar inteiro.
///
/// `split_whitespace` e nao `split(' ')`: hoje a frase chega de
/// [`crate::imagens::interpretar_descricao`] com espacos ja colapsados, mas
/// essa invariante mora em outro modulo e nada aqui a cobra. Um espaco duplo
/// que chegasse por outro caminho sairia como espaco solto no fim de uma
/// linha, e a diferenca entre os dois primitivos e nenhuma para a frase
/// normalizada.
fn quebrar(frase: &str) -> Vec<String> {
    let cabe = LARGURA - RECUO_DA_DESCRICAO.len();
    let mut linhas = Vec::new();
    let mut atual = String::new();

    for palavra in frase.split_whitespace() {
        let com_a_palavra = atual.chars().count() + 1 + palavra.chars().count();
        if !atual.is_empty() && com_a_palavra > cabe {
            linhas.push(std::mem::take(&mut atual));
        }
        if !atual.is_empty() {
            atual.push(' ');
        }
        atual.push_str(palavra);
    }

    if !atual.is_empty() {
        linhas.push(atual);
    }
    linhas
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
            descricao: None,
        }
    }

    fn residuo(nome: &str, dia: &str, tamanho_bytes: u64) -> Pasta {
        Pasta {
            nome: nome.to_string(),
            tamanho_bytes,
            modificado_em: Some(momento(dia)),
            especie: Especie::Residuo,
            descricao: None,
        }
    }

    fn com_descricao(pasta: Pasta, descricao: &str) -> Pasta {
        Pasta {
            descricao: Some(descricao.to_string()),
            ..pasta
        }
    }

    fn apps_descrita() -> Vec<Pasta> {
        vec![com_descricao(
            imagem(
                "2026-08-22_Apps",
                "2026-08-22T09:14:02",
                38_823_623_035,
                Some(Veredito::Aprovada),
            ),
            "Depois do Office e do Visual Studio.",
        )]
    }

    #[test]
    fn a_descricao_aparece_sob_a_imagem_no_arca_list() {
        assert_eq!(
            montar(&apps_descrita(), 196_400_000_000, Descricoes::Mostrar),
            "Imagens em ARCAVAULT:\n\
             \x20 2026-08-22_Apps   22/08 · 36,2 GB · aprovada\n\
             \x20     Depois do Office e do Visual Studio.\n\
             \n\
             183 GB livres\n"
        );
    }

    #[test]
    fn as_telas_que_reusam_a_listagem_nao_ganham_a_descricao() {
        // `arca resultado` e `arca status` reusam esta funcao em vez de
        // formatar as imagens de novo, e as duas sao telas de diagnostico que
        // este projeto vem encurtando. A descricao e do `arca list`, e o
        // parametro existe para que isso seja escolha e nao efeito colateral.
        assert_eq!(
            montar(&apps_descrita(), 196_400_000_000, Descricoes::Omitir),
            "Imagens em ARCAVAULT:\n\
             \x20 2026-08-22_Apps   22/08 · 36,2 GB · aprovada\n\
             \n\
             183 GB livres\n"
        );
    }

    #[test]
    fn imagem_sem_descricao_continua_uma_linha_so() {
        // O caso de toda imagem gravada antes de 27/08/2026: sem o arquivo, a
        // listagem e byte a byte a que sempre foi.
        let pastas = vec![imagem(
            "2026-08-22_Apps",
            "2026-08-22T09:14:02",
            38_823_623_035,
            Some(Veredito::Aprovada),
        )];

        assert_eq!(
            montar(&pastas, 196_400_000_000, Descricoes::Mostrar),
            montar(&pastas, 196_400_000_000, Descricoes::Omitir)
        );
    }

    #[test]
    fn a_descricao_longa_quebra_em_linhas_recuadas() {
        // Ela vem de `interpretar_descricao` como **uma frase so**, de ate 300
        // caracteres. Quem decide onde a linha quebra e esta tela, e nao o
        // arquivo que alguem digitou.
        let pastas = vec![com_descricao(
            imagem("2026-08-22_Apps", "2026-08-22T09:14:02", 1024, None),
            "palavra ".repeat(20).trim(),
        )];

        let saida = montar(&pastas, 0, Descricoes::Mostrar);
        let linhas: Vec<&str> = saida
            .lines()
            .filter(|linha| linha.contains("palavra"))
            .collect();

        assert!(linhas.len() > 1, "a frase tinha de quebrar: {linhas:?}");
        assert!(
            linhas
                .iter()
                .all(|linha| linha.starts_with(RECUO_DA_DESCRICAO)),
            "toda linha da descricao e recuada: {linhas:?}"
        );
        assert!(
            linhas.iter().all(|linha| linha.chars().count() <= LARGURA),
            "nenhuma passa da largura: {linhas:?}"
        );
    }

    #[test]
    fn uma_palavra_maior_que_a_largura_nao_some_nem_trava() {
        let pastas = vec![com_descricao(
            imagem("2026-08-22_Apps", "2026-08-22T09:14:02", 1024, None),
            &"x".repeat(LARGURA * 2),
        )];

        let saida = montar(&pastas, 0, Descricoes::Mostrar);
        assert!(saida.contains(&"x".repeat(LARGURA * 2)));
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
            montar(&pastas, 196_400_000_000, Descricoes::Mostrar),
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

        assert!(
            montar(&pastas, 0, Descricoes::Mostrar)
                .contains("2026-08-22_Interrompido   22/08 · 512 B · residuo")
        );
    }

    #[test]
    fn imagem_sem_check_log_diz_que_nao_ha_veredito() {
        let pastas = vec![imagem("2026-08-22_Apps", "2026-08-22T09:14:02", 1024, None)];
        assert!(montar(&pastas, 0, Descricoes::Mostrar).contains("· sem veredito"));
    }

    #[test]
    fn imagem_reprovada_aparece_reprovada() {
        let pastas = vec![imagem(
            "2026-08-22_Apps",
            "2026-08-22T09:14:02",
            1024,
            Some(Veredito::Reprovada),
        )];
        assert!(montar(&pastas, 0, Descricoes::Mostrar).contains("· reprovada"));
    }

    #[test]
    fn vault_vazio_diz_que_esta_vazio_e_ainda_mostra_o_espaco() {
        // Nenhuma linha e silencio (§5.5).
        assert_eq!(
            montar(&[], 196_400_000_000, Descricoes::Mostrar),
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

        let saida = montar(&pastas, 0, Descricoes::Mostrar);
        let colunas: Vec<usize> = saida
            .lines()
            .filter(|linha| linha.contains("22/08"))
            .map(|linha| linha.find("22/08").unwrap())
            .collect();

        assert_eq!(colunas.len(), 2);
        assert_eq!(colunas[0], colunas[1], "as datas tem de ficar alinhadas");
    }
}
