# WPC-220 · A fonte lenta perde a vez, o download mostra velocidade e quanto falta, e o `--iso` fica guardado - verificação

**Veredito**: PASS
**Perfil**: light
**Intervalo do diff**: 6510940..cef883e (a correção da rodada 2); a feature inteira é c0b4824..cef883e
**Rodada**: 3 - scoped
**Verificador**: subagente independente, que não escreveu a feature nem fez as rodadas 1 e 2 (autor != verificador)

Todas as provas saem com 0 na `cef883e`. As 46 provas distintas de `cargo test --lib` (49 linhas `Proof:`, três reusadas) rodaram numa invocação só, com `--exact`, e cada nome aparece como `test <nome> ... ok` (`46 passed; 873 filtered out`). O `b10` sai com 0, e as oito provas em shell de C28-C32, extraídas literalmente do `checks.md`, também. O gate está verde: 1036 testes passaram e 0 falharam, e o fmt, o clippy e o rustdoc estão limpos.

O veredito é PASS. **A lacuna que derrubou a rodada 2 fechou.** O teste de C18 agora tem a asserção que o checklist exige: `*bancada.sistema.listados.borrow() == vec![iso, parcial]` (`prepare.rs:3362-3366`). É a única leitura de `listados` na árvore. A lacuna 2 da rodada 2 fechou pelo caso (8) de C22. As lacunas 3 e 4 fecharam no README, no ADR-0029 e no comentário de C23. A mudança de produção (`motivo_da_medida`, `prepare.rs:1253-1261`) só muda o texto do `PacoteIncompleto`, e só esse caso é observável. C21 e C26 passam pela mesma função e continuam provados.

Como verificado: **32 de 32 checks provados**. Restam duas observações, e nenhuma muda o veredito (ver "Lacunas que restam").

## O que o perfil `light` roda

- **Passo 1, fontes vinculantes:** é só do perfil `ui`, e não rodou como passo. A redação nova de C22 (o caso 8) foi comparada com os critérios 22 e 23 do issue e com a tabela `--iso` do desenho. O resultado está em "Fontes vinculantes".
- **Recálculo do `Coverage`, linhas de `Test policy`, injeção de falhas:** são de `standard` e `ui`, e não rodaram. O checklist não tem `Test policy`. Sem a injeção, o poder de discriminar das asserções novas foi julgado pela aritmética de cada uma (ver C18 e C22).
- **Passos 2, 3 e 5:** rodaram. O passo 2 rodou sobre as 32 provas, no HEAD novo. O passo 3 julgou de novo C18, que era o único veredito diferente de PASS na rodada 2, e cada check que o diff tocou: C22 e C23 (testes mudados); C21 e C26 (passam pela função de produção mudada); C27 (a varredura de `src/`, que mudou); C28, C30 e C31 (leem o ADR-0029 e o README, que mudaram). Os outros carregam o veredito da rodada 2, com as citações de `prepare.rs` e `duplos.rs` atualizadas, porque a correção moveu as linhas.

## Estado da árvore

- No começo, `git status --porcelain` trazia as cinco linhas `M` que não são desta feature (`.agents/.skill-lock.json` e quatro `.claude/skills/*/.skill-meta.json`). No fim, traz as mesmas cinco mais ` M .specs/features/wpc-220/verification.md`, que é este relatório.
- `git diff --stat 6510940..HEAD` toca `checks.md`, `verification.md`, `README.md` (+1 linha), `docs/adr/0029-...md` (1 linha), `src/comandos/prepare.rs` (+44 -2) e `src/duplos.rs` (+24). `git diff --stat 6510940..HEAD -- .design PRD CONTEXT.md tests recursos examples .specs/features/wpc-134 docs/adr/0028-...md` sai vazio. `src/andamento.rs`, `src/pacote.rs` e `src/adaptadores/` não mudaram.
- Os deslocamentos em `prepare.rs` são +4 depois da linha 1253, +10 depois da 3356, +37 depois da 3633 e +38 depois da 3727. Foram conferidos contra 52 âncoras: cada citação da rodada 2, somada ao deslocamento, cai na asserção que a rodada 2 citava. Em `duplos.rs`, o resumo por caminho foi de `:1185-1191` para `:1210-1225`.

## Fontes vinculantes

| Fonte | Aberta | Contradição | O que nenhum check cobre | Origem |
|---|---|---|---|---|
| Issue WPC-220 (Linear, `get_issue`, `updatedAt` 2026-10-23T14:38:26Z, o mesmo das rodadas 1 e 2; `list_comments` vazio) | sim | nenhuma | - | verified at cef883e |
| `.design/download-do-pacote.md` | sim. O último commit que o toca é `c0b4824`. A tabela `--iso` (`:142-151`) e a Key decision 6 (`:53`) foram relidas para o caso (8) | nenhuma | - | verified at cef883e |
| ADR-0028 e PRD §9.6 | sim, na rodada 1. O diff não os toca | nenhuma | - | carried from 6510940 |

**A redação nova de C22, contra o que ela transcreve:** o critério 22 enumera quatro causas da cópia que não se faz. C22 já trazia (5)-(7), e agora acrescenta (8): a cópia bate o SHA256, mas a listagem não tem os quatro caminhos. Nenhuma fonte decide esse caso. Mandá-lo para a linha de C22, sem zerar, não contradiz nenhuma delas:

- O critério 23 e a linha do desenho para "a cópia não bate" (`.design:149`) falam do SHA256, e o literal deles é `a copia nao bateu com o SHA256`. Esse texto seria falso para uma cópia que bateu.
- Zerar só na divergência é o que o caminho do download já faz com o mesmo `PacoteIncompleto` (`prepare.rs:1015-1027`: só `ResumoDivergente` zera).
- O motivo entre parênteses não é fixado pelo critério (`(<motivo>)`).

## Checks

Os arquivos são `prepare.rs` = `src/comandos/prepare.rs`, `andamento.rs` = `src/andamento.rs`, `sistema.rs` = `src/adaptadores/windows/sistema.rs`, `pacote.rs` = `src/pacote.rs` e `duplos.rs` = `src/duplos.rs`. "lib exact" é a invocação única `cargo test --lib -- --exact <46 nomes>`, rodada na `cef883e`, que saiu com 0 e mostrou os 46 nomes como `ok`. As citações de `andamento.rs`, `sistema.rs`, `pacote.rs` e `tests/` são as da rodada 2, porque esses arquivos não mudaram. As de `prepare.rs` foram atualizadas.

| Check | O que afirma | Prova rodada | Evidência | Resultado | Origem |
|---|---|---|---|---|---|
| C1 | linha `Fonte` e a barra vazia `medindo` | lib exact, exit 0 | `prepare.rs:4168-4175`: `escritas[..2] == ["  Fonte ... (oldstable)\n", "  Baixando Clonezilla ............. [░…░]   0% · 0,0 de 535,5 MB · medindo"]` | PASS | carried from 6510940 |
| C2 | `retomando de 120,0 MB`, 22%, sem zerar | lib exact, exit 0 | `prepare.rs:2744-2751`: `escritas[..2] == [".. · retomando de 120,0 MB\n", ".. [████░…]  22% · 120,0 de 535,5 MB · medindo"]`; `:2758-2765`: `escritos() == [pasta, parcial, guardado]` | PASS | carried from 6510940 |
| C3 | (a) amostras a cada ≤2 s; (b) uma escrita por amostra, no lugar; (c) os arredondamentos e a janela | lib exact, exit 0 (5 provas) | (a) `sistema.rs:808` `avisos.len() >= 3`, `:811-815`, `:816`, `:817-821`, `:822`, `:824` `last == Some(1000)`; (b) `prepare.rs:4205` `viva.len() == 5`, `:4206-4214` só a primeira sem `\r` e nenhum `\n`, `:4215-4218` `viva[3] == "\r  Baixando ... 20% · 110,0 de 535,5 MB · 5,0 MB/s · faltam 2 min"`; (c) `andamento.rs:314-329`, `:336-341`, `:355-356`, `:369` | PASS | carried from 6510940 |
| C4 | `faltam 2 h 19 min` | lib exact, exit 0 | `andamento.rs:376-379`: `com_o_rotulo(&linha_viva()) == "  Baixando ... 2% · 16,0 de 535,5 MB · 64,0 KB/s · faltam 2 h 19 min"` | PASS | carried from 6510940 |
| C5 | `0 B/s · parada ha 5 s` mais a sobra; aos 14 s, sem `parada` | lib exact, exit 0 (2 provas) | `prepare.rs:4240-4243`: `sem_a_sobra(&viva, 3) == "\r  Baixando ... 1% · 10,0 de 535,5 MB · 0 B/s · parada ha 5 s"`, e `:4152-4156` cobra `espacos == maior_antes - largura`; `andamento.rs:385-388`, `:392-396` | PASS | carried from 6510940 |
| C6 | 100% e 5,0 MB/s, sem `faltam`, depois o `\n`; 561.478.647 dá ` 99%` | lib exact, exit 0 (2 provas) | `prepare.rs:4267-4270` `ultima == "\r  Baixando ... 100% · 535,5 de 535,5 MB · 5,0 MB/s"`, `:4271` sem `faltam`, `:4282` `escritas[fonte + 1 + viva.len()] == "\n"`; `andamento.rs:320-324` | PASS | carried from 6510940 |
| C7 | ≤120 caracteres na varredura; só ASCII visível, espaço, `·`, `█`, `░`; cada um ocupa uma coluna no console desta mesa | lib exact, exit 0, nesta rodada. A prova manual é **carried from 6510940**: `src/andamento.rs` e `examples/` não têm hunk no diff | `andamento.rs:477-482` `maior.0 <= 120`; `:483-487` `fora_do_medido == Vec::<char>::new()`, com o filtro em `:446-450`; a varredura em `:420-440` e `:456-475`. Prova manual da rodada 2: `CP = 850`; `"[█░]": 4 caracteres, o cursor andou 4 colunas, desenhado igual = true`; a linha viva de 109 caracteres: `o cursor andou 109 colunas, desenhado igual = true` | PASS | carried from 6510940 |
| C8 | (a) o texto da lentidão; (b) `Break` aos 20 s, o mesmo `.parcial`, nada zera; (c) o adaptador encerra o `curl`; a borda dos 600 s | lib exact, exit 0 (4 provas) | (a) `andamento.rs:521` `velocidade() == 65_536`, `:522-525` `"lenta: faltariam 2 h 23 min a 64,0 KB/s"`; (b) `prepare.rs:4482` `encerrados == [oldstable]`, `:4483-4488` a linha `Fonte abandonada`, `:4491-4496` `"... (stable) · retomando de 1,2 MB\n"`, `:4501-4504` `destinos == [parcial, parcial]`, `:4507-4510` `escritos() == [pasta, parcial, parcial, guardado]`; (c) `sistema.rs:857`, `:858`, `:865-869`, `:870-874`; borda: `andamento.rs:496-497` `no_limite.lentidao() == None`, `:499-510` `matches!(.., Some(Faltariam { velocidade: 65_536, .. }))` | PASS | carried from 6510940 |
| C9 | nenhuma fonte é lenta antes dos 20 s | lib exact, exit 0 (2 provas) | `andamento.rs:539-540` `lentidao() == None` em (19,999 s, 0) e (19 s, 1); `prepare.rs:4528` `encerrados` vazio, `:4529` sem `Fonte abandonada`, `:4530-4533` `baixados == [FONTES[0]]` | PASS | carried from 6510940 |
| C10 | 567 s continua e diz `faltam 10 min`; 600 s exatos continuam | lib exact, exit 0 (3 provas) | `andamento.rs:554` `lentidao() == None`, `:555-559` termina em `" · 64,0 KB/s · faltam 10 min"`; `prepare.rs:4555-4560`; `andamento.rs:496-510` | PASS | carried from 6510940 |
| C11 | `lenta: nada veio nos ultimos 20 s` | lib exact, exit 0 (2 provas) | `andamento.rs:566` e `:574` `== Some(NadaVeio)`, `:567-570` o texto; `prepare.rs:4578-4583` a linha `\n  Fonte abandonada ... lenta: nada veio nos ultimos 20 s\n` | PASS | carried from 6510940 |
| C12 | a segunda volta vai da mais rápida para a mais lenta, sem `Break`; o empate segue `FONTES` | lib exact, exit 0 (2 provas) | `prepare.rs:4613-4616` `baixados == [old, stable, sf, sf]`, `:4617-4621` `encerrados == [old, stable, sf]`, `:4622-4627` `"... downloads.sourceforge.net · segunda volta · retomando de 3,8 MB\n"`; `:4653-4662` `[old, stable, sf, old, sf]` | PASS | carried from 6510940 |
| C13 | na segunda volta, parou, não respondeu e 404 dão a vez à seguinte | lib exact, exit 0 | `prepare.rs:4681-4689` `matches!(.., NenhumaFonteEntregou{..})`, `:4690-4700` os seis `baixados`, `:4702-4715` as três linhas `Fonte abandonada`, em ordem, depois de `segunda volta` | PASS | carried from 6510940 |
| C14 | quem saiu sem ser lenta não volta | lib exact, exit 0 | `prepare.rs:4733-4736` três entradas; `:4754-4757` `[old, stable, sf, old]` | PASS | carried from 6510940 |
| C15 | o texto de `c0b4824`, uma linha por fonte, com a resposta da segunda volta; a variante pelo `executar` | lib exact, exit 0 (3 provas) | `pacote.rs:836-845` o texto inteiro; `prepare.rs:4775-4781` `matches!(erro, Erro::PacoteRecusado(NenhumaFonteEntregou { .. }))`, `:4784-4793` `erro.to_string() == "nao deu para baixar ... nao respondeu em 30 s\n  ... HTTP 404\n  ... parou de mandar por 60 s, com 3,8 MB de 535,5 MB\n..."`, `:4794` `enumeracoes == 0`; outro cenário em `:2819-2825` | PASS | carried from 6510940 |
| C16 | as quatro linhas do `arca.log`, nesta ordem | lib exact, exit 0 (2 provas) | `prepare.rs:4843` `linhas.len() == 4`, `:4844-4849` `ends_with` contra `:4825-4842`; `:4397-4403` contra `:4383-4396` | PASS | carried from 6510940 |
| C17 | as `FONTES`, 30/60 s, `--connect-timeout 30`, no máximo duas tentativas | lib exact, exit 0 (4 provas) | `pacote.rs:687`; `pacote.rs:699-708`; `sistema.rs:680`; `prepare.rs:4869` `len == 6`, `:4871` `baixados[..3] == FONTES`, `:4872-4879` cada URL `count == 2` | PASS | carried from 6510940 |
| C18 | `DoIso`, as escritas, a cópia, `resumidos == [iso, .parcial]` **e `listados == [iso, .parcial]`**, o guardado, a tela | lib exact, exit 0 | `prepare.rs:3346` `DoIso(iso)`, `:3347` `enumeracoes == 0`, `:3348-3352` `escritos() == [pasta, parcial, parcial, guardado]`, `:3353-3356` `copiados == [(iso, parcial)]`, `:3357-3360` `resumidos == [iso, parcial]`, **`:3362-3366` `*bancada.sistema.listados.borrow() == vec![iso.clone(), parcial.clone()]`**, `:3367-3370` `conteudo_de(guardado) == Some("o zip, de mentira")`, `:3371` `.parcial` `None`, `:3372` o `--iso` não escrito, `:3373-3377` o conteúdo dele, `:3378` `baixados` vazio, `:3379-3386` a tela exata. O duplo anota cada chamada em `duplos.rs:1288`, e a asserção é a única leitura de `listados` na árvore (`rg -n listados src tests`). O valor esperado é literal. Uma conferência da cópia só pelo SHA256 deixaria `[iso]`, e uma que listasse o `--iso` duas vezes deixaria `[iso, iso]`. As duas falhariam aqui | PASS | verified at cef883e |
| C19 | o guardado que bate: nada escrito, a tela de antes, a linha do plano | lib exact, exit 0 | `prepare.rs:3258-3261`, `:3273-3281`, `:3282`, `:3283-3288`, `:3291-3296` | PASS | carried from 6510940 |
| C20 | o guardado que não bate só é substituído depois de a cópia passar | lib exact, exit 0 | `prepare.rs:3420-3423` `resumidos == [iso, guardado, parcial]`, `:3426-3429` `escritos == [pasta, parcial, parcial, guardado]`, `:3414-3418` o conteúdo final, `:3430-3436` a tela | PASS | carried from 6510940 |
| C21 | o guardado que não se deixa medir não é tocado | lib exact, exit 0 | Julgado de novo porque a linha dele sai de `motivo_da_medida` (`prepare.rs:1224`). `:3457-3458` o motivo do `certutil` é literal, `:3463-3466` nada escrito na pasta, `:3467-3470` `copiados` vazio, `:3471-3477` a tela com `({motivo})`; `:3498` `escritos` vazio, `:3499-3505` a tela com `(Acesso negado. (os error 5))`. Os dois casos passam pelos braços que a correção não mudou: `NaoDeuParaResumir` (`:1255-1257`) e `outro.motivo()` para o `Erro::Arquivo` (`:1259`). O braço novo (`:1258`) fica entre eles e não os alcança | PASS | verified at cef883e |
| C22 | (1)-(8) viram aviso; com o espaço exato a cópia acontece | lib exact, exit 0 | `prepare.rs:3520` `tela == linha_de(pasta, motivo)` (igualdade exata), `:3522-3527` `AVISO` com o motivo, `:3528-3531` `!foi_escrito(guardado)`; (1)(2) `:3535-3567`; (3) `:3574` `"precisa de 536 MB livres, e ha 535 MB"`, `:3575` nada copiado; no limite `:3578-3581`; (4) `:3591-3599`; (5) `:3608` com `CERTUTIL_SEM_O_ARQUIVO` (literal em `:3676-3678`), `:3609-3613` não zerado; (6) e (7) `:3618-3643`, com `:3637` e `:3638-3642` `arquivos.dentro.conteudo_de(guardado) == None`. **(8)**: o duplo lista a cópia sem `EFI/boot/bootx64.efi` (`:3650-3657`, por `listando_em`, `duplos.rs:1111` e `:1289-1292`), `:3659` `DoIso(iso)`, `:3660-3665` `cobrar(.., "o pacote foi extraido e nao tem o que faz um dispositivo bootar: falta EFI/boot/bootx64.efi. O \`bsdtar\` ter saido com codigo zero nao e prova de que o conteudo esta la")`, que é literal e é o texto de `pacote.rs:524-528`; `:3666-3670` `conteudo_de(parcial) == Some("o zip, de mentira")`, sem zero. Pela aritmética: sem o braço novo (`prepare.rs:1258`), o motivo sairia com o prefixo de `erro.rs:557`, e a igualdade de `:3520` falharia. Uma cópia incompleta renomeada falharia em `:3528-3531`, e uma cópia zerada falharia em `:3666-3670`. Se a chave de `listando_em` não casasse com o caminho, a cópia seria renomeada, e a tela diria `copiado do --iso` | PASS | verified at cef883e |
| C23 | a cópia que não bate termina zerada, e o zero só acontece nesse braço | lib exact, exit 0 (as duas provas) | `prepare.rs:3761` `DoIso`, `:3762` `conteudo_de(parcial) == Some("")`, `:3766` `resumidos == [iso, parcial]`, `:3767-3770` `escritos == [pasta, parcial, parcial]`, `:3771` guardado `None`, `:3772-3778` a tela exata. O braço que bate, por C18: `:3348-3352` escritas exatas, sem zero, e `:3367-3370` `guardado == "o zip, de mentira"`. O duplo resume pelo caminho, e não pelo conteúdo (`duplos.rs:1210`, `:1223-1225`). O comentário de `:3763-3765` não afirma mais a ordem | PASS | verified at cef883e |
| C24 | o passo 7, a extração e PR-3 saem do `--iso` | lib exact, exit 0 (2 provas) | `prepare.rs:3798-3802` `resumidos == [iso, parcial, iso]`, `:3803-3812` a extração de `iso`, `:3813-3821` a cópia de PR-3 de `iso`, `:3822` `baixados` vazio; `:2406-2414` | PASS | carried from 6510940 |
| C25 | as cinco recusas não escrevem na pasta | lib exact, exit 0 | `prepare.rs:3862-3865` `format!("{recusa:?}").starts_with(esperada)`, `:3866-3870` nada escrito na pasta; `:3886-3894` `NaoDeuParaOlhar`, `:3895` | PASS | carried from 6510940 |
| C26 | o ensaio não escreve, e diz o que o preparo faria | lib exact, exit 0 | Julgado de novo porque a linha de C21 no ensaio sai de `motivo_da_medida`. `prepare.rs:3923` `tela == copia_no_preparo`, `:3924-3928` nada escrito; `:3942` `tela == linha_do_iso()`; `:3954-3960` a linha de C21 inteira, com `({CERTUTIL_SEM_O_ARQUIVO}) · deixado como esta\n`; `:3969-3976` a linha de C22 exata, `:3978-3982` sem aviso | PASS | verified at cef883e |
| C27 | B-10, e a pasta fica com os três nomes | `cargo test --test b10_nada_e_apagado` exit 0 (2 ok: `a_porta_do_sistema_de_arquivos_nao_oferece_exclusao`, `nada_no_codigo_apaga_arquivo_ou_diretorio`); lib exact, exit 0 | `tests/b10_nada_e_apagado.rs:87` e `:105`, sobre o `src/` da `cef883e`, que inclui o `listando_em` e o braço novo; `prepare.rs:4012-4019` o conteúdo de `antigo` e `log`, `:4027-4031` `chaves == [log, antigo, guardado]`, `:4032-4038` as escritas só em `{pasta, parcial, guardado}` | PASS | verified at cef883e |
| C28 | o ADR-0029: seis decisões, as medições, `## O que fica do ADR-0028` | shell (`checks.md:179`), exit 0 | O diff mudou o ADR-0029 numa linha só, o item 4 de `## O que ficou em aberto` (`:139`). Ela vem depois de `### 6.` e é um item `4.`, e não um `### 4.`, por isso não altera a contagem nem os intervalos que a prova lê | PASS | verified at cef883e |
| C29 | o ADR-0028 aponta para o 0029, e só o PR-2 foi reescrito | shell (`checks.md:182`), exit 0 | `docs/adr/0028-...md` não mudou desde `6510940` | PASS | carried from 6510940 |
| C30 | o PR-7 e o PR-2 do PRD e do README | shell (`checks.md:185`, `:186`), exit 0 e 0 | O PRD não mudou. O README ganhou uma linha na tabela de exemplos (`README.md:2320`), que não toca as linhas **PR-7** e **PR-2** que a segunda prova lê | PASS | verified at cef883e |
| C31 | a segunda saída, a tabela do `C:` e as telas de exemplo | shell (`checks.md:189`, `:190`, `:191`), exit 0, 0 e 0 | idem: as três provas leem o README da `cef883e` | PASS | verified at cef883e |
| C32 | o **Pacote guardado** do `CONTEXT.md` | shell (`checks.md:194`), exit 0 | `CONTEXT.md` não mudou desde `6510940` | PASS | carried from 6510940 |

## Linha `existing` do Swept

carried from 6510940, com as linhas conferidas na `cef883e`:

- **authorization: existing.** Confirmada. O manifesto `requireAdministrator` está em `build.rs:92`, e a cópia vai para `contexto.registro.pasta()` (`prepare.rs:1125`).
- As linhas `existing` do `Coverage` rodaram no gate: `uma_fonte_que_para_e_abandonada_e_a_proxima_continua_do_mesmo_parcial` (`prepare.rs:4309`) e `um_parcial_que_o_curl_nao_consegue_escrever_recusa_na_primeira_fonte` (`prepare.rs:2972`). A borda 7 continua em `prepare.rs:2836-2840`.

## O resumo da rodada 2 no `checks.md`, contra o diff

Cada afirmação do bloco "Rodada 2 da verificação" (`checks.md:247-251`) foi conferida no código:

- "Ela entrou agora (`um_iso_que_passa_vira_o_pacote_guardado`)": **confere**, em `prepare.rs:3362-3366`.
- "C22 ganha o caso (8) [...] `listando_em`": **confere**. O caso está em `prepare.rs:3645-3670`, e o duplo em `duplos.rs:928`, `:1111-1120` e `:1289-1292`.
- "`motivo_da_medida` passou a usar o texto da própria recusa": **confere**, em `prepare.rs:1258`. Sem esse braço, o `PacoteIncompleto` cairia em `outro.motivo()`, e daí em `mensagem_do_pacote` (`erro.rs:550-559`), que põe na frente `o pacote do Clonezilla foi recusado: `.
- "A tabela de exemplos do README e o item 4 [...] do ADR-0029": **confere**, em `README.md:2320` e `docs/adr/0029-...md:139`. A tabela tem agora os oito arquivos de `examples/`.
- "O comentário do teste de C23 deixa de afirmar a ordem": **confere**, em `prepare.rs:3763-3765`.

## Lacunas que restam, por gravidade

Nenhuma muda o veredito.

1. **A declaração de `Unproven` da linha "a cópia que não se faz" perdeu metade sem prova.** Na rodada 2, a linha declarava sem prova "a listagem da cópia que falha ou não tem os quatro caminhos". Agora (`checks.md:217`) ela prova o segundo caso por C22(8) e diz `-`. O primeiro caso sumiu da tabela sem ganhar prova: o `listar_pacote` que devolve `Err` só sobre a cópia (o `bsdtar` não roda, `sistema.rs:356-364`). Ele cai em `prepare.rs:1194` e sai por `outro.motivo()` (`:1259`). O duplo não o alcança: `listagens` só guarda saídas `Ok` (`duplos.rs:928`, `:1290-1292`). O risco é desprezível. As duas linhas do caminho já rodam em outros testes: `:1194` em C22(8) e o braço `outro` em C21 (`:3499-3505`). Um `bsdtar` que não roda recusaria o `--iso` antes da cópia (`prepare.rs:642`). Isto fica registrado porque o recálculo do `Coverage` não roda no perfil `light`, e a tabela deixou de dizer o que o autor tinha declarado.
2. **Os dois comentários novos citam IDs do checklist sem dizer de onde.** `prepare.rs:3649` ("a divergência de C23") e `:3764` ("é C18 quem mostra") são os únicos IDs do checklist em todo `src/` e `tests/`. O resto do código diz "o critério N da WPC-220" (`prepare.rs:2739`, `:3226-3230`, `:4185`). O `CLAUDE.md` reserva `C-*` para os requisitos comuns do PRD, que hoje vão até `C-16`. Quem procurar `C-18` ou `C-23` no PRD não acha nada, e um `C-18` futuro diria outra coisa. É uma questão de padrão de comentário.

## O que aconteceu com cada lacuna das rodadas anteriores

**Rodada 2:**

1. **C18, `listados` sem asserção**: **fechou**. A asserção está em `prepare.rs:3362-3366`, com o valor esperado literal, e `rg -n listados src tests` acha agora um leitor fora de `duplos.rs`.
2. **O braço da cópia que falha na listagem**: **fechou na parte que tem código próprio**. O `PacoteIncompleto` da cópia é C22(8), em `prepare.rs:3645-3670`. O `Err` do `bsdtar` só sobre a cópia continua sem teste, e saiu da declaração (lacuna 1 acima).
3. **Os dois textos desatualizados**: **fechou**. (a) `README.md:2320` lista `barra_no_console`, e a tabela tem os oito exemplos de `examples/`. (b) `docs/adr/0029-...md:139` diz o que foi medido e mantém a pergunta com o usuário. `rg -i "ninguém mediu"` não acha mais nenhuma frase sobre o `█` e o `░`.
4. **O comentário do teste de C23**: **fechou**. `prepare.rs:3763-3765` não afirma mais a ordem e aponta para C18. Resta o padrão do ID (lacuna 2 acima).

**Rodada 1** (o estado de cada uma na `cef883e`):

1. **C18, os quatro caminhos da cópia**: **fechou nesta rodada**. A rodada 2 a deu como aberta, e ela é a lacuna 1 da rodada 2 acima.
2. **C7, colunas contra caracteres**: **fechada desde a rodada 2**. `andamento.rs:483-487` roda verde nesta rodada, e a medição manual vem da rodada 2.
3. **A borda dos 600 s**: **fechada desde a rodada 2**. `andamento.rs:490-511` roda verde nesta rodada.
4. **C23, a ordem zerar-depois-de-conferir**: **fechada desde a rodada 2**. O comentário que restava fechou nesta rodada.
5. **C26, a linha de C21 pelo prefixo**: **fechada desde a rodada 2**. A asserção exata está em `prepare.rs:3954-3960`.
6. **Valores esperados que o código de produção calcula**: **fechada desde a rodada 2**, e o diff não abriu nenhuma. O motivo novo de C22(8) é literal (`prepare.rs:3663-3664`), e a asserção de C18 compara com `vec![iso.clone(), parcial.clone()]`.
7. **C15, a variante pelo `executar`**: **fechada desde a rodada 2**, em `prepare.rs:4775-4781`.
8. **Braços do `--iso` sem teste**: **fechou nesta rodada, salvo o resíduo da lacuna 1 acima**. O espaço que não se mede e a renomeação recusada estão em `prepare.rs:3618-3643`, e a listagem incompleta da cópia está em `:3645-3670`.

## Gate

Rodado na `cef883e`:

- `cargo test`: 1036 passaram, 0 falharam, em 17 binários: a lib 919; as integrações `b10_nada_e_apagado` 2, `e10` 32, `e11` 9, `e12` 18, `e1` 5, `e2` 6, `e4` 6, `e7` 9, `e8` 6, `e9` 9, `o_build_rs_le_a_copia_ao_rodar` 2, `o_carimbo_ve_o_fonte_editado` 2, `repasse_de_argumentos` 5, `s1_nenhum_acesso_raw` 2 e `s6_o_tempo_nao_decide` 4; e os doc-tests 0. Exit 0.
- `cargo test --lib -- --exact <46 nomes do checklist>`: 46 passaram, 873 filtrados. Os 46 nomes apareceram um a um como `test <nome> ... ok`, e nenhum faltou (conferido nome a nome contra a lista extraída do `checks.md`). Exit 0.
- `cargo test --test b10_nada_e_apagado`: 2 passaram. Exit 0.
- Provas em shell de C28-C32, no Git Bash, cada uma extraída literalmente da sua linha `Proof:`: C28 (`:179`) 0 · C29 (`:182`) 0 · C30a (`:185`) 0 · C30b (`:186`) 0 · C31a (`:189`) 0 · C31b (`:190`) 0 · C31c (`:191`) 0 · C32 (`:194`) 0.
- Prova manual de C7: carried from 6510940. O diff não toca `src/andamento.rs` nem `examples/barra_no_console.rs`.
- `cargo fmt --all --check`: exit 0. `cargo clippy --all-targets -- -D warnings`: exit 0. `cargo rustdoc --lib -- -D warnings`: exit 0.
