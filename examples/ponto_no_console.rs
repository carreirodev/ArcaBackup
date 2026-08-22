//! Diagnostico: o `·` do §5.4 sobrevive ao console sem mexer na pagina de
//! codigo?
//!
//! A duvida e concreta. O separador entre as colunas de uma imagem e um
//! `·` (U+00B7), e a pagina de codigo padrao de um Windows em portugues e a
//! 850. Se o `print!` do Rust entregasse bytes UTF-8 crus ao console, ele
//! sairia como `┬À` — e o ARCA precisaria trocar a pagina de codigo, mexendo
//! num estado que pertence ao console de quem chamou.
//!
//! Este exemplo responde medindo, e nao supondo: imprime, lê de volta o que
//! o console **desenhou** e compara. Roda sem elevacao, porque exemplos nao
//! carregam o manifesto do `arca.exe`.
//!
//! Precisa de um console de verdade: `Start-Process` sem redirecionar a saida.
//! Redirecionada, ela nem passa pelo console e a pergunta perde o sentido.

#[cfg(windows)]
fn main() {
    use std::io::Write;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::Console::{
        CONSOLE_SCREEN_BUFFER_INFO, COORD, GetConsoleOutputCP, GetConsoleScreenBufferInfo,
        GetStdHandle, ReadConsoleOutputCharacterW, STD_OUTPUT_HANDLE, SetConsoleOutputCP,
    };

    /// O que o console desenhou na linha onde o cursor esta.
    fn linha_desenhada(saida: HANDLE, quantos: u32) -> String {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        if unsafe { GetConsoleScreenBufferInfo(saida, &mut info) } == 0 {
            return "<sem informacao de buffer>".to_string();
        }

        let inicio = COORD {
            X: 0,
            Y: info.dwCursorPosition.Y,
        };
        let mut buffer = vec![0u16; quantos as usize];
        let mut lidos = 0u32;

        let ok = unsafe {
            ReadConsoleOutputCharacterW(saida, buffer.as_mut_ptr(), quantos, inicio, &mut lidos)
        };
        if ok == 0 {
            return "<nao consegui ler o buffer>".to_string();
        }

        String::from_utf16_lossy(&buffer[..lidos as usize])
    }

    /// Imprime a marca, lê de volta e diz se o `·` chegou inteiro.
    fn medir(saida: HANDLE, rotulo: &str) -> String {
        print!("[·]");
        let _ = std::io::stdout().flush();

        let desenhado = linha_desenhada(saida, 3);
        let pontos: Vec<String> = desenhado
            .chars()
            .map(|c| format!("U+{:04X}", c as u32))
            .collect();
        println!();

        format!(
            "{rotulo}: desenhado={desenhado:?} pontos=[{}] intacto={}",
            pontos.join(" "),
            desenhado.chars().nth(1) == Some('·')
        )
    }

    let saida = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let mut achados = Vec::new();

    let original = unsafe { GetConsoleOutputCP() };
    achados.push(format!("CP inicial = {original}"));
    achados.push(medir(saida, "com a CP do sistema"));

    unsafe { SetConsoleOutputCP(65001) };
    achados.push(format!("CP agora = {}", unsafe { GetConsoleOutputCP() }));
    achados.push(medir(saida, "com a CP em UTF-8"));

    unsafe { SetConsoleOutputCP(original) };

    let destino = std::env::args().nth(1).expect("o caminho do relatorio");
    std::fs::write(destino, achados.join("\n")).expect("relatorio gravado");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("so faz sentido no Windows");
}
