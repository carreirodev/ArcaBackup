//! Saber se a janela de console e nossa.
//!
//! Quando o UAC eleva, o processo elevado nao herda o console de quem o
//! chamou: ganha uma janela nova. Se ele simplesmente sair, a janela some com
//! a saida dentro. Detectar isso aqui e o que faz `arca list` e `arca status`
//! terem serventia num Windows com UAC ligado.

use windows_sys::Win32::System::Console::GetConsoleProcessList;

// # A pagina de codigo nao precisa ser tocada
//
// A saida do §5.4 do PRD nao e ASCII: o separador entre as colunas de uma
// imagem e um `·` (U+00B7). A suspeita natural e que num console em CP850 —
// o padrao de um Windows em portugues — ele sairia sujo, e que o ARCA
// precisaria trocar a pagina de codigo por UTF-8.
//
// Medido, e nao suposto: `examples/ponto_no_console.rs` imprime o `·` num
// console de verdade e lê de volta o que o console **desenhou**. Com a CP em
// 850 o caractere chega intacto, porque o `print!` do Rust escreve em console
// por `WriteConsoleW`, que recebe UTF-16 e nao passa pela pagina de codigo.
//
// Trocar a CP seria, entao, mexer num estado que pertence ao console de quem
// chamou — e que um Ctrl+C deixaria trocado para sempre — em troca de nada.

/// Verdadeiro quando este e o unico processo anexado ao console — o que
/// significa que a janela foi criada para ele e some quando ele sair.
pub fn janela_propria() -> bool {
    let mut processos = [0u32; 4];

    // SEGURANCA: o tamanho informado e o do proprio vetor da pilha.
    let quantos = unsafe { GetConsoleProcessList(processos.as_mut_ptr(), processos.len() as u32) };

    quantos == 1
}

/// Segura a janela ate o usuario ler o que ficou nela. So faz sentido quando
/// a janela e nossa: se o comando foi digitado num console que ja existia, a
/// saida fica la e nao ha o que segurar.
pub fn pausar_antes_de_fechar(pedida: bool) {
    use std::io::{BufRead, Write};

    if !pedida || !janela_propria() {
        return;
    }

    print!("\nPressione Enter para fechar. ");
    let _ = std::io::stdout().flush();
    let mut descartado = String::new();
    let _ = std::io::stdin().lock().read_line(&mut descartado);
}
