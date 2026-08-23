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
