//! A enumeracao de discos fisicos, pelo WMI.
//!
//! # O que o WMI resolve, e que tres coisas separadas dependiam dele
//!
//! Medido em 22/08/2026, nesta maquina, **sem elevacao e sem abrir handle
//! nenhum**:
//!
//! ```text
//! Index Model                          MediaType
//! 0     KINGSTON SNV3S500G             Fixed hard disk media
//! 1     KGSSE100 256 SCSI Disk Device  External hard disk media
//!
//! Disco #1 -> E: [ARCAVAULT]  e  R: [ARCABOOT]
//! ```
//!
//! - **Fecha a pendencia de [`crate::dispositivo::Dispositivo::boot`]**: os
//!   dois rotulos estao no mesmo disco fisico, e agora da para provar.
//! - **Traz `External hard disk media`**, as palavras da §3.1 do PRD que o
//!   `bcdedit` nao produz (D10). E o sinal antecipado de C-6, e e melhor do
//!   que o `GetDriveType`, que classifica o SSD externo como disco fixo.
//! - **Traz o tamanho e as letras por disco**, que e o que B-4 precisa.
//!
//! # Por que processo filho, e nao COM
//!
//! O `Cargo.toml` tem `Win32_System_Com` desde a E0 e ninguem usou. COM direto
//! seria o caminho nativo, sem processo filho e sem parsing — e sao centenas de
//! linhas de `unsafe` sobre vtables cruas (o `windows-sys` nao tem os
//! auxiliares de COM que o `windows` tem) para **uma** consulta. O preco nao
//! paga.
//!
//! O caminho fechado e o terceiro: pedir os extents de volume ao driver exige
//! a chamada de controle de dispositivo que `tests/s1_nenhum_acesso_raw.rs`
//! proibe a cada build. Ampliar as portas e permitido; passar por cima do
//! teste, nao.
//!
//! O nome dessa chamada nao aparece escrito aqui de proposito, e o motivo vale
//! ser dito: a varredura de S-1 e de **texto**, e nao distingue uma mencao de
//! um uso. Ela pegou a primeira versao deste comentario, que a soletrava. Isso
//! e um acerto do teste, e nao um defeito — o que torna essas varreduras
//! confiaveis e serem burras demais para serem enganadas. Quem quiser o nome
//! exato o encontra no proprio `tests/s1_nenhum_acesso_raw.rs`, que e onde ele
//! deve morar.
//!
//! # Tres armadilhas medidas nesta etapa
//!
//! **1. O CLIXML vai para o stderr.** Com `-EncodedCommand`, o PowerShell
//! despeja registros de progresso em CLIXML no **stderr** — 628 bytes de
//! `#< CLIXML ... Preparando modulos para primeiro uso` nesta maquina. O
//! stdout sai limpo. Isto importa porque
//! [`crate::adaptadores::windows::firmware::Bcdedit`] **concatena stdout e
//! stderr**, e copiar aquele padrao para ca colaria XML antes do JSON. Aqui
//! se lê stdout, e so.
//!
//! **2. `$ProgressPreference='SilentlyContinue'` zera esse stderr.** Medido:
//! com ele, o stderr sai com zero byte. Nao e redundante com o item 1 — e o
//! que torna um stderr **nao vazio** uma informacao de verdade.
//!
//! **3. O `ConvertTo-Json` do PowerShell 5.1 nao escapa nao-ASCII.** Medido:
//! um valor acentuado sai com bytes crus na pagina de codigo do console. A
//! esperanca de que o JSON fosse ASCII por construcao estava errada, e
//! `de_pagina_de_codigo` continua obrigatorio — o `Model` de um disco e texto
//! livre do fabricante.
//!
//! # O `DeviceID` e descartado de proposito
//!
//! O `Win32_DiskDrive` devolve `DeviceID` como, literalmente, o caminho de
//! dispositivo bruto do disco. **Receber essa string como dado nao seria abrir
//! o dispositivo**, e o teste de S-1 varre o codigo-fonte e nao os valores de
//! runtime. Mas escreve-la no fonte para casar com ela faria o teste falhar —
//! e falhar com razao, porque ai ela deixa de ser dado e vira caminho.
//!
//! O ARCA nao pede esse campo. O `Index` e a chave estavel e util, e nao ha
//! nada que o `DeviceID` responda e o `Index` nao responda. Descartar e mais
//! barato do que carregar um caminho de dispositivo pela memoria do processo
//! esperando que ninguem o use.

use crate::erro::{Erro, Resultado};
use crate::portas::{DiscoFisico, TipoDeMidia};
use std::process::Command;

use super::texto::{de_pagina_de_codigo, pagina_do_console};

/// A consulta, exatamente como ela roda.
///
/// Pede **so** o que o ARCA usa. `Select-Object` explicito e nao `*`: o que
/// nao se pede nao chega, e o `DeviceID` e justamente o que nao se quer ver.
const CONSULTA: &str = r#"$ProgressPreference='SilentlyContinue'
$discos = Get-CimInstance Win32_DiskDrive | ForEach-Object {
  $d = $_
  $letras = @(Get-CimAssociatedInstance -InputObject $d -ResultClassName Win32_DiskPartition | ForEach-Object {
    Get-CimAssociatedInstance -InputObject $_ -ResultClassName Win32_LogicalDisk | ForEach-Object { $_.DeviceID }
  })
  $livre = 0
  foreach ($l in $letras) {
    $ld = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='$l'"
    if ($ld) { $livre += $ld.FreeSpace }
  }
  [pscustomobject]@{ Index=$d.Index; Model=$d.Model; Size=$d.Size; MediaType=$d.MediaType; Letras=$letras; Livre=$livre }
}
ConvertTo-Json -InputObject @($discos) -Compress -Depth 4"#;

/// Roda a consulta e devolve o JSON que ela imprimiu.
pub fn consultar() -> Resultado<String> {
    // `-EncodedCommand` com UTF-16LE em base64: nao ha aspa a escapar, nao ha
    // linha a repartir, e a C-8 — que e sobre quem reparte a linha do Windows
    // — deixa de ter o que morder. Passar um script de varias linhas por
    // `-Command` seria pedir para descobrir uma regra de citacao nova.
    let codificada = base64_de_utf16(CONSULTA);

    let saida = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            &codificada,
        ])
        .output()
        .map_err(|origem| Erro::Ferramenta {
            ferramenta: "powershell",
            origem,
        })?;

    let pagina = pagina_do_console();

    // **Somente stdout.** O `Bcdedit` concatena os dois porque o `bcdedit`
    // escreve a recusa no stderr; aqui o stderr carrega CLIXML de progresso, e
    // concatenar colaria XML antes do JSON.
    let texto = de_pagina_de_codigo(&saida.stdout, pagina);

    if !saida.status.success() {
        return Err(Erro::FerramentaRecusou {
            ferramenta: "powershell",
            codigo: saida.status.code().unwrap_or(-1),
            // Aqui sim o stderr interessa: e onde a recusa aparece. Com o
            // progresso silenciado, um stderr nao vazio quer dizer alguma
            // coisa.
            saida: de_pagina_de_codigo(&saida.stderr, pagina).trim().to_string(),
        });
    }

    Ok(texto)
}

/// UTF-16LE em base64, que e o que `-EncodedCommand` espera.
///
/// Escrito a mao pelo mesmo motivo do `estado.json` (ADR-0006): sao dezoito
/// linhas e nenhuma arvore de dependencias.
fn base64_de_utf16(texto: &str) -> String {
    const ALFABETO: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let bytes: Vec<u8> = texto
        .encode_utf16()
        .flat_map(|unidade| unidade.to_le_bytes())
        .collect();

    let mut saida = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for trio in bytes.chunks(3) {
        let a = trio[0] as u32;
        let b = *trio.get(1).unwrap_or(&0) as u32;
        let c = *trio.get(2).unwrap_or(&0) as u32;
        let junto = (a << 16) | (b << 8) | c;

        saida.push(ALFABETO[(junto >> 18) as usize & 63] as char);
        saida.push(ALFABETO[(junto >> 12) as usize & 63] as char);
        saida.push(if trio.len() > 1 {
            ALFABETO[(junto >> 6) as usize & 63] as char
        } else {
            '='
        });
        saida.push(if trio.len() > 2 {
            ALFABETO[junto as usize & 63] as char
        } else {
            '='
        });
    }
    saida
}

/// Os discos, a partir do JSON que a consulta imprimiu.
///
/// O leitor e o mesmo de [`crate::estado`] em espirito: recusa o que nao
/// entende em vez de adivinhar. Um disco que nao se deixe lê inteiro nao vira
/// um disco com campos zerados — zero em `tamanho_bytes` faria a regra de B-4
/// aprovar qualquer coisa.
pub fn ler(json: &str) -> Resultado<Vec<DiscoFisico>> {
    let recusar = |porque: &str| Erro::FerramentaRecusou {
        ferramenta: "powershell",
        codigo: 0,
        saida: format!(
            "a consulta ao WMI respondeu algo que o ARCA nao entende ({porque}). Sem a \
             enumeracao de discos nao da para provar que o ARCAVAULT e o ARCABOOT sao do mesmo \
             dispositivo, nem para conferir o espaco de B-4"
        ),
    };

    let mut discos = Vec::new();
    for objeto in objetos(json) {
        // Os quatro sao exigidos, e nao tres com dois adivinhados. A consulta
        // sempre os emite; faltar algum quer dizer que a resposta nao e a que
        // este leitor conhece, e ai o certo e recusar alto. Um `Model` vazio
        // viajaria ate a linha `Origem:` do §5.2 e ate a mensagem "nenhum
        // disco ... tem o modelo ``", e um `Livre` suposto zero faria o disco
        // parecer cheio sem que nada dissesse por que.
        let indice = numero(&objeto, "Index").ok_or_else(|| recusar("falta o Index"))?;
        let tamanho_bytes = numero(&objeto, "Size").ok_or_else(|| recusar("falta o Size"))?;
        let livre = numero(&objeto, "Livre").ok_or_else(|| recusar("falta o Livre"))?;
        let modelo = cadeia(&objeto, "Model").ok_or_else(|| recusar("falta o Model"))?;

        discos.push(DiscoFisico {
            indice: indice as u32,
            modelo,
            tamanho_bytes,
            // Tudo que nao esta livre num volume com letra conta como em uso.
            // Superestima, e superestimar e o lado seguro de "cabe uma
            // imagem?" — ver o campo em [`DiscoFisico`].
            em_uso_bytes: tamanho_bytes.saturating_sub(livre),
            tipo_de_midia: tipo_de_midia(cadeia(&objeto, "MediaType").as_deref()),
            letras: letras(&objeto),
        });
    }

    Ok(discos)
}

/// O `MediaType` do WMI, que e onde as palavras da §3.1 do PRD moram.
///
/// Casado por **prefixo em minusculas**, e nao por igualdade: a cadeia vem
/// traduzida em algumas instalacoes e o WMI acrescenta sufixos. O que nao se
/// reconhece vira [`TipoDeMidia::Desconhecido`], e nunca `DiscoFixo` por
/// padrao — supor que um disco desconhecido e interno e o erro que faria C-6
/// passar batido.
fn tipo_de_midia(bruto: Option<&str>) -> TipoDeMidia {
    let Some(bruto) = bruto else {
        return TipoDeMidia::Desconhecido;
    };
    let minusculo = bruto.to_lowercase();

    if minusculo.starts_with("external") {
        TipoDeMidia::DiscoExterno
    } else if minusculo.starts_with("removable") {
        TipoDeMidia::Removivel
    } else if minusculo.starts_with("fixed") {
        TipoDeMidia::DiscoFixo
    } else {
        TipoDeMidia::Desconhecido
    }
}

// ─────────────────── o leitor de JSON, do subconjunto que chega ───────────────────

/// Os objetos de nivel superior do vetor, como fatias de texto.
///
/// Nao e um parser de JSON — e o leitor do que a consulta acima produz. Ela
/// devolve sempre um vetor de objetos planos, sem objeto aninhado: o unico
/// vetor interno e `Letras`, de cadeias.
///
/// # A barra invertida importa, e a primeira versao disto a ignorava
///
/// `Model` e texto livre do fabricante, e `SAMSUNG 2.5" SSD` e um modelo
/// plausivel. O `ConvertTo-Json` escapa essa aspa como `\"`, e uma aspa
/// escapada e um numero **impar** de `"` na cadeia: ignorar a barra inverte a
/// paridade do resto do arquivo. Dai em diante o `}` de cada objeto passa a
/// ser visto como estando dentro de texto, e **dois discos se fundem num so
/// em silencio**.
///
/// O silencio e a parte grave. Se o disco que sumisse fosse o do dispositivo
/// ARCA, as duas recusas que dependem de saber em que disco cada letra mora —
/// C-6 e C-10, em [`crate::prevoo::julgar`] — passariam sem dizer nada.
/// Achado pela revisao da etapa E6.
fn objetos(json: &str) -> Vec<String> {
    let mut saida = Vec::new();
    let mut profundidade = 0usize;
    let mut inicio = 0usize;
    let mut dentro_de_texto = false;
    let mut escapado = false;

    for (posicao, caractere) in json.char_indices() {
        if dentro_de_texto {
            if escapado {
                escapado = false;
            } else if caractere == '\\' {
                escapado = true;
            } else if caractere == '"' {
                dentro_de_texto = false;
            }
            continue;
        }
        match caractere {
            '"' => dentro_de_texto = true,
            '{' => {
                if profundidade == 0 {
                    inicio = posicao;
                }
                profundidade += 1;
            }
            '}' => {
                profundidade = profundidade.saturating_sub(1);
                if profundidade == 0 {
                    saida.push(json[inicio..=posicao].to_string());
                }
            }
            _ => {}
        }
    }
    saida
}

/// O texto entre aspas depois de `"chave":`, com os escapes desfeitos.
///
/// Desfaz o que o `ConvertTo-Json` escreve e nada mais: `\"`, `\\`, `\/` e as
/// quebras. Uma sequencia que este leitor nao conheca fica como veio, com a
/// barra — perder um caractere de um modelo e melhor do que recusar a
/// enumeracao inteira, e o modelo so serve para casar com o `blkdev.list`, que
/// compara sem pontuacao de qualquer forma.
fn cadeia(objeto: &str, chave: &str) -> Option<String> {
    let resto = depois_da_chave(objeto, chave)?;
    let mut caracteres = resto.strip_prefix('"')?.chars();

    let mut saida = String::new();
    loop {
        match caracteres.next()? {
            '"' => return Some(saida),
            '\\' => match caracteres.next()? {
                '"' => saida.push('"'),
                '\\' => saida.push('\\'),
                '/' => saida.push('/'),
                'n' => saida.push('\n'),
                'r' => saida.push('\r'),
                't' => saida.push('\t'),
                outro => {
                    saida.push('\\');
                    saida.push(outro);
                }
            },
            outro => saida.push(outro),
        }
    }
}

/// O numero depois de `"chave":`.
fn numero(objeto: &str, chave: &str) -> Option<u64> {
    let resto = depois_da_chave(objeto, chave)?;
    let fim = resto
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(resto.len());
    resto[..fim].parse().ok()
}

/// As letras de `"Letras":["C:","D:"]`.
fn letras(objeto: &str) -> Vec<char> {
    let Some(resto) = depois_da_chave(objeto, "Letras") else {
        return Vec::new();
    };
    let Some(resto) = resto.strip_prefix('[') else {
        return Vec::new();
    };
    let Some(fim) = resto.find(']') else {
        return Vec::new();
    };

    resto[..fim]
        .split(',')
        .filter_map(|item| {
            item.trim()
                .trim_matches('"')
                .chars()
                .next()
                .filter(char::is_ascii_alphabetic)
        })
        .collect()
}

fn depois_da_chave<'a>(objeto: &'a str, chave: &str) -> Option<&'a str> {
    let marca = format!("\"{chave}\":");
    let posicao = objeto.find(&marca)? + marca.len();
    Some(objeto[posicao..].trim_start())
}

#[cfg(test)]
mod testes {
    use super::*;

    /// A resposta desta maquina, copiada da execucao de 22/08/2026.
    const DESTA_MAQUINA: &str = concat!(
        r#"[{"Index":0,"Model":"KINGSTON SNV3S500G","Size":500105249280,"#,
        r#""MediaType":"Fixed hard disk media","Letras":["C:"],"Livre":387131686912},"#,
        r#"{"Index":1,"Model":"KGSSE100 256 SCSI Disk Device","Size":256052966400,"#,
        r#""MediaType":"External hard disk media","Letras":["E:","R:"],"Livre":177392508928}]"#
    );

    #[test]
    fn os_dois_discos_desta_maquina_sao_lidos() {
        let discos = ler(DESTA_MAQUINA).expect("o JSON desta maquina se lê");
        assert_eq!(discos.len(), 2);

        assert_eq!(discos[0].indice, 0);
        assert_eq!(discos[0].modelo, "KINGSTON SNV3S500G");
        assert_eq!(discos[0].tamanho_bytes, 500_105_249_280);
        assert_eq!(discos[0].tipo_de_midia, TipoDeMidia::DiscoFixo);
        assert_eq!(discos[0].letras, vec!['C']);

        assert_eq!(discos[1].indice, 1);
        assert_eq!(discos[1].letras, vec!['E', 'R']);
    }

    #[test]
    fn o_ssd_externo_sai_como_externo_e_e_isso_que_o_bcdedit_nao_diz() {
        // As palavras da §3.1 do PRD saem daqui, e nao do `bcdedit` (D10). E o
        // sinal antecipado de C-6 — melhor do que o `GetDriveType`, que
        // classifica este mesmo disco como fixo.
        let discos = ler(DESTA_MAQUINA).unwrap();
        assert_eq!(discos[1].tipo_de_midia, TipoDeMidia::DiscoExterno);
    }

    #[test]
    fn os_dois_rotulos_estao_no_mesmo_disco_fisico() {
        // A prova que fecha a pendencia de `Dispositivo::boot`.
        let discos = ler(DESTA_MAQUINA).unwrap();
        let com_os_dois = discos
            .iter()
            .find(|disco| disco.tem_a_letra('E') && disco.tem_a_letra('R'))
            .expect("o ARCAVAULT e o ARCABOOT no mesmo disco");

        assert_eq!(com_os_dois.indice, 1);
    }

    #[test]
    fn o_em_uso_conta_o_disco_e_nao_so_os_volumes_com_letra() {
        // O disco 0 tem quatro particoes e so o `C:` tem letra. As outras tres
        // somam ~1,3 GB que a soma por volume ignoraria — e o
        // `Win32_DiskPartition` nem enxerga a MSR.
        let discos = ler(DESTA_MAQUINA).unwrap();
        assert_eq!(discos[0].em_uso_bytes, 500_105_249_280 - 387_131_686_912);

        // E o numero e maior do que o "em uso no `C:`" que o `Win32_LogicalDisk`
        // daria — medido: 498_701_692_928 de tamanho.
        let so_o_volume = 498_701_692_928u64 - 387_131_686_912;
        assert!(
            discos[0].em_uso_bytes > so_o_volume,
            "contar so o volume com letra subestima"
        );
    }

    #[test]
    fn midia_removivel_e_reconhecida() {
        let json = r#"[{"Index":2,"Model":"USB","Size":1000,"MediaType":"Removable Media","Letras":["F:"],"Livre":500}]"#;
        assert_eq!(ler(json).unwrap()[0].tipo_de_midia, TipoDeMidia::Removivel);
    }

    #[test]
    fn midia_que_nao_se_reconhece_nao_vira_disco_fixo() {
        // Supor que o desconhecido e interno faria C-6 passar batido: o aviso
        // so aparece para o que se sabe ser removivel, e o silencio leria como
        // "e um disco normal".
        for bruto in [
            r#""MediaType":null"#,
            r#""MediaType":"Alguma coisa nova""#,
            r#""MediaType":"""#,
        ] {
            let json =
                format!(r#"[{{"Index":0,"Model":"X","Size":1000,{bruto},"Letras":[],"Livre":0}}]"#);
            assert_eq!(
                ler(&json).unwrap()[0].tipo_de_midia,
                TipoDeMidia::Desconhecido,
                "`{bruto}` virou outra coisa"
            );
        }
    }

    #[test]
    fn um_campo_que_falta_e_recusa_e_nao_um_valor_inventado() {
        // Zero em `tamanho_bytes` faria a regra de espaco de B-4 aprovar
        // qualquer coisa: `em uso × 0,45` daria zero, e todo dispositivo teria
        // espaco de sobra. Modelo vazio viajaria ate a linha `Origem:` do
        // §5.2. Os quatro campos sao exigidos — a revisao da E6 pegou dois
        // que adivinhavam.
        let completo = r#"{"Index":0,"Model":"X","Size":1000,"MediaType":"Fixed hard disk media","Letras":["C:"],"Livre":100}"#;
        assert!(ler(&format!("[{completo}]")).is_ok(), "o completo tem de passar");

        for faltando in [
            r#""Index":0,"#,
            r#""Size":1000,"#,
            r#""Livre":100"#,
            r#""Model":"X","#,
        ] {
            let sem = completo.replace(faltando, "");
            assert!(
                ler(&format!("[{sem}]")).is_err(),
                "passou sem `{faltando}`:\n{sem}"
            );
        }
    }

    // ─────────── a barra invertida, achada pela revisao da E6 ───────────

    #[test]
    fn um_modelo_com_aspa_escapada_nao_funde_dois_discos() {
        // `SAMSUNG 2.5" SSD` e um modelo plausivel, e o `ConvertTo-Json`
        // escapa a aspa. Uma aspa escapada e um numero **impar** de `"` na
        // cadeia: ignorar a barra inverte a paridade do resto do arquivo, o
        // `}` de cada objeto passa a ser lido como estando dentro de texto, e
        // dois discos se fundem num so **em silencio**.
        let json = concat!(
            r#"[{"Index":0,"Model":"SAMSUNG 2.5\" SSD","Size":1000,"#,
            r#""MediaType":"Fixed hard disk media","Letras":["C:"],"Livre":100},"#,
            r#"{"Index":1,"Model":"KGSSE100","Size":2000,"#,
            r#""MediaType":"External hard disk media","Letras":["E:","R:"],"Livre":200}]"#
        );

        let discos = ler(json).expect("os dois discos se leem");

        assert_eq!(discos.len(), 2, "os dois objetos se fundiram");
        assert_eq!(discos[0].modelo, "SAMSUNG 2.5\" SSD");
        assert_eq!(discos[1].modelo, "KGSSE100");
        assert_eq!(discos[1].letras, vec!['E', 'R']);
    }

    #[test]
    fn o_disco_do_dispositivo_nao_some_por_causa_de_uma_aspa() {
        // Por que o silencio era grave: se o disco que sumisse fosse o do
        // dispositivo ARCA, as duas recusas que dependem de saber em que disco
        // cada letra mora — C-6 e C-10 — passariam sem dizer nada.
        let json = concat!(
            r#"[{"Index":0,"Model":"ACME 3\" DISK","Size":1000,"#,
            r#""MediaType":"Fixed hard disk media","Letras":["C:"],"Livre":100},"#,
            r#"{"Index":1,"Model":"PENDRIVE","Size":2000,"#,
            r#""MediaType":"Removable Media","Letras":["E:","R:"],"Livre":200}]"#
        );

        let discos = ler(json).unwrap();
        let do_dispositivo = discos
            .iter()
            .find(|disco| disco.tem_a_letra('R'))
            .expect("o disco do dispositivo continua na lista");

        assert_eq!(do_dispositivo.tipo_de_midia, TipoDeMidia::Removivel);
    }

    #[test]
    fn a_barra_invertida_no_modelo_atravessa() {
        let json = r#"[{"Index":0,"Model":"A\\B","Size":1,"MediaType":"Fixed hard disk media","Letras":[],"Livre":0}]"#;
        assert_eq!(ler(json).unwrap()[0].modelo, "A\\B");
    }

    #[test]
    fn a_lista_vazia_nao_e_erro() {
        // Uma maquina sem disco enumeravel e absurda, e ainda assim: lista
        // vazia e uma resposta, e quem decide o que fazer com ela e o pre-voo.
        assert!(ler("[]").unwrap().is_empty());
    }

    #[test]
    fn o_modelo_com_virgula_ou_chave_nao_reparte_o_objeto() {
        // O `Model` e texto livre do fabricante. Um `{` dentro dele repartiria
        // a leitura se ela contasse chaves sem saber onde comecam as aspas.
        let json = r#"[{"Index":0,"Model":"AC{ME, Inc} }","Size":1000,"MediaType":"Fixed hard disk media","Letras":["C:"],"Livre":100}]"#;
        let discos = ler(json).unwrap();

        assert_eq!(discos.len(), 1, "o objeto foi repartido");
        assert_eq!(discos[0].modelo, "AC{ME, Inc} }");
        assert_eq!(discos[0].tamanho_bytes, 1000);
    }

    // ───────────────────── o `-EncodedCommand` ─────────────────────

    #[test]
    fn o_base64_de_utf16_e_o_que_o_powershell_espera() {
        // UTF-16LE, e nao UTF-8: `A` vira `41 00`, e o base64 disso e `QQA=`.
        assert_eq!(base64_de_utf16("A"), "QQA=");
        assert_eq!(base64_de_utf16("AB"), "QQBCAA==");
        assert_eq!(base64_de_utf16("ABC"), "QQBCAEMA");
        assert_eq!(base64_de_utf16(""), "");
    }

    #[test]
    fn o_base64_so_usa_o_alfabeto_e_fecha_em_multiplo_de_quatro() {
        let codificada = base64_de_utf16(CONSULTA);

        assert_eq!(codificada.len() % 4, 0);
        assert!(
            codificada
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='),
            "saiu caractere fora do alfabeto base64"
        );
    }

    #[test]
    fn a_consulta_nao_pede_o_caminho_de_dispositivo() {
        // S-1: o `DeviceID` do `Win32_DiskDrive` e o caminho de dispositivo
        // bruto. O ARCA nao o pede — o que nao se pede nao chega.
        assert!(
            !CONSULTA.contains("DeviceID=$d.DeviceID"),
            "a consulta passou a trazer o caminho de dispositivo do disco"
        );
        assert!(CONSULTA.contains("Index=$d.Index"), "sumiu o Index");
    }

    #[test]
    fn a_consulta_silencia_o_progresso() {
        // Medido: sem isto o `-EncodedCommand` despeja 628 bytes de CLIXML no
        // stderr. Com isto o stderr sai vazio — e e o que faz um stderr nao
        // vazio querer dizer alguma coisa.
        assert!(CONSULTA.starts_with("$ProgressPreference='SilentlyContinue'"));
    }
}
