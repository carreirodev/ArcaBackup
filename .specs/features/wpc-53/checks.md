# `arca sondar` e `arca verify --completo` recusam C-6 e o dispositivo partido - checks

Profile: light
Plan: none - change under the plan threshold, no one-way door; the binding source is the Linear issue WPC-53

## Intent

Hoje `arca sondar` e `arca verify <nome> --completo` armam sem chamar
`prevoo::julgar_o_dispositivo`, e nenhum dos dois consulta `discos_fisicos()`. Com o `ARCABOOT`
em midia removivel (C-6), o `armar` so descobre a rejeicao silenciosa do `bcdedit` na releitura,
depois de a pessoa ter confirmado. Com o `ARCAVAULT` e o `ARCABOOT` em discos fisicos diferentes
(C-10, dispositivo partido), o `estado.json` vai para o `ARCABOOT` de um dispositivo e o desfecho
para o `ARCAVAULT` do outro, e a colheita procura o desfecho no lugar errado. Quem paga e quem
tem dois dispositivos meio prontos na mesa, ou um SSD que o Windows chama de removivel: o comando
arma, reinicia, e o job se perde.

Quando isto entrar, os dois comandos recusam os dois casos **antes** da pergunta/confirmacao,
com as mesmas mensagens do `arca backup` e do `arca restore`, e sem armar nada. `arca verify`
sem `--completo` continua sem pagar a consulta WMI.

Decisoes tomadas sem perguntar (house pattern, reversiveis):

- A consulta `discos_fisicos()` vem **antes** do desarme, como `restore.rs:1025` e `backup.rs:93`:
  uma falha do WMI depois do desarme esconderia a linha que conta que ele aconteceu.
- `julgar_o_dispositivo` vem depois do cabecalho e **antes** do ramo `--dry-run`, como
  `restore.rs:1057` e `backup.rs:138` - entao o ensaio tambem recusa. Confirmed? y - o usuario, em
  28/09/2026, depois da primeira verificacao ("siga todas as suas recomendacoes"); provado por C10 e C11.
- No `verify`, a consulta mora em `armada`, depois de `achar`: `NaoExiste` e `EResiduo` continuam
  vindo primeiro (o mais barato de corrigir primeiro), e V-1 nunca a paga.
- A AC 5 da issue ("os testes seguem os de `restore.rs`") e lida no sentido mais forte: os testes
  de `restore.rs` chamam `julgar_o_dispositivo` direto e passariam sem conserto nenhum; estes
  dirigem `executar` por uma `Bancada` com `DiscosDeMentira`, com o console respondendo o que
  arma, de modo que so a recusa separa o comando de armar.

12 checks in 4 slices · 0 one-way doors · 0 open

## Checks

### S1 - `arca sondar` recusa C-6 e C-10 antes da pergunta · 2 files · 68 KB · ~17k

**C1** - WHEN `sondar::executar` roda com o `ARCAVAULT` (`E:`) e o `ARCABOOT` (`R:`) em discos fisicos diferentes e o console respondendo `s` THEN devolve `Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' })`, com `console.lidas == 0`, sem `R:\arca\estado.json`, `grub.cfg` igual ao inerte, nenhuma escrita no firmware alem de `/deletevalue`, e `reinicios() == 0`; o teste cita C-10 (WPC-53, AC 1, AC 5)
Proof: `cargo test --lib comandos::sondar::testes::o_sondar_recusa_o_dispositivo_partido_antes_da_pergunta -- --exact`

**C2** - IF o disco do `ARCABOOT` tem `tipo_de_midia == Removivel` THEN `sondar::executar`, com o console respondendo `s`, devolve `Erro::PreVooRecusou(RecusaDoPreVoo::MidiaRemovivel)`, com `console.lidas == 0`, sem `estado.json`, `grub.cfg` inerte, so `/deletevalue` no firmware e `reinicios() == 0`; o teste cita C-6 (WPC-53, AC 3, AC 5)
Proof: `cargo test --lib comandos::sondar::testes::o_sondar_recusa_midia_removivel_antes_da_pergunta -- --exact`

**C3** - WHEN `sondar::executar` roda no dispositivo desta mesa (os dois rotulos no disco 1, `DiscoExterno`) com o console respondendo `s` THEN devolve `Ok`, grava `R:\arca\estado.json` com `"situacao": "armado"`, o `grub.cfg` passa a conter `ARCA_PROBE`, o firmware recebe `bootsequence`, e `reinicios() == 1` (controle positivo de C1 e C2)
Proof: `cargo test --lib comandos::sondar::testes::com_o_sim_o_sondar_arma_e_so_entao_reinicia -- --exact`

### S2 - `arca verify --completo` recusa C-6 e C-10 antes da confirmacao digitada · 2 files · 81 KB · ~20k

**C4** - WHEN `verify::executar(_, "2026-08-22_Apps", true)` roda com o `ARCAVAULT` e o `ARCABOOT` em discos fisicos diferentes e o console digitando `2026-08-22_Apps` THEN devolve `Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' })`, com `console.lidas == 0`, sem `estado.json`, `grub.cfg` inerte, so `/deletevalue` no firmware e `reinicios() == 0`; o teste cita C-10 (WPC-53, AC 2, AC 5)
Proof: `cargo test --lib comandos::verify::testes::o_completo_recusa_o_dispositivo_partido_antes_da_confirmacao -- --exact`

**C5** - IF o disco do `ARCABOOT` tem `tipo_de_midia == Removivel` THEN `verify::executar(_, "2026-08-22_Apps", true)`, com o console digitando o nome certo, devolve `Erro::PreVooRecusou(RecusaDoPreVoo::MidiaRemovivel)`, com `console.lidas == 0`, sem `estado.json`, `grub.cfg` inerte, so `/deletevalue` no firmware e `reinicios() == 0`; o teste cita C-6 (WPC-53, AC 3, AC 5)
Proof: `cargo test --lib comandos::verify::testes::o_completo_recusa_midia_removivel_antes_da_confirmacao -- --exact`

**C6** - WHEN `verify::executar(_, "2026-08-22_Apps", true)` roda no dispositivo desta mesa com o console digitando `2026-08-22_Apps` THEN devolve `Ok`, grava `estado.json` com `"situacao": "armado"`, o `grub.cfg` passa a conter `ARCA_VERIFY`, o firmware recebe `bootsequence`, `reinicios() == 1`, e `discos_fisicos()` foi consultado exatamente 1 vez (controle positivo de C4, C5 e do contador de C7)
Proof: `cargo test --lib comandos::verify::testes::com_a_confirmacao_certa_o_completo_arma_e_so_entao_reinicia -- --exact`

**C7** - WHEN `verify::executar(_, "2026-08-22_Apps", false)` roda (V-1, sem `--dry-run`, com um `MD5SUMS` que bate) THEN devolve `Ok` e `discos_fisicos()` foi consultado 0 vezes (WPC-53, AC 4)
Proof: `cargo test --lib comandos::verify::testes::sem_completo_os_discos_fisicos_nao_sao_consultados -- --exact`

### S3 - README · 1 file · 116 KB · ~29k

**C8** - A secao §6.2 (`arca sondar`) do `README.md` nomeia as duas recusas, C-6 e C-10, e diz que elas acontecem antes da pergunta (WPC-53, AC 6)
Proof: `sed -n '/^### 6\.2 /,/^### 6\.3 /p' README.md | grep -q 'C-6' && sed -n '/^### 6\.2 /,/^### 6\.3 /p' README.md | grep -q 'C-10'`

**C9** - A secao §6.6 (`arca verify`), na subsecao Recusas, nomeia C-6 e C-10 como recusas do `--completo` antes da confirmacao digitada (WPC-53, AC 6)
Proof: `sed -n '/^#### Recusas/,/^### 6\.7 /p' README.md | grep -q 'C-6' && sed -n '/^#### Recusas/,/^### 6\.7 /p' README.md | grep -q 'C-10'`

### S4 - o ensaio tambem recusa (decisao confirmada em 28/09/2026) · 3 files · 160 KB · ~40k

**C10** - WHEN `sondar::executar` roda com `dry_run: true` e o `ARCAVAULT` (`E:`) e o `ARCABOOT` (`R:`) em discos fisicos diferentes THEN devolve `Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' })` em vez de `Ok` com o ensaio da receita, e o firmware nao recebe escrita nenhuma (o ensaio nao desarma), sem `estado.json`, `grub.cfg` inerte e `reinicios() == 0`
Proof: `cargo test --lib comandos::sondar::testes::o_ensaio_do_sondar_tambem_recusa_o_dispositivo_partido -- --exact`

**C11** - WHEN `verify::executar(_, "2026-08-22_Apps", true)` roda com `dry_run: true` e o dispositivo partido THEN devolve `Erro::PreVooRecusou(RecusaDoPreVoo::DispositivoPartido { vault: 'E', boot: 'R' })` em vez de `Ok` com o ensaio da receita, e o firmware nao recebe escrita nenhuma, sem `estado.json`, `grub.cfg` inerte e `reinicios() == 0`
Proof: `cargo test --lib comandos::verify::testes::o_ensaio_do_completo_tambem_recusa_o_dispositivo_partido -- --exact`

**C12** - O README diz, com as palavras `também no \`--dry-run\``, que o ensaio recusa: na subsecao "O que ela recusa antes da pergunta" do §6.2 e na subsecao Recusas do §6.6
Proof: `sed -n '/^#### O que ela recusa antes da pergunta/,/^#### O que aparece na tela/p' README.md | grep -qF 'também no `--dry-run`' && sed -n '/^#### Recusas/,/^### 6\.7 /p' README.md | grep -qF 'também no `--dry-run`'`

## Coverage

| Set (size) | Member -> proof | Unproven |
| --- | --- | --- |
| comandos que armam sem a recusa hoje x recusas do dispositivo (4) | `sondar` · partido C1 · `sondar` · removivel C2 · `verify --completo` · partido C4 · `verify --completo` · removivel C5 | - |
| os mesmos comandos numa mesa saudavel, controle positivo (2) | `sondar` C3 · `verify --completo` C6 | - |
| modos do `verify` x consulta a `discos_fisicos()` (2) | sem `--completo` -> 0 C7 · `--completo` -> 1 C6 | - |
| efeitos que "sem armar nada" exclui (5) | pergunta/confirmacao lida C1 C2 C4 C5 · `estado.json` C1 C2 C4 C5 · `grub.cfg` armado C1 C2 C4 C5 · escrita no firmware alem do desarme C1 C2 C4 C5 · reinicio C1 C2 C4 C5 | - |
| secoes do README (2) | §6.2 C8 · §6.6 Recusas C9 | - |
| comando que arma x modo de execucao, na recusa do dispositivo partido (4) | `sondar` real C1 · `sondar` ensaio C10 · `verify --completo` real C4 · `verify --completo` ensaio C11 | - |
| secoes do README que dizem que o ensaio recusa (2) | §6.2 C12 · §6.6 Recusas C12 | - |

- Claims naming an error variant: C1, C2, C4, C5 - each proof drives `executar`, the command's
  entry point, and not `julgar_o_dispositivo` directly
- No other check claims more than the single case its proof exercises

## Swept

- validation: C1, C2, C4, C5 - o dispositivo e julgado antes de armar
- failure modes: existing - `discos_fisicos()?` propaga a falha do WMI, e a consulta fica antes do desarme (como `restore.rs:1025`), entao uma falha ali nao toca em nada; sem prova propria, porque nenhum duplo de `Discos` falha
- idempotency: n/a - a recusa nao escreve nada alem do desarme de C-1, que ja e idempotente e ja tem prova em `desarme.rs`
- authorization: existing - a elevacao (`requireAdministrator` + relancar por UAC em `main.rs`) nao muda
- concurrency: n/a - CLI de um usuario, um comando por vez; o dispositivo e lido uma vez por execucao
- data lifecycle: n/a - esta mudanca nao persiste nada novo
- dependency failure: existing - um disco que o WMI nao devolve para uma das letras nao vira recusa (`prevoo.rs:261-297` - o C-6 em `:261-272` e o C-10 em `:274-295`, ADR-0005)
- state transitions: C1, C2, C4, C5 (inerte continua inerte), C3, C6 (inerte -> armado so passando pela recusa)
- observability: existing - a recusa sobe como `Erro::PreVooRecusou` com o texto de C-6/C-10 que o `main` imprime, e o cabecalho com a linha do desarme sai antes dela

## Out of scope

- README §6.3 passo 5 e §6.7 linha 2 descrevem C-10 como "rotulo repetido", a mesma imprecisao que a issue aponta - achado paralelo, nao corrigido aqui
- `arca prepare` - roda antes de existirem os rotulos (PRD, secao de PR-5); nao arma no sentido de C-6/C-10

## Handoff

- S1 = 17k (sondar.rs 15 KB + duplos.rs 53 KB); S2 = 20k (verify.rs 28 KB + duplos.rs ja lido); S3 = 29k (README 116 KB); referencias lidas (backup.rs 54 KB, prevoo.rs 46 KB) ~25k. Total ~91k, under the 150k budget - one builder

- **Boundary:** C1-C9 closed at `10dd87c` (feature base `880f3f3`)
- **Settled mid-build:** nothing - no clarification asked; commit message follows the repo's Portuguese prose convention and trailers instead of Conventional Commits (project CLAUDE.md + global rule "follow the repo's existing convention")
- **Abandoned:** copying the `restore.rs` tests that call `julgar_o_dispositivo` directly - they pass with no fix; replaced by tests that drive `executar`

- **Boundary:** C10-C12 closed at `630c8ef` (round 2 after the verification of `10dd87c`)
- **Settled mid-build:** the user confirmed on 28/09/2026 that the `--dry-run` also refuses ("siga todas as suas recomendacoes"); the Intent row is now `Confirmed? y`. Measured on the real device (Realtek RTL9210 NVME, `External hard disk media`, disk 1 with `D:` and `E:`): both `--dry-run` pass the two refusals and print the recipe, 1.6 s and 1.45 s for the whole command
- **Abandoned:** nothing
