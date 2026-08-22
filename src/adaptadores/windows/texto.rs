//! Conversao para as strings largas da API do Windows, e de volta do que as
//! ferramentas de console escrevem.

/// UTF-16 terminado em NUL, do jeito que a API do Windows espera.
pub fn para_utf16(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

/// O caminho de volta, cortando no primeiro NUL.
///
/// A API do Windows entrega buffers de tamanho fixo com o resto por
/// preencher; sem cortar no NUL, um rotulo viria com uma cauda de zeros
/// grudada e nenhuma comparacao com `ARCAVAULT` bateria.
pub fn de_utf16(largo: &[u16]) -> String {
    let fim = largo.iter().position(|&c| c == 0).unwrap_or(largo.len());
    String::from_utf16_lossy(&largo[..fim])
}

/// A pagina de codigo em que uma ferramenta de console escreve para o ARCA.
///
/// # Medido, e nao suposto
///
/// `examples/codificacao_do_bcdedit.rs` roda o `bcdedit` pelo mesmo caminho do
/// adaptador e olha os bytes crus. O resultado, nesta maquina:
///
/// | console de quem chama | o que o `bcdedit` escreveu |
/// |---|---|
/// | 850 (o padrao daqui, e o que o UAC da a uma janela nova) | CP850 — `U+FFFD` em todo acento se lido como UTF-8 |
/// | 65001 | UTF-8 |
///
/// A pagina **nao e fixa**: ela e a do console de quem chamou, e o filho a
/// herda junto do console. Fixar 850 quebraria numa sessao em UTF-8, e fixar
/// 65001 quebra na janela elevada — que e o caso normal do ARCA.
///
/// Sem console, `GetConsoleOutputCP` responde zero. Ai vale a pagina OEM da
/// maquina, que e o que a CRT usa quando nao ha console a consultar.
pub fn pagina_do_console() -> u32 {
    use windows_sys::Win32::Globalization::GetOEMCP;
    use windows_sys::Win32::System::Console::GetConsoleOutputCP;

    // SEGURANCA: nenhuma das duas recebe ponteiro.
    let console = unsafe { GetConsoleOutputCP() };
    if console != 0 {
        return console;
    }
    unsafe { GetOEMCP() }
}

/// O texto que uma ferramenta de console escreveu, decodificado pela pagina de
/// codigo em que ela escreveu.
///
/// Byte que a pagina nao conhece vira `U+FFFD`, como no
/// [`String::from_utf8_lossy`] — perder um caractere e melhor do que perder o
/// arquivo inteiro. A diferenca esta em qual byte se perde: com a pagina certa,
/// nenhum.
pub fn de_pagina_de_codigo(bytes: &[u8], pagina: u32) -> String {
    use windows_sys::Win32::Globalization::MultiByteToWideChar;

    // A API recusa comprimento zero, e vazio nao precisa de conversao.
    if bytes.is_empty() {
        return String::new();
    }

    // SEGURANCA: o comprimento informado e o da propria fatia. Com ponteiro de
    // saida nulo e tamanho zero, a funcao so responde de quanto precisaria.
    let largura = unsafe {
        MultiByteToWideChar(
            pagina,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            std::ptr::null_mut(),
            0,
        )
    };
    if largura <= 0 {
        // A pagina de codigo nao existe nesta maquina. Nao ha o que fazer
        // alem de ler como UTF-8 e marcar o que nao couber — que e o
        // comportamento que este modulo veio corrigir, agora restrito a um
        // caso que nao deveria acontecer.
        return String::from_utf8_lossy(bytes).into_owned();
    }

    let mut largo = vec![0u16; largura as usize];
    // SEGURANCA: o destino e o vetor recem-dimensionado pela chamada acima.
    let escritos = unsafe {
        MultiByteToWideChar(
            pagina,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            largo.as_mut_ptr(),
            largura,
        )
    };

    String::from_utf16_lossy(&largo[..escritos.max(0) as usize])
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn termina_em_nul() {
        assert_eq!(para_utf16("ok"), vec![b'o' as u16, b'k' as u16, 0]);
    }

    #[test]
    fn a_volta_corta_no_nul_e_descarta_o_resto_do_buffer() {
        // E assim que a API do Windows devolve um rotulo: buffer grande, nome
        // curto, o resto por preencher.
        let mut buffer = para_utf16("ARCAVAULT");
        buffer.resize(261, 0);

        assert_eq!(de_utf16(&buffer), "ARCAVAULT");
        assert_eq!(de_utf16(&[0u16; 261]), "");
    }

    #[test]
    fn acentos_atravessam() {
        let largo = para_utf16("restauracão");
        assert_eq!(largo.last(), Some(&0));
        let volta = String::from_utf16(&largo[..largo.len() - 1]).unwrap();
        assert_eq!(volta, "restauracão");
    }

    /// O cabecalho que o `bcdedit` desta maquina escreveu, em CP850. Os dois
    /// bytes que nao sao ASCII sao o `ç` (0x87) e o `ã` (0xC6).
    const CABECALHO_EM_850: &[u8] = &[
        b'I', b'n', b'i', b'c', b'i', b'a', b'l', b'i', b'z', b'a', 0x87, 0xC6, b'o',
    ];

    #[test]
    fn a_pagina_certa_devolve_o_acento_e_a_errada_o_perde() {
        assert_eq!(de_pagina_de_codigo(CABECALHO_EM_850, 850), "Inicialização");

        // O que o adaptador fazia antes: ler bytes de CP850 como se fossem
        // UTF-8. Nao e um caractere feio no lugar do certo — e a informacao
        // perdida, sem erro nenhum sendo levantado.
        let como_utf8 = String::from_utf8_lossy(CABECALHO_EM_850);
        assert!(como_utf8.contains('\u{FFFD}'), "veio {como_utf8:?}");
    }

    #[test]
    fn texto_ascii_atravessa_qualquer_pagina_intacto() {
        // Os campos que o parser lê sao todos ASCII, e e por isso que a
        // codificacao errada nao chegou a quebrar o ARCA: ela estraga o
        // cabecalho traduzido e passa longe de `identificador`, `device` e
        // `path`. Continua sendo errado — o `status` imprime a descricao, e
        // uma descricao acentuada sairia suja.
        let identificador = b"identificador           {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}";
        for pagina in [850u32, 65001, 1252] {
            assert_eq!(
                de_pagina_de_codigo(identificador, pagina),
                String::from_utf8_lossy(identificador),
                "pagina {pagina}"
            );
        }
    }

    #[test]
    fn utf8_e_decodificado_quando_a_pagina_e_65001() {
        // O outro lado da medicao: num console em UTF-8 o `bcdedit` escreve
        // UTF-8, e a mesma funcao tem de dar conta.
        assert_eq!(
            de_pagina_de_codigo("Inicialização".as_bytes(), 65001),
            "Inicialização"
        );
    }

    #[test]
    fn vazio_nao_e_erro() {
        // A API do Windows recusa comprimento zero, e uma ferramenta que nao
        // imprimiu nada e um caso normal.
        assert_eq!(de_pagina_de_codigo(&[], 850), "");
    }

    #[test]
    fn a_pagina_do_console_e_uma_pagina_de_verdade() {
        let pagina = pagina_do_console();
        assert!(pagina > 0, "nenhuma pagina de codigo respondeu");
        assert_eq!(de_pagina_de_codigo(b"ARCA", pagina), "ARCA");
    }
}
