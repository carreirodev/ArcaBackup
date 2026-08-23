//! O que ha no `ARCAVAULT`: imagens e residuos.
//!
//! Lê o dispositivo, nunca um catalogo (L-1). Se a informacao esta na
//! listagem de diretorios, nao ha o que armazenar — e essa e a razao de o
//! ARCA nao ter banco nenhum.
//!
//! O que separa imagem de residuo e o `MD5SUMS` (B-3). Uma pasta sem ele e
//! rastro de backup interrompido: nunca e oferecida para restaurar (L-2),
//! nunca e sobrescrita, e o ARCA nunca a apaga (B-10) — quem apaga e o
//! usuario, a mao, depois de olhar.

use crate::erro::Resultado;
use crate::portas::Arquivos;
use chrono::{DateTime, Local};
use std::path::Path;

/// O arquivo que separa imagem de residuo (B-3).
///
/// Reexportado de [`crate::md5sums`], que e quem sabe o que ha **dentro** dele
/// desde a E11. Aqui so a existencia importa; la, o conteudo. Um nome so, num
/// lugar so.
use crate::md5sums::ARQUIVO as MD5SUMS;

/// Onde o `ocs-chkimg` deixa o veredito, por B-9.
///
/// Publico porque quem **escreve** este arquivo e a receita da E3
/// ([`crate::receita`]), e quem o lê e este modulo. Um nome so, num lugar so:
/// mudar o arquivo aqui muda a receita junto, e nao ha como um lado divergir
/// do outro em silencio.
pub const CHECK_LOG: &str = "arca-check.log";

/// Pastas de servico do `ARCAVAULT`: existem no dispositivo e nunca sao
/// imagem nem residuo.
///
/// `ARCA-LOGS` e do §4 do PRD; `ARCA-DOCS` guarda a documentacao do
/// dispositivo; as outras duas sao do NTFS, e o Windows nem deixa abrir.
///
/// Toda pasta fora desta lista e candidata, mesmo vazia. E melhor um residuo
/// de mentira aparecendo na lista do que um de verdade escondido dela: um
/// residuo escondido e um nome que o pre-voo vai recusar sem que ninguem
/// tenha visto por que (B-3).
///
/// Publica porque [`crate::nome`] tem de **recusar** estes nomes: uma imagem
/// chamada `ARCA-LOGS` seria gravada por cima da pasta de logs e, pior,
/// desapareceria desta enumeracao — invisivel no `arca list` e invisivel para
/// o pre-voo de B-3, que e quem recusaria o nome repetido. O que esta lista
/// esconde, aquela recusa tem de impedir que exista.
pub const RESERVADAS: [&str; 4] = [
    crate::dispositivo::ARCA_LOGS,
    "ARCA-DOCS",
    "$RECYCLE.BIN",
    "System Volume Information",
];

/// O parecer do `ocs-chkimg` sobre a integridade de uma imagem.
///
/// E independente do desfecho: um backup pode terminar e a imagem ser
/// reprovada.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Veredito {
    Aprovada,
    Reprovada,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Especie {
    /// Tem `MD5SUMS`. O veredito e `None` quando nao ha `arca-check.log`, ou
    /// quando ha e nao diz nada reconhecivel — nunca se supoe aprovada.
    Imagem { veredito: Option<Veredito> },

    /// Sem `MD5SUMS`: rastro de um backup interrompido.
    Residuo,
}

/// Uma pasta do `ARCAVAULT`, ja julgada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pasta {
    pub nome: String,
    pub tamanho_bytes: u64,

    /// Do sistema de arquivos, so para exibir. Ver [`crate::formato::dia_e_mes`].
    pub modificado_em: Option<DateTime<Local>>,

    pub especie: Especie,
}

impl Pasta {
    pub fn e_imagem(&self) -> bool {
        matches!(self.especie, Especie::Imagem { .. })
    }
}

/// Tudo que ha na raiz do `ARCAVAULT`, em ordem de nome.
pub fn enumerar(arquivos: &dyn Arquivos, raiz_do_vault: &Path) -> Resultado<Vec<Pasta>> {
    let mut pastas = Vec::new();

    for entrada in arquivos.listar(raiz_do_vault)? {
        if !entrada.diretorio {
            continue;
        }
        let nome = entrada.nome();
        if RESERVADAS
            .iter()
            .any(|reservada| reservada.eq_ignore_ascii_case(&nome))
        {
            continue;
        }

        let dentro = arquivos.listar(&entrada.caminho)?;
        let arquivo_chamado = |procurado: &str| {
            dentro
                .iter()
                .find(|item| !item.diretorio && item.nome().eq_ignore_ascii_case(procurado))
        };

        let especie = match arquivo_chamado(MD5SUMS) {
            Some(_) => Especie::Imagem {
                veredito: match arquivo_chamado(CHECK_LOG) {
                    Some(log) => interpretar_veredito(&arquivos.ler_texto_alheio(&log.caminho)?),
                    None => None,
                },
            },
            None => Especie::Residuo,
        };

        pastas.push(Pasta {
            nome,
            tamanho_bytes: somar(arquivos, &dentro, 0)?,
            modificado_em: entrada.modificado_em,
            especie,
        });
    }

    pastas.sort_by(|a, b| a.nome.cmp(&b.nome));
    Ok(pastas)
}

/// Quanto a pasta ocupa, contando o que houver em subpastas.
///
/// Uma imagem do Clonezilla e plana, mas um residuo e o que sobrou de uma
/// gravacao interrompida e nao tem forma garantida — dai a recursao.
///
/// O limite existe porque uma juncao NTFS apontando para um ancestral faria
/// a soma andar para sempre, e uma apontando para o `C:` faria `arca list`
/// varrer o disco do sistema. Nao ha juncao numa pasta de imagem, mas quem
/// lê o `ARCAVAULT` lê o que o usuario deixou la, e o comando que a E1
/// entrega nao pode travar por causa disso. Passando do limite, o que sobra
/// nao e contado: a coluna subestima em vez de nunca aparecer.
const PROFUNDIDADE_MAXIMA: u8 = 8;

fn somar(
    arquivos: &dyn Arquivos,
    entradas: &[crate::portas::Entrada],
    profundidade: u8,
) -> Resultado<u64> {
    let mut total = 0u64;
    for entrada in entradas {
        if !entrada.diretorio {
            total = total.saturating_add(entrada.tamanho_bytes);
            continue;
        }
        if profundidade >= PROFUNDIDADE_MAXIMA {
            continue;
        }
        let dentro = arquivos.listar(&entrada.caminho)?;
        total = total.saturating_add(somar(arquivos, &dentro, profundidade + 1)?);
    }
    Ok(total)
}

/// O veredito escrito no `arca-check.log`, ou `None` quando ele nao diz.
///
/// O arquivo tem duas formas no dispositivo, e as duas sao legitimas. A
/// receita de B-9 redireciona a saida crua do `ocs-chkimg`, cheia de escapes
/// de terminal; um log mais novo traz, no fim, a linha `ARCA_VEREDITO=`
/// acrescentada de proposito pela receita (E3). O marcador e o caminho
/// preferido — e algo que alguem escreveu para ser lido, e nao um texto de
/// terminal que se interpreta.
///
/// **Mas ele nao tem a ultima palavra sobre aprovar.** Toda forma de reprovar
/// vem antes de toda forma de aprovar: o marcador de reprovacao e o
/// `not restorable` do resumo reprovam os dois, e so depois de nenhum deles
/// aparecer e que se procura aprovacao. Um log de falha lista as particoes
/// que prestam junto da que nao presta, e a receita **acrescenta** a linha ao
/// log — as duas marcas cabem no mesmo arquivo, por mais de um caminho.
///
/// Nao havendo nem uma coisa nem outra, o veredito e `None`. Ausencia de
/// prova nunca vira aprovacao: imagem nao verificada e suposicao.
///
/// Registrado em `docs/adr/0003-veredito-lido-do-arca-check-log.md`, com o
/// ajuste da ordem em `docs/adr/0004-a-receita-transcreve-o-que-rodou.md`.
pub fn interpretar_veredito(texto: &str) -> Option<Veredito> {
    // Toda forma de reprovar vem antes de toda forma de aprovar, e nao
    // reprovacao-antes-de-aprovacao dentro de cada caminho. A diferenca
    // passou a importar na etapa E3: ate ela, o `ARCA_VEREDITO=APROVADA` so
    // existia porque **alguem o escreveu depois de olhar**. Agora quem o
    // escreve e a receita, a partir do codigo de saida do `ocs-chkimg`.
    //
    // Se o `ocs-chkimg` sair zero com um `NOT restorable` no texto — que e
    // P-6 aplicado a ele —, o log fica com as duas marcas. Deixar o marcador
    // decidir ali transformaria uma imagem quebrada em imagem aprovada, que e
    // o contrario de S-5. Qualquer sinal de reprovacao reprova.
    let minusculo = texto.to_lowercase();

    if texto.contains("ARCA_VEREDITO=REPROVADA") || minusculo.contains("not restorable") {
        return Some(Veredito::Reprovada);
    }

    // Sem nenhum sinal de reprovacao, o marcador decide — e ele e o caminho
    // preferido porque e algo que alguem escreveu para ser lido, e nao um
    // texto de terminal que se interpreta.
    if texto.contains("ARCA_VEREDITO=APROVADA")
        || minusculo.contains("were checked and are restorable")
    {
        return Some(Veredito::Aprovada);
    }

    None
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::duplos::ArquivosEmMemoria;
    use std::path::PathBuf;

    /// O fim do `arca-check.log` de `2026-08-21_WindowsCompleto`, copiado do
    /// dispositivo. Traz o marcador explicito.
    const CHECK_COM_MARCADOR: &str = concat!(
        "All partition and logical volume images in this image were checked",
        " and are restorable.: 2026-08-21_WindowsCompleto\n",
        "==========================\n",
        "\x1b[24d\x1b[K\x1b[24;1H\n",
        "ARCA_VEREDITO=APROVADA\n",
        "ARCA_FIM\n"
    );

    /// O `arca-check.log` de `ARCA-TESTE-03`, que e so a saida crua do
    /// `ocs-chkimg` — sem marcador nenhum.
    const CHECK_SEM_MARCADOR: &str = concat!(
        "\x1b[3;11HPartclone v0.3.47 http://partclone.org\n",
        "Checked successfully.\n",
        "This partition image is restorable: nvme0n1p4\n",
        "All partition and logical volume images in this image were checked",
        " and are restorable.: ARCA-TESTE-03\n"
    );

    /// Uma falha: o resumo lista particoes que prestam **e** uma que nao.
    const CHECK_COM_FALHA: &str = concat!(
        "This partition image is restorable: nvme0n1p1\n",
        "This partition image is NOT restorable: nvme0n1p3\n",
        "Some of the partition or logical volume images in this image are",
        " NOT restorable.: 2026-08-22_Apps\n"
    );

    fn vault() -> PathBuf {
        PathBuf::from("E:\\")
    }

    #[test]
    fn o_marcador_explicito_decide_o_veredito() {
        assert_eq!(
            interpretar_veredito(CHECK_COM_MARCADOR),
            Some(Veredito::Aprovada)
        );
        assert_eq!(
            interpretar_veredito("ARCA_VEREDITO=REPROVADA\nARCA_FIM\n"),
            Some(Veredito::Reprovada)
        );
    }

    #[test]
    fn sem_marcador_vale_o_resumo_do_ocs_chkimg() {
        assert_eq!(
            interpretar_veredito(CHECK_SEM_MARCADOR),
            Some(Veredito::Aprovada)
        );
    }

    #[test]
    fn com_os_dois_marcadores_no_mesmo_log_a_reprovacao_ganha() {
        // A receita **acrescenta** a linha ao `arca-check.log`. Uma imagem
        // verificada duas vezes fica com as duas marcas no arquivo, e ler a
        // aprovacao primeiro diria `aprovada` sobre uma imagem cuja ultima
        // verificacao reprovou.
        let log = concat!(
            "ARCA_VEREDITO=APROVADA\n",
            "ARCA_FIM\n",
            "This partition image is NOT restorable: nvme0n1p3\n",
            "ARCA_VEREDITO=REPROVADA\n",
            "ARCA_FIM\n"
        );

        assert_eq!(interpretar_veredito(log), Some(Veredito::Reprovada));
    }

    #[test]
    fn o_marcador_de_aprovacao_nao_apaga_uma_reprovacao_no_resumo() {
        // O caso que a etapa E3 criou. Ate ela, o `ARCA_VEREDITO=APROVADA` so
        // aparecia porque alguem o escreveu depois de olhar o log. Agora quem
        // o escreve e a receita, a partir do codigo de saida do `ocs-chkimg`
        // — e um `ocs-chkimg` que saisse zero com um `NOT restorable` no
        // texto (P-6 aplicado a ele) deixaria as duas marcas no arquivo.
        //
        // Se o marcador decidisse, esta imagem quebrada sairia como aprovada,
        // e sairia **por causa** de uma mudanca feita para melhorar a leitura
        // do veredito. Qualquer sinal de reprovacao reprova (S-5).
        let log = concat!(
            "This partition image is restorable: nvme0n1p1\n",
            "This partition image is NOT restorable: nvme0n1p3\n",
            "ARCA_VEREDITO=APROVADA\n",
            "ARCA_FIM\n"
        );

        assert_eq!(interpretar_veredito(log), Some(Veredito::Reprovada));
    }

    #[test]
    fn uma_particao_reprovada_reprova_a_imagem_inteira() {
        // O log de falha contem as duas frases. Se a de sucesso fosse
        // procurada primeiro, uma imagem quebrada sairia como aprovada — e
        // falha parcial e falha total (S-5).
        assert_eq!(
            interpretar_veredito(CHECK_COM_FALHA),
            Some(Veredito::Reprovada)
        );
    }

    #[test]
    fn log_que_nao_diz_nada_nao_vira_aprovacao() {
        assert_eq!(interpretar_veredito(""), None);
        assert_eq!(interpretar_veredito("Starting to check image (-)\n"), None);
    }

    #[test]
    fn imagem_tem_md5sums_e_residuo_nao() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc  nvme0n1p1")
            .com(
                r"E:\2026-08-21_WindowsCompleto\arca-check.log",
                CHECK_COM_MARCADOR,
            )
            .com(
                r"E:\2026-08-22_Interrompido\nvme0n1p3.ntfs-ptcl-img.zst.aa",
                "xx",
            );

        let pastas = enumerar(&arquivos, &vault()).unwrap();

        assert_eq!(pastas.len(), 2);
        assert_eq!(pastas[0].nome, "2026-08-21_WindowsCompleto");
        assert_eq!(
            pastas[0].especie,
            Especie::Imagem {
                veredito: Some(Veredito::Aprovada)
            }
        );
        assert_eq!(pastas[1].nome, "2026-08-22_Interrompido");
        assert_eq!(pastas[1].especie, Especie::Residuo);
    }

    #[test]
    fn a_data_da_pasta_chega_na_listagem() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc")
            .datado(r"E:\2026-08-21_WindowsCompleto", "2026-08-21T12:56:31");

        let pastas = enumerar(&arquivos, &vault()).unwrap();
        assert_eq!(
            pastas[0].modificado_em,
            Some(crate::duplos::momento("2026-08-21T12:56:31"))
        );
    }

    #[test]
    fn imagem_sem_check_log_fica_sem_veredito() {
        let arquivos = ArquivosEmMemoria::novo().com(r"E:\2026-08-22_Apps\MD5SUMS", "abc");

        let pastas = enumerar(&arquivos, &vault()).unwrap();
        assert_eq!(pastas[0].especie, Especie::Imagem { veredito: None });
    }

    #[test]
    fn pasta_vazia_e_residuo() {
        // Um backup interrompido antes do primeiro arquivo deixa isto. Nao
        // reconhece-lo esconderia um nome que o pre-voo vai recusar.
        let arquivos = ArquivosEmMemoria::novo().com_pasta_vazia(r"E:\2026-08-22_Apps");

        let pastas = enumerar(&arquivos, &vault()).unwrap();
        assert_eq!(pastas.len(), 1);
        assert_eq!(pastas[0].especie, Especie::Residuo);
        assert_eq!(pastas[0].tamanho_bytes, 0);
    }

    #[test]
    fn as_pastas_de_servico_nao_sao_imagem_nem_residuo() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(
                r"E:\ARCA-LOGS\2026-08-21_WindowsCompleto\arca-fim.txt",
                "ARCA_FIM",
            )
            .com(r"E:\ARCA-DOCS\progress.md", "# ...")
            .com(r"E:\System Volume Information\tracking.log", "x")
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "abc");

        let pastas = enumerar(&arquivos, &vault()).unwrap();

        assert_eq!(pastas.len(), 1);
        assert_eq!(pastas[0].nome, "2026-08-21_WindowsCompleto");
    }

    #[test]
    fn vault_que_nao_da_para_ler_e_erro_e_nao_lista_vazia() {
        // Uma raiz ilegivel nao pode virar "nenhuma imagem": e a diferenca
        // entre o dispositivo estar vazio e o ARCA nao ter conseguido olhar.
        let arquivos = ArquivosEmMemoria::novo();
        assert!(enumerar(&arquivos, &vault()).is_err());
    }

    #[test]
    fn arquivo_solto_na_raiz_nao_e_imagem() {
        let arquivos = ArquivosEmMemoria::novo().com(r"E:\restore.log", "...");
        assert!(enumerar(&arquivos, &vault()).unwrap().is_empty());
    }

    #[test]
    fn o_tamanho_soma_o_que_ha_dentro_inclusive_em_subpasta() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-22_Apps\MD5SUMS", "1234567890")
            .com(
                r"E:\2026-08-22_Apps\nvme0n1p3.ntfs-ptcl-img.zst.aa",
                "12345",
            )
            .com(r"E:\2026-08-22_Apps\resto\mais", "123");

        let pastas = enumerar(&arquivos, &vault()).unwrap();
        assert_eq!(pastas[0].tamanho_bytes, 18);
    }

    #[test]
    fn a_listagem_sai_em_ordem_de_nome() {
        let arquivos = ArquivosEmMemoria::novo()
            .com(r"E:\2026-08-22_Apps\MD5SUMS", "a")
            .com(r"E:\2026-08-21_WindowsCompleto\MD5SUMS", "a")
            .com_pasta_vazia(r"E:\ARCA-TESTE-03");

        let nomes: Vec<String> = enumerar(&arquivos, &vault())
            .unwrap()
            .into_iter()
            .map(|pasta| pasta.nome)
            .collect();

        assert_eq!(
            nomes,
            vec![
                "2026-08-21_WindowsCompleto",
                "2026-08-22_Apps",
                "ARCA-TESTE-03"
            ]
        );
    }
}
