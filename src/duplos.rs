//! Duplos das portas, para teste sem hardware.
//!
//! Existem porque o que precisa de teste no ARCA — o parser do `bcdedit`, o
//! validador da receita, a regra de espaco, a leitura do desfecho — nao pode
//! depender de um SSD conectado nem de um reinicio. Cada duplo substitui uma
//! porta de [`crate::portas`] e nada mais.

use crate::erro::{Erro, Resultado, erro_de_arquivo};
use crate::portas::{
    Arquivos, DiscoFisico, Discos, Entrada, Firmware, Privilegios, Relogio, Volume,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Um relogio que nao anda. Ver S-6: no ARCA o tempo nunca decide nada, o que
/// torna um relogio parado suficiente para todo teste.
pub struct RelogioParado {
    momento: DateTime<Local>,
}

impl RelogioParado {
    /// A partir de um `2026-08-22T11:42:03`.
    pub fn em(momento_iso: &str) -> RelogioParado {
        let ingenuo = NaiveDateTime::parse_from_str(momento_iso, "%Y-%m-%dT%H:%M:%S")
            .expect("momento em formato ISO-8601 sem fuso");
        RelogioParado {
            momento: Local.from_local_datetime(&ingenuo).unwrap(),
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

/// Discos de mentira, com os volumes e os discos que o teste quiser.
#[derive(Default)]
pub struct DiscosDeMentira {
    pub volumes: Vec<Volume>,
    pub discos: Vec<DiscoFisico>,
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
#[derive(Default)]
pub struct ArquivosEmMemoria {
    conteudo: RefCell<BTreeMap<PathBuf, String>>,
    diretorios: RefCell<Vec<PathBuf>>,
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

    pub fn conteudo_de(&self, caminho: impl AsRef<Path>) -> Option<String> {
        self.conteudo.borrow().get(caminho.as_ref()).cloned()
    }
}

impl Arquivos for ArquivosEmMemoria {
    fn existe(&self, caminho: &Path) -> bool {
        self.conteudo.borrow().contains_key(caminho)
            || self.diretorios.borrow().iter().any(|d| d == caminho)
    }

    fn ler_texto(&self, caminho: &Path) -> Resultado<String> {
        self.conteudo.borrow().get(caminho).cloned().ok_or_else(|| {
            erro_de_arquivo("leitura", caminho)(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "nao existe neste sistema de arquivos de mentira",
            ))
        })
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
        let arquivos = self.conteudo.borrow();
        let mut entradas: Vec<Entrada> = arquivos
            .iter()
            .filter(|(filho, _)| filho.parent() == Some(caminho))
            .map(|(filho, conteudo)| Entrada {
                caminho: filho.clone(),
                diretorio: false,
                tamanho_bytes: conteudo.len() as u64,
            })
            .collect();

        entradas.extend(
            self.diretorios
                .borrow()
                .iter()
                .filter(|filho| filho.parent() == Some(caminho))
                .map(|filho| Entrada {
                    caminho: filho.clone(),
                    diretorio: true,
                    tamanho_bytes: 0,
                }),
        );

        entradas.sort_by(|a, b| a.caminho.cmp(&b.caminho));
        Ok(entradas)
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
