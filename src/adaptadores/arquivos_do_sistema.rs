//! O sistema de arquivos de verdade, por caminho — nunca por dispositivo.

use crate::erro::{Resultado, erro_de_arquivo};
use crate::portas::{Arquivos, Entrada, OQueHa};
use chrono::{DateTime, Local};
use std::fs;
use std::io;
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

    fn o_que_ha(&self, caminho: &Path) -> Resultado<OQueHa> {
        // `fs::metadata`, e nao `Path::is_file`: o `is_file` transforma
        // qualquer erro dos metadados em `false`, e ate 29/09/2026 (WPC-68)
        // era esse `false` que virava "nao esta la".
        match fs::metadata(caminho) {
            Ok(metadados) if metadados.is_dir() => Ok(OQueHa::Pasta),
            Ok(_) => Ok(OQueHa::Arquivo),
            Err(origem) if nada_pode_estar_ali(&origem) => Ok(OQueHa::Nada),
            Err(origem) => Err(erro_de_arquivo("leitura de metadados", caminho)(origem)),
        }
    }

    fn ler_texto(&self, caminho: &Path) -> Resultado<String> {
        fs::read_to_string(caminho).map_err(erro_de_arquivo("leitura", caminho))
    }

    fn ler_texto_alheio(&self, caminho: &Path) -> Resultado<String> {
        let bytes = fs::read(caminho).map_err(erro_de_arquivo("leitura", caminho))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
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
                modificado_em: metadados.modified().ok().map(DateTime::<Local>::from),
            });
        }

        entradas.sort_by(|a, b| a.caminho.cmp(&b.caminho));
        Ok(entradas)
    }

    fn copiar(&self, origem: &Path, destino: &Path) -> Resultado<()> {
        // `fs::copy` usa o `CopyFileExW` do Windows, que copia sem passar o
        // conteudo pela memoria deste processo — o que importa quando o
        // arquivo tem meio giga (a copia do pacote de PR-3).
        //
        // Nao e atomica, e nao precisa ser: os dois usos do `arca prepare`
        // escrevem onde nada havia, num disco que o proprio comando acabou de
        // particionar. Uma copia interrompida deixa um arquivo pela metade
        // num dispositivo que ainda nao esta pronto, e rodar o comando de novo
        // resolve — ele comeca apagando.
        fs::copy(origem, destino)
            .map(|_| ())
            .map_err(erro_de_arquivo("copia", destino))
    }

    fn espaco_livre(&self, caminho: &Path) -> Resultado<u64> {
        espaco_livre_do_volume(caminho)
    }
}

/// Se o erro dos metadados e o Windows respondendo que nada existe ali, e nao
/// deixando de responder.
///
/// `NotFound` cobre o arquivo ausente (2) e o caminho ausente (3), que e
/// tambem o que volta para um drive que nao existe e para um arquivo no meio
/// do caminho. O `ERROR_INVALID_NAME` (123) entra porque num nome invalido
/// nada pode existir, e porque e nele que cai um erro de digitacao comum: o
/// Windows PowerShell 5.1 entrega `--iso "C:\Users\Ana Paula\Downloads\"`
/// como `C:\Users\Ana Paula\Downloads"`, com a aspa no nome. Tudo medido em
/// 29/09/2026. Sem o 123, essa aspa perderia a recusa `NaoEstaLa`, que fala
/// justamente de aspas.
///
/// Compara o codigo, e nao `ErrorKind::InvalidFilename`, porque essa variante
/// e do Rust 1.87 e o `rust-version` do projeto e 1.85.
#[cfg(windows)]
fn nada_pode_estar_ali(erro: &io::Error) -> bool {
    use windows_sys::Win32::Foundation::ERROR_INVALID_NAME;

    erro.kind() == io::ErrorKind::NotFound || erro.raw_os_error() == Some(ERROR_INVALID_NAME as i32)
}

#[cfg(not(windows))]
fn nada_pode_estar_ali(erro: &io::Error) -> bool {
    erro.kind() == io::ErrorKind::NotFound
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
    fn uma_descricao_como_o_bloco_de_notas_a_escreve_chega_a_listagem() {
        // L-3 num sistema de arquivos de verdade, e nao no duplo: o
        // `ArquivosEmMemoria` guarda `String`, e por isso nunca ve os **bytes**
        // que um bloco de notas grava. Aqui eles sao reais — o BOM de UTF-8
        // (`EF BB BF`) que o "Salvar como" escreve, CRLF no fim de cada linha,
        // e um `ç` em dois bytes.
        let vault = std::env::temp_dir().join(format!("arca-descricao-{}", std::process::id()));
        let imagem = vault.join("2026-08-22_Apps");
        fs::create_dir_all(&imagem).unwrap();
        fs::write(imagem.join("MD5SUMS"), b"abc  nvme0n1p3").unwrap();
        fs::write(
            imagem.join("arca-descricao.txt"),
            b"\xEF\xBB\xBFDepois do Office\r\ne das licen\xC3\xA7as.\r\n",
        )
        .unwrap();

        let pastas = crate::imagens::enumerar(&ArquivosDoSistema, &vault).unwrap();

        assert_eq!(
            pastas[0].descricao.as_deref(),
            Some("Depois do Office e das licenças.")
        );

        fs::remove_dir_all(&vault).unwrap();
    }

    #[test]
    fn uma_pasta_existe_e_nao_e_arquivo() {
        // O pre-voo do `arca prepare --iso` separa a pasta do pacote com esta
        // pergunta, e os testes dele rodam sobre o duplo, que responde o que
        // lhe ensinaram. Aqui quem responde e o Windows: o `existe` diz sim
        // para a pasta — foi por isso que ela chegava ao `certutil` ate
        // 28/09/2026 (WPC-64) —, e o `o_que_ha` tem de dizer que e pasta.
        let pasta = std::env::temp_dir().join(format!("arca-pasta-{}", std::process::id()));
        fs::create_dir_all(&pasta).unwrap();
        let pacote = pasta.join("clonezilla-live-3.3.3-15-amd64.zip");
        fs::write(&pacote, b"o zip, de mentira").unwrap();

        assert!(ArquivosDoSistema.existe(&pasta));
        assert_eq!(ArquivosDoSistema.o_que_ha(&pasta).unwrap(), OQueHa::Pasta);
        assert_eq!(
            ArquivosDoSistema.o_que_ha(&pacote).unwrap(),
            OQueHa::Arquivo
        );
        assert_eq!(
            ArquivosDoSistema
                .o_que_ha(&pasta.join("nao-existe.zip"))
                .unwrap(),
            OQueHa::Nada
        );
        assert_eq!(
            ArquivosDoSistema
                .o_que_ha(&pasta.join(r"nao-existe\clonezilla-live-3.3.3-15-amd64.zip"))
                .unwrap(),
            OQueHa::Nada,
            "a pasta que falta no meio do caminho e o erro 3, e nao o 2"
        );

        let _ = fs::remove_dir_all(&pasta);
    }

    #[test]
    fn um_erro_dos_metadados_nao_vira_nada() {
        // A regra da casa no adaptador: "nao consegui olhar" nunca vira "nao
        // ha nada la". O `existe` responde `false` para qualquer erro dos
        // metadados, e ate 29/09/2026 (WPC-68) era assim que o pre-voo e a
        // conferencia decidiam ausencia.
        //
        // Um NUL no nome e o jeito de ter, sem mexer em permissao, um erro que
        // nao e resposta do Windows: a `std` recusa o caminho antes de chama-lo,
        // com `InvalidInput` (medido em 29/09/2026).
        let recusado = std::env::temp_dir().join("arca\0nome.zip");

        assert!(!ArquivosDoSistema.existe(&recusado));
        let erro = ArquivosDoSistema
            .o_que_ha(&recusado)
            .expect_err("um erro dos metadados virou resposta");
        assert!(
            !erro.e_arquivo_ausente(),
            "o erro diz que o arquivo nao esta la: {erro}"
        );
    }

    #[test]
    fn um_nome_que_o_windows_recusa_e_nada() {
        // O `ERROR_INVALID_NAME` e resposta, e nao falta de uma: num nome
        // invalido nada pode existir. O primeiro caminho e o que o Windows
        // PowerShell 5.1 entrega para `--iso "C:\Users\Ana Paula\Downloads\"`
        // (medido em 29/09/2026), e ele tem de continuar saindo `NaoEstaLa`.
        let temporario = std::env::temp_dir();

        for nome in ["Downloads\"", "arca<pacote>.zip", "arca?.zip"] {
            assert_eq!(
                ArquivosDoSistema.o_que_ha(&temporario.join(nome)).unwrap(),
                OQueHa::Nada,
                "{nome}"
            );
        }
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
