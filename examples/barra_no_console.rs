//! Diagnostico: o `█` e o `░` da barra do download ocupam uma coluna cada no
//! console?
//!
//! A duvida e concreta, e e o Unresolved 4 da WPC-220. A linha `Baixando
//! Clonezilla` e reescrita no lugar e tem de caber em 120 colunas, e o teste
//! dela conta caracteres. Isso so vale se cada caractere ocupar uma coluna. O
//! `·` ja foi medido em `examples/ponto_no_console.rs`. O `█` (U+2588) e de
//! largura "ambigua" no Unicode, e o `░` (U+2591), neutra: conferido na
//! `EastAsianWidth.txt` do Unicode 18.0 em 23/10/2026, na revisao do PR 1, que
//! achou este texto dizendo que os dois eram ambiguos. Um console que
//! desenhasse algum deles com duas colunas faria a linha quebrar — e uma linha
//! que quebra nao se reescreve, ela se repete.
//!
//! Este exemplo responde medindo, e nao supondo: imprime cada marca pelo
//! `print!` do Rust, como o ARCA imprime, e lê de volta quanto o cursor andou e
//! o que o console **desenhou**.
//!
//! Medido em 23/10/2026, nesta mesa, com a CP 850: `[█░]` avancou o cursor 4
//! colunas, e uma linha viva de 109 caracteres ocupou 109 colunas, com o que
//! foi desenhado igual ao que foi escrito.
//!
//! Precisa de um console de verdade, como o `ponto_no_console`:
//! `Start-Process` sem redirecionar a saida, com o caminho do relatorio como
//! argumento.

#[cfg(windows)]
fn main() {
    use std::io::Write;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Console::{
        CONSOLE_SCREEN_BUFFER_INFO, COORD, GetConsoleOutputCP, GetConsoleScreenBufferInfo,
        GetStdHandle, ReadConsoleOutputCharacterW, STD_OUTPUT_HANDLE,
    };

    /// A coluna e a linha do cursor, ou `None` quando o console nao responde.
    fn cursor(saida: HANDLE) -> Option<(i16, i16)> {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        (unsafe { GetConsoleScreenBufferInfo(saida, &mut info) } != 0)
            .then_some((info.dwCursorPosition.X, info.dwCursorPosition.Y))
    }

    /// Imprime a marca no comeco de uma linha nova e diz quanto o cursor andou
    /// e o que ficou desenhado.
    fn medir(saida: HANDLE, marca: &str) -> String {
        let Some((antes, linha)) = cursor(saida) else {
            return format!("{marca:?}: <sem informacao de buffer>");
        };
        print!("{marca}");
        let _ = std::io::stdout().flush();
        let depois = cursor(saida).map_or(-1, |(x, _)| x);

        let caracteres = marca.chars().count() as u32;
        let mut buffer = vec![0u16; caracteres as usize];
        let mut lidos = 0u32;
        let inicio = COORD { X: 0, Y: linha };
        let ok = unsafe {
            ReadConsoleOutputCharacterW(saida, buffer.as_mut_ptr(), caracteres, inicio, &mut lidos)
        };
        let desenhado = if ok == 0 {
            "<nao consegui ler o buffer>".to_string()
        } else {
            String::from_utf16_lossy(&buffer[..lidos as usize])
        };
        println!();

        format!(
            "{marca:?}: {caracteres} caracteres, o cursor andou {} colunas, desenhado igual = {}",
            depois - antes,
            desenhado == marca
        )
    }

    let saida = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let mut achados = vec![format!("CP = {}", unsafe { GetConsoleOutputCP() })];
    for marca in [
        "[·]",
        "[█░]",
        "  Baixando Clonezilla ............. [████████░░░░░░░░░░░░]  41% · 220,0 de 535,5 MB · 5,1 MB/s · faltam 1 min",
    ] {
        achados.push(medir(saida, marca));
    }

    let destino = std::env::args().nth(1).expect("o caminho do relatorio");
    std::fs::write(destino, achados.join("\n")).expect("relatorio gravado");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("so faz sentido no Windows");
}
