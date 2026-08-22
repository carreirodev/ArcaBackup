//! Saber se a janela de console e nossa.
//!
//! Quando o UAC eleva, o processo elevado nao herda o console de quem o
//! chamou: ganha uma janela nova. Se ele simplesmente sair, a janela some com
//! a saida dentro. Detectar isso aqui e o que faz `arca list` e `arca status`
//! terem serventia num Windows com UAC ligado.

use windows_sys::Win32::System::Console::GetConsoleProcessList;

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
