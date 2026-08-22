//! Imprime, uma por linha, cada palavra que o Windows entregou a este
//! processo. Existe para que os testes de C-7 possam observar o outro lado da
//! fronteira: o `arca.exe` de verdade exige elevacao, e um teste que
//! dispensasse o UAC nao poderia executa-lo.

fn main() {
    for argumento in std::env::args().skip(1) {
        println!("{argumento}");
    }
}
