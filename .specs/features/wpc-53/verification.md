# WPC-53 - `arca sondar` e `arca verify --completo` recusam C-6 e o dispositivo partido - report

**Verdict**: PASS
**Profile**: light
**Diff range**: 880f3f3..f2563d7
**Round**: 3 - scoped
**Verifier**: independent sub-agent (author != verifier)

All 12 checks are proven at `f2563d7a5e88c377d851806d0f47fff9a9d71783`, each with a located
assertion. The nine cargo proofs ran in one invocation and each shows individually as `ok`. The
three README pipelines exit 0. The full gate is green: 924 tests passed, 0 failed, and fmt and
clippy are clean.

**Scope of this round.** It follows verify.md's "Re-verifying after a fix". The fix
(`630c8ef..f2563d7`) is a single commit that touches only `README.md`, replacing one line inside
§6.2's "O que ela recusa antes da pergunta" (line 691) with a rewording of the same sentence -
same line count (`git diff --stat 630c8ef..f2563d7`: `README.md | 2 +-`, i.e. one line replaced,
nothing added or removed). No `src/`, `tests/`, `Cargo.toml` or `build.rs` line moved:
`git diff --stat 630c8ef..f2563d7 -- src tests Cargo.toml build.rs` was run and is empty, and
`git diff HEAD --stat -- src tests Cargo.toml build.rs README.md` against the working tree is
also empty, so this run is a HEAD run. `git status --porcelain` matches the session-start set
exactly (CLAUDE.md, docs/agents/issue-tracker.md, docs/agents/triage-labels.md, .agents/, .claude/,
.specs/, docs/adr/0027-o-arcaboot-pode-levar-um-nome.md) - no drift. The round-2 finding this fix
targets said the old sentence read as if `--dry-run` disarms.

- C1-C7, C10, C11: **carried from 630c8ef** - proofs and source-file citations
  (`src/comandos/sondar.rs`, `src/comandos/verify.rs`) are unaffected because the fix touched only
  `README.md`. Proofs were re-run in full at `f2563d7` regardless, per verify.md's rule that green
  is a property of a commit.
- C8, C9, C12: **verified at f2563d7** - README citations refreshed as instructed. C8's underlying
  sentence (line 691) is the one the fix rewrote; C9's and C12's second half (§6.6, line 1128) is
  byte-identical to 630c8ef, and that identity was re-confirmed rather than assumed.

The new §6.2 sentence (`README.md:691`) now reads:

> As duas acontecem depois do desarme e do cabeçalho, que já contam o que aconteceu, e **antes**
> da pergunta. Nada é armado. E valem também no `--dry-run`, que não desarma mas recusa igual.

This states plainly that the dry run does **not** disarm ("não desarma") while still refusing
("recusa igual"), still says the refusals happen before the question ("**antes** da pergunta"),
and still contains the literal substring `também no \`--dry-run\``. The old wording implied via
juxtaposition ("...que já contam o que aconteceu, e antes da pergunta — também no `--dry-run`")
that the disarm-and-header line applied to the dry run too; the new wording separates "nothing is
armed" from "and it also holds in `--dry-run`, which does not disarm but refuses all the same"
into two sentences, closing the ambiguity round 2 flagged. Note that this resolution rests on
reading, the same way C8/C9's "before the question" reading does (finding R1#1 below): running the
C12 pipeline against `git show 630c8ef:README.md` also exits 0, because the pipeline only checks
for the literal substring `também no \`--dry-run\``, which was present (just differently placed)
in both versions. No mechanical check distinguishes the old, misleading placement from the new,
clear one - only reading `README.md:691` does.

## Binding sources

*(carried from 630c8ef - `light` profile does not require this step to re-run, and the fix did not
touch the interface; retained for continuity since round 2 recorded it.)*

| Source | Opened | Contradiction | Uncovered |
| --- | --- | --- | --- |
| Linear WPC-53 | yes, in round 2 (fetched 2026-09-28T11:21Z); ACs 1-6 unaffected by this round's README-only fix | none | - |
| user decision of 2026-09-28 (the ensaio also refuses) | not opened directly - relayed by the brief and recorded at `checks.md:25-27` (`Confirmed? y`) and `checks.md:125` | none | - |

AC-to-check mapping (carried from 630c8ef, unaffected by this round):

- **AC 1** -> C1, with C3 as positive control.
- **AC 2** -> C4.
- **AC 3** -> C2 (sondar) and C5 (verify).
- **AC 4** -> C7, counter-controlled by C6.
- **AC 5** -> folded into C1, C2, C4, C5, each citing its rule in a comment (C-10 at
  `src/comandos/sondar.rs:535`, `src/comandos/verify.rs:918`; C-6 at `src/comandos/sondar.rs:559`,
  `src/comandos/verify.rs:942`).
- **AC 6** -> C8 and C9.

## Checks

| Check | Claim | Proof run | Evidence | Result |
| --- | --- | --- | --- | --- |
| C1 | `sondar::executar`, vault E and boot R on different physical disks, console answers `s` -> `DispositivoPartido { vault: 'E', boot: 'R' }`, nothing armed, cites C-10 | `cargo test --lib -- comandos::sondar::testes comandos::verify::testes` exit 0, 27 passed; `o_sondar_recusa_o_dispositivo_partido_antes_da_pergunta ... ok` | **carried from 630c8ef.** `src/comandos/sondar.rs:544-553` has `matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' }))` on `executar(&bancada.contexto()).unwrap_err()` at `src/comandos/sondar.rs:542`; `:554` calls `bancada.nada_foi_armado()` (helper detailed below). File unchanged since 630c8ef (README-only diff) | PASS |
| C2 | boot disk `Removivel` -> `sondar::executar` with `s` answered returns `MidiaRemovivel`, nothing armed, cites C-6 | same invocation; `o_sondar_recusa_midia_removivel_antes_da_pergunta ... ok` | **carried from 630c8ef.** `src/comandos/sondar.rs:569` has `matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::MidiaRemovivel))` on `executar(...)` at `:566`; `:572` calls `nada_foi_armado()`. File unchanged since 630c8ef | PASS |
| C3 | this desk's device, `s` answered -> `Ok`, `estado.json` armado, `grub.cfg` has `ARCA_PROBE`, firmware gets `bootsequence`, `reinicios() == 1` | same invocation; `com_o_sim_o_sondar_arma_e_so_entao_reinicia ... ok` | **carried from 630c8ef.** `src/comandos/sondar.rs:611` `executar(...).expect("arma e reinicia")`; `:617` situacao armado; `:623` `grub.contains("ARCA_PROBE")`; `:630` firmware write contains `"bootsequence"`; `:633` `assert_eq!(bancada.sistema.reinicios(), 1)`. File unchanged since 630c8ef | PASS |
| C4 | `verify::executar(_, "2026-08-22_Apps", true)`, split disks, console types the name -> `DispositivoPartido { vault: 'E', boot: 'R' }`, nothing armed, cites C-10 | same invocation; `o_completo_recusa_o_dispositivo_partido_antes_da_confirmacao ... ok` | **carried from 630c8ef.** `src/comandos/verify.rs:927-936` has `matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' }))` on `executar(&bancada.contexto(), IMAGEM, true).unwrap_err()` at `:925`; `:937` calls `nada_foi_armado()`. File unchanged since 630c8ef | PASS |
| C5 | boot disk `Removivel` -> `verify --completo` with the right name typed returns `MidiaRemovivel`, nothing armed, cites C-6 | same invocation; `o_completo_recusa_midia_removivel_antes_da_confirmacao ... ok` | **carried from 630c8ef.** `src/comandos/verify.rs:952` has `matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::MidiaRemovivel))` on `executar(..., true)` at `:949`; `:955` calls `nada_foi_armado()`. File unchanged since 630c8ef | PASS |
| C6 | this desk's device, name typed -> `Ok`, armado, `ARCA_VERIFY`, `bootsequence`, `reinicios() == 1`, `discos_fisicos()` queried exactly once | same invocation; `com_a_confirmacao_certa_o_completo_arma_e_so_entao_reinicia ... ok` | **carried from 630c8ef.** `src/comandos/verify.rs:994` `.expect("arma e reinicia")`; `:1000` situacao armado; `:1006` `grub.contains("ARCA_VERIFY")`; `:1013` bootsequence; `:1016` `assert_eq!(bancada.sistema.reinicios(), 1)`; `:1017` `assert_eq!(bancada.discos.consultas_aos_discos.get(), 1)`. File unchanged since 630c8ef | PASS |
| C7 | `verify::executar(_, "2026-08-22_Apps", false)`, no `--dry-run`, MD5SUMS matches -> `Ok` and 0 queries | same invocation; `sem_completo_os_discos_fisicos_nao_sao_consultados ... ok` | **carried from 630c8ef.** `src/comandos/verify.rs:1027` `executar(&bancada.contexto(), IMAGEM, false).expect("V-1 aprova")`; `:1029` `assert_eq!(bancada.discos.consultas_aos_discos.get(), 0)`. File unchanged since 630c8ef | PASS |
| C8 | README §6.2 names C-6 and C-10 and says they happen before the question | the checks.md pipeline, run as written: exit 0 at HEAD (`f2563d7`) | **verified at f2563d7** (this section's sentence is the one the fix rewrote). `README.md:688` is the C-6 bullet and `README.md:689` the C-10 bullet, under the heading "O que ela recusa antes da pergunta" at `README.md:684` (unique - single match for `^### 6\.2 ` and `^### 6\.3 `). `README.md:691` (rewritten) says "...e **antes** da pergunta." The "antes" part still rests on reading, same precision gap as round 1/2 (finding R1#1, non-blocking) | PASS |
| C9 | README §6.6 Recusas names C-6 and C-10 as `--completo` refusals before the typed confirmation | the checks.md pipeline: exit 0 at HEAD (`f2563d7`) | **verified at f2563d7.** `README.md:1128` still reads "O `--completo` recusa ainda, depois do desarme e **antes** da confirmação digitada, ... (**C-6**) ... (**C-10**)", byte-identical to 630c8ef (§6.6 is outside the diff, re-confirmed with `git diff --stat 630c8ef..f2563d7`). `#### Recusas` is still unique (`README.md:1124`), inside §6.6 (`README.md:1033`-`README.md:1132`) | PASS |
| C10 | `sondar::executar` with `dry_run: true` and split disks -> `DispositivoPartido { vault: 'E', boot: 'R' }` instead of `Ok` with the recipe; no firmware write at all; no `estado.json`, `grub.cfg` inert, `reinicios() == 0` | same invocation; `o_ensaio_do_sondar_tambem_recusa_o_dispositivo_partido ... ok` | **carried from 630c8ef.** `src/comandos/sondar.rs:585-594` `matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' }))` on `executar(&bancada.ensaio()).unwrap_err()` at `:583`; `:596-600` `assert!(bancada.firmware.executados().is_empty(), ...)`; `:601` `nada_foi_armado()`. File unchanged since 630c8ef | PASS |
| C11 | `verify::executar(_, "2026-08-22_Apps", true)` with `dry_run: true` and split disks -> `DispositivoPartido { vault: 'E', boot: 'R' }` instead of `Ok` with the recipe; no firmware write; no `estado.json`, `grub.cfg` inert, `reinicios() == 0` | same invocation; `o_ensaio_do_completo_tambem_recusa_o_dispositivo_partido ... ok` | **carried from 630c8ef.** `src/comandos/verify.rs:968-977` `matches!(erro, Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' }))` on `executar(&bancada.ensaio(), IMAGEM, true).unwrap_err()` at `:966`; `:979-983` `assert!(bancada.firmware.executados().is_empty(), ...)`; `:984` `nada_foi_armado()`. File unchanged since 630c8ef | PASS |
| C12 | README says, with the words ``também no `--dry-run` ``, that the ensaio refuses: in §6.2's "O que ela recusa antes da pergunta" and in §6.6's Recusas | the checks.md pipeline, run as written: exit 0 at HEAD (`f2563d7`) | **verified at f2563d7.** `README.md:691` (rewritten) reads "...E valem também no `--dry-run`, que não desarma mas recusa igual." inside the range `README.md:684`-`README.md:693` (unique anchors). `README.md:1128` (unchanged) reads "...(**C-10**) — também no `--dry-run`, que não desarma mas recusa igual." inside the Recusas range `README.md:1124`-`README.md:1132`. Both halves match the literal grep pattern `também no \`--dry-run\``. The pipeline itself cannot tell old wording from new (see note above the checks table) | PASS |

### README section anchors, re-confirmed unique at f2563d7

`grep -n '^### 6\.2 \|^### 6\.3 \|^#### O que ela recusa antes da pergunta\|^#### O que aparece na tela\|^#### Recusas\|^### 6\.7 ' README.md` returns exactly one hit per anchor (684, 638, 693, 752, 1124, 1132), so the `sed` ranges the three pipelines depend on are well-formed.

### The "nada foi armado" helper (carried from 630c8ef)

C1, C2 and C10 rely on `Bancada::nada_foi_armado` at `src/comandos/sondar.rs:482-507`. C4, C5 and
C11 rely on the identical helper at `src/comandos/verify.rs:865-890`. Neither file is in the diff
`630c8ef..f2563d7`, so both bodies are unchanged.

| Effect | sondar | verify |
| --- | --- | --- |
| console read 0 times (`assert_eq!` on `self.console.lidas.get()`) | `src/comandos/sondar.rs:483` | `src/comandos/verify.rs:866` |
| `conteudo_de(r"R:\arca\estado.json").is_none()` | `src/comandos/sondar.rs:489` | `src/comandos/verify.rs:872` |
| `grub.cfg` equals `Some(GRUB_INERTE)` | `src/comandos/sondar.rs:492-498` | `src/comandos/verify.rs:875-881` |
| every firmware write's first argument is `/deletevalue` | `src/comandos/sondar.rs:500` | `src/comandos/verify.rs:883` |
| `assert_eq!(self.sistema.reinicios(), 0, "reiniciou")` | `src/comandos/sondar.rs:506` | `src/comandos/verify.rs:889` |

C10 and C11 additionally assert `bancada.firmware.executados().is_empty()`
(`src/comandos/sondar.rs:596-600`, `src/comandos/verify.rs:979-983`), which is not vacuous: a
disarm during the ensaio would leave one entry in `executados()`, since
`FirmwareDeMentira::executar` records every call before doing anything else
(`src/duplos.rs:335-338`) and `desarme::executar` always issues `/deletevalue`
(`src/desarme.rs:207`, no guard). This is the assertion round-2's mutant M2 (below) killed.

### Intent ordering claims against the code (carried from 630c8ef; production lines unchanged since 10dd87c)

| Claim in `## Intent` | Code | Holds |
| --- | --- | --- |
| sondar: `discos_fisicos()` before the desarme | `src/comandos/sondar.rs:58`, then the desarme at `src/comandos/sondar.rs:67-75` | yes |
| sondar: judge after the header and before `--dry-run`, so the ensaio refuses too (`Confirmed? y`, `checks.md:25-27`) | header at `src/comandos/sondar.rs:80-83`, judge at `:88`, dry-run at `:90`, SD-6 question at `:99` | yes; proven by C10 |
| verify: query only on the `--completo` path, after `achar` | `achar` at `src/comandos/verify.rs:98`, branch at `:100-101`, query at `src/comandos/verify.rs:394` inside `armada` (`:382`) | yes |
| verify: `discos_fisicos()` before the desarme | `src/comandos/verify.rs:394`, then the desarme at `src/comandos/verify.rs:397-405` | yes |
| verify: judge after the header and before `--dry-run` and the S-2 typed confirmation | header at `src/comandos/verify.rs:411-450`, judge at `:456`, dry-run at `:458`, `confirmacao::pedir` at `:477` | yes; proven by C11 |
| house pattern `restore.rs:1025` / `:1057`, `backup.rs:93` / `:138` | `git diff --stat 880f3f3..f2563d7 -- src/comandos/restore.rs src/comandos/backup.rs` is empty - neither file is touched anywhere in the whole feature range | yes |

### Swept row re-check: `prevoo.rs:261-297` (finding R1#2, corrected in checks.md)

`checks.md:107` now cites `prevoo.rs:261-297` for the "dependency failure" row, split as
"o C-6 em `:261-272` e o C-10 em `:274-295`". Read against `src/prevoo.rs`:

- `:261-263` is the comment naming C-6; `:270-272` is `if do_dispositivo.is_some_and(...) { return Err(RecusaDoPreVoo::MidiaRemovivel); }` - the C-6 half.
- `:274-278` is the comment naming C-10; `:279-295` is the `if let (Some(vault), Some(boot)) = (...)` block ending in `return Err(RecusaDoPreVoo::DispositivoPartido { vault, boot });` at `:293` - the C-10 half.
- `:297` is `Ok(())`, the function's fall-through.

The citation accurately spans both refusal branches inside `julgar_o_dispositivo`
(`src/prevoo.rs:257-298`). Finding R1#2 is resolved.

### Other Swept rows that say `existing` or `n/a` (carried from 630c8ef)

`git diff --stat 10dd87c..f2563d7 -- src/prevoo.rs src/main.rs build.rs src/erro.rs
src/comandos/restore.rs` is empty - none of the files these rows cite moved since `10dd87c`, so the
round-1/2 judgments stand unchanged (`prevoo.rs`'s only change in the whole feature landed in
`880f3f3..10dd87c`, confirmed by `git diff --stat 880f3f3..10dd87c -- src/prevoo.rs`: `4 ++++`):

- **validation:** C1, C2, C4, C5 - the device is judged before arming (see Checks table above).
- **failure modes:** holds - `discos_fisicos()?` propagates the WMI failure, and the query sits
  before the desarme (`restore.rs:1025` pattern), so a failure there touches nothing.
- **idempotency:** n/a - the refusal writes nothing beyond C-1's desarme, already idempotent and
  proven in `desarme.rs`.
- **authorization:** holds - elevation (`requireAdministrator` + UAC relaunch in `main.rs`)
  unchanged.
- **concurrency:** n/a - single-user CLI, one command at a time.
- **data lifecycle:** n/a - this change persists nothing new.
- **dependency failure:** holds, re-read at `f2563d7` - see the dedicated re-check above.
- **state transitions:** C1, C2, C4, C5 (inert stays inert), C3, C6 (inert -> armed only through
  the refusal).
- **observability:** holds - the refusal surfaces as `Erro::PreVooRecusou` with the C-6/C-10 text
  that `main` prints, after the header line that already reports the disarm.

### Stale count re-check (finding R2-new#1, corrected in checks.md)

`checks.md:35` now reads "12 checks in 4 slices · 0 one-way doors · 0 open" - the file has 12
checks (C1-C12), matching the count. Finding R2-new#1 is resolved.

## Findings, by origin

- **R1#1 - precision gap in C8 and C9. Open, non-blocking, carried from 630c8ef.** The pipelines
  still `grep -q` only the IDs; the "before the question or confirmation" reading rests on reading
  `README.md:684`, `:691` and `:1128`. The pipeline definitions in `checks.md` did not change this
  round.
- **R1#2 - short Swept citation. Resolved this round** (checks.md now cites `prevoo.rs:261-297`,
  confirmed above to span both refusal branches).
- **R2-new#1 - stale "11 checks" count. Resolved this round** (checks.md now says 12, confirmed
  above).
- **R2-new#2 - C-6 in the ensaio is backed by structure, not by a dedicated test. Open,
  non-blocking, carried from 630c8ef.** C10 and C11 exercise only the C-10 branch of the dry run;
  the C-6 half of the same refusal holds because both checks come from the single call to
  `julgar_o_dispositivo` before the `--dry-run` branch (`src/comandos/sondar.rs:88`,
  `src/comandos/verify.rs:456`), which takes no `dry_run` parameter (`src/prevoo.rs:257-260`) and
  judges C-6 first (`:270-272`). Unaffected by the README-only fix.
- **R2-new#3 - README wording implying the dry run disarms. Resolved this round.**
  `README.md:691` now reads "Nada é armado. E valem também no `--dry-run`, que não desarma mas
  recusa igual.", separating "nothing is armed" from "and it holds in `--dry-run` too, which does
  not disarm but refuses all the same" into two sentences. This removes the juxtaposition that made
  round 2 read the old sentence as if the disarm-and-header line covered the dry run. As noted
  above the checks table, the resolution itself rests on reading, not on a new pipeline distinction
  - `git show 630c8ef:README.md` still passes C12's pipeline.

## Faults injected

Optional under `light`. **Carried from 630c8ef** - this round's fix touched only prose in
`README.md`, no code, so there is no new surface to fault-inject and the round-2 mutants remain the
last empirical evidence for C10/C11:

| Mutation | Location | Killed |
| --- | --- | --- |
| M1: `julgar_o_dispositivo` moved below the `if contexto.dry_run { ...; return Ok(()) }` block, in both commands | `src/comandos/sondar.rs:88`, `src/comandos/verify.rs:456` | yes |
| M2: the desarme also runs in the ensaio (`let desarme = if contexto.dry_run` changed to `if false`) | `src/comandos/sondar.rs:67`, `src/comandos/verify.rs:397` | yes |

The round-1 revert experiment (C1, C2, C4, C5 and the C6 counter going red with the production
hunks reverted) is carried from `10dd87c`; the fix did not touch those production lines either.

## Gate

All commands were run in the real tree at `f2563d7`.

`cargo test --lib -- comandos::sondar::testes comandos::verify::testes`: 27 passed, 0 failed, exit
0 - all nine named proofs (C1-C7, C10, C11) individually `ok`.

`cargo test`: 924 passed, 0 failed, exit 0.

- 811 in the lib.
- 113 across the 13 integration files: b10 2, e1 5, e2 6, e4 6, e7 9, e8 6, e9 9, e10 32, e11 9,
  e12 18, repasse 5, s1 2, s6 4.
- 0 doctests.

Totals are identical to round 2's 924 passed (630c8ef), consistent with the fix touching no test
code and no production code.

**Hardware skip status, re-checked at f2563d7** (not assumed carried-forward): an ARCA device was
connected and the session elevated. `cargo test -- --show-output 2>&1 | grep -nE "pulado|nenhum
dispositivo ARCA conectado"` returned zero `nenhum dispositivo ARCA conectado` lines and six
`pulado:` lines, mapped by `---- <test> stdout ----` headers to the same four tests round 2 found,
skipping for the same historical-file reasons (this device does not carry these specific dated
artifacts):

- `a_verificacao_nao_encostou_no_desfecho_do_backup` (e11) - 1 line, "os dois desfechos de
  `2026-08-22_Apps` nao estao no dispositivo".
- `as_copias_armadas_do_dispositivo_desarmam_para_o_inerte_corrente` (e4) - 3 lines, one per
  missing file (`grub.cfg.teste01`, `.teste02`, `.backup02`).
- `o_grub_cfg_que_o_clonezilla_entrega_desarma_para_o_inerte_deste_dispositivo` (e4) - 1 line,
  `grub.cfg.original`.
- `o_arca_fim_de_21_08_continua_sem_selo` (e8) - 1 line, `arca-fim.txt`.

No other hardware test printed a skip message, so the remaining hardware tests ran against the
connected device rather than skipping.

`cargo fmt --all --check`: exit 0.

`cargo clippy --all-targets -- -D warnings`: exit 0.

README pipelines (checks.md, run as written):

- C8: exit 0.
- C9: exit 0.
- C12: exit 0.
