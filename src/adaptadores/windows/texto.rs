//! Conversao para as strings largas da API do Windows.

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
}
