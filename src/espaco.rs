//! A regra de espaco de B-4.
//!
//! > Espaco minimo: o maior entre `maior imagem do dispositivo × 1,3` e
//! > `em uso × 0,45`. Entre 1× e 1,5× disso: avisar e pedir confirmacao
//! > digitada.
//!
//! Codigo puro, e por isso separado: a regra tem tres faixas e um `max` entre
//! duas estimativas, e nenhuma delas merece depender de dispositivo conectado
//! para ser testada.
//!
//! # As duas estimativas dizem coisas diferentes, e por isso ha as duas
//!
//! `maior imagem × 1,3` e um numero **medido neste dispositivo**: e o tamanho
//! que uma imagem desta maquina de fato ocupou, com folga. Ele so existe
//! depois do primeiro backup.
//!
//! `em uso × 0,45` e uma **estimativa de compressao**: o §3.3 do PRD registra
//! que o `-z9p` levou a imagem a ~39% do volume em uso, e 0,45 e isso com
//! folga. Funciona no dispositivo vazio, que e quando a outra nao existe.
//!
//! O maior das duas ganha porque errar para menos e o unico erro caro aqui: um
//! backup que enche o dispositivo no meio deixa um residuo de dezenas de
//! gigabytes que o usuario tem de apagar a mao (B-10).
//!
//! # Medido em 22/08/2026, nesta maquina
//!
//! | | |
//! |---|---|
//! | Maior imagem (`2026-08-21_WindowsCompleto`) | 38.823.813.652 |
//! | × 1,3 | 50.470.957.747 |
//! | Em uso no disco do Windows | 112.973.562.368 |
//! | × 0,45 | 50.838.103.065 |
//! | **Minimo exigido** (o maior dos dois) | **50.838.103.065** |
//! | Livre no `ARCAVAULT` | 176.291.147.776 |
//! | Folga | 3,47× |
//!
//! As duas estimativas caem a menos de 1% uma da outra, o que e um bom sinal
//! sobre as duas — e coincidencia desta maquina, nao propriedade da regra.

use crate::formato::tamanho;
use std::fmt;

/// Quanto a maior imagem do dispositivo sugere, em milesimos.
const FATOR_DA_MAIOR_IMAGEM: u64 = 1_300;

/// Quanto o disco em uso sugere, em milesimos. O §3.3 mediu ~39% com `-z9p`.
const FATOR_DO_EM_USO: u64 = 450;

/// Ate onde vai a faixa de aviso, em milesimos do minimo.
const TETO_DO_AVISO: u64 = 1_500;

/// O que a regra respondeu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veredito {
    /// Acima de 1,5× do minimo. Segue sem perguntar nada.
    Suficiente,

    /// Entre 1× e 1,5× do minimo. Cabe, e por pouco: B-4 manda avisar e pedir
    /// confirmacao digitada.
    Apertado,

    /// Abaixo do minimo. Recusa.
    Insuficiente,
}

/// A conta inteira, com as parcelas a vista.
///
/// As parcelas ficam porque a saida do §5.2 mostra a estimativa, e porque quem
/// vir "espaco insuficiente" precisa poder conferir de onde o numero saiu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Estimativa {
    /// `maior imagem × 1,3`, ou zero se nao ha imagem no dispositivo.
    pub pela_maior_imagem: u64,
    /// `em uso × 0,45`.
    pub pelo_em_uso: u64,
    /// O maior dos dois: o que se exige de fato.
    pub minimo: u64,
    pub livre: u64,
    pub veredito: Veredito,
}

// Nao ha `folga_em_milesimos` aqui, e houve. Ela nao tinha chamador, o doc
// prometia saturar em zero e o codigo devolvia `u64::MAX` — quem a usasse
// comparando com `TETO_DO_AVISO` leria "folga infinita" onde o contrato dizia
// "folga nenhuma". Um metodo sem uso e com o doc mentindo e pior do que nao
// existir: [`Veredito`] ja diz em que faixa a estimativa caiu, e quem precisar
// da razao a calcula a partir de `livre` e `minimo`, que estao a vista.

impl fmt::Display for Estimativa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quanto = tamanho(self.minimo);
        match self.veredito {
            Veredito::Suficiente => {
                write!(f, "~{quanto} · espaco suficiente")
            }
            Veredito::Apertado => write!(
                f,
                "~{quanto} · CABE POR POUCO: ha {} livres, menos de 1,5x o necessario",
                tamanho(self.livre)
            ),
            Veredito::Insuficiente => write!(
                f,
                "~{quanto} · NAO CABE: ha {} livres",
                tamanho(self.livre)
            ),
        }
    }
}

/// A regra de B-4.
///
/// `maior_imagem` e zero quando o dispositivo nao tem imagem nenhuma — e ai a
/// estimativa por compressao e a unica que existe, que e exatamente o caso do
/// primeiro backup de um dispositivo novo.
pub fn avaliar(maior_imagem: u64, em_uso: u64, livre: u64) -> Estimativa {
    // Em milesimos, e nao em ponto flutuante: sao bytes, e o arredondamento de
    // um `f64` num numero de doze digitos nao e o que se quer numa regra que
    // decide se um backup cabe.
    let pela_maior_imagem = proporcao(maior_imagem, FATOR_DA_MAIOR_IMAGEM);
    let pelo_em_uso = proporcao(em_uso, FATOR_DO_EM_USO);
    let minimo = pela_maior_imagem.max(pelo_em_uso);

    let veredito = if livre < minimo {
        Veredito::Insuficiente
    } else if livre < proporcao(minimo, TETO_DO_AVISO) {
        Veredito::Apertado
    } else {
        Veredito::Suficiente
    };

    Estimativa {
        pela_maior_imagem,
        pelo_em_uso,
        minimo,
        livre,
        veredito,
    }
}

/// `valor × milesimos / 1000`, sem estourar em numeros de doze digitos.
fn proporcao(valor: u64, milesimos: u64) -> u64 {
    // `(valor / 1000) * milesimos` perde ate 999 bytes de precisao, e o resto
    // recupera o que se perdeu. Multiplicar antes estouraria `u64` em disco de
    // alguns exabytes — o que nao acontece, mas a versao certa custa uma linha.
    (valor / 1_000) * milesimos + ((valor % 1_000) * milesimos) / 1_000
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Os numeros deste dispositivo, medidos em 22/08/2026.
    const MAIOR_IMAGEM: u64 = 38_823_813_652;
    const EM_USO: u64 = 112_973_562_368;
    const LIVRE: u64 = 176_291_147_776;

    #[test]
    fn os_numeros_desta_maquina_dao_espaco_suficiente() {
        let estimativa = avaliar(MAIOR_IMAGEM, EM_USO, LIVRE);

        assert_eq!(estimativa.pela_maior_imagem, 50_470_957_747);
        assert_eq!(estimativa.pelo_em_uso, 50_838_103_065);
        assert_eq!(estimativa.minimo, 50_838_103_065);
        assert_eq!(estimativa.veredito, Veredito::Suficiente);
    }

    #[test]
    fn o_maior_dos_dois_ganha_e_nao_o_ultimo_calculado() {
        // Nas duas ordens, porque errar para menos e o unico erro caro: um
        // backup que enche o dispositivo no meio deixa um residuo de dezenas
        // de gigabytes que o usuario apaga a mao.
        let imagem_manda = avaliar(100_000, 1_000, 10_000_000);
        assert_eq!(imagem_manda.minimo, imagem_manda.pela_maior_imagem);

        let uso_manda = avaliar(1_000, 100_000, 10_000_000);
        assert_eq!(uso_manda.minimo, uso_manda.pelo_em_uso);
    }

    #[test]
    fn sem_imagem_no_dispositivo_a_regra_e_so_a_compressao() {
        // O primeiro backup de um dispositivo novo. A estimativa por imagem
        // nao existe, e a por compressao tem de bastar — senao o minimo seria
        // zero e todo dispositivo teria espaco.
        let estimativa = avaliar(0, EM_USO, LIVRE);

        assert_eq!(estimativa.pela_maior_imagem, 0);
        assert_eq!(estimativa.minimo, 50_838_103_065);
        assert_eq!(estimativa.veredito, Veredito::Suficiente);
    }

    #[test]
    fn as_tres_faixas_ficam_nos_lugares_certos() {
        let minimo = avaliar(0, 100_000, 0).minimo;
        assert_eq!(minimo, 45_000);

        // Abaixo do minimo: recusa. Um byte abaixo ja e recusa.
        assert_eq!(avaliar(0, 100_000, minimo - 1).veredito, Veredito::Insuficiente);

        // Exatamente o minimo: cabe, e por pouco.
        assert_eq!(avaliar(0, 100_000, minimo).veredito, Veredito::Apertado);

        // Um byte abaixo de 1,5x: ainda apertado.
        let teto = minimo * 3 / 2;
        assert_eq!(avaliar(0, 100_000, teto - 1).veredito, Veredito::Apertado);

        // 1,5x exatos: suficiente.
        assert_eq!(avaliar(0, 100_000, teto).veredito, Veredito::Suficiente);
    }

    #[test]
    fn a_faixa_de_aviso_deste_dispositivo_e_a_que_o_prd_descreve() {
        // Entre 1x e 1,5x do minimo: avisar e pedir confirmacao digitada.
        let minimo = 50_838_103_065u64;

        assert_eq!(
            avaliar(MAIOR_IMAGEM, EM_USO, minimo + 1).veredito,
            Veredito::Apertado
        );
        assert_eq!(
            avaliar(MAIOR_IMAGEM, EM_USO, minimo * 3 / 2 + 1).veredito,
            Veredito::Suficiente
        );
    }

    #[test]
    fn dispositivo_cheio_e_recusa() {
        assert_eq!(
            avaliar(MAIOR_IMAGEM, EM_USO, 0).veredito,
            Veredito::Insuficiente
        );
    }

    #[test]
    fn a_conta_nao_estoura_nem_perde_bytes_em_numeros_de_disco() {
        // Um disco de 16 TB, que e maior do que qualquer coisa que este
        // projeto vai ver, e ainda assim cabe em `u64` sem estouro.
        let dezesseis_tb = 16u64 * 1024 * 1024 * 1024 * 1024;
        let estimativa = avaliar(0, dezesseis_tb, u64::MAX / 2);

        // A conta certa e a de `u128`, e nao `(v / 1000) * 450` — esta ultima
        // descarta o resto e foi o que este teste afirmava na primeira versao.
        // Sao 187 bytes de diferenca em dezesseis terabytes, e o ponto nao e o
        // tamanho: e que a versao "em milesimos" existe justamente para nao
        // perder o resto.
        let esperado = (dezesseis_tb as u128 * 450 / 1_000) as u64;
        assert_eq!(estimativa.pelo_em_uso, esperado);
        assert_eq!(estimativa.veredito, Veredito::Suficiente);
    }

    #[test]
    fn a_proporcao_bate_com_a_conta_direta_nos_numeros_reais() {
        // A versao "em milesimos" existe para nao estourar; ela nao pode
        // divergir da conta obvia nos numeros que de fato aparecem.
        for valor in [MAIOR_IMAGEM, EM_USO, LIVRE, 0, 1, 999, 1_000] {
            for fator in [FATOR_DA_MAIOR_IMAGEM, FATOR_DO_EM_USO, TETO_DO_AVISO] {
                let esperado = (valor as u128 * fator as u128 / 1_000) as u64;
                assert_eq!(
                    proporcao(valor, fator),
                    esperado,
                    "proporcao({valor}, {fator})"
                );
            }
        }
    }

    #[test]
    fn cada_veredito_tem_texto_proprio() {
        let suficiente = avaliar(MAIOR_IMAGEM, EM_USO, LIVRE).to_string();
        assert!(suficiente.contains("espaco suficiente"), "{suficiente}");

        let apertado = avaliar(MAIOR_IMAGEM, EM_USO, 50_838_103_066).to_string();
        assert!(apertado.contains("CABE POR POUCO"), "{apertado}");

        let sem = avaliar(MAIOR_IMAGEM, EM_USO, 1000).to_string();
        assert!(sem.contains("NAO CABE"), "{sem}");
    }
}
