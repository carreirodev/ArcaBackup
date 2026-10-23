//! O andamento de um download: a medida dos ultimos 20 s, a linha que a tela
//! reescreve no lugar e o julgamento da fonte lenta (PR-7, WPC-220).
//!
//! Codigo puro, sem I/O e sem relogio. Quem mede o tempo e o adaptador do
//! `curl`, que entrega cada [`Amostra`] com o `decorrido` monotonico daquela
//! fonte; aqui so se faz conta com elas. E isso que deixa a regra da fonte
//! lenta ser testada com amostras inventadas, sem esperar 20 s.
//!
//! # A tela mostra o mesmo numero que decide
//!
//! A velocidade e o "faltam" da linha `Baixando Clonezilla` saem da mesma
//! [`Medicao`] que julga a fonte lenta, pelo mesmo caminho. Uma fonte nunca
//! sai por lenta com a tela dizendo que falta pouco, e a linha que diz que ela
//! saiu traz os dois numeros. Ver o ADR-0029.

use crate::formato::{em_megabytes, linha, tamanho};
use crate::pacote::{JANELA_DA_VELOCIDADE, LENTA_SE_FALTAREM_MAIS_DE, TAMANHO_BYTES};
use crate::portas::Amostra;
use std::fmt;
use std::time::Duration;

/// O rotulo da linha que se reescreve.
pub const ROTULO: &str = "Baixando Clonezilla";

/// Quantas casas a barra tem: uma por 5%.
const CASAS: u64 = 20;

/// Quanto tempo sem o `.parcial` crescer ate a linha trocar a estimativa por
/// ha quanto tempo a fonte parou (Unresolved 3 da WPC-220: 5 s, ate o usuario
/// dizer outro numero).
const PARADA_APARECE_EM: Duration = Duration::from_secs(5);

/// As amostras de uma fonte, desde que ela comecou.
#[derive(Debug, Clone)]
pub struct Medicao {
    /// Da mais antiga que ainda importa para a mais recente. A primeira e a
    /// do comeco da fonte — o `.parcial` como estava, em `decorrido` zero —
    /// ate a janela de 20 s passar dela.
    amostras: Vec<Amostra>,

    /// O `decorrido` da amostra em que o `.parcial` mudou de tamanho pela
    /// ultima vez.
    cresceu_em: Duration,
}

impl Medicao {
    /// A medida de uma fonte que comeca com `recebido` bytes no `.parcial`.
    pub fn comecando_com(recebido: u64) -> Medicao {
        Medicao {
            amostras: vec![Amostra {
                recebido,
                decorrido: Duration::ZERO,
            }],
            cresceu_em: Duration::ZERO,
        }
    }

    /// Acrescenta uma amostra, e esquece as que a janela deixou para tras.
    pub fn anotar(&mut self, amostra: Amostra) {
        if amostra.recebido != self.atual().recebido {
            self.cresceu_em = amostra.decorrido;
        }
        self.amostras.push(amostra);

        // A referencia so anda para frente: o que fica antes dela nao volta a
        // ser usado.
        let referencia = self.indice_da_referencia();
        self.amostras.drain(..referencia);
    }

    /// Quanto o `.parcial` tem na amostra mais recente.
    pub fn recebido(&self) -> u64 {
        self.atual().recebido
    }

    fn atual(&self) -> Amostra {
        *self
            .amostras
            .last()
            .expect("a medicao nasce com a amostra do comeco")
    }

    /// A amostra de onde a velocidade e medida: a ultima de ate 20 s antes da
    /// atual, ou a do comeco, quando a fonte corre ha menos que isso.
    ///
    /// A ultima de **ate** 20 s antes, e nao a primeira de depois: assim a
    /// janela nunca e mais curta que 20 s, e uma fonte que corre ha 20 s ou
    /// mais e julgada por 20 s de medida, nunca por 19,8.
    fn indice_da_referencia(&self) -> usize {
        let atual = self.atual().decorrido;
        self.amostras
            .iter()
            .rposition(|amostra| amostra.decorrido + JANELA_DA_VELOCIDADE <= atual)
            .unwrap_or(0)
    }

    fn referencia(&self) -> Amostra {
        self.amostras[self.indice_da_referencia()]
    }

    /// Bytes por segundo na janela, e `None` quando ela ainda nao tem
    /// tempo nenhum.
    fn velocidade(&self) -> Option<f64> {
        let (de, ate) = (self.referencia(), self.atual());
        let intervalo = ate.decorrido.saturating_sub(de.decorrido).as_secs_f64();
        (intervalo > 0.0).then(|| ate.recebido.saturating_sub(de.recebido) as f64 / intervalo)
    }

    /// Ha quanto tempo o `.parcial` nao cresce.
    fn parada_ha(&self) -> Duration {
        self.atual().decorrido.saturating_sub(self.cresceu_em)
    }

    /// O valor da linha `Baixando Clonezilla` enquanto a fonte corre.
    pub fn linha_viva(&self) -> String {
        let recebido = self.recebido();
        if recebido >= TAMANHO_BYTES {
            return self.linha_de_entrega(recebido);
        }
        if self.parada_ha() >= PARADA_APARECE_EM {
            return montar(recebido, Ritmo::Parada(self.parada_ha()));
        }
        match self.velocidade() {
            // Sem nada na janela e antes dos 5 s de parada, o que ha para
            // dizer e que ainda se mede: `0 B/s · faltam` infinito nao e uma
            // estimativa.
            Some(velocidade) if velocidade > 0.0 => montar(
                recebido,
                Ritmo::Baixando {
                    velocidade,
                    faltam: falta_em(recebido, velocidade),
                },
            ),
            _ => montar(recebido, Ritmo::Medindo),
        }
    }

    /// O valor da linha quando a fonte entregou `recebido` bytes: sem
    /// `faltam`, e sem velocidade quando nada a mediu.
    ///
    /// `recebido` vem de quem chama, e nao da ultima amostra: o duplo entrega
    /// sem amostra nenhuma, e a linha tem de terminar no que o destino tem.
    pub fn linha_de_entrega(&self, recebido: u64) -> String {
        montar(
            recebido,
            Ritmo::Entregue(self.velocidade().filter(|velocidade| *velocidade > 0.0)),
        )
    }

    /// O julgamento da primeira volta: `Some` quando a fonte perde a vez.
    ///
    /// Nunca antes de 20 s de medida, qualquer que seja a velocidade. Depois
    /// disso, uma fonte que nada mandou na janela nunca terminaria, e sai; e
    /// uma que manda sai quando, na velocidade da janela, o que falta levaria
    /// mais de 600 s. A conta usa o tempo exato, e nao o arredondado da tela:
    /// a tela arredonda quanto falta para cima justamente para nunca mostrar
    /// 10 minutos ou menos numa fonte que sai.
    pub fn lentidao(&self) -> Option<Lentidao> {
        let atual = self.atual();
        if atual.decorrido < JANELA_DA_VELOCIDADE || atual.recebido >= TAMANHO_BYTES {
            return None;
        }
        let velocidade = self.velocidade().unwrap_or(0.0);
        if velocidade <= 0.0 {
            return Some(Lentidao::NadaVeio);
        }
        let faltam = falta_em(atual.recebido, velocidade);
        (faltam > LENTA_SE_FALTAREM_MAIS_DE).then_some(Lentidao::Faltariam {
            faltam,
            velocidade: velocidade as u64,
        })
    }
}

/// O que falta do pacote, na velocidade dada.
fn falta_em(recebido: u64, velocidade: f64) -> Duration {
    Duration::from_secs_f64(TAMANHO_BYTES.saturating_sub(recebido) as f64 / velocidade)
}

/// Por que uma fonte da primeira volta perdeu a vez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lentidao {
    /// O que falta levaria mais de 600 s na velocidade dos ultimos 20 s.
    Faltariam {
        faltam: Duration,
        /// Bytes por segundo.
        velocidade: u64,
    },

    /// Nada veio nos ultimos 20 s.
    NadaVeio,
}

impl Lentidao {
    /// A velocidade medida quando a fonte saiu, que ordena a segunda volta.
    pub fn velocidade(&self) -> u64 {
        match self {
            Lentidao::Faltariam { velocidade, .. } => *velocidade,
            Lentidao::NadaVeio => 0,
        }
    }
}

impl fmt::Display for Lentidao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lentidao::Faltariam { faltam, velocidade } => write!(
                f,
                "lenta: faltariam {} a {}",
                quanto_falta(*faltam),
                por_segundo(*velocidade)
            ),
            Lentidao::NadaVeio => write!(
                f,
                "lenta: nada veio nos ultimos {} s",
                JANELA_DA_VELOCIDADE.as_secs()
            ),
        }
    }
}

/// A linha inteira, com o rotulo e sem a quebra de linha: e o que a
/// `LinhaViva` do `arca prepare` escreve, e o que tem de caber em 120 colunas.
pub fn com_o_rotulo(valor: &str) -> String {
    linha(ROTULO, valor).trim_end_matches('\n').to_string()
}

/// Quanto falta, arredondado para cima na unidade em que sai: segundos abaixo
/// de 1 min, minutos abaixo de 1 h, horas e minutos a partir dai.
///
/// Para cima, e nao para o mais perto: `faltam 10 min` numa fonte que sai por
/// lenta seria a tela desmentindo o julgamento.
pub fn quanto_falta(falta: Duration) -> String {
    let segundos = falta.as_secs_f64();
    let em_segundos = segundos.ceil() as u64;
    if em_segundos < 60 {
        return format!("{em_segundos} s");
    }
    let minutos = (segundos / 60.0).ceil() as u64;
    if minutos < 60 {
        return format!("{minutos} min");
    }
    format!("{} h {} min", minutos / 60, minutos % 60)
}

/// `64,0 KB/s`, na mesma escala de [`tamanho`].
fn por_segundo(bytes: u64) -> String {
    format!("{}/s", tamanho(bytes))
}

/// O que a linha diz depois de quanto veio.
enum Ritmo {
    Medindo,
    Baixando { velocidade: f64, faltam: Duration },
    Parada(Duration),
    Entregue(Option<f64>),
}

/// A barra, a porcentagem e quanto veio — arredondados para baixo, para 100%
/// so aparecer com o pacote inteiro —, e depois o ritmo.
fn montar(recebido: u64, ritmo: Ritmo) -> String {
    let parte = recebido.min(TAMANHO_BYTES);
    let casas = parte * CASAS / TAMANHO_BYTES;
    let porcentagem = parte * 100 / TAMANHO_BYTES;
    let barra: String = (0..CASAS)
        .map(|casa| if casa < casas { '█' } else { '░' })
        .collect();

    let comeco = format!(
        "[{barra}] {porcentagem:>3}% · {} de {} MB",
        em_megabytes(recebido),
        em_megabytes(TAMANHO_BYTES)
    );
    match ritmo {
        Ritmo::Medindo => format!("{comeco} · medindo"),
        Ritmo::Baixando { velocidade, faltam } => format!(
            "{comeco} · {} · faltam {}",
            por_segundo(velocidade as u64),
            quanto_falta(faltam)
        ),
        Ritmo::Parada(ha) => format!("{comeco} · 0 B/s · parada ha {} s", ha.as_secs()),
        Ritmo::Entregue(Some(velocidade)) => {
            format!("{comeco} · {}", por_segundo(velocidade as u64))
        }
        Ritmo::Entregue(None) => comeco,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn amostra(segundos: f64, recebido: u64) -> Amostra {
        Amostra {
            recebido,
            decorrido: Duration::from_secs_f64(segundos),
        }
    }

    /// A medida de uma fonte que comeca em `comeco` bytes e recebe as
    /// amostras dadas.
    fn medindo(comeco: u64, amostras: &[(f64, u64)]) -> Medicao {
        let mut medicao = Medicao::comecando_com(comeco);
        for (segundos, recebido) in amostras {
            medicao.anotar(amostra(*segundos, *recebido));
        }
        medicao
    }

    #[test]
    fn a_barra_e_a_porcentagem_arredondam_para_baixo() {
        let comeco = |recebido| montar(recebido, Ritmo::Medindo);

        assert!(comeco(0).starts_with("[░░░░░░░░░░░░░░░░░░░░]   0% · 0,0 de 535,5 MB"));
        assert!(
            comeco(125_829_120).starts_with("[████░░░░░░░░░░░░░░░░]  22% · 120,0 de 535,5 MB"),
            "{}",
            comeco(125_829_120)
        );
        assert!(
            comeco(561_478_647).starts_with("[███████████████████░]  99% · 535,5 de 535,5 MB"),
            "{}",
            comeco(561_478_647)
        );
        assert!(
            comeco(561_478_648).starts_with("[████████████████████] 100% · 535,5 de 535,5 MB"),
            "{}",
            comeco(561_478_648)
        );
    }

    #[test]
    fn quanto_falta_arredonda_para_cima() {
        let falta = |segundos: f64| quanto_falta(Duration::from_secs_f64(segundos));

        assert_eq!(falta(0.4), "1 s");
        assert_eq!(falta(59.0), "59 s");
        assert_eq!(falta(59.5), "1 min");
        assert_eq!(falta(85.1), "2 min");
        assert_eq!(falta(3_599.5), "1 h 0 min");
        assert_eq!(falta(8_311.4), "2 h 19 min");
    }

    #[test]
    fn a_velocidade_e_a_dos_ultimos_20_s() {
        // As amostras do critério 3: de 10 s a 30 s vieram 100 MB, 5,0 MB/s.
        let medicao = medindo(0, &[(0.0, 0), (10.0, 10_485_760), (30.0, 115_343_360)]);
        assert_eq!(
            com_o_rotulo(&medicao.linha_viva()),
            "  Baixando Clonezilla ............. [████░░░░░░░░░░░░░░░░]  20% · 110,0 de 535,5 MB · 5,0 MB/s · faltam 2 min"
        );

        // Com menos de 20 s, a janela e desde o comeco da fonte: 10 MB em 10 s.
        let medicao = medindo(0, &[(10.0, 10_485_760)]);
        assert!(
            medicao.linha_viva().contains(" · 1,0 MB/s · "),
            "{}",
            medicao.linha_viva()
        );

        // A janela nunca fica mais curta que 20 s: com amostras a cada 1,5 s,
        // a referencia de 31 s e a de 10,5 s (20,5 s antes), e nao a de 12 s.
        let mut medicao = Medicao::comecando_com(0);
        for passo in 1..=20 {
            let segundos = passo as f64 * 1.5;
            medicao.anotar(amostra(segundos, (segundos * 1_000.0) as u64));
        }
        medicao.anotar(amostra(31.0, 41_000));
        assert_eq!(medicao.referencia(), amostra(10.5, 10_500));
    }

    #[test]
    fn mais_de_uma_hora_sai_em_horas_e_minutos() {
        // O critério 4: 16 MB no `.parcial`, 1.310.720 bytes nos últimos 20 s.
        let medicao = medindo(15_466_496, &[(20.0, 16_777_216)]);
        assert_eq!(
            com_o_rotulo(&medicao.linha_viva()),
            "  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   2% · 16,0 de 535,5 MB · 64,0 KB/s · faltam 2 h 19 min"
        );
    }

    #[test]
    fn a_fonte_que_nao_cresce_ha_5_s_diz_ha_quanto_tempo_parou() {
        let parada = medindo(0, &[(0.0, 0), (10.0, 10_485_760), (15.0, 10_485_760)]);
        assert_eq!(
            com_o_rotulo(&parada.linha_viva()),
            "  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   1% · 10,0 de 535,5 MB · 0 B/s · parada ha 5 s"
        );

        // Com 4 s sem crescer, a linha ainda estima.
        let quase = medindo(0, &[(0.0, 0), (10.0, 10_485_760), (14.0, 10_485_760)]);
        assert!(
            !quase.linha_viva().contains("parada"),
            "{}",
            quase.linha_viva()
        );
        assert!(
            quase.linha_viva().contains("faltam"),
            "{}",
            quase.linha_viva()
        );

        // E, antes dos 5 s, nada na janela e `medindo`, e nao `faltam` infinito.
        let comecando = medindo(1_024, &[(1.0, 1_024), (3.0, 1_024)]);
        assert!(
            comecando.linha_viva().ends_with(" · medindo"),
            "{}",
            comecando.linha_viva()
        );
    }

    #[test]
    fn a_linha_viva_cabe_em_120_colunas() {
        // A largura e contada em caracteres, e a linha so tem caracteres que
        // o console desenha com uma coluna cada — o que a ultima assercao
        // cobra. O `·` foi medido em `examples/ponto_no_console.rs`; o `█` e
        // o `░`, em `examples/barra_no_console.rs`, em 23/10/2026, nesta
        // mesa, com a CP 850: cada um avancou o cursor uma coluna, e uma
        // linha viva de 109 caracteres ocupou 109 colunas.
        let mut recebidos: Vec<u64> = (0..=1_000)
            .map(|milesimo| TAMANHO_BYTES * milesimo / 1_000)
            .collect();
        for escala in [1u64, 1 << 10, 1 << 20] {
            for perto in [escala.saturating_sub(1), escala, escala + 1] {
                recebidos.push(perto);
            }
        }
        recebidos.extend([1, 99_999_999, 104_857_599, 104_857_600, TAMANHO_BYTES - 1]);

        let mut velocidades: Vec<f64> = Vec::new();
        for escala in [1u64, 1 << 10, 1 << 20, 1 << 30] {
            for fator in [1u64, 9, 10, 99, 100, 999, 1_000, 1_023] {
                let velocidade = escala * fator;
                if velocidade <= 1 << 30 {
                    velocidades.push(velocidade as f64);
                }
            }
            velocidades.push((escala * 1_024 - 1) as f64);
        }
        velocidades.push((1u64 << 30) as f64);

        let mut maior = (0, String::new());
        let mut fora_do_medido: Vec<char> = Vec::new();
        let mut medir = |valor: String| {
            let linha = com_o_rotulo(&valor);
            fora_do_medido.extend(
                linha
                    .chars()
                    .filter(|c| !(c.is_ascii_graphic() || " ·█░".contains(*c))),
            );
            let largura = linha.chars().count();
            if largura > maior.0 {
                maior = (largura, valor);
            }
        };
        for &recebido in &recebidos {
            medir(montar(recebido, Ritmo::Medindo));
            medir(montar(recebido, Ritmo::Entregue(None)));
            for segundos in [5, 59, 60, 999, 3_600] {
                medir(montar(
                    recebido,
                    Ritmo::Parada(Duration::from_secs(segundos)),
                ));
            }
            for &velocidade in &velocidades {
                medir(montar(
                    recebido,
                    Ritmo::Baixando {
                        velocidade,
                        faltam: falta_em(recebido, velocidade),
                    },
                ));
                medir(montar(recebido, Ritmo::Entregue(Some(velocidade))));
            }
        }

        assert!(
            maior.0 <= 120,
            "{} colunas: {}",
            maior.0,
            com_o_rotulo(&maior.1)
        );
        assert_eq!(
            fora_do_medido,
            Vec::<char>::new(),
            "caracteres que ninguem mediu no console"
        );
    }

    #[test]
    fn o_limite_de_600_s_e_estrito() {
        // A borda dos critérios 8 e 10: a 64 KB/s, com 39.321.600 bytes
        // faltando, faltam exatamente 600 s, e a fonte continua; um byte a
        // mais e ela sai. `>=` no lugar de `>` passaria pelos outros testes,
        // que ficam em 567 s e em 8.547 s.
        let no_limite = medindo(520_846_328, &[(20.0, 522_157_048)]);
        assert_eq!(no_limite.lentidao(), None);

        let um_byte_alem = medindo(520_846_327, &[(20.0, 522_157_047)]);
        assert!(
            matches!(
                um_byte_alem.lentidao(),
                Some(Lentidao::Faltariam {
                    velocidade: 65_536,
                    ..
                })
            ),
            "{:?}",
            um_byte_alem.lentidao()
        );
    }

    #[test]
    fn a_fonte_lenta_e_julgada_pelo_que_falta() {
        // O critério 8: 1.310.720 bytes em 20 s, do zero, sao 64 KB/s, e o
        // que falta levaria 8.547 s.
        let lenta = medindo(0, &[(20.0, 1_310_720)]);
        let lentidao = lenta
            .lentidao()
            .expect("a 64 KB/s, do zero, a fonte e lenta");
        assert_eq!(lentidao.velocidade(), 65_536);
        assert_eq!(
            lentidao.to_string(),
            "lenta: faltariam 2 h 23 min a 64,0 KB/s"
        );

        // E a linha da tela, na mesma amostra, diz os mesmos numeros.
        assert!(
            lenta
                .linha_viva()
                .ends_with(" · 64,0 KB/s · faltam 2 h 23 min"),
            "{}",
            lenta.linha_viva()
        );
    }

    #[test]
    fn antes_de_20_s_nenhuma_fonte_e_lenta() {
        assert_eq!(medindo(0, &[(19.999, 0)]).lentidao(), None);
        assert_eq!(medindo(0, &[(19.0, 1)]).lentidao(), None);

        // Aos 20 s, as mesmas fontes ja sao julgadas.
        assert_eq!(
            medindo(0, &[(20.0, 0)]).lentidao(),
            Some(Lentidao::NadaVeio)
        );
        assert!(medindo(0, &[(20.0, 1)]).lentidao().is_some());
    }

    #[test]
    fn perto_do_fim_a_fonte_lenta_termina() {
        // O critério 10: com 524.288.000 bytes e 64 KB/s, faltam 567 s.
        let perto = medindo(522_977_280, &[(20.0, 524_288_000)]);
        assert_eq!(perto.lentidao(), None);
        assert!(
            perto.linha_viva().ends_with(" · 64,0 KB/s · faltam 10 min"),
            "{}",
            perto.linha_viva()
        );
    }

    #[test]
    fn nada_em_20_s_e_lenta() {
        // Parada desde o comeco da fonte, com o que ja tinha vindo antes.
        let parada = medindo(1_048_576, &[(20.0, 1_048_576)]);
        assert_eq!(parada.lentidao(), Some(Lentidao::NadaVeio));
        assert_eq!(
            Lentidao::NadaVeio.to_string(),
            "lenta: nada veio nos ultimos 20 s"
        );

        // Cresceu no comeco e parou: o que conta sao os ultimos 20 s.
        let parou = medindo(0, &[(5.0, 1_048_576), (25.0, 1_048_576)]);
        assert_eq!(parou.lentidao(), Some(Lentidao::NadaVeio));
    }
}
