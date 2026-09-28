# ARCABOOT com nome - checks

Profile: light
Plan: `.specs/features/arcaboot-com-nome/plan.md`

23 checks in 3 slices · 1 one-way door · 0 open

Decisões tomadas ao derivar, sem mudar o plano aprovado:

- **C3 prova a porta 1 com `ARCAÉ`, e não só com `ÉRCA-X`.** Em `ÉRCA-X` o `É` ocupa os bytes 0 e 1, e o byte 5 é fronteira de caractere: `&rotulo[..5]` devolveria `ÉRCA` sem pânico, e o exemplo da AC 5 passaria sob a implementação que a porta 1 proíbe. Em `ARCAÉ` o `É` ocupa os bytes 4 e 5, e a fatia cai no meio dele. `ARC` cobre o outro pânico, o de rótulo mais curto que o prefixo. A linha da porta 1 no plano diz "um rótulo que começa com acento", o que vale só para alguns: fica como foi aprovada, e esta nota diz qual rótulo prova.
- **A AC 12 tem por oráculo a saída de hoje.** "Byte a byte como hoje" só se prova comparando com o texto literal que o código produz antes da mudança. Os literais de C12, C13 e C14 são capturados do código atual **antes** do primeiro commit que o altera, e entram num commit próprio, verde sem mudança nenhuma.
- **C5 exige a mesma variante, `Erro::DispositivosDemais`.** A AC 4 diz que a recusa é essa, e o código de saída de C-10 sai dela (Observable: "C-10 sai pelo mesmo código de hoje").

## Checks

### S1 - o `ARCABOOT` renomeado continua sendo o dispositivo (WPC-55, AC 1 a 7) · 4 files · 229 KB · ~57k

**C1** - WHEN `dispositivo::encontrar` recebe `Windows` em `C:`, `ARCAVAULT` em `E:` e `ARCA-CASA` em `R:` THEN devolve `Ok` com `boot` igual ao volume `R:` de rótulo `ARCA-CASA`, `caminho_do_grub() == R:\boot\grub\grub.cfg` e `caminho_do_estado() == R:\arca\estado.json` (AC 1)
Proof: `cargo test --lib dispositivo::testes::o_arcaboot_renomeado_e_o_boot_do_dispositivo -- --exact`

**C2** - Para cada rótulo em `ARCABOOT`, `arcaboot`, `ARCA-CASA`, `arca-Casa` e `ARCA-`, com `ARCAVAULT` em `E:` e o rótulo em `R:`, `encontrar` devolve `boot` com letra `R` e rótulo igual, byte a byte, ao que o volume trouxe (AC 2; `ARCA-` pela suposição confirmada em 28/09/2026)
Proof: `cargo test --lib dispositivo::testes::o_rotulo_de_boot_casa_arcaboot_ou_arca_hifen_em_qualquer_caixa -- --exact`

**C3** - Para cada rótulo em `ARCACASA`, `ARC`, `ÉRCA-X` e `ARCAÉ`, com `ARCAVAULT` em `E:` e o rótulo em `R:`, `encontrar` devolve `Ok` com `boot == None`, sem pânico (AC 3, AC 5; Landing, porta 1)
Proof: `cargo test --lib dispositivo::testes::o_que_nao_tem_a_forma_fica_fora_do_arcaboot_sem_panico -- --exact`

**C4** - Com só `ARCAVAULT` em `E:`, `encontrar` devolve `boot == None`: o próprio `ARCAVAULT` não casa como boot. É o teste que já existe, e é ele que mata o prefixo `ARCA` sem hífen (AC 3)
Proof: `cargo test --lib dispositivo::testes::sem_arcaboot_o_dispositivo_ainda_serve_para_listar -- --exact`

**C5** - Para cada par em `R:` e `S:` (`ARCABOOT` + `ARCA-CASA`, `ARCA-CASA` + `ARCA-ESCRIT` e `ARCA-CASA` + `ARCA-CASA`), com `ARCAVAULT` em `E:`, `encontrar` devolve `Err(Erro::DispositivosDemais)` com `quantos == 2` (AC 4, C-10)
Proof: `cargo test --lib dispositivo::testes::dois_volumes_de_boot_sao_recusa_dura_com_rotulos_iguais_ou_diferentes -- --exact`

**C6** - WHEN o disco escolhido no `arca prepare` tem `ARCAVAULT` e `ARCA-CASA` THEN `montar_o_plano` contém `ESTE DISCO JA E UM DISPOSITIVO ARCA` e `AS IMAGENS` (AC 6, PR-4)
Proof: `cargo test --lib comandos::prepare::testes::preparar_por_cima_de_um_dispositivo_nomeado_avisa_das_imagens -- --exact`

**C7** - Para cada operação (backup, restauração, verificação e sondagem), `Receita::parametros_do_grub()` contém `dev:///LABEL=ARCAVAULT`, contém `LABEL=` uma vez só e não contém `ARCABOOT` em caixa nenhuma (AC 7, S-3)
Proof: `cargo test --lib receita::testes::nenhuma_receita_cita_o_rotulo_do_arcaboot -- --exact`

### S2 - o nome aparece onde o ARCA mostra o dispositivo (WPC-56, AC 8 a 12) · 2 new files · 97 KB · ~24k

**C8** - WHEN o `boot` do dispositivo tem rótulo `ARCA-CASA`, em `R:`, FAT32, 1.700.000.000 bytes THEN a saída do `arca status` contém `linha("ARCA-CASA", "R: · FAT32 · 1,6 GB")` e não contém `linha("ARCABOOT", "R: · FAT32 · 1,6 GB")`. Com o rótulo `arca-Casa` a saída contém `linha("arca-Casa", "R: · FAT32 · 1,6 GB")`, como o Windows o devolve (AC 8)
Proof: `cargo test --lib comandos::status::testes::o_status_mostra_o_nome_do_arcaboot -- --exact`

**C9** - WHEN os volumes são `ARCAVAULT` em `E:`, `ARCA-CASA` em `R:` e `ARCA-ESCRIT` em `S:` THEN a mensagem de `encontrar` contém `R: ARCA-CASA, S: ARCA-ESCRIT`, `estado do job`, `receita`, `Desconecte os demais` e `Se voce acabou de preparar um dispositivo`, e não contém `com o rotulo ARCABOOT`, `repetido` nem `e pelo rotulo que a receita resolve o destino`. Com `ARCABOOT` em `R:` e `ARCA-CASA` em `S:`, a mensagem contém `R: ARCABOOT, S: ARCA-CASA` (AC 9)
Proof: `cargo test --lib dispositivo::testes::a_recusa_por_dois_volumes_de_boot_nomeados_diz_a_letra_e_o_rotulo_de_cada_um -- --exact`

**C10** - WHEN os volumes são `ARCAVAULT` em `E:` e em `F:`, `ARCA-CASA` em `R:` e `ARCABOOT` em `S:` THEN `encontrar` devolve `Erro::DispositivosDemais` com `rotulo == ARCAVAULT` e `onde == "E:, F:"`, e a mensagem contém `(E:, F:)` e `R: ARCA-CASA, S: ARCABOOT` (AC 10)
Proof: `cargo test --lib dispositivo::testes::a_recusa_por_dois_arcavault_nomeia_os_volumes_de_boot_quando_ha_nome -- --exact`

**C11** - WHEN o menu do `arca prepare` lista um disco com `ARCAVAULT` em `E:` e `ARCA-CASA` em `F:` THEN a linha desse disco termina em `· JA E UM DISPOSITIVO ARCA (ARCA-CASA)` (AC 11)
Proof: `cargo test --lib comandos::prepare::testes::o_menu_diz_o_nome_do_dispositivo_arca -- --exact`

**C12** - WHILE o `boot` tem rótulo `ARCABOOT`, e também `arcaboot`, THEN o bloco `Dispositivo ARCA` do `arca status` é igual, byte a byte, ao literal capturado do código de hoje, com `ARCABOOT` na segunda linha nos dois casos (AC 12)
Proof: `cargo test --lib comandos::status::testes::sem_nome_o_bloco_do_dispositivo_sai_como_antes -- --exact`

**C13** - WHILE nenhum volume tem a forma `ARCA-<texto>` THEN as duas recusas de C-10 são iguais, byte a byte, aos literais capturados do código de hoje: `ARCAVAULT` em `E:` e `F:` com `ARCABOOT` em `R:`, e `ARCABOOT` em `R:` e `S:` com `ARCAVAULT` em `E:` (AC 12)
Proof: `cargo test --lib dispositivo::testes::sem_volume_nomeado_as_recusas_de_c10_saem_como_antes -- --exact`

**C14** - WHILE o disco tem `ARCAVAULT` e `ARCABOOT` THEN a saída inteira de `montar_o_menu` para o duplo desta mesa é igual, byte a byte, ao literal capturado do código de hoje, com a linha terminando em `· JA E UM DISPOSITIVO ARCA` sem parêntese (AC 12)
Proof: `cargo test --lib comandos::prepare::testes::o_menu_do_dispositivo_sem_nome_sai_como_antes -- --exact`

### S3 - a documentação (WPC-57) · 4 files · 340 KB · ~85k

Seguem a L-001: toda prova de documentação procura a redação, e não só o identificador da regra.

**C15** - O PRD tem uma linha de tabela que começa com `| C-16 |` e contém `ARCA-<texto>`, e o README §13 tem uma que começa com `| **C-16** |` e contém `ARCA-<texto>`
Proof: `grep -qE '^\| C-16 \|.*ARCA-<texto>' PRD/PRD-ARCA-v5_1.md && grep -qE '^\| \*\*C-16\*\* \|.*ARCA-<texto>' README.md`

**C16** - A linha de C-10 no PRD §9.1 contém `ARCA-<texto>`, e o README §13 também
Proof: `grep -qE '^\| C-10 \|.*ARCA-<texto>' PRD/PRD-ARCA-v5_1.md && grep -qE '^\| \*\*C-10\*\* \|.*ARCA-<texto>' README.md`

**C17** - O README não contém mais `há rótulo repetido na mesa` (§6.3), `e rótulo repetido |` (§6.7) nem `Recusar rótulo repetido` (§13)
Proof: `! grep -qF 'há rótulo repetido na mesa' README.md && ! grep -qF 'e rótulo repetido |' README.md && ! grep -qF 'Recusar rótulo repetido' README.md`

**C18** - O README §5 não contém `rotuladas sempre com os mesmos nomes` e contém `Explorer`, `ARCA-<texto>` e `6 caracteres`
Proof: `S=$(sed -n '/^## 5\. /,/^## 6\. /p' README.md); ! printf '%s' "$S" | grep -qF 'rotuladas sempre com os mesmos nomes' && printf '%s' "$S" | grep -qF 'Explorer' && printf '%s' "$S" | grep -qF 'ARCA-<texto>' && printf '%s' "$S" | grep -qF '6 caracteres'`

**C19** - A seção §6.8 (`arca status`) do README contém `ARCA-CASA`
Proof: `sed -n '/^### 6\.8 /,/^### 6\.9 /p' README.md | grep -qF 'ARCA-CASA'`

**C20** - A recusa de dois dispositivos mostra o nome nos dois lugares em que o README a descreve: a subseção `Regra única de operação` do §5 e o §12 contêm `ARCA-CASA`
Proof: `sed -n '/^### Regra única de operação/,/^### Os dois estados/p' README.md | grep -qF 'ARCA-CASA' && sed -n '/^## 12\. /,/^## 13\. /p' README.md | grep -qF 'ARCA-CASA'`

**C21** - No `CONTEXT.md`, os verbetes `**ARCABOOT**` e `**Dispositivo**` contêm `ARCA-<texto>`
Proof: `awk '/^\*\*ARCABOOT\*\*:/,/^_Evitar_/' CONTEXT.md | grep -qF 'ARCA-<texto>' && awk '/^\*\*Dispositivo\*\*:/,/^_Evitar_/' CONTEXT.md | grep -qF 'ARCA-<texto>'`

**C22** - Os docs de código deixam de dizer que os rótulos são sempre os mesmos. `src/dispositivo.rs` não contém `Os rotulos sao os mesmos em todo dispositivo` e contém `ARCA-<texto>`. `src/portas/discos.rs` contém `ARCA-<texto>`. `src/erro.rs` não contém `Dois rotulos iguais tornam o destino ambiguo`
Proof: `! grep -qF 'Os rotulos sao os mesmos em todo dispositivo' src/dispositivo.rs && grep -qF 'ARCA-<texto>' src/dispositivo.rs && grep -qF 'ARCA-<texto>' src/portas/discos.rs && ! grep -qF 'Dois rotulos iguais tornam o destino ambiguo' src/erro.rs`

**C23** - O ADR-0027 está versionado, e a tabela de ADRs do README tem uma linha que começa com `| 0027 |`
Proof: `git ls-files --error-unmatch docs/adr/0027-o-arcaboot-pode-levar-um-nome.md && grep -qE '^\| 0027 \|' README.md`

## Coverage

| Set (size) | Member -> proof | Unproven |
| --- | --- | --- |
| rótulos que casam como boot (5) | `ARCABOOT` C2 · `arcaboot` C2 · `ARCA-CASA` C1 C2 · `arca-Casa` C2 · `ARCA-` C2 | - |
| rótulos que não casam (5) | `ARCACASA` C3 · `ARCAVAULT` C4 · `ARC` C3 · `ÉRCA-X` C3 · `ARCAÉ` C3 | - |
| Landing, porta 1: a forma e o `str::get(..5)` (2) | forma C2 C3 C4 · fatia que cairia no meio de um caractere C3 | - |
| lugares que ramificam no rótulo de boot, do Impact (4) | `dispositivo::encontrar` C1 · contagem de C-10 C5 · `prepare::e_um_dispositivo_arca` C6 C11 · linha do `arca status` C8 | - |
| pares de volumes de boot, de C-10 (4) | `ARCABOOT` + `ARCA-CASA` C5 C9 · `ARCA-CASA` + `ARCA-ESCRIT` C5 C9 · `ARCA-CASA` + `ARCA-CASA` C5 · `ARCABOOT` + `ARCABOOT` C13 | - |
| recusas de C-10 x volume nomeado (4) | por boot, com nome C9 · por boot, sem nome C13 · por `ARCAVAULT`, com boot nomeado C10 · por `ARCAVAULT`, sem nome C13 | - |
| telas x volume nomeado (4) | `arca status` com nome C8 · `arca status` sem nome C12 · menu do `prepare` com nome C11 · menu do `prepare` sem nome C14 | - |
| operações da receita (4) | backup C7 · restauração C7 · verificação C7 · sondagem C7 | - |
| lugares da documentação, do Impact (17) | PRD C-16 C15 · README §13 C-16 C15 · PRD C-10 C16 · README §13 C-10 C16 · README §6.3 C17 · README §6.7 C17 · README §13 redação C17 · README §5 anatomia C18 · README §5 regra única C20 · README §6.8 C19 · README §12 C20 · `CONTEXT.md` `ARCABOOT` C21 · `CONTEXT.md` `Dispositivo` C21 · `dispositivo.rs` C22 · `portas/discos.rs` C22 · `erro.rs` C22 · ADR-0027 e a tabela C23 | - |

- Claims naming an error variant: C5, C10 - each proof drives `dispositivo::encontrar`, the function every command calls, and matches the variant
- Claims about a whole screen or message, byte for byte: C12, C13, C14 - each compares with a literal captured before the change, never with a string rebuilt by the code under test
- No other check claims more than the single case its proof exercises

## Swept

- validation: C2, C3, C4 - a forma do rótulo é a validação, e o que não a tem fica fora do boot
- failure modes: C3 - rótulo mais curto que o prefixo e rótulo com caractere de dois bytes no quinto byte respondem "não é boot", sem pânico
- idempotency: n/a - o ARCA só lê o nome, e não escreve rótulo nenhum
- authorization: existing - a elevação (`requireAdministrator` e o relançamento por UAC em `main.rs`) não muda
- concurrency: n/a - CLI de um usuário, um comando por vez, e os volumes são lidos uma vez por execução
- data lifecycle: n/a - nada novo é persistido. Um dispositivo nomeado que passe de novo pelo `prepare` volta a `ARCABOOT` (plan, Impact)
- dependency failure: existing - um volume que não responde fica de fora da enumeração (`adaptadores/windows/volumes.rs`, `ler_volume`), e isso não muda
- state transitions: n/a - o job, o firmware e o `estado.json` não ganham estado nenhum
- observability: C8, C9, C10, C11 - o nome aparece nas telas e nas recusas. C12, C13 e C14 - sem nome, elas não mudam

## Handoff

- S1 = 57k (`dispositivo.rs` 13,8 KB + `comandos/prepare.rs` 112,0 KB + `receita.rs` 96,8 KB + `portas/discos.rs` 6,5 KB = 229 KB / 4). S2 entra em `erro.rs` 25,8 KB e `comandos/status.rs` 70,7 KB, com `dispositivo.rs` e `prepare.rs` já contados: 97 KB / 4 = 24k, acumulado 81k. S3 entra na documentação, com `README.md` 117,3 KB, `PRD-ARCA-v5_1.md` 207,1 KB, `CONTEXT.md` 11,3 KB e o ADR 5,4 KB: 340 KB / 4 = 85k, acumulado 166k, acima do budget de 150k -> corte proposto depois de S2, onde a superfície muda de código para documentação
- Mechanism: one builder (compaction accepted) - escolha do usuário em 28/09/2026, antes de qualquer código. A sessão tem 1M de contexto, e a fatia de documentação edita seções localizadas, e não os 207 KB do PRD inteiros

- **Boundary:** C12-C14 closed at `6840274`, C1-C7 at `05bd7f0`, C8-C11 at `e4896fd`, C15-C23 at `9112f30` (feature base `f2563d7`). `.specs/` was untracked while the feature was built, and was versioned afterwards at the user's request
- **Settled mid-build:** the user confirmed the display scope and the three open assumptions (28/09/2026), and chose one builder over the budget miss. Measured on the real device (`D:` ARCAVAULT, `E:` ARCABOOT): the boot configs carry no `LABEL=`, and `cargo run -- status` prints the unnamed block as before. `thiserror` 2.0.18 extra format args (`.campo`) confirmed in Context7 before `mensagem_de_dispositivos_demais` relied on them. `dois_arcavault_sao_recusa_dura` gained `..` in its pattern because the variant gained a field; its three assertions are untouched. Swapping `get(..5)` for `&rotulo[..5]` turned C3 red (panic), checked before `05bd7f0`. `CONTEXT.md` forbids "partição de boot" as a synonym for `ARCABOOT`, so the docs say "volume de boot" for the candidate and define it in the glossary entry
- **Abandoned:** nothing

- **Boundary:** round-2 fix `c7e858f` (README §5 caption no longer claims a test pins the whole refusal text), re-verified scoped: PASS
- **Settled mid-build:** commit messages follow the repo's Portuguese-prose convention with the session trailers, not Conventional Commits, so `check_commit.py` was not used (project CLAUDE.md + global rule "follow the repo's existing convention", same as WPC-53). `core.hooksPath` is not set in this copy, so the pre-commit hook never ran; fmt, clippy and the suite were run by hand before each code commit
- **Abandoned:** nothing
- **Measured after the feature (28/09/2026):** the user renamed the real `E:` to `ARCA-TEST` in Explorer (`Get-Volume` returns `ARCA-TEST`). With the `c7e858f` build: `arca status` prints `ARCA-TEST` on the boot line; `arca prepare --dry-run` menu ends in `JA E UM DISPOSITIVO ARCA (ARCA-TEST)`; the disk-1 plan shows `ESTE DISCO JA E UM DISPOSITIVO ARCA`; `arca sondar --dry-run` passes C-6 and the split-device refusal and targets `E:\boot\grub\grub.cfg`. A real arm (reboot) with the renamed label was not run
- **Measured on hardware (28/09/2026):** with the boot still labelled `ARCA-TEST`, a real `arca sondar` armed, the machine booted through the firmware entry, and `arca resultado` harvested `concluida — o selo bate e a receita chegou ao fim` (Discos vistos: `sda`, `nvme0n1`; desarme at `E:\boot\grub\grub.cfg`; ordem de boot ok). Recorded in ADR-0027 at the commit after `c7e858f`
