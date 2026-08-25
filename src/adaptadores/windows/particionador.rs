//! A porta do particionamento, implementada pelos cmdlets de armazenamento.
//!
//! # Por que PowerShell, e a resposta é a mesma de [`super::wmi`]
//!
//! Os cmdlets `Get-Disk`, `Clear-Disk`, `Initialize-Disk`, `New-Partition` e
//! `Format-Volume` falam com o **Storage Management Provider** por WMI, no
//! namespace `root/Microsoft/Windows/Storage`. Chamar aquilo por COM direto
//! seriam centenas de linhas de `unsafe` sobre vtables cruas para invocar
//! métodos com parâmetros nomeados — e o `windows-sys` não traz os auxiliares
//! de COM que o `windows` tem.
//!
//! O caminho fechado é o terceiro: escrever a tabela de partição à mão exige
//! abrir o disco por caminho de dispositivo, que é exatamente o que
//! `tests/s1_nenhum_acesso_raw.rs` proíbe a cada build.
//!
//! # Tudo o que este arquivo faz foi medido à mão antes de virar código
//!
//! Em 23/08/2026, no segundo dispositivo desta mesa, e preservado em
//! `recursos/capturas/particionamento-medido-2026-08-23.txt`. Como a E7 fez
//! com o `bootsequence` e o C-13 com o `displayorder`.
//!
//! Quatro coisas saíram daquela medição, e três não eram óbvias:
//!
//! 1. **`New-Partition` cria as duas com `MbrType 6`**, e quem acerta para 7 e
//!    12 é o `Format-Volume`. Não há `Set-Partition -MbrType` no caminho — o
//!    tipo é efeito colateral de outra operação, e é por isso que a releitura
//!    de [`crate::preparacao::conferir_o_que_saiu`] importa.
//! 2. **`Clear-Disk` deixa o disco `RAW`**, com `LargestFreeExtent` em zero.
//!    Sem o `Initialize-Disk` depois, não há onde criar partição.
//! 3. **As duas nascem sem letra**, e o ARCA exige letra
//!    (`Erro::VolumeSemLetra`). Quem atribui é o
//!    `Add-PartitionAccessPath -AssignDriveLetter`.
//! 4. **`IsActive` sai `False` sozinho**, que é o que a captura da estrutura
//!    registra.
//!
//! # A letra é atribuída, e não escolhida
//!
//! Medido: `Set-Partition -NewDriveLetter C` responde *"The requested access
//! path is already in use"*. Escolher a letra é supor que ela está livre — e
//! S-3 diz que a letra não importa, o rótulo importa. `-AssignDriveLetter`
//! deixa o Windows escolher, e o ARCA lê qual foi.
//!
//! **E ele não é idempotente**, o que também foi medido: a segunda passada
//! responde *"Cannot assign multiple drive letters to a partition"* e **não
//! muda nada**. É exatamente o caso do `bcdedit /deletevalue` do
//! [ADR-0005](../../../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md):
//! *manda fazer, descarta o que a ferramenta responde, e pergunta de novo*.
//! Por isso o erro do `Add-PartitionAccessPath` não derruba nada — quem
//! responde é a releitura.

use crate::erro::{Erro, Resultado};

use crate::portas::particionador::{
    DiscoParaPreparar, ParticaoExistente, ParticaoFeita, Particionador, ParticoesFeitas,
    PlanoDeParticoes,
};
use crate::preparacao::{ARCABOOT, ARCAVAULT, TIPO_GPT_MSR, UNIDADE_DE_ALOCACAO};
use std::process::Command;

use super::texto::{de_pagina_de_codigo, pagina_do_console};
use super::wmi::{base64_de_utf16, cadeia, numero, objetos};

/// A consulta que descreve os discos, com o que as sete defesas precisam.
///
/// Junta as duas fontes de propósito, e a razão é a armadilha do ADR-0010: o
/// `MSFT_Disk` responde `FriendlyName`, `IsSystem`, `IsBoot` e o tamanho na
/// régua boa; o `Win32_DiskDrive` responde o `MediaType`, que é onde moram as
/// palavras `External hard disk media` (§3.1, D10). **Nenhuma das duas
/// responde tudo.**
const CONSULTA: &str = r#"$ProgressPreference='SilentlyContinue'
$midia = @{}
foreach ($w in (Get-CimInstance Win32_DiskDrive)) { $midia[[int]$w.Index] = @{ Tipo=$w.MediaType; Modelo=$w.Model } }
$discos = Get-Disk | ForEach-Object {
  $d = $_
  $m = $midia[[int]$d.Number]
  $parts = @(Get-Partition -DiskNumber $d.Number -ErrorAction SilentlyContinue | ForEach-Object {
    $p = $_
    $v = Get-Volume -Partition $p -ErrorAction SilentlyContinue
    [pscustomobject]@{ Numero=$p.PartitionNumber; Letra=[string]$p.DriveLetter; Rotulo=[string]$v.FileSystemLabel; Sistema=[string]$v.FileSystem; Tamanho=$p.Size }
  })
  [pscustomobject]@{ Indice=$d.Number; Modelo=$d.FriendlyName; ModeloWmi=$m.Modelo; Tamanho=$d.Size; Barramento=[string]$d.BusType; Midia=$m.Tipo; Estilo=[string]$d.PartitionStyle; Sistema=[bool]$d.IsSystem; Boot=[bool]$d.IsBoot; SomenteLeitura=[bool]$d.IsReadOnly; Particoes=$parts }
}
ConvertTo-Json -InputObject @($discos) -Compress -Depth 5"#;

#[derive(Debug, Clone, Copy, Default)]
pub struct ParticionadorDoWindows;

impl Particionador for ParticionadorDoWindows {
    fn descrever(&self, indice: u32) -> Resultado<Option<DiscoParaPreparar>> {
        Ok(self
            .enumerar()?
            .into_iter()
            .find(|disco| disco.indice == indice))
    }

    fn enumerar(&self) -> Resultado<Vec<DiscoParaPreparar>> {
        ler(&rodar(CONSULTA)?)
    }

    fn particionar(&self, plano: &PlanoDeParticoes) -> Resultado<ParticoesFeitas> {
        // **A falha do script tem de dizer que o disco pode já ter sido
        // apagado**, e a genérica de [`rodar`] não diz.
        //
        // O `Clear-Disk` é o primeiro passo, e é irreversível. Um erro que
        // chegue a quem lê como *"powershell recusou (codigo 1)"* deixa a
        // pergunta que mais importa sem resposta — e ela tem uma resposta
        // barata: se o script chegou a rodar, o `Clear-Disk` foi a primeira
        // coisa que ele fez.
        //
        // O `ler_o_que_saiu` já diz isso quando a resposta chega e não se
        // deixa ler; o que faltava era o caso em que ela **não chega**.
        let saida =
            rodar(&script_do_particionamento(plano)).map_err(|erro| Erro::FerramentaRecusou {
                ferramenta: "powershell",
                codigo: 0,
                saida: format!(
                    "o particionamento do disco {} falhou: {erro}. **O DISCO PODE JA TER SIDO \
                     APAGADO** — o primeiro passo do script e o `Clear-Disk`, e ele e \
                     irreversivel. Olhe o disco no Gerenciamento de Disco antes de concluir \
                     qualquer coisa, e rode `arca prepare --dispositivo {} --dry-run` para ver \
                     em que estado ele esta",
                    plano.indice_do_disco, plano.indice_do_disco
                ),
            })?;
        ler_o_que_saiu(&saida)
    }
}

/// O script que apaga, cria e formata — na ordem medida à mão.
///
/// # A ordem, e o que cada passo deixa se o seguinte não acontecer
///
/// | Passo | Se parar aqui, o disco fica |
/// |---|---|
/// | `Clear-Disk` | `RAW`, sem partição nenhuma. **O conteúdo já se foi.** |
/// | `Initialize-Disk` | GPT com uma MSR que o Windows criou sozinho |
/// | `Remove-Partition` | GPT vazio, pronto para receber partições |
/// | `New-Partition` ×2 | duas partições cruas, sem sistema de arquivos e sem letra |
/// | `Format-Volume` ×2 | as duas formatadas e rotuladas, ainda sem letra |
/// | `Add-PartitionAccessPath` ×2 | pronto |
///
/// **Nenhum desses estados é pior do que o anterior**, e o primeiro já é
/// irreversível — que é o ponto: o `Clear-Disk` é o ponto sem volta do
/// `arca prepare`, e tudo o que vem depois é construção. Um `prepare`
/// interrompido no meio deixa um disco vazio ou meio pronto, e rodá-lo de novo
/// resolve — ele começa apagando.
///
/// # A linha do `Remove-Partition`, que em MBR não existia
///
/// Em MBR o `Initialize-Disk` deixa o disco vazio. **Em GPT ele cria sozinho
/// uma *Microsoft Reserved*** de 16 759 808 bytes no offset 17 408, com
/// [`crate::preparacao::TIPO_GPT_MSR`] — medido em 25/08/2026 nos **dois**
/// dispositivos do marco, com os três números idênticos.
///
/// Deixá-la em pé faria a `ARCAVAULT` nascer partição 2 e a `ARCABOOT`
/// partição 3, mudaria o device path da entrada de firmware para
/// `HD(3,GPT,…)`, e faria a releitura ver três partições onde o plano pede
/// duas. Ela não serve para nada num dispositivo de dados.
///
/// **Removem-se todas, e não só as do tipo `Reserved`.** Neste ponto do script
/// o `Clear-Disk` acabou de rodar: o que existir aqui é obra do
/// `Initialize-Disk`, e a linha não precisa saber o nome do que remove para
/// estar certa. Duas medições bastam para isso ser um passo, e não uma
/// condicional — ver o
/// [ADR-0025](../../../docs/adr/0025-o-arca-particiona-em-gpt.md).
///
/// O `$ErrorActionPreference='Stop'` faz o script **parar no primeiro erro** em
/// vez de seguir construindo sobre um passo que falhou. É o contrário do que a
/// medição à mão fez de propósito (lá se queria ver o que cada passo respondia,
/// inclusive os que falham).
fn script_do_particionamento(plano: &PlanoDeParticoes) -> String {
    format!(
        r#"$ProgressPreference='SilentlyContinue'
$ErrorActionPreference='Stop'
$n = {indice}
Clear-Disk -Number $n -RemoveData -RemoveOEM -Confirm:$false
Initialize-Disk -Number $n -PartitionStyle GPT
Get-Partition -DiskNumber $n -ErrorAction SilentlyContinue | Remove-Partition -Confirm:$false
$p1 = New-Partition -DiskNumber $n -Size {vault}
$p2 = New-Partition -DiskNumber $n -UseMaximumSize
Format-Volume -Partition $p1 -FileSystem NTFS -NewFileSystemLabel '{vault_rotulo}' -AllocationUnitSize {unidade} -Force -Confirm:$false | Out-Null
Format-Volume -Partition $p2 -FileSystem FAT32 -NewFileSystemLabel '{boot_rotulo}' -AllocationUnitSize {unidade} -Force -Confirm:$false | Out-Null
foreach ($numero in @($p1.PartitionNumber, $p2.PartitionNumber)) {{
  try {{ Add-PartitionAccessPath -DiskNumber $n -PartitionNumber $numero -AssignDriveLetter -ErrorAction Stop }} catch {{ }}
}}
$saida = @(Get-Partition -DiskNumber $n | ForEach-Object {{
  $p = $_
  $v = Get-Volume -Partition $p -ErrorAction SilentlyContinue
  [pscustomobject]@{{ Numero=$p.PartitionNumber; Letra=[string]$p.DriveLetter; Rotulo=[string]$v.FileSystemLabel; Sistema=[string]$v.FileSystem; Tipo=[string]$p.GptType; Tamanho=$p.Size; Offset=$p.Offset; Unidade=$v.AllocationUnitSize; Ativa=[bool]$p.IsActive }}
}})
ConvertTo-Json -InputObject @($saida) -Compress -Depth 4"#,
        indice = plano.indice_do_disco,
        vault = plano.vault_bytes,
        vault_rotulo = ARCAVAULT,
        boot_rotulo = ARCABOOT,
        unidade = UNIDADE_DE_ALOCACAO,
    )
}

/// Roda um script e devolve o `stdout`.
///
/// **Somente `stdout`**, pela mesma razão de [`super::wmi`]: o
/// `-EncodedCommand` despeja CLIXML de progresso no `stderr`, e concatenar
/// colaria XML antes do JSON.
fn rodar(script: &str) -> Resultado<String> {
    let saida = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            &base64_de_utf16(script),
        ])
        .output()
        .map_err(|origem| Erro::Ferramenta {
            ferramenta: "powershell",
            origem,
        })?;

    let pagina = pagina_do_console();
    let texto = de_pagina_de_codigo(&saida.stdout, pagina);

    if !saida.status.success() {
        return Err(Erro::FerramentaRecusou {
            ferramenta: "powershell",
            codigo: saida.status.code().unwrap_or(-1),
            saida: de_pagina_de_codigo(&saida.stderr, pagina)
                .trim()
                .to_string(),
        });
    }

    Ok(texto)
}

/// Os discos, a partir do JSON.
///
/// Recusa o que não entende, como o leitor do [`super::wmi`] e o do
/// [`crate::estado`]: um disco que não se deixa ler inteiro **não** vira um
/// disco com campos zerados. Aqui isso vale mais do que em qualquer outro
/// leitor deste projeto — um `IsSystem` suposto `false` porque o campo não veio
/// é a defesa 2 de PR-5 desligada em silêncio.
pub fn ler(json: &str) -> Resultado<Vec<DiscoParaPreparar>> {
    let recusar = |porque: &str| Erro::FerramentaRecusou {
        ferramenta: "powershell",
        codigo: 0,
        saida: format!(
            "a consulta de discos do `arca prepare` respondeu algo que o ARCA nao entende \
             ({porque}). Sem ela nao da para saber qual disco e o do sistema, e o comando \
             que vem depois apaga um disco inteiro"
        ),
    };

    let mut discos = Vec::new();
    for objeto in objetos(json) {
        let indice = numero(&objeto, "Indice").ok_or_else(|| recusar("falta o Indice"))?;
        let modelo = cadeia(&objeto, "Modelo").ok_or_else(|| recusar("falta o Modelo"))?;
        let tamanho = numero(&objeto, "Tamanho").ok_or_else(|| recusar("falta o Tamanho"))?;

        // `IsSystem` e `IsBoot` sao **obrigatorios**, e nao ha valor padrao.
        // Um `false` suposto porque a chave nao veio e a defesa 2 desligada
        // sem que nada diga.
        let e_do_sistema =
            booleano(&objeto, "Sistema").ok_or_else(|| recusar("falta o IsSystem"))?;
        let e_de_boot = booleano(&objeto, "Boot").ok_or_else(|| recusar("falta o IsBoot"))?;

        discos.push(DiscoParaPreparar {
            indice: indice as u32,
            modelo,
            modelo_no_wmi: cadeia(&objeto, "ModeloWmi"),
            tamanho_bytes: tamanho,
            barramento: cadeia(&objeto, "Barramento").unwrap_or_default(),
            tipo_de_midia: super::wmi::tipo_de_midia(cadeia(&objeto, "Midia").as_deref()),
            estilo_de_particao: cadeia(&objeto, "Estilo").unwrap_or_default(),
            e_do_sistema,
            e_de_boot,
            // Este cai de volta em `false`, e a assimetria e deliberada: um
            // disco somente-leitura falharia no `Clear-Disk` de qualquer jeito,
            // e a recusa antecipada e conveniencia. As duas de cima sao defesa.
            somente_leitura: booleano(&objeto, "SomenteLeitura").unwrap_or(false),
            particoes: particoes(&objeto),
        });
    }

    Ok(discos)
}

/// O que saiu do particionamento, relido do disco (C-3).
fn ler_o_que_saiu(json: &str) -> Resultado<ParticoesFeitas> {
    let recusar = |porque: String| Erro::FerramentaRecusou {
        ferramenta: "powershell",
        codigo: 0,
        saida: format!(
            "o disco foi particionado e a releitura nao respondeu o que se espera ({porque}). \
             O DISCO JA FOI APAGADO — o que estivesse nele nao esta mais. Rode \
             `arca prepare --dispositivo <indice>` de novo, ou olhe o disco no Gerenciamento \
             de Disco"
        ),
    };

    // A letra vem como `Option` ate a contagem estar conferida, e a ordem das
    // duas conferencias e deliberada. Sobrando uma MSR — o `Remove-Partition`
    // do script tendo falhado —, ela nao tem letra, e exigir letra primeiro
    // faria o ARCA reclamar de letra faltando quando o problema e outro.
    // Contar primeiro deixa a recusa dizer o que de fato aconteceu.
    let mut lidas = Vec::new();
    for objeto in objetos(json) {
        let numero_da_particao =
            numero(&objeto, "Numero").ok_or_else(|| recusar("falta o numero".to_string()))?;
        let letra = cadeia(&objeto, "Letra")
            .and_then(|texto| texto.chars().next())
            .filter(char::is_ascii_alphabetic)
            .map(|letra| letra.to_ascii_uppercase());

        lidas.push((
            letra,
            ParticaoFeita {
                numero: numero_da_particao as u32,
                letra: '?',
                rotulo: cadeia(&objeto, "Rotulo").unwrap_or_default(),
                sistema_de_arquivos: cadeia(&objeto, "Sistema").unwrap_or_default(),
                tipo_gpt: cadeia(&objeto, "Tipo").unwrap_or_default(),
                tamanho_bytes: numero(&objeto, "Tamanho").unwrap_or(0),
                offset_bytes: numero(&objeto, "Offset").unwrap_or(0),
                unidade_de_alocacao: numero(&objeto, "Unidade").unwrap_or(0),
                ativa: booleano(&objeto, "Ativa").unwrap_or(false),
            },
        ));
    }

    // Duas, e exatamente duas. Um disco que voltasse com tres particoes teria
    // sobrado alguma coisa de antes — e escrever o Clonezilla em cima disso
    // produziria um dispositivo que ninguem sabe o que e.
    if lidas.len() != 2 {
        let msr = lidas
            .iter()
            .any(|(_, particao)| particao.tipo_gpt.eq_ignore_ascii_case(TIPO_GPT_MSR));
        let porque = if msr {
            // Vale nomear: a MSR e a unica particao que este comando espera
            // encontrar e mandar embora, e quem ler a recusa merece saber que
            // o passo que falhou tem endereco.
            format!(
                "o disco voltou com {} particoes, e o plano pede duas. Uma delas e a Microsoft \
                 Reserved que o `Initialize-Disk -PartitionStyle GPT` cria sozinho, e que o \
                 `Remove-Partition` devia ter tirado",
                lidas.len()
            )
        } else {
            format!(
                "o disco voltou com {} particoes, e o plano pede duas",
                lidas.len()
            )
        };
        return Err(recusar(porque));
    }

    let mut feitas = Vec::new();
    for (letra, particao) in lidas {
        let Some(letra) = letra else {
            return Err(recusar(format!(
                "a particao {} ficou SEM LETRA, e o ARCA precisa de uma para achar o `grub.cfg` \
                 e o `estado.json` do lado Windows",
                particao.numero
            )));
        };
        feitas.push(ParticaoFeita { letra, ..particao });
    }

    let [vault, boot] = <[ParticaoFeita; 2]>::try_from(feitas)
        .unwrap_or_else(|_| unreachable!("a contagem foi conferida logo acima"));

    Ok(ParticoesFeitas { vault, boot })
}

/// As partições de um objeto de disco.
fn particoes(objeto: &str) -> Vec<ParticaoExistente> {
    let Some(bruto) = fatia_do_vetor(objeto, "Particoes") else {
        return Vec::new();
    };

    objetos(bruto)
        .iter()
        .filter_map(|particao| {
            Some(ParticaoExistente {
                numero: numero(particao, "Numero")? as u32,
                letra: cadeia(particao, "Letra")
                    .and_then(|texto| texto.chars().next())
                    .filter(char::is_ascii_alphabetic),
                rotulo: cadeia(particao, "Rotulo").filter(|texto| !texto.is_empty()),
                sistema_de_arquivos: cadeia(particao, "Sistema").filter(|texto| !texto.is_empty()),
                tamanho_bytes: numero(particao, "Tamanho").unwrap_or(0),
            })
        })
        .collect()
}

/// O texto do vetor de `"chave":[...]`, com os colchetes.
///
/// Conta colchetes respeitando aspas, pela mesma razão que [`objetos`] conta
/// chaves: um rótulo de volume é texto livre do usuário, e `]` cabe nele.
fn fatia_do_vetor<'a>(objeto: &'a str, chave: &str) -> Option<&'a str> {
    let marca = format!("\"{chave}\":[");
    let inicio = objeto.find(&marca)? + marca.len() - 1;

    let mut profundidade = 0usize;
    let mut dentro_de_texto = false;
    let mut escapado = false;

    for (posicao, caractere) in objeto[inicio..].char_indices() {
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
            '[' => profundidade += 1,
            ']' => {
                profundidade -= 1;
                if profundidade == 0 {
                    return Some(&objeto[inicio..=inicio + posicao]);
                }
            }
            _ => {}
        }
    }
    None
}

/// O `true`/`false` depois de `"chave":`.
///
/// `None` quando a chave não está lá — e quem chama decide se isso é recusa ou
/// um padrão. Para `IsSystem` e `IsBoot` é recusa.
fn booleano(objeto: &str, chave: &str) -> Option<bool> {
    let marca = format!("\"{chave}\":");
    let posicao = objeto.find(&marca)? + marca.len();
    let resto = objeto[posicao..].trim_start();

    if resto.starts_with("true") {
        Some(true)
    } else if resto.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::portas::TipoDeMidia;
    use crate::preparacao::TIPO_GPT_DADOS_BASICOS;

    /// A resposta desta máquina em 23/08/2026, com os três discos na mesa.
    const DESTA_MESA: &str = concat!(
        r#"[{"Indice":1,"Modelo":"JMicron Generic","ModeloWmi":"JMicron Generic SCSI Disk Device","#,
        r#""Tamanho":480103981056,"Barramento":"USB","Midia":"External hard disk media","#,
        r#""Estilo":"MBR","Sistema":false,"Boot":false,"SomenteLeitura":false,"#,
        r#""Particoes":[{"Numero":1,"Letra":"E","Rotulo":"Dell Beta Apps NO IA WSL","Sistema":"NTFS","Tamanho":480099958784}]},"#,
        r#"{"Indice":2,"Modelo":"KGSSE100 256","ModeloWmi":"KGSSE100 256 SCSI Disk Device","#,
        r#""Tamanho":256060514304,"Barramento":"USB","Midia":"External hard disk media","#,
        r#""Estilo":"MBR","Sistema":false,"Boot":false,"SomenteLeitura":false,"#,
        r#""Particoes":[{"Numero":1,"Letra":"D","Rotulo":"ARCAVAULT","Sistema":"NTFS","Tamanho":254379294720},"#,
        r#"{"Numero":2,"Letra":"R","Rotulo":"ARCABOOT","Sistema":"FAT32","Tamanho":1677721600}]},"#,
        r#"{"Indice":0,"Modelo":"KINGSTON SNV3S500G","ModeloWmi":"KINGSTON SNV3S500G","#,
        r#""Tamanho":500107862016,"Barramento":"NVMe","Midia":"Fixed hard disk media","#,
        r#""Estilo":"GPT","Sistema":true,"Boot":true,"SomenteLeitura":false,"#,
        r#""Particoes":[{"Numero":3,"Letra":"C","Rotulo":"Windows","Sistema":"NTFS","Tamanho":498701697024}]}]"#
    );

    #[test]
    fn os_tres_discos_desta_mesa_sao_lidos() {
        let discos = ler(DESTA_MESA).expect("o JSON desta mesa se lê");
        assert_eq!(discos.len(), 3);

        let jmicron = &discos[0];
        assert_eq!(jmicron.indice, 1);
        assert_eq!(jmicron.modelo, "JMicron Generic");
        assert_eq!(
            jmicron.modelo_no_wmi.as_deref(),
            Some("JMicron Generic SCSI Disk Device")
        );
        assert_eq!(jmicron.tamanho_bytes, 480_103_981_056);
        assert_eq!(jmicron.tipo_de_midia, TipoDeMidia::DiscoExterno);
        assert!(!jmicron.e_do_sistema && !jmicron.e_de_boot);
        assert_eq!(jmicron.particoes.len(), 1);
        assert_eq!(
            jmicron.particoes[0].rotulo.as_deref(),
            Some("Dell Beta Apps NO IA WSL")
        );
    }

    #[test]
    fn os_dois_modelos_do_mesmo_disco_sao_guardados_separados() {
        // Nesta mesa o `MSFT_Disk` diz `JMicron Generic` e o WMI diz `JMicron
        // Generic SCSI Disk Device`. A tela pede um deles na confirmacao
        // digitada; guardar so um faria a defesa julgar um texto e a tela
        // afirmar outro sem que ninguem visse.
        let discos = ler(DESTA_MESA).unwrap();
        assert_ne!(discos[0].modelo, discos[0].modelo_no_wmi.clone().unwrap());
    }

    #[test]
    fn o_disco_do_windows_vem_marcado_nos_dois_campos() {
        let discos = ler(DESTA_MESA).unwrap();
        let interno = discos
            .iter()
            .find(|disco| disco.indice == 0)
            .expect("o disco 0");

        assert!(interno.e_do_sistema, "IsSystem");
        assert!(interno.e_de_boot, "IsBoot");
        assert_eq!(interno.tipo_de_midia, TipoDeMidia::DiscoFixo);
        assert_eq!(interno.letras(), vec!['C']);
    }

    #[test]
    fn o_dispositivo_ja_preparado_traz_os_dois_rotulos() {
        let discos = ler(DESTA_MESA).unwrap();
        let dispositivo = discos
            .iter()
            .find(|disco| disco.indice == 2)
            .expect("o disco 2");

        let rotulos: Vec<&str> = dispositivo
            .particoes
            .iter()
            .filter_map(|particao| particao.rotulo.as_deref())
            .collect();
        assert_eq!(rotulos, vec!["ARCAVAULT", "ARCABOOT"]);
        assert_eq!(dispositivo.letras(), vec!['D', 'R']);
    }

    #[test]
    fn um_issystem_que_falta_e_recusa_e_nao_um_false() {
        // A defesa 2 de PR-5 e "nao e o disco do sistema". Um `false` suposto
        // porque a chave nao veio a desliga em silencio — e o comando que vem
        // depois apaga um disco inteiro.
        let completo = r#"{"Indice":0,"Modelo":"X","ModeloWmi":"X","Tamanho":1000,"Barramento":"USB","Midia":"External hard disk media","Estilo":"MBR","Sistema":true,"Boot":false,"SomenteLeitura":false,"Particoes":[]}"#;
        assert!(ler(&format!("[{completo}]")).is_ok());

        for faltando in [
            r#""Sistema":true,"#,
            r#""Boot":false,"#,
            r#""Indice":0,"#,
            r#""Tamanho":1000,"#,
        ] {
            let sem = completo.replace(faltando, "");
            assert!(
                ler(&format!("[{sem}]")).is_err(),
                "passou sem `{faltando}`:\n{sem}"
            );
        }
    }

    #[test]
    fn um_rotulo_com_colchete_nao_reparte_a_lista_de_particoes() {
        // O rotulo de um volume e texto livre de quem formatou. `]` cabe nele,
        // e uma leitura que contasse colchetes sem saber onde comecam as aspas
        // truncaria a lista — fazendo uma particao **sumir** da tela de PR-4,
        // que e a tela que existe para mostrar o que vai ser destruido.
        let json = concat!(
            r#"[{"Indice":1,"Modelo":"X","ModeloWmi":"X","Tamanho":1000,"Barramento":"USB","#,
            r#""Midia":"External hard disk media","Estilo":"MBR","Sistema":false,"Boot":false,"#,
            r#""SomenteLeitura":false,"Particoes":[{"Numero":1,"Letra":"E","Rotulo":"foto]s [2024]","Sistema":"NTFS","Tamanho":500},"#,
            r#"{"Numero":2,"Letra":"F","Rotulo":"backup","Sistema":"NTFS","Tamanho":400}]}]"#
        );

        let discos = ler(json).unwrap();
        assert_eq!(discos[0].particoes.len(), 2, "uma particao sumiu da tela");
        assert_eq!(
            discos[0].particoes[0].rotulo.as_deref(),
            Some("foto]s [2024]")
        );
    }

    #[test]
    fn um_disco_sem_particao_nenhuma_e_lido_como_vazio() {
        // O caso normal do `arca prepare`: um disco em branco. Lista vazia nao
        // e erro — e a resposta, e a tela de PR-4 diz "nao ha particao".
        let json = r#"[{"Indice":1,"Modelo":"X","ModeloWmi":"X","Tamanho":1000,"Barramento":"USB","Midia":"Removable Media","Estilo":"RAW","Sistema":false,"Boot":false,"SomenteLeitura":false,"Particoes":[]}]"#;

        let discos = ler(json).unwrap();
        assert!(discos[0].particoes.is_empty());
        assert!(discos[0].letras().is_empty());
    }

    #[test]
    fn uma_particao_sem_letra_e_sem_rotulo_aparece_assim_mesmo() {
        // Uma particao crua nao tem volume, entao nao tem rotulo nem sistema
        // de arquivos. Ela **continua na tela** — o que vai ser destruido
        // aparece inteiro, e nao so o que o Windows soube nomear.
        let json = r#"[{"Indice":1,"Modelo":"X","ModeloWmi":"X","Tamanho":1000,"Barramento":"USB","Midia":"Removable Media","Estilo":"MBR","Sistema":false,"Boot":false,"SomenteLeitura":false,"Particoes":[{"Numero":1,"Letra":"","Rotulo":"","Sistema":"","Tamanho":900}]}]"#;

        let discos = ler(json).unwrap();
        assert_eq!(discos[0].particoes.len(), 1);
        assert_eq!(discos[0].particoes[0].letra, None);
        assert_eq!(discos[0].particoes[0].rotulo, None);
        assert_eq!(discos[0].particoes[0].tamanho_bytes, 900);
    }

    // ─────────────────── o que saiu do particionamento ───────────────────

    /// O que o Windows respondeu em 25/08/2026, depois de o particionamento em
    /// GPT rodar à mão no KGSSE100 256 — o dispositivo que bootou no marco.
    ///
    /// Note o `Tipo`: **o mesmo nas duas**, e é assim que a captura o registra.
    const O_QUE_SAIU: &str = concat!(
        r#"[{"Numero":1,"Letra":"D","Rotulo":"ARCAVAULT","Sistema":"NTFS","#,
        r#""Tipo":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","#,
        r#""Tamanho":254381391872,"Offset":1048576,"Unidade":4096,"Ativa":false},"#,
        r#"{"Numero":2,"Letra":"E","Rotulo":"ARCABOOT","Sistema":"FAT32","#,
        r#""Tipo":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","#,
        r#""Tamanho":1677721600,"Offset":254382440448,"Unidade":4096,"Ativa":false}]"#
    );

    #[test]
    fn a_releitura_medida_em_hardware_se_lê() {
        let feitas = ler_o_que_saiu(O_QUE_SAIU).expect("a resposta medida se lê");

        assert_eq!(feitas.vault.letra, 'D');
        assert_eq!(feitas.vault.rotulo, "ARCAVAULT");
        assert_eq!(feitas.vault.tipo_gpt, TIPO_GPT_DADOS_BASICOS);
        assert_eq!(feitas.vault.offset_bytes, 1_048_576);

        assert_eq!(feitas.boot.letra, 'E');
        assert_eq!(feitas.boot.rotulo, "ARCABOOT");
        assert_eq!(feitas.boot.tipo_gpt, TIPO_GPT_DADOS_BASICOS);
        assert_eq!(feitas.boot.tamanho_bytes, 1_677_721_600);

        // O achado que muda o criterio: o tipo e o **mesmo** nas duas, e nao
        // serve mais para dizer qual e qual.
        assert_eq!(feitas.vault.tipo_gpt, feitas.boot.tipo_gpt);

        // E ela passa na conferencia de PR-5.
        assert_eq!(crate::preparacao::conferir_o_que_saiu(&feitas), Ok(()));
    }

    #[test]
    fn uma_particao_sem_letra_derruba_o_particionamento() {
        // Medido: `New-Partition` **nao** atribui letra, e o
        // `Add-PartitionAccessPath` que a atribui pode falhar. Sem letra o ARCA
        // nao acha o `grub.cfg` nem o `estado.json` — e o disco ja foi apagado
        // quando isto se descobre, entao a mensagem tem de dizer isso.
        let sem_letra = O_QUE_SAIU.replace(r#""Letra":"E""#, r#""Letra":"""#);

        let erro = ler_o_que_saiu(&sem_letra).unwrap_err();
        assert!(erro.to_string().contains("SEM LETRA"), "{erro}");
        assert!(erro.to_string().contains("JA FOI APAGADO"), "{erro}");
    }

    #[test]
    fn tres_particoes_sao_recusa() {
        // Um disco que voltasse com tres teria sobrado alguma coisa de antes, e
        // escrever o Clonezilla em cima disso produziria um dispositivo que
        // ninguem sabe o que e.
        let com_tres = O_QUE_SAIU.replace(
            r#""Ativa":false}]"#,
            r#""Ativa":false},{"Numero":3,"Letra":"G","Rotulo":"SOBRA","Sistema":"NTFS","Tipo":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Tamanho":1,"Offset":2,"Unidade":4096,"Ativa":false}]"#,
        );

        let erro = ler_o_que_saiu(&com_tres).unwrap_err();
        assert!(erro.to_string().contains("3 particoes"), "{erro}");
    }

    #[test]
    fn a_msr_sobrevivente_e_recusada_pelo_nome_e_nao_por_falta_de_letra() {
        // O caso que o GPT trouxe e o MBR nao tinha: o `Initialize-Disk` cria
        // uma Microsoft Reserved sozinho, o script a remove, e se essa remocao
        // falhar o disco volta com tres. A MSR **nao tem letra** — e a recusa
        // tem de falar da particao a mais, que e o problema, e nao da letra que
        // falta, que e consequencia. E por isso que `ler_o_que_saiu` conta
        // antes de exigir letra.
        let com_msr = O_QUE_SAIU.replace(
            r#"[{"Numero":1"#,
            &format!(
                r#"[{{"Numero":1,"Letra":"","Rotulo":"","Sistema":"","Tipo":"{TIPO_GPT_MSR}","Tamanho":16759808,"Offset":17408,"Unidade":0,"Ativa":false}},{{"Numero":1"#
            ),
        );

        let erro = ler_o_que_saiu(&com_msr).unwrap_err();
        assert!(erro.to_string().contains("3 particoes"), "{erro}");
        assert!(erro.to_string().contains("Microsoft Reserved"), "{erro}");
        assert!(
            !erro.to_string().contains("SEM LETRA"),
            "a MSR nao tem letra, e reclamar disso esconderia o que houve: {erro}"
        );
    }

    #[test]
    fn uma_particao_so_tambem_e_recusa() {
        let com_uma = format!(
            "[{}]",
            O_QUE_SAIU
                .trim_start_matches('[')
                .split("},{")
                .next()
                .unwrap()
                .to_string()
                + "}"
        );

        assert!(ler_o_que_saiu(&com_uma).is_err());
    }

    // ─────────────────── o script ───────────────────

    #[test]
    fn o_script_transcreve_a_sequencia_medida() {
        let script = script_do_particionamento(&PlanoDeParticoes {
            indice_do_disco: 1,
            vault_bytes: 478_423_285_760,
            boot_bytes: 1_677_721_600,
        });

        // A ordem dos seis passos, e ela nao e negociavel: `Clear-Disk` deixa
        // o disco RAW e sem espaco livre, entao o `Initialize-Disk` **tem** de
        // vir depois — e o `Remove-Partition` que tira a MSR tem de vir entre
        // o `Initialize-Disk`, que a cria, e o `New-Partition`, que numeraria
        // as duas a partir dela.
        let posicoes: Vec<usize> = [
            "Clear-Disk",
            "Initialize-Disk",
            "Remove-Partition",
            "New-Partition",
            "Format-Volume",
            "Add-PartitionAccessPath",
        ]
        .iter()
        .map(|passo| {
            script
                .find(passo)
                .unwrap_or_else(|| panic!("sumiu o {passo}"))
        })
        .collect();

        assert!(
            posicoes.windows(2).all(|par| par[0] < par[1]),
            "a ordem dos passos mudou:\n{script}"
        );
    }

    #[test]
    fn o_script_nao_marca_particao_ativa() {
        // A captura registra `IsActive: False` nas duas, e e isso que confirma
        // que o boot do dispositivo e UEFI puro. Um `-IsActive` aqui seria
        // inventar uma estrutura em vez de transcrever a medida (ADR-0014).
        let script = script_do_particionamento(&PlanoDeParticoes {
            indice_do_disco: 1,
            vault_bytes: 100,
            boot_bytes: 200,
        });

        assert!(!script.contains("-IsActive"), "{script}");
        assert!(!script.contains("Set-Partition"), "{script}");
    }

    #[test]
    fn o_script_inicializa_em_gpt_e_tira_a_msr() {
        // O ADR-0014 mandava resistir a "modernizar para GPT" porque seria
        // trocar um esquema medido por um suposto. O ADR-0025 troca por outro
        // **medido**: em 25/08/2026 um dispositivo GPT bootou, e o device path
        // foi lido de dentro do boot pelo `efibootmgr`.
        //
        // A MSR e o que o GPT trouxe de novo. Sem a linha que a remove, a
        // `ARCAVAULT` nasceria particao 2, a `ARCABOOT` particao 3, e o device
        // path da entrada de firmware viraria `HD(3,GPT,...)`.
        let script = script_do_particionamento(&PlanoDeParticoes {
            indice_do_disco: 1,
            vault_bytes: 100,
            boot_bytes: 200,
        });

        assert!(script.contains("-PartitionStyle GPT"), "{script}");
        assert!(!script.contains("-PartitionStyle MBR"), "{script}");
        assert!(script.contains("Remove-Partition"), "{script}");
    }

    #[test]
    fn o_script_le_o_gpttype_de_volta_e_nao_o_mbrtype() {
        // Em GPT o `MbrType` sai **vazio** — nao zero, ausente —, e ler um
        // campo ausente como numero daria `0` em silencio, que passaria por
        // uma conferencia frouxa. O que a releitura le e o `GptType`, que e o
        // campo que existe.
        let script = script_do_particionamento(&PlanoDeParticoes {
            indice_do_disco: 1,
            vault_bytes: 100,
            boot_bytes: 200,
        });

        assert!(script.contains("[string]$p.GptType"), "{script}");
        assert!(!script.contains("MbrType"), "{script}");
    }

    #[test]
    fn o_script_carrega_o_indice_e_os_dois_rotulos() {
        let script = script_do_particionamento(&PlanoDeParticoes {
            indice_do_disco: 7,
            vault_bytes: 123_456,
            boot_bytes: 1_677_721_600,
        });

        assert!(script.contains("$n = 7"), "{script}");
        assert!(script.contains("-Size 123456"), "{script}");
        assert!(script.contains("'ARCAVAULT'"), "{script}");
        assert!(script.contains("'ARCABOOT'"), "{script}");
        assert!(script.contains("-AllocationUnitSize 4096"), "{script}");
    }

    #[test]
    fn o_script_para_no_primeiro_erro() {
        // Sem isto, um `Clear-Disk` que falhasse deixaria o `New-Partition`
        // rodar sobre a tabela antiga — e o resultado seria um disco meio
        // preparado que a releitura teria de desfazer.
        let script = script_do_particionamento(&PlanoDeParticoes {
            indice_do_disco: 1,
            vault_bytes: 100,
            boot_bytes: 200,
        });

        assert!(script.contains("$ErrorActionPreference='Stop'"), "{script}");
    }

    #[test]
    fn a_consulta_traz_o_issystem_e_o_mediatype_das_duas_fontes() {
        // A armadilha do ADR-0010 aplicada aqui: **nenhuma das duas fontes
        // responde tudo**. O `MSFT_Disk` tem `IsSystem` e o tamanho na regua
        // boa; o `Win32_DiskDrive` tem o `MediaType`, que e onde moram as
        // palavras da §3.1.
        assert!(CONSULTA.contains("IsSystem"), "sumiu o IsSystem");
        assert!(CONSULTA.contains("IsBoot"), "sumiu o IsBoot");
        assert!(CONSULTA.contains("Win32_DiskDrive"), "sumiu o MediaType");
        assert!(CONSULTA.contains("Get-Disk"), "sumiu o Get-Disk");
    }

    #[test]
    fn a_consulta_silencia_o_progresso() {
        // Medido na E6: sem isto o `-EncodedCommand` despeja CLIXML no stderr.
        assert!(CONSULTA.starts_with("$ProgressPreference='SilentlyContinue'"));
    }

    #[test]
    fn a_consulta_nao_pede_o_caminho_de_dispositivo() {
        // S-1, como em `super::wmi`: o que nao se pede nao chega.
        assert!(!CONSULTA.contains("DeviceID"), "{CONSULTA}");
        assert!(!CONSULTA.contains("Path="), "{CONSULTA}");
    }
}
