//! Diagnostico: quanto da linha de comando do kernel o marco de 22/08 gastou?
//!
//! O §10.2.3 do PRD orca a linha contra o `COMMAND_LINE_SIZE` de 2048 e
//! reserva 512 para o `menuentry` base — numeros tirados das capturas feitas a
//! mao, quando nenhuma linha montada pelo ARCA tinha rodado. O marco em
//! hardware de 22/08/2026 rodou uma, e este exemplo a mede.
//!
//! # Por que ele reproduz em vez de ler
//!
//! O `grub.cfg` armado **nao existe mais**. O `arca resultado` desarma ao
//! colher (E8), e desarmar reescreve o arquivo: o original do marco foi
//! substituido pelo inerte as 21:14:50 de 22/08/2026, e o `desarme` nao guarda
//! copia. O que se pode fazer e reproduzir, e a reproducao e exata porque as
//! quatro entradas sao conhecidas: o `grub.cfg` inerte esta em
//! `recursos/capturas/` conferido por SHA256, e o nome, o disco e o selo estao
//! registrados no `estado.json` e no `arca.log`.
//!
//! **Reproducao nao e captura.** Ela prova o que o codigo de hoje gera, e nao
//! o que a maquina bootou — a diferenca que este projeto ja pagou caro para
//! nomear (ADR-0003, ADR-0004, P-16). O que atesta que a linha reproduzida e a
//! que rodou e outra coisa, e ela e um original: o
//! `recursos/capturas/ocs-sr-linha-de-comando-2026-08-22.txt`, que o proprio
//! Clonezilla escreveu com o comando que executou.
//!
//! Rode com `cargo run --example orcamento_da_linha_do_kernel`. Nao precisa de
//! elevacao nem do dispositivo: e codigo puro sobre uma captura.
//!
//! Com `--arquivo`, imprime o `grub.cfg` armado reproduzido inteiro, em vez da
//! medicao.

use arca::grub;
use arca::menuentry;
use arca::nome::Nome;
use arca::receita::{Disco, Operacao, Pedido, Receita, Selo};

const INERTE: &str = include_str!("../recursos/capturas/grub-inerte-arcaboot.cfg");

/// O teto do `COMMAND_LINE_SIZE` do kernel no x86_64, do §10.2.3.
const TETO: usize = 2048;
/// O que o §10.2.3 reserva para o `menuentry` base.
const RESERVADO: usize = 512;

/// O pedido do marco, como o `estado.json` e o `arca.log` o registram.
fn pedido_do_marco() -> Pedido {
    Pedido {
        operacao: Operacao::Backup,
        nome: Some(Nome::novo("2026-08-22_Apps").expect("nome valido")),
        disco: Some(Disco::novo("nvme0n1").expect("disco valido")),
        selo: Selo::novo("7d2d2f5153625b38").expect("o selo que o arca-fim.txt devolveu"),
    }
}

/// A linha `$linux_cmd` de um bloco `menuentry`, sem recuo e sem terminador.
///
/// **O `trim` das duas pontas e o que se mede, e nao so o do fim.** O recuo do
/// bloco e do `grub.cfg`, e nao da linha que o kernel recebe: contá-lo inflaria
/// o orcamento do §10.2.3 por uma coisa que o `grub` nao entrega ao kernel.
fn linha_do_kernel(bloco: &str) -> &str {
    bloco
        .lines()
        .find(|linha| linha.trim_start().starts_with("$linux_cmd"))
        .expect("o bloco tem linha de comando do kernel")
        .trim()
}

fn main() {
    let receita = Receita::montar(&pedido_do_marco()).expect("a receita passa por C-2");
    let bloco = menuentry::derivar(INERTE, receita.parametros()).expect("o inerte tem o modelo");
    let armado = grub::armar(INERTE, &bloco).expect("arma");

    if std::env::args().any(|argumento| argumento == "--arquivo") {
        print!("{armado}");
        return;
    }

    let modelo = grub::bloco_com_id(INERTE, menuentry::ID_DO_MODELO).expect("ha modelo");
    let base = linha_do_kernel(&modelo);
    let armada = linha_do_kernel(&bloco);

    println!("═══ A LINHA QUE RODOU EM 22/08/2026 ═══\n");
    println!("{armada}\n");

    println!("═══ O ORCAMENTO DO §10.2.3, CONTRA A MEDICAO ═══\n");
    println!("  {:<46} {:>6}", "Teto do kernel (COMMAND_LINE_SIZE)", TETO);
    println!("  {:<46} {:>6}", "Reservado para o menuentry base", RESERVADO);
    println!(
        "  {:<46} {:>6}",
        "Sobra orcada para o que o ARCA gera",
        TETO - RESERVADO
    );
    println!();
    println!(
        "  {:<46} {:>6}",
        "Linha do `live-toram` — o menuentry base",
        base.len()
    );
    println!(
        "  {:<46} {:>6}",
        "Linha armada, inteira",
        armada.len()
    );
    println!();

    // Os dois numeros que o codigo de fato confere e a receita de fato gera. A
    // diferenca entre eles e o `ocs_live_run="bash -c '…'"` em volta, e o
    // limite incide sobre os parametros — nunca sobre a linha pronta.
    println!(
        "  {:<46} {:>6}",
        "Os 5 parametros do ARCA (o que se confere)",
        receita.parametros_do_grub().len()
    );
    println!(
        "  {:<46} {:>6}",
        "Receita sozinha (dentro do ocs_live_run)",
        receita.comando().len()
    );
    println!();
    println!(
        "  {:<46} {:>6}",
        "Folga da linha pronta contra o teto",
        TETO.saturating_sub(armada.len())
    );
    println!(
        "  {:<46} {:>5.0}%",
        "Do teto do kernel, gasto",
        armada.len() as f64 * 100.0 / TETO as f64
    );

    println!("\n═══ O PIOR CASO QUE B-2 AINDA DEIXA PASSAR ═══\n");
    let nome_maximo = "A".repeat(48);
    let pior = Receita::montar(&Pedido {
        nome: Some(Nome::novo(&nome_maximo).expect("48 e o teto de B-2")),
        ..pedido_do_marco()
    })
    .expect("cabe, e e o que o §10.2.3 preve");
    let bloco_pior = menuentry::derivar(INERTE, pior.parametros()).expect("deriva");
    let linha_pior = linha_do_kernel(&bloco_pior);
    println!(
        "  {:<46} {:>6}",
        "Os 5 parametros, com o nome de 48",
        pior.parametros_do_grub().len()
    );
    println!(
        "  {:<46} {:>6}",
        "Receita sozinha, com o nome de 48",
        pior.comando().len()
    );
    println!(
        "  {:<46} {:>6}",
        "Linha armada, inteira",
        linha_pior.len()
    );
    println!(
        "  {:<46} {:>6}",
        "Folga da linha pronta contra o teto",
        TETO.saturating_sub(linha_pior.len())
    );
}
