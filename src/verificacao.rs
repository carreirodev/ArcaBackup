//! Conferir uma imagem contra o `MD5SUMS` dela, no Windows e sem reiniciar
//! (V-1).
//!
//! # O que esta verificacao responde, e o que ela nao responde
//!
//! Ela responde **"os bytes que estao no dispositivo sao os que o Clonezilla
//! gravou?"** — corrupcao de midia, copia truncada, arquivo que sumiu. Pega o
//! setor que degradou no `ARCAVAULT` depois do backup.
//!
//! Ela **nao** responde "esta imagem e restauravel?". Quem responde isso e o
//! `ocs-chkimg`, que descomprime cada particao e olha o que sai — e por isso
//! V-2 existe ao lado e **nao substitui B-9**, que continua obrigatoria em
//! todo backup. Um `.zst` intacto byte a byte que carregue dentro de si um
//! NTFS inconsistente passa por aqui e reprova la.
//!
//! As duas foram medidas em 23/08/2026, sobre a mesma imagem de 39,7 GB:
//!
//! | | le | tempo | reinicios |
//! |---|---|---|---|
//! | V-1, este modulo | os 39 MD5 do `MD5SUMS` | **3 min 23 s** | 0 |
//! | V-2, `--completo` | `ocs-chkimg` descomprime | **5 min 12 s** | 1 |
//!
//! O reinicio e o que separa as duas na pratica, e nao os dois minutos: V-2
//! desliga a maquina, e quem esta trabalhando nela para de trabalhar.
//!
//! # As quatro coisas que se pode achar, e a diferenca entre as duas ultimas
//!
//! [`Achado`] separa "o arquivo nao esta la" de "o arquivo esta la e nao se
//! deixou lê", e a distincao e a mesma que
//! [`crate::desfecho::Encontrado::NaoDeuParaLer`] paga desde a E5: **"nao
//! consegui olhar" nunca vira "nao ha nada la"**. Um `ARCAVAULT` que negou
//! acesso a um arquivo e uma imagem sobre a qual nao se sabe; um arquivo
//! ausente e uma imagem quebrada. As duas reprovam, e quem lê precisa saber
//! qual das duas tem na mao.
//!
//! # S-5: falha parcial e falha total
//!
//! Um arquivo que nao bate reprova a imagem inteira, e a tela mostra **todos**
//! os que nao bateram, e nao o primeiro. Parar no primeiro faria a segunda
//! execucao descobrir o segundo, e assim por diante — e quem tem uma imagem
//! com dois arquivos ruins precisa saber disso de uma vez.

use crate::erro::Resultado;
use crate::imagens::Veredito;
use crate::md5sums::Entrada;
use crate::portas::{Arquivos, Sistema};
use crate::resumo::{self, Algoritmo, RecusaDoResumo, Resumo};
use std::fmt;
use std::path::Path;

/// O que se achou sobre um arquivo listado no `MD5SUMS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Achado {
    /// O resumo do arquivo e o que o `MD5SUMS` registra.
    Bate,

    /// O arquivo esta la e os bytes mudaram.
    NaoBate {
        esperado: Resumo,
        encontrado: Resumo,
    },

    /// O `MD5SUMS` o lista e ele nao esta na pasta.
    Ausente,

    /// Ele esta la e nao se deixou resumir. **Nao e o mesmo que ausente.**
    NaoDeuParaResumir { motivo: RecusaDoResumo },
}

impl Achado {
    pub fn bate(&self) -> bool {
        matches!(self, Achado::Bate)
    }
}

impl fmt::Display for Achado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Achado::Bate => write!(f, "ok"),
            Achado::NaoBate {
                esperado,
                encontrado,
            } => write!(
                f,
                "NAO BATE · o MD5SUMS diz {} e o arquivo soma {}",
                esperado.abreviado(),
                encontrado.abreviado()
            ),
            Achado::Ausente => write!(
                f,
                "AUSENTE · o MD5SUMS o lista e ele nao esta na pasta da imagem"
            ),
            Achado::NaoDeuParaResumir { motivo } => write!(
                f,
                "NAO DEU PARA LER · o arquivo esta la e nao se deixou resumir ({motivo}). Isto NAO e o mesmo que ele estar ausente"
            ),
        }
    }
}

/// Um arquivo conferido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conferido {
    pub arquivo: String,
    pub achado: Achado,

    /// Quanto o arquivo ocupa, do lado Windows. Zero quando ele nao esta la.
    pub bytes: u64,
}

/// O que se vai conferir, medido antes de comecar.
///
/// # Por que isto existe separado da conferencia
///
/// Duas razoes, e as duas apareceram **rodando o comando de verdade**, com a
/// suite verde — que e como a E6, a E7, a E9 e a E10 acharam os defeitos delas.
///
/// A primeira: a tela precisa dizer **quanto vai demorar** antes de ficar tres
/// minutos parada, e para isso precisa saber quantos bytes vai lê. Sem o plano,
/// a unica coisa que ela podia dizer era o tempo da imagem desta mesa — o que
/// prometeria `3 min 23 s` para uma imagem de 1 GB.
///
/// A segunda: a coluna das linhas de andamento tem de caber no **maior nome da
/// lista**, e so quem tem a lista inteira sabe qual e. Com a coluna fixa de
/// [`crate::formato::linha`], quatorze das trinta e nove linhas estouravam —
/// os nomes que o Clonezilla da aos pedacos de uma particao sao longos por
/// construcao, e o caso que aquela funcao trata como excepcional e aqui o caso
/// normal.
///
/// E ha um ganho de graca: a listagem do sistema de arquivos acontece **uma
/// vez**, aqui, em vez de uma vez por arquivo no meio de uma operacao que ja
/// lê dezenas de gigabytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plano {
    /// O que o `MD5SUMS` lista, e o tamanho de cada um do lado Windows. Zero
    /// para o que nao esta la.
    pub arquivos: Vec<(Entrada, u64)>,

    pub bytes_totais: u64,

    /// Quantos arquivos a pasta tem e o `MD5SUMS` nao lista. Ver
    /// [`Conferencia::fora_do_md5sums`].
    pub fora_do_md5sums: usize,
}

impl Plano {
    pub fn quantos(&self) -> usize {
        self.arquivos.len()
    }

    /// A largura do maior nome, para a coluna do andamento.
    pub fn largura_do_nome(&self) -> usize {
        self.arquivos
            .iter()
            .map(|(entrada, _)| entrada.arquivo.chars().count())
            .max()
            .unwrap_or(0)
    }
}

/// A taxa de leitura medida neste dispositivo, em bytes por segundo.
///
/// Medido em 23/08/2026 sobre a `2026-08-22_Apps`: 42.604.877.207 bytes em
/// 202,6 s, o que da 200,5 MB/s. Um arquivo sozinho de 812 MB deu 202,2 MB/s
/// — as duas taxas sao a mesma dentro do ruido, e quem manda e o USB.
///
/// Serve para **estimar** e nada mais. A tela diz de onde o numero veio, e um
/// dispositivo diferente vai dar outro — o que a estimativa evita e a tela
/// prometer o tempo desta imagem para qualquer imagem.
const BYTES_POR_SEGUNDO: u64 = 200 * 1024 * 1024;

/// Quanto a conferencia deste plano deve levar, pela taxa medida.
pub fn estimar(bytes: u64) -> std::time::Duration {
    std::time::Duration::from_secs(bytes / BYTES_POR_SEGUNDO)
}

/// O andamento, para quem quiser imprimir enquanto os minutos passam.
///
/// A conferencia da imagem desta mesa leva 3 min 23 s, e uma tela parada
/// durante tres minutos e indistinguivel de um comando travado. Quem imprime e
/// o comando; este modulo so avisa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Andamento<'a> {
    /// De 1 ate `total`.
    pub numero: usize,
    pub total: usize,
    pub arquivo: &'a str,
    pub conferido: &'a Conferido,

    /// A largura do maior nome do plano, para a coluna nao dancar.
    pub largura_do_nome: usize,

    /// Quantos bytes ja foram lidos, contando este arquivo.
    pub bytes_lidos: u64,
    pub bytes_totais: u64,
}

/// O resultado da conferencia inteira.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conferencia {
    pub conferidos: Vec<Conferido>,
    pub bytes_lidos: u64,

    /// Quantos arquivos a pasta tem e o `MD5SUMS` nao lista.
    ///
    /// **Nunca e falha**, e a contagem esta aqui para que isso fique dito em
    /// codigo e nao so em comentario. Medido na `2026-08-22_Apps`: sao quatro
    /// — o proprio `MD5SUMS`, o `clonezilla-img` e o `Info-img-id.txt`, que
    /// estavam abertos no segundo em que ele foi escrito, e o `arca-check.log`
    /// de B-9, que so existe cinco minutos depois. E a hora em que cada um
    /// nasceu, e nao falta de nada.
    pub fora_do_md5sums: usize,
}

impl Conferencia {
    /// Os que nao bateram, na ordem do `MD5SUMS`.
    pub fn falhas(&self) -> Vec<&Conferido> {
        self.conferidos
            .iter()
            .filter(|conferido| !conferido.achado.bate())
            .collect()
    }

    /// O veredito da imagem (S-5: qualquer falha reprova).
    ///
    /// Devolve o mesmo tipo que [`crate::imagens::Veredito`] de proposito. E o
    /// mesmo julgamento sobre a mesma coisa, por outro caminho, e dois tipos
    /// diferentes convidariam a tratar como diferentes duas respostas que
    /// significam o mesmo para quem lê.
    pub fn veredito(&self) -> Veredito {
        if self
            .conferidos
            .iter()
            .all(|conferido| conferido.achado.bate())
        {
            Veredito::Aprovada
        } else {
            Veredito::Reprovada
        }
    }

    pub fn quantos(&self) -> usize {
        self.conferidos.len()
    }
}

/// Mede o que se vai conferir, com **uma** ida ao sistema de arquivos.
///
/// Ver [`Plano`] para as duas coisas que a tela precisa saber antes de a
/// conferencia comecar.
pub fn planejar(
    arquivos: &dyn Arquivos,
    pasta_da_imagem: &Path,
    entradas: &[Entrada],
) -> Resultado<Plano> {
    let na_pasta = arquivos.listar(pasta_da_imagem)?;

    // Sem diferenciar caixa, do mesmo jeito que `Arquivos::existe` acha o
    // arquivo: quem abre e o Windows, onde `DISK` e `disk` sao o mesmo. Achar
    // o tamanho por um criterio e a existencia por outro faria um arquivo
    // aparecer com zero byte e contar como "fora do MD5SUMS" ao mesmo tempo.
    let tamanho_de = |procurado: &str| -> u64 {
        na_pasta
            .iter()
            .find(|item| !item.diretorio && item.nome().eq_ignore_ascii_case(procurado))
            .map_or(0, |item| item.tamanho_bytes)
    };

    let listados: Vec<(Entrada, u64)> = entradas
        .iter()
        .map(|entrada| (entrada.clone(), tamanho_de(&entrada.arquivo)))
        .collect();

    let fora_do_md5sums = na_pasta
        .iter()
        .filter(|item| !item.diretorio)
        .filter(|item| {
            !entradas
                .iter()
                .any(|entrada| entrada.arquivo.eq_ignore_ascii_case(&item.nome()))
        })
        .count();

    Ok(Plano {
        bytes_totais: listados.iter().map(|(_, bytes)| bytes).sum(),
        arquivos: listados,
        fora_do_md5sums,
    })
}

/// Confere cada arquivo do `MD5SUMS` contra o que esta na pasta.
///
/// Nao para no primeiro que falha (S-5, e ver o cabecalho). `avisar` e
/// chamado depois de **cada** arquivo, para quem quiser mostrar andamento.
pub fn conferir(
    arquivos: &dyn Arquivos,
    sistema: &dyn Sistema,
    pasta_da_imagem: &Path,
    plano: &Plano,
    avisar: &mut dyn FnMut(&Andamento),
) -> Resultado<Conferencia> {
    let listados = plano.quantos();
    let largura_do_nome = plano.largura_do_nome();

    let mut conferidos = Vec::with_capacity(listados);
    let mut bytes_lidos = 0u64;

    for (indice, (entrada, bytes)) in plano.arquivos.iter().enumerate() {
        let caminho = pasta_da_imagem.join(&entrada.arquivo);

        // A ausencia se pergunta ao sistema de arquivos, e nao ao `certutil`.
        // Ele responde `0x80070002` para arquivo que nao existe, e cair nesse
        // ramo faria "nao esta la" chegar como "nao consegui resumir" — que e
        // exatamente a distincao que este modulo existe para manter.
        let achado = if !arquivos.existe(&caminho) {
            Achado::Ausente
        } else {
            match resumo::do_certutil(&sistema.resumir(&caminho, Algoritmo::Md5)?, Algoritmo::Md5) {
                Ok(encontrado) if encontrado == entrada.soma => Achado::Bate,
                Ok(encontrado) => Achado::NaoBate {
                    esperado: entrada.soma.clone(),
                    encontrado,
                },
                Err(motivo) => Achado::NaoDeuParaResumir { motivo },
            }
        };

        bytes_lidos += bytes;

        let conferido = Conferido {
            arquivo: entrada.arquivo.clone(),
            achado,
            bytes: *bytes,
        };

        avisar(&Andamento {
            numero: indice + 1,
            total: listados,
            arquivo: &entrada.arquivo,
            conferido: &conferido,
            largura_do_nome,
            bytes_lidos,
            bytes_totais: plano.bytes_totais,
        });

        conferidos.push(conferido);
    }

    Ok(Conferencia {
        conferidos,
        bytes_lidos,
        fora_do_md5sums: plano.fora_do_md5sums,
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{ArquivosEmMemoria, SistemaDeMentira};
    use crate::md5sums;
    use std::path::PathBuf;

    const A: &str = "bf6850d736dc6b480994de0cee9c0f63";
    const B: &str = "cee6e84e46cf5e1971efb6aac331eb18";

    fn pasta() -> PathBuf {
        PathBuf::from(r"D:\2026-08-22_Apps")
    }

    fn entradas(texto: &str) -> Vec<Entrada> {
        md5sums::ler(texto).expect("MD5SUMS de teste valido")
    }

    /// Roda o plano e a conferencia, e joga fora o andamento.
    fn conferir_sem_avisar(
        arquivos: &ArquivosEmMemoria,
        sistema: &SistemaDeMentira,
        lista: &[Entrada],
    ) -> Conferencia {
        let plano = planejar(arquivos, &pasta(), lista).expect("plano");
        conferir(arquivos, sistema, &pasta(), &plano, &mut |_| {}).expect("conferencia")
    }

    #[test]
    fn tudo_batendo_aprova() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "nvme0n1")
            .com(r"D:\2026-08-22_Apps\parts", "p1 p2");
        let sistema = SistemaDeMentira::novo()
            .com_resumo(r"D:\2026-08-22_Apps\disk", A)
            .com_resumo(r"D:\2026-08-22_Apps\parts", B);

        let conferencia = conferir_sem_avisar(
            &arquivos,
            &sistema,
            &entradas(&format!("{A}  disk\n{B}  parts\n")),
        );

        assert_eq!(conferencia.veredito(), Veredito::Aprovada);
        assert_eq!(conferencia.quantos(), 2);
        assert!(conferencia.falhas().is_empty());
    }

    #[test]
    fn um_arquivo_que_nao_bate_reprova_a_imagem_inteira() {
        // S-5: falha parcial e falha total.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "nvme0n1")
            .com(r"D:\2026-08-22_Apps\parts", "p1 p2");
        let sistema = SistemaDeMentira::novo()
            .com_resumo(r"D:\2026-08-22_Apps\disk", A)
            .com_resumo(r"D:\2026-08-22_Apps\parts", A); // devia ser B

        let conferencia = conferir_sem_avisar(
            &arquivos,
            &sistema,
            &entradas(&format!("{A}  disk\n{B}  parts\n")),
        );

        assert_eq!(conferencia.veredito(), Veredito::Reprovada);
        assert_eq!(conferencia.falhas().len(), 1);
        assert_eq!(conferencia.falhas()[0].arquivo, "parts");
        assert!(matches!(
            conferencia.falhas()[0].achado,
            Achado::NaoBate { .. }
        ));
    }

    #[test]
    fn a_conferencia_nao_para_no_primeiro_que_falha() {
        // Parar no primeiro faria a segunda execucao descobrir o segundo. Quem
        // tem dois arquivos ruins precisa saber dos dois de uma vez.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "x")
            .com(r"D:\2026-08-22_Apps\parts", "y");
        let sistema = SistemaDeMentira::novo()
            .com_resumo(r"D:\2026-08-22_Apps\disk", B)
            .com_resumo(r"D:\2026-08-22_Apps\parts", A);

        let conferencia = conferir_sem_avisar(
            &arquivos,
            &sistema,
            &entradas(&format!("{A}  disk\n{B}  parts\n")),
        );

        assert_eq!(conferencia.falhas().len(), 2);
        assert_eq!(conferencia.quantos(), 2, "os dois foram conferidos");
    }

    #[test]
    fn arquivo_ausente_nao_e_o_mesmo_que_nao_deu_para_ler() {
        // A distincao que a E5 pagou caro para existir, aplicada aqui. O
        // `certutil` responde `0x80070002` para arquivo ausente, e cair nesse
        // ramo faria as duas chegarem iguais.
        let arquivos = ArquivosEmMemoria::novo().com(r"D:\2026-08-22_Apps\parts", "y");
        let sistema = SistemaDeMentira::novo().com_resposta_do_certutil(
            r"D:\2026-08-22_Apps\parts",
            5,
            "CertUtil: -hashfile comando FALHOU: 0x80070005 (WIN32: 5 ERROR_ACCESS_DENIED)\r\n",
        );

        let conferencia = conferir_sem_avisar(
            &arquivos,
            &sistema,
            &entradas(&format!("{A}  disk\n{B}  parts\n")),
        );

        assert_eq!(conferencia.conferidos[0].achado, Achado::Ausente);
        assert!(matches!(
            conferencia.conferidos[1].achado,
            Achado::NaoDeuParaResumir { .. }
        ));
        assert_eq!(
            conferencia.veredito(),
            Veredito::Reprovada,
            "as duas reprovam"
        );
    }

    #[test]
    fn o_certutil_nao_e_chamado_para_arquivo_ausente() {
        // Quem responde sobre existencia e o sistema de arquivos. Chamar o
        // `certutil` para saber se um arquivo existe seria perguntar a coisa
        // errada — e, num arquivo de 4 GB que existe, seria caro.
        let arquivos = ArquivosEmMemoria::novo().com(r"D:\2026-08-22_Apps\parts", "y");
        let sistema = SistemaDeMentira::novo().com_resumo(r"D:\2026-08-22_Apps\parts", B);

        conferir_sem_avisar(
            &arquivos,
            &sistema,
            &entradas(&format!("{A}  disk\n{B}  parts\n")),
        );

        assert_eq!(
            sistema.resumidos(),
            vec![PathBuf::from(r"D:\2026-08-22_Apps\parts")],
            "o `disk` ausente nao devia ter ido ao certutil"
        );
    }

    #[test]
    fn os_arquivos_fora_do_md5sums_sao_contados_e_nao_reprovam() {
        // Medido na imagem de verdade: quatro ficam de fora por construcao —
        // o proprio `MD5SUMS`, o `clonezilla-img` e o `Info-img-id.txt`, que
        // estavam abertos quando ele foi escrito, e o `arca-check.log`, que so
        // existe cinco minutos depois. Chamar isso de falha reprovaria toda
        // imagem que o Clonezilla ja fez.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "nvme0n1")
            .com(r"D:\2026-08-22_Apps\MD5SUMS", "...")
            .com(r"D:\2026-08-22_Apps\clonezilla-img", "...")
            .com(r"D:\2026-08-22_Apps\Info-img-id.txt", "...")
            .com(r"D:\2026-08-22_Apps\arca-check.log", "...");
        let sistema = SistemaDeMentira::novo().com_resumo(r"D:\2026-08-22_Apps\disk", A);

        let conferencia =
            conferir_sem_avisar(&arquivos, &sistema, &entradas(&format!("{A}  disk\n")));

        assert_eq!(conferencia.veredito(), Veredito::Aprovada);
        assert_eq!(conferencia.fora_do_md5sums, 4);
    }

    #[test]
    fn o_andamento_conta_do_um_ate_o_total_e_soma_os_bytes() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "12345678")
            .com(r"D:\2026-08-22_Apps\parts", "1234567890");
        let sistema = SistemaDeMentira::novo()
            .com_resumo(r"D:\2026-08-22_Apps\disk", A)
            .com_resumo(r"D:\2026-08-22_Apps\parts", B);

        let lista = entradas(&format!("{A}  disk\n{B}  parts\n"));
        let plano = planejar(&arquivos, &pasta(), &lista).unwrap();

        let mut visto: Vec<(usize, usize, String, u64, u64)> = Vec::new();
        conferir(&arquivos, &sistema, &pasta(), &plano, &mut |andamento| {
            visto.push((
                andamento.numero,
                andamento.total,
                andamento.arquivo.to_string(),
                andamento.bytes_lidos,
                andamento.bytes_totais,
            ));
        })
        .unwrap();

        assert_eq!(
            visto,
            vec![
                (1, 2, "disk".to_string(), 8, 18),
                (2, 2, "parts".to_string(), 18, 18),
            ]
        );
    }

    #[test]
    fn a_pasta_que_nao_da_para_listar_e_erro_e_nao_imagem_aprovada() {
        // Uma pasta ilegivel nao pode virar "nenhum arquivo a conferir" e,
        // dai, "aprovada". E o mesmo raciocinio de
        // `imagens::enumerar`, que recusa uma raiz ilegivel em vez de devolver
        // lista vazia.
        let arquivos = ArquivosEmMemoria::novo();
        assert!(planejar(&arquivos, &pasta(), &entradas(&format!("{A}  disk\n"))).is_err());
    }

    #[test]
    fn a_largura_da_coluna_vem_do_maior_nome_da_lista() {
        // O defeito que a execucao real pegou com a suite verde: com a coluna
        // fixa de `formato::linha`, quatorze das trinta e nove linhas do
        // andamento estouravam, porque os nomes que o Clonezilla da aos
        // pedacos de uma particao sao longos por construcao. A coluna tem de
        // vir da lista, e nao de uma constante.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "x")
            .com(r"D:\2026-08-22_Apps\nvme0n1p3.ntfs-ptcl-img.zst.aa", "y");
        let lista = entradas(&format!("{A}  disk\n{B}  nvme0n1p3.ntfs-ptcl-img.zst.aa\n"));

        let plano = planejar(&arquivos, &pasta(), &lista).unwrap();
        assert_eq!(plano.largura_do_nome(), 30);
    }

    #[test]
    fn o_plano_mede_os_bytes_antes_de_conferir() {
        // E o que permite a tela dizer quanto vai demorar **antes** de ficar
        // tres minutos parada. Sem isto, a unica coisa que ela podia dizer era
        // o tempo da imagem desta mesa, para qualquer imagem.
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"D:\2026-08-22_Apps\disk", "12345678")
            .com(r"D:\2026-08-22_Apps\parts", "1234567890");

        let plano = planejar(
            &arquivos,
            &pasta(),
            &entradas(&format!("{A}  disk\n{B}  parts\n")),
        )
        .unwrap();

        assert_eq!(plano.bytes_totais, 18);
        assert_eq!(plano.quantos(), 2);
    }

    #[test]
    fn a_estimativa_sai_da_taxa_medida_neste_dispositivo() {
        // 42.604.877.207 bytes a 200 MB/s dao pouco mais de tres minutos, e o
        // comando levou 199,4 s de verdade em 23/08/2026. A estimativa nao
        // precisa acertar o segundo; ela precisa nao prometer o tempo desta
        // imagem para uma imagem de 1 GB.
        assert_eq!(estimar(42_604_877_207).as_secs(), 203);
        assert_eq!(estimar(1024 * 1024 * 1024).as_secs(), 5);
        assert_eq!(estimar(0).as_secs(), 0);
    }

    #[test]
    fn a_caixa_do_nome_nao_faz_um_arquivo_sumir() {
        // Quem abre e o Windows: `DISK` e `disk` sao o mesmo arquivo, e o
        // tamanho tem de ser achado do mesmo jeito que o `existe` acha.
        let arquivos = ArquivosEmMemoria::novo().com(r"D:\2026-08-22_Apps\DISK", "nvme0n1");
        let sistema = SistemaDeMentira::novo().com_resumo(r"D:\2026-08-22_Apps\disk", A);

        let conferencia =
            conferir_sem_avisar(&arquivos, &sistema, &entradas(&format!("{A}  disk\n")));

        assert_eq!(conferencia.conferidos[0].bytes, 7);
        assert_eq!(conferencia.fora_do_md5sums, 0);
    }
}
