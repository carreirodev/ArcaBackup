//! O que o Clonezilla deixou escrito, e a quem aquilo pertence (§5.5, C-11,
//! C-12, R-6).
//!
//! O `arca-fim.txt` e a unica coisa que atravessa o reinicio no sentido de
//! volta. Este modulo lê esse arquivo e o julga contra o selo do job pendente
//! — e nada mais: quem desarma, lê o veredito da imagem e imprime a §5.4 e a
//! etapa E8.
//!
//! # Por selo, e nunca por data
//!
//! O Clonezilla lê o RTC — hora local do Windows — como se fosse UTC e roda
//! 3 h adiantado, de forma permanente (P-7). Uma trava construida sobre
//! comparacao de datas ja reprovou um backup perfeito neste projeto. Nada
//! aqui olha para data nenhuma, e `tests/s6_o_tempo_nao_decide.rs` cobra isso
//! a cada build.
//!
//! # Toda forma de nao ter dado certo vem antes de toda forma de ter dado
//!
//! A ordem do julgamento nao e estetica. E a mesma licao que a revisao da E3
//! impos ao leitor do `arca-check.log`: as duas marcas cabem no mesmo arquivo,
//! e ler a boa primeiro transforma uma falha em exito. Aqui vale igual — a
//! receita **acrescenta** ao `arca-fim.txt` com `>>`, e um arquivo que
//! recebesse duas passadas ficaria com as duas marcas.
//!
//! # A linha que o §5.5 nao tinha: `arca-fim.txt` sem selo nenhum
//!
//! O unico `arca-fim.txt` que existe neste dispositivo tem exatamente duas
//! linhas — `ARCA_RESTORE=OK` e `ARCA_FIM` — e **nenhum `ARCA_SELO=`**. E o
//! P-16 outra vez: ele veio do trabalho manual de validacao, e nao de receita
//! nenhuma. A tabela do §5.5 nao tinha linha para ele: tem "selo nao bate" e
//! tem "sem `ARCA_FIM`", e este arquivo nao e nem um nem outro.
//!
//! Dizer "o selo nao bate" seria mentira, porque nao ha selo a bater. Dai a
//! linha propria — e ela **vale codigo**, por duas razoes medidas:
//!
//! - **Aquele arquivo e inalcancavel hoje**, e isso e verdade: a E3 decidiu
//!   que a pasta do log leva a operacao no nome (`restauracao-<nome>`), e ele
//!   esta em `ARCA-LOGS\2026-08-21_WindowsCompleto\`. O ARCA de hoje nunca vai
//!   olhar para la.
//! - **Mas "sem selo" e alcancavel por outro caminho.** Toda receita comeca
//!   com `echo ARCA_SELO=... > arca-fim.txt`, e o `>` **trunca ao abrir**,
//!   antes de o `echo` rodar. Medido: um redirecionamento que abre e nao
//!   escreve deixa o arquivo em zero byte. Um desligamento nessa janela deixa
//!   exatamente um `arca-fim.txt` sem linha de selo — o caso que §4.3 diz que
//!   o selo cobre, com o selo sendo justamente o que foi cortado.
//!
//! Sem esta linha, esse arquivo cairia no ramo que o codigo tomasse por
//! descuido, e o ramo natural — comparar o selo achado com o esperado —
//! produziria a mensagem que este modulo existe para nao dar.

use crate::nome::Nome;
use crate::receita::{MARCA_DO_FIM, MARCA_DO_SELO, Operacao, Selo};
use std::fmt;

/// O que o `arca-fim.txt` diz, antes de ser julgado contra job nenhum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arquivo {
    /// O selo da primeira linha, quando ha um que se reconheca.
    pub selo: Option<Selo>,
    /// Quantas linhas comecam por `ARCA_SELO=`. Mais de uma e recusa.
    pub linhas_de_selo: usize,
    /// Se apareceu algum `ARCA_<algo>=FALHOU`.
    pub falhou: bool,
    /// Se apareceu algum `ARCA_<algo>=OK`.
    pub deu_certo: bool,
    /// Se a linha `ARCA_FIM` esta la.
    pub fim: bool,
}

/// Lê o `arca-fim.txt` sem julgar de quem ele e.
///
/// O selo sai da **primeira linha, e so dela**. A receita o escreve com `>`,
/// que trunca: num arquivo que a receita produziu ele nao pode estar em outro
/// lugar. Aceitar um selo achado no meio faria um arquivo com rastro de dois
/// jobs passar pelo segundo deles.
pub fn ler(texto: &str) -> Arquivo {
    let mut linhas = texto.lines().map(str::trim);

    let primeira = linhas.next().unwrap_or_default();
    let selo = primeira
        .strip_prefix(MARCA_DO_SELO)
        .and_then(|bruto| Selo::novo(bruto).ok());

    let mut arquivo = Arquivo {
        selo,
        linhas_de_selo: 0,
        falhou: false,
        deu_certo: false,
        fim: false,
    };

    for linha in texto.lines().map(str::trim) {
        if linha.starts_with(MARCA_DO_SELO) {
            arquivo.linhas_de_selo += 1;
        }
        if linha == MARCA_DO_FIM {
            arquivo.fim = true;
        }
        // Por sufixo, e nao pelo marcador inteiro: o `ARCA_BACKUP` e o
        // `ARCA_RESTORE` sao os dois de hoje, e um terceiro amanha continuaria
        // a ser lido. O que decide e o desfecho, nao qual operacao o escreveu.
        if let Some(resto) = linha.strip_prefix("ARCA_") {
            match resto.split_once('=') {
                Some((_, "FALHOU")) => arquivo.falhou = true,
                Some((_, "OK")) => arquivo.deu_certo = true,
                _ => {}
            }
        }
    }

    arquivo
}

/// Por que um arquivo encontrado nao e desfecho de job nenhum do ARCA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NaoEDesfecho {
    /// A primeira linha nao e `ARCA_SELO=` com dezesseis digitos.
    ///
    /// Um arquivo anterior ao mecanismo, escrito por outra coisa, ou cortado
    /// na janela entre o `>` truncar e o `echo` escrever.
    SemLinhaDeSelo,

    /// Mais de uma linha de selo. Duas receitas escreveram no mesmo arquivo
    /// sem que a segunda o truncasse — o que nao deveria acontecer, e por isso
    /// mesmo nao se adivinha qual das duas vale.
    SeloRepetido,

    /// Selo e `ARCA_FIM` no lugar, e nenhuma linha dizendo o que aconteceu.
    /// O `if/then/else` de R-5 sempre escreve uma; um arquivo sem ela nao veio
    /// de uma receita inteira.
    SemMarcadorDeDesfecho,
}

impl fmt::Display for NaoEDesfecho {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NaoEDesfecho::SemLinhaDeSelo => write!(
                f,
                "a primeira linha nao traz `{MARCA_DO_SELO}`: este arquivo nao pertence a job nenhum do ARCA. Ou e anterior ao mecanismo do selo, ou o desligamento pegou a receita entre truncar o arquivo e escrever a primeira linha"
            ),
            NaoEDesfecho::SeloRepetido => write!(
                f,
                "ha mais de uma linha `{MARCA_DO_SELO}` no arquivo: duas operacoes escreveram nele sem que a segunda o truncasse, e nao ha como saber qual delas o terminou"
            ),
            NaoEDesfecho::SemMarcadorDeDesfecho => write!(
                f,
                "o arquivo tem selo e tem `{MARCA_DO_FIM}`, e nao diz o que aconteceu: nenhuma linha `ARCA_BACKUP=` ou `ARCA_RESTORE=`. A receita sempre escreve uma"
            ),
        }
    }
}

/// A tabela do §5.5, com um desfecho encontrado na mao.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Julgamento {
    /// Selo bate, `ARCA_FIM` presente, desfecho `OK`.
    Concluida,

    /// Selo bate, desfecho `FALHOU`. O Clonezilla falhou e disse.
    Falhou,

    /// Selo bate, sem `ARCA_FIM`: desligamento no meio. A pasta e residuo.
    Truncado,

    /// Selo nao bate: o desfecho e de outro job (R-6).
    JobFantasma { encontrado: Selo },

    /// O arquivo existe e nao e desfecho de job nenhum. **Linha nova do
    /// §5.5**, aberta na etapa E5 — ver o cabecalho deste modulo.
    NaoPertenceAoArca(NaoEDesfecho),
}

/// Julga o que se encontrou contra o selo do job pendente (C-11, R-6).
///
/// A ordem e "toda forma de nao ter dado certo antes de toda forma de ter
/// dado", e dentro dela "toda forma de nao ser deste job antes de qualquer
/// leitura do que ele diz". Julgar o conteudo antes da procedencia leria o
/// desfecho de outro job como se fosse deste.
pub fn julgar(arquivo: &Arquivo, esperado: &Selo) -> Julgamento {
    // 1. De quem e este arquivo. Antes de tudo, porque o resto so faz sentido
    //    depois de ele ser deste job.
    if arquivo.linhas_de_selo > 1 {
        return Julgamento::NaoPertenceAoArca(NaoEDesfecho::SeloRepetido);
    }

    let Some(encontrado) = &arquivo.selo else {
        // E aqui que a linha nova ganha a vez. O ramo natural seria comparar
        // com o esperado e dizer "o selo nao bate" — e seria mentira: nao ha
        // selo a bater.
        return Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo);
    };

    if encontrado != esperado {
        return Julgamento::JobFantasma {
            encontrado: encontrado.clone(),
        };
    }

    // 2. O arquivo esta inteiro? Sem `ARCA_FIM` nao ha o que acreditar do que
    //    esta escrito nele: o desligamento pode ter cortado depois do `OK` que
    //    a conferencia nativa ainda iria desmentir.
    if !arquivo.fim {
        return Julgamento::Truncado;
    }

    // 3. So agora o que ele diz — e a falha antes do exito.
    if arquivo.falhou {
        return Julgamento::Falhou;
    }
    if arquivo.deu_certo {
        return Julgamento::Concluida;
    }

    Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemMarcadorDeDesfecho)
}

/// O que se encontrou no lugar do desfecho, incluindo nao haver nada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encontrado {
    /// Ha job pendente e ha arquivo: o julgamento vale.
    Arquivo(Julgamento),

    /// Ha job pendente e nao ha `arca-fim.txt`. **Nunca e silencio** (C-12):
    /// o boot nao aconteceu, ou o Clonezilla abriu o menu.
    SemArquivo,

    /// O arquivo esta la e nao se deixou lê.
    ///
    /// Distinta de [`Encontrado::SemArquivo`], e a distincao e o ponto: sem
    /// ela, um `arca-fim.txt` ilegivel sairia como "o boot nao aconteceu", e
    /// quem lesse concluiria que o backup nunca rodou — quando ele pode ter
    /// terminado bem. E o mesmo padrao que o ADR-0005 nomeou no firmware:
    /// **"nao consegui olhar" nunca vira "nao ha nada la"**.
    NaoDeuParaLer { motivo: String },
}

impl fmt::Display for Encontrado {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Encontrado::Arquivo(Julgamento::Concluida) => {
                write!(f, "concluida — o selo bate e a receita chegou ao fim")
            }
            Encontrado::Arquivo(Julgamento::Falhou) => {
                write!(f, "o Clonezilla falhou e disse: o desfecho e FALHOU")
            }
            Encontrado::Arquivo(Julgamento::Truncado) => write!(
                f,
                "truncado — o selo bate e falta o `{MARCA_DO_FIM}`: o desligamento pegou a operacao no meio, e a pasta e residuo"
            ),
            Encontrado::Arquivo(Julgamento::JobFantasma { encontrado }) => write!(
                f,
                "job fantasma — o desfecho tras o selo `{encontrado}`, que nao e o do job pendente. O ARCA ignora este arquivo"
            ),
            Encontrado::Arquivo(Julgamento::NaoPertenceAoArca(motivo)) => {
                write!(f, "nao e desfecho de job nenhum: {motivo}")
            }
            Encontrado::SemArquivo => write!(
                f,
                "nao ha `arca-fim.txt`: ou o boot nao aconteceu, ou o Clonezilla abriu o menu em vez de executar a receita"
            ),
            Encontrado::NaoDeuParaLer { motivo } => write!(
                f,
                "o `arca-fim.txt` esta la e nao se deixou lê ({motivo}). Isto NAO e o mesmo que o boot nao ter acontecido: a operacao pode ter terminado bem, e o que falhou foi olhar"
            ),
        }
    }
}

/// O nome da pasta em que este job escreveria o desfecho, para a tela.
pub fn pasta_do_job(comando: Operacao, nome: Option<&Nome>) -> String {
    crate::receita::pasta_do_log(comando, nome)
}

#[cfg(test)]
mod testes {
    use super::*;

    const DO_JOB: &str = "a3f1c9e07b2d4856";
    const DE_OUTRO: &str = "7e02b4d1af963c85";

    /// O `arca-fim.txt` que a receita de backup produz num exito.
    const CONCLUIDO: &str = "ARCA_SELO=a3f1c9e07b2d4856\nARCA_BACKUP=OK\nARCA_FIM\n";

    /// O unico `arca-fim.txt` que existe neste dispositivo, copiado dele em
    /// 22/08/2026. Vinte e cinco bytes, duas linhas, **nenhum selo** — veio do
    /// trabalho manual de validacao, e nao de receita nenhuma (P-16).
    const O_DO_DISPOSITIVO: &str = "ARCA_RESTORE=OK\nARCA_FIM\n";

    fn selo(bruto: &str) -> Selo {
        Selo::novo(bruto).expect("selo de teste valido")
    }

    fn julgar_texto(texto: &str) -> Julgamento {
        julgar(&ler(texto), &selo(DO_JOB))
    }

    #[test]
    fn o_desfecho_do_proprio_job_e_concluido() {
        assert_eq!(julgar_texto(CONCLUIDO), Julgamento::Concluida);
    }

    #[test]
    fn a_restauracao_bem_sucedida_tambem() {
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_RESTORE=OK\nARCA_FIM\n");
        assert_eq!(julgar_texto(&texto), Julgamento::Concluida);
    }

    #[test]
    fn o_desfecho_falho_e_falha() {
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_BACKUP=FALHOU\nARCA_FIM\n");
        assert_eq!(julgar_texto(&texto), Julgamento::Falhou);
    }

    #[test]
    fn sem_arca_fim_o_desfecho_e_truncado() {
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_BACKUP=OK\n");
        assert_eq!(julgar_texto(&texto), Julgamento::Truncado);
    }

    // ───────────────────── o criterio de aceite da E5 ─────────────────────

    #[test]
    fn selo_divergente_e_job_fantasma_com_mensagem_propria() {
        // O "pronto quando" da etapa, na letra. A mensagem tem de nomear o
        // selo encontrado: sem ele, quem lê nao tem como saber de que job
        // aquele arquivo veio.
        let texto = format!("ARCA_SELO={DE_OUTRO}\nARCA_BACKUP=OK\nARCA_FIM\n");

        let julgamento = julgar_texto(&texto);
        assert_eq!(
            julgamento,
            Julgamento::JobFantasma {
                encontrado: selo(DE_OUTRO)
            }
        );

        let mensagem = Encontrado::Arquivo(julgamento).to_string();
        assert!(mensagem.contains("job fantasma"), "{mensagem}");
        assert!(mensagem.contains(DE_OUTRO), "{mensagem}");
        assert!(
            !mensagem.contains(DO_JOB),
            "o selo do job vazou: {mensagem}"
        );
    }

    #[test]
    fn o_job_fantasma_e_julgado_antes_de_o_conteudo_ser_lido() {
        // Um desfecho de outro job que dissesse `OK` nao pode virar operacao
        // concluida, e um que dissesse `FALHOU` nao pode virar falha desta.
        // O selo decide antes de qualquer leitura do que esta escrito.
        for corpo in ["ARCA_BACKUP=OK", "ARCA_BACKUP=FALHOU"] {
            let texto = format!("ARCA_SELO={DE_OUTRO}\n{corpo}\nARCA_FIM\n");
            assert!(
                matches!(julgar_texto(&texto), Julgamento::JobFantasma { .. }),
                "`{corpo}` de outro job foi lido como desfecho deste"
            );
        }
    }

    // ─────────────────── a linha nova do §5.5 ───────────────────

    #[test]
    fn o_arca_fim_do_dispositivo_nao_e_desfecho_e_nao_diz_que_o_selo_nao_bate() {
        // O achado que abriu a linha. Este arquivo **existe**, tem `ARCA_FIM`
        // e um desfecho `OK`, e nao pertence a job nenhum: nao ha selo nele.
        // A mensagem nao pode falar em selo que nao bate — nao ha selo a bater.
        let julgamento = julgar_texto(O_DO_DISPOSITIVO);
        assert_eq!(
            julgamento,
            Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo)
        );

        let mensagem = Encontrado::Arquivo(julgamento).to_string();
        assert!(
            !mensagem.contains("nao bate") && !mensagem.contains("fantasma"),
            "a mensagem mente sobre haver selo: {mensagem}"
        );
        assert!(mensagem.contains("ARCA_SELO="), "{mensagem}");
    }

    #[test]
    fn o_arquivo_cortado_entre_truncar_e_escrever_cai_na_linha_nova() {
        // Medido: `>` trunca ao abrir, antes de o `echo` rodar. Um
        // desligamento nessa janela deixa o arquivo em zero byte — e o passo
        // seguinte da receita ja nao aconteceria. E o caso que torna esta
        // linha alcancavel de verdade, e nao so historica.
        for cortado in ["", "\n", "ARCA_SEL", "ARCA_SELO=", "ARCA_SELO=a3f1c9e0"] {
            assert_eq!(
                julgar_texto(cortado),
                Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo),
                "`{cortado}` nao caiu na linha nova"
            );
        }
    }

    #[test]
    fn selo_fora_da_primeira_linha_nao_e_selo() {
        // A receita o escreve com `>`, que trunca: num arquivo que ela
        // produziu ele nao pode estar em outro lugar. Aceitar um selo achado
        // no meio faria o rastro de dois jobs passar pelo segundo deles.
        let texto = format!("lixo de outra coisa\nARCA_SELO={DO_JOB}\nARCA_BACKUP=OK\nARCA_FIM\n");
        assert_eq!(
            julgar_texto(&texto),
            Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo)
        );
    }

    #[test]
    fn duas_linhas_de_selo_sao_recusa_e_nao_escolha() {
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_BACKUP=OK\nARCA_SELO={DE_OUTRO}\nARCA_FIM\n");
        assert_eq!(
            julgar_texto(&texto),
            Julgamento::NaoPertenceAoArca(NaoEDesfecho::SeloRepetido)
        );
    }

    #[test]
    fn selo_certo_e_fim_sem_marcador_nao_e_silencio() {
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_FIM\n");
        assert_eq!(
            julgar_texto(&texto),
            Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemMarcadorDeDesfecho)
        );
    }

    // ─────────────── a ordem entre falhar e ter dado certo ───────────────

    #[test]
    fn com_as_duas_marcas_no_mesmo_arquivo_a_falha_ganha() {
        // A receita **acrescenta** com `>>`. Um arquivo com as duas marcas nao
        // deveria existir, e e exatamente o caso em que ler a boa primeiro
        // transforma uma falha em exito — a licao que a revisao da E3 impos ao
        // leitor do `arca-check.log`.
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_BACKUP=OK\nARCA_BACKUP=FALHOU\nARCA_FIM\n");
        assert_eq!(julgar_texto(&texto), Julgamento::Falhou);

        // E na outra ordem tambem, que e a que a leitura ingenua acertaria por
        // acaso.
        let invertido =
            format!("ARCA_SELO={DO_JOB}\nARCA_BACKUP=FALHOU\nARCA_BACKUP=OK\nARCA_FIM\n");
        assert_eq!(julgar_texto(&invertido), Julgamento::Falhou);
    }

    #[test]
    fn o_truncado_ganha_do_ok_que_ficou_escrito() {
        // Um `OK` sem `ARCA_FIM` e um `OK` que o desligamento cortou antes de
        // a receita terminar. Acreditar nele seria dizer que a operacao
        // terminou porque a metade dela terminou (S-5).
        let texto = format!("ARCA_SELO={DO_JOB}\nARCA_BACKUP=OK\n");
        assert_eq!(julgar_texto(&texto), Julgamento::Truncado);
    }

    // ───────────────────────── a leitura crua ─────────────────────────

    #[test]
    fn a_leitura_aguenta_o_crlf_e_o_espaco_a_mais() {
        // O arquivo e escrito por um `echo` do bash e lido pelo Windows; nada
        // garante em que forma as linhas chegam.
        let texto = format!("ARCA_SELO={DO_JOB}\r\n  ARCA_BACKUP=OK  \r\nARCA_FIM\r\n");
        assert_eq!(julgar_texto(&texto), Julgamento::Concluida);
    }

    #[test]
    fn selo_com_forma_errada_na_primeira_linha_nao_e_selo() {
        for bruto in [
            "A3F1C9E07B2D4856",
            "a3f1c9e07b2d485",
            "a3f1c9e07b2d48567",
            "zzzzzzzzzzzzzzzz",
        ] {
            let texto = format!("ARCA_SELO={bruto}\nARCA_BACKUP=OK\nARCA_FIM\n");
            assert_eq!(
                julgar_texto(&texto),
                Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo),
                "`{bruto}` passou por selo"
            );
        }
    }

    #[test]
    fn sem_arquivo_nenhum_a_mensagem_nomeia_as_duas_causas() {
        // C-12: ausencia de desfecho e falha, nunca silencio, e as duas causas
        // possiveis tem de aparecer.
        let mensagem = Encontrado::SemArquivo.to_string();
        assert!(mensagem.contains("boot nao aconteceu"), "{mensagem}");
        assert!(mensagem.contains("menu"), "{mensagem}");
    }

    #[test]
    fn nao_conseguir_lê_nao_e_o_mesmo_que_nao_haver() {
        // O padrao que o ADR-0005 nomeou no firmware, aqui: um `arca-fim.txt`
        // ilegivel que saisse como "o boot nao aconteceu" faria alguem
        // concluir que o backup nunca rodou — e ele pode ter terminado bem.
        let ilegivel = Encontrado::NaoDeuParaLer {
            motivo: "acesso negado".to_string(),
        };

        assert_ne!(ilegivel, Encontrado::SemArquivo);

        let mensagem = ilegivel.to_string();
        assert!(mensagem.contains("acesso negado"), "{mensagem}");
        assert!(
            !mensagem.contains("boot nao aconteceu")
                || mensagem.contains("nao e o mesmo que o boot nao ter acontecido"),
            "a mensagem sugere que o boot falhou: {mensagem}"
        );
    }

    // ────────── os dois lados do reinicio concordam no conteudo ──────────

    /// O `arca-fim.txt` que a receita **de verdade** deixaria, no caminho de
    /// exito, montado a partir da string que ela grava no `grub.cfg`.
    ///
    /// Sem isto, os dois lados do reinicio so estao amarrados pelo marcador:
    /// a constante `MARCA_DO_SELO` e compartilhada, e um teste de
    /// [`crate::receita`] prende o texto literal a §10.1. O que ficava de fora
    /// e a **forma** — se o leitor passasse a exigir o selo na segunda linha,
    /// nada reclamaria.
    fn arca_fim_que_a_receita_produziria(receita: &str, desfecho: &str) -> String {
        let mut linhas = Vec::new();

        // O ramo de falha do `if` escreve no mesmo arquivo; aqui se monta o
        // caminho de exito, que e o unico em que os tres `echo` correm.
        //
        // # Um `else` pode ter mais de um comando, e ate a E11 nao tinha
        //
        // A primeira versao pulava so o passo que **comeca** com `else `, o
        // que bastava enquanto cada ramo tinha um comando so. A receita de
        // verificacao tem dois — o `ARCA_VEREDITO=` no `arca-check.log` e o
        // `ARCA_VERIFY=` no `arca-fim.txt` —, e o segundo entrava no caminho de
        // exito como se fosse dele. O teste falou, e o defeito era daqui.
        //
        // Entao acompanha-se o ramo: liga em `else `, desliga no `fi` que o
        // fecha — que pode vir como passo proprio ou grudado no fim do
        // anterior, como em `... >> arquivo; fi`.
        let mut no_ramo_de_falha = false;

        for passo in receita.split("; ") {
            let passo = passo.trim().trim_start_matches("then ");
            let fecha_o_ramo = passo == "fi" || passo.ends_with("; fi") || passo.ends_with(" fi");

            if passo.starts_with("else ") {
                no_ramo_de_falha = true;
            }
            if no_ramo_de_falha {
                if fecha_o_ramo {
                    no_ramo_de_falha = false;
                }
                continue;
            }

            let Some(resto) = passo.strip_prefix("echo ") else {
                continue;
            };

            for redirecionamento in [" >> ", " > "] {
                if let Some((texto, alvo)) = resto.split_once(redirecionamento) {
                    if alvo.trim().trim_end_matches("; fi") == desfecho {
                        linhas.push(texto.to_string());
                    }
                    break;
                }
            }
        }

        // A guarda que torna este teste honesto: uma extracao que deixasse de
        // achar os passos passaria em silencio sobre um arquivo vazio, e um
        // arquivo vazio ja e recusado pelo leitor — o teste "passaria" pelo
        // motivo errado.
        assert_eq!(
            linhas.len(),
            3,
            "a extracao achou {} `echo` para o desfecho, e a receita escreve tres \
             (selo, marcador, fim). O formato da receita mudou:\n{receita}",
            linhas.len()
        );

        format!("{}\n", linhas.join("\n"))
    }

    #[test]
    fn o_leitor_entende_o_arca_fim_que_a_receita_escreve() {
        // O oraculo e a receita, e nao um arquivo que este teste inventou. Os
        // dois lados do reinicio tem de concordar em **tres** coisas: o
        // marcador, a ordem das linhas e o selo estar na primeira.
        use crate::nome::Nome;
        use crate::receita::{Disco, Pedido, Receita};

        // As **tres**, e nao duas: a `Verificacao` entrou na E11 e escreve
        // `ARCA_VERIFY=`, que este modulo lê pelo **sufixo** — `=OK` ou
        // `=FALHOU` — e nao pelo marcador inteiro. O comentario de [`ler`] ja
        // dizia que "um terceiro amanha continuaria a ser lido"; este laco e o
        // que cobra que a promessa valha.
        for operacao in [
            Operacao::Backup,
            Operacao::Restauracao,
            Operacao::Verificacao,
        ] {
            let nome = Nome::novo("2026-08-22_Apps").unwrap();
            let esperado = selo(DO_JOB);

            let receita = Receita::montar(&Pedido {
                operacao,
                nome: Some(nome.clone()),
                // So as duas primeiras nomeiam disco; o `ocs-chkimg` opera
                // sobre a imagem, e `Receita::montar` recusa a incoerencia.
                disco: operacao
                    .nomeia_disco()
                    .then(|| Disco::novo("nvme0n1").unwrap()),
                selo: esperado.clone(),
            })
            .unwrap();

            let caminho = format!(
                "/home/partimag/ARCA-LOGS/{}/arca-fim.txt",
                pasta_do_job(operacao, Some(&nome))
            );
            let arquivo = arca_fim_que_a_receita_produziria(receita.comando(), &caminho);

            assert_eq!(
                julgar(&ler(&arquivo), &esperado),
                Julgamento::Concluida,
                "o leitor nao entende o que a receita de {} escreve:\n{arquivo}",
                operacao.nome()
            );

            // E o mesmo arquivo, com o selo de outro job, tem de virar
            // fantasma — senao o teste acima estaria provando so que o leitor
            // aprova tudo.
            assert!(
                matches!(
                    julgar(&ler(&arquivo), &selo(DE_OUTRO)),
                    Julgamento::JobFantasma { .. }
                ),
                "o desfecho da receita de {} passou por um selo alheio",
                operacao.nome()
            );
        }
    }

    #[test]
    fn a_tabela_do_paragrafo_5_5_nao_tem_caso_sem_mensagem() {
        // Nenhuma linha da tabela e silencio. Este teste percorre todas em vez
        // dos dois casos que ocorreriam a quem o escreveu.
        let todos = [
            Encontrado::Arquivo(Julgamento::Concluida),
            Encontrado::Arquivo(Julgamento::Falhou),
            Encontrado::Arquivo(Julgamento::Truncado),
            Encontrado::Arquivo(Julgamento::JobFantasma {
                encontrado: selo(DE_OUTRO),
            }),
            Encontrado::Arquivo(Julgamento::NaoPertenceAoArca(NaoEDesfecho::SemLinhaDeSelo)),
            Encontrado::Arquivo(Julgamento::NaoPertenceAoArca(NaoEDesfecho::SeloRepetido)),
            Encontrado::Arquivo(Julgamento::NaoPertenceAoArca(
                NaoEDesfecho::SemMarcadorDeDesfecho,
            )),
            Encontrado::SemArquivo,
            Encontrado::NaoDeuParaLer {
                motivo: "acesso negado".to_string(),
            },
        ];

        for caso in todos {
            let mensagem = caso.to_string();
            assert!(
                mensagem.chars().count() > 20,
                "{caso:?} nao tem mensagem propria: {mensagem:?}"
            );
        }
    }
}
