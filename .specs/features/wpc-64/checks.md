# WPC-64 · Uma pasta no `--iso` é recusada no pré-voo, apontando o pacote - checks

Profile: light (nenhum `AGENTS.md` declara outro)
Plan: `.tasks/wpc-64.md` - a tarefa, com os 7 critérios que viram C1-C7

Sources:

- `.tasks/wpc-64.md` - os critérios 1-7, o Out of scope, o Observable, o Swept, o Impact e o Unresolved. É o registro da decisão
- [WPC-64](https://linear.app/wpcsolutions/issue/WPC-64/uma-pasta-no-iso-ainda-chega-ao-certutil-e-a-recusa-naoestala-nao) - o defeito e a medição de 28/09/2026: `certutil -hashfile <pasta> SHA256` responde `0x80070002 (WIN32: 2 ERROR_FILE_NOT_FOUND)`, código `-2147024894`, o mesmo que para um arquivo ausente
- A decisão do usuário de 28/09/2026, transcrita no `Sources` da tarefa - **vinculante para o texto da recusa**: "Própria, olhando dentro"; os dois textos aprovados, com o `--` do quadro trocado pelo travessão das vizinhas em `src/pacote.rs`
- WPC-60 e o commit `4d2c773` - de onde vêm o pré-voo, `NaoEstaLa` e os testes que esta mudança estende
- `docs/adr/0024-o-prepare-oferece-a-lista-e-nao-deduz-o-disco.md` - por que o pacote de dentro da pasta é apontado e não usado
- [`std::path::Path`](https://doc.rust-lang.org/std/path/struct.Path.html) - `is_file` segue links simbólicos e responde `false` quando os metadados não se deixam ler

## Out of scope

- Usar sozinho o pacote achado dentro da pasta - o `prepare` oferece e não deduz (ADR-0024), e o `--iso` nomeia o arquivo (PR-2)
- Separar "não consegui olhar" de "não está lá" no pré-voo - é a WPC-68, que segue em Backlog e não é feita junto. A pergunta nova nasce `bool`
- `src/verificacao.rs:320` - é a WPC-67
- WPC-65 e WPC-66 - issues próprias. A documentação nova não liga para função privada (C8)
- **Unresolved #1 da tarefa, carregado aberto:** com um `--iso` relativo, o caminho sugerido vai na forma dada (`<caminho dado>\clonezilla-live-3.3.3-15-amd64.zip`), que é a resposta provisória da tarefa. Nenhum check depende disso: C1 e C2 usam caminho absoluto

## Landing

Toca a porta `Arquivos` e as três implementações dela (o adaptador e os dois duplos), a enum `RecusaDoPacote`, o pré-voo do `prepare` e o README. Reusa o pré-voo e a `Bancada` dos testes da WPC-60, e o `resumo_padrao` do `SistemaDeMentira`, que é a resposta medida do `certutil` para arquivo ausente e, desde 28/09/2026, também para pasta.

Os nomes que a tarefa deixou para a construção: a pergunta é `Arquivos::e_um_arquivo(&self, caminho: &Path) -> bool`, e a variante é `RecusaDoPacote::EUmaPasta { caminho: PathBuf, dentro: Option<PathBuf> }`. O `dentro` guarda o caminho sobre o qual o pré-voo perguntou, e não um `bool`, para que a mensagem aponte exatamente esse caminho, sem refazer a conta no `Display`.

- None - não há dado persistido, contrato consumido por fora nem dependência nova (o `Decided` da tarefa). A variante e o método se desfazem com uma refatoração, e os dois seguem padrões que já existem: uma variante de recusa por caso, como `NaoEstaLa`, e uma pergunta `bool` por caminho na porta, como `existe`. O método é obrigatório, sem implementação padrão: um padrão `self.existe(caminho)` deixaria um duplo esquecido respondendo "arquivo" para pasta, que é o defeito

## Checks

### S1 - Uma pasta no `--iso` é recusada no pré-voo, apontando o pacote · 6 files · 337 KB · ~84k

**C1** (critério 1) - WHEN `prepare::executar` roda com `--dispositivo 1` e `--iso "C:\Users\Ana Paula\Downloads"`, uma pasta sem `clonezilla-live-3.3.3-15-amd64.zip` dentro (só um `clonezilla-live-3.3.3-15-amd64.iso`) THEN devolve `Erro::PacoteRecusado(RecusaDoPacote::EUmaPasta { dentro: None, .. })`, e não `NaoEstaLa` nem `NaoDeuParaResumir`, e a mensagem contém `C:\Users\Ana Paula\Downloads`, `e uma pasta`, `clonezilla-live-3.3.3-15-amd64.zip`, `nao esta nesta pasta` e `Nada foi apagado`
Proof: `cargo test --lib comandos::prepare::testes::uma_pasta_no_iso_sem_o_pacote_dentro_recusa_no_pre_voo -- --exact`
Proof: `cargo test --lib adaptadores::arquivos_do_sistema::testes::uma_pasta_existe_e_nao_e_arquivo -- --exact` - num sistema de arquivos de verdade, `e_um_arquivo` responde `false` para uma pasta que `existe` e `true` para o arquivo dentro dela. Sem esta prova, todas as outras rodam sobre o duplo, e um adaptador que respondesse `exists()` passaria nelas

**C2** (critério 2) - WHEN o mesmo comando roda com `C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip` dentro da pasta, como arquivo, com o `certutil` ensinado a responder o SHA256 certo para ele e o console respondendo `s` e `JMicron Generic` THEN devolve `EUmaPasta { dentro: Some(..), .. }`, a mensagem contém `e uma pasta`, o caminho inteiro `C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip` e `Nada foi apagado`, e o comando termina na recusa: `console.lidas == 0`, `particionou()` falso e `extraidos` vazio
Proof: `cargo test --lib comandos::prepare::testes::uma_pasta_no_iso_com_o_pacote_dentro_aponta_o_arquivo_e_para -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_pasta_com_o_nome_do_pacote_dentro_nao_e_apontada -- --exact` - "como arquivo": uma **pasta** chamada `clonezilla-live-3.3.3-15-amd64.zip` dentro de Downloads não é o pacote, e a recusa sai com `dentro: None` e `nao esta nesta pasta`

**C3** (critério 3) - WHEN o `--iso` é uma pasta, sem e com o pacote dentro THEN o `certutil` não é chamado, nem sobre a pasta nem sobre o arquivo de dentro (`sistema.resumidos()` vazio), e a mensagem não contém `0x80070002`
Proof: `cargo test --lib -- comandos::prepare::testes::uma_pasta_no_iso_sem_o_pacote_dentro_recusa_no_pre_voo comandos::prepare::testes::uma_pasta_no_iso_com_o_pacote_dentro_aponta_o_arquivo_e_para --exact` - os dois casos, cada um com as duas asserções

**C4** (critério 4) - WHEN `executar` roda sem `--dispositivo`, com `--iso` numa pasta e o console respondendo `1`, `s` e `JMicron Generic` THEN devolve `EUmaPasta`, com `console.lidas == 0` e `particionou()` falso; e, no mesmo teste, a mesma bancada sem `--iso` lê ao menos uma linha, que é o menu do passo 0 (o controle)
Proof: `cargo test --lib comandos::prepare::testes::uma_pasta_no_iso_sem_dispositivo_recusa_antes_do_menu -- --exact`

**C5** (critério 5) - WHEN `obter_o_pacote`, que são os passos 6 e 7, depois do ponto sem volta, recebe como `--iso` uma pasta THEN a recusa é a do `certutil` (`RecusaDoPacote::NaoDeuParaResumir`), a mensagem não contém `Nada foi apagado`, `sistema.resumidos() == [pasta]` e o sistema de arquivos não foi consultado sobre a pasta (`arquivos.foi_consultado(pasta)` falso). A metade "nasce no pré-voo, antes do passo 0" é C4
Proof: `cargo test --lib comandos::prepare::testes::depois_do_ponto_sem_volta_uma_pasta_cai_no_certutil -- --exact`

**C6** (critério 6) - WHEN o `--iso` é um caminho ausente THEN a recusa continua `NaoEstaLa`; WHEN é o arquivo certo THEN ele continua indo ao `certutil` duas vezes, antes e depois do ponto sem volta (`sistema.resumidos()` traz o caminho duas vezes). As duas provas são testes da WPC-60, e esta mudança não altera o corpo deles
Proof: `cargo test --lib -- comandos::prepare::testes::um_iso_que_nao_esta_la_recusa_com_o_disco_intacto comandos::prepare::testes::o_pacote_local_bom_e_conferido_de_novo_depois_do_ponto_sem_volta --exact`

**C7** (critério 7) - A frase de "Instalar sem rede" que lista o que o pré-voo recusa inclui `uma pasta no lugar do arquivo`; e a seção de diagnóstico ganha, logo depois da entrada `o arquivo ... nao esta la, e e o caminho que o --iso deu`, a entrada `` ### `... e uma pasta, e o --iso nomeia o arquivo` ``, com a mensagem num bloco (`passe o caminho inteiro, entre aspas:` e `"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip"`), o que fazer (`passe ao `--iso` o caminho que ela dá`), a outra forma da mensagem (`nao esta nesta pasta`) e o aviso `**Nada foi apagado**`
Proof: `grep -F 'recusados no pré-voo' README.md | grep -qF 'uma pasta no lugar do arquivo'`
Proof: `grep '^### ' README.md | grep -A1 -F 'nao esta la, e e o caminho que o --iso deu' | tail -n1 | grep -qF '... e uma pasta, e o --iso nomeia o arquivo'`
Proof: `E=$(sed -n '/^### `\.\.\. e uma pasta, e o --iso nomeia o arquivo`$/,/^### `o disco N NAO/p' README.md); printf '%s\n' "$E" | grep -q '^```$' && printf '%s\n' "$E" | grep -qF 'passe o caminho inteiro, entre aspas:' && printf '%s\n' "$E" | grep -qF '"C:\Users\Ana Paula\Downloads\clonezilla-live-3.3.3-15-amd64.zip"' && printf '%s\n' "$E" | grep -qF 'passe ao `--iso` o caminho que ela dá' && printf '%s\n' "$E" | grep -qF 'nao esta nesta pasta' && printf '%s\n' "$E" | grep -qF '**Nada foi apagado**'`

**C8** (Out of scope da tarefa, a linha da WPC-65) - WHEN `cargo doc --no-deps` roda THEN a lib continua com os 13 avisos medidos antes da mudança, em 28/09/2026, e nenhum aviso cita `EUmaPasta` nem `e_um_arquivo`: a documentação nova não liga para `conferir_o_pacote_local` nem para outro item privado
Proof: `O=$(cargo doc --no-deps 2>&1); printf '%s\n' "$O" | grep -qF '(lib doc) generated 13 warnings' && ! printf '%s\n' "$O" | grep -qE 'EUmaPasta|e_um_arquivo'`

## Swept

- validation: C1, C2
- failure modes: existing - `Path::exists` e `Path::is_file` respondem `false` quando os metadados não se deixam ler, e o pré-voo trata isso como hoje, caindo em `NaoEstaLa`. A pergunta sobre o arquivo de dentro da pasta herda a mesma coerção. As duas são a WPC-68 (Out of scope)
- idempotency and retry: n/a - a recusa não escreve nada, e repetir com a mesma pasta dá a mesma recusa
- authorization: n/a - o processo já roda elevado (`requireAdministrator`), e a pergunta nova só lê os metadados do caminho que a pessoa deu
- concurrency and ordering: existing - um arquivo que muda entre o pré-voo e o passo 7, inclusive virando pasta, cai no `certutil` do passo 7 de propósito (`src/comandos/prepare.rs:561-564`). C5 prova a metade "virando pasta"
- data lifecycle: n/a - nada é criado, guardado nem apagado. `tests/b10_nada_e_apagado.rs:100-107` cobra os nomes dos métodos da porta, e `e_um_arquivo` não é verbo de exclusão
- external-dependency failure: C3
- state transitions: C4, C5
- observability: existing - `src/main.rs:137-138`: a recusa sai como `erro: <mensagem>` no stderr e como linha `ERRO` no `arca.log`

## Handoff

- S1 = ~84k: `wc -c` dos seis arquivos que a fatia toca (`src/comandos/prepare.rs` 128 KB, `README.md` 123 KB, `src/duplos.rs` 53 KB, `src/pacote.rs` 20 KB, `src/adaptadores/arquivos_do_sistema.rs` 10 KB, `src/portas/arquivos.rs` 4 KB) dividido por quatro. Abaixo dos 150k: um construtor só, sem handoff

- **Boundary:** C1-C8 fechados em `601c174`. A base da feature é `4d2c773`, e o checklist entrou sozinho em `c99bab9`
- **Settled mid-build:** nada foi perguntado ao usuário. As mensagens de commit seguem a prosa em português do repositório, com os trailers, e não Conventional Commits (o `CLAUDE.md` do projeto e a regra global de seguir a convenção do repositório)
- **Abandoned:** nada. Duas regressões foram injetadas e desfeitas antes do commit, para ver as provas caírem: a pergunta por pasta dentro de `conferir_o_pacote` (C5 caiu) e `existe` no lugar de `e_um_arquivo` na pergunta de dentro da pasta (só a segunda prova de C2 caiu). O adaptador começou respondendo `exists()`, o defeito original, e a segunda prova de C1 saiu vermelha antes de ele passar a `is_file()`
