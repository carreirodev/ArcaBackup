//! A receita que o ARCA gera, ao lado da que rodou em hardware.
//!
//! Os testes de `src/receita.rs` cobram trecho a trecho contra as capturas de
//! `recursos/capturas/`. Este exemplo existe para a outra pergunta, a que
//! nenhum `assert` responde: **como as duas se parecem quando alguem olha**.
//!
//! Rode com `cargo run --example receita_ao_lado_da_que_rodou`. Nao precisa
//! de elevacao nem do dispositivo conectado: o manifesto
//! `requireAdministrator` do `build.rs` vale so para o binario `arca`, e esta
//! montagem e codigo puro.

use arca::nome::Nome;
use arca::receita::{Disco, Operacao, Pedido, Receita, Selo};

const BACKUP_02: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-02.cfg");
const BACKUP_03: &str = include_str!("../recursos/capturas/grub-backup-arca-teste-03.cfg");
const RESTAURACAO_02: &str =
    include_str!("../recursos/capturas/grub-restauracao-arca-teste-02.cfg");

fn receita_da_captura(grub_cfg: &str) -> String {
    const ABERTURA: &str = "ocs_live_run=\"bash -c '";
    let linha = grub_cfg
        .lines()
        .find(|linha| linha.contains(ABERTURA))
        .expect("a captura tem a receita");
    let depois = &linha[linha.find(ABERTURA).unwrap() + ABERTURA.len()..];
    depois[..depois.find("'\"").unwrap()].to_string()
}

fn gerar(operacao: Operacao, nome: &str) -> Receita {
    Receita::montar(&Pedido {
        operacao,
        nome: Nome::novo(nome).expect("nome valido"),
        disco: Some(Disco::novo("nvme0n1").expect("disco valido")),
        selo: Selo::novo("a3f1c9e07b2d4856").expect("selo valido"),
    })
    .expect("a receita passa por C-2")
}

fn main() {
    println!("═══ BACKUP ═══\n");
    println!("o que rodou (ARCA-TESTE-02, R:\\boot\\grub\\grub.cfg.backup02):\n");
    println!("  {}\n", receita_da_captura(BACKUP_02));
    println!("o que rodou (ARCA-TESTE-03, E:\\ARCA-LOGS\\grub.cfg.original):\n");
    println!("  {}\n", receita_da_captura(BACKUP_03));
    println!("o que o ARCA gera:\n");
    println!("  {}\n", gerar(Operacao::Backup, "ARCA-TESTE-02").comando());

    println!("═══ RESTAURACAO ═══\n");
    println!("o que rodou (ARCA-TESTE-02, R:\\boot\\grub\\grub.cfg.teste02):\n");
    println!("  {}\n", receita_da_captura(RESTAURACAO_02));
    println!("o que o ARCA gera:\n");
    println!(
        "  {}\n",
        gerar(Operacao::Restauracao, "ARCA-TESTE-02").comando()
    );

    println!("═══ A LINHA INTEIRA, COMO ENTRA NO grub.cfg ═══\n");
    println!(
        "  {}\n",
        gerar(Operacao::Backup, "ARCA-TESTE-02").parametros_do_grub()
    );
}
