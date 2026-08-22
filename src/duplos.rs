//! Duplos das portas, para teste sem hardware.
//!
//! Existem porque o que precisa de teste no ARCA — o parser do `bcdedit`, o
//! validador da receita, a regra de espaco, a leitura do desfecho — nao pode
//! depender de um SSD conectado nem de um reinicio. Cada duplo substitui uma
//! porta de [`crate::portas`] e nada mais.

use crate::erro::{Erro, Resultado, erro_de_arquivo};
use crate::portas::{
    Arquivos, DiscoFisico, Discos, Entrada, Firmware, Privilegios, Relogio, TipoDeMidia, Volume,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Um momento a partir de um `2026-08-21T12:56:31`.
pub fn momento(iso: &str) -> DateTime<Local> {
    let ingenuo = NaiveDateTime::parse_from_str(iso, "%Y-%m-%dT%H:%M:%S")
        .expect("momento em formato ISO-8601 sem fuso");
    Local.from_local_datetime(&ingenuo).unwrap()
}

/// Um relogio que nao anda. Ver S-6: no ARCA o tempo nunca decide nada, o que
/// torna um relogio parado suficiente para todo teste.
pub struct RelogioParado {
    momento: DateTime<Local>,
}

impl RelogioParado {
    /// A partir de um `2026-08-22T11:42:03`.
    pub fn em(momento_iso: &str) -> RelogioParado {
        RelogioParado {
            momento: momento(momento_iso),
        }
    }
}

impl Relogio for RelogioParado {
    fn agora(&self) -> DateTime<Local> {
        self.momento
    }
}

/// Um firmware de mentira: devolve o texto que lhe deram e guarda o que
/// mandaram executar.
#[derive(Default)]
pub struct FirmwareDeMentira {
    respostas: BTreeMap<String, String>,
    pub executados: RefCell<Vec<Vec<String>>>,
}

impl FirmwareDeMentira {
    pub fn novo() -> FirmwareDeMentira {
        FirmwareDeMentira::default()
    }

    /// Ensina o duplo a responder a um `/enum <alvo>`.
    pub fn respondendo(mut self, alvo: &str, saida: &str) -> FirmwareDeMentira {
        self.respostas.insert(alvo.to_string(), saida.to_string());
        self
    }
}

impl Firmware for FirmwareDeMentira {
    fn enumerar(&self, alvo: &str) -> Resultado<String> {
        Ok(self.respostas.get(alvo).cloned().unwrap_or_default())
    }

    fn executar(&self, argumentos: &[&str]) -> Resultado<String> {
        self.executados
            .borrow_mut()
            .push(argumentos.iter().map(|a| a.to_string()).collect());
        Ok(String::new())
    }
}

/// Um volume de mentira, identificado pelo rotulo.
///
/// A descoberta do dispositivo so olha rotulo (S-3); a letra existe para
/// montar caminho de arquivo do lado Windows. O resto e enfeite crivel.
pub fn volume(rotulo: &str, letra: char, total_bytes: u64, livre_bytes: u64) -> Volume {
    Volume {
        rotulo: Some(rotulo.to_string()),
        letra: Some(letra),
        sistema_de_arquivos: "NTFS".to_string(),
        total_bytes,
        livre_bytes,
        tipo_de_midia: TipoDeMidia::DiscoFixo,
    }
}

/// Discos de mentira, com os volumes e os discos que o teste quiser.
#[derive(Default)]
pub struct DiscosDeMentira {
    pub volumes: Vec<Volume>,
    pub discos: Vec<DiscoFisico>,
}

impl DiscosDeMentira {
    /// Um dispositivo ARCA inteiro: `ARCAVAULT` e `ARCABOOT`, como o §4 do PRD
    /// descreve.
    pub fn com_dispositivo() -> DiscosDeMentira {
        DiscosDeMentira {
            volumes: vec![
                volume("Windows", 'C', 498_700_000_000, 361_400_000_000),
                volume("ARCAVAULT", 'E', 254_000_000_000, 176_400_000_000),
                volume("ARCABOOT", 'R', 1_700_000_000, 1_070_000_000),
            ],
            discos: Vec::new(),
        }
    }

    pub fn com_volumes(volumes: Vec<Volume>) -> DiscosDeMentira {
        DiscosDeMentira {
            volumes,
            discos: Vec::new(),
        }
    }
}

impl Discos for DiscosDeMentira {
    fn volumes(&self) -> Resultado<Vec<Volume>> {
        Ok(self.volumes.clone())
    }

    fn discos_fisicos(&self) -> Resultado<Vec<DiscoFisico>> {
        Ok(self.discos.clone())
    }
}

/// Um sistema de arquivos na memoria. A escrita e atomica de graca: ou a
/// entrada do mapa mudou, ou nao.
///
/// Os diretorios sao **implicitos**: gravar `E:\imagem\MD5SUMS` faz `E:\`
/// listar `imagem` como diretorio, sem que ninguem precise cria-lo. E o que
/// torna crivel montar uma arvore de imagens em tres linhas de teste — e um
/// diretorio de verdade tambem so existe porque alguma coisa esta dentro
/// dele. [`ArquivosEmMemoria::com_pasta_vazia`] cobre o caso restante.
#[derive(Default)]
pub struct ArquivosEmMemoria {
    conteudo: RefCell<BTreeMap<PathBuf, String>>,
    diretorios: RefCell<Vec<PathBuf>>,
    datas: RefCell<BTreeMap<PathBuf, DateTime<Local>>>,
    pub espaco_livre: u64,
}

impl ArquivosEmMemoria {
    pub fn novo() -> ArquivosEmMemoria {
        ArquivosEmMemoria::default()
    }

    pub fn com(self, caminho: impl Into<PathBuf>, conteudo: &str) -> ArquivosEmMemoria {
        self.conteudo
            .borrow_mut()
            .insert(caminho.into(), conteudo.to_string());
        self
    }

    /// Uma pasta sem nada dentro — o residuo de um backup interrompido cedo
    /// demais para ter escrito o primeiro arquivo.
    pub fn com_pasta_vazia(self, caminho: impl Into<PathBuf>) -> ArquivosEmMemoria {
        self.diretorios.borrow_mut().push(caminho.into());
        self
    }

    /// Carimba uma data num caminho, para os testes que exibem data de imagem.
    pub fn datado(self, caminho: impl Into<PathBuf>, momento_iso: &str) -> ArquivosEmMemoria {
        self.datas
            .borrow_mut()
            .insert(caminho.into(), momento(momento_iso));
        self
    }

    pub fn conteudo_de(&self, caminho: impl AsRef<Path>) -> Option<String> {
        self.conteudo.borrow().get(caminho.as_ref()).cloned()
    }

    /// O filho imediato de `base` no caminho de um descendente, e se esse
    /// filho e diretorio — o que se sabe por haver mais componentes depois
    /// dele.
    fn filho_imediato(base: &Path, descendente: &Path) -> Option<(PathBuf, bool)> {
        let resto = descendente.strip_prefix(base).ok()?;
        let mut componentes = resto.components();
        let primeiro = componentes.next()?;
        let e_diretorio = componentes.next().is_some();
        Some((base.join(primeiro), e_diretorio))
    }
}

impl Arquivos for ArquivosEmMemoria {
    fn existe(&self, caminho: &Path) -> bool {
        // `starts_with` cobre os dois casos de uma vez: o caminho exato, que
        // comeca com ele mesmo, e o diretorio implicito, que existe porque
        // alguma coisa mora dentro dele. Uma raiz so existe se ha algo nela —
        // e por isso que um `ArquivosEmMemoria` vazio nao tem nem `E:\`.
        let arquivos = self.conteudo.borrow();
        arquivos.keys().any(|arquivo| arquivo.starts_with(caminho))
            || self
                .diretorios
                .borrow()
                .iter()
                .any(|diretorio| diretorio.starts_with(caminho))
    }

    fn ler_texto(&self, caminho: &Path) -> Resultado<String> {
        self.conteudo.borrow().get(caminho).cloned().ok_or_else(|| {
            erro_de_arquivo("leitura", caminho)(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "nao existe neste sistema de arquivos de mentira",
            ))
        })
    }

    fn ler_texto_alheio(&self, caminho: &Path) -> Resultado<String> {
        self.ler_texto(caminho)
    }

    fn escrever_atomico(&self, caminho: &Path, conteudo: &str) -> Resultado<()> {
        self.conteudo
            .borrow_mut()
            .insert(caminho.to_path_buf(), conteudo.to_string());
        Ok(())
    }

    fn criar_diretorio(&self, caminho: &Path) -> Resultado<()> {
        let mut diretorios = self.diretorios.borrow_mut();
        if !diretorios.iter().any(|d| d == caminho) {
            diretorios.push(caminho.to_path_buf());
        }
        Ok(())
    }

    fn listar(&self, caminho: &Path) -> Resultado<Vec<Entrada>> {
        // Como o `read_dir` de verdade: caminho que nao existe e erro, nao
        // diretorio vazio. Sem isto, um `arca list` apontado para a raiz
        // errada imprimiria "Nenhuma imagem em ARCAVAULT" em todo teste e
        // falharia em producao — divergencia que nenhum duplo pode ter.
        if !self.existe(caminho) {
            return Err(erro_de_arquivo("listagem", caminho)(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "nao existe neste sistema de arquivos de mentira",
            )));
        }

        let datas = self.datas.borrow();
        let mut entradas: BTreeMap<PathBuf, Entrada> = BTreeMap::new();

        let mut anotar = |filho: PathBuf, diretorio: bool, tamanho_bytes: u64| {
            let modificado_em = datas.get(&filho).copied();
            entradas.entry(filho.clone()).or_insert(Entrada {
                caminho: filho,
                diretorio,
                tamanho_bytes,
                modificado_em,
            });
        };

        for (arquivo, conteudo) in self.conteudo.borrow().iter() {
            if let Some((filho, e_diretorio)) = Self::filho_imediato(caminho, arquivo) {
                let tamanho = if e_diretorio {
                    0
                } else {
                    conteudo.len() as u64
                };
                anotar(filho, e_diretorio, tamanho);
            }
        }

        for diretorio in self.diretorios.borrow().iter() {
            if let Some((filho, _)) = Self::filho_imediato(caminho, diretorio) {
                anotar(filho, true, 0);
            }
        }

        Ok(entradas.into_values().collect())
    }

    fn espaco_livre(&self, _caminho: &Path) -> Resultado<u64> {
        Ok(self.espaco_livre)
    }
}

/// Privilegios de mentira: diz se esta elevado e guarda o que teria sido
/// repassado ao relancar. E com ele que C-7 se verifica sem UAC.
pub struct PrivilegiosDeMentira {
    /// `None` reproduz a consulta de token que falhou — o caso em que "nao
    /// sei" nao pode virar "nao elevado".
    pub elevado: Option<bool>,
    pub codigo_do_relancamento: i32,
    pub recusar: bool,
    pub repassados: RefCell<Vec<Vec<String>>>,
}

impl PrivilegiosDeMentira {
    pub fn elevado() -> PrivilegiosDeMentira {
        PrivilegiosDeMentira {
            elevado: Some(true),
            codigo_do_relancamento: 0,
            recusar: false,
            repassados: RefCell::new(Vec::new()),
        }
    }

    pub fn sem_elevacao() -> PrivilegiosDeMentira {
        PrivilegiosDeMentira {
            elevado: Some(false),
            ..PrivilegiosDeMentira::elevado()
        }
    }

    pub fn recusando() -> PrivilegiosDeMentira {
        PrivilegiosDeMentira {
            recusar: true,
            ..PrivilegiosDeMentira::sem_elevacao()
        }
    }

    /// A consulta de token nao responde.
    pub fn indeterminado() -> PrivilegiosDeMentira {
        PrivilegiosDeMentira {
            elevado: None,
            ..PrivilegiosDeMentira::elevado()
        }
    }

    /// O que foi repassado no ultimo relancamento.
    pub fn ultimo_repasse(&self) -> Option<Vec<String>> {
        self.repassados.borrow().last().cloned()
    }
}

impl Privilegios for PrivilegiosDeMentira {
    fn elevado(&self) -> Resultado<bool> {
        self.elevado
            .ok_or_else(|| Erro::ElevacaoIndeterminada("consulta de token de mentira".to_string()))
    }

    fn relancar_elevado(&self, argumentos: &[String]) -> Resultado<i32> {
        self.repassados.borrow_mut().push(argumentos.to_vec());
        if self.recusar {
            return Err(Erro::ElevacaoRecusada);
        }
        Ok(self.codigo_do_relancamento)
    }
}
