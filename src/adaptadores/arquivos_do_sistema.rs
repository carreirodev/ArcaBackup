//! O sistema de arquivos de verdade, por caminho — nunca por dispositivo.

use crate::erro::{Resultado, erro_de_arquivo};
use crate::portas::{Arquivos, Entrada};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default)]
pub struct ArquivosDoSistema;

impl ArquivosDoSistema {
    /// O caminho temporario da escrita atomica, vizinho do destino para que a
    /// renomeacao fique dentro do mesmo volume — entre volumes ela deixa de
    /// ser atomica.
    fn temporario_de(caminho: &Path) -> PathBuf {
        let mut nome = caminho.file_name().unwrap_or_default().to_os_string();
        nome.push(".arca-tmp");
        caminho.with_file_name(nome)
    }
}

impl Arquivos for ArquivosDoSistema {
    fn existe(&self, caminho: &Path) -> bool {
        caminho.exists()
    }

    fn ler_texto(&self, caminho: &Path) -> Resultado<String> {
        fs::read_to_string(caminho).map_err(erro_de_arquivo("leitura", caminho))
    }

    fn escrever_atomico(&self, caminho: &Path, conteudo: &str) -> Resultado<()> {
        use std::io::Write;

        let temporario = Self::temporario_de(caminho);

        // O conteudo precisa estar no disco **antes** da renomeacao. Sem o
        // `sync_all`, um desligamento no meio pode deixar o nome novo
        // apontando para um arquivo vazio — e o estado do job mora no
        // `ARCABOOT`, que e justamente o que se lê depois de um desligamento.
        {
            let mut arquivo =
                fs::File::create(&temporario).map_err(erro_de_arquivo("escrita", &temporario))?;
            arquivo
                .write_all(conteudo.as_bytes())
                .map_err(erro_de_arquivo("escrita", &temporario))?;
            arquivo
                .sync_all()
                .map_err(erro_de_arquivo("sincronizacao", &temporario))?;
        }

        // No Windows a renomeacao falha se o destino existir; `fs::rename`
        // usa `MoveFileEx` com `MOVEFILE_REPLACE_EXISTING`, que substitui.
        match fs::rename(&temporario, caminho) {
            Ok(()) => Ok(()),
            Err(origem) => {
                let _ = fs::remove_file(&temporario);
                Err(erro_de_arquivo("renomeacao", caminho)(origem))
            }
        }
    }

    fn criar_diretorio(&self, caminho: &Path) -> Resultado<()> {
        fs::create_dir_all(caminho).map_err(erro_de_arquivo("criacao de diretorio", caminho))
    }

    fn listar(&self, caminho: &Path) -> Resultado<Vec<Entrada>> {
        let leitura = fs::read_dir(caminho).map_err(erro_de_arquivo("listagem", caminho))?;
        let mut entradas = Vec::new();

        for item in leitura {
            let item = item.map_err(erro_de_arquivo("listagem", caminho))?;
            let metadados = item
                .metadata()
                .map_err(erro_de_arquivo("leitura de metadados", item.path()))?;
            entradas.push(Entrada {
                caminho: item.path(),
                diretorio: metadados.is_dir(),
                tamanho_bytes: metadados.len(),
            });
        }

        entradas.sort_by(|a, b| a.caminho.cmp(&b.caminho));
        Ok(entradas)
    }

    fn espaco_livre(&self, caminho: &Path) -> Resultado<u64> {
        espaco_livre_do_volume(caminho)
    }
}

/// O diretorio ao qual perguntar pelo espaco livre.
///
/// O `GetDiskFreeSpaceExW` exige um **diretorio**: com caminho de arquivo ele
/// devolve `ERROR_DIRECTORY_NAME_IS_INVALID`. E a pergunta de B-4 — "cabe uma
/// imagem chamada assim?" — e feita justamente sobre um caminho que ainda nao
/// existe, entao subir ate o primeiro diretorio existente e o certo.
fn diretorio_para_consulta(caminho: &Path) -> &Path {
    let mut candidato = caminho;
    loop {
        if candidato.is_dir() {
            return candidato;
        }
        match candidato.parent() {
            Some(pai) if !pai.as_os_str().is_empty() => candidato = pai,
            // Sem pai existente, resta entregar o que veio e deixar o Windows
            // dizer o que ha de errado com ele.
            _ => return caminho,
        }
    }
}

#[cfg(windows)]
fn espaco_livre_do_volume(caminho: &Path) -> Resultado<u64> {
    use std::io;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let diretorio = diretorio_para_consulta(caminho);
    let largo = super::windows::texto::para_utf16(&diretorio.to_string_lossy());
    let mut livre_para_o_usuario: u64 = 0;

    // SEGURANCA: `largo` termina em NUL e vive ate o fim da chamada; o
    // ponteiro de saida aponta para uma variavel da pilha desta funcao.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            largo.as_ptr(),
            &mut livre_para_o_usuario,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if ok == 0 {
        return Err(erro_de_arquivo("consulta de espaco livre", caminho)(
            io::Error::last_os_error(),
        ));
    }
    Ok(livre_para_o_usuario)
}

#[cfg(not(windows))]
fn espaco_livre_do_volume(caminho: &Path) -> Resultado<u64> {
    Err(erro_de_arquivo("consulta de espaco livre", caminho)(
        std::io::Error::new(std::io::ErrorKind::Unsupported, "o ARCA so roda no Windows"),
    ))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::portas::Arquivos;

    #[test]
    fn a_consulta_sobe_ate_um_diretorio_que_existe() {
        let temporario = std::env::temp_dir();
        let inexistente = temporario.join("arca-imagem-que-ainda-nao-existe/MD5SUMS");

        assert_eq!(diretorio_para_consulta(&inexistente), temporario);
        assert_eq!(diretorio_para_consulta(&temporario), temporario);
    }

    #[test]
    fn ha_espaco_livre_num_caminho_que_ainda_nao_existe() {
        // B-4 pergunta pelo espaco **antes** de a imagem existir. Perguntar
        // com o caminho da imagem tem de funcionar.
        let alvo = std::env::temp_dir().join("arca-imagem-futura/2026-08-22_Apps");
        let livre = ArquivosDoSistema
            .espaco_livre(&alvo)
            .expect("o volume responde mesmo sem a pasta existir");

        assert!(livre > 0, "o volume temporario nao deveria estar cheio");
    }

    #[test]
    fn a_escrita_atomica_deixa_o_conteudo_no_lugar() {
        let diretorio = std::env::temp_dir().join(format!("arca-atomico-{}", std::process::id()));
        fs::create_dir_all(&diretorio).unwrap();
        let alvo = diretorio.join("estado.json");

        ArquivosDoSistema
            .escrever_atomico(&alvo, r#"{"selo":"a3f1c9e07b2d4856"}"#)
            .unwrap();
        assert_eq!(
            ArquivosDoSistema.ler_texto(&alvo).unwrap(),
            r#"{"selo":"a3f1c9e07b2d4856"}"#
        );

        // Sobrescrever tem de substituir, nao falhar nem concatenar.
        ArquivosDoSistema
            .escrever_atomico(&alvo, "segundo")
            .unwrap();
        assert_eq!(ArquivosDoSistema.ler_texto(&alvo).unwrap(), "segundo");

        // E o temporario nao pode ficar para tras.
        let restos: Vec<_> = fs::read_dir(&diretorio)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains("arca-tmp"))
            .collect();
        assert!(restos.is_empty(), "sobrou temporario: {restos:?}");

        let _ = fs::remove_dir_all(&diretorio);
    }
}
