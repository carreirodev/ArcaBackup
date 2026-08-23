//! Duplos das portas, para teste sem hardware.
//!
//! Existem porque o que precisa de teste no ARCA — o parser do `bcdedit`, o
//! validador da receita, a regra de espaco, a leitura do desfecho — nao pode
//! depender de um SSD conectado nem de um reinicio. Cada duplo substitui uma
//! porta de [`crate::portas`] e nada mais.

use crate::erro::{Erro, Resultado, erro_de_arquivo};
use crate::portas::{
    Arquivos, Console, DiscoFisico, Discos, Entrada, Entropia, Firmware, Medida, Privilegios,
    Relogio, SaidaDeFerramenta, Sistema, TipoDeMidia, Volume,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::cell::{Cell, RefCell};
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

/// Uma entropia de mentira: entrega os bytes que lhe deram, ou recusa.
///
/// Existe porque um teste sobre o `estado.json` precisa saber que selo
/// esperar, e um gerador de verdade nunca deixaria. Sabe fazer as duas coisas
/// que importam: entregar bytes conhecidos e **recusar** — sem o segundo, o
/// caminho em que a falta de entropia vira um selo de zeros ficaria sem teste,
/// e zeros sao o selo de ensaio.
pub struct EntropiaDeMentira {
    bytes: Vec<u8>,
    recusar: bool,
}

impl EntropiaDeMentira {
    pub fn com(bytes: &[u8]) -> EntropiaDeMentira {
        EntropiaDeMentira {
            bytes: bytes.to_vec(),
            recusar: false,
        }
    }

    pub fn recusando() -> EntropiaDeMentira {
        EntropiaDeMentira {
            bytes: Vec::new(),
            recusar: true,
        }
    }
}

impl Entropia for EntropiaDeMentira {
    fn preencher(&self, destino: &mut [u8]) -> Resultado<()> {
        if self.recusar {
            return Err(Erro::EntropiaIndisponivel {
                estado: -1073741823,
            });
        }

        // Preencher parcialmente e o que a porta proibe: ou vai tudo, ou
        // falha. Um duplo mais frouxo que o contrato esconderia justamente o
        // caso em que o selo sai com zeros no fim.
        if destino.len() != self.bytes.len() {
            panic!(
                "a entropia de mentira tem {} byte(s) e pediram {}",
                self.bytes.len(),
                destino.len()
            );
        }

        destino.copy_from_slice(&self.bytes);
        Ok(())
    }
}

/// Um firmware de mentira: devolve o texto que lhe deram e guarda o que
/// mandaram executar.
///
/// Sabe fazer tres coisas que o `bcdedit` de verdade faz e que os testes
/// precisam reproduzir: responder de um jeito **antes** de uma escrita e de
/// outro **depois** dela — que e como se testa C-3 sem hardware —, recusar o
/// `/enum` como ele recusa sem privilegio, e recusar a escrita como ele recusa
/// quando nao ha o que apagar.
#[derive(Default)]
pub struct FirmwareDeMentira {
    respostas: BTreeMap<String, String>,

    /// O que o `/enum` passa a responder depois da primeira escrita. E assim
    /// que um firmware que **obedeceu** se comporta, e sem isso nao ha como
    /// testar a releitura de C-3.
    respostas_depois: BTreeMap<String, String>,

    /// Como o `bcdedit` recusa um `/enum`: sem privilegio, ele escreve "Acesso
    /// negado" na saida padrao e sai com codigo 1.
    recusa_do_enumerar: Option<Erro>,

    /// Como o `bcdedit` recusa uma escrita. Medido: apagar um `bootsequence`
    /// que nao existe sai com codigo 1 **sem mudar nada**.
    recusa_do_executar: Option<Erro>,

    /// O `{fwbootmgr}` **modelado**, em vez de respondido de cor.
    ///
    /// As duas formas anteriores — resposta fixa, e uma resposta antes da
    /// primeira escrita e outra depois — chegaram ao limite na E7, e o limite
    /// vale ser registrado: um comando que **desarma e depois arma** escreve
    /// duas vezes no mesmo alvo, e as duas escritas esperam respostas
    /// contrarias. Com "a primeira escrita vira a chave para sempre", o
    /// `/deletevalue` do desarmar ja produzia a resposta do armar, e o
    /// desarmar falhava dizendo que a marca sobreviveu.
    ///
    /// Aqui o duplo guarda o `bootsequence` como estado e o aplica: `/set`
    /// poe, `/deletevalue` tira, e o `/enum` conta o que ha. E o que o
    /// `bcdedit` de verdade faz — inclusive sair com codigo 1 ao apagar o que
    /// nao existe, que e o comportamento medido na E4.
    fwbootmgr: Option<RefCell<Fwbootmgr>>,

    pub executados: RefCell<Vec<Vec<String>>>,
}

/// O estado do `{fwbootmgr}` que o duplo modela.
#[derive(Debug, Clone)]
struct Fwbootmgr {
    ordem_permanente: Vec<String>,
    bootsequence: Vec<String>,
}

impl Fwbootmgr {
    /// Como o `bcdedit` desta maquina enumera o gerenciador de firmware.
    fn como_o_bcdedit_escreve(&self) -> String {
        let mut saida = String::from(
            "\r\nGerenciador de Inicialização de Firmware\r\n\
             ----------------------------------------\r\n\
             identificador           {fwbootmgr}\r\n",
        );

        for (campo, valores) in [
            ("displayorder", &self.ordem_permanente),
            ("bootsequence", &self.bootsequence),
        ] {
            for (indice, valor) in valores.iter().enumerate() {
                if indice == 0 {
                    saida.push_str(&format!("{campo:<24}{valor}\r\n"));
                } else {
                    saida.push_str(&format!("{:<24}{valor}\r\n", ""));
                }
            }
        }

        saida.push_str("timeout                 1\r\n");
        saida
    }
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

    /// O que o `/enum <alvo>` responde **depois** de a primeira escrita ter
    /// acontecido — um firmware que obedeceu.
    pub fn respondendo_depois(mut self, alvo: &str, saida: &str) -> FirmwareDeMentira {
        self.respostas_depois
            .insert(alvo.to_string(), saida.to_string());
        self
    }

    pub fn recusando_o_enumerar(mut self, recusa: Erro) -> FirmwareDeMentira {
        self.recusa_do_enumerar = Some(recusa);
        self
    }

    pub fn recusando_o_executar(mut self, recusa: Erro) -> FirmwareDeMentira {
        self.recusa_do_executar = Some(recusa);
        self
    }

    /// Modela o `{fwbootmgr}` em vez de responder de cor: o `/set` poe a
    /// marca, o `/deletevalue` tira, e o `/enum` conta o que ha.
    ///
    /// E o que um comando que **desarma e depois arma** exige — ver o campo
    /// [`FirmwareDeMentira::fwbootmgr`].
    pub fn modelando_o_fwbootmgr(mut self, ordem_permanente: &[&str]) -> FirmwareDeMentira {
        self.fwbootmgr = Some(RefCell::new(Fwbootmgr {
            ordem_permanente: ordem_permanente.iter().map(|o| o.to_string()).collect(),
            bootsequence: Vec::new(),
        }));
        self
    }

    fn ja_escreveu(&self) -> bool {
        !self.executados.borrow().is_empty()
    }

    /// O que se mandou o `bcdedit` executar, na ordem.
    ///
    /// Existe para cobrar que a escrita **aconteceu** — que e o que separa
    /// "nao precisava" de "nao fez", e o que uma leitura do modelo depois nao
    /// distingue: uma ordem ja certa fica igual nos dois casos.
    pub fn executados(&self) -> Vec<Vec<String>> {
        self.executados.borrow().clone()
    }

    /// A ordem permanente como o modelo a tem agora.
    ///
    /// So responde com [`FirmwareDeMentira::modelando_o_fwbootmgr`] ligado —
    /// sem modelo nao ha estado a inspecionar, e devolver uma lista vazia
    /// faria um teste passar por engano.
    pub fn ordem_permanente(&self) -> Vec<String> {
        self.fwbootmgr
            .as_ref()
            .expect("`ordem_permanente` exige `modelando_o_fwbootmgr`")
            .borrow()
            .ordem_permanente
            .clone()
    }

    /// Aplica ao modelo o que o `bcdedit` faria, e diz se a chamada teria
    /// saido com codigo zero.
    ///
    /// `None` quando esta escrita nao e do `{fwbootmgr}` modelado — e quem
    /// responde por ela e o caminho de sempre, com a
    /// [`FirmwareDeMentira::recusando_o_executar`] se houver uma.
    ///
    /// **O `_` devolvia `Some(true)`, e a revisao pegou.** Com isso, o
    /// `recusa_do_executar` ficava morto sempre que o modelo estivesse ligado:
    /// um teste escrito como `.modelando_o_fwbootmgr(...).recusando_o_executar(
    /// acesso_negado())` — que e a forma natural de exercitar um `/set
    /// description` ou `/set device` que falha — passava verde com a recusa
    /// nunca disparando. Dois construtores que parecem compor e nao compõem
    /// sao piores do que um que nao existe.
    fn aplicar_ao_modelo(&self, argumentos: &[&str]) -> Option<bool> {
        let modelo = self.fwbootmgr.as_ref()?;
        match argumentos {
            ["/set", "{fwbootmgr}", "bootsequence", entradas @ ..] => {
                modelo.borrow_mut().bootsequence = entradas.iter().map(|e| e.to_string()).collect();
                Some(true)
            }
            ["/deletevalue", "{fwbootmgr}", "bootsequence"] => {
                let mut modelo = modelo.borrow_mut();
                let havia = !modelo.bootsequence.is_empty();
                modelo.bootsequence.clear();
                // Medido na E4: apagar o que nao existe sai com codigo 1 e nao
                // muda nada. E o caso normal, e e ele que um desarmar ingenuo
                // transformaria em falha.
                Some(havia)
            }
            // Medido a mao em 23/08/2026, e o help do `bcdedit` diz a mesma
            // coisa: **move para o topo se ja estiver na lista, e insere se
            // nao estiver.** Nada sai da ordem, e a chamada sai com codigo 0
            // inclusive quando nao muda nada — que e o caso idempotente de
            // C-13.
            [
                "/set",
                "{fwbootmgr}",
                "displayorder",
                identificador,
                "/addfirst",
            ] => {
                let mut modelo = modelo.borrow_mut();
                modelo
                    .ordem_permanente
                    .retain(|id| !id.eq_ignore_ascii_case(identificador));
                modelo.ordem_permanente.insert(0, identificador.to_string());
                Some(true)
            }
            // Escrita em outro alvo — a descricao de C-4, o `device` de C-6.
            // O modelo so conhece o `{fwbootmgr}`.
            _ => None,
        }
    }
}

impl Firmware for FirmwareDeMentira {
    fn enumerar(&self, alvo: &str) -> Resultado<String> {
        if let Some(recusa) = &self.recusa_do_enumerar {
            return Err(clonar_a_recusa(recusa));
        }

        // **Os dois alvos nao respondem a mesma coisa, e o duplo os tratava
        // como se respondessem.** Medido em 23/08/2026, rodando o comando de
        // verdade: `/enum {fwbootmgr}` devolve o bloco do gerenciador
        // **sozinho**, e `/enum firmware` devolve o gerenciador **e** as
        // entradas. Com os dois iguais aqui, `crate::ordem` passava nos testes
        // lendo o alvo errado e saia com um GUID onde a tela promete um nome.
        if let Some(modelo) = &self.fwbootmgr {
            if alvo == "{fwbootmgr}" {
                return Ok(modelo.borrow().como_o_bcdedit_escreve());
            }
            if alvo == "firmware" {
                let mut saida = modelo.borrow().como_o_bcdedit_escreve();
                if let Some(entradas) = self.respostas.get(alvo) {
                    saida.push_str(entradas);
                }
                return Ok(saida);
            }
        }

        if self.ja_escreveu() {
            if let Some(saida) = self.respostas_depois.get(alvo) {
                return Ok(saida.clone());
            }
        }
        Ok(self.respostas.get(alvo).cloned().unwrap_or_default())
    }

    fn executar(&self, argumentos: &[&str]) -> Resultado<String> {
        self.executados
            .borrow_mut()
            .push(argumentos.iter().map(|a| a.to_string()).collect());

        // A recusa injetada vem **antes** do modelo, e nao depois: quem a
        // pediu quer exercitar um `bcdedit` que recusa, e o modelo nao pode
        // engoli-la.
        if let Some(recusa) = &self.recusa_do_executar {
            return Err(clonar_a_recusa(recusa));
        }

        if let Some(deu_certo) = self.aplicar_ao_modelo(argumentos) {
            if deu_certo {
                return Ok("A operação foi concluída com êxito.".to_string());
            }
            // A recusa medida em 22/08/2026, com o `{fwbootmgr}` sem
            // `bootsequence`: codigo 1, e nada muda.
            return Err(Erro::FerramentaRecusou {
                ferramenta: "bcdedit",
                codigo: 1,
                saida: "Erro ao tentar excluir o elemento de dados especificado.\nElemento não encontrado.".to_string(),
            });
        }

        Ok(String::new())
    }
}

/// O [`Erro`] nao e `Clone` — ele carrega `io::Error`, que nao e —, e o duplo
/// precisa devolver a mesma recusa mais de uma vez. Aqui so as variantes que
/// o `bcdedit` produz interessam; qualquer outra vira uma recusa generica,
/// visivelmente rotulada, em vez de um `unwrap` escondido.
fn clonar_a_recusa(recusa: &Erro) -> Erro {
    match recusa {
        Erro::FerramentaRecusou {
            ferramenta,
            codigo,
            saida,
        } => Erro::FerramentaRecusou {
            ferramenta,
            codigo: *codigo,
            saida: saida.clone(),
        },
        outro => Erro::FerramentaRecusou {
            ferramenta: "duplo",
            codigo: -1,
            saida: outro.to_string(),
        },
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
    ///
    /// Os dois rotulos vem no **mesmo disco fisico**, como nesta mesa. Um
    /// duplo em que eles estivessem em discos diferentes reprovaria o pre-voo
    /// da E6 em todo teste.
    pub fn com_dispositivo() -> DiscosDeMentira {
        DiscosDeMentira {
            volumes: vec![
                volume("Windows", 'C', 498_701_692_928, 387_131_686_912),
                volume("ARCAVAULT", 'E', 254_379_290_624, 176_291_147_776),
                volume("ARCABOOT", 'R', 1_673_527_296, 1_101_361_152),
            ],
            discos: discos_desta_mesa(),
        }
    }

    pub fn com_volumes(volumes: Vec<Volume>) -> DiscosDeMentira {
        DiscosDeMentira {
            volumes,
            discos: Vec::new(),
        }
    }

    pub fn com_discos(mut self, discos: Vec<DiscoFisico>) -> DiscosDeMentira {
        self.discos = discos;
        self
    }
}

/// Os dois discos desta maquina, medidos pelo WMI em 22/08/2026, com a medida
/// do `MSFT_Disk` acrescentada em 23/08/2026.
///
/// Numeros de verdade, e nao redondos: um teste que passe com
/// `498_700_000_000` e falhe com o tamanho real nao esta testando nada.
///
/// **Os dois tamanhos do mesmo disco estao aqui de proposito.** O
/// `tamanho_bytes` e o `Win32_DiskDrive.Size`, e a `medida` e o
/// `MSFT_Disk.Size`: no `KINGSTON SNV3S500G` eles diferem em 2.612.736 bytes,
/// e e nessa diferenca que R-7 tropecaria se medisse as duas pontas em reguas
/// diferentes. Um duplo que trouxesse o mesmo numero nos dois campos faria
/// todo teste de R-7 passar sem exercitar nada. Ver [`crate::gpt`].
pub fn discos_desta_mesa() -> Vec<DiscoFisico> {
    vec![
        DiscoFisico {
            indice: 0,
            modelo: "KINGSTON SNV3S500G".to_string(),
            tamanho_bytes: 500_105_249_280,
            medida: Some(Medida {
                bytes: 500_107_862_016,
                bytes_por_setor: 512,
            }),
            em_uso_bytes: 112_973_562_368,
            tipo_de_midia: TipoDeMidia::DiscoFixo,
            letras: vec!['C'],
        },
        DiscoFisico {
            indice: 1,
            modelo: "KGSSE100 256 SCSI Disk Device".to_string(),
            tamanho_bytes: 256_052_966_400,
            medida: Some(Medida {
                bytes: 256_060_514_304,
                bytes_por_setor: 512,
            }),
            em_uso_bytes: 78_660_457_472,
            tipo_de_midia: TipoDeMidia::DiscoExterno,
            letras: vec!['E', 'R'],
        },
    ]
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

    /// Todo caminho que alguem tentou lê ou olhar, na ordem.
    ///
    /// Existe por causa de C-1: "sem consultar estado nenhum" so vira teste
    /// se der para perguntar ao duplo o que foi consultado. Sem isto, um teste
    /// de C-1 provaria no maximo que o `estado.json` nao **mudou** — e o
    /// requisito nao e sobre mudar, e sobre nem olhar.
    consultados: RefCell<Vec<PathBuf>>,

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

    /// Se alguem chegou a olhar para este caminho — lendo ou perguntando se
    /// existe. E o que torna C-1 verificavel.
    pub fn foi_consultado(&self, caminho: impl AsRef<Path>) -> bool {
        let caminho = caminho.as_ref();
        self.consultados
            .borrow()
            .iter()
            .any(|consultado| consultado == caminho)
    }

    fn anotar_consulta(&self, caminho: &Path) {
        self.consultados.borrow_mut().push(caminho.to_path_buf());
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
        self.anotar_consulta(caminho);

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
        self.anotar_consulta(caminho);
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

    fn copiar(&self, origem: &Path, destino: &Path) -> Resultado<()> {
        // A copia registra a origem em `consultados` porque ela **lê** —
        // C-1 pergunta o que foi consultado, e uma copia que nao aparecesse
        // ali seria uma leitura escondida do teste.
        self.anotar_consulta(origem);

        // Um arquivo que nao existe e erro, como no sistema de arquivos de
        // verdade. Copiar o nada em silencio deixaria o `arca prepare`
        // declarando um dispositivo pronto sem o binario do ARCA dentro.
        let conteudo = self.conteudo.borrow().get(origem).cloned().ok_or_else(|| {
            erro_de_arquivo("copia", origem)(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "nao existe neste sistema de arquivos de mentira",
            ))
        })?;

        self.conteudo
            .borrow_mut()
            .insert(destino.to_path_buf(), conteudo);
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

/// Um sistema de mentira: responde o que lhe ensinaram sobre a Inicializacao
/// Rapida e sobre o `chkdsk`.
///
/// O padrao — `HiberbootEnabled = 0` e `chkdsk` com codigo 0 — e o que esta
/// maquina respondeu em 22/08/2026: e o caso normal, e nao o caso conveniente.
pub struct SistemaDeMentira {
    pub inicializacao_rapida: Resultado<Option<u32>>,
    pub chkdsk: Resultado<SaidaDeFerramenta>,
    pub conferidos: RefCell<Vec<char>>,

    /// Quantas vezes mandaram reiniciar. E o unico jeito de um teste afirmar
    /// que o reinicio **nao** aconteceu — e a E7 tem mais casos em que ele nao
    /// deve acontecer do que casos em que deve.
    pub reinicios: Cell<usize>,

    /// A recusa do `shutdown`, quando se quer exercitar o caminho em que o
    /// reinicio falha com o dispositivo ja armado.
    pub recusa_ao_reiniciar: Option<Erro>,

    /// O que o `certutil` responde por arquivo, para V-1 e PR-1.
    ///
    /// A chave e o caminho **em minusculas**, porque quem abriria o arquivo e
    /// o Windows, onde `DISK` e `disk` sao o mesmo. Um caminho que ninguem
    /// ensinou cai em [`SistemaDeMentira::resumo_padrao`].
    pub resumos: RefCell<BTreeMap<String, SaidaDeFerramenta>>,

    /// O que responder por um caminho que nao foi ensinado.
    ///
    /// O padrao e a resposta do `certutil` para arquivo ausente, medida em
    /// 23/08/2026 — e nao um resumo plausivel. Um duplo que inventasse hash
    /// para arquivo que ninguem pôs na bancada faria um teste de verificacao
    /// passar sem que o arquivo existisse, que e o oposto do que V-1 confere.
    pub resumo_padrao: SaidaDeFerramenta,

    /// Os caminhos resumidos, na ordem. E o que permite a um teste afirmar
    /// **quais** arquivos foram lidos, e nao so quantos.
    pub resumidos: RefCell<Vec<PathBuf>>,

    /// O que o `curl` responde, e o **conteudo** que ele deixa no destino.
    ///
    /// O conteudo importa porque o passo seguinte de PR-1 e resumir o arquivo
    /// baixado: um duplo que baixasse o nada faria a conferencia de SHA256 cair
    /// no caminho de "arquivo ausente" em vez de exercitar o que ela existe
    /// para exercitar.
    pub download: Resultado<SaidaDeFerramenta>,
    pub conteudo_baixado: String,

    /// As URLs pedidas, na ordem. `arca prepare --iso` **nao** pode baixar
    /// nada, e este vetor e o que torna isso afirmavel.
    pub baixados: RefCell<Vec<String>>,

    /// O que o `bsdtar -x` responde, e os caminhos que ele "extrai".
    pub extracao: Resultado<SaidaDeFerramenta>,
    pub extraidos: RefCell<Vec<(PathBuf, PathBuf)>>,

    /// O que o `bsdtar -t` lista. O padrao e o pacote de verdade, abreviado —
    /// ver [`SistemaDeMentira::listando`].
    pub listagem: Resultado<SaidaDeFerramenta>,
}

impl Default for SistemaDeMentira {
    fn default() -> SistemaDeMentira {
        SistemaDeMentira {
            inicializacao_rapida: Ok(Some(0)),
            chkdsk: Ok(SaidaDeFerramenta {
                codigo: 0,
                texto: "Nao ha problemas no sistema de arquivos.\n".to_string(),
            }),
            conferidos: RefCell::new(Vec::new()),
            reinicios: Cell::new(0),
            recusa_ao_reiniciar: None,
            resumos: RefCell::new(BTreeMap::new()),
            resumo_padrao: SaidaDeFerramenta {
                codigo: -2147024894,
                texto: concat!(
                    "CertUtil: -hashfile comando FALHOU: 0x80070002",
                    " (WIN32: 2 ERROR_FILE_NOT_FOUND)\r\n",
                    "CertUtil: O sistema nao pode encontrar o arquivo especificado.\r\n"
                )
                .to_string(),
            },
            resumidos: RefCell::new(Vec::new()),
            download: Ok(SaidaDeFerramenta {
                codigo: 0,
                texto: String::new(),
            }),
            conteudo_baixado: "o pacote do Clonezilla, de mentira".to_string(),
            baixados: RefCell::new(Vec::new()),
            extracao: Ok(SaidaDeFerramenta {
                codigo: 0,
                texto: String::new(),
            }),
            extraidos: RefCell::new(Vec::new()),
            listagem: Ok(SaidaDeFerramenta {
                codigo: 0,
                texto: LISTAGEM_DO_PACOTE.to_string(),
            }),
        }
    }
}

/// A listagem do pacote de verdade, abreviada — extraída com o `bsdtar` do
/// `System32` em 23/08/2026. São 356 entradas no total; estas dez cobrem os
/// quatro caminhos obrigatórios de [`crate::pacote::CAMINHOS_OBRIGATORIOS`] e
/// a forma das outras.
const LISTAGEM_DO_PACOTE: &str = "\
.disk/info
Clonezilla-Live-Version
EFI/boot/bootx64.efi
boot/grub/grub.cfg
home/partimag/
live/filesystem.squashfs
live/initrd.img
live/vmlinuz
syslinux/isolinux.cfg
utils/
";

impl SistemaDeMentira {
    pub fn novo() -> SistemaDeMentira {
        SistemaDeMentira::default()
    }

    /// O que o `curl` responde e o que ele deixa no destino.
    pub fn baixando(mut self, codigo: i32, texto: &str, conteudo: &str) -> SistemaDeMentira {
        self.download = Ok(SaidaDeFerramenta {
            codigo,
            texto: texto.to_string(),
        });
        self.conteudo_baixado = conteudo.to_string();
        self
    }

    /// O que o `bsdtar -x` responde.
    pub fn extraindo(mut self, codigo: i32, texto: &str) -> SistemaDeMentira {
        self.extracao = Ok(SaidaDeFerramenta {
            codigo,
            texto: texto.to_string(),
        });
        self
    }

    /// O que o `bsdtar -t` lista dentro do pacote.
    pub fn listando(mut self, codigo: i32, texto: &str) -> SistemaDeMentira {
        self.listagem = Ok(SaidaDeFerramenta {
            codigo,
            texto: texto.to_string(),
        });
        self
    }

    /// O valor bruto do registro. `None` reproduz o valor ausente, que **nao**
    /// e o mesmo que desativada.
    pub fn com_inicializacao_rapida(mut self, valor: Option<u32>) -> SistemaDeMentira {
        self.inicializacao_rapida = Ok(valor);
        self
    }

    pub fn com_chkdsk(mut self, codigo: i32, texto: &str) -> SistemaDeMentira {
        self.chkdsk = Ok(SaidaDeFerramenta {
            codigo,
            texto: texto.to_string(),
        });
        self
    }

    pub fn recusando_o_reiniciar(mut self, recusa: Erro) -> SistemaDeMentira {
        self.recusa_ao_reiniciar = Some(recusa);
        self
    }

    /// Ensina o resumo de um arquivo, na forma em que o `certutil` responde.
    ///
    /// Monta as **tres linhas** medidas, e nao so o hash: o que esta sendo
    /// exercitado do outro lado e o leitor de [`crate::resumo::do_certutil`],
    /// que acha a linha pela forma no meio das duas frases traduzidas. Um
    /// duplo que devolvesse so o hash tiraria do teste justamente a parte que
    /// pode dar errado em producao.
    pub fn com_resumo(self, caminho: &str, digitos: &str) -> SistemaDeMentira {
        self.com_resposta_do_certutil(
            caminho,
            0,
            &format!(
                "MD5 hash de {caminho}:\r\n{digitos}\r\nCertUtil: -hashfile : comando concluido com exito.\r\n"
            ),
        )
    }

    /// Ensina uma resposta crua do `certutil`, para os casos que não são um
    /// resumo bem-sucedido.
    pub fn com_resposta_do_certutil(
        self,
        caminho: &str,
        codigo: i32,
        texto: &str,
    ) -> SistemaDeMentira {
        self.resumos.borrow_mut().insert(
            caminho.to_ascii_lowercase(),
            SaidaDeFerramenta {
                codigo,
                texto: texto.to_string(),
            },
        );
        self
    }

    /// Os caminhos que passaram pelo `certutil`, na ordem.
    pub fn resumidos(&self) -> Vec<PathBuf> {
        self.resumidos.borrow().clone()
    }

    /// Quantas vezes o reinicio foi pedido. Zero e a resposta esperada na
    /// maioria dos testes desta etapa.
    pub fn reinicios(&self) -> usize {
        self.reinicios.get()
    }
}

impl Sistema for SistemaDeMentira {
    fn inicializacao_rapida(&self) -> Resultado<Option<u32>> {
        match &self.inicializacao_rapida {
            Ok(valor) => Ok(*valor),
            Err(erro) => Err(clonar_a_recusa(erro)),
        }
    }

    fn conferir_volume(&self, letra: char) -> Resultado<SaidaDeFerramenta> {
        self.conferidos.borrow_mut().push(letra);
        match &self.chkdsk {
            Ok(saida) => Ok(saida.clone()),
            Err(erro) => Err(clonar_a_recusa(erro)),
        }
    }

    fn resumir(
        &self,
        caminho: &Path,
        algoritmo: crate::resumo::Algoritmo,
    ) -> Resultado<SaidaDeFerramenta> {
        self.resumidos.borrow_mut().push(caminho.to_path_buf());

        let chave = caminho.to_string_lossy().to_ascii_lowercase();
        let saida = self
            .resumos
            .borrow()
            .get(&chave)
            .cloned()
            .unwrap_or_else(|| self.resumo_padrao.clone());

        // O algoritmo entra no texto da primeira linha, como o `certutil`
        // faz. Sem isso, um teste de SHA256 leria `MD5 hash de ...` e o duplo
        // estaria mentindo sobre qual pergunta foi feita.
        Ok(SaidaDeFerramenta {
            texto: saida
                .texto
                .replace("MD5 hash de", &format!("{} hash de", algoritmo.nome())),
            ..saida
        })
    }

    fn baixar(&self, url: &str, _destino: &Path) -> Resultado<SaidaDeFerramenta> {
        // Registra **antes** de decidir se recusa, pelo mesmo motivo do
        // `reiniciar`: um download que falhou tambem foi tentado, e `arca
        // prepare --iso` nao pode ter tentado nenhum.
        self.baixados.borrow_mut().push(url.to_string());
        match &self.download {
            Ok(saida) => Ok(saida.clone()),
            Err(erro) => Err(clonar_a_recusa(erro)),
        }
    }

    fn extrair(&self, pacote: &Path, destino: &Path) -> Resultado<SaidaDeFerramenta> {
        self.extraidos
            .borrow_mut()
            .push((pacote.to_path_buf(), destino.to_path_buf()));
        match &self.extracao {
            Ok(saida) => Ok(saida.clone()),
            Err(erro) => Err(clonar_a_recusa(erro)),
        }
    }

    fn listar_pacote(&self, _pacote: &Path) -> Resultado<SaidaDeFerramenta> {
        match &self.listagem {
            Ok(saida) => Ok(saida.clone()),
            Err(erro) => Err(clonar_a_recusa(erro)),
        }
    }

    fn reiniciar(&self) -> Resultado<()> {
        // Conta antes de decidir se recusa: um `shutdown` que falhou tambem
        // **foi chamado**, e um teste que confunde "nao chamou" com "chamou e
        // nao deu" nao distingue os dois estados que importam aqui.
        self.reinicios.set(self.reinicios.get() + 1);
        match &self.recusa_ao_reiniciar {
            Some(erro) => Err(clonar_a_recusa(erro)),
            None => Ok(()),
        }
    }
}

// ─────────────────────── o particionador de mentira ───────────────────────

/// Um particionador que responde o que lhe ensinaram e **registra o que lhe
/// mandaram fazer**.
///
/// O registro é o ponto: este é o único duplo do projeto cuja operação de
/// verdade apaga um disco, e o que a maioria dos testes precisa afirmar é que
/// ela **não** foi chamada. Um duplo que só respondesse não distinguiria
/// "recusou antes de escrever" de "escreveu e deu certo".
pub struct ParticionadorDeMentira {
    pub discos: Vec<crate::portas::particionador::DiscoParaPreparar>,

    /// Os planos executados, na ordem. Vazio é o que a maioria dos testes
    /// desta etapa espera.
    pub particionados: RefCell<Vec<crate::portas::particionador::PlanoDeParticoes>>,

    /// O que a releitura responde depois de particionar.
    pub saida: Resultado<crate::portas::particionador::ParticoesFeitas>,

    /// Quantas vezes descreveram um disco. É o que permite provar que o
    /// terceiro tempo de PR-4 releu antes de agir, em vez de reusar a leitura
    /// que imprimiu o plano.
    pub descricoes: Cell<usize>,
}

impl ParticionadorDeMentira {
    /// Os três discos desta mesa em 23/08/2026, com o segundo dispositivo
    /// ainda intacto.
    pub fn desta_mesa() -> ParticionadorDeMentira {
        ParticionadorDeMentira {
            discos: discos_para_preparar_desta_mesa(),
            particionados: RefCell::new(Vec::new()),
            saida: Ok(o_que_o_particionamento_deixou()),
            descricoes: Cell::new(0),
        }
    }

    pub fn com_discos(
        discos: Vec<crate::portas::particionador::DiscoParaPreparar>,
    ) -> ParticionadorDeMentira {
        ParticionadorDeMentira {
            discos,
            particionados: RefCell::new(Vec::new()),
            saida: Ok(o_que_o_particionamento_deixou()),
            descricoes: Cell::new(0),
        }
    }

    /// O que a releitura responde. Serve para exercitar a conferência de
    /// [`crate::preparacao::conferir_o_que_saiu`] pelo caminho da divergência.
    pub fn devolvendo(
        mut self,
        saida: Resultado<crate::portas::particionador::ParticoesFeitas>,
    ) -> ParticionadorDeMentira {
        self.saida = saida;
        self
    }

    /// Se o disco foi apagado. A pergunta que a maioria dos testes faz.
    pub fn particionou(&self) -> bool {
        !self.particionados.borrow().is_empty()
    }
}

impl crate::portas::Particionador for ParticionadorDeMentira {
    fn descrever(
        &self,
        indice: u32,
    ) -> Resultado<Option<crate::portas::particionador::DiscoParaPreparar>> {
        self.descricoes.set(self.descricoes.get() + 1);
        Ok(self
            .discos
            .iter()
            .find(|disco| disco.indice == indice)
            .cloned())
    }

    fn enumerar(&self) -> Resultado<Vec<crate::portas::particionador::DiscoParaPreparar>> {
        Ok(self.discos.clone())
    }

    fn particionar(
        &self,
        plano: &crate::portas::particionador::PlanoDeParticoes,
    ) -> Resultado<crate::portas::particionador::ParticoesFeitas> {
        self.particionados.borrow_mut().push(plano.clone());
        match &self.saida {
            Ok(saida) => Ok(saida.clone()),
            Err(erro) => Err(clonar_a_recusa(erro)),
        }
    }
}

/// Os três discos desta mesa em 23/08/2026, como o `arca prepare` os vê.
///
/// Números de verdade: o `JMicron Generic` de 447 GB que a E10 destrói de
/// propósito, o dispositivo já preparado, e o `KINGSTON` do Windows. E os dois
/// modelos de cada disco são **diferentes de propósito** onde eles diferem de
/// verdade — o `MSFT_Disk` diz `JMicron Generic` e o WMI diz `JMicron Generic
/// SCSI Disk Device`.
pub fn discos_para_preparar_desta_mesa() -> Vec<crate::portas::particionador::DiscoParaPreparar> {
    use crate::portas::particionador::{DiscoParaPreparar, ParticaoExistente};

    vec![
        DiscoParaPreparar {
            indice: 0,
            modelo: "KINGSTON SNV3S500G".to_string(),
            modelo_no_wmi: Some("KINGSTON SNV3S500G".to_string()),
            tamanho_bytes: 500_107_862_016,
            barramento: "NVMe".to_string(),
            tipo_de_midia: TipoDeMidia::DiscoFixo,
            estilo_de_particao: "GPT".to_string(),
            e_do_sistema: true,
            e_de_boot: true,
            somente_leitura: false,
            particoes: vec![ParticaoExistente {
                numero: 3,
                letra: Some('C'),
                rotulo: Some("Windows".to_string()),
                sistema_de_arquivos: Some("NTFS".to_string()),
                tamanho_bytes: 498_701_697_024,
            }],
        },
        DiscoParaPreparar {
            indice: 1,
            modelo: "JMicron Generic".to_string(),
            modelo_no_wmi: Some("JMicron Generic SCSI Disk Device".to_string()),
            tamanho_bytes: 480_103_981_056,
            barramento: "USB".to_string(),
            tipo_de_midia: TipoDeMidia::DiscoExterno,
            estilo_de_particao: "MBR".to_string(),
            e_do_sistema: false,
            e_de_boot: false,
            somente_leitura: false,
            particoes: vec![ParticaoExistente {
                numero: 1,
                letra: Some('E'),
                rotulo: Some("Dell Beta Apps NO IA WSL".to_string()),
                sistema_de_arquivos: Some("NTFS".to_string()),
                tamanho_bytes: 480_099_958_784,
            }],
        },
        DiscoParaPreparar {
            indice: 2,
            modelo: "KGSSE100 256".to_string(),
            modelo_no_wmi: Some("KGSSE100 256 SCSI Disk Device".to_string()),
            tamanho_bytes: 256_060_514_304,
            barramento: "USB".to_string(),
            tipo_de_midia: TipoDeMidia::DiscoExterno,
            estilo_de_particao: "MBR".to_string(),
            e_do_sistema: false,
            e_de_boot: false,
            somente_leitura: false,
            particoes: vec![
                ParticaoExistente {
                    numero: 1,
                    letra: Some('D'),
                    rotulo: Some("ARCAVAULT".to_string()),
                    sistema_de_arquivos: Some("NTFS".to_string()),
                    tamanho_bytes: 254_379_294_720,
                },
                ParticaoExistente {
                    numero: 2,
                    letra: Some('R'),
                    rotulo: Some("ARCABOOT".to_string()),
                    sistema_de_arquivos: Some("FAT32".to_string()),
                    tamanho_bytes: 1_677_721_600,
                },
            ],
        },
    ]
}

/// O que o Windows respondeu depois do particionamento à mão de 23/08/2026.
pub fn o_que_o_particionamento_deixou() -> crate::portas::particionador::ParticoesFeitas {
    use crate::portas::particionador::{ParticaoFeita, ParticoesFeitas};

    ParticoesFeitas {
        vault: ParticaoFeita {
            numero: 1,
            letra: 'E',
            rotulo: "ARCAVAULT".to_string(),
            sistema_de_arquivos: "NTFS".to_string(),
            tipo_mbr: 7,
            tamanho_bytes: 478_423_285_760,
            offset_bytes: 1_048_576,
            unidade_de_alocacao: 4096,
            ativa: false,
        },
        boot: ParticaoFeita {
            numero: 2,
            letra: 'F',
            rotulo: "ARCABOOT".to_string(),
            sistema_de_arquivos: "FAT32".to_string(),
            tipo_mbr: 12,
            tamanho_bytes: 1_677_721_600,
            offset_bytes: 478_424_334_336,
            unidade_de_alocacao: 4096,
            ativa: false,
        },
    }
}

/// Um console que responde o que lhe ensinaram, uma linha por chamada.
///
/// Existe para S-2 ter teste: a confirmacao digitada e o que separa "armou" de
/// "nao armou", e um requisito de seguranca sem teste e uma frase. Esgotadas
/// as respostas, devolve linha vazia — que e o que o `stdin` fechado faz, e o
/// que nunca confirma nada.
pub struct ConsoleDeMentira {
    respostas: RefCell<std::collections::VecDeque<String>>,
    pub lidas: Cell<usize>,
}

impl ConsoleDeMentira {
    pub fn respondendo(linhas: &[&str]) -> ConsoleDeMentira {
        ConsoleDeMentira {
            respostas: RefCell::new(linhas.iter().map(|linha| linha.to_string()).collect()),
            lidas: Cell::new(0),
        }
    }

    /// Um console em que ninguem digitou nada.
    pub fn mudo() -> ConsoleDeMentira {
        ConsoleDeMentira::respondendo(&[])
    }
}

impl Console for ConsoleDeMentira {
    fn ler_linha(&self) -> Resultado<String> {
        self.lidas.set(self.lidas.get() + 1);
        Ok(self.respostas.borrow_mut().pop_front().unwrap_or_default())
    }
}

/// Um sistema de arquivos em que um caminho **existe e nao se deixa lê**.
///
/// Existe por causa de um achado da revisao da etapa E5: a diferenca entre
/// "nao esta la" e "nao consegui olhar" nao tem como ser testada com um
/// duplo que so sabe ter ou nao ter o arquivo. Sem ele, o codigo que confunde
/// os dois casos passa em todo teste — e foi o que aconteceu.
///
/// Delega tudo a um [`ArquivosEmMemoria`], menos o caminho recusado.
pub struct ArquivosQueRecusam {
    dentro: ArquivosEmMemoria,
    recusado: PathBuf,
    especie: std::io::ErrorKind,
    mensagem: String,
}

impl ArquivosQueRecusam {
    pub fn com(
        recusado: impl Into<PathBuf>,
        especie: std::io::ErrorKind,
        mensagem: &str,
    ) -> ArquivosQueRecusam {
        ArquivosQueRecusam {
            dentro: ArquivosEmMemoria::novo(),
            recusado: recusado.into(),
            especie,
            mensagem: mensagem.to_string(),
        }
    }

    /// Um arquivo que se lê normalmente, ao lado do recusado.
    pub fn com_arquivo(self, caminho: impl Into<PathBuf>, conteudo: &str) -> ArquivosQueRecusam {
        ArquivosQueRecusam {
            dentro: self.dentro.com(caminho, conteudo),
            ..self
        }
    }

    fn recusa(&self, caminho: &Path) -> Option<Erro> {
        (caminho == self.recusado).then(|| {
            erro_de_arquivo("leitura", caminho)(std::io::Error::new(
                self.especie,
                self.mensagem.clone(),
            ))
        })
    }
}

impl Arquivos for ArquivosQueRecusam {
    /// **`true` para o caminho recusado**, que e o ponto do duplo: um arquivo
    /// que esta la. Um `Path::exists` de verdade diria `false` aqui, e e essa
    /// mentira que o duplo existe para nao reproduzir.
    fn existe(&self, caminho: &Path) -> bool {
        caminho == self.recusado || self.dentro.existe(caminho)
    }

    fn ler_texto(&self, caminho: &Path) -> Resultado<String> {
        match self.recusa(caminho) {
            Some(erro) => Err(erro),
            None => self.dentro.ler_texto(caminho),
        }
    }

    fn ler_texto_alheio(&self, caminho: &Path) -> Resultado<String> {
        match self.recusa(caminho) {
            Some(erro) => Err(erro),
            None => self.dentro.ler_texto_alheio(caminho),
        }
    }

    fn escrever_atomico(&self, caminho: &Path, conteudo: &str) -> Resultado<()> {
        self.dentro.escrever_atomico(caminho, conteudo)
    }

    fn criar_diretorio(&self, caminho: &Path) -> Resultado<()> {
        self.dentro.criar_diretorio(caminho)
    }

    fn listar(&self, caminho: &Path) -> Resultado<Vec<Entrada>> {
        self.dentro.listar(caminho)
    }

    /// A recusa vale para a **origem**, que e o lado que se lê.
    fn copiar(&self, origem: &Path, destino: &Path) -> Resultado<()> {
        match self.recusa(origem) {
            Some(erro) => Err(erro),
            None => self.dentro.copiar(origem, destino),
        }
    }

    fn espaco_livre(&self, caminho: &Path) -> Resultado<u64> {
        self.dentro.espaco_livre(caminho)
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
