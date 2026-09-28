# WPC-65 · O `cargo doc` sai sem aviso, e um link quebrado novo não entra - verification

**Verdict**: PASS
**Profile**: light
**Diff range**: 106df91..df0524b
**Round**: 1 - full
**Verifier**: independent sub-agent (author != verifier)

Todas as nove provas rodaram em `df0524b` (rustc 1.98.1, cargo 1.98.1) e saíram 0, e cada check tem evidência localizada. As lacunas abaixo são de precisão dos checks, não de comportamento, e nenhuma derruba o veredito.

## Checks

| Check | Claim | Proof run | Evidence | Result |
| --- | --- | --- | --- | --- |
| C1 | `cargo rustdoc --lib -- -D warnings` e `cargo doc --no-deps` saem 0, sem nenhuma linha começando com `warning` ou `error` (AC 1) | As duas provas, como escritas: exit 0 e exit 0 (cargo 0 e 0). A primeira foi resolvida pelo cache do cargo (`Finished ... in 0.08s`, sem documentar). Por isso rodei a mesma asserção com `CARGO_TARGET_DIR` numa pasta nova do scratchpad: `Compiling arca`, `Documenting arca`, `Finished ... in 7.15s`, exit 0, nenhuma linha `warning` ou `error`. A segunda documentou de fato (`Documenting arca`, 1,36 s) | Não há `assert`: a asserção é a própria prova (código 0 e nenhuma linha `warning`/`error` na saída). As 13 linhas que a produzem conferem com os hunks do diff: `src/comandos/prepare.rs:90`, `:91`, `src/comandos/status.rs:405`, `:478`, `src/duplos.rs:223`, `src/estado.rs:27`, `src/firmware.rs:72`, `:77`, `src/pacote.rs:144`, `src/receita.rs:36`, `:37`, `:478`, `src/sondagem.rs:12` | PASS |
| C2 | Cada um dos 13 lugares nomeia em crase, sem link, o item para onde o link apontava; `chamada` vira `Leitura::chamada` (AC 2) | Os 13 `grep -F`/`grep -xF` encadeados: exit 0 | As 13 linhas acima, uma por `grep`. O contexto à esquerda de cada padrão exclui o `[`, então "sem link" se sustenta. Cada nome aponta para um item que existe: `src/comandos/prepare.rs:943` (`ler_o_firmware_antes`), `:809` (`criar_a_entrada`), `:567` (`conferir_o_pacote_local`), `src/comandos/status.rs:645`, `:535`, `src/duplos.rs:144` (campo `fwbootmgr`), `src/estado.rs:462` (`campo`), `src/firmware.rs:269` (`fn chamada`, dentro do `impl Leitura` de `src/firmware.rs:243`), `src/receita.rs:1063`, `:187`, `:215` | PASS |
| C3 | Em `src/`, 13 linhas saem e 13 entram, todas `///` ou `//!`; os cinco links que não geram aviso ficam onde estão (AC 3) | As duas provas: exit 0 e exit 0 | Li o `git diff 106df91..HEAD -- src/` inteiro: 8 arquivos, 13 `-` e 13 `+`, todos `///` ou `//!`. Os cinco links que ficaram: `src/comandos/prepare.rs:196` (comentário `//`), `src/comandos/prepare.rs:802` (doc do `criar_a_entrada` privado, `:809`), `src/comandos/prepare.rs:2934` (doc da fn de teste privada `entrada_com`, `:2935`), `src/receita.rs:182` (doc do `FLAGS_DE_SONDAGEM` privado, `:187`), `src/receita.rs:1015` (doc do `montar_sondagem` privado, `:1063`) | PASS |
| C4 | Um link público para item privado (o `[campo]` de `src/estado.rs:27` de volta) faz o hook sair 1 com `links to private item` e `o commit NAO foi feito`, numa worktree descartável (AC 4) | A prova como escrita, com um `cp` da saída do hook para o scratchpad depois do `ok=$?` e antes do `rm -rf`: exit 0 (hook exit 1), 15 s | Falha injetada em `src/estado.rs:27`. Saída capturada: `error: public documentation for estado links to private item campo` em `src\estado.rs:27:13`, `-D rustdoc::private-intra-doc-links implied by -D warnings`, depois `pre-commit: cargo rustdoc (os links da documentação) reprovou — o commit NAO foi feito.` O rótulo é o do passo novo, `.githooks/pre-commit:51-52`, e só aparece se o fmt (`:41-42`) e o clippy (`:44-45`) passaram antes dele | PASS |
| C5 | Um link que não resolve (o de `NaoEstaLa` para `conferir_o_pacote_local` de volta) faz o hook sair 1 com `unresolved link to` esse caminho e `o commit NAO foi feito` (AC 5) | A prova como escrita, com o mesmo `cp` depois do `ok=$?`: exit 0 (hook exit 1), 12 s | Falha injetada em `src/pacote.rs:144`. Saída capturada: `error: unresolved link to crate::comandos::prepare::conferir_o_pacote_local` em `src\pacote.rs:144:11` (`no item named conferir_o_pacote_local in module prepare`), `-D rustdoc::broken-intra-doc-links implied by -D warnings`, depois o mesmo rótulo do passo novo (`.githooks/pre-commit:51-52`) com `o commit NAO foi feito` | PASS |
| C6 | Sobre a árvore de `HEAD`, o hook sai 0 e a última linha é `pre-commit: ok` (AC 6) | A prova como escrita: exit 0 | Saída inteira do hook: `pre-commit: formatação, clippy, documentação e a suíte` e `pre-commit: ok`. O passo novo fica em `.githooks/pre-commit:51-52` (`cargo rustdoc --lib --quiet -- -D warnings`), entre o clippy (`:44-45`) e a suíte (`:56-57`); o `echo "pre-commit: ok"` fica em `.githooks/pre-commit:59` | PASS |
| C7 | `semanal.yml` tem um passo cujo `run` é `cargo rustdoc --lib -- -D warnings`, depois do clippy e antes de `Suíte` (AC 7) | A prova como escrita: exit 0 (c=40, d=43, s=53) | `.github/workflows/semanal.yml:40` (clippy), `.github/workflows/semanal.yml:42-43` (`Documentação, com aviso valendo erro` / `run: cargo rustdoc --lib -- -D warnings`), `.github/workflows/semanal.yml:53` (`- name: Suíte`). Prova estática: mostra presença e ordem, não execução no GitHub (ver Findings) | PASS |
| C8 | O README §15 e o `CLAUDE.md` citam `cargo rustdoc --lib -- -D warnings` e não contêm `os três` nem `os mesmos três`, nem quebrados entre linhas (AC 8) | A prova como escrita: exit 0 | `README.md:2152-2153` (o comando na mesma frase do fmt e do clippy), `README.md:2158` (o hook "roda esses comandos e a suíte"), `README.md:2169-2170` ("os mesmos passos"); `CLAUDE.md:20` (bloco de comandos), `CLAUDE.md:31` ("formatação, clippy, documentação e a suíte ... os mesmos passos"). Uma varredura extra, sem distinguir maiúsculas e com as linhas juntadas, em `README.md`, `CLAUDE.md`, `.githooks/`, `.github/`, `docs/` e `CONTEXT.md`, achou zero contagens de passos | PASS |
| C9 | O cabeçalho do hook (antes do `set -e`), o `echo` inicial dele e o cabeçalho do `semanal.yml` (antes do `name:`) citam a documentação (AC 9) | A prova como escrita: exit 0 | `.githooks/pre-commit:3` ("Formatação, clippy, documentação e a suíte"), `.githooks/pre-commit:16` (o `echo` inicial), `.github/workflows/semanal.yml:4-5` ("formatação, clippy, documentação e a suíte") | PASS |

## Swept existing

| Swept row | Cited | What the code says | Holds |
| --- | --- | --- | --- |
| dependency failure | `.githooks/pre-commit:18-21` | `if ! command -v cargo`, imprime "sem cargo no PATH — pulando" e faz `exit 0`, antes de qualquer `passo`. O passo novo (`:51-52`) vem depois e herda o pulo | sim |
| observability | `.githooks/pre-commit:29-39` | O `passo` manda a saída do comando para um arquivo. Quando reprova, imprime essa saída, o rótulo e `o commit NAO foi feito`, e sai 1. As saídas capturadas de C4 e C5 mostram isso para o passo novo | sim |

As linhas `n/a` do Swept são política aprovada, e não há nada no código para elas contradizerem.

## Steps not run (light)

- **Passo 1, binding sources** (`ui`): não rodou. O plano não marca fonte binding.
- **Recompute da Coverage** (`standard`, `ui`): não rodou. Li a tabela Coverage de `checks.md` e conferi os 13 números de linha contra os hunks do diff. Isso foi leitura, não recompute.
- **Test policy** (`standard`, `ui`): n/a, porque `checks.md` não tem essa seção.
- **Passo 4, injeção de falhas pelo Verifier** (`standard`, `ui`): não rodou. C4 e C5 injetam uma falha por desenho, mas são provas dos checks, não o passo 4. Por isso ninguém mostrou que as provas de C2, C3, C7, C8 e C9 falhariam diante de uma regressão.
- **Passo 5, fluxo com o usuário**: n/a, porque não há interface.
- **Passo 7, lições**: não rodou. O orquestrador restringiu a escrita a este arquivo, e o `lessons.py` escreve em outro. As lacunas de precisão abaixo são o insumo para ele.

## Deviations from the proofs as written

- **C4 e C5:** acrescentei um `cp "$T/saida"` para o scratchpad, depois do `ok=$?` e antes do `rm -rf "$T"`, para ler a evidência. Ele não altera o código de saída.
- **C1:** a primeira prova, como escrita, foi satisfeita pelo cache do cargo, em 0,08 s. Rodei a mesma asserção de novo, do zero, com `CARGO_TARGET_DIR` numa pasta do scratchpad, que apaguei em seguida: exit 0, 7,15 s.
- **C6:** o passo `cargo rustdoc` do hook, na árvore real, provavelmente também saiu do cache. A execução do zero descrita no C1 cobre o mesmo código-fonte.
- **Gate:** rodei `cargo test --quiet --no-fail-fast` uma vez para contar os resultados, porque o hook descarta a saída da suíte quando ela passa.

## Tree state

- **Antes das provas:** `git status --porcelain` vazio, e `git worktree list` só com `C:/Users/Eduardo/Repository/ArcaBackup df0524b [main]`.
- **Depois de C4 e C5, e de novo depois de todas as execuções:** idêntico.
- **O incidente do Handoff não se repetiu.** As duas saídas do `build.rs` na `target/` real (`target/debug/build/arca-*/output`) apontam `/MANIFESTINPUT` para `C:\Users\Eduardo\Repository\ArcaBackup\recursos/arca.manifest`. A hora delas é 19:57, antes de C4 começar (20:08:56). A pasta de compilação própria das provas funcionou como isolamento.

## Gate

`sh .githooks/pre-commit` (C6) - exit 0, `pre-commit: ok` (fmt, clippy, rustdoc e suíte); `cargo test --quiet --no-fail-fast` em `df0524b` - 949 passed, 0 failed, 0 ignored, em 15 binários de teste

## Findings

Nenhum bloqueia. Em ordem de peso:

1. **C1 (primeira prova) e o passo `rustdoc` do C6 podem passar pelo cache do cargo, sem documentar de novo.** Em `df0524b`, o `cargo rustdoc --lib -- -D warnings` respondeu em 0,08 s. O cache só é válido se a última execução com os mesmos argumentos passou, então o risco é pequeno, mas a prova não força a execução. A rodada do zero (7,15 s, exit 0) fecha a lacuna nesta verificação.
2. **A metade `CLAUDE.md` do C8 é um `grep` no arquivo inteiro.** Ele não distingue "citado entre o que o hook e o semanal rodam" de "aparece em qualquer lugar". Em HEAD, o comando está no bloco de comandos (`CLAUDE.md:20`), ao lado do fmt e do clippy, que é o que a linha Observable do plano pede. A frase sobre o hook (`CLAUDE.md:31`) nomeia "documentação", mas não o comando. Além disso, o padrão `os +(mesmos +)?três` distingue maiúsculas e deixaria passar um "Os três". A varredura sem distinguir maiúsculas não achou nenhum, então o impacto hoje é zero.
3. **C4 e C5 conferem só a linha genérica `o commit NAO foi feito`, e não o rótulo do passo `rustdoc`.** O diagnóstico específico do `rustdoc`, somado ao hook parar no primeiro passo que reprova, torna a leitura inequívoca. As saídas capturadas mostram o rótulo do passo novo.
4. **Provas mais fracas do que o texto dos checks:**
   - C2 usa `grep` de substring no arquivo inteiro, sem fixar a linha. O contexto de cada padrão ancora o parágrafo, e o C3 limita a mudança a 13 linhas de doc.
   - O padrão `//[/!]` do C3a também aceita `////`, que é comentário comum. Em HEAD, as 26 linhas do diff são `///` ou `//!`.
   - O C3b conta linhas que casam, não ocorrências nem lugares. O `grep` com número de linha confirmou os cinco lugares, um link por linha.
5. **C7 é estático.** A execução do passo no GitHub exige `push`, que não foi autorizado, e fica fora desta verificação. É um limite declarado: o claim do C7 se restringe a presença e ordem, e o comando é o mesmo que o C1 rodou aqui.
6. **Não verificado:** o "hoje os dois dão 13 (em `106df91`)" do claim do C1 é contexto histórico, e nenhuma prova o toca. Não rodei nada em `106df91`. C4 e C5 reintroduzem dois dos 13 links e mostram que são pegos.
