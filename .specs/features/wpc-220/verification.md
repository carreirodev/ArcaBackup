# WPC-220 · A fonte lenta perde a vez, o download mostra velocidade e quanto falta, e o `--iso` fica guardado - verificação

**Veredito**: FAIL
**Perfil**: light
**Intervalo do diff**: c0b4824..e613d86
**Rodada**: 1 - full
**Verificador**: subagente independente (autor != verificador)

Todas as provas saem com 0 na `e613d86`. As 45 provas de `cargo test --lib` rodaram numa invocação só, com `--exact`, e cada uma aparece como `ok` (`45 passed; 873 filtered out`). O `b10` saiu com 0, e as oito provas em shell de C28-C32 também. O gate está verde: 1035 testes passaram e 0 falharam, e o fmt, o clippy e o rustdoc estão limpos.

O veredito é FAIL porque três checks prometem mais do que as provas deles asseguram, embora nenhuma esteja vermelha:

- **C18 deixa de fora metade do critério 18.** O issue diz que o `.parcial` copiado é conferido pelo SHA256 compilado **e pelos quatro caminhos obrigatórios**. O check só transcreve o SHA256, e nenhum teste confere a listagem da cópia. O duplo nem consegue observar isso, porque `listar_pacote` ignora o caminho.
- **C7 afirma uma largura em colunas que ninguém mediu.** O teste conta caracteres. A cláusula de que `█` e `░` ocupam uma coluna cada não tem asserção, e o próprio teste diz que ela é o Unresolved 4.
- **C23 afirma uma ordem que as asserções não enxergam.** O teste não consegue mostrar que a escrita que zera o `.parcial` vem depois da conferência.

Como verificado: **30 de 32 checks provados por inteiro**. C7 e C23 estão provados em parte. C18 está provado como foi escrito, mas o texto dele diverge do critério que transcreve.

## O que o perfil `light` roda

- **Passo 1, fontes vinculantes:** é só do perfil `ui`, e não rodou como passo. A comparação estreita entre os checks e os literais do issue e do desenho foi feita assim mesmo, como o brief pede. Está em "Fontes vinculantes".
- **Recálculo do `Coverage`, linhas de `Test policy`, injeção de falhas:** são de `standard` e `ui`. Não rodaram, e o checklist não tem a seção `Test policy`. Vários achados abaixo são lacunas de amostragem que uma injeção de falhas teria matado ou deixado vivas.
- **Passos 2, 3 e 5:** rodaram por inteiro, sobre os 32 checks.
- **O `light` é fino demais para esta feature.** Ela tem uma porta de mão única num contrato de porta e uma máquina de estados de duas voltas, e o limite de 600 s (`>` contra `>=`) é exatamente o mutante que o `standard` mataria e que o `light` não procura.

## Estado da árvore

- `git status --porcelain` no começo trazia as cinco linhas `M` que não são desta feature (`.agents/.skill-lock.json` e quatro `.claude/skills/*/.skill-meta.json`). No fim, traz as mesmas cinco mais `?? .specs/features/wpc-220/verification.md`, que é este relatório.
- `git diff --stat c0b4824..HEAD -- tests recursos .specs/features/wpc-134` sai vazio: as capturas, as integrações e o histórico da WPC-134 estão intocados, como o Impact manda.

## Fontes vinculantes

| Fonte | Aberta | Contradição | O que nenhum check cobre |
|---|---|---|---|
| Issue WPC-220 (Linear, `get_issue`, atualizado em 2026-10-23T14:38Z; nenhum comentário) | sim | **C18 contra o critério 18**: o critério diz "o `.parcial` é conferido (o SHA256 compilado e os quatro caminhos obrigatórios)", e o check só tem `resumidos == [<iso>, .parcial]`. **C7 contra o critério 7**: o critério diz "120 colunas", e a prova conta `chars()`. | os quatro caminhos da cópia do `--iso` (ver o achado 1) |
| `.design/download-do-pacote.md` (`c0b4824`, igual na HEAD) | sim | nenhuma. As linhas literais de Andamento (7 estados), de Fontes do pacote (7) e do `--iso` (8) batem com C1-C6, C8, C11-C13 e C18-C26. As linhas do `--iso` aparecem no desenho com `..` no lugar dos pontos, e o issue e os checks usam a coluna 33. | - |
| ADR-0028 e PRD §9.6 | sim | nenhuma. O PR-7 e o PR-2 do PRD (`PRD-ARCA-v5_1.md:1679` e `:1684`) trazem o texto do Decided palavra por palavra, mais as duas anotações | - |

Uma nota que não muda o veredito: o issue conta "3 portas de mão única" (as três linhas do Decided), e o checklist conta "1 one-way door". As duas linhas de texto do Decided, PR-7 e PR-2, estão provadas por C30.

## Checks

Os arquivos são `prepare.rs` = `src/comandos/prepare.rs`, `andamento.rs` = `src/andamento.rs`, `sistema.rs` = `src/adaptadores/windows/sistema.rs` e `pacote.rs` = `src/pacote.rs`. "lib exact" é a invocação única `cargo test --lib -- --exact <45 nomes>`, que saiu com 0.

| Check | O que afirma | Prova rodada | Evidência | Resultado |
|---|---|---|---|---|
| C1 | linha `Fonte` e a barra vazia `medindo` | lib exact, exit 0 | `prepare.rs:4036-4043`: `assert_eq!(escritas[..2], ["  Fonte ... (oldstable)\n", "  Baixando Clonezilla ............. [░…░]   0% · 0,0 de 535,5 MB · medindo"])` | PASS |
| C2 | `retomando de 120,0 MB`, 22%, sem zerar | lib exact, exit 0 | `prepare.rs:2740-2747`: `escritas[..2] == [".. · retomando de 120,0 MB\n", ".. [████░…]  22% · 120,0 de 535,5 MB · medindo"]`; `:2754-2761`: `escritos() == [pasta, parcial, guardado]` | PASS |
| C3 | (a) o adaptador manda amostras a cada ≤2 s; (b) uma escrita por amostra, no lugar; (c) os arredondamentos e a janela | lib exact, exit 0 (o adaptador passou também no gate) | (a) `sistema.rs:808` `avisos.len() >= 3`, `:811-815` `depois - antes <= 2 s`, `:816` `b.decorrido > a.decorrido`, `:817-821` `b.decorrido - a.decorrido <= 2 s`, `:822` `b.recebido >= a.recebido`, `:824` `last == Some(1000)`; (b) `prepare.rs:4073` `viva.len() == 5`, `:4074-4082` só a primeira sem `\r`, e nenhum `\n`, `:4083-4086` `viva[3] == "\r  Baixando ... 20% · 110,0 de 535,5 MB · 5,0 MB/s · faltam 2 min"`; (c) `andamento.rs:314-329` (0/4/19/20 casas, `  0%`/` 22%`/` 99%`/`100%`), `:336-341` (os seis valores de quanto falta), `:355-356` `1,0 MB/s` com menos de 20 s, `:369` `referencia() == amostra(10.5, 10_500)` | PASS |
| C4 | `faltam 2 h 19 min` | lib exact, exit 0 | `andamento.rs:376-379`: `com_o_rotulo(&linha_viva()) == "  Baixando ... 2% · 16,0 de 535,5 MB · 64,0 KB/s · faltam 2 h 19 min"` | PASS |
| C5 | `0 B/s · parada ha 5 s` mais a sobra; aos 14 s, nada de `parada` | lib exact, exit 0 | `prepare.rs:4108-4111`: `sem_a_sobra(&viva, 3) == "\r  Baixando ... 1% · 10,0 de 535,5 MB · 0 B/s · parada ha 5 s"`, e `:4020-4024` cobra `espacos == maior_antes - largura`; `andamento.rs:385-388` o literal; `:392-396` `!contains("parada")` aos 14 s | PASS |
| C6 | 100% e 5,0 MB/s, sem `faltam`, e depois o `\n`; 561.478.647 dá ` 99%` | lib exact, exit 0 | `prepare.rs:4134-4138` `ultima == "\r  Baixando ... 100% · 535,5 de 535,5 MB · 5,0 MB/s"`, `:4139` sem `faltam`, `:4150` `escritas[fonte + 1 + viva.len()] == "\n"`; `andamento.rs:320-324` ` 99%` | PASS |
| C7 | ≤120 caracteres em toda a varredura, **e** cada glifo ocupa uma coluna | lib exact, exit 0 | `andamento.rs:470-475` `maior.0 <= 120`, com `largura = chars().count()` (`:444`); a varredura em `:420-440` e os estados em `:449-467`. **A cláusula dos glifos não tem asserção**, e `:414-419` diz que `█` e `░` não foram medidos (Unresolved 4) | PARCIAL (achado 2) |
| C8 | (a) o texto da lentidão; (b) `Break` aos 20 s, o mesmo `.parcial`, nada zera entre as fontes; (c) o adaptador encerra o `curl` | lib exact, exit 0 | (a) `andamento.rs:486` `velocidade() == 65_536`, `:487-490` `"lenta: faltariam 2 h 23 min a 64,0 KB/s"`; (b) `prepare.rs:4350` `encerrados == [oldstable]`, `:4351-4356` a linha `Fonte abandonada`, `:4359-4365` `(stable) · retomando de {tamanho(1_310_720)}`, `:4370-4373` `destinos == [parcial, parcial]`, `:4376-4379` `escritos() == [pasta, parcial, parcial, guardado]`; (c) `sistema.rs:857` `encerrada`, `:858` `!parou`, `:865-869` `!veio.is_empty() && veio.len() < 1000`, `:870-874` `veio == corpo()[..len]` | PASS |
| C9 | nenhuma fonte é lenta antes dos 20 s | lib exact, exit 0 | `andamento.rs:504-505` `lentidao() == None` em (19,999 s, 0) e (19 s, 1); `prepare.rs:4397` `encerrados` vazio, `:4398` sem `Fonte abandonada`, `:4399-4402` `baixados == [FONTES[0]]` | PASS |
| C10 | 567 s continua, e a tela diz `faltam 10 min` | lib exact, exit 0 | `andamento.rs:519` `lentidao() == None`, `:520-524` termina em `" · 64,0 KB/s · faltam 10 min"`; `prepare.rs:4424-4429` | PASS |
| C11 | `lenta: nada veio nos ultimos 20 s` | lib exact, exit 0 | `andamento.rs:531` e `:539` `== Some(NadaVeio)`, `:532-535` o texto; `prepare.rs:4447-4452` a linha `\n  Fonte abandonada ... lenta: nada veio nos ultimos 20 s\n` | PASS |
| C12 | a segunda volta vai da mais rápida para a mais lenta, sem `Break`; o empate segue `FONTES` | lib exact, exit 0 | `prepare.rs:4482-4485` `baixados == [old, stable, sf, sf]`, `:4486-4490` `encerrados == [old, stable, sf]` (só a primeira volta), `:4491-4497` a linha `· segunda volta · retomando de`; `:4523-4532` `[old, stable, sf, old, sf]` | PASS |
| C13 | na segunda volta, parou, não respondeu e 404 dão a vez à seguinte | lib exact, exit 0 | `prepare.rs:4551-4559` `matches!(.., NenhumaFonteEntregou{..})`, `:4560-4570` os seis `baixados`, `:4572-4588` as três linhas `Fonte abandonada`, em ordem, depois de `segunda volta` | PASS |
| C14 | quem saiu sem ser lenta não volta | lib exact, exit 0 | `prepare.rs:4606-4609` três entradas; `:4627-4630` `[old, stable, sf, old]` | PASS |
| C15 | o texto de `c0b4824`, uma linha por fonte, com a resposta da segunda volta | lib exact, exit 0 | `pacote.rs:836-845` (teste intocado desde `c0b4824`) o texto inteiro; `prepare.rs:4650-4662` `erro.to_string() == "nao deu para baixar ... nao respondeu em 30 s\n  ... HTTP 404\n  ... parou de mandar ..."` e `:4663` `enumeracoes == 0`; a variante via `executar` em `:2815-2821` | PASS (achado 7) |
| C16 | as quatro linhas do `arca.log`, nesta ordem | lib exact, exit 0 | `prepare.rs:4712` `linhas.len() == 4`, `:4713-4718` `ends_with` contra `:4694-4711`; `:4265-4271`, cujas linhas esperadas não mudaram desde `c0b4824` | PASS |
| C17 | as `FONTES`, 30/60 s, `--connect-timeout 30`, no máximo duas tentativas | lib exact, exit 0 | `pacote.rs:687` `tem == esperadas`; `pacote.rs:699-708` `LIMITES == {30 s, 60 s}`; `sistema.rs:680` `depois("--connect-timeout") == Some("30")`; `prepare.rs:4738` `len == 6`, `:4740` `baixados[..3] == FONTES`, `:4741-4748` cada URL `count == 2` | PASS |
| C18 | `DoIso`, as escritas, a cópia, os resumos, a tela | lib exact, exit 0 | `prepare.rs:3342` `DoIso(iso)`, `:3343` `enumeracoes == 0`, `:3344-3348` `[pasta, parcial, parcial, guardado]`, `:3349-3352` `copiados == [(iso, parcial)]`, `:3353-3356` `resumidos == [iso, parcial]`, `:3357-3360`, `:3361`, `:3362-3367`, `:3369-3376` a tela exata | PASS como escrito; diverge do critério 18 (achado 1) |
| C19 | o guardado que bate: nada escrito, a tela de antes, a linha do plano | lib exact, exit 0 | `prepare.rs:3254-3257` `particionou()`, `:3269-3277` nada escrito sob a `pasta`, `:3278` `baixados` vazio, `:3279-3284` `tela == "  Pacote conferido ... (PR-2)\n"`, `:3287-3292` o plano contém `"  Clonezilla 3.3.3-15 · do arquivo local <iso> (PR-2)\n"` | PASS |
| C20 | o guardado que não bate só é substituído depois de a cópia passar | lib exact, exit 0 | `prepare.rs:3410-3413` `resumidos == [iso, guardado, parcial]`, `:3416-3419` `escritos == [pasta, parcial, parcial, guardado]`, `:3404-3408` o conteúdo final (`"o zip, de mentira"`, ou `"o pacote que nao bate"` quando a cópia também não bate, `:3384`), `:3420-3426` a tela | PASS |
| C21 | o guardado que não se deixa medir não é tocado | lib exact, exit 0 | `prepare.rs:3460-3463` nada escrito na pasta, `:3464-3467` `copiados` vazio, `:3468-3474` a tela com `({motivo})`; `:3495` `escritos` vazio, `:3496-3502` a tela com `(Acesso negado. (os error 5))` | PASS (achado 6) |
| C22 | (1)-(5) viram aviso, e com o espaço exato a cópia acontece | lib exact, exit 0 | `prepare.rs:3517` `tela == linha_de(pasta, motivo)`, `:3519-3524` `AVISO` com o motivo, `:3560-3563` o guardado não escrito; `:3571` `"precisa de 536 MB livres, e ha 535 MB"`, `:3572` e `:3593-3596` nada copiado, `:3576-3578` com `TAMANHO_BYTES` livres, `"copiado do --iso para"`; `:3588-3592` a pasta; `:3611-3616` (5), não zerado | PASS (achado 6) |
| C23 | a cópia que não bate é zerada **depois** de conferida | lib exact, exit 0 | `prepare.rs:3632` `.parcial == ""`, `:3635` `resumidos == [iso, parcial]`, `:3636-3639` `escritos == [pasta, parcial, parcial]`, `:3640` sem guardado, `:3641-3647` a tela. **A ordem zerar-depois-de-conferir não tem asserção** | PARCIAL (achado 4) |
| C24 | o passo 7, a extração e PR-3 saem do `--iso` | lib exact, exit 0 | `prepare.rs:3667-3671` `resumidos == [iso, parcial, iso]`, `:3672-3681` a extração de `iso`, `:3683-3690` a cópia de PR-3 de `iso` e não do guardado, `:3691` `baixados` vazio; `:2402-2410` | PASS |
| C25 | as cinco recusas não escrevem na pasta | lib exact, exit 0 | `prepare.rs:3731-3734` `format!("{recusa:?}").starts_with(esperada)` para NaoEstaLa/EUmaPasta/ResumoDivergente/PacoteIncompleto, `:3735-3739` nada escrito na pasta; `:3755-3763` `NaoDeuParaOlhar`, `:3764` | PASS |
| C26 | o ensaio não escreve, e diz o que o preparo faria | lib exact, exit 0 | `prepare.rs:3792` `tela == copia_no_preparo` (ausente e não bate), `:3793-3797` nada escrito; `:3811` `tela == linha_do_iso()`; `:3823-3828` só o **prefixo** da linha de C21; `:3837-3844` a linha de C22 exata, `:3846-3850` sem aviso | PASS (achado 5) |
| C27 | B-10, e a pasta fica com os três nomes | `cargo test --test b10_nada_e_apagado` exit 0 (2 ok); lib exact, exit 0 | `tests/b10_nada_e_apagado.rs:87` e `:105`; `prepare.rs:3880-3887` o conteúdo de `antigo` e `log`, `:3895-3899` `chaves == [log, antigo, guardado]`, `:3900-3906` as escritas só em `{pasta, parcial, guardado}` | PASS |
| C28 | o ADR-0029: seis decisões, as medições, `## O que fica do ADR-0028` | shell, exit 0 | `docs/adr/0029-...md:37-91` `### 1.`-`### 6.`, `:24` as medições, `:151` | PASS |
| C29 | o ADR-0028 aponta para o 0029, e só o PR-2 foi reescrito | shell, exit 0 | o diff tem só a nota na decisão 6 e a linha de PR-2 trocada | PASS |
| C30 | o PR-7 e o PR-2 do PRD e do README | shell (2 provas), exit 0 e 0 | `PRD/PRD-ARCA-v5_1.md:1679` e `:1684` iguais ao Decided; `README.md:2148` e `:2153` | PASS |
| C31 | a segunda saída, a tabela do `C:` e as telas de exemplo | shell (3 provas), exit 0, 0 e 0 | `README.md` §"nao deu para baixar", item 2; `README.md:1801`; `README.md:630-631` e `PRD:1145-1146` | PASS |
| C32 | o **Pacote guardado** do `CONTEXT.md` | shell, exit 0 | o parágrafo "Dois caminhos o alimentam" | PASS |

## Linha `existing` do Swept

- **authorization: existing.** Confirmada. O manifesto `requireAdministrator` está em `build.rs:92`, e a cópia vai para `contexto.registro.pasta()`, a pasta do `arca.log` (`prepare.rs:1125`).
- As linhas `existing` do `Coverage` também existem e rodaram no gate: `uma_fonte_que_para_e_abandonada_e_a_proxima_continua_do_mesmo_parcial` (`prepare.rs:4177`) e `um_parcial_que_o_curl_nao_consegue_escrever_recusa_na_primeira_fonte` (`prepare.rs:2968`). A borda 7, em que `parou` na primeira volta não volta, está provada em `prepare.rs:2832-2836`.

## Lacunas, por gravidade

1. **C18 diverge do critério 18: os quatro caminhos da cópia não têm prova.** O critério diz que o `.parcial` copiado é conferido "o SHA256 compilado e os quatro caminhos obrigatórios", e o check só transcreve o SHA256. O código faz as duas coisas: `guardar_o_iso` chama `conferir_o_pacote(contexto, &parcial)` em `prepare.rs:1174`, que lista o pacote em `prepare.rs:642`. Mas nenhum teste o cobra, e o duplo não consegue: `SistemaDeMentira::listar_pacote` ignora o caminho (`src/duplos.rs:1261`). Uma refatoração que conferisse a cópia só pelo SHA256 passaria. O risco prático é baixo, porque uma cópia com o mesmo SHA256 tem os mesmos bytes do `--iso`, que já passou na listagem. Mesmo assim, o check deixa de fora uma frase do registro da decisão. Os braços que isso deixa sem teste são `PacoteIncompleto` e os outros erros da conferência da cópia que não são divergência, que caem em `nao_guardou` em `prepare.rs:1194`.
2. **C7 afirma colunas e prova caracteres.** O critério 7 diz "120 colunas". A prova mede `chars().count()` (`andamento.rs:444`, `:470-475`), e a cláusula do check de que "os quatro glifos ocupam uma coluna cada" não tem asserção. O próprio teste diz que isso não foi medido (`andamento.rs:414-419`, Unresolved 4), e o Out of scope do checklist diz o mesmo, o que contradiz o THEN de C7. `█` (U+2588), `░` (U+2591) e `·` (U+00B7) têm largura East Asian Ambiguous. É precisão do checklist: C7 deveria dizer "caracteres" e deixar a equivalência com colunas para o Unresolved 4.
3. **O limite de 600 s não foi amostrado** (C8 e C10). O julgamento é `faltam > LENTA_SE_FALTAREM_MAIS_DE` (`andamento.rs:168`), e as amostras ficam longe da borda: 567 s (`andamento.rs:518`, `prepare.rs:4418`) e cerca de 8.547 s (`andamento.rs:482`). Um mutante `>=`, ou um limite de 599 ou de 601, passaria pela suíte. Os 20 s (19,999 e 20,0) e os 5 s de `parada` (4 s e 5 s) estão amostrados nas duas bordas, e os 600 s não. Isso pede uma amostra com exatamente 600 s, por exemplo 39.321.600 bytes faltando a 65.536 B/s.
4. **C23: a ordem "zerado depois de conferido" não é observável.** As asserções `resumidos == [iso, parcial]` e `escritos == [pasta, parcial, parcial]` (`prepare.rs:3635-3639`) estão em dois registros separados, sem ordem entre eles, e o duplo devolve o mesmo resumo seja qual for o conteúdo. Um código que zerasse antes de conferir passaria. O comentário em `:3633-3634` afirma uma coisa que o teste não vê.
5. **C26: no ensaio, a linha de C21 é cobrada só pelo prefixo.** `prepare.rs:3823-3828` faz `contains("  Pacote guardado ................. nao deu para medir o que ja esta guardado (")`, e o motivo e o `· deixado como esta` ficam de fora. As outras três telas do ensaio são cobradas exatas.
6. **Valores esperados que o próprio código de produção calcula.** O motivo de C21 vem de `crate::resumo::do_certutil(...)` (`prepare.rs:3447-3455`), e o de C22(5) também (`:3605-3610`). O `retomando de` de C8, C12 e C13 e o `parou ... com` de C15 saem de `tamanho(...)` (`:4362`, `:4494`, `:4575`, `:4660`). Se o formatador mudar, a asserção acompanha. O handoff de S4 já tinha notado que o literal de `tamanho(1.310.720)` não está provado em lugar nenhum.
7. **C15: a variante pelo `executar`, no cenário de C13, está provada só pelo texto.** `prepare.rs:4650-4662` cobra o `Display`. A variante `NenhumaFonteEntregou` está provada no mesmo cenário pelo pré-voo (`:4551-4559`) e pelo `executar` noutro cenário (`:2815-2821`). A lacuna é pequena e fica registrada só por precisão.
8. **Braços do `--iso` sem teste.** O `Coverage` já declara um deles, a renomeação recusada (`prepare.rs:1196-1198`). Há mais dois que ele não nomeia: o `espaco_livre` que falha (`prepare.rs:1156`) e o achado 1. Os três caem no mesmo `nao_guardou`.

## Gate

- `cargo test`: 1035 passaram, 0 falharam, em 17 binários (lib 918; as integrações, inclusive `s1_nenhum_acesso_raw`, `s6_o_tempo_nao_decide` e `b10_nada_e_apagado`; e os doc-tests 0). Exit 0.
- `cargo test --lib -- --exact <45 nomes do checklist>`: 45 passaram, 873 filtrados. Os 45 nomes apareceram um a um como `test <nome> ... ok`, e nenhum faltou. Exit 0. Os dois testes do adaptador do `curl` com servidor local passaram nas duas execuções, no gate e aqui.
- `cargo test --test b10_nada_e_apagado`: 2 passaram. Exit 0.
- Provas em shell de C28-C32, no Git Bash: C28 0 · C29 0 · C30a 0 · C30b 0 · C31a 0 · C31b 0 · C31c 0 · C32 0.
- `cargo fmt --all --check`: exit 0. `cargo clippy --all-targets -- -D warnings`: exit 0. `cargo rustdoc --lib -- -D warnings`: exit 0.
