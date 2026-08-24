//! Onde a sondagem grava, e como ela se lê de volta (E12, SD-2, SD-4).
//!
//! A sondagem escreve **dois** arquivos, e os dois na mesma pasta:
//!
//! ```text
//! ARCAVAULT\ARCA-LOGS\sondagem\
//!   ├── arca-fim.txt    ← o desfecho, como em toda operacao
//!   └── blkdev.list     ← o que o `lsblk` viu
//! ```
//!
//! Quem escreve e a receita, do outro lado do reinicio
//! ([`crate::receita::montar_sondagem`]); quem lê e este modulo, na volta. Os
//! dois caminhos saem da **mesma** funcao — [`crate::receita::pasta_do_log`] —
//! pelo motivo de sempre: dois lugares onde o nome da pasta se escreva
//! divergem na primeira mudanca, e o rastro disso seria um desfecho procurado
//! onde nao esta.
//!
//! # Por que este modulo existe, e nao um `blkdev::ler_do_dispositivo`
//!
//! [`crate::blkdev`] e puro: ele recebe texto e devolve discos, e e isso que
//! permite testar as quatro recusas de §4.5 sem dispositivo conectado. Ler
//! arquivo e conhecer caminho e outra responsabilidade, e ela e desta etapa.
//!
//! # A pasta esta **fora** da listagem de imagens, de proposito
//!
//! `ARCA-LOGS` e uma das [`crate::imagens::RESERVADAS`], e
//! [`crate::imagens::enumerar`] a pula — ha teste cobrando isso desde a E1
//! (`tests/e1_dispositivo_conectado.rs`). Entao quem quiser a sondagem tem de
//! vir busca-la aqui, e nao esperar que a enumeracao a entregue. Um `arca
//! list` que mostrasse `sondagem` como imagem seria pior: B-3 passaria a
//! recusar o nome, e L-2 a chamaria de residuo por nao ter `MD5SUMS`.

use crate::blkdev::{ARQUIVO, Fonte, Lista};
use crate::dispositivo::ARCA_LOGS;
use crate::portas::Arquivos;
use crate::receita::{Operacao, pasta_do_log};
use std::path::{Path, PathBuf};

/// O `blkdev.list` que a sondagem gravou, do lado Windows.
pub fn caminho(raiz_do_vault: &Path) -> PathBuf {
    raiz_do_vault
        .join(ARCA_LOGS)
        .join(pasta_do_log(Operacao::Sondagem, None))
        .join(ARQUIVO)
}

/// A sondagem do dispositivo, quando ha uma que se deixe lê.
///
/// # Toda falha vira `None`, e isso esta certo aqui
///
/// Nao haver sondagem e o caso normal; o arquivo estar la e nao se deixar lê
/// leva ao **mesmo** lugar, que e o §4.5 sem esta fonte. A diferenca que o
/// ADR-0005 defende — *"nao consegui olhar" nunca vira "nao ha nada la"* —
/// vale onde a distincao muda o que o ARCA faz, e aqui ela nao muda: as
/// imagens respondem, ou o nome fica por determinar e a tela diz por quê.
///
/// O que **nao** se perde e o desfecho: se a sondagem rodou e o arquivo dela
/// nao se deixa lê, quem diz isso e o `arca resultado`, que julga o
/// `arca-fim.txt` da mesma pasta pelo caminho de sempre.
///
/// `ler_texto_alheio` porque quem escreveu foi o `lsblk` do outro lado do
/// reinicio: um byte solto nao pode fazer a sondagem inteira sumir.
pub fn ler(arquivos: &dyn Arquivos, raiz_do_vault: &Path) -> Option<Lista> {
    let caminho = caminho(raiz_do_vault);
    let texto = arquivos.ler_texto_alheio(&caminho).ok()?;

    // O `mtime` sai da listagem da pasta, e nao de uma chamada propria: a
    // porta [`Arquivos`] nao tem "quando este arquivo mudou", e acrescentar um
    // metodo para um campo informativo custaria mais do que procurar a entrada
    // aqui. `None` quando nao se acha — que e o que [`crate::formato::dia_e_hora`]
    // imprime como `--/-- --:--`.
    let quando = caminho
        .parent()
        .and_then(|pasta| arquivos.listar(pasta).ok())
        .and_then(|entradas| {
            entradas
                .into_iter()
                .find(|entrada| entrada.caminho == caminho)
                .and_then(|entrada| entrada.modificado_em)
        });

    Some(Lista {
        fonte: Fonte::Sondagem { quando },
        texto,
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::{ArquivosEmMemoria, momento};

    /// O `blkdev.list` como a sondagem o grava — as colunas do cabecalho
    /// capturado, com o `ARCAVAULT` montado em `/home/partimag`.
    const DA_SONDAGEM: &str = concat!(
        "KNAME     NAME          SIZE TYPE FSTYPE   MOUNTPOINT                           MODEL\n",
        "sda       sda         238.5G disk                                               KGSSE100256\n",
        "sda1      |-sda1      236.9G part ntfs     /home/partimag                       \n",
        "nvme0n1   nvme0n1     465.8G disk                                               KINGSTON SNV3S500G\n",
    );

    #[test]
    fn o_caminho_e_a_pasta_fixa_dentro_do_arca_logs() {
        assert_eq!(
            caminho(Path::new(r"E:\")),
            PathBuf::from(r"E:\ARCA-LOGS\sondagem\blkdev.list")
        );
    }

    #[test]
    fn sem_sondagem_no_dispositivo_nao_ha_lista() {
        let arquivos = ArquivosEmMemoria::novo();
        assert_eq!(ler(&arquivos, Path::new(r"E:\")), None);
    }

    #[test]
    fn a_sondagem_volta_com_a_fonte_e_a_hora() {
        // A hora e o que separa a medicao de agora da de manhã, e ela sai do
        // `mtime` do arquivo — relogio do Windows, e nao o do live (P-7).
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\ARCA-LOGS\sondagem\blkdev.list", DA_SONDAGEM)
            .datado(r"E:\ARCA-LOGS\sondagem\blkdev.list", "2026-08-23T21:14:07");

        let lista = ler(&arquivos, Path::new(r"E:\")).expect("ha sondagem");

        assert_eq!(
            lista.fonte,
            Fonte::Sondagem {
                quando: Some(momento("2026-08-23T21:14:07"))
            }
        );
        assert_eq!(lista.texto, DA_SONDAGEM);
    }

    #[test]
    fn sem_mtime_a_sondagem_continua_valendo() {
        // A hora e informativa. Um sistema de arquivos que nao a responda nao
        // pode fazer o oraculo desaparecer — seria trocar o nome do disco por
        // um detalhe de tela.
        let arquivos =
            ArquivosEmMemoria::novo().com(r"E:\ARCA-LOGS\sondagem\blkdev.list", DA_SONDAGEM);

        let lista = ler(&arquivos, Path::new(r"E:\")).expect("ha sondagem");
        assert_eq!(lista.fonte, Fonte::Sondagem { quando: None });
    }

    #[test]
    fn o_arquivo_da_sondagem_tem_o_nome_do_que_ele_imita() {
        // Mesmo formato, mesmo parser, mesmo nome. Um nome diferente sugeriria
        // um segundo formato, e nao ha um.
        assert_eq!(ARQUIVO, "blkdev.list");
    }
}
