//! O registro local do lado Windows.
//!
//! Fica em `%LOCALAPPDATA%\ARCA\arca.log` e e **descartavel**: quem julga uma
//! operacao e o `estado.json` no `ARCABOOT` (secao 4.1), que sobrevive a uma
//! restauracao. Este arquivo existe para reconstituir o que o ARCA fez do
//! lado de ca — sobretudo a linha de comando que ele recebeu, que e o que
//! denuncia um argumento perdido na elevacao (C-7).
//!
//! Nenhuma falha de escrita aqui interrompe uma operacao. Perder o registro e
//! ruim; parar um backup por causa dele seria pior.

use crate::portas::Relogio;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Acima disto o arquivo corrente vira `arca.log.anterior`. Duas geracoes
/// bastam: o registro serve ao ultimo problema, nao ao historico.
const LIMITE_DE_ROTACAO_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nivel {
    Info,
    Aviso,
    Erro,
}

impl Nivel {
    fn etiqueta(self) -> &'static str {
        match self {
            Nivel::Info => "INFO ",
            Nivel::Aviso => "AVISO",
            Nivel::Erro => "ERRO ",
        }
    }
}

pub struct Registro {
    caminho: PathBuf,
    relogio: Box<dyn Relogio>,
}

impl Registro {
    /// Abre o registro no lugar de sempre. Sem `%LOCALAPPDATA%`, cai no
    /// diretorio temporario — o ARCA nao fica sem registro por causa de um
    /// perfil de usuario incomum.
    pub fn padrao(relogio: Box<dyn Relogio>) -> Registro {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        Registro::em(base.join("ARCA"), relogio)
    }

    pub fn em(diretorio: impl Into<PathBuf>, relogio: Box<dyn Relogio>) -> Registro {
        let diretorio = diretorio.into();
        let _ = fs::create_dir_all(&diretorio);
        Registro {
            caminho: diretorio.join("arca.log"),
            relogio,
        }
    }

    pub fn caminho(&self) -> &Path {
        &self.caminho
    }

    pub fn info(&self, mensagem: impl AsRef<str>) {
        self.anotar(Nivel::Info, mensagem.as_ref());
    }

    pub fn aviso(&self, mensagem: impl AsRef<str>) {
        self.anotar(Nivel::Aviso, mensagem.as_ref());
    }

    pub fn erro(&self, mensagem: impl AsRef<str>) {
        self.anotar(Nivel::Erro, mensagem.as_ref());
    }

    pub fn anotar(&self, nivel: Nivel, mensagem: &str) {
        self.rotacionar_se_grande();

        let momento = self.relogio.agora().format("%Y-%m-%d %H:%M:%S%.3f");
        let linha = format!(
            "{momento} {} [{}] {mensagem}\n",
            nivel.etiqueta(),
            std::process::id()
        );

        // Uma anotacao perdida nao para o ARCA.
        if let Ok(mut arquivo) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.caminho)
        {
            let _ = arquivo.write_all(linha.as_bytes());
        }
    }

    fn rotacionar_se_grande(&self) {
        let Ok(metadados) = fs::metadata(&self.caminho) else {
            return;
        };
        if metadados.len() < LIMITE_DE_ROTACAO_BYTES {
            return;
        }
        let anterior = self.caminho.with_extension("log.anterior");
        let _ = fs::rename(&self.caminho, anterior);
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::RelogioParado;

    fn diretorio_temporario(nome: &str) -> PathBuf {
        let caminho =
            std::env::temp_dir().join(format!("arca-testes-{nome}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&caminho);
        caminho
    }

    #[test]
    fn anota_com_momento_nivel_e_mensagem() {
        let diretorio = diretorio_temporario("registro");
        let registro = Registro::em(
            &diretorio,
            Box::new(RelogioParado::em("2026-08-22T11:42:03")),
        );

        registro.info("arca 0.1.0 iniciado");

        let conteudo = fs::read_to_string(registro.caminho()).unwrap();
        assert!(conteudo.contains("2026-08-22 11:42:03"), "{conteudo}");
        assert!(conteudo.contains("INFO"), "{conteudo}");
        assert!(conteudo.contains("arca 0.1.0 iniciado"), "{conteudo}");

        let _ = fs::remove_dir_all(&diretorio);
    }

    #[test]
    fn anotacoes_se_acumulam_entre_processos() {
        let diretorio = diretorio_temporario("acumulo");

        for mensagem in ["primeira", "segunda"] {
            let registro = Registro::em(
                &diretorio,
                Box::new(RelogioParado::em("2026-08-22T11:42:03")),
            );
            registro.info(mensagem);
        }

        let conteudo = fs::read_to_string(diretorio.join("arca.log")).unwrap();
        assert!(
            conteudo.contains("primeira") && conteudo.contains("segunda"),
            "{conteudo}"
        );
        assert_eq!(conteudo.lines().count(), 2, "{conteudo}");

        let _ = fs::remove_dir_all(&diretorio);
    }

    #[test]
    fn rotaciona_ao_passar_do_limite() {
        let diretorio = diretorio_temporario("rotacao");
        let registro = Registro::em(
            &diretorio,
            Box::new(RelogioParado::em("2026-08-22T11:42:03")),
        );

        fs::write(
            registro.caminho(),
            "x".repeat(LIMITE_DE_ROTACAO_BYTES as usize + 1),
        )
        .unwrap();
        registro.info("depois da rotacao");

        let corrente = fs::read_to_string(registro.caminho()).unwrap();
        assert!(corrente.contains("depois da rotacao"));
        assert!(
            !corrente.contains("xxx"),
            "o arquivo corrente devia estar novo"
        );
        assert!(diretorio.join("arca.log.anterior").exists());

        let _ = fs::remove_dir_all(&diretorio);
    }

    #[test]
    fn falha_de_escrita_nao_interrompe() {
        // Um caminho impossivel: o registro engole e segue.
        let registro = Registro::em(
            Path::new(r"Z:\nao-existe\ARCA"),
            Box::new(RelogioParado::em("2026-08-22T11:42:03")),
        );
        registro.info("ninguem lê isto, e esta tudo bem");
    }
}
