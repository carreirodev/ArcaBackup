# ARCABOOT com nome - report

**Verdict**: PASS
**Profile**: light
**Diff range**: f2563d7..c7e858f
**Round**: 2 - scoped
**Verifier**: independent sub-agent (author != verifier)

All 23 checks are proven at `c7e858f8542b3d315013ccffedb0e9862d3c2986`, and each has a located
assertion or a located document line.

- **Rust proofs.** The 14 Rust proofs (C1-C14) were re-run in one `cargo test --lib -- --exact` invocation at `c7e858f`, and each shows individually as `ok`.
- **Documentation proofs.** The nine pipelines (C15-C23) were re-run as written, and each exits 0 at `c7e858f`.
- **Gate.** The full gate was re-run at `c7e858f` and is green: 937 passed, 0 failed. `fmt` and `clippy -D warnings` are clean.

Of round 1's four findings, finding 2 is partly addressed and findings 1, 3 and 4 stay open. None of
them blocks.

**Scope of this round.** It follows verify.md's "Re-verifying after a fix". The fix is one commit,
`c7e858f`, on top of `9112f30`.

- `git diff --stat 9112f30..c7e858f`: `README.md | 2 +-`, a single line replaced, so no README line number moves.
- `git diff --stat 9112f30..c7e858f -- . ':!README.md'` is empty. No `src/`, `tests/`, PRD, CONTEXT, ADR, `Cargo.toml` or `build.rs` line changed.
- The replaced line is the caption at `README.md:392`, under the named-refusal example in §5 "Regra única de operação".
  - Before: "*(o texto que o teste `a_recusa_por_dois_arcavault_nomeia_os_volumes_de_boot_quando_ha_nome` exige, quebrado em linhas; ainda sem captura em hardware)*".
  - After: "*(o texto que `src/erro.rs` monta hoje, quebrado em linhas; ainda sem captura em hardware)*".

The round was scoped as follows:

- **Re-run in full at `c7e858f`:** every proof, and the gate, because green is a property of a commit.
- **Refreshed:** the `README.md` citations (C17-C20, C23, and the README lines in C15, C16 and the findings), the only file the fix touched. All README anchors and cited lines were re-read at `c7e858f` and sit at the same line numbers as in round 1.
- **Carried from `9112f30`:** everything else, marked per section. That covers the `src/`, PRD and CONTEXT citations, the assertion notes, level and sampling, the Swept re-read, the base-vs-HEAD vacuity runs and the `6840274` oracle run.

**Tree state (verified at c7e858f).** `git diff HEAD --stat -- src tests Cargo.toml build.rs README.md PRD CONTEXT.md docs/adr`
was empty before any run, so every run in this round is a run of HEAD. `git status --porcelain`
shows the same set as round 1: `CLAUDE.md` and `docs/agents/*` (unrelated, untouched), plus the
untracked `.agents/`, `.claude/` and `.specs/`.

## Binding sources

*(carried from 9112f30)* Step 1 runs under `ui` only, and this feature is `light`, so it did not
run. The plan marks no binding source; the ADR-0027 it cites was read as context.

## Checks

*(proofs verified at c7e858f; citations in `src/`, PRD and CONTEXT carried from 9112f30, since those
files are unchanged by the fix; `README.md` citations refreshed at c7e858f)*

Batched run at `c7e858f`: `cargo test --lib -- --exact` with the 14 check tests plus the one
pre-existing test the feature edited (`dispositivo::testes::dois_arcavault_sao_recusa_dura`). Result:
`15 passed; 0 failed; 809 filtered out`, exit 0. Each named test printed its own `... ok` line.
"Base" below means the `f2563d7` versions of the files, run in round 1 (carried from 9112f30).

| Check | Claim | Proof run | Evidence | Result |
| --- | --- | --- | --- | --- |
| C1 | `Windows` C:, `ARCAVAULT` E:, `ARCA-CASA` R: -> `Ok`, boot = R: `ARCA-CASA`, grub and estado paths on R: | batched run at c7e858f; `dispositivo::testes::o_arcaboot_renomeado_e_o_boot_do_dispositivo ... ok` | carried from 9112f30. Setup `src/dispositivo.rs:381-383`; `Ok` via `.expect(...)` at `:386`. `src/dispositivo.rs:387` `assert_eq!(dispositivo.boot, Some(volume("ARCA-CASA", 'R', 1000, 500)))`. `:388-391` `assert_eq!(dispositivo.caminho_do_grub().unwrap(), PathBuf::from(r"R:\boot\grub\grub.cfg"))`. `:392-395` `assert_eq!(dispositivo.caminho_do_estado().unwrap(), PathBuf::from(r"R:\arca\estado.json"))` | PASS |
| C2 | `ARCABOOT`, `arcaboot`, `ARCA-CASA`, `arca-Casa`, `ARCA-` -> boot on R:, label byte-identical | batched run at c7e858f; `...o_rotulo_de_boot_casa_arcaboot_ou_arca_hifen_em_qualquer_caixa ... ok` | carried from 9112f30. `src/dispositivo.rs:403` loops over all 5 labels. `:410-414` asserts the boot's letter equals `Some('R')`. `:415` asserts the boot's label `.as_deref()` equals `Some(rotulo)`. Both are reproduced verbatim under "Assertion notes", because their closure pipes would break this table | PASS |
| C3 | `ARCACASA`, `ARC`, `ÉRCA-X`, `ARCAÉ` -> `Ok` with `boot == None`, no panic | batched run at c7e858f; `...o_que_nao_tem_a_forma_fica_fora_do_arcaboot_sem_panico ... ok` | carried from 9112f30. `src/dispositivo.rs:427` loops over all 4 labels. `:433` `assert!(encontrar(&discos).unwrap().boot.is_none(), "{rotulo}")`. Byte dump (node) of the source literals: `ARCAÉ` = `41 52 43 41 c3 89`, with U+00C9 precomposed, so byte 5 is a continuation byte. `ARC` = 3 bytes. `&rotulo[..5]` panics on both; production uses `.get(..PREFIXO_DO_NOME.len())` at `:184` | PASS |
| C4 | only `ARCAVAULT` in E: -> `boot == None` (kills the hyphen-less `ARCA` prefix) | batched run at c7e858f; `...sem_arcaboot_o_dispositivo_ainda_serve_para_listar ... ok` | carried from 9112f30. Setup `src/dispositivo.rs:565`. `:568` `assert!(dispositivo.boot.is_none())`. The test predates the feature and is untouched, but it bears on new code: the boot filter at `:125` now calls `e_rotulo_de_boot` (`:181-186`), and a hyphen-less prefix would make `ARCAVAULT` a boot and fail `:568`. See "Assertion notes" | PASS |
| C5 | pairs `ARCABOOT`+`ARCA-CASA`, `ARCA-CASA`+`ARCA-ESCRIT`, `ARCA-CASA`+`ARCA-CASA` -> `Err(DispositivosDemais)`, `quantos == 2` | batched run at c7e858f; `...dois_volumes_de_boot_sao_recusa_dura_com_rotulos_iguais_ou_diferentes ... ok` | carried from 9112f30. `src/dispositivo.rs:441-445` loops over all 3 pairs. `:453-454` `Erro::DispositivosDemais { quantos, .. } => { assert_eq!(quantos, 2, "{primeiro} + {segundo}") }`. Any other variant panics at `:456`. Exit code unchanged: the variant falls to `_ => 1` at `src/erro.rs:493` | PASS |
| C6 | `prepare` disk with `ARCAVAULT` + `ARCA-CASA` -> plan has `ESTE DISCO JA E UM DISPOSITIVO ARCA` and `AS IMAGENS` | batched run at c7e858f; `comandos::prepare::testes::preparar_por_cima_de_um_dispositivo_nomeado_avisa_das_imagens ... ok` | carried from 9112f30. Precondition: `src/comandos/prepare.rs:1761` sets `discos[2].particoes[1].rotulo = Some("ARCA-CASA")`, over the fixture's `ARCAVAULT` (`src/duplos.rs:1190`) and `ARCABOOT` (`:1197`). `src/comandos/prepare.rs:1767-1770` `assert!(saida.contains("ESTE DISCO JA E UM DISPOSITIVO ARCA"))`. `:1771` `assert!(saida.contains("AS IMAGENS"))` | PASS |
| C7 | all 4 recipes: `dev:///LABEL=ARCAVAULT`, `LABEL=` once, no `ARCABOOT` in any case | batched run at c7e858f; `receita::testes::nenhuma_receita_cita_o_rotulo_do_arcaboot ... ok` | carried from 9112f30. `src/receita.rs:1341` `for receita in [backup(), restauracao(), verificacao(), sondagem()]`. `:1343-1346` `assert!(parametros.contains("dev:///LABEL=ARCAVAULT"))`. `:1347` `assert_eq!(parametros.matches("LABEL=").count(), 1)`. `:1348-1351` `assert!(!parametros.to_ascii_uppercase().contains("ARCABOOT"))` | PASS |
| C8 | `arca status` shows `linha("ARCA-CASA", "R: · FAT32 · 1,6 GB")`, not the `ARCABOOT` line; `arca-Casa` as Windows returns it | batched run at c7e858f; `comandos::status::testes::o_status_mostra_o_nome_do_arcaboot ... ok` | carried from 9112f30. `src/comandos/status.rs:1719` loops over `["ARCA-CASA", "arca-Casa"]` (FAT32, R:, 1_700_000_000). `:1729-1732` `assert!(saida.contains(&linha(nome, "R: · FAT32 · 1,6 GB")))`. `:1733-1736` `assert!(!saida.contains(&linha(dispositivo::ARCABOOT, "R: · FAT32 · 1,6 GB")))`. `saida` comes from `montar_com`, which calls production `montar` (`:238`) | PASS |
| C9 | named boot refusal has `R: ARCA-CASA, S: ARCA-ESCRIT` and 4 more required substrings, none of the 3 forbidden ones; `R: ARCABOOT, S: ARCA-CASA` for the mixed pair | batched run at c7e858f; `...a_recusa_por_dois_volumes_de_boot_nomeados_diz_a_letra_e_o_rotulo_de_cada_um ... ok` | carried from 9112f30. `src/dispositivo.rs:475-480` lists all 5 required substrings from the claim, each checked by `:482` `assert!(mensagem.contains(deve))`. `:484-487` lists all 3 forbidden ones, each checked by `:489-492` `assert!(!mensagem.contains(nao_deve))`. `:501` `assert!(mensagem.contains("R: ARCABOOT, S: ARCA-CASA"))` | PASS |
| C10 | two `ARCAVAULT` (E:, F:) + `ARCA-CASA` R: + `ARCABOOT` S: -> `rotulo == ARCAVAULT`, `onde == "E:, F:"`, message has `(E:, F:)` and `R: ARCA-CASA, S: ARCABOOT` | batched run at c7e858f; `...a_recusa_por_dois_arcavault_nomeia_os_volumes_de_boot_quando_ha_nome ... ok` | carried from 9112f30. `src/dispositivo.rs:519-521` `Erro::DispositivosDemais { rotulo, onde, .. } => { assert_eq!(*rotulo, ARCAVAULT); assert_eq!(onde, "E:, F:"); }`. `:526` `assert!(mensagem.contains("(E:, F:)"))`. `:527` `assert!(mensagem.contains("R: ARCA-CASA, S: ARCABOOT"))` | PASS |
| C11 | menu line for a disk with `ARCAVAULT` E: + `ARCA-CASA` F: ends in `· JA E UM DISPOSITIVO ARCA (ARCA-CASA)` | batched run at c7e858f; `comandos::prepare::testes::o_menu_diz_o_nome_do_dispositivo_arca ... ok` | carried from 9112f30. Precondition: `src/comandos/prepare.rs:2145-2162` (E: `ARCAVAULT`, F: `ARCA-CASA` at `:2158`). `:2169-2172` `assert!(linha_do_disco.ends_with("· JA E UM DISPOSITIVO ARCA (ARCA-CASA)"))` | PASS |
| C12 | `ARCABOOT` and `arcaboot` -> `Dispositivo ARCA` block byte-identical to the captured literal | batched run at c7e858f; `comandos::status::testes::sem_nome_o_bloco_do_dispositivo_sai_como_antes ... ok` | carried from 9112f30. Literal: `src/comandos/status.rs:1745-1747` `const ANTES: &str = "Dispositivo ARCA\n  ARCAVAULT ....... E: · NTFS · 236,6 GB\n  ARCABOOT ........ R: · FAT32 · 1,6 GB\n"`. `:1749` `assert_eq!(secao_do_dispositivo(&dispositivo_conectado()), ANTES)`, whose fixture boot is `dispositivo::ARCABOOT` at `:823`. `:1758` `assert_eq!(secao_do_dispositivo(&em_minuscula), ANTES)` with `"arcaboot"` at `:1754`. Green at `6840274` against pre-change code | PASS |
| C13 | no `ARCA-<texto>` -> both C-10 refusals byte-identical to captured literals | batched run at c7e858f; `dispositivo::testes::sem_volume_nomeado_as_recusas_de_c10_saem_como_antes ... ok` | carried from 9112f30. `src/dispositivo.rs:541-547` `assert_eq!(encontrar(&dois_vaults).unwrap_err().to_string(), "ha 2 volumes com o rotulo ARCAVAULT conectados (E:, F:), ... o novo e o de antes")`. `:554-560` does the same for `"ha 2 volumes com o rotulo ARCABOOT conectados (R:, S:), ..."`. Both are string literals, and both are green at `6840274` | PASS |
| C14 | `ARCAVAULT` + `ARCABOOT` -> whole `montar_o_menu` output byte-identical to captured literal, no parenthesis | batched run at c7e858f; `comandos::prepare::testes::o_menu_do_dispositivo_sem_nome_sai_como_antes ... ok` | carried from 9112f30. Literal: `src/comandos/prepare.rs:2181-2190` `const ANTES`, with the disk-2 line ending `· JA E UM DISPOSITIVO ARCA\n`. `:2192-2198` `assert_eq!(montar_o_menu(&preparacao::Oferta::de(&discos_para_preparar_desta_mesa(), Some('C'))), ANTES)`. Green at `6840274` | PASS |
| C15 | PRD table row for `C-16` and README §13 row for `**C-16**`, both with `ARCA-<texto>` | checks.md pipeline as written: exit 0 at c7e858f; exit 1 at base | PRD carried from 9112f30: `PRD/PRD-ARCA-v5_1.md:1530`, row `C-16`: "**O `ARCABOOT` pode levar um nome, `ARCA-<texto>`; o `ARCAVAULT`, não.** ...". README verified at c7e858f: `README.md:1968`, row `**C-16**`: "O `ARCABOOT` **pode levar um nome**, `ARCA-<texto>`, com até 6 caracteres depois do hífen. O `ARCAVAULT`, não" | PASS |
| C16 | PRD C-10 row and README §13 C-10 row contain `ARCA-<texto>` | pipeline: exit 0 at c7e858f; exit 1 at base | PRD carried from 9112f30: `PRD/PRD-ARCA-v5_1.md:1524`, row `C-10`: "Dois volumes de boot também, com rótulos iguais ou diferentes (`ARCABOOT` ou `ARCA-<texto>`, C-16)". README verified at c7e858f: `README.md:1962`, row `**C-10**`: "Recusar mais de um dispositivo conectado — dois `ARCAVAULT`, ou dois volumes de boot (`ARCABOOT` ou `ARCA-<texto>`, iguais ou não) — ..." | PASS |
| C17 | README no longer has the three "rótulo repetido" phrasings | pipeline: exit 0 at c7e858f; exit 1 at base | verified at c7e858f. At base they sat at `README.md:789` (§6.3), `:1157` (§6.7) and `:1923` (§13). At c7e858f, `grep -nF` finds zero hits for all three. Replaced by `README.md:812`, `:1180` and `:1962` (finding 3) | PASS |
| C18 | §5 has no `rotuladas sempre com os mesmos nomes`, and has `Explorer`, `ARCA-<texto>`, `6 caracteres` | pipeline: exit 0 at c7e858f; exit 1 at base | verified at c7e858f. The §5 range is `README.md:326-415`. The phrase was at base `README.md:328`, which now reads "com duas partições rotuladas". `README.md:396` contains all three: "renomeie o `ARCABOOT` de cada um no Explorer ... para `ARCA-<texto>` ... o texto tem até 6 caracteres" | PASS |
| C19 | §6.8 contains `ARCA-CASA` | pipeline: exit 0 at c7e858f; exit 1 at base | verified at c7e858f. The §6.8 range is `README.md:1252-1326`. `README.md:1302` `  ARCA-CASA ....................... R: · FAT32 · 1,6 GB`, inside the `Dispositivo ARCA` example block at `:1299-1303` | PASS |
| C20 | the refusal shows the name in "Regra única de operação" and in §12 | pipeline: exit 0 at c7e858f; exit 1 at base | verified at c7e858f. `README.md:387` "repetido nao ha o que escolher. Os volumes de boot conectados sao R: ARCA-CASA," is inside the refusal block under "Regra única" (`:369-393`). §12 (`README.md:1795-1944`) carries it at `:1811` ("como `R: ARCA-CASA, S: ARCABOOT`") and `:1813` (`### \`ha 2 volumes de boot ARCA conectados (R: ARCA-CASA, S: ARCA-ESCRIT)\``). Precision gap in the first half (finding 1, open) | PASS |
| C21 | CONTEXT `**ARCABOOT**` and `**Dispositivo**` entries contain `ARCA-<texto>` | pipeline: exit 0 at c7e858f; exit 1 at base | carried from 9112f30 (CONTEXT.md unchanged by the fix). Ranges `CONTEXT.md:9-11` and `:17-19` (each ends at its own `_Evitar_`). `CONTEXT.md:10`: "o `ARCABOOT` de cada um pode levar um nome, `ARCA-<texto>` (C-16)". `CONTEXT.md:18`: "o usuário pode renomeá-lo no Explorer para `ARCA-<texto>`, com até 6 caracteres depois do hífen (C-16)" | PASS |
| C22 | code docs drop "same labels" and carry `ARCA-<texto>` | pipeline: exit 0 at c7e858f; exit 1 at base | carried from 9112f30 (src unchanged by the fix). `src/dispositivo.rs:6` "O `ARCABOOT` pode levar um nome, `ARCA-<texto>`, desde 27/09/2026". `src/portas/discos.rs:27` "`ARCA-<texto>` (C-16)." Both removed sentences appear as `-` lines in the diff (`src/dispositivo.rs` module doc, `src/erro.rs` variant doc), with zero hits at HEAD | PASS |
| C23 | ADR-0027 versioned; README ADR table has a row starting `0027` | pipeline: exit 0 at c7e858f (`git ls-files` printed the path); the table half exits 1 at base | ADR carried from 9112f30: `docs/adr/0027-o-arcaboot-pode-levar-um-nome.md` is tracked and added in the range. README verified at c7e858f: `README.md:2227`, row `0027`: "O `ARCABOOT` **pode levar um nome**, e o `ARCAVAULT` não" | PASS |

### Assertion notes

*(carried from 9112f30. The fix touched no `src/` or `tests/` file, so no assertion, test body or
production line moved.)*

- **C2's two assertions, verbatim.** `src/dispositivo.rs:410-414` is `assert_eq!(boot.as_ref().and_then(|boot| boot.letra), Some('R'), "{rotulo}")`, and `:415` is `assert_eq!(boot.and_then(|boot| boot.rotulo).as_deref(), Some(rotulo))`. Both run for each of the 5 labels.
- **C3 carries the label that makes a byte slice panic.** `ARCAÉ` is precomposed U+00C9 in the source. `ÉRCA-X` alone would not panic, since its byte 5 is a boundary, and checks.md notes as much.
- **C9 and C10 assert both sides of the claim.** C9's two lists at `src/dispositivo.rs:475-487` match the claim's 5 required and 3 forbidden substrings one for one. C10 asserts the variant fields and both message substrings.
- **C4 resolves to a test the feature never touched, and that is sound here.** Its assertion runs through the new code path (`src/dispositivo.rs:125` -> `e_rotulo_de_boot`). C3 kills the same hyphen-less-prefix mutant a second way: `ARCAVAULT` plus `ARCACASA` would become two boots, and the `unwrap()` at `:433` would fail.
- **C12, C13 and C14 compare against string literals.** None of them uses a string rebuilt by the code under test. `6840274` is 80 insertions, all inside `mod testes` (`git show --stat`; every hunk header is `@@ ... mod testes`). `git log -L` over each of the three test bodies returns only `6840274`, so the literals were not edited afterwards. The three tests were run at `6840274`, in a `git archive` extract with its own target dir, against the unchanged production code: `3 passed; 0 failed`. The literals are the pre-change output.
- **No pre-existing test assertion was weakened or deleted.** `git diff --stat f2563d7..HEAD -- tests` is empty. Every `-` line in `git diff -U0 f2563d7..HEAD -- src` is production code or a doc comment. Every hunk inside a `mod testes` is a pure insertion (`-N,0`). The one insertion inside an existing test is `@@ -249,0 +325 @@`, which adds `..` to the match pattern of `dois_arcavault_sao_recusa_dura`. Its assertions at `src/dispositivo.rs:327-328` and after are unchanged, and it ran `ok` in the batched run in both rounds. The author's statement is confirmed.

### Level and sampling

*(carried from 9112f30)*

- **Level.** Every Rust proof uses the house pattern: port doubles (`DiscosDeMentira`, `discos_para_preparar_desta_mesa`) feeding the real render and decision functions.
  - Production reaches every function the tests call. `dispositivo::encontrar` is what every command calls. `status::montar_com` wraps production `montar` (`src/comandos/status.rs:840-847` -> `:238`), which `executar` calls at `:129` and which calls `secao_do_dispositivo` at `:241`. `prepare::executar` prints `montar_o_plano` at `src/comandos/prepare.rs:157` and `montar_o_menu` at `:335`.
  - The claims naming an error variant (C5, C10) match the variant on `encontrar`'s return.
  - There is no level gap against what the claims name. One limit sits outside the claims (finding 4).
- **Sampling.** No claim is proven on fewer members than it names:
  - C2: 5 labels at `:403`
  - C3: 4 labels at `:427`
  - C5: 3 pairs at `:441`
  - C7: 4 recipes at `receita.rs:1341`
  - C8: 2 labels at `status.rs:1719`
  - C12: 2 cases at `:1749` and `:1758`
  - C13: 2 literals at `:541` and `:554`
- **Missed branch points.** A search of `src` for remaining exact comparisons against the boot label (`eq_ignore_ascii_case(ARCABOOT)`, `== ARCABOOT`, `com_rotulo(.., ARCABOOT)`) finds only `src/dispositivo.rs:182` and `:195`, both inside the new matcher. The four places the plan's Impact names are the only ones that branch.

### Documentation proofs against L-001

*(anchors and cited lines verified at c7e858f; the L-001 judgment carried from 9112f30, since no
pipeline and no range anchor changed)*

Each pipeline greps the wording its claim states:

- C15, C16 and C21 grep the form `ARCA-<texto>` inside the row or entry, not just the rule id.
- C17 greps the exact removed phrasings.
- C18, C19 and C22 grep content words.
- C23's claim is structural (file tracked, table row present), so an id grep is the claim.
- C20 greps the right token, but over a range that no longer matches its claim (finding 1, open).

Every `sed`/`awk` anchor was confirmed unique and in order, and the README anchors were re-read at `c7e858f`:

- `README.md`: `## 5.` 326, `### Regra única` 369, `### Dar nome` 394, `### Os dois estados` 405, `## 6.` 416, `### 6.8` 1252, `### 6.9` 1327, `## 12.` 1795, `## 13.` 1945.
- `CONTEXT.md` (carried): `**Dispositivo**:` 9 -> `_Evitar_` 11, and `**ARCABOOT**:` 17 -> `_Evitar_` 19.

### The fix's new caption, checked against the code (verified at c7e858f)

The caption at `README.md:392` now claims the block above it is "o texto que `src/erro.rs` monta
hoje, quebrado em linhas". I checked this mechanically with node:

- Input: README lines `385-389` (between the fences at `:384` and `:390`), joined with single spaces.
- The block starts with `erro: `, the prefix that `src/main.rs:104` prints (`eprintln!("erro: {falha}")`). With that prefix removed, it is byte-identical to the `Some(boots) =>` branch format string at `src/erro.rs:523`, filled in with `quantos = 2`, `rotulo = ARCAVAULT`, `onde = "E:, F:"`, `boots = "R: ARCA-CASA, S: ARCABOOT"` and `O_QUE_FAZER`.

The new caption is true today. This check is a Verifier-run comparison, not a test in the suite. Nothing pins that text, so it can still drift (finding 2, open part).

### Swept rows re-read (`existing`)

*(carried from 9112f30. `git diff --stat 9112f30..c7e858f -- src build.rs` is empty, so none of the
cited files moved.)*

- **authorization: holds.** `build.rs:65` still emits `/MANIFESTUAC:level='requireAdministrator'`. `src/main.rs` still elevates (`elevacao` at `:19`, `privilegios()` at `:53`, analyse-before-elevate at `:65-66`). `git diff --stat f2563d7..HEAD -- src/main.rs build.rs src/adaptadores` is empty.
- **dependency failure: holds.** `src/adaptadores/windows/volumes.rs:32` `Ok(letras_montadas().filter_map(ler_volume).collect())`. `ler_volume` returns `None` when `GetVolumeInformationW` fails (`:89-90`) and when `GetDiskFreeSpaceExW` fails (`:105-106`). The file is untouched in the range.
- The rows that say `n/a` (idempotency, concurrency, data lifecycle, state transitions) are approved policy, and there is nothing in the code for them to be wrong about.

## Coverage

*(carried from 9112f30)* This step did not run: the `Coverage` recompute belongs to `standard` and `ui`,
and the profile is `light`. checks.md's Coverage table was not re-derived. The per-check sampling
above covers what `light` owes.

## Faults injected

*(carried from 9112f30)* This step did not run: fault injection belongs to `standard` and `ui`, and the
profile is `light`. The fix added no code surface. checks.md records that the author swapped
`get(..5)` for `&rotulo[..5]` before `05bd7f0` and C3 turned red. That is the author's report, not a
mutant this Verifier ran.

## Findings, ranked (all non-blocking)

*(status verified at c7e858f; README citations refreshed, all at unchanged line numbers)*

1. **C20 - precision gap in the proof. Open, carried from 9112f30; the fix did not act on it.** The first `sed` range, `/^### Regra única de operação/,/^### Os dois estados/`, spans `README.md:369-405`. The subsection `### Dar nome a um dispositivo` at `README.md:394` sits inside that span, and its paragraph at `:396` contains `ARCA-CASA` as a naming example.
   - Consequence: the first half of C20 would stay green even if the refusal example under "Regra única" dropped the name.
   - The claim itself holds on reading, at `README.md:387`.
   - Fix, per L-001: end the range at `/^### Dar nome/`, or grep the refusal wording itself, e.g. `Os volumes de boot conectados sao R: ARCA-CASA`.
2. **C9/C10 - README reproduces named-refusal text that no literal pins. Partly addressed at c7e858f.**
   - **Resolved:** the §5 caption at `README.md:392` no longer claims the test requires the whole text. It now attributes the text to `src/erro.rs`, and that attribution was confirmed byte-for-byte above.
   - **Still open:** no test pins the full wording of either named branch (`src/erro.rs:519-524`). The tests pin substrings only (`src/dispositivo.rs:475-487`, `:526-527`), so the §5 block (`README.md:385-389`) and the §12 heading at `README.md:1813` can still go stale silently when the message changes.
   - No test was added in the fix. AC 9 and AC 10 ask only for those substrings, so no check fails.
3. **C17 - absence-only proof. Open, carried from 9112f30; the fix did not act on it.** C17 proves the three old phrasings are gone. The replacement wording is asserted by no check at `README.md:812` (§6.3 pre-flight: "o `ARCAVAULT` e o volume de boot estão em discos físicos diferentes ... (Mais de um dispositivo inteiro já foi recusado no passo 2.)") or at `:1180` (§6.7 defence 2). §13's replacement is covered by C16 at `:1962`.
   - Read by hand, both are consistent with the code: step 2 at `README.md:805` is "Acha o dispositivo", i.e. `encontrar`, where C-10's count lives.
4. **Level - no renamed label through a real adapter. Open, carried from 9112f30; the fix did not act on it.** Every proof feeds `ARCA-<texto>` through a port double.
   - The claim that the Windows adapters pass labels through unfiltered rests on reading: `src/adaptadores/windows/volumes.rs:32`, and `src/adaptadores/windows/particionador.rs:79`, whose `FileSystemLabel` comes with no filter. No adapter code compares a label.
   - The desk device is labelled `ARCABOOT`, and renaming it would be an outward action, so the hardware tests cannot reach this path.
   - The plan acknowledges this in `Sources`. It is recorded so the gap is visible.

## Gate

*(verified at c7e858f)* All commands were run in the real tree at `c7e858f`, with no drift under
`src`, `tests`, `Cargo.toml`, `build.rs` or the docs in scope.

**Batched proofs.** `cargo test --lib -- --exact <14 check tests> dispositivo::testes::dois_arcavault_sao_recusa_dura`: 15 passed, 0 failed, exit 0.

**Full suite.** `cargo test --no-fail-fast -- --show-output`: 937 passed, 0 failed, exit 0. The totals are identical to round 1, as expected for a README-only fix.

- 824 in the lib.
- 113 across the 13 integration files: b10 2, e10 32, e11 9, e12 18, e1 5, e2 6, e4 6, e7 9, e8 6, e9 9, repasse 5, s1 2, s6 4.
- 0 doctests.

**Hardware skips, re-checked at c7e858f.** An ARCA device was connected. There were zero `nenhum dispositivo ARCA conectado` lines. The same six `pulado:` lines as round 1 appeared, mapped by their `---- <test> stdout ----` headers to the same four tests, which skip for missing historical files:

- `a_verificacao_nao_encostou_no_desfecho_do_backup` (1 line)
- `o_grub_cfg_que_o_clonezilla_entrega_desarma_para_o_inerte_deste_dispositivo` (1)
- `as_copias_armadas_do_dispositivo_desarmam_para_o_inerte_corrente` (3)
- `o_arca_fim_de_21_08_continua_sem_selo` (1)

**Lint and format.**

- `cargo clippy --all-targets --quiet -- -D warnings`: exit 0.
- `cargo fmt --all --check`: exit 0.

**Documentation pipelines** (checks.md, run as written in Git Bash from the repo root):

- C15-C23 each exit 0 at `c7e858f`.
- Base runs, carried from 9112f30: against the `f2563d7` files each exits 1. The fix did not change any pipeline or any base file.

**Pre-change oracle.** C12-C14 at `6840274` gave 3 passed, 0 failed (carried from 9112f30).
