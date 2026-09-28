# WPC-65 · O `cargo doc` sai sem aviso, e um link quebrado novo não entra - checks

Profile: light (nenhum `AGENTS.md` declara outro)
Plan: `.specs/features/wpc-65/plan.md`

Base da feature: `106df91`. Os comandos rodam no Git Bash, a partir da raiz do repositório.

9 checks in 3 slices · 0 one-way doors · 0 open

## Checks

### S1 - O `cargo doc` da lib sai sem aviso · 8 files · 491 KB · ~123k

**C1** - WHEN `cargo rustdoc --lib -- -D warnings` roda THEN sai com código 0, e a saída não tem nenhuma linha que comece com `warning` ou `error`. O `cargo doc --no-deps`, que é o comando que o README ensina, também não tem essas linhas. Hoje os dois dão 13 (medido em 28/09/2026, em `106df91`) (AC 1)
Proof: `O=$(cargo rustdoc --lib -- -D warnings 2>&1); rc=$?; [ "$rc" -eq 0 ] && ! printf '%s\n' "$O" | grep -qE '^(warning|error)'`
Proof: `O=$(cargo doc --no-deps 2>&1); rc=$?; [ "$rc" -eq 0 ] && ! printf '%s\n' "$O" | grep -qE '^(warning|error)'`

**C2** - Cada um dos 13 lugares continua nomeando, em crase e sem link, o item que o link apontava, no mesmo trecho de texto. O `chamada` de `src/firmware.rs:72` passa a `Leitura::chamada` (AC 2)
Proof: `grep -qF 'morria na mesma leitura. Ver `ler_o_firmware_antes`,' src/comandos/prepare.rs && grep -qxF '//! `criar_a_entrada` e o ADR-0026.' src/comandos/prepare.rs && grep -qF '/// A versao de julgamento do `confere_com_o_arcaboot`, e ela responde' src/comandos/status.rs && grep -qxF '/// guarda em `secao_da_ordem_de_boot`.' src/comandos/status.rs && grep -qxF '    /// `FirmwareDeMentira::fwbootmgr`.' src/duplos.rs && grep -qF '//! assim `campo` confere antes de escrever' src/estado.rs && grep -qF 'sem ter medido por que: `Leitura::chamada`' src/firmware.rs && grep -qF 'pela mesma razao de `Leitura::chamada`: quem' src/firmware.rs && grep -qxF '    /// `crate::comandos::prepare::conferir_o_pacote_local`, que roda antes' src/pacote.rs && grep -qF '**Codigo novo** — ver `montar_sondagem` |' src/receita.rs && grep -qF 'terceira coluna para isso — ver `FLAGS_DE_SONDAGEM` |' src/receita.rs && grep -qF 'truncaria em silencio. Ver `TETO_DOS_PARAMETROS`.' src/receita.rs && grep -qF '//! (`crate::receita::montar_sondagem`); quem lê e este modulo' src/sondagem.rs`

**C3** - Em `src/`, a feature troca exatamente 13 linhas: 13 saem e 13 entram, e todas são comentário de documentação (`///` ou `//!`). Os outros cinco links para os mesmos itens (`src/comandos/prepare.rs:196`, `:802`, `:2934` e `src/receita.rs:182`, `:1015`) estão em comentário comum ou em documentação privada, não geram aviso e ficam como estão (AC 3)
Proof: `D=$(git diff 106df91..HEAD -- src/ | grep -E '^[+-]' | grep -vE '^(\+\+\+|---) '); [ "$(printf '%s\n' "$D" | grep -c '^-')" -eq 13 ] && [ "$(printf '%s\n' "$D" | grep -c '^+')" -eq 13 ] && ! printf '%s\n' "$D" | grep -vqE '^[+-][[:space:]]*//[/!]'`
Proof: `[ "$(grep -rcE '\[`(ler_o_firmware_antes|criar_a_entrada|montar_sondagem|FLAGS_DE_SONDAGEM)`\]' src/comandos/prepare.rs src/receita.rs | awk -F: '{s+=$2} END {print s}')" -eq 5 ]` - os cinco links que não geram aviso continuam lá

### S2 - Um link quebrado novo não entra · 2 files · 3 KB · ~1k

**C4** - IF a árvore de trabalho traz um link de documentação pública para item privado (o `[`campo`]` de `src/estado.rs:27` de volta) THEN `sh .githooks/pre-commit` sai com código 1, e a saída contém `links to private item `campo`` e `o commit NAO foi feito`. A falha roda numa worktree descartável, e a árvore real não é tocada (AC 4)
Proof: `T=$(mktemp -d); W="$T/wt"; git worktree add -q --detach "$W" HEAD && sed -i 's/assim `campo` confere/assim [`campo`] confere/' "$W/src/estado.rs" && grep -qF '[`campo`]' "$W/src/estado.rs" && (cd "$W" && CARGO_TARGET_DIR="$T/target" sh .githooks/pre-commit) >"$T/saida" 2>&1; rc=$?; git worktree remove --force "$W"; [ "$rc" -eq 1 ] && grep -qF 'links to private item `campo`' "$T/saida" && grep -qF 'o commit NAO foi feito' "$T/saida"; ok=$?; rm -rf "$T"; [ "$ok" -eq 0 ]`

**C5** - IF a árvore de trabalho traz um link de documentação que não resolve (o de `NaoEstaLa` para `conferir_o_pacote_local` de volta) THEN `sh .githooks/pre-commit` sai com código 1, e a saída contém `unresolved link to `crate::comandos::prepare::conferir_o_pacote_local`` e `o commit NAO foi feito` (AC 5)
Proof: `T=$(mktemp -d); W="$T/wt"; git worktree add -q --detach "$W" HEAD && sed -i 's/`crate::comandos::prepare::conferir_o_pacote_local`/[&]/' "$W/src/pacote.rs" && grep -qF '[`crate::comandos::prepare::conferir_o_pacote_local`]' "$W/src/pacote.rs" && (cd "$W" && CARGO_TARGET_DIR="$T/target" sh .githooks/pre-commit) >"$T/saida" 2>&1; rc=$?; git worktree remove --force "$W"; [ "$rc" -eq 1 ] && grep -qF 'unresolved link to `crate::comandos::prepare::conferir_o_pacote_local`' "$T/saida" && grep -qF 'o commit NAO foi feito' "$T/saida"; ok=$?; rm -rf "$T"; [ "$ok" -eq 0 ]`

**C6** - WHEN `sh .githooks/pre-commit` roda sobre a árvore de `HEAD`, sem aviso de documentação THEN sai com código 0, e a última linha é `pre-commit: ok` (AC 6)
Proof: `S=$(mktemp); sh .githooks/pre-commit >"$S" 2>&1 && tail -n1 "$S" | grep -qxF 'pre-commit: ok'`

**C7** - O `.github/workflows/semanal.yml` tem um passo cujo `run` é `cargo rustdoc --lib -- -D warnings`, depois do passo do clippy e antes do passo `Suíte` (AC 7)
Proof: `F=.github/workflows/semanal.yml; c=$(grep -nxF '        run: cargo clippy --all-targets -- -D warnings' "$F" | cut -d: -f1); d=$(grep -nxF '        run: cargo rustdoc --lib -- -D warnings' "$F" | cut -d: -f1); s=$(grep -nxF '      - name: Suíte' "$F" | cut -d: -f1); [ -n "$c" ] && [ -n "$d" ] && [ -n "$s" ] && [ "$c" -lt "$d" ] && [ "$d" -lt "$s" ]`

### S3 - A documentação diz o que os guarda-corpos cobram · 4 files · 134 KB · ~34k

**C8** - O README §15 ("Os dois guarda-corpos, e o que cada um alcança", até a tabela `| Arquivo | O que prova |`) e o `CLAUDE.md` citam `cargo rustdoc --lib -- -D warnings`, e nenhum dos dois contém `os três` nem `os mesmos três`, nem quebrados entre linhas (AC 8)
Proof: `S=$(sed -n '/^### Os dois guarda-corpos/,/^| Arquivo | O que prova |/p' README.md | tr '\n' ' '); C=$(tr '\n' ' ' < CLAUDE.md); printf '%s' "$S" | grep -qF 'cargo rustdoc --lib -- -D warnings' && printf '%s' "$C" | grep -qF 'cargo rustdoc --lib -- -D warnings' && ! printf '%s' "$S" | grep -qE 'os +(mesmos +)?três' && ! printf '%s' "$C" | grep -qE 'os +(mesmos +)?três'`

**C9** - O comentário de cabeçalho do `.githooks/pre-commit` (antes do `set -e`), o `echo` inicial dele e o comentário de cabeçalho do `semanal.yml` (antes do `name:`) citam a documentação (AC 9)
Proof: `sed -n '1,/^set -e$/p' .githooks/pre-commit | grep -qi 'documenta' && grep -E '^echo "pre-commit: ' .githooks/pre-commit | grep -qi 'documenta' && sed -n '1,/^name: /p' .github/workflows/semanal.yml | grep -qi 'documenta'`

## Coverage

| Set (size) | Member -> proof | Unproven |
| --- | --- | --- |
| os 13 links que geravam aviso (13) | `prepare.rs:90` C1 C2 · `prepare.rs:91` C1 C2 · `status.rs:405` C1 C2 · `status.rs:478` C1 C2 · `duplos.rs:223` C1 C2 · `estado.rs:27` C1 C2 · `firmware.rs:72` C1 C2 · `firmware.rs:77` C1 C2 · `pacote.rs:144` C1 C2 · `receita.rs:36` C1 C2 · `receita.rs:37` C1 C2 · `receita.rs:478` C1 C2 · `sondagem.rs:12` C1 C2 | - |
| as duas classes de aviso (2) | link para item privado C4 · link que não resolve C5 | - |
| os guarda-corpos que cobram a documentação (2) | hook C4 C5 C6 · `semanal.yml` C7 | - |
| os textos que contavam os passos (5) | README §15 C8 · `CLAUDE.md` C8 · cabeçalho do hook C9 · `echo` do hook C9 · cabeçalho do `semanal.yml` C9 | - |

- C1 prova os 13 juntos, porque um aviso só já o derrubaria. C2 prova cada um pelo texto que fica no lugar
- C7 é estática: prova que o passo existe e em que posição, e não que ele roda no GitHub. O comando dele é o mesmo de C1, que roda aqui. A execução no GitHub exige `push`, e fica fora da verificação

## Swept

- validation: n/a - a feature não valida entrada. O que ela cobra é o texto da documentação, e isso é C1
- failure modes: C4, C5
- idempotency: n/a - o hook e o `cargo rustdoc` só escrevem em `target/`, e rodar de novo dá o mesmo resultado
- authorization: n/a - um hook local e um job de CI que já existem. Nenhuma permissão muda
- concurrency: n/a - o hook roda uma vez por commit, e o semanal numa cópia própria no runner
- data lifecycle: n/a - nada é criado, guardado nem apagado fora de `target/`
- dependency failure: existing - sem `cargo` no PATH, o hook pula tudo com uma mensagem (`.githooks/pre-commit:18-21`), e o passo novo herda isso
- state transitions: n/a - não há estado
- observability: existing - o `passo` do hook imprime a saída do passo que reprovou e o rótulo dele (`.githooks/pre-commit:29-39`). C4 e C5 provam que a saída do `rustdoc` aparece

## Handoff

- S1 = ~123k: `wc -c` dos oito arquivos de `src/` (`prepare.rs` 137 KB, `receita.rs` 98 KB, `status.rs` 73 KB, `estado.rs` 57 KB, `duplos.rs` 54 KB, `firmware.rs` 44 KB, `pacote.rs` 22 KB, `sondagem.rs` 6 KB), 490.646 bytes, dividido por quatro. S2 entra no hook e no workflow com 3.278 bytes, ~1k, total ~124k. S3 entra no README e no `CLAUDE.md` com 131.083 bytes, ~33k, total ~156k: acima dos 150k do orçamento padrão
- A conta supõe que o construtor lê cada arquivo inteiro. Aqui cada arquivo de `src/` recebe uma ou duas linhas, e a leitura é do trecho em volta, não do arquivo
- Mechanism: one builder (compaction accepted) - escolha do usuário em 28/09/2026, antes da primeira linha de código: "Um construtor só (Recomendado)"

- **Boundary:** C1-C3 fechados em `e5b2e26`. C4-C9 fechados no commit que traz esta linha. A base da feature é `106df91`, e o plano e os checks entraram sozinhos em `f6eabf8`
- **Settled mid-build:** nada foi perguntado ao usuário depois da escolha do construtor. As mensagens de commit seguem a prosa em português do repositório, com os trailers, e não Conventional Commits nem a regra da skill contra trailers. É o que mandam o `CLAUDE.md` do projeto e a regra global de seguir a convenção do repositório, como na WPC-64. Por isso o `check_commit.py` não roda
- **Settled mid-build, as provas de C4 e C5:** as duas trocaram `CARGO_TARGET_DIR="$R/target"`, a pasta de compilação da árvore real, por `CARGO_TARGET_DIR="$T/target"`, uma pasta própria que é apagada no fim. As asserções não mudaram. O motivo foi medido: o pré-teste de C4 com a pasta compartilhada estragou a árvore real. O `build.rs` grava o caminho do manifesto com `env!("CARGO_MANIFEST_DIR")`, que é avaliado quando o próprio script compila. O cargo reaproveitou o script compilado na worktree, e o commit seguinte reprovou na suíte com `LNK1327`, procurando `recursos/arca.manifest` dentro da worktree já apagada. Um `touch build.rs` recompilou o script, e as duas saídas dele voltaram a apontar para a árvore real
- **Abandoned:** a pasta de compilação compartilhada nas provas de C4 e C5 (acima). Antes do commit da trava, C4 rodou numa worktree de `e5b2e26` com o hook da árvore de trabalho copiado para dentro: código 1, com `links to private item `campo`` e `o commit NAO foi feito`. No README, o parágrafo sobre o `cargo rustdoc` começou entre "Duas coisas cobram isso:" e a lista que essa frase abre, e foi para antes dela
