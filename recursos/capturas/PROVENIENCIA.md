# De onde vieram estas capturas

Duas coisas do ARCA não podem ser testadas contra exemplo inventado: o parser
do `bcdedit`, único ponto onde uma leitura errada leva a máquina a bootar no
lugar errado com uma receita armada; e a **receita**, que é a string que o
Clonezilla executa quando não há mais ninguém olhando. Testar as duas contra o
que eu imaginei provaria que eu sei imaginar. Estes arquivos são o que o
hardware **escreveu e executou de verdade**.

O `.gitattributes` marca esta pasta como `-text` para que o git não normalize
nada — nem as quebras de linha CRLF do `bcdedit`, nem as LF dos `grub.cfg`.

## As leituras do `bcdedit` (etapa E2)

Convertidas de CP850 para UTF-8 na gravação, e só nisso.

| Arquivo | O que é |
|---|---|
| `bcdedit-enum-firmware-pt.txt` | `bcdedit /enum firmware` desta máquina, 22/08/2026, console em CP850 |
| `bcdedit-enum-firmware-en.txt` | **o mesmo BCD, no mesmo instante**, pelo mesmo `bcdedit`, com os recursos `en-US` ao lado |
| `bcdedit-enum-firmware-legado-pt.txt` | `E:\ARCA-LOGS\nvram-windows-antes.txt`, capturado em 20/08/2026, antes de a entrada ser renomeada |

## As receitas que rodaram em hardware (etapa E3)

Cópias byte a byte, sem conversão nenhuma. Cada uma é um `grub.cfg` como
estava no dispositivo no momento em que a máquina bootou nele e executou a
receita sozinha.

| Arquivo | O que é |
|---|---|
| `grub-backup-arca-teste-02.cfg` | `R:\boot\grub\grub.cfg.backup02` — o backup de `ARCA-TESTE-02`, 19/08/2026 |
| `grub-backup-arca-teste-03.cfg` | `E:\ARCA-LOGS\grub.cfg.original` — o backup de `ARCA-TESTE-03`, 20/08/2026 |
| `grub-restauracao-arca-teste-02.cfg` | `R:\boot\grub\grub.cfg.teste02` — a restauração de `ARCA-TESTE-02`, 19/08/2026 |
| `ocs-sr-help.txt` | `E:\ARCA-LOGS\ocs-sr-help.txt` — o `--help` do `ocs-sr` **desta versão** do Clonezilla |

## O estado inerte e a quarta cópia armada (etapa E4)

Cópias byte a byte, conferidas por SHA256 contra o dispositivo depois de
gravadas.

| Arquivo | O que é | SHA256 |
|---|---|---|
| `grub-inerte-arcaboot.cfg` | `R:\boot\grub\grub.cfg` — o **estado inerte** deste dispositivo, 11069 bytes | `4b33da61…f947aa3d` |
| `grub-clonezilla-original.cfg` | `R:\boot\grub\grub.cfg.original` — o que o **Clonezilla instalou**, 05/07/2026, 11058 bytes | `9ebfa1eb…068d331b` |
| `grub-backup-arca-teste-01.cfg` | `R:\boot\grub\grub.cfg.teste01` — uma **quarta** cópia armada, 19/08/2026, não usada na E3 | `cbbe6d5a…63c3f762` |

O `grub-inerte-arcaboot.cfg` é o alvo do desarmar, e o oráculo da etapa E4
inteira. Não é um arquivo montado por teste: é o que está no dispositivo agora.
`tests/e4_desarmar_o_dispositivo.rs` compara os dois a cada execução com o SSD
conectado — uma cópia que divergiu do que documenta deixou de ser evidência.

O `grub-clonezilla-original.cfg` responde de onde vem o estado inerte. Ele
difere do inerte em **uma linha**: traz `set default="0"` onde o inerte traz
`set default="live-default"`. Desarmar o dele produz o inerte byte a byte, e é
isso que torna a regra do [ADR-0005](../../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)
verificável em vez de arbitrária.

### O que estas três mostram e a E3 não tinha visto

- **Armar são duas mudanças, e uma delas não estava documentada.** O inerte e a
  `teste-03` diferem em exatamente duas coisas: o `set default` e o bloco de
  quatro linhas. **É o `set default` que faz o boot ser desatendido** — o
  `menuentry` sozinho só põe mais uma linha no menu. Ver §3.2 do PRD.
- **As quatro cópias armadas põem o bloco na mesma posição**, linhas 93–97,
  precedido de duas linhas em branco e seguido de uma. Um `diff` contra o
  inerte ancora umas depois da linha 91 e outras depois da 92, o que sugere
  duas formas de inserção — mas é artefato do algoritmo desambiguando linhas
  em branco repetidas. Os arquivos são iguais nessa região.
- **Só uma das quatro tem `set default="arca-backup"`**: a `teste-03`, que veio
  do `ARCAVAULT`. As três que estavam no `ARCABOOT` têm o bloco e
  `set default="live-default"` — o estado em que a máquina esperaria trinta
  segundos e bootaria no menu normal. Por quê é pergunta **fechada por falta de
  evidência**, com as três vias nomeadas no ADR-0005: datas não (S-6),
  `BootNext` não (o firmware o consome), dedução não (foi o que produziu os
  dois casos anteriores de fundação que não era).
- **Os blocos do ARCA não são iguais entre si.** A `teste-02` preserva o
  `hostname=cl-3.3.3-15` e as blacklists de driver do `menuentry` base; a
  `teste-03` perdeu os dois. Não há forma canônica transcrita — escolher qual a
  E7 vai inserir é decidir que linha de comando o kernel recebe, e é da E7.

### O que continua sem estar aqui

**Nenhum `bootsequence`.** Continua valendo o que a E2 registrou: não há job
armado nesta máquina, e armar é a E7.

A E4 é a **primeira etapa que escreve no firmware**, e escreve sem original
nenhum de onde transcrever. O `bcdedit /deletevalue {fwbootmgr} bootsequence` é
**código novo**, do mesmo jeito que o `arca-fim.txt` do ADR-0004 — marcado como
tal em `src/desarme.rs` e nos testes. O que dele foi medido em hardware, em
22/08/2026, é o comportamento **sem** `bootsequence`: código de saída 1, texto
"Elemento não encontrado", e nada muda. O caso com `bootsequence` presente está
coberto por caso construído no duplo, e a E7 o confirma.

A receita está numa linha só de cada arquivo: a `$linux_cmd` do `menuentry`
com `--id arca-backup`. `src/receita.rs` a extrai de lá nos testes, em vez de
repetir a string a mão — uma string repetida a mão prova que eu sei copiar; o
arquivo prova o que o hardware executou.

### O help se capturou sozinho

O `ocs-sr-help.txt` não foi digitado por ninguém. Ele saiu da própria receita
de `ARCA-TESTE-03`, que começa com
`ocs-sr --help > /home/partimag/ARCA-LOGS/ocs-sr-help.txt 2>&1`. A primeira
linha do arquivo é `/usr/sbin/ocs-sr: --help: invalid option` — o `ocs-sr`
desta versão não conhece `--help` e responde com o *usage* completo, que é o
que se queria. É o help **desta** versão, tirado **desta** execução, e é com
ele na mão que as decisões sobre `-scs`, `-p` e `-batch` foram tomadas.

### O que estas capturas mostram e o PRD não dizia

- **A receita nunca foi um script.** §10.1 e §10.2 do PRD mostravam um
  `#!/bin/bash` de várias linhas. O que rodou foi sempre uma string única
  dentro de `ocs_live_run="bash -c '...'"`. O ADR-0002 já havia decidido a
  forma; era o §10 que contradizia.
- **As três encadeiam com `;`, nunca com `if/then/else`.** A armadilha que
  R-5 descreve é real, mas a defesa contra ela é código novo — não há original
  de onde transcrevê-la.
- **Nenhuma das três escreve `arca-fim.txt`.** O `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\arca-fim.txt`
  que existe no dispositivo (`ARCA_RESTORE=OK` / `ARCA_FIM`) veio de trabalho
  manual de validação — o mesmo padrão que o ADR-0003 registrou para o
  `ARCA_VEREDITO=`. Todo o mecanismo de desfecho, do qual a E5 e a E8
  dependem, **nunca foi exercitado em hardware**.
- **As flags de backup não eram as de B-8.** Rodou
  `-q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true`: `-batch` no fim,
  `-p true` presente e não listado, `-scs` ausente.
- **A restauração usou `-e1 auto -e2`**, que R-4 não lista, e `-p poweroff`.

Ver `docs/adr/0004-a-receita-transcreve-o-que-rodou.md` para o que foi feito
com cada uma dessas divergências.

## Por que o par pt/en prova alguma coisa

O plano de implementação nomeia a fixture em inglês como metade do risco desta
etapa, e com razão: um parser afinado num só idioma passa em todo teste e
falha na máquina de outra pessoa.

As duas primeiras capturas descrevem **a mesma configuração de boot**, lida com
segundos de diferença. Não são uma tradução de outra: são duas leituras do
mesmo dado. Isso permite o teste que fecha o risco — o parser tem de extrair
delas exatamente o mesmo resultado, campo a campo. Qualquer dependência de
texto traduzido aparece como diferença.

O `bcdedit.exe` do Windows carrega suas mensagens de
`System32\<idioma>\bcdedit.exe.mui`. Esta máquina tem `pt-BR` e `en-US`
instalados. Copiando o `bcdedit.exe` para uma pasta onde só existe
`en-US\bcdedit.exe.mui`, o carregador de recursos usa o que está ali — e a
mesma consulta ao mesmo BCD sai em inglês.

## O que o par confirma

- **Só `identificador` é traduzido** entre os nomes de campo. `device`, `path`,
  `description`, `locale`, `inherit`, `displayorder`, `timeout` e os demais
  saem idênticos nos dois idiomas. É a fundação §3.1 do PRD, agora com as duas
  metades medidas.
- **Os títulos de bloco também são traduzidos** — `Windows Boot Manager` /
  `Gerenciador de Inicialização do Windows`. O PRD não diz isso, e é por isso
  que o parser não pode usá-los para decidir nada.
- **A entrada legada é reconhecível pela `description`**, que não é traduzida.

## A entrada desta máquina mudou de nome entre as capturas

A captura de 20/08 traz `description Clonezilla`; a de 22/08 traz
`description ARCA`. O identificador é o mesmo nas duas —
`{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}`.

Não é acidente de captura: é exatamente o que C-4 descreve, dos dois lados. A
captura antiga é a única evidência real do caso "não há entrada `ARCA`, há a
legada `Clonezilla`", e é por isso que ela está aqui em vez de ter sido
descartada por estar desatualizada.

## As capturas de NVRAM que ficaram no dispositivo (etapa E7)

Não estão neste diretório — estão em `E:\ARCA-LOGS\`, no dispositivo. Ficam
onde estão de propósito: são oito arquivos grandes de `efibootmgr -v`, e o que
importa deles cabe numa tabela. Ela está no **§3.1 do PRD**, e é lá que a
próxima pessoa deve procurar.

O resumo do que elas mostram, para quem chegar aqui primeiro:

| Quando | Arquivo | `BootOrder` | `BootCurrent` |
|---|---|---|---|
| 20/08 | `nvram-original.txt` | `0000,0001` | `0001` |
| 20/08 | `nvram-windows-antes.txt` (é `bcdedit`, não `efibootmgr`) | `{bootmgr}`, `{f4057bd0}`, +3 | — |
| 20/08 | `R1/nvram-antes.txt`, `R1/nvram-depois.txt` | `0000,0001` | `0001` |
| 20/08 | `R2/nvram-antes.txt`, `R2/nvram-depois.txt` — **restauração R2** | `0003,0000` | `0003` |
| **21/08 12:51** | `2026-08-21_WindowsCompleto/efi-nvram.dat` — **o backup** | **`0000,0001`** | `0001` |
| **21/08 14:28 e 14:46** | `ARCA-LOGS/2026-08-21_WindowsCompleto/nvram-antes.txt` e `-depois.txt` — **a restauração** | `0001,0000` | `0001` |
| 22/08 manhã | `bcdedit /enum {fwbootmgr}` desta máquina | `{bootmgr}` | — |
| 22/08 ~20:57 | **`nvram-live-2026-08-22.txt`** — está neste diretório | **`0000,0001`** | **`0001`** |
| 22/08 21:17 | `bcdedit-enum-firmware-2026-08-22-pos-marco.txt` | `{f4057bd0}`, `{bootmgr}`, `{687478f2}` | — |
| 23/08 manhã | `bcdedit-enum-firmware-2026-08-23-antes-da-restauracao.txt` | `{f4057bd0}`, `{687478f2}`, `{bootmgr}` | — |
| **23/08 12:12** | **`bcdedit-enum-firmware-2026-08-23-pos-restauracao.txt`** — depois da restauração | **`{bootmgr}`** — e a `{687478f2}` sumiu inteira | — |

> **As duas linhas de 21/08 eram uma só até a etapa E9, e a que estava aqui é
> da restauração.** Os `nvram-antes.txt` e `-depois.txt` moram em
> `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\` — ao lado do `arca-fim.txt` de
> `ARCA_RESTORE=OK` —, e o `mtime` deles é 14:28 e 14:46; o `savedisk` daquele
> dia terminou às 12:54. A NVRAM do boot do **backup** é o `efi-nvram.dat` de
> dentro da imagem, e ele diz `0000,0001`, com o Windows à frente. Ver
> [ADR-0011](../../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md).

Três coisas que só aparecem lendo os arquivos inteiros, e não a tabela:

- **`Boot0001` é a mesma entrada que o `{f4057bd0-…}` do `bcdedit`.** O
  `efibootmgr -v` imprime os dados da variável, e lá está
  `BCDOBJECT={f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}` em UTF-16 hexadecimal. É o
  que liga as duas ferramentas, e o que dispensa deduzir a correspondência.
  **A captura de 22/08 é a exceção, e a exceção é o achado**: ali `Boot0001`
  chama-se `UEFI OS`, carrega `\EFI\BOOT\BOOTX64.EFI` em maiúsculas e traz
  `data: 00 00 42 4f` — sem `BCDOBJECT` nenhum. Mesmo device path, outra
  escrita: quem a escreveu foi o firmware, e não o `bcdedit`.
- **O número da entrada mudou de `0001` para `0003`** entre as capturas de
  20/08, o que só acontece quando ela é recriada.
- **Nenhuma das dez tem `BootNext`**, e isso continua não provando nada: o
  firmware o consome ao usá-lo, e todas foram feitas de dentro do Clonezilla.

**A ordem permanente desta máquina muda no ciclo de boot, e não à mão.** Esta
seção dizia o contrário — "foi alterada por alguém" — e o marco de 22/08
mostrou o que de fato acontece: o firmware reescreve a entrada ao bootar por
ela, e o Windows a recria no `displayorder` ao subir. As três mudanças que se
atribuíam a trabalho manual têm essa causa, inclusive o `0001` virando `0003`.
Ver [ADR-0009](../../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md).

**E a linha de 20:57 é a que fecha P-18.** `BootCurrent: 0001` com
`BootOrder: 0000,0001`: a máquina bootou por uma entrada que **não** era a
primeira da ordem. Só o `bootsequence` explica.

> **O que separa os dois backups não é a ordem de boot.** Este parágrafo dizia
> que em 21/08 o dispositivo estava à frente e que era por isso que aquele
> backup não provava nada. Ele estava em **segundo**, igual ao de 22/08 — a
> leitura que dizia o contrário é da restauração daquele dia. O que os separa é
> que em 21/08 **não existia ARCA** (o `git log` começa em 22/08 às 11:47), e
> com o Windows à frente aquele boot só pode ter vindo de alguém: F12, ou um
> `BootNext` posto à mão. Em 22/08 havia `bootsequence` gravado pelo ARCA e
> ninguém tocou na máquina. Corrigido na etapa E9 (ADR-0011).

## O `bootsequence`, medido pela primeira vez (etapa E7)

A E2 e a E4 registraram aqui que **nenhuma captura tem `bootsequence`**, e que
o formato estava coberto por caso construído. A E7 mediu, em 22/08/2026, com a
entrada do ARCA **fora** do `displayorder`:

```text
> bcdedit /set {fwbootmgr} bootsequence {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
A operação foi concluída com êxito.                             (código 0)

> bcdedit /enum {fwbootmgr}
identificador           {fwbootmgr}
displayorder            {bootmgr}
bootsequence            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
timeout                 1

> bcdedit /deletevalue {fwbootmgr} bootsequence
A operação foi concluída com êxito.                             (código 0)
```

O caso construído da E2 estava certo, byte a byte: `bootsequence` com o mesmo
recuo dos outros campos, entre `displayorder` e `timeout`. E três coisas que
não estavam medidas: o `bcdedit` **aceita** a marca para uma entrada de fora da
ordem, o `displayorder` **não muda** nem ao pôr nem ao tirar, e com
`bootsequence` presente o `/deletevalue` sai com **código 0** — ao contrário do
código 1 medido na E4 quando não há o que apagar.

**O firmware honra a marca, e isso foi medido em 22/08/2026.** Esta seção dizia
que faltava um reinício; o reinício aconteceu, e quem o registrou foi o
`nvram-live-2026-08-22.txt` — escrito **durante** o boot que se queria
explicar. Ver [ADR-0007](../../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md)
e [ADR-0009](../../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md).

## O primeiro desfecho, e as cinco capturas do marco (etapa E8)

O backup `2026-08-22_Apps` foi armado às 20:53:48 de 22/08/2026, disparado por
boot único, e colhido às 21:14:49. É a primeira vez que uma receita montada
pelo ARCA rodou em hardware, e a primeira vez que o mecanismo de desfecho —
`arca-fim.txt`, selo, `ARCA_FIM`, `if/then/else` — existiu fora de um teste.

Cópias byte a byte, conferidas por SHA256 contra o dispositivo depois de
gravadas.

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `arca-fim-2026-08-22_Apps.txt` | `E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt`, 51 bytes | `a19d051d…375acac5` | **P-16.** O primeiro desfecho que uma receita do ARCA escreveu |
| `arca-check-2026-08-22_Apps.log` | `E:\2026-08-22_Apps\arca-check.log`, 3832 bytes | `98024c08…76d8d44b` | As **duas formas** do ADR-0003 no mesmo arquivo, escritas pela receita e não à mão |
| `ocs-sr-linha-de-comando-2026-08-22.txt` | `E:\2026-08-22_Apps\Info-saved-by-cmd.txt`, 103 bytes | `cc76de36…9e442653` | O `ocs-sr` que de fato rodou, escrito pelo próprio Clonezilla |
| `nvram-live-2026-08-22.txt` | `E:\2026-08-22_Apps\efi-nvram.dat`, 1642 bytes | `44345e21…ac114a83` | **P-18.** A ordem de boot no instante do boot |
| `bcdedit-enum-firmware-2026-08-22-pos-marco.txt` | `bcdedit /enum firmware`, 21:17, 1716 bytes | `3cd147f5…6ab02e56` | O firmware **depois** do marco: três entradas, e **duas** delas em `partition=R:` |

> **O `bcdedit` redirecionado sai em UTF-8, e não em CP850.** A E2 mediu que a
> página de código é a do console de quem chama; esta captura foi feita com
> `Start-Process -RedirectStandardOutput`, que não dá console nenhum ao filho —
> e o `bcdedit` escreveu UTF-8. A primeira tentativa a converteu de CP850 por
> hábito e produziu `Inicializa├º├úo`; os bytes crus é que são a captura. Vale
> escrito porque a regra da E2 continua certa e a conclusão prática dela — "o
> `bcdedit` sai em CP850" — só vale quando há console.

### O `arca-fim.txt` tem original, e o original é ele próprio

Três linhas, e cada uma é um pedaço de código que nunca tinha rodado:

```text
ARCA_SELO=7d2d2f5153625b38
ARCA_BACKUP=OK
ARCA_FIM
```

O selo bate com o do `estado.json` do mesmo job, conferido a olho e não só pelo
julgamento da E5: as duas cadeias de dezesseis dígitos são a mesma.

**Este arquivo é o contrário do padrão que este documento vinha nomeando.** O
`ARCA_VEREDITO=` (ADR-0003) e o `arca-fim.txt` de 21/08 pareciam prova de que a
receita os escrevia, e nenhuma escrevia — vieram do trabalho de validação em
volta. Este veio da receita, e o que o atesta é a
`ocs-sr-linha-de-comando-2026-08-22.txt` ao lado dele: o Clonezilla registrou
sozinho o comando que executou, e ele é o que a receita mandou executar.

O `arca-fim.txt` de 21/08 continua em `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\`
e continua **sem linha de selo** — 25 bytes, `ARCA_RESTORE=OK` e `ARCA_FIM`. Os
dois lado a lado são a diferença entre o que uma pessoa escreveu e o que a
receita escreve.

### O relógio do live está três horas atrás, e não é defeito novo

O `arca-fim.txt` tem `mtime` de **18:06** lido do Windows, e foi escrito às
**21:06**. É a armadilha do §11 pelo outro lado: o live lê o RTC — que o
Windows grava em hora local — como se fosse UTC, e o NTFS guarda o resultado em
UTC. Três horas, que é exatamente o offset desta máquina.

Vale escrito aqui porque a próxima pessoa vai comparar esses `mtime` com o
`armado_em` do `estado.json` e concluir que o desfecho é anterior ao job. Não
é: é a mesma hora em dois fusos. É o motivo de S-6 e do selo existirem.

### O que **não** está aqui, e por quê

**O `grub.cfg` como ficou armado.** Ele não existe mais: o `arca resultado`
desarma ao colher, e desarmar reescreve o arquivo — o armado foi substituído
pelo inerte às 21:14:50, e `src/desarme.rs` não guarda cópia. A primeira
receita que o ARCA gravou num dispositivo durou vinte e um minutos e não
sobreviveu à colheita.

**E não há reprodução guardada no lugar dela, de propósito.** A derivação é
determinística e as quatro entradas são conhecidas — o inerte está aqui
conferido por SHA256, e o nome, o disco e o selo estão no `estado.json` e no
`arca.log` —, então reproduzi-la é fácil:

```text
cargo run --example orcamento_da_linha_do_kernel -- --arquivo
```

Guardar o resultado disso **nesta pasta** seria pôr um arquivo derivado ao lado
de originais, que é a ambiguidade que este documento inteiro existe para
desfazer. Quem quiser a linha, gera; e o exemplo diz, no cabeçalho, que ela é
reprodução e não captura.

**As telas do `arca resultado`, pelo mesmo critério.** As duas colheitas — o
backup em 22/08 às 21:14:36 e a restauração em 23/08 às 11:50:53 — foram vistas
em tela e transcritas para o §5.4 e o §6.3 do PRD, e a do §5.4 foi **conferida
linha a linha** contra a original em 23/08. Uma transcrição conferida é uma boa
transcrição, e continua não sendo um arquivo que o hardware escreveu: o ARCA
imprime no console, e o que ele grava em disco são o `arca.log` e o
`estado.json`, que estão aqui. O lugar das telas é o PRD, onde elas
documentam a interface; esta pasta é dos arquivos.

## O modelo do bloco do ARCA é o `live-toram` (etapa E7)

Achado ao decidir a forma do `menuentry` que a E7 insere. A captura
`grub-backup-arca-teste-02.cfg` é o `menuentry --id live-toram` do
`grub-inerte-arcaboot.cfg` com **exatamente cinco** substituições — as cinco de
§10.2.1 do PRD — e nada mais.

O `live-default`, que era o candidato óbvio, **não tem** `toram`. O
`live-toram` tem, e exatamente na posição em que as capturas armadas o mostram.
Ninguém acrescentou o `toram`: ele veio junto do modelo, e o §10.2.1 do PRD o
atribuía ao `menuentry` base sem dizer qual.

Duas coisas mais, das mesmas comparações:

- **O único byte em que a derivação e a `teste-02` divergem** é um espaço: a
  captura tem dois entre `locales=en_US.UTF-8` e `keyboard-layouts=NONE`. É
  rastro de edição à mão. O ARCA escreve um, e o teste **nomeia** a diferença
  em vez de copiá-la — reproduzir um artefato de edição seria confundir o que
  rodou com o que se quis.
- **A `teste-03` perdeu nove parâmetros** que o modelo tem: `hostname`,
  `ocs_live_extra_param`, as três `*.blacklist`, `vmwgfx.enable_fbdev`,
  `ocs_1_cpu_udev`, `scsi_mod.use_blk_mq` e `nvme.poll_queues`. É a única das
  quatro com `set default="arca-backup"` — a única que provavelmente rodou
  desatendida —, e perdeu o parâmetro de NVMe numa máquina cujo disco de origem
  é NVMe.

## As quatro capturas da etapa E9

Cópias byte a byte, conferidas por SHA256 contra o dispositivo depois de
gravadas. As três de NVRAM estavam em `E:\` e vieram para cá porque o ADR-0011
argumenta sobre a **forma** da entrada em cada uma, e não só sobre a ordem de
boot — e a tabela acima não carrega isso.

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `nvme0n1-gpt-2026-08-22_Apps.sgdisk` | `E:\2026-08-22_Apps\nvme0n1-gpt.sgdisk`, 840 bytes | `ddcaf4ff…` | **R-7.** O tamanho do disco de origem, em setores, escrito pelo Clonezilla dentro da imagem. É o oráculo de `src/gpt.rs` |
| `nvram-live-backup-2026-08-21.txt` | `E:\2026-08-21_WindowsCompleto\efi-nvram.dat`, 1642 bytes | `44345e21…` | A NVRAM durante o **backup** de 21/08: `0000,0001`, e a entrada como `UEFI OS`. **É byte-idêntica à de 22/08** |
| `nvram-live-restauracao-2026-08-21.txt` | `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\nvram-antes.txt`, 2299 bytes | `6697a7cf…` | A NVRAM durante a **restauração** de 21/08, uma hora e meia depois: `0001,0000`, e a entrada como `ARCA` com `BCDOBJECT` |
| `nvram-live-restauracao-2026-08-20-R2.txt` | `E:\ARCA-LOGS\R2\nvram-depois.txt`, 2305 bytes | `53fefaea…` | Um boot pelo dispositivo em que a entrada **não** foi reescrita: `Clonezilla`, caminho em minúsculas, `BCDOBJECT` presente. É o que descarta a primeira metade de P-19 |

E uma quinta, tirada **antes** do marco em vez de depois:

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `bcdedit-enum-firmware-2026-08-23-antes-da-restauracao.txt` | `bcdedit /enum firmware`, 23/08/2026, 1713 bytes | `7bdae900…` | O firmware **antes** da restauração da E9: três entradas, `displayorder` com `{f4057bd0}` em primeiro, e **nenhum `bootsequence`** |

Ela existe porque o §3.4 diz que `-iefi` não toca na NVRAM, e a evidência disso
é um **par** — antes e depois do mesmo evento. As três restaurações que
sustentam aquela seção foram feitas à mão, e o par delas foi escrito de dentro
do live; esta é a primeira metade do par pelo lado do **Windows**, de uma
restauração que o ARCA vai disparar. A outra metade se tirou depois de religar,
e está na seção seguinte — **o par não fechou idêntico**, e é o achado do marco
([ADR-0012](../../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)).

**As três de NVRAM só significam alguma coisa juntas**, e é por isso que estão
aqui as três. Uma delas sozinha diz uma ordem de boot; as três em sequência
dizem que a entrada foi reescrita entre 20/08 e 21/08 12:51, voltou à forma do
`bcdedit` até 21/08 14:28, e estava reescrita de novo em 22/08 — três mudanças
em três dias, nenhuma delas feita pelo ARCA.

E o `nvme0n1-gpt.sgdisk` está aqui pela razão de sempre: `src/gpt.rs` extrai
`976773168 sectors` e `Sector size (logical/physical): 512/512 bytes` daquele
formato, e um teste contra texto inventado provaria que eu sei imaginar o
formato do `sgdisk`. `500.107.862.016 ÷ 512 = 976.773.168` é o mesmo número que
o `MSFT_Disk` responde hoje, byte a byte — e o `Win32_DiskDrive` responde
`976.768.065` para o mesmo disco (ADR-0010).

## As cinco capturas do marco da E9 (23/08/2026)

A restauração foi armada às 11:10:50, o `ocs-sr` terminou às 11:31:55 do
relógio do live, e a colheita foi às 11:50:53. Estas cinco são o que sobrou
dela, e **três não tinham original nenhum** antes deste dia.

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `arca-fim-restauracao-2026-08-22_Apps.txt` | `E:\ARCA-LOGS\restauracao-2026-08-22_Apps\arca-fim.txt`, 52 bytes | `95991759…457a1828` | O primeiro `ARCA_RESTORE=OK` **com selo** que existiu. O `if/then/else` de R-5 tomou o ramo do êxito numa restauração |
| `estado-restauracao-2026-08-22_Apps.json` | `R:\arca\estado.json`, 181 bytes | `7f0be5f7…785355ea` | A outra ponta do selo, escrita no Windows **antes** do reinício. É o único registro do armar que sobreviveu (§4.1) |
| `arca-restore-2026-08-22_Apps.log` | `E:\ARCA-LOGS\restauracao-2026-08-22_Apps\arca-restore.log`, 16600 bytes | `e4cba0de…faffaa8e` | O log do Clonezilla que a receita redireciona (D2). **Começa no meio** — ver abaixo |
| `bcdedit-enum-firmware-2026-08-23-pos-restauracao.txt` | `bcdedit /enum firmware`, 1334 bytes | `d837093d…f204f15e` | A segunda metade do par. **É byte a byte a captura da E2**, de 22/08 de manhã |
| `arca-log-windows-2026-08-23-pos-restauracao.txt` | `%LOCALAPPDATA%\ARCA\arca.log`, 24583 bytes | `fb07ca73…1244d4aa` | §4.1 medida: o log salta de 22/08 20:53:48 direto para 23/08 11:50:53 |

### As duas pontas do selo, e cada uma de um lado do reinício

`ce04819cf0ee96f7`, no `estado.json` do `ARCABOOT` e na primeira linha do
`arca-fim.txt` do `ARCAVAULT`. Uma foi escrita pelo Windows antes de a máquina
desligar; a outra pelo `bash` do live, depois de o disco inteiro ter sido
apagado e reescrito. E o Windows que lê as duas agora **não é o mesmo** que
escreveu a primeira: ele veio de dentro da imagem.

### O `arca-restore.log` não é o log inteiro, e isso só se vê medindo

Ele tem o fim da operação e não tem o começo. Uma passagem só do Partclone — a
da `nvme0n1p4`, 1,1 GB em 8,64 s, a última das quatro —, nenhuma da `p1`, `p2`
ou `p3`, e nenhum `Starting /usr/sbin/ocs-sr` para o `Ending` que está lá. O
arquivo abre com as sequências de limpeza de tela com que cada passagem do
Partclone começa.

Está aqui assim mesmo, e a cópia é byte a byte do que estava no `ARCAVAULT`:
**o que ele é vale mais do que o que ele deveria ser.** O §6.3 manda quem
colheu uma restauração procurar ali, e quem procurar precisa saber que o que
está ali pode não cobrir a parte que falhou.

A causa não está determinada. A pergunta para a próxima restauração é se o
corte cai sempre no mesmo lugar.

### O `arca.log` é a captura que prova o buraco, e não o que ele contém

Ele é o único arquivo deste diretório que está aqui **pelo que lhe falta**. A
última linha do lado de lá é de 22/08 às 20:53:48 — o armar do backup —, e a
seguinte é de 23/08 às 11:50:53, a colheita da restauração. Sumiram no meio a
colheita do backup das 21:14, o `--dry-run` da manhã de 23/08, a recusa da
confirmação errada, e **a linha do armar desta restauração**.

A operação destruiu o registro do próprio armar. O que sobrou está no
`estado.json`, no `ARCABOOT`, e é para isto que o §4.1 existe.

### O que se perdeu, e não dá para recuperar

**A tela do `arca restore` depois da confirmação** — as cinco linhas do §6.1 e
o aviso de C-9. Elas foram impressas de verdade, e a sessão que as imprimiu
morreu no reinício que ela mesma disparou. É o mesmo caso do `grub.cfg` armado
que a E8 registrou: o código as reproduz de forma determinística, e
**reprodução não é captura**, que é a razão de este diretório existir.

## A estrutura de partições do dispositivo (etapa E10)

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `estrutura-de-particoes-do-dispositivo-2026-08-23.txt` | `Get-Disk`, `Get-Partition`, `Get-Volume` e `Win32_DiskDrive` do dispositivo, 1625 bytes | `a2b8ae68…00ee3504` | **PR-5.** O esquema que `arca prepare` transcreve, e o `MediaType` que o separa de um disco fixo |

Lida com os cmdlets `MSFT_*` — a **mesma régua** que o ADR-0010 escolheu para
R-7, e não o `Win32_DiskDrive`, que responde outro tamanho para o mesmo disco.

**O achado é que o dispositivo é MBR, e boota por UEFI assim mesmo.**
`MbrType 7` para o `ARCAVAULT` e `MbrType 12` (FAT32 LBA) para o `ARCABOOT`,
este no **fim** do disco; nenhuma das duas é `IsActive`, o que confirma que o
boot é UEFI puro e não BIOS. O esquema canônico moderno seria GPT com uma ESP —
este não é ele, e é o que está bootando nesta máquina desde 19/08.

Por isso ele está aqui: **`arca prepare` transcreve uma estrutura medida, em vez
de inventar uma que deveria funcionar.** É o ADR-0004 aplicado a partições, e o
que ele protege é o modo de falha pior deste projeto — um dispositivo que não
boota, descoberto depois de o Windows já ter sido apagado, porque é aí que
alguém precisa dele.

**O que ele não é: prova de que só este esquema funciona.** É uma configuração
que funciona, medida uma vez. Um GPT+ESP provavelmente também bootaria; o ponto
é que ninguém mediu, e a E10 não é onde se descobre.

## `md5sums-2026-08-22_Apps.txt` e `verificacao-md5-medida-2026-08-23.txt`

Copiados e medidos em **23/08/2026**, na etapa E11, com o dispositivo ARCA
conectado — e nesta sessão ele veio em **`D:`**, e não no `E:` que todas as
outras capturas mostram. A letra muda de uma conexão para outra e o rótulo não,
que é o que B-1 e S-3 dizem; é a primeira vez que este diretório tem os dois
valores para prová-lo.

**O `md5sums-2026-08-22_Apps.txt` é cópia byte a byte** do
`D:\2026-08-22_Apps\MD5SUMS`, escrito pelo Clonezilla em 22/08 às 18:00:49 do
relógio do Windows (21:00:49 do relógio do live — P-7). 2129 bytes, 39 linhas,
LF puro. É o **oráculo do parser** de `src/md5sums.rs`: nenhum teste daquele
módulo pode ser ajustado para passar, porque o alvo é este arquivo.

O `.gitattributes` marca `recursos/capturas/** -text` justamente para que o LF
sobreviva ao git, e há um teste que falha se um CR aparecer aqui.

**O `verificacao-md5-medida-2026-08-23.txt` é medição, e não cópia.** Ele
registra o que nenhum arquivo do dispositivo diz: quanto V-1 custa, quanto V-2
custou, a forma exata da resposta do `certutil` e a versão das três ferramentas
do `System32`.

Dois achados dele valem mais do que os números:

**A ordem do `MD5SUMS` não é alfabética pura.** Os catorze `nvme0n1p*` — os
39,7 GB — ficam no **meio**, entre o `nvme0n1-mbr` e o `nvme0n1-pt.parted`. Quem
olhar as primeiras e as últimas linhas conclui que ele cobre só os metadados, e
V-1 nasceria aprovando imagens tendo lido 2 KB de 39,7 GB. É a armadilha *"ler
as pontas de uma lista e concluir o que há no meio"*, no §11 do PRD.

**Quatro arquivos da pasta ficam fora do `MD5SUMS`, e cada um tem hora.** O
`MD5SUMS`, o `clonezilla-img` e o `Info-img-id.txt` levam o **mesmo mtime** —
18:00:49, o fim do `savedisk` —, e o `arca-check.log` é de 18:06:02, escrito
cinco minutos depois pelo `ocs-chkimg`. Não é falta: é a hora em que cada um
nasceu, e é isso que faz `arca verify` contar esses arquivos sem chamá-los de
problema.

**O que estes dois arquivos não são: prova de que V-2 funciona.** O que está
medido neles é V-1, que roda no Windows, e o tempo de V-2 sai de `mtime` de uma
operação de 22/08 disparada por um `arca backup` — e não por um
`arca verify --completo`. **Quem prova V-2 são os três arquivos abaixo.**

## Os três do marco de V-2, em 23/08/2026

`arca-fim-verificacao-2026-08-22_Apps.txt`,
`estado-verificacao-2026-08-22_Apps.json` e
`arca-check-2026-08-22_Apps-pos-verificacao.log`. Copiados do dispositivo logo
depois da colheita, com a máquina ainda na sessão que a colheu.

**O desfecho é o original do `ARCA_VERIFY=`**, que era código novo desde a
escrita da E11 e não tinha rodado:

```text
ARCA_SELO=aefa48f71fc66a46
ARCA_VERIFY=OK
ARCA_FIM
```

Cinquenta e um bytes, três linhas, e o selo bate com o do `estado.json` do mesmo
job — conferido a olho, como o do marco da E8.

**O `estado.json` é o original do campo `disco` vazio.** `"comando":
"verificacao"` com `"disco": ""` é o sentinela que a E11 escolheu, e ele deu a
volta pelo binário que mora no `ARCABOOT`:

```json
{ "selo": "aefa48f71fc66a46", "comando": "verificacao",
  "nome": "2026-08-22_Apps", "disco": "", "armado_em": "…", "situacao": "colhido" }
```

**E o `arca-check.log` de depois é evidência de uma previsão que falhou.** A
E11 escreveu que o `>>` deixaria **duas** marcas `ARCA_VEREDITO=` no arquivo —
o caso que o ADR-0003 previu em 22/08. Ficou **uma**, e o log do backup sumiu.

O que separa append de truncamento sem depender de tamanho: toda execução do
`ocs-chkimg` abre com a mesma sequência de escapes de terminal
(`ESC ) 0 ESC [ 1 ; 2 4 r`). Comparando os dois arquivos:

```text
arca-check-2026-08-22_Apps.log ............. 3832 bytes · 1 marca · 1 abertura
arca-check-…-pos-verificacao.log ........... 4759 bytes · 1 marca · 1 abertura
                                  (append daria >7600 bytes e 2 de cada)
```

**Por isso os dois ficam lado a lado, e o antigo não é substituído.** Ele é a
única cópia do veredito que o backup de 22/08 escreveu, e o dispositivo já não
o tem. É P-25, e quem for fechá-la precisa dos dois arquivos para comparar.

**O que eles não são: a causa.** Nenhum deles diz **por que** o arquivo foi
substituído. A receita tinha `>>` — o `--dry-run` a imprimiu assim minutos
antes de armar —, e o ensaio em bash prova que `>>` acrescenta. O que aconteceu
entre o redirecionamento e o disco não está medido.

## `grub-verificacao-2026-08-24.cfg` — a receita que fechou P-25

Copiada do `boot/grub/grub.cfg` do dispositivo **antes** de `arca resultado`
desarmar, na segunda verificação armada da `2026-08-22_Apps`. 12.442 bytes,
`dcc1cb65…0d46cd66`, LF puro.

**É a primeira captura de uma receita de verificação que de fato rodou.** As
três capturas de `grub-*` anteriores são de backup e de restauração; a de V-2
faltava, e era ela que faltava para P-25 — até aqui, a afirmação *"a receita
tinha `>>`"* vinha do `--dry-run`, e não do arquivo que o GRUB leu. Ela tem:

```text
ocs-chkimg -b -or /home/partimag 2026-08-22_Apps >> /home/partimag/2026-08-22_Apps/arca-check.log 2>&1
```

**O que a segunda verificação mediu**, com o tamanho lido antes de armar:

| | Antes | Depois |
|---|---|---|
| Tamanho | 4759 bytes | **4759 bytes** |
| SHA256 | `0ebf57a0…05bdf843` | **o mesmo** |
| `mtime` | 23/08 | **24/08 13:32:54** |

**Escreveu, e escreveu por cima.** O `arca-fim.txt` desta receita — selo
`b668820c0a23ab5f` — leva o **mesmo `mtime` ao segundo**, o que prende a
escrita a esta execução e não a outra. O conteúdo saiu byte a byte igual ao de
23/08: duas execuções do `ocs-chkimg` sobre a mesma imagem dão o mesmo arquivo,
e é por isso que o de 23/08 parecia o antigo. **Por isso o log não foi copiado
de novo** — seria o mesmo arquivo com outro nome.

**E os dois de 23/08 respondem a segunda metade, sem reinício nenhum.**
Comparados byte a byte, eles não diferem em conteúdo, e sim em **onde** o bloco
de relatório de 927 bytes foi depositado:

```text
antes (`>`)   [moldura 0–2569][RELATORIO 2569–3496][cauda 3496–3809][ARCA_VEREDITO]
depois (`>>`) [moldura 0–2569][progresso 2569–3496][cauda 3496–3809][RELATORIO][ARCA_VEREDITO]
```

`antes[2569:3496]` e `depois[3809:4736]` são o mesmo bloco de 927 bytes, e
`antes[3496:3809]` e `depois[3496:3809]` são idênticos. Com `>` o relatório cai
no **meio** e sobrescreve o progresso do partclone; com `>>` ele cai no **fim**,
que é o efeito de `O_APPEND`. **O `>>` chega ao `ocs-chkimg`**, e quem esvazia
o arquivo age antes do primeiro byte.

**Um achado de tabela sai daí: todo `arca-check.log` de backup tem um buraco.**
O de 22/08 perdeu 927 bytes de progresso — `Starting to check image`,
`File system`, `Device size` — e sobrou o pedaço cortado no meio da palavra,
`maining: 00:00:00Ave. Rate:`. O fixture do `ARCA-TESTE-03` em `src/imagens.rs`
tem o mesmo padrão: o banner do partclone colado direto em
`Checked successfully.`. O veredito sobrevive porque é a última linha, escrita
pelo bash; o que se perde é diagnóstico, e nenhuma tela promete diagnóstico.

## A etapa E10 — `arca prepare` (23/08/2026)

Sete arquivos, e eles se dividem em três grupos: **o que foi medido à mão antes
de o código existir**, **o pacote que o ARCA instala**, e **o marco**.

### O que foi medido à mão, antes de escrever código

Como a E7 fez com o `bootsequence` e o C-13 com o `displayorder`. Os quatro
arquivos são a saída dos comandos, com o código de saída de cada um.

| Arquivo | O que é |
|---|---|
| `medicao-criacao-de-entrada-2026-08-23.txt` | `bcdedit /copy {bootmgr}` numa máquina de verdade, com o `{fwbootmgr}` lido antes e depois |
| `medicao-criacao-de-entrada-parte2-2026-08-23.txt` | os dois `/set`, a releitura de C-3, a comparação com a entrada `ARCA` que já existia, e o `/delete` |
| `medicao-particionamento-2026-08-23.txt` | `Clear-Disk`, `Initialize-Disk`, `New-Partition` ×2, `Format-Volume` ×2 e a releitura, no segundo dispositivo desta mesa |
| `medicao-letras-e-ordem-2026-08-23.txt` | como se atribui letra a uma partição, e o `/remove` sobre uma entrada recém-criada |

**A medição da entrada de firmware criou uma entrada de boot nesta máquina e a
apagou.** O último bloco de cada arquivo é a prova disso: `a entrada de medicao
sumiu: True` e `o displayorder tem so o {bootmgr}: True`. É o mesmo cuidado que
o ADR-0013 teve ao medir `/addfirst` e conferir a NVRAM no fim.

**A medição do particionamento destruiu um disco de 447 GB de propósito** — o
segundo dispositivo, que existe para isso. O arquivo registra o que havia nele
antes, que é o que a tela de PR-4 mostra a quem vai perder dados.

Três coisas dessas medições não eram óbvias e mudaram o desenho:

- **`New-Partition` cria com `MbrType 6`**, e quem acerta para 7 e 12 é o
  `Format-Volume`. Não há `Set-Partition -MbrType` no caminho: o tipo é efeito
  colateral de outra operação, e é por isso que a releitura de PR-5 importa.
- **As duas partições nascem sem letra**, e o ARCA exige letra. Quem atribui é
  o `Add-PartitionAccessPath -AssignDriveLetter`, que **não é idempotente**: a
  segunda passada recusa e não muda nada — o caso do `bcdedit /deletevalue` do
  ADR-0005.
- **`bcdedit /copy` põe a entrada nova no `displayorder` sozinho**, que é o
  perigo que C-5 nomeia. Ver o [ADR-0017](../../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md).

> **Um detalhe de codificação, e ele não é do ARCA.** Os quatro arquivos foram
> escritos por scripts `.ps1` sem BOM, e o PowerShell 5.1 lê `.ps1` sem BOM
> como **ANSI**: os travessões saíram como três bytes de mojibake e foram
> corrigidos na gravação aqui. Nada mais foi tocado. O ARCA não passa por isso
> porque fala com o PowerShell por `-EncodedCommand` em UTF-16 — o que
> `src/adaptadores/windows/wmi.rs` já registrava por outro motivo.

### O pacote que o ARCA instala

| Arquivo | O que é |
|---|---|
| `clonezilla-checksums-2026-08-23.txt` | o `CHECKSUMS.TXT` de `free.nchc.org.tw/clonezilla-live/stable/`, o mirror do próprio projeto |
| `grub-clonezilla-do-pacote-3.3.3-15.cfg` | o `boot/grub/grub.cfg` de dentro do `clonezilla-live-3.3.3-15-amd64.zip` |

**O SHA256 tem duas fontes, e é isso que o torna verificação.** O
`CHECKSUMS.TXT` veio do mirror do projeto; o arquivo veio do **SourceForge** e
foi medido com `certutil -hashfile … SHA256`. Servidores diferentes, o mesmo
número — `00cee7700433e63017e2ea9eb40519108829710132364a8028a6c039a6046304`,
561.478.648 bytes.

**E o `grub.cfg` do pacote responde de onde veio o dispositivo desta mesa.**
Comparado com o `grub-clonezilla-original.cfg`, ele difere em duas coisas: o
`noeject` em treze `menuentry`, e **seis segundos** no carimbo do rodapé
(`04:11:28` contra `04:11:22`). Seis segundos é o `ocs-live-dev` gerando o ISO
e o zip na mesma execução — é a mesma build, e o dispositivo veio do ISO. Ver o
[ADR-0018](../../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md).

### O marco, em duas execuções

| Arquivo | O que é |
|---|---|
| `arca-prepare-2026-08-23-marco.txt` | a tela do primeiro `arca prepare --dispositivo 1`, inteira |
| `arca-prepare-2026-08-23-com-iso.txt` | a segunda execução, com `--iso` (PR-2) e a entrada de firmware **criada** |
| `arca-prepare-2026-08-23-criacao-da-entrada.txt` | o `bcdedit` lido antes e depois da segunda, com o disco relido no fim |
| `arca-status-dois-dispositivos-2026-08-23.txt` | o que o `arca status` responde com os **dois** dispositivos na mesa |

**Foram duas execuções, e a segunda não é redundância.** A primeira **reusou** a
entrada de firmware que esta máquina já tinha, que é C-4 na letra — e com isso o
caminho da **criação** não foi exercitado pelo código, só pela medição à mão.
Por isso a entrada `ARCA` foi apagada e o comando rodou de novo: ele criou a
`{f4057bd3-…}`, apontou-a para `partition=F:` e a tirou da ordem permanente.

É a diferença entre *"o comando funcionou"* e *"o caminho que a etapa existe
para escrever funcionou"*, e um marco que só exercita o ramo fácil é o caso
construído mais fácil do que o real que o §11 nomeia.

**O que a primeira tela tem e a segunda não:** o aviso `ESTE DISCO JA E UM
DISPOSITIVO ARCA`. O disco que o marco destruiu tinha os dois rótulos — sobra da
medição à mão —, e essa é a única captura desse aviso.

**O que a segunda tem e a primeira não:** o `--iso`, o `criada` no lugar do
`reusada e reapontada`, e o `a entrada saiu da ordem permanente` no lugar do
`ja estava fora`.

**E a captura do `arca status` é de um defeito que o marco criou.** Com o
dispositivo novo pronto, a mesa passou a ter **dois** `ARCAVAULT` — e C-10
recusa, corretamente, todo comando que se localiza pelos rótulos. Inclusive o
de diagnóstico, que é o que se roda quando a situação ficou confusa.

A recusa está certa e não mudou. O que mudou foi a **mensagem**: ela nasceu na
E1, quando ter dois dispositivos ARCA exigia comprá-los, e dizia *"Desconecte os
demais"* sem dizer quais. Desde a E10 o ARCA faz o segundo, e o arquivo guarda a
forma nova — com as letras e com a causa provável nomeada.

## A etapa E12 — `arca sondar` (23/08/2026)

### `arca-sondar-antes-do-marco-2026-08-23.txt`

Três telas do binário da E12 rodando **de dentro do `ARCABOOT`**
(`F:\arca\arca.exe`), no dispositivo que o `arca prepare` criou horas antes e que
está **vazio de imagens**. Nenhuma delas arma nada.

O arquivo existe para fixar o estado **antes** do marco, e o que ele fixa são
três afirmações que o marco confirma ou desmente:

| A tela | O que ela afirma |
|---|---|
| `arca backup … --dry-run` | `Disco de origem … POR DETERMINAR`, e a recusa manda `arca sondar` — não há `blkdev.list` nenhum no dispositivo |
| `arca sondar --dry-run` | a receita que será armada, com o selo de ensaio (dezesseis zeros) |
| `arca status` | a entrada `ARCA` `{f4057bd3-…}` apontando para `partition=F:`, e **`1 entrada(s), nenhuma para o dispositivo`** |

**A terceira linha é a que faz o marco valer por P-26 inteira.** Com a entrada
fora da ordem permanente, o boot único é a **única** forma de a máquina chegar
ao dispositivo: se ela bootar, (a) o dispositivo boota e (b) a entrada que o
`arca prepare` criou leva a ele — as duas metades de uma vez. Um F12 responderia
só (a).

> **Por que o binário foi copiado antes.** O `arca prepare` instala no
> `ARCABOOT` o executável que está rodando, e o que estava lá era o da E10 — ele
> **não conhece** `Operacao::Sondagem`. Armar com o binário novo e colher com
> aquele deixaria o `arca resultado` recusando o `estado.json` do job que ele
> mesmo tem de colher, e mandando rodar `arca desarmar`, que resolve o
> dispositivo e **perde o desfecho**. A E11 já pagou exatamente por isso.
>
> Rodar o `--dry-run` **de lá** é como se confere que a cópia aconteceu, e é o
> que estas três telas fazem: elas saíram do binário do dispositivo, e não do
> `target\release`.

### `arca-sondar-marco-2026-08-24.txt`

O marco. Duas telas do lado Windows, **depois** de a máquina ter bootado pelo
dispositivo, rodado o `lsblk` sozinha e desligado:

| A tela | O que ela mostra |
|---|---|
| `arca resultado` | `Desfecho: concluida`, `Discos vistos: sda (Maxtor Z1 SSD 480GB), nvme0n1 (KINGSTON SNV3S500G)`, selo `354da624e7fa0d21` |
| `arca backup … --dry-run` | `Disco de origem ..... nvme0n1 · lido da sondagem de 24/08 11:58 (carimbo do Clonezilla, P-7)` |

O arquivo traz, no cabeçalho, o conteúdo dos dois arquivos que a receita
escreveu — 50 bytes de `arca-fim.txt` e 852 de `blkdev.list` —, porque a
colheita **desarma e encerra o job**, e a próxima operação vai truncar aquele
`arca-fim.txt`.

> **A segunda tela é a corrigida, e o arquivo diz isso.** A primeira versão dela
> imprimiu duas linhas afirmando fontes diferentes para o mesmo nome: o pré-voo
> dizia `lido da sondagem`, e o ensaio, quatro linhas abaixo, dizia `lido do
> blkdev.list de uma imagem` — uma frase fixa de antes de a sondagem existir. O
> defeito foi corrigido entre uma execução e outra, e o cabeçalho do arquivo o
> registra em vez de escondê-lo.
>
> **É a mesma decisão das capturas do `arca prepare`**, tomada pelo outro lado:
> lá o texto errado ficou preservado porque era o que a tela imprimiu; aqui a
> execução foi refeita porque a captura existe para mostrar o comando **certo**,
> e o registro do errado vive no cabeçalho e no plano de etapas.

### Os três da sondagem que deu certo

`blkdev-list-da-sondagem-2026-08-24.txt`, `arca-fim-sondagem-2026-08-24.txt` e
`estado-sondagem-2026-08-24.json` — cópias byte a byte do que ficou no
dispositivo depois do marco.

**Eles existem porque a pasta da sondagem é fixa e a segunda sondagem escreve por
cima da primeira** (SD-4). Sem estas cópias, a falha forçada de 15:32 teria
apagado o único original do primeiro `ARCA_PROBE=OK` deste projeto — e foi
exatamente isso que ela fez no dispositivo.

O `estado.json` está aqui com `"situacao": "colhido"`, que é o estado **depois**
da colheita: ele é o único lugar que liga o selo `354da624e7fa0d21` ao job, e o
`arca sondar` seguinte o sobrescreveu.

### `arca-sondar-falha-forcada-2026-08-24.txt`

**O primeiro `FALHOU` deste projeto**, e o único arquivo deste diretório que
nasceu de uma execução montada para falhar.

A sondagem foi armada com uma coluna inventada no `lsblk` — `FLAGQUENAOEXISTE` —,
e o dispositivo voltou com 54 bytes de `arca-fim.txt` e 40 de `blkdev.list`:

```text
ARCA_SELO=95772dae07463701      lsblk: unknown column: FLAGQUENAOEXISTE
ARCA_PROBE=FALHOU
ARCA_FIM
```

O arquivo traz as três telas do lado Windows — `arca resultado` (com código de
saída **1**), `arca backup --dry-run` e `arca status` —, e o cabeçalho diz **como
a falha foi montada e como foi desfeita**: a mutação de `FLAGS_DE_SONDAGEM` não
está no repositório, e quem colheu foi o binário normal.

> **É o mesmo movimento do ADR-0017**, em que a entrada de firmware de medição
> foi criada, medida e apagada, e da segunda execução do marco da E10. O que se
> quer é exercitar o caminho que nenhuma execução normal exercita — e desfazer o
> que foi montado para isso, para que o repositório não fique com uma mentira
> compilável dentro.

**O que ele permite conferir, e nenhuma captura anterior permitia:** que o
`if/then/else` de R-5 toma o **ramo do erro** em hardware; que o `2>&1` da
receita guarda a causa no dispositivo em vez de deixá-la sumir com o `poweroff`;
e que as duas telas seguintes **concordam** — `FALHOU` e `POR DETERMINAR` —, que
é exatamente o que a forma com `;` teria tornado contraditório.

### O que este par de arquivos permite conferir, e nenhum outro permitia

**Que o dispositivo boota pela entrada que o ARCA criou.** As três telas do
arquivo de *antes* mostram a entrada `ARCA` `{f4057bd3-…}` apontando para
`partition=F:` e **fora da ordem permanente** — `1 entrada(s), nenhuma para o
dispositivo`. As duas telas do arquivo do *marco* mostram um desfecho escrito
por uma receita que só roda se aquela entrada tiver sido honrada.

Entre um e outro há um reinício e **nenhum F12**, e é isso que fecha as duas
metades de P-26 de uma vez.

## O par que fechou P-22, e o que aparece nele sem que ninguém escreva

Duas leituras do `bcdedit /enum firmware` separadas por um religar limpo, em
24/08/2026 — SSD ARCA conectado, sem job armado, `grub.cfg` conferido inerte
byte a byte (`4b33da61…f947aa3d`). A máquina foi direto ao Windows.

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `bcdedit-enum-firmware-2026-08-24-antes-do-religar.txt` | `bcdedit /enum firmware`, 17:11:50, 1398 bytes | `89ca7ad1…7b8df3b9` | A ordem depois de C-13: `{bootmgr}`, `{f4057bd3}` — **duas** entradas |
| `bcdedit-enum-firmware-2026-08-24-pos-religar.txt` | `bcdedit /enum firmware`, 17:26:14, 2133 bytes | `7ba552b5…4f0599a2` | **P-22.** A mesma ordem com **cinco**, e as três novas só o firmware escreve |
| `arca-status-2026-08-24-antes-do-religar.txt` | `arca status`, 1352 bytes | `58485816…f2f1d086` | A tela lendo a de cima: `dispositivo em 2o de 2` |
| `arca-status-2026-08-24-pos-religar.txt` | `arca status` | `4ffd901c…3668886d` | A tela lendo a de baixo: `dispositivo em 2o de 5` |

**O que o par permite conferir, e nenhum outro permitia.** O `diff` é limpo —
nada removido, nada alterado, três entradas acrescentadas ao `displayorder`:
`UEFI:CD/DVD Drive`, `UEFI:Removable Device` e `UEFI:Network Device`. São
classes de dispositivo que o firmware enumera no POST; não têm `device` nem
`path`, e nada no BCD as originaria. **Logo o `bcdedit` imprime conteúdo que só
existe na NVRAM.** Ver
[ADR-0020](../../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md).

### Duas coisas que só aparecem cruzando com as capturas antigas

- **As mesmas três já estiveram lá, com outros GUIDs.** O
  `bcdedit-enum-firmware-legado-pt.txt`, de 20/08, traz as três descrições
  idênticas em `{c71136d7/d8/d9-9c6a-11f1-8a41-…}`; as de agora são
  `{6cc093db/dc/dd-9ff9-11f1-8a4e-…}`. São UUIDs versão 1, e o `time_mid`
  avançou: **foram geradas de novo**, e não recuperadas de cache. É o mesmo
  sinal que esta página já registrou quando o número de uma entrada foi de
  `0001` para `0003`.
- **O `node` do UUID separa as duas origens, e o padrão é total.** Em todas as
  capturas de `bcdedit` deste diretório, sem exceção: `806e6f6e6963` é sempre
  uma entrada `Aplicativo de Firmware (101fffff)` — `{687478f2}`, as três de
  20/08, as três de 24/08 —, e `aa4ed9bd2b34` é sempre um objeto do BCD —
  `{f4057bca}`, `{f4057bd0}`, `{f4057bd3}`. O `{f4057bd3}` nasce de
  `bcdedit /copy {bootmgr}` (ADR-0017), que é por que ele fica do lado do BCD.
  **Quem ler uma captura futura separa as duas coisas sem sair do arquivo.**

> **Sobre a codificação, e a regra da E2 não previa este caso.** Esta página
> registra que `Start-Process -RedirectStandardOutput` "não dá console nenhum ao
> filho" e por isso o `bcdedit` sai em UTF-8. Nestas duas ele saiu em **CP850**,
> pelo mesmo comando, com `-Wait` e sem `-NoNewWindow`. Foram convertidas para
> UTF-8 na gravação, como as da E2, e só nisso. A regra continua certa no que
> afirma — a página de código é a do console de quem chama —, e o que não está
> determinado é **quando** esse caminho dá console ao filho.

## O que nenhuma delas contém

**Nenhum `bootsequence`.** As capturas de `bcdedit` deste diretório continuam
sem ele, e continuam certas: não há job armado nesta máquina. A medição do boot
único da E7 está transcrita acima e não foi guardada em arquivo, porque o
comando que a produziu desfez o que fez.

**Nenhuma menção a `Removable Media` ou `External hard disk media`.** Estas
palavras não são do `bcdedit`: são valores de `MediaType` do WMI
(`Win32_DiskDrive`, em `cimwin32.dll`). Nem o `bcdedit.exe` nem os seus
`.mui` contêm qualquer uma delas — procurado nos dois idiomas. Ver o que
`src/firmware.rs` diz sobre C-6.

> **`UEFI:Removable Device` não é uma delas, e a semelhança engana.** Ela é
> `description` de uma entrada de firmware, escrita pelo próprio firmware desta
> placa, e aparece nas capturas de 20/08 e de 24/08. Não tem relação com o
> `MediaType` do WMI nem com o parágrafo acima — o que a torna interessante é
> outra coisa, e é P-28: ela não declara para onde aponta.
>
> **As duas capturas viraram fixture em 24/08/2026**, e é a mesma medição nas
> duas: a `legado-pt` e a `pos-religar` têm cinco entradas na ordem, as três
> `UEFI:*` sem `device` nenhum, e as duas primeiras posições com alvo — que é o
> que impede o aviso de C-14 de sair em toda tela. Ver
> [ADR-0021](../../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).

## O par que fechou P-28, e o terceiro arquivo é o de antes

O experimento das 18:39–18:47 de 24/08/2026: a `{6cc093dc}` `UEFI:Removable
Device` promovida ao **topo** da ordem permanente com um `bcdedit /set … /addfirst`
à mão, `grub.cfg` conferido inerte byte a byte (`4b33da61…9f47aa3d`), sem job
armado, e um reinício com o SSD conectado.

| Arquivo | O que é | SHA256 | O que prova |
|---|---|---|---|
| `bcdedit-enum-firmware-2026-08-24-removable-em-primeiro.txt` | 18:39, 2133 bytes | `82ebe078…32842023` | A entrada opaca em **1º**, o `{bootmgr}` em 2º e o `ARCA` em 3º |
| `arca-status-2026-08-24-removable-em-primeiro.txt` | 18:39, 1846 bytes | `e4b1bf3c…390de091` | **O aviso de C-14 em hardware**, pela primeira vez: `dispositivo em 3o de 5 · UEFI:Removable Device vem antes` seguido do parágrafo |
| `bcdedit-enum-firmware-2026-08-24-pos-boot-removable.txt` | 18:47, 1398 bytes | `89ca7ad1…7b8df3b9` | **P-28.** As três `UEFI:*` sumiram da ordem e da enumeração, e o `{bootmgr}` voltou ao topo |
| `arca-status-2026-08-24-pos-boot-removable.txt` | 18:47, 1380 bytes | `1d015549…0e51db55` | A tela lendo a de cima: `dispositivo em 2o de 2`, sem aviso |

**O terceiro arquivo é o primeiro de novo.** O SHA256 dele é o mesmo do
`bcdedit-enum-firmware-2026-08-24-antes-do-religar.txt`, das 17:11:50: o
firmware desfez, num POST, tudo o que dois eventos tinham feito — as três
entradas que ele mesmo acrescentara às 17:26 e o `/addfirst` das 18:39. **O ARCA
não escreveu nada nesse intervalo**; o `arca resultado` não chegou a rodar.

Quem desfez não está separado: o firmware ao reconstruir no POST, ou o Windows
ao subir. Ver
[ADR-0021](../../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).

> **A escrita à mão fica registrada, porque este diretório distingue as duas
> coisas.** O `/addfirst` das 18:39 foi feito **por uma pessoa**, com o
> `bcdedit` elevado, para produzir o estado a medir — o mesmo método do
> ADR-0013. O que as capturas guardam é o que as ferramentas responderam
> **depois** dele, e o desfecho de 18:47 é do firmware, não de ninguém.

## A medição do dispositivo em GPT, e ela está pela metade (25/08/2026)

O roteiro de `PRD/marco-em-hardware-gpt-2026-08-25.md` tem nove etapas, e este
arquivo é o que as **seis primeiras** escreveram. Elas não decidem nada — quem
decide é a Etapa 7, o boot. O que elas produzem é a preparação medida, e duas
das três perguntas que o ADR novo precisa responder já estão respondidas.

| Arquivo | O que é | SHA256 |
|---|---|---|
| `medicao-gpt-2026-08-25.txt` | As etapas 1 a 6 em **dois** dispositivos de 238,5 GB, 40 650 bytes | `faad0a3a…6f52849a` |

**O SHA256 vale para o arquivo parado na Etapa 6.** Ele muda quando as etapas
seguintes escreverem nele, e é para isso que serve estar anotado aqui: a
medição continua no mesmo arquivo, e não numa cópia.

**São dois dispositivos, e a troca no meio é parte do achado.** O roteiro correu
no Kingston DataTraveler Max até a Etapa 6, emperrou lá, e recomeçou da Etapa 2
num KGSSE100 256 do mesmo tamanho. As Etapas 3, 4 e 5 estão medidas **duas
vezes**, e as duas bateram em tudo — o que faz delas medição repetida, e não
medição refeita.

### O que já está respondido

**Houve MSR, e nos dois dispositivos.** O `Initialize-Disk -PartitionStyle GPT`
criou sozinho uma partição `Reserved` de 16 759 808 bytes no offset 17 408, com
`GptType {e3c9e316-0b5c-4db8-817d-f92df00215ae}` — coisa que em MBR o
`Initialize-Disk` não faz, e que o `arca prepare` não espera. Deixada em pé,
ela empurraria a ARCAVAULT para partição 2 e a ARCABOOT para 3, o device path
da entrada de firmware viraria `HD(3,GPT,…)`, e a releitura que confere a ordem
das duas partições no disco passaria a ver três. Foi removida, e o disco voltou
a zero partições — que é como o MBR sai do `Initialize-Disk`.

Ela apareceu de novo no segundo dispositivo, com os mesmos três números. Isso é
o que separa "aconteceu" de "acontece": `particionador.rs` tem de removê-la
**sempre**, e não "se houver".

**A GPT cobra 1 400 832 bytes, e o `LargestFreeExtent` já os desconta.** Num
disco de 256 060 514 304 bytes, o extent livre depois de remover a MSR é
256 059 113 472. A diferença é a tabela primária no começo e a cópia secundária
no fim — a razão pela qual a Etapa 4 tira as contas do `LargestFreeExtent` lido
na hora, e não de constante.

**A NVRAM desta máquina tem uma entrada só.** `{fwbootmgr}` com `displayorder`
apontando para `{bootmgr}`, e mais nada. Não há entrada `ARCA` nenhuma: o
Windows foi reinstalado, e o que as capturas de 22 a 24/08 mediram nesta NVRAM
não está mais lá. É o número de referência da Etapa 6 — ao final dela devem ser
duas, e a Etapa 9 volta a uma.

**O `GptType` sai pronto do `New-Partition`, e o `Format-Volume` não encosta
nele.** As duas partições nascem `{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}` e
continuam assim depois de formatadas — medido nos dois dispositivos. É o
contrário do MBR, onde as duas nascem `MbrType 6` e só chegam a 7 e a 12 depois
do `Format-Volume`.

**E a resposta trouxe uma pergunta que ninguém tinha feito: em GPT o tipo não
distingue as duas.** Em MBR, `7` (IFS) e `12` (FAT32 LBA) separavam a ARCAVAULT
da ARCABOOT, e é disso que vivem as constantes de `preparacao.rs:573,576`. Em
GPT as duas têm o mesmo `GptType`. A releitura não perde a conferência — perde o
**critério**: quem quiser saber qual é qual precisa do rótulo, do sistema de
arquivos ou da ordem no disco.

**O `bcdedit` recusa em silêncio o Kingston DataTraveler Max.** O
`/set <id> device partition=E:` responde *"A operação foi concluída com êxito"*,
código 0, e a releitura traz o device antigo. Por caminho de dispositivo, igual.
O `path`, no mesmo comando e na mesma entrada, pega de primeira. Quatro alvos
cercaram a causa: `partition=C:` (NVMe, GPT) pega; `partition=D:` e
`partition=E:` (DataTraveler, GPT) não; `partition=F:` (KGSSE100, MBR) pega; e
`partition=E:` (**o mesmo KGSSE100 convertido para GPT**) pega. Não é o GPT e não
é o USB — é aquele dispositivo.

Isto é o C-6 que `prepare.rs:678` já descrevia sem ter um caso, e é a razão de
`Erro::AlvoDoFirmwareRecusado` existir em `prepare.rs:694`. A releitura de C-3 é
a única coisa entre esse silêncio e um `arca prepare` que diria ter preparado um
dispositivo que não boota.

**Um detalhe de leitura que só o segundo dispositivo esclareceu:** apontar o
device por `\Device\HarddiskVolumeN` é relido como `partition=X:` quando aquele
volume tem letra. É **normalização**, e não recusa — o que confirma o
`Alvo::ParticaoSemLetra` de `firmware.rs:79` como forma de escrita válida.

### O que ainda não está

O boot é a Etapa 7, e é ele que decide se o ADR-0014 muda ou se confirma. O
dispositivo está armado para ele desde 25/08/2026: `{fwbootmgr}` com
`bootsequence` apontando para `{f4057bd6-65a4-11f1-b0f1-aa4ed9bd2b34}`, e
`displayorder` inalterado, trazendo só o `{bootmgr}` — que é C-5.

> **A primeira tentativa da Etapa 3 rodou e o registro dela se perdeu.** O
> script foi executado por `powershell.exe` 5.1, que leu o arquivo UTF-8 como
> ANSI: o `—` dos títulos virou `â€"`, e a aspa tipográfica que aparece no meio
> fechou as strings cedo. O `Clear-Disk` chegou a rodar e deixou o disco RAW; o
> `Initialize-Disk` não. A segunda tentativa rodou sob `pwsh` 7.6.5, encontrou
> o disco RAW, e é ela que está no arquivo — com o parágrafo dizendo que o
> `Clear-Disk` do registro é o efeito da primeira. Entre uma e outra nada foi
> escrito no disco.

> **Dois comandos do roteiro não faziam o que diziam, e a captura registra as
> duas coisas — o que eles fizeram e o que os substituiu.** O
> `-replace 'set timeout=\d+'` da Etapa 5 não casa com a linha real do
> `grub.cfg`, que é `set timeout="30"`, com o número entre aspas: respondeu sem
> erro e não mudou nada. E o `bcdedit /displayorder <id> /remove` e o
> `bcdedit /bootsequence <id>` da Etapa 6, sem alvo, operam sobre o `{bootmgr}` —
> o gerenciador do Windows — e não sobre o `{fwbootmgr}`. O segundo chegou a
> escrever um `bootsequence` no `{bootmgr}` apontando para um `bootx64.efi` que a
> ESP do sistema não tem; foi desfeito com `bcdedit /deletevalue` na mesma
> sessão, e a releitura do `{bootmgr}` está no arquivo, sem `bootsequence`.
>
> As duas entradas de firmware criadas no DataTraveler — a de teste e a de
> controle — foram removidas na mesma noite, com a NVRAM relida em ambos os
> casos e voltando às duas entradas da Etapa 1. Deixar entrada morta na ordem é
> o que o [ADR-0021](../../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md)
> diz não ser segurança, e vale para entrada de medição também.
