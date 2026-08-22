//! Diagnostico: em que codificacao o `bcdedit` escreve quando o ARCA o chama?
//!
//! A duvida era concreta e cara. O adaptador
//! [`arca::adaptadores::windows::firmware::Bcdedit`] fazia
//! `String::from_utf8_lossy` sobre os bytes que o `bcdedit` devolveu. Se o
//! `bcdedit` escreve numa pagina de codigo OEM — 850 num Windows em portugues
//! —, cada acento vira `U+FFFD` em silencio. E este e o unico parser do
//! sistema cuja leitura errada leva a maquina a bootar no lugar errado com uma
//! receita armada.
//!
//! Este exemplo respondeu medindo. Roda o `bcdedit` pelo mesmo caminho que o
//! ARCA usa — `Command::output`, que da ao filho um par de canos no lugar da
//! saida — e olha os **bytes crus**: quais nao sao ASCII, quantos `U+FFFD` o
//! `from_utf8_lossy` produziria, e o que sai decodificando pela pagina de
//! codigo do console.
//!
//! Mede tambem se a pagina de codigo do **console de quem chama** muda a
//! resposta. Ela importa porque o ARCA elevado nao herda o console de onde foi
//! digitado: o UAC lhe da uma janela nova, com a pagina de codigo padrao da
//! maquina.
//!
//! # O que ele mediu, em 22/08/2026
//!
//! | console de quem chama | o `bcdedit` escreveu | `from_utf8_lossy` |
//! |---|---|---|
//! | 850 (`GetOEMCP` desta maquina) | CP850 | 6 caracteres perdidos |
//! | 65001 | UTF-8 | nenhum |
//!
//! A pagina **nao e fixa**: e a do console de quem chamou, e o filho a herda
//! junto do console. Dai a correcao —
//! [`arca::adaptadores::windows::texto::pagina_do_console`] —, e dai o teste
//! `o_texto_que_chega_do_bcdedit_nao_tem_caractere_perdido`, que passou a
//! cobrar isso contra a ferramenta de verdade.
//!
//! Continua aqui porque quem duvidar da correcao mede de novo, em vez de
//! confiar na tabela acima.
//!
//! Precisa de privilegio administrativo, porque `bcdedit /enum` precisa. Um
//! exemplo nao carrega o manifesto do `arca.exe`; quem o roda tem de ja estar
//! elevado.

#[cfg(windows)]
fn main() {
    use arca::portas::Firmware;
    use std::process::Command;
    use windows_sys::Win32::Globalization::{
        GetACP, GetOEMCP, MB_ERR_INVALID_CHARS, MultiByteToWideChar,
    };
    use windows_sys::Win32::System::Console::{GetConsoleOutputCP, SetConsoleOutputCP};

    /// Os bytes decodificados por uma pagina de codigo, ou o motivo de nao
    /// darem. `MB_ERR_INVALID_CHARS` faz a funcao recusar em vez de inventar:
    /// e assim que se distingue "esta e a codificacao certa" de "coube".
    fn decodificar(bytes: &[u8], pagina: u32) -> Result<String, String> {
        // SEGURANCA: o comprimento informado e o da propria fatia; o primeiro
        // passo pergunta o tamanho necessario, com ponteiro de saida nulo.
        let largura = unsafe {
            MultiByteToWideChar(
                pagina,
                MB_ERR_INVALID_CHARS,
                bytes.as_ptr(),
                bytes.len() as i32,
                std::ptr::null_mut(),
                0,
            )
        };
        if largura <= 0 {
            return Err(format!("a pagina {pagina} recusou estes bytes"));
        }

        let mut largo = vec![0u16; largura as usize];
        // SEGURANCA: o destino e o vetor recem-dimensionado por esta chamada.
        let escritos = unsafe {
            MultiByteToWideChar(
                pagina,
                MB_ERR_INVALID_CHARS,
                bytes.as_ptr(),
                bytes.len() as i32,
                largo.as_mut_ptr(),
                largura,
            )
        };
        Ok(String::from_utf16_lossy(&largo[..escritos as usize]))
    }

    /// Os bytes que o `bcdedit` escreveu, pelo mesmo caminho do adaptador.
    fn bytes_do_bcdedit() -> Vec<u8> {
        let saida = Command::new("bcdedit")
            .args(["/enum", "firmware"])
            .output()
            .expect("o bcdedit tem de rodar");
        saida.stdout
    }

    /// O que se aprende de um punhado de bytes do `bcdedit`.
    fn examinar(rotulo: &str, bytes: &[u8], achados: &mut Vec<String>) {
        let nao_ascii: Vec<String> = {
            let mut vistos: Vec<u8> = bytes.iter().copied().filter(|b| *b > 127).collect();
            vistos.sort_unstable();
            vistos.dedup();
            vistos.iter().map(|b| format!("{b:02X}")).collect()
        };

        let perdido = String::from_utf8_lossy(bytes)
            .chars()
            .filter(|c| *c == '\u{FFFD}')
            .count();

        achados.push(format!(
            "\n[{rotulo}] {} bytes · nao-ASCII distintos: [{}] · U+FFFD do from_utf8_lossy: {perdido}",
            bytes.len(),
            nao_ascii.join(" ")
        ));

        // A linha do cabecalho traduzido e a primeira com acento. E ela que
        // separa uma decodificacao certa de uma que so nao quebrou.
        let primeira_suja = bytes
            .split(|b| *b == b'\n')
            .find(|linha| linha.iter().any(|b| *b > 127))
            .map(|linha| linha.to_vec())
            .unwrap_or_default();

        for pagina in [850u32, 65001, 1252] {
            let lido = match decodificar(&primeira_suja, pagina) {
                Ok(texto) => format!("{:?}", texto.trim_end()),
                Err(motivo) => motivo,
            };
            achados.push(format!("  CP{pagina:<6} {lido}"));
        }
        achados.push(format!(
            "  lossy    {:?}",
            String::from_utf8_lossy(&primeira_suja).trim_end()
        ));
    }

    let mut achados = Vec::new();

    // SEGURANCA: as tres consultas de pagina de codigo nao recebem ponteiro.
    let (console, oem, ansi) = unsafe { (GetConsoleOutputCP(), GetOEMCP(), GetACP()) };
    achados.push(format!(
        "paginas de codigo deste processo: console={console} OEM={oem} ANSI={ansi}"
    ));

    examinar(
        &format!("console como veio ({console})"),
        &bytes_do_bcdedit(),
        &mut achados,
    );

    // A pergunta que decide o desenho do adaptador: a saida do filho segue a
    // pagina de codigo do console de quem chamou?
    for pagina in [850u32, 65001] {
        // SEGURANCA: a funcao recebe so a pagina de codigo.
        let trocou = unsafe { SetConsoleOutputCP(pagina) };
        let efetiva = unsafe { GetConsoleOutputCP() };
        if trocou == 0 {
            achados.push(format!(
                "\n[CP {pagina}] o console recusou a troca; ficou {efetiva}"
            ));
            continue;
        }
        examinar(
            &format!("console em {efetiva}"),
            &bytes_do_bcdedit(),
            &mut achados,
        );
    }

    // SEGURANCA: devolve o console como estava. O ARCA nunca deixa alterado um
    // estado que pertence ao console de quem chamou.
    unsafe { SetConsoleOutputCP(console) };

    // E, por fim, o que o adaptador de verdade entrega — que e o numero que
    // interessa, porque e o texto que chega ao parser.
    let pela_porta = arca::adaptadores::windows::firmware::Bcdedit
        .enumerar("firmware")
        .expect("o bcdedit tem de responder");
    achados.push(format!(
        "\n[pela porta Firmware::enumerar] {} caracteres · U+FFFD: {}",
        pela_porta.chars().count(),
        pela_porta.chars().filter(|c| *c == '\u{FFFD}').count()
    ));
    achados.push(format!(
        "  cabecalho traduzido: {:?}",
        pela_porta
            .lines()
            .find(|linha| !linha.is_ascii())
            .unwrap_or("<nenhuma linha com acento>")
    ));

    let destino = std::env::args().nth(1).expect("o caminho do relatorio");
    std::fs::write(destino, achados.join("\n")).expect("relatorio gravado");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("so faz sentido no Windows");
}
