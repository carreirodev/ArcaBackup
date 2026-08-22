//! Conversao para as strings largas da API do Windows.

/// UTF-16 terminado em NUL, do jeito que a API do Windows espera.
pub fn para_utf16(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn termina_em_nul() {
        assert_eq!(para_utf16("ok"), vec![b'o' as u16, b'k' as u16, 0]);
    }

    #[test]
    fn acentos_atravessam() {
        let largo = para_utf16("restauracão");
        assert_eq!(largo.last(), Some(&0));
        let volta = String::from_utf16(&largo[..largo.len() - 1]).unwrap();
        assert_eq!(volta, "restauracão");
    }
}
