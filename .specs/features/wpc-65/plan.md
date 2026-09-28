# WPC-65 · O `cargo doc` sai sem aviso, e um link quebrado novo não entra

## Problem

A documentação do ARCA é o lugar que o README e o `CLAUDE.md` indicam como "onde as razões estão", e hoje ela tem 13 links que não levam a lugar nenhum. O `cargo doc --no-deps` do `main` avisa 13 vezes (medido em 28/09/2026, `106df91`, rustc 1.98.1):

- **Dez são links de documentação pública para item privado.** O `cargo doc` padrão não gera página para item privado, então quem clica não chega a lugar nenhum.
- **Três são links que não resolvem.** Um deles é o de `RecusaDoPacote::NaoEstaLa` para `conferir_o_pacote_local`, que chegou com a WPC-60.

Nem o hook nem o `semanal.yml` rodam o `cargo doc`, então a contagem só sobe, sem ninguém ver. A WPC-65 mediu 12 avisos antes do merge da WPC-60 e 13 depois. Cada aviso novo se perde no meio dos antigos, e quem lê a documentação dá com um link que parece levar à razão e não leva.

Quando isto for entregue, o `cargo doc` termina sem aviso. Um commit que traga um link quebrado, ou um link para item privado, é recusado pelo hook, e o `semanal.yml` pega o que escapar do hook.

## Flow

Reusa o `passo` do `.githooks/pre-commit` e a forma que o clippy já tem nos dois guarda-corpos, `-- -D warnings` no fim da chamada. Não cria configuração nova, nem no `Cargo.toml` nem em `.cargo/config.toml`.

1. a documentação de 8 arquivos de `src/` (exists) - cada um dos 13 links vira o nome do item em crase, no mesmo lugar do texto
2. `git commit` -> `.githooks/pre-commit` (exists) - um `passo` novo roda `cargo rustdoc --lib -- -D warnings`. Um aviso da documentação vira erro, e o `passo` reprova o commit com a saída do `rustdoc`
3. segunda-feira, 06:00 UTC -> `.github/workflows/semanal.yml` (exists) - um passo novo roda o mesmo comando, e o job reprova no GitHub
4. out: `README.md` §15, `CLAUDE.md` e os cabeçalhos do hook e do workflow citam a documentação entre o que os dois guarda-corpos cobram, sem contar os passos

## Impact

| Front | What changes |
| --- | --- |
| domain | nenhum termo muda. A documentação passa a ser exigível, como a formatação e o clippy já são |
| guarda-corpos | cada commit ganha um passo. Medido em 28/09/2026: o `cargo doc --no-deps` da lib levou 2,00 s depois de um `touch src/lib.rs`, e a suíte leva ~7 s |
| verificação da WPC-64 | o C8 de `.specs/features/wpc-64/checks.md` cobra exatamente `generated 13 warnings`, e passa a falhar quando isto chegar. O check não é editado: ele pertence a uma feature fechada, cuja verificação passou (`ee8e028`). Ele deixa de valer porque os 13 avisos que protegia viraram zero, e este plano é o registro disso |
| documentação que conta os passos | README §15 ("roda os três", "os mesmos três"), `CLAUDE.md:30` ("os mesmos três"), o cabeçalho e o `echo` do hook e o cabeçalho do `semanal.yml`. Cada um passa a citar a documentação sem contar os passos, como o `CLAUDE.md` pede ("Contagens envelhecem sozinhas") |
| stored data | nada a migrar |

## Relations

`None - no stored-data shape change`

## Surface

`None - nothing consumed outside`

## Landing

- None - nada aqui é caro de desfazer. A trava copia o que o clippy já faz nos dois guarda-corpos, e desfazê-la é apagar um passo em cada arquivo. Não há dependência nova, dado persistido nem configuração nova. Trocar um link por texto em crase se desfaz com um `git revert`
- **A forma do comando é `cargo rustdoc --lib -- -D warnings`, e não `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`.** Os dois dão o mesmo resultado para a lib, e medi os dois: 13 erros, código 101. O `cargo rustdoc` escreve igual no `sh` do hook, no `pwsh` do runner e no `powershell` do `CLAUDE.md`, sem variável de ambiente. Tem a mesma forma do `cargo clippy --all-targets -- -D warnings` que já está nos dois lugares. O `--lib` é obrigatório: com argumentos depois do `--`, o `cargo rustdoc` exige um alvo só, e o pacote tem lib e bin
- **Documentar os itens privados foi rejeitado.** Medido: com `--document-private-items`, os 13 avisos continuam. Os 10 de item privado só trocam de nota ("this link resolves only because you passed `--document-private-items`"), e os 3 que não resolvem continuam sem resolver
- **Silenciar o lint foi rejeitado.** `#![allow(rustdoc::private_intra_doc_links)]` afrouxaria o linter, e a regra global proíbe isso

## Criteria

### S1: O `cargo doc` da lib sai sem aviso (P1)

A documentação para de apontar para páginas que o `cargo doc` não gera, sem perder o nome do que apontava.

**Acceptance Criteria**

1. WHEN `cargo rustdoc --lib -- -D warnings` roda na raiz THEN o comando SHALL sair com código 0, e a saída SHALL NOT ter nenhuma linha que comece com `warning:` ou `error:`
2. The documentação SHALL continuar nomeando em crase, sem link, o item que cada um dos 13 links apontava, no mesmo parágrafo. Os itens são:
   - `ler_o_firmware_antes` e `criar_a_entrada`, em `src/comandos/prepare.rs`
   - `confere_com_o_arcaboot` e `secao_da_ordem_de_boot`, em `src/comandos/status.rs`
   - `FirmwareDeMentira::fwbootmgr`, em `src/duplos.rs`
   - `campo`, em `src/estado.rs`
   - `Leitura::chamada`, duas vezes em `src/firmware.rs`. Uma delas era o link `chamada`, que não resolvia, e o nome ganha o tipo porque o método mora em `impl Leitura`
   - `crate::comandos::prepare::conferir_o_pacote_local`, em `src/pacote.rs`
   - `montar_sondagem`, `FLAGS_DE_SONDAGEM` e `TETO_DOS_PARAMETROS`, em `src/receita.rs`
   - `crate::receita::montar_sondagem`, em `src/sondagem.rs`
3. The mudança em `src/` SHALL tocar somente linhas de comentário de documentação (`///` e `//!`)

**Independent test:** rodar `cargo rustdoc --lib -- -D warnings` e ver o código 0.

### S2: Um link quebrado novo não entra (P1)

O hook reprova o commit que traz um link quebrado ou um link para item privado, e o semanal cobra o mesmo.

**Acceptance Criteria**

4. IF a árvore de trabalho traz um link de documentação pública para item privado THEN o `.githooks/pre-commit` SHALL sair com código 1, e a saída SHALL conter `links to private item` e `o commit NAO foi feito`
5. IF a árvore de trabalho traz um link de documentação que não resolve THEN o `.githooks/pre-commit` SHALL sair com código 1, e a saída SHALL conter `unresolved link` e `o commit NAO foi feito`
6. WHEN o `.githooks/pre-commit` roda sobre uma árvore sem aviso de documentação THEN ele SHALL sair com código 0 e imprimir `pre-commit: ok`
7. The `.github/workflows/semanal.yml` SHALL ter um passo cujo `run` é `cargo rustdoc --lib -- -D warnings`, antes do passo `Suíte`

**Independent test:** reintroduzir na árvore de trabalho um dos 13 links, rodar `sh .githooks/pre-commit`, ver o código 1 e depois desfazer.

### S3: A documentação diz o que os guarda-corpos cobram (P2)

Quem lê o README, o `CLAUDE.md` ou os próprios arquivos sabe que a documentação é cobrada, e nenhum desses textos conta passos.

**Acceptance Criteria**

8. The README §15 ("Os dois guarda-corpos, e o que cada um alcança") e o `CLAUDE.md` SHALL citar `cargo rustdoc --lib -- -D warnings` entre o que o hook e o semanal rodam, e SHALL NOT conter `os três` nem `os mesmos três`
9. The cabeçalho e o `echo` inicial do `.githooks/pre-commit` e o comentário de cabeçalho do `semanal.yml` SHALL citar a documentação entre o que rodam

**Independent test:** ler o README §15 e a seção de comandos do `CLAUDE.md`.

## Out of scope

| Excluded | Why |
| --- | --- |
| Mudar a cadência do `semanal.yml` para mensal | perguntado na conversa de 28/09/2026. O repositório é público e o runner padrão é gratuito em repositório público, então a cadência não gasta o limite do plano Free. A recomendação foi manter semanal. Se mudar, é issue próprio |
| Documentar os itens privados no `cargo doc` padrão | é outra decisão, sobre o que a documentação publicada mostra. Também não tiraria os avisos (medido, ver Landing) |

## Assumptions

| Assumption | Chosen default | Rationale | Confirmed? |
| --- | --- | --- | --- |
| Onde o passo novo entra no hook | depois do clippy e antes da suíte | leva ~2 s e reprova antes dos ~7 s da suíte, e o `passo` para no primeiro que reprova. Apresentado ao usuário ("entre o clippy e a suíte"), que respondeu "siga com o plano" em 28/09/2026 | y |
| Onde o passo novo entra no `semanal.yml` | depois do clippy e antes da `Suíte`, com o nome no mesmo tom dos vizinhos ("Documentação, com aviso valendo erro") | a mesma ordem do hook, para os dois se lerem igual. Apresentado ao usuário ("na mesma posição"), que respondeu "siga com o plano" em 28/09/2026 | y |

**Open questions:** none - all resolved or logged above.

## Observable

| Surface | Decision | Landing |
| --- | --- | --- |
| hook `.githooks/pre-commit` | output format and verbosity | existing - o `passo` roda calado e só mostra a saída quando reprova. O passo novo herda isso |
| hook `.githooks/pre-commit` | exit codes | AC 4, AC 5, AC 6 |
| hook `.githooks/pre-commit` | what it prints when it fails halfway | AC 4, AC 5 - a saída do `rustdoc`, e a linha `o commit NAO foi feito` que o `passo` já imprime |
| hook `.githooks/pre-commit` | flags and defaults | n/a - o hook não recebe flag. O `--no-verify` é do `git`, e continua valendo |
| workflow `semanal.yml` | output and exit codes | existing - um passo que reprova derruba o job no GitHub. O comando é o mesmo do AC 1 |
| workflow `semanal.yml` | flags and defaults | n/a - o gatilho (`schedule`, `workflow_dispatch`) não muda |
| documento README §15 / `CLAUDE.md` | structure, and what the reader does next | AC 8 - o comando aparece na mesma frase que já lista o `fmt` e o clippy, e quem lê roda o mesmo comando para reproduzir o hook |

## Sources

- [WPC-65](https://linear.app/wpcsolutions/issue/WPC-65/o-link-de-naoestala-para-conferir-o-pacote-local-nao-resolve-no-cargo) - o defeito e a medição de 12 para 13 avisos
- a decisão do usuário em 28/09/2026, registrada literalmente no comentário da WPC-65: "Os 13, trava no hook e no semanal (Recomendado)"
- [Cargo, `cargo rustdoc`](https://doc.rust-lang.org/cargo/commands/cargo-rustdoc.html) - os argumentos depois do `--` vão para a invocação final do `rustdoc`, e com mais de um alvo o `--lib` é obrigatório
