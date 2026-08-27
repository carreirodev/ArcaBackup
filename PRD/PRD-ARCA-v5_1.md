# PRD — ARCA v5.1

**Automatizador de Clonezilla para backup e restauração de imagem de disco.**

Versão 5.1 · 22/08/2026 · Substitui a v4
Última revisão: 24/08/2026 à noite, **o ciclo de backup e restauração que fechou P-23 e refutou a hipótese de P-19**. O `arca-restore.log` não começa no meio: o Clonezilla reabre o próprio log com truncamento na última passagem e o descritor da receita retoma de um offset alto — o vão vira NULs, 53% do arquivo, e o corte cai **onde o `ocs-sr` chegou**, não sempre no mesmo lugar ([ADR-0022](../docs/adr/0022-o-arca-restore-log-e-truncado-por-baixo.md)). O mecanismo e as cinco consequências foram commitados **antes** da restauração que as mediu, e quatro bateram inteiras; a quinta cobrava demais, e a correção está registrada. **P-19 mudou de enunciado**: o `efi-nvram.dat` que vem de graça dentro de cada imagem — o segundo braço do experimento, que não custou nada — mostra a entrada intacta num boot por `bootsequence` idêntico ao de 22/08, que a reescreveu. Mesmo gatilho, mesmo dispositivo, resultados opostos: **não é o `bootsequence`**, e não fecha por reinício ([ADR-0023](../docs/adr/0023-o-bootsequence-nao-e-o-gatilho-da-reescrita.md)). §11 ganha o caso mais claro que tem da armadilha de medir o firmware depois do reinício: em 22/08 a leitura de dentro do boot e a de depois discordam sobre a existência da entrada do ARCA, e as duas estão certas
Revisão anterior: 24/08/2026, **etapa E12, marco cumprido — e o primeiro `FALHOU` deste projeto**. Uma segunda sondagem foi armada com uma coluna inventada no `lsblk`, e o dispositivo voltou com `ARCA_PROBE=FALHOU` e `lsblk: unknown column: FLAGQUENAOEXISTE` dentro do próprio `blkdev.list`: **o `if/then/else` de R-5 tomou o ramo do erro em hardware pela primeira vez**, o `arca resultado` reportou com código 1, e a tela seguinte disse `POR DETERMINAR` — as duas concordando, que é o que o `;` teria tornado contraditório. §5.5 ganha a **segunda** linha com original. **P-6 continua aberta**, e a distinção é o ponto: ela pergunta pelo `ocs-sr`, e quem falhou aqui foi o `lsblk`. A falha também expôs um teste que aceitava mais do que devia — o das colunas passava com uma coluna a mais, e a mutação atravessou a suíte
Revisão anterior: 24/08/2026, **etapa E12, marco cumprido** — `arca sondar` armou às 14:56:55, a máquina bootou **pela entrada de firmware que o `arca prepare` criou** num dispositivo sem imagem nenhuma, o `lsblk` rodou sozinho e ela desligou; a colheita saiu `concluida` com `ARCA_PROBE=OK`. **P-26 fecha inteira, as duas metades de uma vez** — a entrada estava fora da ordem permanente, então o boot único era o único caminho possível. §8 ganha o nono comando; §9.7 nasce com **SD-1 a SD-6**; §10.2.5 nasce com a quarta receita; §4.5 ganha a **segunda fonte** do oráculo, e o custo que ele cobrava — o primeiro backup pelo menu do Clonezilla — deixou de existir; §10.2.2 ganha uma **terceira procedência**, a *reconstrução*, para o que tem original do resultado e não da linha de comando ([ADR-0019](../docs/adr/0019-a-sondagem-e-a-quarta-operacao.md)). **E a medição que nenhuma etapa tinha**: o boot do Clonezilla isolado custa **1 min 40 s** nesta máquina, do reinício ao desligamento. §11 ganha três armadilhas, e duas delas o marco imprimiu na tela com a suíte verde — uma frase fixa afirmando a fonte errada do nome do disco, e a data da sondagem com o dono do relógio trocado na própria doc
Revisão anterior: 23/08/2026, **etapa E10, marco cumprido** — o `arca prepare` particionou um SSD de 447 GB, instalou o Clonezilla, criou a entrada de boot e a tirou da ordem permanente, **sem um único reinício**: é o primeiro comando destrutivo do ARCA que não custa um. §7.1 ganha a tela real e §7.2 nasce com o pacote, a versão e o SHA256 de **duas fontes**; §8 perde o `--destino` e ganha o `--dispositivo` obrigatório; **C-4 ganha a outra metade** — criar entrada de firmware deixou de ser código sem original, e a entrada `ARCA` desta máquina era a cópia do `{bootmgr}` que a explicava desde sempre ([ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md)); **C-5 ganha a segunda aplicação, e ela nasceu de um achado** — `bcdedit /copy` põe a entrada nova na ordem permanente **sozinho**; §11 ganha cinco armadilhas, entre elas duas ferramentas com o mesmo nome respondendo por perguntas diferentes; a **dívida do ADR-0015 foi paga**, e R-7 passou de `>=` para `==` no código. E o `grub.cfg` do zip respondeu de onde veio o dispositivo desta mesa: **seis segundos** de carimbo separam o ISO do zip da mesma build ([ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md))
Revisão anterior: 23/08/2026, **etapa E11, marco cumprido** — a verificação armada rodou às 16:53:30 e foi colhida `concluida` com veredito `APROVADA`; **P-24 fecha** e o `ARCA_VERIFY=` ganha original. **E o marco desmentiu uma linha desta etapa**: o `>>` do §10.2.4 devia deixar duas marcas no `arca-check.log` e deixou uma — o log do backup sumiu, a causa não está medida, e é **P-25**, a primeira vez neste projeto em que uma receita rodou e o rastro divergiu do que a string manda fazer. §11 ganha as três armadilhas novas, e uma delas custou uma operação inteira: a tela não dizia que o menu do Clonezilla fica **trinta segundos** parado antes de a receita começar. **V-1 perde o "em segundos", e o requisito é que estava errado**: são **202,6 s** para 39,7 GB, medidos, e a tela passa a estimar pelo tamanho real ([ADR-0016](../docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md)); §9.5 ganha a tela de V-1, que é **execução real**, e a de reprovação junto; §8 ganha o `--completo` e diz por que ele pede confirmação sem destruir nada; §10.2.4 nasce com a terceira receita, e o `>>` do `arca-check.log` é a decisão que ela carrega; §10.2.2 e §3.5 registram que **V-2 não rodou** (P-24). E o `MD5SUMS` foi lido de verdade antes de o leitor existir — o formato é do Clonezilla, e a ordem dele não é alfabética, o que quase fez V-1 nascer conferindo só metadados
Revisão anterior: 23/08/2026, **etapa E9, escrita** — R-7 reescrito contra o help do `ocs-sr` e contra a medição das duas réguas do mesmo disco ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)), **P-17 fecha**; §6.1 ganha a tela real e perde o `498,7 GB` — a **sexta** vez do mesmo número medido na coisa errada; §6.2 ganha o que a imagem de fato carrega; §3.1 corrigido — as duas leituras de NVRAM de 21/08 são de **dois boots diferentes**, e a que o documento usava é da restauração ([ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md)); P-19 **estreita**: a primeira metade está descartada por medição; §8 ganha `--destino`; §11 ganha a armadilha de datar a captura e não saber de que operação ela é
Revisão anterior: 22/08/2026, **marco em hardware das etapas E7 e E8** — o primeiro backup disparado e colhido pelo ARCA, sem uma única tela. **P-16 e P-18 fecharam** (§3.5); §3.1 mostra que a ordem permanente muda **no ciclo de boot**, e não à mão ([ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)); §5.2 e §5.4 são de execução real, com o `Desfecho esperado em` que a revisão da E7 já tinha corrigido no código; §10.2.3 ganhou o orçamento medido da linha que rodou; §11 ganhou a armadilha de medir o firmware **depois** do reinício; P-19 aberta
Revisão anterior: etapas E7 e E8, escrita — §3.1 ganhou a **tabela de ordem de boot** desta máquina; §10.2.1 corrigido (o `menuentry` base é o **`live-toram`**, e o `toram` nunca foi acrescentado); §5.2 ganhou as cinco linhas do armar e a ordem certa entre confirmação, aviso e reinício; §4.5 decide o que fazer sem nome de disco (**recusar**); §4.3 e §5.4 ganharam o `estado.json` de seis campos e a linha `Job: encerrado`
Revisão anterior: etapa E6 — §4.5 diz **de onde sai o nome do disco de origem**, que o documento nunca disse; §5.2 corrigido contra medição (o `498,7 GB` era a partição `C:`, não o disco); B-4, B-5, B-6, C-6 e C-10 ganharam o que a medição mostrou
Revisão anterior: etapa E5 — §4.3 ganhou o **formato do selo** e os três lugares por onde ele passa; §5.5 ganhou a linha do `arca-fim.txt` **sem selo nenhum**, que a tabela não tinha
E antes: etapa E4 — §3.2 ganhou o `set default`, que é o que faz o boot ser desatendido e não estava documentado; §4.4 define o **estado inerte**, que o §5.2, o §5.4 e o §6.3 pressupunham; §8 ganhou `arca desarmar`; P-18 aberta sobre a §3.1
E antes: etapa E3 — §3.1, §3.2, §3.5, §10 e os requisitos B-8, B-9, C-6, R-4, R-5, R-6, S-4 reescritos contra as receitas preservadas em `recursos/capturas/`
Uso pessoal · Um usuário · Sem distribuição

> **As fundações não são hipótese.** O mecanismo descrito neste documento foi
> executado em hardware real: backup completo validado e restauração completa
> bem-sucedida. Este PRD especifica o **aplicativo** a ser construído sobre um
> mecanismo já provado — não um experimento a validar.

---

## Índice

1. [O que é](#1-o-que-é)
2. [O que não é](#2-o-que-não-é)
3. [Fundações validadas](#3-fundações-validadas)
4. [Estrutura de um dispositivo](#4-estrutura-de-um-dispositivo)
5. [Fluxo: backup](#5-fluxo-backup)
6. [Fluxo: restauração](#6-fluxo-restauração)
7. [Fluxo: preparar dispositivo](#7-fluxo-preparar-dispositivo)
8. [Comandos](#8-comandos)
9. [Requisitos](#9-requisitos)
10. [Implementação](#10-implementação)
11. [Armadilhas conhecidas](#11-armadilhas-conhecidas)
12. [Decisões e pendências](#12-decisões-e-pendências)

---

## 1. O que é

Uma ferramenta de linha de comando que prepara dispositivos de backup autocontidos e dispara operações neles.

**Cada dispositivo carrega o Clonezilla e as imagens juntos.** Boota nele e escolhe: fazer um backup, ou restaurar um dos que estão ali. O dispositivo é tudo que você precisa — não há nada externo a consultar.

**O ARCA não lê nem escreve disco.** Quem faz isso é o Clonezilla. O ARCA prepara o ambiente, monta a receita, dispara o boot único e lê o resultado.

### O problema que resolve

O procedimento manual exige ~20 telas em modo texto, em inglês técnico, sendo que errar em duas delas destrói o disco. É longo demais para ser feito com a frequência devida — e foi na ausência dele que duas reinstalações de Windows aconteceram em agosto/2026.

**Pelo ARCA:** um comando e uma confirmação digitada. Nenhuma tela, nenhuma decisão técnica.

## 2. O que não é

- ❌ Catálogo ou banco de dados de imagens
- ❌ Rastreamento de número de série ou de qual imagem está em qual disco
- ❌ Backup incremental ou diferencial
- ❌ Agendamento
- ❌ Retenção automática
- ❌ Interface gráfica
- ❌ Gerenciador de discos de uso geral. O `arca prepare` particiona **o dispositivo**, e só ele, com o disco nomeado pelo usuário e confirmado por escrito — disco fixo é recusa dura (ver [P1](#71--o-arca-particiona-o-dispositivo-e-nunca-escolhe-o-disco))
- ❌ Suporte a BIOS legada, BitLocker, RAID, Storage Spaces

**Princípio:** se a informação já existe na listagem de diretórios do dispositivo, não há o que armazenar.

## 3. Fundações validadas

Tudo abaixo foi medido em hardware, não projetado.

### 3.1 — Mecanismo de boot único

| Fato | Evidência |
|---|---|
| Entrada de firmware apontando para SSD externo funciona | A máquina bootou pela entrada de firmware do ARCA, múltiplas vezes |
| **O firmware honra o `bootsequence` sobre uma entrada que não está à frente da ordem de boot** | Medido no marco em hardware de 22/08/2026: o `efibootmgr` registrou, **durante** aquele boot, `BootCurrent: 0001` com `BootOrder: 0000,0001`. A máquina bootou pela entrada `0001` estando a `0000` à frente — nenhuma ordem permanente explica isso. **Fecha P-18**, e é o que torna C-5 sustentável na prática, e não só no papel ([ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)) |
| `bcdedit` **rejeita mídia removível em silêncio** — responde "êxito" e mantém o valor antigo | Pendrive testado e recusado; SSD aceito |
| Partição primária comum basta — não precisa marcar tipo EFI | SSD preparado assim boota normalmente |
| O `bcdedit` **não traduz** os nomes de campo: só `identificador` sai em português | Parser por valor é o correto |
| A entrada legada desta máquina chama-se **`Clonezilla`**, GUID `{f4057bd0-…}` | Procurar só por `ARCA` criaria entrada órfã |
| **O `bcdedit` aceita `bootsequence` para uma entrada que não está no `displayorder`**, e o `displayorder` não muda ao pôr nem ao tirar | Medido na etapa E7, 22/08/2026, com a entrada do ARCA fora da ordem. É o que torna C-5 possível: se o boot único exigisse a entrada na ordem, armar obrigaria a violá-lo ([ADR-0007](../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md)) |

#### A ordem permanente desta máquina muda no ciclo de boot

Levantada na etapa E7, explicada na E8. Todas as capturas de `efibootmgr` do
dispositivo, mais as três leituras do `bcdedit`, na ordem em que foram feitas:

| Quando | Ferramenta | Ordem de boot | Bootou por | `BootNext` |
|---|---|---|---|---|
| 20/08 | `efibootmgr` (`nvram-original.txt`) | `0000,0001` — Windows, ARCA | `0001` (ARCA) | nenhum |
| 20/08 | `bcdedit` (`nvram-windows-antes.txt`) | `{bootmgr}`, **`{f4057bd0}`**, +3 pseudo-entradas | — | — |
| 20/08 | `efibootmgr` (`R1/nvram-antes.txt` e `-depois`) | `0000,0001` | `0001` (ARCA) | nenhum |
| 20/08 | `efibootmgr` (`R2/nvram-antes.txt` e `-depois`) — **restauração R2** | `0003,0000` — **ARCA, Windows** | `0003` (ARCA) | nenhum |
| **21/08 12:51** | `efibootmgr` (`2026-08-21_WindowsCompleto/efi-nvram.dat`) — **o backup** | **`0000,0001` — Windows, ARCA** | `0001` (ARCA) | nenhum |
| **21/08 14:28 e 14:46** | `efibootmgr` (`ARCA-LOGS/2026-08-21_WindowsCompleto/nvram-antes` e `-depois`) — **a restauração** | `0001,0000` — **ARCA, Windows** | `0001` (ARCA) | nenhum |
| 22/08 manhã | `bcdedit` | `{bootmgr}` — **a entrada do ARCA saiu da ordem** | — | — |
| **22/08 ~20:57** | `efibootmgr` (`nvram-live-2026-08-22.txt`) | **`0000,0001` — Windows, ARCA** | **`0001` (ARCA)** | nenhum |
| **22/08 21:17** | `bcdedit` (`…-pos-marco.txt`) | **`{f4057bd0}`, `{bootmgr}`, `{687478f2}` — o ARCA voltou, e em primeiro** | — | — |
| **23/08 manhã** | `bcdedit` (`…-antes-da-restauracao.txt`) | **`{f4057bd0}`, `{687478f2}`, `{bootmgr}` — as duas do dispositivo à frente do Windows** | — | — |
| **23/08 12:12** | `bcdedit` (`…-pos-restauracao.txt`) — **depois da restauração** | **`{bootmgr}` — e a `{687478f2}` sumiu inteira** | — | — |

> **A última linha é a única em que a ordem melhorou, e a causa é nova.** A
> leitura de depois da restauração é **byte a byte** a de 22/08 de manhã —
> mesmo SHA256, `d837093d…f204f15e`. O ARCA não a escreveu (C-5, e há releitura
> no armar e no desarme), e o `ocs-sr` não escreve na NVRAM (§3.4). O que
> sobra é que **a ordem permanente estava dentro da imagem**: o BCD mora na
> partição EFI, e a partição EFI é restaurada junto.
>
> A `{687478f2}` fecha o argumento pela data. Ela nasceu na NVRAM durante o
> boot do backup de 22/08, com o Windows desligado, e nunca chegou ao BCD que
> a imagem daquela noite carrega. Restaurar a apagou porque ela nunca esteve lá
> dentro. Ver
> [ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md).
>
> **O mecanismo desta última frase mudou em 24/08/2026, com P-22 fechada.** O
> `bcdedit` lê a NVRAM, então o sumiço da `{687478f2}` foi da NVRAM — e uma
> entrada dela não some por causa de uma restauração de disco com `-iefi`. O
> candidato medido é outro: **o firmware reconstrói entradas a cada POST**, e
> podar a de um dispositivo desconectado é o que essa reconstrução faria. A
> poda em si não está medida, e fica nomeada como hipótese. Ver
> [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md).

> **As duas linhas de 21/08 eram uma só, e a que estava aqui é da
> restauração.** Corrigido na etapa E9. Os arquivos `nvram-antes.txt` e
> `-depois.txt` moram em `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\`, ao lado de
> um `arca-fim.txt` com `ARCA_RESTORE=OK`, e o `mtime` deles é **14:28 e
> 14:46** — o `savedisk` daquele dia terminou às 12:54. Eles são a restauração
> de 21/08, e são justamente o par que o §3.4 usa para dizer que `-iefi` não
> toca na NVRAM.
>
> A NVRAM do boot do **backup** de 21/08 é o `efi-nvram.dat` de dentro da
> imagem, escrito às 12:51:25, no meio da gravação. Ele diz `0000,0001` —
> **Windows à frente**, exatamente como em 22/08.
>
> Ver [ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md),
> inclusive para o que isso muda em P-18 (nada) e em P-19 (metade dela fecha).

**As duas ferramentas não discordam: elas foram lidas em momentos
diferentes.** Em 20/08 as duas dizem a mesma coisa — a entrada do ARCA estava
na ordem permanente, em segundo lugar. O `bcdedit` mostra três pseudo-entradas
a mais (`UEFI:CD/DVD Drive`, `UEFI:Removable Device`, `UEFI:Network Device`)
que o `efibootmgr` não vê de dentro do Clonezilla; nas entradas reais os dois
concordam. O número da entrada do ARCA também mudou — `0001` virou `0003` —, o
que só acontece quando ela é recriada.

Três coisas saem daí, e as três mudaram de sentido no marco de 22/08:

- **A ordem permanente não foi alterada à mão: ela muda no ciclo de boot.**
  Esta seção dizia que "foi alterada por alguém", e a E8 mediu o que de fato
  acontece. Entre 20:41 e 21:17 de 22/08 ninguém tocou no `bcdedit` a não ser o
  ARCA — que só escreve `bootsequence`, e cuja releitura de C-5 teria falhado
  se a ordem tivesse mudado numa escrita dele. Assim mesmo a entrada `0001` foi
  reescrita pelo firmware (`ARCA` virou `UEFI OS`, sem o `BCDOBJECT`), uma
  terceira entrada apareceu, e o `displayorder` passou de só `{bootmgr}` para
  três entradas com a do ARCA à frente. **O firmware reescreve a entrada ao
  bootar por ela, e o Windows a recria no `displayorder` ao subir.** É a
  explicação que cobre as três mudanças anteriores sem pedir ninguém, inclusive
  o `0001` virando `0003`. Ver
  [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md).
- **Hoje o dispositivo está em primeiro na ordem**, por **duas** entradas: a
  `{f4057bd0}` do ARCA e a `{687478f2}` `UEFI OS`, ambas em `partition=R:`.
  Enquanto o SSD estiver conectado, todo reinício boota nele — inerte, ele para
  no menu do Clonezilla; armado, a receita roda. `arca status` diz isso em toda
  execução, e `tests/e7_armar_o_dispositivo.rs` cobra a invariante que importa:
  **alguma entrada que leva ao `ARCABOOT` em primeiro exige o dispositivo
  inerte.** Repare que a pergunta é sobre **para onde a entrada aponta**, e não
  sobre como ela se chama: procurar só a que se chama `ARCA` deixaria a
  `{687478f2}` passar, e foi por ela que a máquina bootou. O aviso de C-9 —
  remover o SSD antes de religar — é a defesa, e estava escrita antes de alguém
  saber disto.
- **O backup de 21/08 continua não provando nada, e o motivo é outro.** Esta
  seção dizia que era porque o dispositivo estava em primeiro na ordem; ele
  estava em **segundo**, e a leitura que dizia o contrário é da restauração
  daquele dia (ADR-0011). O que separa os dois backups é quem apertou o botão:
  **em 21/08 não existia ARCA** — o `git log` deste repositório começa em 22/08
  às 11:47 —, e com o Windows à frente da ordem aquele boot só pode ter vindo
  de alguém, por F12 ou por um `BootNext` posto à mão. Em 22/08 havia
  `bootsequence` gravado pelo ARCA às 20:53:48 e **ninguém tocou na máquina**.
  O argumento fica mais forte, e não mais fraco: um `BootCurrent` fora da
  frente da `BootOrder` é explicado por F12 tão bem quanto por `bootsequence`,
  e o que o marco tem é a ausência de qualquer mão.

O `BootNext` ausente em todas as dez capturas continua não provando nada — o
firmware o consome ao usá-lo, e todas foram feitas já de dentro do Clonezilla.
O que prova é o par `BootCurrent`/`BootOrder` da captura de 22/08.

> **As palavras `Removable Media` e `External hard disk media` não saem do
> `bcdedit`.** Procuradas no `bcdedit.exe` e nos seus recursos `pt-BR` e
> `en-US`: não estão lá. São valores de `MediaType` do WMI (`Win32_DiskDrive`,
> em `cimwin32.dll`) — outra ferramenta, outra pergunta. Nenhum parser da
> saída do `bcdedit` pode produzi-las. **Quem revela a rejeição silenciosa é a
> releitura de C-3**: um `device` que não mudou depois da escrita. O
> `GetDriveType` do Windows dá o sinal antecipado, antes de qualquer
> tentativa. Medido na etapa E2; ver `recursos/capturas/PROVENIENCIA.md`.

### 3.2 — Receita desatendida

| Fato | Evidência |
|---|---|
| `ocs_repository="dev:///LABEL=..."` funciona e elimina a ambiguidade `sda`/`sdb` | Backup real gravado no destino certo |
| `locales=` vazio abre tela de idioma mesmo em batch — fixar `locales=en_US.UTF-8` | Observado |
| `-batch -sfsck -senc` suprimem todas as perguntas | Backup real sem uma única tela |
| **`-batch`, e nunca `-b`** | O help do `ocs-sr` desta versão: *"You have to use '-batch' instead of '-b' when you want to use it in the boot parameters. Otherwise the program init on system will honor '-b', too."* |
| **O padrão de `-p\|--postaction` é `reboot`** — sem `-p true`, o `ocs-sr` reinicia assim que termina de gravar, e nada depois dele roda | Help do `ocs-sr`. É por isso que as duas receitas reais o trazem |
| `ask_user` é válido para imagem e dispositivo, salvando e restaurando | Documentação oficial + uso |
| **Verificação não roda sozinha em batch** — `ocs-chkimg` tem que ser chamado | Primeiro backup gerou checksum que ninguém conferiu |
| **Pipe (`\|`, `tee`) invalida a string inteira**: o Clonezilla descarta a receita e abre o menu interativo, sem executar nada | Medido. Só redirecionamento simples (`>`, `>>`) é permitido |
| **A receita é uma string única em `ocs_live_run="bash -c '...'"`**, numa linha só do `grub.cfg` — nunca um script de várias linhas | As três receitas preservadas em `recursos/capturas/` |
| **É o `set default` do `grub.cfg` que faz o boot ser desatendido**, e não o `menuentry` da receita. Inserir o bloco só põe mais uma linha no menu | Diff do `grub.cfg` inerte contra `grub-backup-arca-teste-03.cfg`: duas diferenças, e uma delas é o `set default` |

> **O `set default` não estava documentado, e é ele que dispara a receita.**
> Medido na etapa E4. Um `grub.cfg` armado difere do inerte em **exatamente
> duas coisas**: `set default="live-default"` vira `set default="arca-backup"`,
> e um `menuentry ... --id arca-backup` de quatro linhas entra antes do
> `live-default`. Nada mais no arquivo muda — nem `timeout`, nem os outros
> `menuentry`.
>
> A ordem de importância entre as duas é o achado. **O `menuentry` sozinho não
> arma nada**: a máquina espera os trinta segundos do `timeout` e boota no
> Clonezilla normal. Duas das três receitas preservadas estão exatamente nesse
> estado — bloco presente, `set default` em `live-default` —, e nelas nenhuma
> receita rodaria.
>
> **`live-default` e nunca `0`.** O `grub.cfg` que o Clonezilla entrega traz
> `set default="0"`, que aponta por **posição** — e a posição muda, porque o
> bloco do ARCA entra antes do `live-default` e passa a ser o índice 0. Um
> dispositivo com `"0"` está armado no instante em que o bloco é inserido, sem
> que ninguém toque no `set default`: não é o estado inerte, é um estado que
> parece inerte. Ver [ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md).

### 3.3 — Backup validado

Backup real executado ponta a ponta: gravado, verificado e aprovado, sem nenhuma intervenção.

| Fato | Evidência |
|---|---|
| A receita desatendida grava a imagem completa | Imagem real com as 4 partições do `nvme0n1` |
| `ocs-chkimg` aprova a imagem e grava o veredito em arquivo | `arca-check.log` lido na volta |
| Compressão com `-z9p` | ~39% do volume em uso |
| **O ARCA arma, dispara e colhe o ciclo inteiro** | `2026-08-22_Apps`, 22/08/2026: armado às 20:53:48, boot único honrado, imagem de 39,7 GB gravada, `ocs-chkimg` aprovada, desfecho escrito, colhido às 21:14:49. Vinte e um minutos, sem uma única tela e sem ninguém tocar na máquina |

> **Os backups de 21/08 e de 22/08 provam coisas diferentes, e a distinção
> custou P-18.** O de 21/08 provou que a receita grava e que a imagem é
> restaurável — mas rodou com o dispositivo **à frente da ordem permanente**, e
> por isso não separava boot único de ordem de boot. O de 22/08 rodou com o
> Windows à frente, e é ele que prova o mecanismo (§3.1). O primeiro validou o
> Clonezilla; o segundo validou o ARCA.

### 3.4 — Restauração validada

Restauração real sobre o `nvme0n1`. Do comando ao Windows restaurado, **sem intervenção, na primeira tentativa**.

| Fato | Evidência |
|---|---|
| O `ocs-sr` não toca na NVRAM | NVRAM byte-idêntica antes e depois — `ARCA-LOGS/2026-08-21_WindowsCompleto/nvram-antes.txt` e `-depois.txt`, mesmo SHA256, escritos às 14:28:36 e 14:46:51, **do mesmo boot** |
| `-k0` preserva os PARTUUIDs **mesmo com a GPT zerada** | A entrada de boot preexistente continua resolvendo |
| `bcdboot` não é necessário neste hardware | Consequência do anterior |
| O Windows da imagem sobe normalmente | Máquina restaurada e em uso |
| **Uma restauração disparada pelo ARCA vai do comando ao Windows restaurado** | **23/08/2026**, marco da E9: `2026-08-22_Apps` armada às 11:10:50, `ocs-sr` encerrado às 11:31:55, colhida às 11:50:53. `arca-fim-restauracao-2026-08-22_Apps.txt` e `arca-restore-2026-08-22_Apps.log` |
| **A ordem permanente volta ao que está dentro da imagem** | O par `bcdedit` de 23/08, antes e depois, pelo lado Windows: **não é idêntico** ([ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)) |

> **O `-iefi` era a pergunta que originou o projeto.** Está respondida: o
> `ocs-sr` não toca na NVRAM e o Windows sobe.

> **A primeira linha desta tabela mudou de nome no marco da E9, e a evidência
> dela não mudou.** Ela dizia *"`-iefi` funciona — NVRAM byte-idêntica antes e
> depois"*, e o par que a sustenta foi lido **de dentro do mesmo boot do live**,
> separado por dezoito minutos. Isso continua verdadeiro, e o que ele mede é o
> `ocs-sr`.
>
> O par que a E9 acrescentou atravessa o reinício e é lido do lado Windows.
> Ele responde outra pergunta — *o que a máquina tem na ordem permanente depois de
> voltar?* — e a resposta é **o que a imagem carregava**: a leitura de depois da
> restauração é byte a byte a que a E2 tirou em 22/08 de manhã, e as duas
> entradas que o ciclo de boot do backup tinha posto na ordem sumiram junto com
> o disco antigo. A partição EFI está dentro da imagem, e o BCD está dentro
> dela.
>
> Nada aqui foi desmentido; o que faltava era **alcance**. A lição é a que este
> documento já pagou cinco vezes: conferir se a evidência fala sobre a
> pergunta. Ver o ADR-0012, e o [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md)
> para o que P-22 respondeu em 24/08/2026.

> **As três restaurações à mão continuam sendo as três.** R1 e R2 em 20/08, e a
> de 21/08, feitas pelo menu do Clonezilla. O que a E9 acrescentou é a quarta,
> e a diferença dela não é o `ocs-sr` — é o envoltório que diz se deu certo
> (§10.2.2), e ele estreou nesta operação do lado da restauração.

### 3.5 — Ainda não medido

| # | Pendência |
|---|---|
| P-6 | **O `ocs-sr` devolve código diferente de zero quando falha?** O ramo de sucesso foi medido; o de falha não. Uma restauração bem-sucedida não fecha isso, por definição. Fecha com falha forçada, provavelmente em VM. *(24/08/2026: **a forma de R-5 deixou de ser hipótese**, e a pergunta continua aberta. Uma sondagem com coluna inventada no `lsblk` escreveu o primeiro `ARCA_PROBE=FALHOU` deste projeto, e o `arca resultado` o reportou com código 1 — o `if/then/else` funciona nos dois ramos em hardware. **Isso não fala pelo `ocs-sr`**, que é o sujeito de P-6: o que se sabe é que a estrutura da receita está certa, e não que aquele programa devolve o que se supõe.)* |
| P-19 | **Em que condição o firmware cria uma `UEFI OS` no lugar da entrada?** — o enunciado mudou em 24/08/2026, e o candidato que ela carregava **foi refutado**. A primeira metade fechou na E9, pela negativa: o firmware NÃO reescreve a entrada em todo boot pelo dispositivo. O que sobrou era *"só quando ela foi consumida por `bootsequence`?"*, e o experimento de 24/08 eliminou essa hipótese: **duas leituras do `efibootmgr` feitas durante o boot, ambas de um backup, ambas com `BootOrder: 0000,0001` e `BootCurrent: 0001` — logo o mesmo gatilho, pelo argumento de P-18 —, no mesmo dispositivo e no mesmo device path byte a byte, dão resultados opostos**: em 22/08 a entrada `0001` é `UEFI OS` · `\EFI\BOOT\BOOTX64.EFI` · `0000424f`, e em 24/08 é `ARCA` · `\EFI\boot\bootx64.efi` · `BCDOBJECT`. Nenhuma variável conhecida separa os dois casos dos quatro; o que separa é a **data**. E o verbo do enunciado estava errado: em 22/08 a entrada do ARCA `{f4057bd0}` **sobreviveu intacta** do lado do Windows, com a `{687478f2}` `UEFI OS` nascendo ao lado dela — não é reescrita, é uma segunda entrada. **Não fecha por reinício**, e nenhuma tela do ARCA depende da resposta. Ver [ADR-0023](../docs/adr/0023-o-bootsequence-nao-e-o-gatilho-da-reescrita.md) |
| ~~P-21~~ | **Fechada por escopo em 23/08/2026** ([ADR-0015](../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md)): só o disco de origem é destino válido, então o caso que esta pergunta descreve não é alcançável pelo ARCA. ~~O `ocs-sr` sai com código diferente de zero quando desiste por destino menor?~~ Aberta na E9, e é P-6 com outra roupa: o help diz que ele *"quit"*, e se esse `quit` sair com zero o `if/then/else` de R-5 escreve `ARCA_RESTORE=OK` sobre uma restauração que não aconteceu. **Não é urgente**, e a razão é o desenho: R-7 recusa antes, do lado Windows, e essa pergunta só chega a importar se a recusa do ARCA tiver um furo ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)) |
| ~~P-20~~ | ~~O `arca resultado` deve devolver o `{bootmgr}` à frente do `displayorder`.~~ **Fechada em 23/08/2026, etapa E10.** Virou **C-13**. Os quatro comandos foram medidos à mão antes de virar código — `/addfirst` move e não duplica, `/remove` tira sem apagar o objeto, e os quatro respondem "êxito" com código 0 inclusive quando não mudam nada. `/addfirst {bootmgr}` ficou, e `/remove` foi descartado pelo modo de falha: ele precisa acertar **quais** entradas tirar, e essa é a pergunta que a revisão do marco da E8 já pegou respondida errado. C-5 ganhou limite explícito em vez de exceção. Ver [ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md) |
| ~~P-24~~ | ~~A verificação armada (V-2) nunca rodou.~~ **Fechada em 23/08/2026, etapa E11.** `arca verify 2026-08-22_Apps --completo` armou às 16:53:30, a máquina bootou pelo dispositivo, o `ocs-chkimg` rodou sozinho e ela desligou; a colheita saiu `concluida` com veredito `APROVADA`. O `arca-fim.txt` traz `ARCA_SELO=aefa48f71fc66a46`, `ARCA_VERIFY=OK` e `ARCA_FIM` — **o `ARCA_VERIFY=` era código novo**, e agora tem original em `recursos/capturas/`. E a pasta própria provou o que ela existe para provar: o desfecho do backup de 22/08 continua intacto ao lado, com o selo `7d2d2f5153625b38`. **Abriu P-25** |
| P-25 | **Por que o `arca-check.log` foi substituído, se a receita usa `>>`?** Aberta no marco da E11. A verificação armada devia **acrescentar** ao log — o `--dry-run` imprimiu `>> …/arca-check.log 2>&1` minutos antes de armar, e `recursos/ensaio-da-receita.sh` prova que `>>` acrescenta num bash de verdade. Medido depois: o arquivo tem **uma** ocorrência de `ARCA_VEREDITO=`, no fim, e **uma** inicialização de terminal — ou seja, **uma execução do `ocs-chkimg`**, e não duas. O log do backup de 22/08 sumiu. Um append daria mais de 7600 bytes; o arquivo tem 4759. **Alguma coisa entre o redirecionamento e o disco truncou o arquivo, e não se sabe o quê.** O `>>` fica assim mesmo, com a razão trocada: ele não compra a preservação, mas elimina a janela em que o `>` deixa o log em zero byte. Fecha comparando uma segunda verificação armada com esta — e o experimento custa um reinício ([ADR-0016](../docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md)) |
| ~~P-23~~ | ~~**Por que o `arca-restore.log` começa no meio?**~~ **Fechada em 24/08/2026, e ele não começa no meio: é cortado por baixo.** O `>` da receita abre o log e o `ocs-sr` escreve por ele; na última passagem o Clonezilla **reabre o mesmo arquivo com truncamento** e o partclone escreve a tela dele nos bytes 0–4.084; o descritor da receita, com o offset intacto, retoma lá em cima, e o intervalo vira zeros — **53% do arquivo é NUL**. A pergunta era se o corte cai sempre no mesmo lugar, e a resposta é **não: ele cai onde o `ocs-sr` tinha chegado** (fim do buraco em 12.890 numa restauração, 12.924 na outra). O início é constante porque é o tamanho da tela do partclone, e os primeiros 4.085 bytes dos dois logs são byte a byte idênticos. **O corte não é do ARCA nem do redirecionamento.** O §6.3 continua verdadeiro; o que ele não diz é que o log traz uma passagem só — a última, que numa falha é aquela em que a operação parou. Ver [ADR-0022](../docs/adr/0022-o-arca-restore-log-e-truncado-por-baixo.md) |
| ~~P-22~~ | ~~O `bcdedit /enum firmware` mostra a NVRAM do firmware, ou o BCD do disco?~~ **Fechada em 24/08/2026: a NVRAM, e quem provou foi o firmware.** Um religar limpo — SSD conectado, sem job armado, `grub.cfg` inerte — parou no Windows, e isso responderia só a metade operacional. O que fecha a pergunta literal são **três entradas que apareceram no `displayorder` no meio do reinício**: `UEFI:CD/DVD Drive`, `UEFI:Removable Device` e `UEFI:Network Device`, classes de dispositivo que o firmware enumera no POST e que nada no BCD originaria. **O `bcdedit` imprime conteúdo que só existe na NVRAM.** Cai junto a dúvida do ADR-0013: C-13 conserta o firmware, e não um espelho dele. Ver [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md) |
| ~~P-28~~ | ~~**`UEFI:Removable Device` alcança o `ARCABOOT`?**~~ **Aberta e fechada em 24/08/2026, e o firmware apagou a testemunha.** Com aquela entrada posta em **primeiro** na ordem permanente às 18:39 — `grub.cfg` inerte conferido byte a byte, sem job armado — a máquina reiniciou com o SSD conectado e **subiu o Windows**: ela não desvia o boot. **E o que apareceu depois vale mais**: às 18:47, sem o ARCA ter escrito nada, as três `UEFI:*` sumiram da ordem **e da enumeração**, e o `bcdedit /enum firmware` é byte a byte o das 17:11 (`89ca7ad1…7b8df3b9`), com o `{bootmgr}` de volta ao topo. **Duas leituras que a medição não separa**: ou a entrada foi tentada e não alcançou, ou foi descartada antes de ser tentada, na mesma reconstrução que apagou as três — e para o efeito operacional dá no mesmo. Isso mede pela metade a hipótese de poda do ADR-0020: podar ele poda; o que falta é a poda de uma entrada cujo dispositivo saiu. **No código nada muda** — C-14 foi escrito para não depender desta resposta. O texto abaixo é o do dia em que ela foi aberta. Ver [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md). ~~Aberta em 24/08/2026, no fechamento de P-22.~~ As três entradas que o firmware acrescentou **não têm `device` nem `path`** — só `identificador` e `description` —, e o ARCA lia a ausência de alvo como *não levam ao dispositivo*, que é a resposta tranquilizadora. Mas `UEFI:Removable Device` é a classe que boota o primeiro removível, e o `ARCABOOT` é um SSD USB removível. **Duas telas dependiam disso, e a segunda só apareceu na medição em duplo**: com aquela entrada em primeiro, a linha saía `dispositivo em 3o de 5 · UEFI:Removable Device vem antes` — correta ao pé da letra, e **sem** o parágrafo de perigo, que morava só no ramo em que o dispositivo é o primeiro; e com a entrada `ARCA` **fora** da ordem — o estado que o `arca prepare` deixa — a tela não engolia um aviso e sim **afirmava**: `so o boot unico leva a ele`. É a terceira forma da falha que o ADR-0009 pegou uma vez — *"diria 'o Windows vem antes' e engoliria o aviso"* — e C-6 pegou noutra: aqui não é nome errado nem alvo errado, é a **ausência** de alvo virando segurança. **O código foi consertado em 24/08/2026, sem esperar a medição** ([ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md)): o julgamento tem três estados, `NaoSeSabe` não vale como segurança em lugar nenhum, e o `arca prepare` parou de prometer o boot em texto fixo. **Não é urgente**, e por duas razões melhores do que a posição das três: C-13 põe o `{bootmgr}` na frente de tudo a cada colheita, com ou sem alvo declarado, e o ramo brando do `arca restore` nunca silenciou. **O que sobra** é a pergunta do título, e ela fecha com um F12 escolhendo aquela linha em vez da entrada `ARCA` — o que a resposta muda hoje é a dureza de um texto, não o silêncio de um aviso. Ver [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md) e [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md) |
| ~~P-26~~ | ~~Um dispositivo preparado pelo `arca prepare` boota?~~ **Fechada em 24/08/2026, marco da E12**, e **inteira**: `arca sondar` armou às 14:56:55, a máquina bootou, o `lsblk` rodou sozinho e ela desligou; a colheita saiu `concluida` com `ARCA_PROBE=OK` e selo `354da624e7fa0d21`. **As duas metades de uma vez** — (a) o dispositivo boota, e (b) a entrada que o ARCA criou leva a ele, porque ela estava **fora da ordem permanente** e o boot único era o único caminho possível. Um F12 teria respondido só (a). Aberta no marco da E10 |
| ~~P-27~~ | ~~As flags do `lsblk` da sondagem são reconstrução.~~ **Fechada em 24/08/2026, no mesmo marco.** `ARCA_PROBE=OK` diz que aquele util-linux aceitou as sete colunas do `-o`, e a árvore saiu em **ASCII** — `|-sda1`, `` `-sda2 `` —, o que diz que o `-i` foi aceito e produziu a forma do arquivo que ele imita. A **reconstrução** fica no vocabulário do §10.2.2 como terceira procedência, e agora com um caso em que ela deu certo |

**P-26 e P-27 fecharam no marco da E12, em 24/08/2026**, e o mesmo reinício
respondeu as duas:

| # | Fechada | Como |
|---|---|---|
| P-26 | **O dispositivo que o ARCA fez boota, e a entrada que ele criou leva a ele.** | `arca sondar` armado às 14:56:55; a máquina bootou, o `lsblk` rodou sozinho e ela desligou. O `arca-fim.txt` traz `ARCA_SELO=354da624e7fa0d21`, `ARCA_PROBE=OK`, `ARCA_FIM` — 50 bytes, com o selo batendo com o do `estado.json`. **As duas metades de uma vez**, e o que as junta é a leitura de `arca status` de minutos antes: `1 entrada(s), nenhuma para o dispositivo · so o boot unico leva a ele`. Com a entrada fora da ordem permanente não havia outro caminho, e é isso que um F12 não teria respondido |
| P-27 | **As flags reconstruídas do `lsblk` foram aceitas.** | `ARCA_PROBE=OK` diz que o `if` tomou o ramo do êxito, e a **forma** do arquivo diz o resto: a árvore saiu em ASCII (`\|-sda1`, `` `-sda2 ``), que é o que o `-i` compra sobre o `locales=en_US.UTF-8` do boot. O arquivo tem 852 bytes, sete colunas, e é lido pelo mesmo parser que lê o de dentro das imagens |
| P-29 | **Por que o `bcdedit` de 26/08 aceitou a entrada pendurada, e o de 27/08 não?** O `arca prepare` de 26/08 (18:17) apagou um dispositivo ARCA e reapontou a entrada sem que nenhum `/enum` saísse com código; o de 27/08 (19:03) fez o mesmo e **todo** `/enum` passou a sair com 1 (*"Foi especificado um dispositivo inexistente."*). O que difere entre os dois dias, datado pelos GUIDs v1 do BCD: às 18:26 de 27/08 o firmware acrescentou as três `UEFI:*` (CD/DVD, Removable, Network) à ordem, e às 05:32 uma `UEFI:  USB, Partition 1` apontando para um pendrive MBR de 14 GB que não estava conectado. A hipótese é que a sincronização NVRAM↔BCD que o `bcdedit` faz ao abrir o repositório só tropeça na entrada pendurada quando há mais alguma coisa para sincronizar — **não medido**, e C-15 e PR-6 foram escritos para não depender da resposta. O experimento que fecha: reproduzir o estado (entrada apontando para partição apagada) com e sem as `UEFI:*` na ordem, e ler o código do `/enum` nos dois. Ver [ADR-0026](../docs/adr/0026-a-recusa-do-bcdedit-nao-apaga-o-que-ele-listou.md). |

E dois achados de graça, nenhum dos dois pedido:

- **O repositório estava montado no `mkdir`**, e o próprio arquivo testemunha:
  o `sda1` sai com `/home/partimag` no `MOUNTPOINT`. Era o único pressuposto
  genuinamente novo da sondagem, e ele já tinha original na E11 (§10.2.5).
- **O modelo do dispositivo não é o mesmo nas duas fontes.** O `lsblk` o chama
  de `Maxtor Z1 SSD 480GB` e o WMI de `JMicron Generic SCSI Disk Device`: a
  ponte USB responde ao Windows com o nome dela, e o Linux lê o disco atrás
  dela. O disco de **origem** casa nas duas — é o que o backup precisa —, e o
  que fica inerte é a **segunda** barreira de R-8, que resolvia o nome Linux do
  dispositivo pelo mesmo oráculo. Ela não falha errado; só não dispara, e a
  primeira barreira (por letra) continua valendo. O ADR-0015 já previa que ela
  viraria redundante; o que não estava medido é **por que** ela pode ficar
  inerte.

> **E a etapa mediu o que nenhuma outra tinha medido: quanto custa o boot do
> Clonezilla, isolado.** **1 min 40 s** do reinício ao desligamento,
> cronometrado à mão. Tirando os 30 s do `set timeout` do `grub.cfg` e os 20 s
> do `sleep` da receita, sobram **≈ 50 s** para POST, kernel, `initrd`, `toram`
> e o live subir — e esses 50 s são **aritmética sobre um número cronometrado**,
> não uma terceira medição.
>
> O `~2 minutos` que foi dito na mesa antes da etapa ficou acima do total real.
> O que a etapa trocou não foi o número: foi a procedência.

**P-16 e P-18 fecharam no marco em hardware de 22/08/2026**, e as duas ficam
registradas aqui porque a forma como fecharam é o que a próxima etapa precisa
saber:

| # | Fechada | Como |
|---|---|---|
| P-16 | **O mecanismo de desfecho rodou.** | `E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt`, 51 bytes, três linhas: `ARCA_SELO=7d2d2f5153625b38`, `ARCA_BACKUP=OK`, `ARCA_FIM`. O selo bate com o do `estado.json` do mesmo job — conferido a olho, e não só pelo julgamento da E5. S-4, C-11, C-12, R-5 e R-6 rodaram todos de uma vez, e o `if/then/else` tomou o ramo do sucesso. O ramo de falha continua sem rodar (P-6). Cópia em `recursos/capturas/arca-fim-2026-08-22_Apps.txt` |
| P-18 | **O boot foi disparado por boot único.** | `nvram-live-2026-08-22.txt`, escrito pelo `efibootmgr` **durante** aquele boot: `BootCurrent: 0001` com `BootOrder: 0000,0001`. A máquina bootou pela entrada `0001` estando a `0000` à frente. Nem F12 — ninguém tocou na máquina — nem ordem permanente explicam; o `bootsequence` explica. Aberta na E4, estreitada na E7, fechada na E8 ([ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)) |

> **Uma advertência sobre esta seção inteira, e ela mudou de sinal.** Cinco
> vezes se descobriu que algo documentado como fundação validada tinha vindo do
> **trabalho de validação em volta dela**: o `ARCA_VEREDITO=` (ADR-0003), o
> `arca-fim.txt` de 21/08 (ADR-0004), o `set default` (ADR-0005), o `498,7 GB`
> do §5.2 (E6) e a ordem de boot com o dispositivo à frente (E7). O padrão se
> repete porque a evidência que sobra no dispositivo não distingue o que a
> receita escreveu do que uma pessoa escreveu depois.
>
> **Na sexta vez ele não se repetiu.** O `arca-fim.txt` de 22/08 tem original, e
> o original é ele próprio — e o que atesta que a receita o escreveu está ao
> lado, em `ocs-sr-linha-de-comando-2026-08-22.txt`, onde o próprio Clonezilla
> registrou o comando que executou. Continua valendo procurar o original em
> `recursos/capturas/` antes de tratar qualquer linha desta seção como medida;
> o que mudou é que agora há um caso em que ele existe.
>
> **E o marco da E9 trouxe o vizinho do padrão, que é mais difícil de ver.** A
> primeira linha do §3.4 tinha original, e o original era um par de leituras de
> verdade, do mesmo evento, com hora. Nada nele veio de trabalho manual. O que
> estava errado era o **alcance**: aquele par foi lido de dentro do mesmo boot
> do live, e por isso só podia falar do `ocs-sr` — e a linha o apresentava como
> resposta sobre a restauração inteira. Ninguém teria achado isso procurando
> original, porque o original estava lá.
>
> A pergunta que separa os dois casos é outra, e vale a pena carregá-la:
> **entre que dois instantes esta evidência foi tirada, e a pergunta cabe
> dentro deles?** Ver o ADR-0012.

## 4. Estrutura de um dispositivo

```
[dispositivo]  — um SSD externo, duas partições
├── sda2 — FAT32, ~1,5 GB, label ARCABOOT
│     ├── EFI/boot/bootx64.efi
│     ├── live/  (kernel, initrd, filesystem.squashfs)
│     ├── boot/grub/grub.cfg      ← receita, reescrita a cada operação
│     └── arca/                   ← o próprio ARCA e o estado do job
└── sda1 — NTFS, resto, label ARCAVAULT
      ├── ARCA-LOGS/
      ├── 2026-08-21_WindowsCompleto/
      └── ...
```

Os rótulos são **sempre os mesmos** em todo dispositivo. É o que torna a receita reprodutível e os dispositivos intercambiáveis.

**Regra única de operação:** um dispositivo ARCA conectado por vez.

### 4.1 — O ARCA e o estado moram no dispositivo

Não é preferência. A imagem captura o `nvme0n1` — o disco **interno**. O dispositivo é externo, logo **não entra na imagem**.

Consequência: uma restauração substitui o `C:` e devolve, junto, qualquer ARCA que estivesse lá dentro — inclusive versões antigas com defeitos já corrigidos. **O que julga a restauração não pode morar no disco que a restauração substitui.**

Morando no `ARCABOOT`, o ARCA e o `estado.json` sobrevivem a qualquer restauração.

> **E há uma consequência desta seção que só a etapa E9 fez doer.** O
> `%LOCALAPPDATA%\ARCA\arca.log` mora no `C:`, que é o que a restauração
> substitui: **o registro do lado Windows de que o job foi armado é destruído
> pela própria operação.** O `arca.log` que estiver lá depois veio de dentro da
> imagem, e as linhas dele são de outro tempo.
>
> O que sobrevive é o `estado.json` do `ARCABOOT` — que é exatamente o que esta
> seção existe para garantir, e é a única coisa que liga o desfecho ao job
> quando o `arca resultado` roda. A colheita de uma restauração **se vira só
> com ele**, e a tela diz isso em vez de deixar por conta de quem lê (§6.3).

### 4.2 — O ambiente precisa estar fora da imagem

A máquina boota nele **antes** de a imagem ser restaurada. Um ambiente que só existisse dentro da imagem seria inalcançável no momento em que é necessário.

Custo de mantê-lo fora: **zero** — a imagem não engorda por causa dele.

### 4.3 — O selo liga o job ao desfecho

Ao armar, o ARCA gera um identificador aleatório — o **selo** — grava no `estado.json` e o embute na receita. O Clonezilla o devolve na primeira linha do `arca-fim.txt`. Na volta, só é aceito o desfecho cujo selo case com o job pendente.

Isso existe porque **não há relógio comum**: o Clonezilla lê o RTC (hora local do Windows) como UTC e fica 3 h adiantado, permanentemente. Uma trava construída sobre comparação de datas já reprovou um backup perfeito.

O selo resolve quatro casos com um mecanismo só: desfecho de um job anterior, desfecho vindo de dentro de uma imagem antiga (§11, job fantasma), desfecho ausente porque o boot nunca aconteceu, e arquivo truncado por desligamento no meio.

#### O formato, e os três lugares por onde ele passa

Construído na etapa E5. Estava só no código, e o C-11 fala de três lugares sem
dizer que forma tem a coisa que atravessa os três:

| | |
|---|---|
| **Forma** | 16 dígitos hexadecimais **minúsculos** — `a3f1c9e07b2d4856` |
| **De onde vem** | 8 bytes de `BCryptGenRandom` ([ADR-0006](../docs/adr/0006-o-selo-e-o-estado-sem-dependencia-nova.md)) |
| **1 · `estado.json`** | campo `"selo"`, em `ARCABOOT\arca\estado.json` |
| **2 · receita** | `echo ARCA_SELO=<selo> > .../arca-fim.txt`, o **primeiro** passo que escreve |
| **3 · `arca-fim.txt`** | `ARCA_SELO=<selo>` na **primeira linha, e só nela** |

Minúsculas, e não `hexdigit` genérico: um selo que mudasse de caixa entre o
`estado.json` e o `arca-fim.txt` deixaria de casar, e casar é a única coisa que
o selo faz. **Primeira linha e só ela** porque a receita o escreve com `>`, que
trunca — num arquivo que a receita produziu, ele não pode estar em outro lugar,
e aceitar um selo achado no meio faria o rastro de dois jobs passar pelo
segundo deles.

Mudar essa forma obriga a mexer nos três lugares de uma vez, como o
[ADR-0001](../docs/adr/0001-selo-liga-job-ao-desfecho.md) já avisava. Dois deles
são código deste lado do reinício, e eles **compartilham a constante**
`MARCA_DO_SELO`: `src/receita.rs` a declara e escreve, `src/desfecho.rs` a
importa e lê. O terceiro é o `arca-fim.txt` que o Clonezilla produz a partir da
receita, do outro lado. O `src/estado.rs` guarda o selo como valor e não conhece
o marcador — quem quiser mudar a forma mexe nos dois primeiros.

**O momento do armar não pertence a este mecanismo.** Ele está no
`estado.json` e é informativo: nunca é comparado com nada escrito pelo Linux
(S-6). O tipo que o carrega guarda texto e não deriva ordenação, para que a
violação exija uma linha deliberada em vez de um descuido —
`tests/s6_o_tempo_nao_decide.rs` cobra isso a cada build.

**O `estado.json` tem seis campos desde a etapa E8**, e o sexto é `situacao`:
`armado` ou `colhido`. Colher o desfecho encerra o job marcando esse campo, e
não apagando o arquivo — o `estado.json` é o único lugar que liga um selo a um
nome de imagem, e apagá-lo faria "job fantasma" virar a resposta para tudo. Ele
**não é uma data** de propósito: dois instantes lado a lado num arquivo cujo
tipo de tempo existe para tornar a comparação difícil seriam um convite a
subtraí-los. Ver
[ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md).

### 4.4 — O estado inerte

O documento pressupunha este estado sem nunca defini-lo. O §6.4 conta com ele
("boote pelo dispositivo com F12 e use o menu do Clonezilla"), e o §5.2 e o §5.4
mostram `Desarmando ... ok` sem dizer o que é desarmar. Definido na etapa E4:

> **Estado inerte** é o `grub.cfg` do dispositivo **sem** nenhum
> `menuentry --id arca-backup` e com `set default="live-default"`, e o
> `{fwbootmgr}` **sem** `bootsequence`.

Um dispositivo inerte boota no menu normal do Clonezilla e fica esperando
alguém. Um dispositivo armado executa a receita e desliga sozinho.

**Desarmar é levar o dispositivo a esse estado**, incondicionalmente e sem
consultar estado nenhum (C-1). O estado inerte não é uma cópia guardada em
lugar nenhum: é o que sai de aplicar essa regra ao `grub.cfg` que está no
dispositivo. Isso o torna idempotente de graça — a segunda passada não acha
bloco e encontra o `set default` já no lugar — e faz o desarmar funcionar num
dispositivo que o ARCA nunca viu, inclusive num armado à mão.

**A definição é verificável sem reiniciar**, que é o que permite a E4 fechar
sem um marco em hardware: o `grub.cfg` reescrito ou sai byte a byte igual ao
inerte conhecido, ou não sai.

Ver [ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)
para por que `live-default` e não `0`, e para os dois caminhos descartados —
embutir a cópia no binário e guardá-la no dispositivo.

### 4.5 — O nome do disco de origem vem de dentro de uma imagem

Construído na etapa E6. O documento nunca disse de onde sai o `nvme0n1` que a
receita nomeia, e o problema é maior do que parece: **`nvme0n1` é o nome do
disco no Linux, e o Windows não o conhece.** Nenhuma API do Windows responde
por ele.

O que existe para ligar os dois lados:

| Fonte | O que diz | Medido? |
|---|---|---|
| `blkdev.list` **dentro de cada imagem** | `nvme0n1` ↔ `KINGSTON SNV3S500G`, `sda` ↔ `KGSSE100256` | **Sim** — conferido nas duas imagens do dispositivo |
| `Win32_DiskDrive` do WMI | o disco onde o `C:` mora é `KINGSTON SNV3S500G`, índice 0 | **Sim** |
| `BusType` + `Index` → `nvmeNn1` | plausível | **Não** |

**O ARCA usa as duas primeiras e recusa a terceira.** O modelo é a chave: o
WMI diz qual disco tem o `C:`, o `blkdev.list` diz que nome o Linux dá àquele
modelo, e o nome sai de uma medição. A derivação por índice ficou de fora
porque **o índice do Windows não é o do Linux por construção** — aqui os dois
coincidem porque a máquina tem um NVMe só, e numa com dois um `nvme1n1` viraria
`nvme0n1` e a receita nomearia o disco errado.

O preço é que o oráculo **só existe depois do primeiro backup**. Não havendo
imagem de onde ler, o nome fica *por determinar*, e o pré-voo diz isso com
todas as letras em vez de chutar. Isso é uma resposta, e não uma falha a
contornar: este documento já registrou cinco vezes (os ADRs 0003, 0004, 0005, o
`498,7 GB` da E6 e a ordem de boot da E7) que chamar de fundação validada o que
veio do trabalho de validação em volta dela é o erro que mais custou neste
projeto. Inventar uma derivação e documentá-la como descoberta seria a sexta.

**Sem nome determinado, `arca backup` recusa — e não pergunta.** Decidido na
etapa E7, que era quem tinha de decidir. Pedir o nome ao usuário parece o
caminho gentil e é o pior dos dois: `nvme0n1` é um nome do **Linux**, quem o
digitaria está no Windows, e não há nada deste lado contra o que conferi-lo. Um
`nvme1n1` digitado por engano passaria por bom, entraria na receita e nomearia
o disco errado — numa receita que na E9 é destrutiva. O `blkdev.list` tem
oráculo; um valor digitado não tem nenhum.

A recusa acontece **antes** da confirmação digitada, para que ninguém digite o
nome inteiro da imagem para ouvir um não depois.

**A saída sempre diz de onde o nome veio** — `nvme0n1 · lido de
2026-08-21_WindowsCompleto/blkdev.list, casando o modelo …`. Uma receita
destrutiva que nomeie um disco sem dizer a origem do nome é pior do que não
imprimir nada.

#### A segunda fonte, e ela não depende de imagem nenhuma

**O custo do oráculo só existir depois do primeiro backup era maior do que esta
seção deixava parecer**, e isso só apareceu quando o ARCA passou a **criar**
dispositivos, na E10: num dispositivo recém-nascido não há imagem, logo não há
nome, logo **nenhum dos três comandos que armam funciona** — `arca backup` pela
razão acima, `arca restore` e `arca verify --completo` porque não há imagem para
restaurar nem para verificar. O texto anterior mandava fazer o primeiro backup
uma vez pelo menu do Clonezilla (§6.4), que é exatamente aquilo que este app
existe para não precisar.

**A etapa E12 dá uma segunda fonte para o mesmo arquivo.** `arca sondar` (§9.7)
arma um boot único que roda `lsblk`, grava a saída no `ARCAVAULT` no mesmo
formato e desliga — um reinício, nenhuma tela do Clonezilla. **O parser não
muda**: `crate::blkdev` continua sendo o único lugar que lê aquele formato, e o
arquivo leva o mesmo nome.

| Fonte | O que descreve | Quando existe |
|---|---|---|
| `blkdev.list` **de dentro de uma imagem** | a máquina de **quando o backup foi feito** | depois do primeiro backup |
| `blkdev.list` **da sondagem** | a máquina de **agora** | depois de um `arca sondar` |

**Havendo as duas, a sondagem ganha** — e a divergência é dita na tela, nunca
resolvida em silêncio (SD-5). Um disco trocado entre o backup e hoje faz a
imagem nomear um disco que não está mais lá, e a sondagem sabe disso. A defesa
velha continua embaixo: o casamento é por **modelo**, e uma sondagem obsoleta que
descrevesse outro disco cai em recusa, não em palpite.

A saída também diz **quando** a sondagem foi feita — `nvme0n1 · lido da sondagem
de 23/08 21:14, casando o modelo …` —, porque uma sondagem de um mês atrás pode
estar descrevendo uma máquina que mudou. É informativo, e nunca comparado (S-6).

## 5. Fluxo: backup

### 5.1 — O que o usuário faz

| # | Ação |
|---|---|
| 1 | Conectar o dispositivo |
| 2 | `arca backup <nome>` |
| 3 | Confirmar digitando |
| 4 | *(esperar — pode sair de perto)* |
| 5 | **Remover o SSD antes de religar** |
| 6 | Ligar a máquina |
| 7 | `arca resultado` (com o SSD reconectado) |

> **O passo 5 não é zelo.** Após restauração seguida de `poweroff`, o boot seguinte foi para o dispositivo removível, apesar de não haver `bootsequence` pendente. Causa não determinada, não reproduzido. Remover o SSD elimina o cenário.

### 5.2 — Diálogo

Executado de verdade em 22/08/2026, às 20:53:48 — a tela inteira, incluindo o
que vem depois da confirmação. Foi este comando que disparou o marco em
hardware:

```
> arca backup 2026-08-22_Apps

Dispositivo ARCA: ARCAVAULT (E:) · 164 GB livres
Origem: KINGSTON SNV3S500G · 465,8 GB · 105,9 GB em uso
Imagem estimada: ~47,7 GB · espaco suficiente
Imagem: 2026-08-22_Apps

  Desarmando receita anterior ..... ok · ja estava inerte · R:\boot\grub\grub.cfg
  Inicializacao rapida ............ desativada   ok
  chkdsk /scan .................... limpo        ok
  Nome disponivel ................. ok
  Disco de origem ................. nvme0n1 · lido de 2026-08-21_WindowsCompleto/blkdev.list, casando o modelo `KINGSTON SNV3S500G`

Pre-voo concluido, e o dispositivo esta inerte. Nada foi armado ainda —
o ponto sem volta e a confirmacao abaixo.

Digite o nome do backup para confirmar: 2026-08-22_Apps

  Entrada de firmware ............. ARCA · {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34} · partition=R:
  Receita armada .................. ok · R:\boot\grub\grub.cfg
  Boot unico ...................... ok · relido no bcdedit · {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
  Selo do job ..................... 7d2d2f5153625b38
  Desfecho esperado em ............ E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt

A maquina vai reiniciar agora e desligar sozinha ao terminar.
AO TERMINAR: remova o SSD antes de religar.

Reiniciando...
```

> **A ordem das três últimas linhas mudou na etapa E7, e a mudança é o
> requisito.** A versão anterior mostrava o aviso de C-9 **antes** da
> confirmação digitada, e por isso não podia mostrar mais nada depois dela: o
> documento supunha que confirmar e reiniciar eram a mesma coisa. Não são —
> entre os dois está o ponto sem volta, e ele tem cinco linhas.
>
> **A confirmação vem antes de qualquer escrita** (S-2), e o aviso de C-9 vem
> **depois de armado e antes de reiniciar**, que é onde ele é a última coisa
> que alguém lê. As cinco linhas do meio são a releitura de C-3: cada uma delas
> é algo que o ARCA mandou fazer e **conferiu** perguntando de novo, porque o
> sucesso do `bcdedit` nunca é prova.
>
> `Selo do job` aparece na tela porque é ele que a colheita vai cobrar do
> `arca-fim.txt`. Um selo que só existisse dentro do `estado.json` não daria a
> ninguém como conferir, à mão, se o desfecho que voltou é deste job. **E foi
> conferido assim**: o `7d2d2f5153625b38` desta tela e o da primeira linha do
> `arca-fim.txt` que voltou são a mesma cadeia, lida a olho antes de qualquer
> conclusão sobre o marco.
>
> **As cinco linhas do meio só existiram numa execução real na etapa E8**, e
> uma delas estava errada aqui. A revisão da E7 tinha corrigido o código para
> mostrar o caminho inteiro do desfecho — `backup-2026-08-22_Apps` sozinho não
> é um lugar, nada ali diz que aquilo mora sob `ARCA-LOGS\` no `ARCAVAULT` —, e
> esta tela ficou com a versão antiga. É a lição da E6 e da E7 pela terceira
> vez: **depois de corrigir, releia o que a correção encostou.** Desta vez o
> que ficou para trás não foi um rodapé vinte linhas abaixo, foi o documento.

> **Os números desta tela eram de outra máquina, e três estavam errados.**
> Corrigidos na etapa E6, contra medição de 22/08/2026:
>
> | O que dizia | O que é | De onde vinha o erro |
> |---|---|---|
> | `498,7 GB` de disco | **465,8 GB** | `498.701.697.024` é o tamanho da **partição `C:`**, em base 1000, apresentado como o do disco. O disco tem `500.105.249.280` bytes, que são 465,8 GiB — e `src/formato.rs` usa base 1024 por decisão medida |
> | `92 GB em uso` | **105,6 GB** | Não era erro, era tempo passando |
> | `183 GB livres` | **164 GB** | Idem |
>
> O primeiro merece nome: não é um número inventado, é um número **medido na
> coisa errada**. Quem o repetir vai errar de novo pelo mesmo caminho — o
> tamanho do volume do sistema não é o tamanho do disco, e a diferença são as
> outras três partições.
>
> **A linha do desarmar diz o que de fato aconteceu**, e não só `ok`. Ela
> distingue "já estava inerte" de "havia receita armada", e no `--dry-run` diz
> que não desarmou — um `ok` sobre uma ação que não aconteceu é a mesma mentira
> que o `--dry-run` deste projeto já contou uma vez (§11).
>
> **O caminho continua na linha**, e o motivo original acabou: a E6 enumera
> discos físicos e prova que o `ARCAVAULT` e o `ARCABOOT` estão no mesmo, o que
> fecha a brecha de C-10 recusar rótulo **repetido** e não rótulo órfão. O
> caminho fica porque continua sendo útil ver em que disco se mexeu.
>
> **A linha `Receita validada` saiu.** Ela pertence ao momento de armar, e não
> ao pré-voo: a receita é montada com o nome e o disco já decididos, e no
> pré-voo o disco pode nem ter nome. O que a substitui é `Disco de origem`,
> que diz **de onde o nome do disco veio** — ver §4.5.

### 5.3 — O que acontece sem intervenção

Firmware carrega o Clonezilla → monta `LABEL=ARCAVAULT` em `/home/partimag` → executa a receita → grava → verifica → escreve o veredito em arquivo → desliga.

**Zero telas.**

### 5.4 — Ao voltar

Executado de verdade em 22/08/2026, às 21:14:49, colhendo o primeiro backup que
o ARCA disparou. **Transcrição conferida contra a tela original em 23/08/2026,
e ela bate linha a linha** — o que estava escrito aqui era o que saiu, e não a
lembrança de quem escreveu:

```
> arca resultado

Backup 2026-08-22_Apps
  22/08 · 39,7 GB
  Desfecho: concluida — o selo bate e a receita chegou ao fim
  Verificacao: APROVADA
  Selo: 7d2d2f5153625b38

  Desarmando SSD .................. ok · R:\boot\grub\grub.cfg
  Job ............................. encerrado · o desfecho foi lido e dito

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

125 GB livres
```

> **Esta transcrição é de 22/08 e ficou como estava, de propósito: falta nela
> uma linha que o comando de hoje imprime.** C-13 nasceu na E10, em 23/08, e
> acrescentou `Ordem de boot` logo abaixo de `Job`:
>
> ```
>   Ordem de boot ................... devolvida · o Windows voltou ao topo, na frente de ARCA · {f4057bd0-…}
> ```
>
> Enfiá-la no bloco acima faria a transcrição afirmar que aquela execução a
> imprimiu, e ela não imprimiu — é a mesma distinção entre reprodução e captura
> que a E8 pagou para manter. Com o Windows já em primeiro a linha sai
> `ok · o Windows ja era o primeiro`, e o parágrafo de conselho não aparece.
> A forma acima é execução real de 23/08/2026, do caminho "já colhido".

> **O `Desfecho` e a `Verificacao` são duas linhas, e S-5 é o motivo.** A
> versão anterior desta tela mostrava só a verificação, o que faria um backup
> com `ARCA_BACKUP=OK` e imagem **reprovada** sair parecendo um problema de
> verificação — quando é uma falha da operação inteira. Os dois sinais são
> independentes (§4.3, ADR-0003) e nenhum pode esconder o outro. Quando os dois
> não estão bons, o comando ainda imprime a tela inteira e **sai com código
> diferente de zero**: quem chamou o ARCA de um script não pode ler um desfecho
> ruim como êxito.
>
> **`Job: encerrado` é a linha que fecha o par que a etapa E5 deixou aberto.**
> Colher encerra o job — o `estado.json` é marcado como colhido, e nunca
> apagado (B-10 não precisa ser discutido, e o registro que liga o selo ao nome
> sobrevive). Depois disto, `arca status` diz "já colhido, nada esperando" em
> vez de "job por colher" ao lado de um boot único não armado. Ver
> [ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md).
>
> A linha diz `CONTINUA PENDENTE` no único caso em que o job **não** se
> encerra: o `arca-fim.txt` está lá e não se deixou ler. "Não consegui olhar"
> não é veredito, e encerrar ali perderia o selo que liga o desfecho ao job.
>
> **Os números desta tela são de uma execução real, e três mudaram.** Ela
> esperava o marco em hardware; ele veio em 22/08/2026. O que a execução
> corrigiu:
>
> | O que dizia | O que é | Por quê |
> |---|---|---|
> | `36,2 GB` de imagem | **39,7 GB** | Era o tamanho da imagem de 21/08, repetido. O pré-voo estimou `~47,7 GB` para 105,9 GB em uso, e a imagem saiu em 39,7 GB — a estimativa é conservadora, e é assim que ela serve |
> | `164 GB livres` | **125 GB** | A imagem de 39,7 GB acabou de entrar no dispositivo |
> | duas imagens na lista | **três** | `ARCA-TESTE-03` estava no `ARCAVAULT` e a tela do documento não a trazia |
>
> A `ARCA-TESTE-03` na lista não é detalhe: ela é a evidência de que a listagem
> mostra **o que está no dispositivo**, e não o que a operação acabou de
> escrever. Uma tela montada de memória tende a trazer só a última.

### 5.5 — Desfechos possíveis

Vale para backup e para restauração. Nenhuma linha desta tabela é silêncio: toda combinação tem mensagem própria.

| O que se encontra | Significado | O que o ARCA faz |
|---|---|---|
| Selo bate, `ARCA_FIM` presente, desfecho `OK` | Operação concluída | Mostra o veredito da imagem |
| Selo bate, desfecho `FALHOU` | O Clonezilla falhou e disse | Reporta falha e aponta o log |
| Selo bate, sem `ARCA_FIM` | Truncado — desligamento no meio | Falha; a pasta é resíduo (B-3) |
| Selo não bate | Job fantasma | Ignora o arquivo e avisa |
| **`arca-fim.txt` sem linha de selo, com selo repetido, ou sem marcador de desfecho** | **Não é desfecho de job nenhum do ARCA** — anterior ao mecanismo, escrito por outra coisa, ou cortado antes de a primeira linha existir | **Recusa nomeando qual dos três. Nunca diz "o selo não bate": não há selo a bater** |
| Sem `arca-fim.txt`, com job pendente | O boot não aconteceu, ou o Clonezilla abriu menu | Falha, nomeando as duas causas |
| Sem `arca-fim.txt`, sem job pendente | Não há nada a colher | Diz isso e para |

> **A primeira linha saiu do papel em 22/08/2026, e a tabela não ganhou
> nenhuma.** O marco em hardware da E8 produziu exatamente o primeiro caso —
> selo batendo, `ARCA_FIM` presente, `ARCA_BACKUP=OK`, veredito `APROVADA` — e
> o `arca resultado` o classificou nele. Uma execução real que não obriga a
> mexer na tabela é o resultado que se queria, e vale dizer que foi conferido.

> **A segunda linha saiu do papel em 24/08/2026, e ela é o primeiro `FALHOU`
> deste projeto.** Uma sondagem foi armada com uma coluna inventada no `lsblk`
> — `FLAGQUENAOEXISTE` —, e o dispositivo voltou com:
>
> ```text
> ARCA_SELO=95772dae07463701      lsblk: unknown column: FLAGQUENAOEXISTE
> ARCA_PROBE=FALHOU               (no proprio blkdev.list, por causa do 2>&1)
> ARCA_FIM
> ```
>
> O `arca resultado` classificou na linha `selo bate, desfecho FALHOU`, reportou
> a falha, apontou o arquivo que tem a causa e **saiu com código 1** (S-5). O
> `if/then/else` de R-5 existe desde a E3 e só tinha rodado no ramo do êxito, em
> cinco execuções.
>
> **Isso não fecha P-6**, e a distinção importa: a pergunta de lá é se o
> **`ocs-sr`** devolve código diferente de zero ao falhar, e nenhuma resposta do
> `lsblk` fala por ele. O que ficou provado é a **forma** — o `if` de R-5
> funciona nos dois ramos em hardware, e o `arca resultado` sabe imprimir um
> desfecho ruim. Ver [ADR-0019](../docs/adr/0019-a-sondagem-e-a-quarta-operacao.md).
>
> **Faltam cinco linhas**, e a mais valiosa continua sendo o `FALHOU` de uma
> operação que grava — que é P-6 com a roupa cara.

> **A linha do "sem selo" nasceu na etapa E5, e ela não estava aqui.** A tabela
> tinha *"selo não bate"* e tinha *"sem `ARCA_FIM`"*, e o único `arca-fim.txt`
> que existe neste dispositivo não é nem um nem outro: ele tem `ARCA_FIM`, tem
> `ARCA_RESTORE=OK` e **não tem `ARCA_SELO=` nenhum**. Vinte e cinco bytes,
> duas linhas, conferido em 22/08/2026. É P-16 outra vez — ele veio do trabalho
> manual de validação, e não de receita alguma.
>
> Dizer *"o selo não bate"* sobre esse arquivo seria mentira, e é o ramo que um
> leitor tomaria por descuido: comparar o selo achado com o esperado, achando
> vazio, e reportar divergência.
>
> **Aquele arquivo é inalcançável pelo ARCA de hoje**, e isso é verdade: a E3
> decidiu que a pasta do log leva a operação no nome (`restauracao-<nome>`), e
> ele está em `ARCA-LOGS\2026-08-21_WindowsCompleto\`. Mesmo assim a linha vale
> código, porque **"sem selo" é alcançável por outro caminho**: toda receita
> começa com `echo ARCA_SELO=... > arca-fim.txt`, e o `>` **trunca ao abrir**,
> antes de o `echo` rodar. Medido em bash: um redirecionamento que abre e não
> escreve deixa o arquivo em zero byte. Um desligamento nessa janela produz
> exatamente um `arca-fim.txt` sem linha de selo — o caso que o §4.3 diz que o
> selo cobre, com o selo sendo justamente o que foi cortado.

## 6. Fluxo: restauração

### 6.1 — Windows funcionando

Construída na etapa E9. Tudo até a confirmação é execução real desta máquina,
em 23/08/2026:

```
> arca restore

Dispositivo ARCA: ARCAVAULT (E:) · 125 GB livres

  Desarmando receita anterior ..... ok · ja estava inerte · R:\boot\grub\grub.cfg

Imagens em ARCAVAULT:
  [1] 2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  [2] 2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  [3] ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

Qual restaurar? 2

  Imagem escolhida ................ 2026-08-22_Apps
  Origem da imagem ................ KINGSTON SNV3S500G · nvme0n1 · 976773168 setores de 512 B · 465,8 GB
  Destino ......................... KINGSTON SNV3S500G · disco 0 do Windows · nvme0n1 · 976773168 setores de 512 B · 465,8 GB
  Cabe (R-7) ...................... ok · o destino tem exatamente o tamanho da origem
  Conferido contra a imagem ....... ok · `disk`, `nvme0n1-gpt.sgdisk` e `blkdev.list`
  Imagem criada por ............... /usr/sbin/ocs-sr -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true savedisk 2026-08-22_Apps nvme0n1

ATENCAO: a restauracao APAGA o disco de destino.
Tudo que estiver nele sera perdido.

Digite o nome da imagem para confirmar: 2026-08-22_Apps

  Entrada de firmware ............. ARCA · {f4057bd0-…} · partition=R:
  Receita armada .................. ok · R:\boot\grub\grub.cfg
  Boot unico ...................... ok · relido no bcdedit · {f4057bd0-…}
  Selo do job ..................... <16 digitos hexadecimais>
  Desfecho esperado em ............ E:\ARCA-LOGS\restauracao-2026-08-22_Apps\arca-fim.txt

A maquina vai reiniciar agora e desligar sozinha ao terminar.
AO TERMINAR: remova o SSD antes de religar.

  E REMOVER O SSD NAO E ZELO NESTA OPERACAO. O dispositivo esta em
  PRIMEIRO na ordem permanente de boot: enquanto ele estiver conectado,
  todo reinicio boota nele — sem boot unico nenhum. Entre o fim da
  restauracao e o `arca resultado` o `grub.cfg` continua armado, e um
  reinicio nessa janela RESTAURA DE NOVO, por cima do Windows que acabou
  de voltar. Foram oito minutos no backup de 22/08.
  O ARCA nao mexe na ordem permanente (C-5, ADR-0009): ele lê e avisa.
  Remova o SSD ao desligar, religue, e so entao reconecte para
  `arca resultado`.

Reiniciando...
```

A escolha acontece **no Windows**, com a lista à vista. O Clonezilla executa sem perguntar nada.

> **O `498,7 GB` desta tela era a sexta vez do mesmo número.** A linha
> `Destino: KINGSTON SNV3S500G · 498,7 GB` trazia o tamanho da partição `C:`
> apresentado como o do disco — o mesmo erro que a E6 corrigiu no §5.2 e que
> sobreviveu aqui, porque ninguém releu o §6 ao corrigir o §5. Este disco tem
> **465,8 GB**, e `src/comandos/restore.rs` tem um teste que reprova qualquer
> tela onde `498,7` reapareça.
>
> **A tela mostra os dois discos em setores, e não só em GB**, e isso não é
> excesso: é a comparação de R-7 impressa em vez de resumida. Quem está
> prestes a apagar um disco tem de poder refazer a conta — e os dois números
> saírem da **mesma régua** é o achado que custou a etapa
> ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)).
>
> **São duas leituras do usuário, e elas não são redundantes.** O índice
> **escolhe** — apontar numa lista, e um número é a forma mais curta de
> apontar. O nome por extenso **confirma** (R-3, S-2), e existe justamente para
> custar o trabalho de ler e digitar. Trocar a segunda pela primeira faria um
> `2` apagar um disco.
>
> **Resíduo nunca ganha número** (L-2). A lista do §6.1 diverge da do §5.4 de
> propósito: aquela responde "o que há no dispositivo" e mostra resíduo
> marcado; esta responde "o que dá para restaurar". Um número ao lado de um
> resíduo seria um número que não se pode digitar — e, pior, ele ocuparia um
> índice, e aí os números passariam a depender de coisas não escolhíveis. Os
> resíduos aparecem **nomeados e sem número**, embaixo da lista: omiti-los
> faria a lista parecer incompleta para quem sabe que há outra pasta ali.
>
> **A linha do desarmar sai antes da lista, e isso é C-1 na letra.** O desarmar
> acontece incondicionalmente como primeiro passo, e uma recusa posterior —
> destino errado, imagem inexistente — não pode engolir a notícia de que ele
> aconteceu. Foi o defeito que a revisão da E7 pegou no `arca backup`, cometido
> de novo na E9 e achado **rodando o comando de verdade**, e não lendo o
> código.
>
> **As cinco linhas depois da confirmação e o aviso final rodaram em
> 23/08/2026, e são a única parte desta tela sem captura.** Elas foram
> impressas de verdade — o job foi armado às 11:10:50, e o `estado.json` que
> sobrou registra selo, comando, nome, disco e momento. O que não sobrou foi a
> tela: **a sessão que a imprimiu morreu no reinício que ela mesma disparou**, e
> o `arca.log` que teria a linha do armar mora no `C:` e foi destruído pela
> própria restauração (§4.1).
>
> É o mesmo caso do `grub.cfg` armado que a E8 registrou. O código as reproduz
> de forma determinística, e **reprodução não é captura**. O texto acima é o
> que o código monta; o que atesta que a operação aconteceu está ao lado dele,
> em `recursos/capturas/`, e é o desfecho com o mesmo selo do `estado.json`.

### 6.2 — O que a imagem carrega, e o que R-2 confere

Reescrito na etapa E9, abrindo a `E:\2026-08-22_Apps` e lendo o que está lá. O
requisito falava de dois arquivos; há mais, e todos escritos pelo Clonezilla:

| Arquivo | O que traz |
|---|---|
| `disk` | `nvme0n1` — oito bytes, o nome Linux do disco de origem |
| `parts` | os quatro `nvme0n1pN` |
| `blkdev.list` | o `lsblk`, com `SIZE` e `MODEL` de cada disco daquele boot |
| `nvme0n1-gpt.sgdisk` | **setores totais, tamanho do setor e modelo** |
| `nvme0n1-pt.sf` | o `sfdisk`: `last-lba`, `sector-size`, cada partição |
| `nvme0n1-chs.sf` | `cylinders=476940 heads=64 sectors=32` |
| `Info-saved-by-cmd.txt` | **a linha de comando que criou a imagem** |
| `Info-img-size.txt` | `40G` |
| `efi-nvram.dat` | a NVRAM de dentro daquele boot (§3.1) |
| `clonezilla-img` | a versão do Clonezilla e o log inteiro do `savedisk` |

**R-2 confere quatro coisas, e as quatro têm original na imagem:**

1. **`disk`** existe e nomeia o disco de origem.
2. **`<disco>-gpt.sgdisk`** dá o tamanho da origem, em setores e com o tamanho
   do setor. É a única medida da origem que existe do lado Windows, e é o que
   torna R-7 respondível.
3. **`disk` contra `sgdisk`**, e **`blkdev.list` contra `sgdisk`**: duas fontes
   independentes da mesma pasta dizendo que disco foi retratado, e que modelo
   ele era. Discordarem é **recusa**, e não escolha — escolher entre duas
   fontes é adivinhar num comando que apaga um disco.
4. **O modelo do destino** contra o da origem. Iguais, é o caso normal;
   diferentes, é R-7, e a confirmação nomeia os dois.

O `Info-saved-by-cmd.txt` **não confere nada** — é procedência, e vai para a
tela. Ele é o primeiro original de um comando que o ARCA gerou: em
`2026-08-22_Apps` ele traz, literalmente, o `ocs-sr` de B-8 na ordem de B-8,
escrito pelo próprio Clonezilla.

> **As duas geometrias falsas.** O `nvme0n1-chs.sf` da imagem registra
> `cylinders=476940 heads=64 sectors=32`, e o Windows reporta `255/63` para o
> mesmo disco. São dois CHS inventados por ferramentas diferentes, e nenhum
> deles é o do hardware — discos modernos não têm CHS. **Nada no ARCA usa
> nenhum dos dois**, e é justamente por isso que a comparação de R-7 sai em
> setores: `-e1 auto -e2` existe na receita para o Clonezilla acertar a
> geometria sozinho (R-4), e o §10.2.2 registra que os dois vieram da única
> restauração que deu certo. O que **não** pode acontecer é um deles entrar
> numa conta — e o `Win32_DiskDrive.Size`, que é o produto do CHS do Windows
> truncado, já entrava (ADR-0010).

### 6.3 — Ao voltar de uma restauração

O mesmo `arca resultado` do §5.4, e a saída muda em três coisas. **Execução
real desta máquina, em 23/08/2026 às 11:50:53** — a colheita do marco da E9:

```
> arca resultado

Restauracao 2026-08-22_Apps
  22/08 · 39,7 GB
  Desfecho: concluida — o selo bate e a receita chegou ao fim
  Imagem de origem: APROVADA — veredito do backup que a criou, e nao desta operacao
  Selo: ce04819cf0ee96f7

  Desarmando SSD .................. ok · R:\boot\grub\grub.cfg
  Job ............................. encerrado · o desfecho foi lido e dito

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

125 GB livres

  A RESTAURACAO TERMINOU, e o `OK` acima vem de UM sinal so. Num backup o
  ARCA tem dois — a conferencia nativa do Clonezilla e o `ocs-chkimg` de
  B-9 —, e aqui nao ha nada depois do `ocs-sr` para desmenti-lo (P-6). O
  juiz que falta e o Windows subir: religue e confira.
  O log do Clonezilla desta operacao esta em
  ARCA-LOGS\restauracao-2026-08-22_Apps\arca-restore.log, no ARCAVAULT, que
  a restauracao nao tocou.
  O `arca.log` do lado Windows foi DESTRUIDO por esta operacao: ele mora
  em %LOCALAPPDATA%\ARCA, no C:, que e o que a imagem substituiu. O que
  estiver la agora veio de dentro da imagem, e e de outro tempo. Quem
  sobreviveu foi o `estado.json` do ARCABOOT, e e para isto que §4.1
  existe.

  O dispositivo ja foi desarmado acima, e com isso fechou a janela em que
  um reinicio com o SSD conectado restauraria de novo (ADR-0009).
```

**Falta aqui a mesma linha que falta no §5.4**, e pela mesma razão: esta
transcrição é de 23/08 às 11:50, e C-13 entrou horas depois. Numa restauração
ela sai quase sempre `ok · o Windows ja era o primeiro`, porque a operação já
devolveu a ordem sozinha (ADR-0012) — o conserto de C-13 é sobre o **backup**.

**E as duas telas existem de verdade, lado a lado.** A do §5.4 saiu às 21:14:36
de 22/08 e esta às 11:50:53 de 23/08 — mesmo dispositivo, catorze horas e uma
operação destrutiva entre elas. O que as separa é o que está abaixo, e o que
**não** as separa vale igual:

| | Backup, 22/08 21:14 | Restauração, 23/08 11:50 |
|---|---|---|
| Título | `Backup 2026-08-22_Apps` | `Restauracao 2026-08-22_Apps` |
| Veredito | `Verificacao: APROVADA` | `Imagem de origem: APROVADA — veredito do backup que a criou, e nao desta operacao` |
| Selo | `7d2d2f5153625b38` | `ce04819cf0ee96f7` |
| Conselho | nenhum | os três do §6.3 |
| Desarmar, `Job`, listagem, espaço | **idênticos** | **idênticos** |

A última linha é a que ninguém tinha pedido: **as três imagens saem com os
mesmos tamanhos e os mesmos vereditos, e o espaço livre exibido é o mesmo.** A
restauração leu 39,7 GB do `ARCAVAULT` e escreveu 16 KB nele — o
`arca-restore.log` e o `arca-fim.txt` —, o que não move um número em GB. É a
frase *"no ARCAVAULT, que a restauracao nao tocou"* deixando de ser afirmação e
virando observação.

**As três diferenças, e todas vêm do mesmo lugar: numa restauração a pasta é a
imagem de *origem*, e não o que a operação produziu.**

1. **`Verificacao:` vira `Imagem de origem:`.** O veredito é do backup que
   criou aquela imagem, dias antes e sobre outra coisa. Chamá-lo de
   verificação faria quem lê concluir que a restauração foi verificada.
2. **Ele deixa de reprovar a operação.** Até a E9, `julgar_o_conjunto` reprovava
   um desfecho `OK` cuja pasta não tivesse veredito — e numa restauração isso
   estava errado do jeito mais caro: uma imagem trazida de outro dispositivo,
   ou verificada por `arca verify` em vez de por B-9, não tem `arca-check.log`,
   e uma restauração bem-sucedida a partir dela saía relatada como falha. S-5
   continua valendo; o que muda é qual é o segundo sinal, e **na restauração
   não há um**.
3. **O conselho diz as três coisas que só se sabem aqui**: que o `OK` veio de um
   sinal só (P-6), que o `arca.log` do lado Windows foi destruído pela operação
   (§4.1), e que o desarmar acima fechou a janela do ADR-0009 — que numa
   restauração é destrutiva.

> **O terceiro conselho estava certo, e a execução real mostrou que ele é
> curto.** O `arca.log` foi mesmo destruído, e dá para ver exatamente onde: a
> última linha do lado de lá é de 22/08 às 20:53:48 — o armar do **backup** —, e
> a seguinte já é a desta colheita. Sumiu no meio, entre outras coisas, **a
> linha do armar desta própria restauração**, escrita quarenta minutos antes.
> A operação apaga o registro de que ela foi armada.
>
> Quem lê a tela não precisa disso para agir, e por isso o conselho fica como
> está. Mas é a diferença entre *"o arquivo lá é de outro tempo"* e *"o arquivo
> lá não tem esta operação"*, e a segunda é a que explica por que o
> `estado.json` do `ARCABOOT` não é redundância. Medido em
> `recursos/capturas/arca-log-windows-2026-08-23-pos-restauracao.txt`.

> **E o segundo juiz apareceu, pelo caminho que a tela manda usar.** *"O juiz
> que falta é o Windows subir: religue e confira"* — o Windows subiu, e este
> documento está sendo editado nele. P-6 continua aberta, porque um êxito não
> exercita o ramo de falha; o que a operação fecha é a dúvida sobre esta
> restauração, e não sobre o mecanismo.

### 6.4 — Windows não boota

Não há `arca restore` a rodar. Boote pelo dispositivo com F12 e use o menu do Clonezilla. O ambiente está lá, íntegro, porque nunca esteve dentro da imagem.

## 7. Fluxo: preparar dispositivo

### 7.1 — O ARCA particiona o dispositivo, e nunca escolhe o disco

> **Princípio P1 (revisado em 23/08/2026).** O ARCA destrói dados quando o
> usuário nomeou o alvo e confirmou por escrito, e **nunca por dedução**. O que
> ele não faz é agir sobre um disco que ele mesmo escolheu.

> **A versão anterior deste princípio dizia que o ARCA não executa a operação
> mais destrutiva do fluxo, e isso nunca foi verdade.** `arca restore` apaga
> 465 GB do disco de sistema desta máquina, e é a razão de o projeto existir;
> particionar um pen drive vazio não chega perto. O princípio classificava por
> **categoria da operação** quando o que separa uma da outra é **o que se perde
> quando dá errado** — e por esse critério a ordem era a inversa. Ver
> [ADR-0014](../docs/adr/0014-o-arca-particiona-o-dispositivo.md).

`arca prepare` **cria as duas partições e as rotula**: uma FAT32 de ≥ 1 GB para
o `ARCABOOT` e o resto do espaço em NTFS para o `ARCAVAULT`. Antes de escrever
qualquer coisa, imprime o que vai acontecer com aquele disco e exige
confirmação digitada (PR-4, PR-5, S-2).

**A estrutura é transcrita, e não inventada.** Medida no dispositivo desta mesa
em 23/08/2026 e preservada em
`recursos/capturas/estrutura-de-particoes-do-dispositivo-2026-08-23.txt`:

| | Valor medido |
|---|---|
| Estilo | **MBR** — e boota por UEFI assim mesmo |
| Partição 1 | `MbrType 7` (IFS/NTFS), offset 1.048.576, o resto do disco → `ARCAVAULT` |
| Partição 2 | `MbrType 12` (FAT32 LBA), 1.677.721.600 bytes, no fim → `ARCABOOT` |
| `IsActive` | nenhuma das duas — o boot é UEFI puro, não BIOS |
| Unidade de alocação | 4096 nas duas |

O esquema canônico moderno seria GPT com uma ESP. **Este não é ele, e é o que
está bootando aqui desde 19/08** — o `bcdedit` aponta `partition=R:` para
`\EFI\boot\bootx64.efi`, e o `efi-nvram.dat` de dentro das imagens registra a
máquina tendo bootado por ali. Trocar por GPT seria abandonar um esquema medido
por um suposto, num lugar cujo modo de falha é um dispositivo que não boota —
descoberto **depois** de o Windows já ter sido apagado, porque é aí que alguém
precisa dele.

**O que o ARCA não faz é escolher o disco.** O alvo vem por
`--dispositivo <índice>`, no molde do `--destino <índice>` da E9, e mesmo
havendo um só candidato ele é mostrado e confirmado, nunca assumido. Disco fixo,
disco de sistema e disco de boot são **recusa dura**, sem opção de forçar: a
identificação de disco é onde este código já errou uma vez (revisão da E9,
R-8), e `arca prepare` roda antes de existirem os rótulos que B-1, S-3 e C-10
usariam para se defender.

```
> arca prepare --dispositivo 1

  Disco 1 ......................... JMicron Generic · USB · 447,1 GB
  Tipo de midia ................... External hard disk media · nao e disco fixo (PR-5)
  Sistema ......................... IsSystem false · IsBoot false · nao carrega o C:
  Tabela de particao hoje ......... MBR · vai ser reescrita como MBR

O QUE EXISTE NESTE DISCO HOJE, e vai ser APAGADO:
  1  NTFS    445,6 GB  "ARCAVAULT"                E:
  2  FAT32     1,6 GB  "ARCABOOT"                 F:

  ESTE DISCO JA E UM DISPOSITIVO ARCA. Os rotulos acima sao os dele, e o
  que esta no ARCAVAULT sao AS IMAGENS — todas. Preparar por cima apaga
  cada uma, e o ARCA nunca apaga imagem em nenhum outro caminho (B-10).
  Se o que voce quer e reinstalar o Clonezilla sem perder as imagens,
  este comando NAO faz isso: ele comeca reescrevendo a tabela de particao.

O QUE VAI FICAR NO LUGAR:
  MBR  1  NTFS   445,6 GB  ARCAVAULT   as imagens moram aqui
       2  FAT32    1,6 GB  ARCABOOT    o Clonezilla e o ARCA moram aqui

  A estrutura e MBR, e nao GPT: e a que esta bootando neste projeto desde
 19/08, e trocar por GPT+ESP seria abandonar um esquema medido por um
 suposto — num lugar cujo modo de falha e um dispositivo que NAO BOOTA, e
 que so se descobre depois de o Windows ja ter sido apagado (ADR-0014).

E O QUE MAIS VAI ACONTECER:
  Clonezilla 3.3.3-15 · baixado (535,5 MB), com o SHA256 conferido contra
     o valor compilado neste ARCA — e nao contra um baixado junto (PR-1)
  Uma copia do pacote fica no ARCAVAULT, para o dispositivo se reconstruir
     sozinho (PR-3)
  Uma entrada de boot chamada `ARCA` e criada no firmware, apontando para o
     ARCABOOT — e **tirada da ordem permanente** logo em seguida, para que
     ligar a maquina continue subindo o Windows (C-5)
  O proprio `arca.exe` e instalado no ARCABOOT, porque o que julga uma
     restauracao nao pode morar no disco que ela substitui (§4.1)

  O `grub.cfg` fica INERTE: nada roda sozinho ate um `arca backup` (§4.4)

Podemos continuar? (s/N): s

  Conferido antes de escrever ..... ok · o disco 1 continua sendo `JMicron Generic` de 447,1 GB

Digite o modelo do disco para confirmar: JMicron Generic

  Particionando ................... ok · MBR, 2 particoes · MbrType 7 e 12
  Formatando e rotulando .......... ok · ARCAVAULT (NTFS) em E: · ARCABOOT (FAT32) em F:
  Conferido apos escrever ......... ok · relido do disco · nenhuma particao ativa, unidade 4096 (C-3)
  Baixando Clonezilla ............. 3.3.3-15 · 535,5 MB · pode levar minutos
  SHA256 conferido ................ ok · 00cee7700433 · de https://downloads.sourceforge.net/…
  Copia do pacote em ARCAVAULT .... ok · E:\clonezilla-live-3.3.3-15-amd64.zip (PR-3)
  Extraindo ....................... ok · F:\
  Estado inerte ................... ok · o `set default` do pacote era "0", e voltou para `live-default`
  Instalando o ARCA em ARCABOOT ... ok · F:\arca\arca.exe (§4.1)
  Entrada de firmware ............. criada · ARCA · {f4057bd3-…} · partition=F:
  Ordem de boot ................... ok · a entrada saiu da ordem permanente · o boot unico nao precisa dela la (C-5)

Dispositivo pronto.

  O `grub.cfg` esta INERTE: bootar neste dispositivo abre o menu do
  Clonezilla e espera alguem (§4.4). Nada roda sozinho ate um `arca backup`.
  A entrada de firmware existe e esta FORA da ordem permanente — ligar a
  maquina continua subindo o Windows, com ou sem este dispositivo conectado.

  O ARCAVAULT esta em E: e o ARCABOOT em F:. As letras mudam de uma
  conexao para outra; os rotulos, nao — e e por rotulo que o ARCA acha o
  dispositivo (B-1, S-3).

  SE VOCE TEM OUTRO DISPOSITIVO ARCA CONECTADO, desconecte um dos dois: o
  ARCA opera um por vez, e com dois `arca backup` e `arca restore` recusam
  por rotulo repetido (C-10).

  ANTES DO PRIMEIRO BACKUP, RODE:  arca sondar

  A receita nomeia o disco pelo nome que o LINUX lhe da (`nvme0n1`), e o
  Windows nao conhece esse nome. O ARCA o descobre lendo um `blkdev.list`,
  casando o modelo do disco (§4.5) — e este dispositivo acabou de nascer,
  entao nao ha nenhum aqui. Um `arca backup` agora RECUSA, dizendo isso.

  O ARCA nao pergunta o nome nem o deduz do indice: um `nvme1n1` digitado
  por engano entraria numa receita que apaga um disco, e nao ha nada do
  lado Windows contra o que conferi-lo.

  `arca sondar` resolve isso num reinicio: ele NAO faz backup nem
  restauracao — roda o `lsblk` no Linux do Clonezilla, grava a saida no
  ARCAVAULT e desliga. Depois de `arca resultado`, `arca backup <nome>`
  funciona.

  E ele responde, de quebra, a unica coisa que o `arca prepare` NAO consegue
  conferir sozinho: se este dispositivo boota mesmo, pela entrada de firmware
  que acabou de ser criada (P-26).
```

> **O fim desta tela mudou duas vezes, e as duas por motivo registrado.**
>
> **A primeira versão dizia `Primeiro backup: arca backup <nome>`, e era
> falsa**: esse comando **recusa** num dispositivo recém-preparado, porque o
> nome do disco no Linux sai do `blkdev.list` de dentro de uma imagem e não há
> imagem nenhuma (§4.5). O mesmo vale para `arca restore` e `arca verify
> --completo` — **nenhum dos três comandos que armam funciona num dispositivo
> recém-nascido**. É o padrão que o §11 nomeia: peça nova encaixada em peça
> antiga que ninguém releu ao encaixar, e a peça antiga aqui é uma decisão de
> duas etapas atrás.
>
> **A segunda mandava para o menu do Clonezilla** — F12, backup manual pelo
> §6.4, e daí em diante o ARCA. Ela não estava errada sobre os fatos, e continua
> sendo o caminho manual quando tudo o mais falhar. O que ela era: exatamente
> aquilo que este app existe para não precisar, cobrado logo na primeira vez que
> alguém usa um dispositivo novo — dois reinícios e cerca de quarenta minutos.
> **Ela nasceu com data de validade escrita no plano de etapas**, para não
> sobreviver ao motivo dela como a primeira quase sobreviveu.
>
> **A terceira é a de cima**, e é da E12: `arca sondar` custa um reinício e
> nenhuma tela do Clonezilla ([ADR-0019](../docs/adr/0019-a-sondagem-e-a-quarta-operacao.md)).
>
> As duas capturas em `recursos/capturas/` são de antes das correções e ficam
> como estão: elas são o que a tela imprimiu, e reescrevê-las seria confundir o
> que rodou com o que se quis.

> **O segundo parágrafo tem uma outra forma desde 24/08/2026, e ela sai quando
> a ordem permanente tem entrada que não diz para onde aponta** (P-28,
> [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md)):
>
> ```text
>   A entrada de firmware existe e esta FORA da ordem permanente. Mas a
>   ordem tem `UEFI:Removable Device`, que NAO DIZ para onde aponta — quem a
>   resolve e o firmware, no POST, pelo que estiver conectado —, e por
>   isso o ARCA nao afirma o que ligar a maquina vai subir. Remova o SSD
>   antes de religar se quiser certeza (P-28).
> ```
>
> **A promessa é sobre o que ficou na ordem, e não sobre o que saiu dela.** O
> texto anterior era fixo: ele derivava *"ligar a maquina continua subindo o
> Windows"* de um fato só — a entrada do ARCA saiu da ordem —, sem olhar quem
> continuava lá. Custa uma leitura de `firmware` a mais no fim da criação da
> entrada, e ela **recusa** se o `{fwbootmgr}` não se deixar ler: uma tela que
> promete o boot sem ter lido a ordem é a mesma coisa que a linha `Ordem de
> boot` do `arca status` deixou de fazer no ADR-0009.

> **Esta tela é execução real**, de 23/08/2026, e é a montagem de **duas**: a
> primeira execução do comando produziu tudo, e a linha `Entrada de firmware`
> dela dizia `reusada e reapontada`, porque esta máquina já tinha uma entrada
> `ARCA`. A forma `criada` acima é da segunda execução, feita depois de apagar
> aquela entrada — e é a que interessa a quem prepara um dispositivo numa
> máquina que nunca teve ARCA.
>
> As duas telas inteiras estão em `recursos/capturas/`, separadas. Juntá-las no
> documento sem dizer isto seria abandonar a distinção entre reprodução e
> captura que a E8 pagou para manter.

> **O `arca prepare` é o único comando destrutivo do ARCA que não custa um
> reinício.** `arca backup`, `arca restore` e `arca verify --completo` armam,
> reiniciam e só dizem o que aconteceu na volta; este faz tudo do lado Windows,
> com a tela na frente. É por isso que ele pode dar-se ao luxo de **perguntar
> duas vezes** — uma barata, uma cara —, e é por isso que o §11 não ganha
> nenhuma armadilha de "o que se vê do outro lado do reinício" para ele.

> **PR-4 são quatro tempos, e o terceiro é o que faz os outros valerem.** O
> plano inteiro; a pergunta; **a conferência, que é do ARCA e não do usuário**;
> e a confirmação digitada. A resposta do passo 2 diz que a pessoa **quer**
> prosseguir — ela não é evidência sobre o disco.
>
> Por isso o ARCA relê o disco antes de escrever, e compara **modelo e
> tamanho** com o que imprimiu. **O índice do Windows não é identidade**: em
> 23/08/2026 o dispositivo desta mesa era o disco 1 e virou o disco 2 quando um
> segundo SSD foi conectado, e o `ARCAVAULT` dele, que sempre aparecera em `E:`,
> veio em `D:`. Entre imprimir o plano e apagar a tabela há uma pessoa lendo e
> digitando, e nesse intervalo cabe trocar um cabo.
>
> É a mesma família de C-3 — não acreditar no que se pediu, perguntar de novo —,
> aplicada ao intervalo em que o ARCA não estava olhando.

> **A confirmação pede o modelo do disco, e não o índice.** Pelo mesmo motivo
> que a restauração pede o nome da imagem por extenso (S-2, R-3): é o que está
> na tela, e digitá-lo custa lê-lo. Um `1` é curto demais para custar alguma
> coisa — e é justamente o número que muda de uma conexão para outra.

> **O aviso `ESTE DISCO JA E UM DISPOSITIVO ARCA` nasceu de rodar o comando de
> verdade.** O disco do marco tinha os dois rótulos, e a lista de partições os
> mostrava — o que é dizer a quem sabe ler rótulo, e não a quem está com dois
> SSDs iguais na mesa. PR-4 pede que quem vai perder dados possa reconhecê-los;
> um `ARCAVAULT` de 445 GB numa linha não é reconhecimento, é uma pista.

> **A última linha do bloco de ações diz `criada`, e não *migrada*:** num
> dispositivo novo não há entrada `Clonezilla` a migrar, e criar entrada de
> firmware do zero era o código sem original que a E7 recusou escrever e mandou
> para cá. **Ele deixou de ser sem original em 23/08/2026**: a entrada `ARCA`
> desta máquina é, campo a campo, uma cópia do `{bootmgr}` com `device`, `path`
> e `description` trocados, e criá-la é `bcdedit /copy {bootmgr} /d ARCA`
> seguido de dois `/set` — medido, com a entrada de medição apagada no fim e o
> firmware voltando byte a byte ao que era. Ver
> [ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md).
>
> **E a medição trouxe o que ninguém tinha previsto**: o `/copy` põe a entrada
> nova no `displayorder` **sozinho**. Isso é o perigo que C-5 nomeia — o ARCA
> acrescentando um caminho permanente para bootar no dispositivo —, e é por isso
> que a linha `Ordem de boot` existe nesta tela.

> **A tela anterior desta seção mostrava as duas partições já prontas, com um
> `ok` ao lado de cada uma — e escondia uma pergunta que ninguém tinha feito:
> quem põe o rótulo `ARCAVAULT`?** As linhas de ação abaixo dela só falavam do
> `ARCABOOT`, e a única coisa que o ARCA escrevia no `ARCAVAULT` era a cópia de
> PR-3. A pergunta apareceu em 23/08/2026 e foi o que levou à revisão de P1: se
> era o usuário quem rotulava, o modo de falha real — rotular a partição errada
> e o ARCA aceitar sem discutir — continuava de pé, e a fricção não comprava
> nada.
>
> **E uma consequência que só apareceu ao escrever isto:** `arca prepare` é **o
> único comando que não consegue se localizar pelos rótulos** (B-1, S-3),
> porque no disco que ele vai preparar eles ainda não existem. Ele roda num
> mundo onde as defesas dos outros comandos não se aplicam — nem C-10, que não
> tem o que recusar quando não há rótulo nenhum na mesa —, e é por isso que as
> sete defesas de PR-5 são dele sozinho. Ver a seção E10 do
> [plano de etapas](implementation_stages.md).

### 7.2 — O que o dispositivo recebe, e de onde

Construída na etapa E10. O `arca prepare` instala o **zip** que o Clonezilla
publica, na versão `3.3.3-15` — a mesma que roda no dispositivo desta mesa, e
sobre a qual rodaram os quatro marcos em hardware deste projeto.

| | |
|---|---|
| Arquivo | `clonezilla-live-3.3.3-15-amd64.zip` · 561.478.648 bytes |
| SHA256 | `00cee7700433e63017e2ea9eb40519108829710132364a8028a6c039a6046304` |
| De onde vem | SourceForge, o canal que o site do Clonezilla aponta |
| De onde vem o **número** | `CHECKSUMS.TXT` do mirror do projeto em `free.nchc.org.tw`, **e** o `certutil` sobre o arquivo baixado — dois servidores, o mesmo número |
| Como se extrai | `C:\Windows\System32\tar.exe`, que é o `bsdtar 3.8.8` |

**O dispositivo desta mesa veio do ISO, e não do zip.** Medido: o
`boot/grub/grub.cfg` dos dois difere em exatamente duas coisas — o `noeject` em
treze `menuentry`, e **seis segundos** no carimbo do rodapé. Seis segundos é o
`ocs-live-dev` gerando os dois artefatos na mesma execução: é a mesma build.

Isso não é argumento contra o zip; é a favor. `noeject` é o parâmetro certo para
mídia removível — ejetar um USB no desligamento é o oposto do que se quer —, e
o ISO não o tem porque mídia óptica se ejeta mesmo. O custo é **oito bytes** na
linha de comando do kernel: o `menuentry` base do §10.2.3 passa de 471 para 479,
e a folga dentro dos 512 reservados cai de 41 para 33.

**E o zip entrega um `grub.cfg` que não está inerte:** ele vem com `set
default="0"`, que o §4.4 e o [ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)
nomeiam como *"um estado que parece inerte"* — `"0"` aponta por **posição**, e a
posição muda quando o bloco do ARCA entra antes do `live-default`. Por isso o
`arca prepare` **desarma o que acabou de instalar**, e a tela diz que fez.

O oráculo: desarmar o `grub.cfg` do zip produz **exatamente** o `grub.cfg`
inerte deste dispositivo, a menos das duas diferenças de origem. Ver
[ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md).

**A armadilha do `tar`, e ela é cara.** `tar` no `PATH` pode não ser o `bsdtar`:
com o Git para Windows instalado, ele resolve para o **GNU tar 1.35**, que não
abre zip. O ARCA chama `curl` e `bsdtar` por **caminho absoluto** no `System32`
— e o modo de falha de não fazer isso é falhar na extração **depois** de o disco
já ter sido apagado.

## 8. Comandos

```
arca prepare --dispositivo <indice>
                          # particiona o disco, instala o Clonezilla e o ARCA,
                          #   cria a entrada de boot (§7.1)
                          #   --iso <caminho>  instala de arquivo local (PR-2)
arca backup <nome>        # monta a receita, arma o boot, reinicia
arca resultado            # le o veredito e desarma o SSD
arca list                 # imagens no dispositivo conectado
arca restore [<nome>]     # lista, confirma e reinicia para restaurar
arca verify <nome>        # confere os MD5SUMS, sem reiniciar (~3,5 min em 39,7 GB)
                          #   --completo  arma boot unico para o ocs-chkimg (V-2)
arca sondar               # arma boot unico que so roda `lsblk` e desliga (SD-1)
                          #   e o que da ao §4.5 um oraculo sem exigir imagem
arca status               # diagnostico: dispositivo, firmware, job pendente
arca desarmar             # devolve o dispositivo ao estado inerte (§4.4)
```

> **Desde a etapa E10 os oito primeiros fazem o trabalho.** Eles existem na
> superfície da linha de comando desde a E0, e até aqui os que ainda não tinham
> etapa construída respondiam dizendo qual etapa os entregava — o que fez a
> fundação ser executável de verdade desde o primeiro dia. O `prepare` era o
> último da lista, e ela esvaziou.

> **O nono nasceu na E12, e ele é o único que não estava na superfície desde a
> E0.** `arca sondar` não foi previsto pelo plano: ele nasceu de uma pergunta na
> mesa em 23/08/2026 — *"e quando o outro SSD não estiver lá?"* —, cuja resposta
> expôs que **nenhum dos três comandos que armam funciona num dispositivo
> recém-preparado** (§4.5). A tela do `arca prepare` mandava, então, fazer o
> primeiro backup pelo menu do Clonezilla: exatamente aquilo que este app existe
> para não precisar.
>
> Ele é o **quarto que arma** e o mais barato de todos: a receita dele não chama
> programa nenhum do Clonezilla — nem `ocs-sr`, nem `ocs-chkimg` —, e nada é
> escrito fora do `ARCAVAULT`. Ver §9.7 e
> [ADR-0019](../docs/adr/0019-a-sondagem-e-a-quarta-operacao.md).
>
> **E ele não aceita argumento nenhum**, o que o separa dos outros três que
> armam: eles nomeiam uma imagem, e a sondagem pergunta *"que discos há nesta
> máquina?"* — uma pergunta sem sujeito a escolher, feita justamente no
> dispositivo que ainda não tem imagem. A confirmação, por isso, é uma tecla e
> não um nome digitado (SD-6).

> **`--dispositivo <índice>` é obrigatório, e a obrigatoriedade é P1 revisado
> na letra**: *o ARCA destrói dados quando o usuário nomeou o alvo, e nunca por
> dedução*. Um `arca prepare` sem alvo teria de escolher um disco sozinho —
> mesmo havendo um só candidato, mesmo quando a escolha pareceria óbvia.
>
> E o índice **não é identidade**: ele muda quando se conecta ou desconecta um
> disco, medido em 23/08/2026. Por isso a confirmação de S-2 pede o **modelo**,
> que a tela acabou de imprimir, e não o número que se digitou aqui (§7.1).

> **`arca verify --completo` é o terceiro comando que arma**, e entrou na etapa
> E11. Ele desarma primeiro (C-1), pede a confirmação digitada e reinicia, como
> os outros dois — e pede a confirmação **mesmo não destruindo nada**, pelo
> mesmo motivo que o `arca backup` a pede: a máquina vai reiniciar e desligar
> sozinha, e quem digitou `--completo` sem ler está a um Enter de perder o que
> estiver aberto. Sem `--completo`, o comando só lê: não escreve, não arma, não
> reinicia e **não desarma** — C-1 fala dos comandos que armam, e este não arma
> (o mesmo raciocínio que o `arca resultado` já usava).

**`arca desarmar` não substitui C-1.** Desarmar continua sendo o primeiro passo
de todo comando que arma — não é algo que o usuário precise lembrar de fazer. O
comando existe pelo caso que o §5.5 já descreve e não atendia: *"sem
`arca-fim.txt`, com job pendente — o boot não aconteceu"*. Depois dele o
dispositivo continua armado e não havia nada a rodar, porque `arca resultado`
exige desfecho e `arca backup` armaria de novo. É também a única forma de
exercitar a idempotência de C-1 sem armar. Acrescentado na etapa E4.

> *(Etapa E8: aquele caso passou a ter outra saída — `arca resultado` agora
> **atende** a ausência de desfecho, reportando as duas causas de C-12,
> desarmando e encerrando o job. O `arca desarmar` continua valendo pelo resto:
> desarmar sem colher, e exercitar C-1 sem armar. `arca resultado` **não**
> desarma quando não há job nenhum a colher — misturar "colhi" com "arrumei"
> tiraria de quem lê a saída a informação de qual das duas aconteceu.)*

Quatro flags:

```
--dry-run                 # imprime o que faria; nao arma e nao escreve nada
--completo                # em verify: arma boot unico para o ocs-chkimg (V-2)
--dispositivo <indice>    # em prepare: o disco a preparar (PR-5) — obrigatorio
--iso <caminho>           # em prepare: instala de arquivo local (PR-2)
```

> **`--destino <indice>` saiu em 23/08/2026**, e a ausência é decisão. Ele
> existiu da E9 até ali para alcançar a metade permissiva de R-7 — *"destino
> diferente é permitido"* —, e o
> [ADR-0015](../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md)
> fechou esse caso por escopo: **o único destino válido é o disco de origem**, e
> quem troca o disco reinstala o Windows.
>
> Sem destino divergente, a flag passa a ser um jeito de apontar um disco para
> apagar — e é isso que P1 revisado proíbe: *o ARCA não age sobre um disco que
> ele mesmo escolheu, e também não age sobre um que lhe apontaram sem poder
> conferir*. O ARCA acha o disco de origem pelo modelo (§4.5) e prova que é ele
> pelos setores (R-7); não achando, ou achando dois, ele **para**.

> **`--dry-run` no `arca prepare` vale mais do que em qualquer outro comando.**
> Nos que armam ele imprime a receita; aqui ele é a **única forma de ver o plano
> de partições sem executá-lo** (PR-5, defesa 6). Ele para antes da pergunta de
> PR-4, não escreve nada e não diz que escreveu.

> **`arca restore <nome>` pula a lista.** É o atalho de quem já a leu, e é o
> que torna `arca restore <nome> --dry-run` utilizável sem console. A
> confirmação por extenso (R-3) continua obrigatória nos dois caminhos.

Todos exigem privilégio administrativo.

## 9. Requisitos

### 9.1 — Comuns a toda operação

| ID | Requisito |
|---|---|
| C-1 | **Desarmar a receita anterior incondicionalmente**, como primeiro passo, sem consultar estado nenhum. O estado a que se volta está definido no §4.4, e é reconstruído do `grub.cfg` corrente — o que torna a operação idempotente sem que ninguém precise garanti-lo |
| C-2 | **Validar a receita antes de gravar** no `grub.cfg`: rejeitar pipes, **toda** aspa (não só as desbalanceadas — um par de aspas simples fecha o `bash -c` e abre outra string), substituição de comando, caractere de controle, não-ASCII, e a linha que não coubesse no `COMMAND_LINE_SIZE` do kernel (§10.2.3). Nomes inseguros já param antes, em B-2 |
| C-3 | Nunca confiar no retorno do `bcdedit`; sempre conferir com `/enum` e parsear **por valor** |
| C-4 | Procurar a entrada `ARCA`; não havendo, migrar a legada `Clonezilla` em vez de criar outra. **Migrar é renomear a `description`** — o GUID, o `device` e o `path` já são os certos, e criar uma segunda entrada deixaria a máquina com duas formas de bootar no Clonezilla. *(Etapa E7: **não havendo nenhuma das duas, o ARCA recusa em vez de criar.** Criar uma entrada de firmware do zero é código sem original — nenhuma captura mostra a forma —, e o lugar disso é o `arca prepare` da E10. Armar não é a hora de estrear a criação de entrada de boot.)* *(Etapa E10, 23/08/2026: **o `arca prepare` cria, e a criação deixou de ser código sem original.** A entrada `ARCA` desta máquina é, campo a campo, uma cópia do `{bootmgr}` — `locale`, `inherit`, `flightsigning`, `resumeobject`, `toolsdisplayorder {memdiag}` —, e criá-la é `bcdedit /copy {bootmgr} /d ARCA` seguido de `/set device` e `/set path`. Medido, com a entrada de medição apagada no fim e o firmware voltando byte a byte ao que era; a entrada criada saiu **idêntica** à que já existia. O identificador se acha **pela forma** — 36 caracteres entre chaves —, e nunca pelo texto da resposta, que vem traduzido. **E o `arca prepare` reusa antes de criar**: havendo `ARCA` ou `Clonezilla`, ele reaponta a que existe, que é este requisito na letra. Ver [ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md). A recusa do armar continua valendo, e agora com saída: quem cai nela roda `arca prepare`.)* |
| C-5 | Boot único — nunca alterar a ordem permanente. *(Etapa E7: medido que o `bcdedit` **aceita** `bootsequence` para uma entrada de fora do `displayorder`, e que o `displayorder` não muda nem ao pôr nem ao tirar. Sem isso, armar obrigaria a violar este requisito. A ordem permanente é lida antes de escrever e comparada depois — em `armar` como em `desarme` —, e uma divergência é falha ainda que a marca tenha pegado.)* *(Etapa E10, 23/08/2026: **este requisito fala das operações que armam.** A revisão que P-20 pedia aconteceu, e a distinção que faltava está feita: o perigo que C-5 nomeia é o ARCA **acrescentar** um caminho permanente para o dispositivo, e pôr o `{bootmgr}` à frente não acrescenta caminho nenhum — põe o Windows na frente dos que já existem, e não remove nada. Colher passa a fazer isso, e é **C-13**. C-5 continua valendo inteiro no armar e no desarme, que releem a ordem e falham se ela mudou. Ver [ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md), que supersede a decisão do ADR-0009.)* *(Etapa E10, `arca prepare`: **o perigo que este requisito nomeia aconteceu, e não foi o ARCA que o causou.** Medido em 23/08/2026: `bcdedit /copy` põe a entrada nova no `displayorder` **sozinho**, sem que ninguém peça — é literalmente acrescentar um caminho permanente para bootar no dispositivo. Por isso o `arca prepare` a **tira da ordem** logo depois de criá-la, com `/set {fwbootmgr} displayorder {novo} /remove` e releitura de C-3; e confere, na mesma releitura, que nenhuma outra entrada saiu junto. Tirar não quebra o armar: o `bootsequence` funciona sobre entrada fora da ordem, medido na E7 e exercitado no marco de 22/08. **E não é o `/remove` que o ADR-0013 descartou** — lá o problema era acertar *quais* entradas tirar, e aqui o alvo é a entrada que o próprio comando acabou de criar, com o identificador em mãos. Ver [ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md).)* |
| C-6 | **Recusar mídia removível como alvo de entrada de boot; orientar F12.** A recusa não se lê numa etiqueta do `bcdedit` — essas palavras não saem dele (§3.1). Verifica-se de dois jeitos: o **`MediaType` do WMI** dá o sinal antecipado, e a releitura de C-3 revela a rejeição como um `device` que não mudou. *(Etapa E6: o sinal antecipado era o `GetDriveType`, que classifica o SSD externo desta mesa como disco **fixo** e não distingue nada. O `MediaType` responde literalmente `External hard disk media` e `Removable Media` — são as palavras da §3.1, e é de lá que elas saem.)* *(Etapa E7: a **segunda** metade passa a existir. Ao armar, o ARCA escreve o `device` da entrada apontando para o `ARCABOOT` que está na mesa e relê; um `device` que não mudou é a rejeição silenciosa, e o armar para ali. Escreve **sempre**, mesmo quando o valor já está certo — é a releitura que responde, e pular a escrita no caso normal deixaria justamente o caminho normal sem exercício, que é o mesmo raciocínio de `desarme` sobre o `deletevalue`.)* |
| C-7 | Repassar os argumentos ao relançar com elevação por UAC |
| C-8 | Escapar aspas com **barra invertida**, não crase — quem reparte a linha é o parser do Windows |
| C-9 | Avisar, antes de reiniciar, para remover o SSD ao terminar. **Depois de armado e antes do reinício** — é a última coisa que alguém lê antes de a tela apagar, e não há tela do outro lado (§5.2) |
| C-10 | *(Etapa E9: as duas recusas que falam do **dispositivo** — esta e C-6 — passam a valer também para o `arca restore`, e antes da confirmação digitada. O `armar` pegaria a rejeição silenciosa de C-6 na releitura, mas depois de a pessoa ter digitado o nome de uma imagem que vai apagar um disco; e o dispositivo partido levaria o `estado.json` para o `ARCABOOT` de um dispositivo com o desfecho indo para o `ARCAVAULT` do outro.)* **Recusar mais de um dispositivo ARCA conectado.** Dois `ARCAVAULT` ou dois `ARCABOOT` tornam o destino ambíguo, e é por LABEL que a receita resolve (S-3). **E recusar também o dispositivo partido**: os dois rótulos em discos físicos diferentes são dois dispositivos meio prontos, e não um — cada rótulo aparece uma vez, a contagem passa, e a receita iria para um enquanto as imagens estão no outro. *(A brecha do rótulo órfão ficou aberta da E1 à E5, com a letra impressa na tela como única defesa; a enumeração de discos da E6 a fecha.)* |
| C-11 | **Gerar um selo ao armar**, gravá-lo no `estado.json` e embuti-lo na receita; aceitar como desfecho apenas o `arca-fim.txt` cujo selo case (§4.3) |
| C-12 | **Ausência de desfecho é falha, nunca silêncio.** Havendo job pendente e nenhum `arca-fim.txt`, reportar as duas causas possíveis: o boot não ocorreu, ou o Clonezilla abriu menu (§5.5). *(Etapa E8: ausência de desfecho **encerra** o job, porque é um veredito. O que não encerra é o `arca-fim.txt` que está lá e não se deixou ler — "não consegui olhar" não é veredito, e encerrar ali perderia o selo. Ver [ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md).)* |
| C-13 | **Ao colher, devolver o `{bootmgr}` ao topo da ordem permanente.** Uma escrita só, com alvo fixo — `/set {fwbootmgr} displayorder {bootmgr} /addfirst` —, incondicional, e conferida pela releitura de C-3 sobre a pós-condição que importa: *o primeiro da ordem é o `{bootmgr}`?* **Nada é removido**: as entradas do dispositivo continuam na ordem, atrás do Windows, e por isso o conserto vale para todas de uma vez, inclusive as que o firmware criar depois. Acontece nos três caminhos do `arca resultado` — colheu, não havia job, já estava colhido —, porque a ordem permanente é estado da NVRAM e não do job. A saída diz isto em **linha própria**, separada da do desarmar (E8), e o conselho só aparece quando houve conserto. *(Etapa E10, 23/08/2026: nasce de P-20, medido à mão antes de virar código, e supersede a decisão do [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md). Ver [ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md).)* |
| C-14 | **Ausência de resposta do firmware nunca vira segurança.** Uma entrada da ordem permanente **sem `device`** não diz para onde aponta — as `UEFI:*` que o firmware acrescenta no POST são assim —, e nenhuma tela pode ler isso como *"não leva ao dispositivo"*. **Três estados, e não dois**: leva, não leva, não se sabe; e `não se sabe` não autoriza afirmação de segurança em lugar nenhum. É a mesma forma de C-3 e da guarda de `viu_o_gerenciador`, aplicada à ordem. **O discriminante é a falta de `device`, e não a de letra**: o `{bootmgr}` aponta para `partition=\Device\HarddiskVolume1` — alvo concreto, só não conferível por letra —, e tratá-lo como opaco faria o aviso disparar sempre, que é o mesmo que não avisar. *(24/08/2026, nasce de P-28, e o código veio antes da medição porque a regra deixa de afirmar em vez de afirmar. Ver [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).)* |
| C-15 | **A recusa do `bcdedit` não apaga o que ele listou.** Um `/enum` que sai com código diferente de zero **e** traz o gerenciador de firmware (ou, num `/enum {guid}`, a entrada pedida) é uma leitura, e o código é informação a mais — guardada em `Leitura::codigo_da_recusa`, e nunca transformada em "não li". **Medido em 27/08/2026**: com a entrada `ARCA` apontando para o GUID de uma partição que o `arca prepare` tinha acabado de apagar, **todo** `bcdedit /enum` desta máquina — `{fwbootmgr}`, `firmware`, `{bootmgr}`, `all`, o de uma `UEFI:*` sem `device` — imprimia a listagem inteira e terminava com *"Foi especificado um dispositivo inexistente."*, código 1; o `/set device` para a partição nova devolveu o código 0 a todos. Tratar o código como recusa deixou `prepare`, `sondar`, `backup` e `status` sem ler o firmware, e o comando que consertaria o estado era o que o próprio `prepare` executaria três linhas depois. **O que não muda**: *"Acesso negado"* — texto sem listagem, código 1 — continua sendo recusa, com o texto inteiro; e **nenhuma leitura com código cria entrada** (`/copy`): o `arca prepare` só reusa a que leu, e decide isso antes do ponto sem volta (PR-6, C-4). É a forma de `viu_o_gerenciador` aplicada ao código de saída. *(Ver [ADR-0026](../docs/adr/0026-a-recusa-do-bcdedit-nao-apaga-o-que-ele-listou.md).)* |

### 9.2 — Backup

| ID | Requisito |
|---|---|
| B-1 | Localizar o dispositivo pela partição `ARCAVAULT` |
| B-2 | Recusar nome com espaço, acento ou caractere inválido para nome de pasta. **Por lista de permissão** (`A-Z a-z 0-9 . _ -`), e não de recusa: uma lista de recusa só está certa enquanto ninguém esquecer um caractere. Recusar também: nome reservado do Windows (`CON`, `COM0`–`COM9`, `LPT0`–`LPT9`, …), as pastas de serviço do dispositivo (`ARCA-LOGS`, `ARCA-DOCS`), nome começando com `-` (o `ocs-sr` o leria como opção) ou com `.`, e nome acima de 48 caracteres (§10.2.3) |
| B-3 | **Recusar nome cuja pasta já exista** — mesmo sem `MD5SUMS`. Pasta sem `MD5SUMS` é resíduo de backup interrompido; o usuário apaga à mão |
| B-4 | Espaço mínimo: o maior entre `maior imagem do dispositivo × 1,3` e `em uso × 0,45`. Entre 1× e 1,5× disso: avisar e pedir confirmação digitada. **`em uso` é do disco, e não dos volumes com letra**: o disco desta máquina tem quatro partições e só o `C:` tem letra — as outras três somam ~1,3 GB que a soma por volume ignora, e o `Win32_DiskPartition` nem enxerga a MSR. Contado como `tamanho do disco menos o livre nos volumes com letra`, o que superestima, e superestimar é o lado seguro de "cabe uma imagem?" |
| B-5 | Verificar Inicialização Rápida; oferecer `powercfg /h off`. **A leitura é do registro** (`HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power` → `HiberbootEnabled`, um `REG_DWORD`), e nunca do `powercfg /a`: ele responde traduzido, e parsear frase traduzida é o erro que a E2 nomeou. Valor ausente é **"não se sabe"**, nunca "desativada". **Oferecer é dizer o comando e o que ele custa** — o ARCA não o roda, e a oferta diz que `powercfg /h off` desliga a hibernação **inteira**, não só a Inicialização Rápida |
| B-6 | Rodar `chkdsk /scan`; oferecer agendar `/f` se acusar erro. **Julgado pelo código de saída, nunca pelo texto** — ele responde traduzido. Confere o volume do **sistema**, que é o que o Clonezilla vai ler. Medido: elevado, `chkdsk C: /scan` sai com código 0 em 16,3 s, e o texto vem em CP850 mesmo chamado de um console em UTF-8 |
| B-7 | Receita com nome e disco embutidos — **sem `ask_user`** |
| B-8 | Sempre `-q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true`, **nesta ordem** — é a sequência que rodou nos dois backups validados. `-batch` é o que suprime as perguntas (§3.2). `-p true` é o que impede o `ocs-sr` de reiniciar antes de o `ocs-chkimg` rodar, já que o padrão de `-p` é `reboot`. **Nunca `-scs`**: ele é `--skip-check-restorable` e pula a conferência nativa, o oposto do que B-9 quer |
| B-9 | Sempre chamar `ocs-chkimg` explicitamente, com saída redirecionada para arquivo, **dentro do ramo de êxito do `savedisk`**: com o backup falhando, a pasta da imagem pode nem existir, e o redirecionamento falharia junto |
| B-10 | Nunca apagar nada |

### 9.3 — Restauração

| ID | Requisito |
|---|---|
| R-1 | Listar as imagens **no Windows**; a escolha acontece antes do reinício |
| R-2 | Conferir o destino contra `disk`/`blkdev.list` da imagem |
| R-3 | Exigir o nome da imagem digitado por extenso |
| R-4 | Sempre `-e1 auto -e2 -batch -j2 -k0 -iefi -p true`, **nesta ordem**, sempre **sem** `-g auto`. `-e1` e `-e2` acertam a geometria CHS da partição de boot NTFS: inócuos no mesmo disco, e é o que faz a restauração funcionar em outro (R-7). Estavam na restauração validada e o requisito não os listava. `-p true` em vez do `-p poweroff` que rodou, porque com a máquina desligando dentro do `ocs-sr` o desfecho de R-5 nunca seria escrito |
| R-5 | Receita com `if/then/else`: escrever `ARCA_RESTORE=OK` ou `ARCA_RESTORE=FALHOU`. Era código novo — as três receitas preservadas encadeiam com `;`. **A forma rodou em 22/08/2026 no backup**, tomando o ramo do sucesso; para a restauração continua sem original, e o ramo de falha depende de P-6 |
| R-6 | Ler esse arquivo na volta e **conferir o selo antes de acreditar nele** (C-11). O job fantasma que isto previne é **risco herdado**, e não corrente: §4.1 eliminou a causa ao tirar o ARCA do `C:`, e só imagens feitas antes disso carregam estado dentro de si. O selo cobre de qualquer forma, e é o mesmo mecanismo dos outros três casos (§4.3) |
| R-7 | **O único destino válido é o disco de origem**, e a medição prova **identidade**, não capacidade: os setores que a GPT de dentro da imagem registra têm de bater **exatamente** com os que o `MSFT_Disk` responde para o disco na mesa. Não batendo — para mais ou para menos —, recusa. *(Revisado em 23/08/2026, [ADR-0015](../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md): destino divergente deixou de existir por decisão de escopo, e com ele o `--destino <indice>`. Igualdade exata é mais difícil de satisfazer por acidente do que `≥`, então a defesa endureceu sem custar código. Havendo dois discos do mesmo modelo, o ARCA **recusa e para** em vez de pedir que alguém aponte: uma dúvida do ARCA não vira afirmação do usuário sobre a qual não há como conferir nada — é o mesmo raciocínio da E7 ao não pedir o nome do disco do Linux.)* **Texto anterior:** ~~destino diferente do disco de origem é permitido, e nomeado por `--destino <indice>` — o índice do Windows, que o ARCA traduz para o nome do Linux pelo `blkdev.list` (§4.5) e que nunca chega à receita. Recusar sempre que o destino for menor que a origem.~~ *(Etapa E9: ~~`-k0` copia a tabela inteira e, num disco menor, corrompe em vez de falhar~~ — **a premissa estava errada, e P-17 é isso**. O help do `ocs-sr` desta versão diz que o Clonezilla **confere o tamanho do destino por padrão e desiste** se for menor; `-icds` é quem desligaria a conferência, e a receita não o usa. A recusa do ARCA fica, e a razão passa a ser **onde** ela acontece: a do Clonezilla custa um reinício de uma operação destrutiva, e a do ARCA custa zero. A comparação sai em **setores**, com o destino medido pelo `MSFT_Disk` e a origem lida do `<disco>-gpt.sgdisk` de dentro da imagem — as duas na mesma régua. Setor lógico diferente entre origem e destino é recusa, e não conversão. Ver [ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md).)* ~~Em disco novo, `-iefi` não encontra entrada correspondente e o `bcdboot` volta a ser necessário — ao contrário do que §3.4 mediu no disco original.~~ *(Etapa E10, 23/08/2026: **a dívida do ADR-0015 foi paga no código**, e esta última frase saiu junto com o caso que ela descrevia — em disco novo não há restauração. O `--destino` saiu do `cli.rs`, `escolher_o_destino` deixou de aceitar um índice, `DestinoAmbiguo` virou **recusa terminal** com mensagem que manda desconectar o disco a mais em vez de apontar um, a comparação passou de `>=` para `==`, e a tela do §6.1 trocou a linha `Cabe (R-7)` pela de identidade. **A recusa R-8 ficou, e ficou testável**: com o filtro por modelo excluindo o dispositivo, ela deixou de ser alcançável pelo caminho normal, e o julgamento do candidato saiu para função própria para que a segunda barreira continue exercitada — o ADR-0015 previa que ela viraria redundante e disse por que fica.)* |
| R-8 | **Recusar o próprio dispositivo ARCA como destino, sempre, e sem confirmação que libere.** Restaurar nele apagaria o Clonezilla que está executando a receita e as imagens que ela lê — inclusive a que está sendo restaurada. *(Etapa E9: não havia requisito escrito para isto, e ele é a recusa mais dura do comando. Verificado pelas letras dos volumes do dispositivo contra as letras do disco escolhido, e julgado **antes** de qualquer outra recusa: o disco do dispositivo desta mesa também é menor que a origem, e se a ordem mudasse a mensagem passaria a falar de tamanho — quem lesse acharia que um SSD maior resolveria.)* |

### 9.4 — Segurança

| ID | Requisito |
|---|---|
| S-1 | O ARCA nunca abre o disco de origem em **acesso raw** de escrita. Chamar `powercfg` ou `chkdsk` (B-5, B-6) não é isso: são operações do próprio sistema, pelas quais o Windows responde |
| S-2 | Operação destrutiva exige texto digitado, nunca só `s`. **Comparação exata**, sem ignorar caixa e sem aceitar prefixo: B-2 permite maiúscula e minúscula, e `2026-08-22_apps` é uma imagem diferente de `2026-08-22_Apps`. Uma tentativa só — quem digitou errado repete o comando, que até ali não armou nada. `--dry-run` pula a confirmação **e** o armar, e não diz que armou |
| S-3 | Destino sempre por LABEL — nunca por letra, `sda` ou número de série |
| S-4 | Veredito e desfecho sempre gravados em arquivo, nunca só em tela — o `arca-check.log` e o `arca-fim.txt`, ambos escritos pela receita. **Rodou em 22/08/2026**, e os dois originais estão em `recursos/capturas/`: o `arca-fim.txt` com selo, desfecho e `ARCA_FIM`, e o `arca-check.log` terminando em `ARCA_VEREDITO=APROVADA` |
| S-5 | Falha parcial é tratada como falha total. *(Etapa E8: o desfecho e o veredito saem em **duas linhas** da §5.4, e nenhuma esconde a outra. Um `ARCA_BACKUP=OK` com imagem reprovada, sem veredito, resíduo, ou sem pasta nenhuma — os quatro são falha, e o comando sai com código diferente de zero depois de imprimir a tela inteira.)* |
| S-6 | **Nunca comparar uma data escrita pelo Windows com outra escrita pelo Linux.** O que liga um job ao seu desfecho é o selo (C-11), nunca o tempo |

### 9.5 — Consulta e verificação

| ID | Requisito |
|---|---|
| L-1 | `arca list` lê o dispositivo, nunca um catálogo — se a informação está na listagem de diretórios, não há o que armazenar |
| L-2 | Pasta sem `MD5SUMS` aparece como **resíduo**, não como imagem, e nunca é oferecida para restaurar |
| L-3 | A descrição de uma pasta é um `arca-descricao.txt` **dentro dela**, escrito pelo usuário e nunca pelo ARCA. Ausência do arquivo é ausência de descrição, jamais erro; ela aparece só no `arca list`, e nada julga por ela. *(27/08/2026. Um arquivo por pasta, e não um índice: L-1 proíbe o catálogo porque um índice central afirma coisas sobre pastas que ninguém olhou e envelhece sozinho na primeira renomeada à mão — dentro da pasta, a descrição anda junto da imagem, como o `arca-check.log`. Não entra em receita nenhuma, então C-2 não a alcança e acento é livre; o nome da imagem continua sob B-2. O `MD5SUMS` não a lista, então ela entra na contagem de arquivos fora do `MD5SUMS`, que nunca é falha.)* |
| V-1 | `arca verify <nome>` confere os `MD5SUMS` no Windows, **sem reiniciar**. Pega corrupção de mídia e cópia truncada. *(Etapa E11: o requisito dizia **"em segundos"**, e isso era uma afirmação sobre 39,7 GB que ninguém tinha medido. Medido em 23/08/2026: **202,6 s** — 3 min 23 s, a 200,5 MB/s —, e o comando confirmou com 199,4 s e 202,8 s. O que a tela diz agora não é outro número fixo: ela **estima pelo tamanho real**, com a taxa medida, e diz de onde o número veio. Ver [ADR-0016](../docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md).)* |
| V-2 | `arca verify <nome> --completo` arma boot único que só roda `ocs-chkimg`. É outra força de verificação: **não substitui B-9**, que continua obrigatória em todo backup. *(Etapa E11: as duas respondem perguntas **diferentes**, e é isso que faz as duas existirem — V-1 pergunta "os bytes são os que o Clonezilla gravou?" e V-2 pergunta "esta imagem é restaurável?". Um `.zst` intacto byte a byte que carregue dentro de si um NTFS inconsistente **passa em V-1 e reprova em V-2**. Medido: V-1 leva 3 min 23 s e zero reinícios; V-2 levou 5 min 12 s em 22/08 e custa um reinício — e o que separa as duas na prática é o reinício, não os dois minutos.)* |

#### A tela de V-1, e ela é execução real

Rodada em 23/08/2026 sobre a `2026-08-22_Apps`, do dispositivo desta mesa —
**202,8 s**, e a estimativa da terceira linha acertou o segundo. Abreviada nas
trinta e sete linhas do meio:

```
> arca verify 2026-08-22_Apps

Dispositivo ARCA: ARCAVAULT (D:) · 125 GB livres
Imagem: 2026-08-22_Apps · 22/08 · 39,7 GB

  MD5SUMS lido .................... 39 arquivos · D:\2026-08-22_Apps\MD5SUMS
  A conferir ...................... 39,7 GB

Conferindo 39 arquivos · 39,7 GB. Estimativa: 3 min 23 s.
A tela vai andando um arquivo por vez — parada nao e travamento.

  [ 1/39] blkdev.json ..................... ok
  [ 2/39] blkdev.list ..................... ok
  ...
  [24/39] nvme0n1p3.ntfs-ptcl-img.zst.aa .. ok
  ...
  [39/39] parts ........................... ok

  Conferidos ...................... 39 de 39 · 39,7 GB lidos
  Fora do MD5SUMS ................. 4 arquivos · normal — o proprio MD5SUMS e o que nasce depois dele
  Veredito ........................ APROVADA — os bytes sao os que o Clonezilla gravou

  Isto conferiu que os bytes nao mudaram desde o backup. NAO conferiu que a
  imagem e restauravel — quem responde isso e o `ocs-chkimg`, e para isso ha
  `arca verify <nome> --completo`, que custa um reinicio.
```

> **`D:` e não `E:`, e isso não é erro de transcrição.** Todas as outras telas
> deste documento mostram o `ARCAVAULT` em `E:`; nesta sessão ele veio em `D:`.
> A letra muda de uma conexão para outra e o rótulo não, que é exatamente o que
> B-1 e S-3 dizem — e é a primeira vez que o documento tem os dois valores lado
> a lado para prová-lo.

> **A coluna do andamento não é a coluna de 33 do §5.2.** Ela sai do maior nome
> da lista, e a razão apareceu **rodando o comando**: os nomes que o Clonezilla
> dá aos pedaços de uma partição têm trinta caracteres, e com a coluna fixa
> **catorze das trinta e nove linhas estouravam** com um ponto só. O caso que
> `formato::linha` trata como excepcional é aqui o caso normal.

> **A linha `Fora do MD5SUMS` nunca é problema, e é o que ela existe para
> dizer.** A pasta tem 43 arquivos e o `MD5SUMS` lista 39. Os quatro que sobram
> têm hora: o `MD5SUMS`, o `clonezilla-img` e o `Info-img-id.txt` levam o
> **mesmo mtime** — 18:00:49, o fim do `savedisk` —, e o `arca-check.log` é de
> 18:06:02, escrito cinco minutos depois pelo `ocs-chkimg` de B-9. Não é falta:
> é a hora em que cada um nasceu, e chamar isso de falha reprovaria toda imagem
> que o Clonezilla já fez.

**E o caminho de reprovação rodou**, sobre uma pasta montada de propósito com
um resumo errado e um arquivo ausente. A tela sai inteira antes do erro, com
cada falha nomeada, e o comando sai com código diferente de zero (S-5):

```
  [1/3] disk .... ok
  [2/3] parts ... NAO BATE · o MD5SUMS diz 000000000000 e o arquivo soma b9c383232530
  [3/3] sumido .. AUSENTE · o MD5SUMS o lista e ele nao esta na pasta da imagem
```

**`AUSENTE` e `NAO DEU PARA LER` são linhas diferentes**, e a distinção é a
mesma que a E5 pagou caro para existir: *"não consegui olhar" nunca vira "não
há nada lá"*. O `certutil` responde `0x80070002` para arquivo ausente, e cair
nesse ramo faria as duas chegarem iguais — por isso quem responde sobre
existência é o sistema de arquivos, antes de o `certutil` ser chamado.

### 9.6 — Preparação de dispositivo

| ID | Requisito |
|---|---|
| PR-1 | Versão do Clonezilla **fixada**, com o SHA256 esperado **compilado no binário do ARCA** — nunca baixado junto do arquivo, o que não verificaria nada. Não batendo, recusar e parar. *(Etapa E10, 23/08/2026: a versão é **`3.3.3-15`**, e ela não foi escolhida — é a que está no `hostname=cl-3.3.3-15` do `grub.cfg` deste dispositivo, e sobre a qual rodaram os quatro marcos em hardware deste projeto. O número tem **duas fontes independentes**: o `CHECKSUMS.TXT` do mirror do projeto em `free.nchc.org.tw` e o `certutil` sobre o arquivo baixado do **SourceForge** — servidores diferentes, o mesmo `00cee770…6046304`. E a conferência acontece **antes de extrair**: extrair antes tornaria a conferência decorativa. Ver §7.2 e [ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md).)* |
| PR-2 | `arca prepare --iso <caminho>` instala de arquivo local. É o que salva quando a máquina que precisa preparar o dispositivo é justamente a que está sem Windows. *(Etapa E10: **rodou em 23/08/2026** — conferiu o mesmo SHA256 sem passar pelo `curl`.)* |
| PR-3 | Guardar no `ARCAVAULT` uma cópia do pacote usado. Dispositivo autocontido inclui poder reconstruir o dispositivo. *(Etapa E10: a cópia é feita **depois** de o SHA256 passar. Guardar um pacote que não passou seria guardar lixo com cara de fonte confiável.)* |
| PR-4 | **`arca prepare` imprime o plano inteiro antes de agir, pergunta se pode continuar, e só então escreve.** O plano nomeia o disco — índice, modelo, `MediaType`, tamanho — e **o que existe nele hoje**, com rótulo, sistema de arquivos e tamanho de cada partição: quem vai perder dados tem de poder reconhecê-los na tela antes. A escrita só começa depois da confirmação digitada de S-2. *(Pedido em 23/08/2026. O sujeito mudou junto com P1: as instruções eram "como particionar no Gerenciamento de Disco" e passaram a ser "o que vai acontecer com este disco".)* *(Etapa E10, construído: são **quatro tempos**, e o terceiro é o que faz os outros valerem — o plano; a pergunta; **a conferência, que é do ARCA e não do usuário**; e a confirmação digitada. O ARCA relê o disco entre o "sim" e a primeira escrita, e compara modelo e tamanho com o que imprimiu: a resposta do usuário diz que ele **quer** prosseguir, e não é evidência sobre o disco. O índice muda quando se conecta um cabo, medido nesta mesa. E o plano diz também **o que não é o disco**: que uma entrada de boot vai ser criada no firmware e tirada da ordem permanente, e que o `arca.exe` vai para o `ARCABOOT` — quem lê um plano antes de apagar um disco tem o direito de saber que o plano não para no disco.)* |
| PR-5 | **`arca prepare` cria as duas partições e as rotula**, transcrevendo a estrutura medida em §7.1 — MBR, NTFS grande para o `ARCAVAULT`, FAT32 de ≥ 1 GB no fim para o `ARCABOOT`. **Sete defesas, e nenhuma opcional**: `MediaType` removível ou externo; não ser o disco do `%SystemDrive%`, nem `IsSystem`, nem `IsBoot`; disco escolhido por `--dispositivo <índice>` e **nunca deduzido**, mesmo havendo um só candidato; o plano na tela (PR-4); confirmação digitada (S-2); `--dry-run` de primeira classe; e releitura do disco depois de escrever, no espírito de C-3. **Disco fixo é recusa dura, sem opção de forçar** — o modo de falha apaga o Windows de alguém, e nenhuma confirmação compra isso. *(Ver [ADR-0014](../docs/adr/0014-o-arca-particiona-o-dispositivo.md). A objeção que ficou registrada e virou esta lista: o perigo não é particionar, é acertar em qual disco — e `arca prepare` roda antes de existirem os rótulos que B-1, S-3 e C-10 usariam.)* *(Etapa E10, construído e **rodado em 23/08/2026**. Três coisas que a medição à mão mudou no desenho: o `New-Partition` cria as duas com `MbrType 6`, e **quem acerta para 7 e 12 é o `Format-Volume`** — o tipo é efeito colateral de outra operação, e é por isso que a releitura importa; as duas nascem **sem letra**, e quem atribui é o `Add-PartitionAccessPath -AssignDriveLetter`, que **não é idempotente** e cuja recusa não muda nada, como o `bcdedit /deletevalue` do ADR-0005; e `IsActive` sai `False` sozinho, que é o que a captura registra. A releitura confere a estrutura inteira — os dois rótulos, os dois sistemas de arquivos, os dois `MbrType`, a unidade 4096, **nenhuma partição ativa** e a ordem das duas no disco. A defesa por `MediaType` recusa também o **desconhecido**: supor que o que não se classifica é externo faria a defesa passar batido justamente onde ela mais importa. E a do disco de sistema tem **dois canais** — o `IsSystem`/`IsBoot` do `MSFT_Disk`, que fala do boot corrente, e a letra do `%SystemDrive%`, que fala de onde este Windows mora: numa máquina com dois Windows as duas divergem, e o vão entre canais de identidade é onde este projeto já errou.)* |
| PR-6 | **`arca prepare` lê o firmware antes do ponto sem volta, e o `device` é a primeira escrita depois dele.** Apagar um dispositivo ARCA existente (passo 5) deixa a entrada de firmware apontando para uma partição que não existe mais — e é o próprio `prepare` que a deixa assim. Por isso a ordem permanente e a entrada a reusar são lidas **antes** do passo 5, com o firmware coerente, numa chamada só ao `firmware`; o plano diz qual dos dois vai acontecer — *reapontada* ou *criada*; e o primeiro `bcdedit` depois do apagar é o `/set device` para o `ARCABOOT` novo, que é o comando medido como o que devolve o firmware a um estado que o `bcdedit` aceita. **Sem conseguir ler o firmware, o `prepare` não apaga nada** — a recusa vem antes do plano, e custa rodar de novo. E é ali que C-4 decide: uma leitura que veio com código serve para **reusar**, nunca para **criar** (`EntradaNaoNasceDeLeituraRecusada`). *(27/08/2026, nasce da medição de C-15. Ver [ADR-0026](../docs/adr/0026-a-recusa-do-bcdedit-nao-apaga-o-que-ele-listou.md).)* |

### 9.7 — Sondagem

Acrescentada na etapa E12. Ela existe por uma consequência do §4.5 que só
apareceu quando o ARCA passou a **criar** dispositivos: o nome do disco no Linux
sai do `blkdev.list` de dentro de uma imagem, e um dispositivo recém-preparado
não tem imagem — logo `arca backup` recusa, e `arca restore` e `arca verify
--completo` também.

| ID | Requisito |
|---|---|
| SD-1 | **`arca sondar` arma um boot único que não faz backup nem restauração**: a receita roda `lsblk`, grava a saída no `ARCAVAULT` e desliga. Ela **não chama programa nenhum do Clonezilla** — sem `ocs-sr` não há `savedisk` nem `restoredisk`, e sem `ocs-chkimg` não há escrita dentro de pasta de imagem —, e **nada é escrito fora do `ARCAVAULT`**. É a única operação deste projeto cujo pior caso não envolve gravação: o pior é a máquina parar num menu (§3.2, §4.4) |
| SD-2 | **A saída sai no formato que o §4.5 já sabe ler**, e pelo **mesmo parser**: `crate::blkdev` continua sendo o único lugar do ARCA que lê aquele formato, e o arquivo leva o mesmo nome — `blkdev.list`. As colunas são **reconstruídas** a partir do cabeçalho capturado (`KNAME NAME SIZE TYPE FSTYPE MOUNTPOINT MODEL`), e não transcritas de linha de comando nenhuma: a que produziu aqueles arquivos mora nos scripts do Clonezilla, dentro do `filesystem.squashfs`. O código diz que é reconstrução, e o `--dry-run` diz na tela |
| SD-3 | **O `lsblk` roda dentro de um `if`** (R-5), escrevendo `ARCA_PROBE=OK` ou `ARCA_PROBE=FALHOU`. Encadear com `;` — a forma proposta na mesa — escreveria `OK` sobre um `lsblk` que falhou, e a contradição apareceria na mesma sessão: o `arca resultado` diria que a sondagem concluiu, e a tela seguinte diria `Disco de origem … POR DETERMINAR`. **E o `2>&1` aponta para o próprio `blkdev.list`**: uma flag recusada deixa a mensagem do `lsblk` no dispositivo em vez de sumir com o `poweroff`, o que é o que torna a reconstrução de SD-2 aceitável — o modo de falha custa um reinício e diz o que consertar |
| SD-4 | **A sondagem grava em pasta fixa** — `ARCA-LOGS\sondagem\`, com o `arca-fim.txt` e o `blkdev.list` juntos —, e a sondagem anterior é **substituída**. Ela não nomeia imagem, então o `nome` do `Pedido` e do `estado.json` é ausente, com a **string vazia** como sentinela (o mesmo argumento do `disco` da E11: `Nome::novo("")` recusa desde a E1). Substituir é o certo aqui, e a diferença é o que se perde: entre backup e restauração perdia-se o desfecho de **outro job**; aqui perde-se a **medição anterior da mesma pergunta**, e a mais recente é a que vale |
| SD-5 | **Havendo sondagem e `blkdev.list` de imagem, a sondagem ganha**, e a divergência é **dita na tela** — nunca resolvida em silêncio. A sondagem descreve a máquina de agora; a imagem descreve a de quando o backup foi feito. A saída sempre diz de qual das duas o nome veio, e **quando** a sondagem foi feita. `SemOraculo` é a única recusa da sondagem que deixa as imagens falar: as outras são afirmações sobre a máquina de agora, e `ModeloAmbiguo` resolvido por uma imagem antiga seria o chute que aquela recusa existe para não dar |
| SD-6 | **A confirmação é uma tecla, com o padrão no não**, e não o texto por extenso de S-2. A decisão sobrevive à pergunta *"o que essa confirmação impede?"*: ela impede o **reinício** de quem digitou o comando sem saber que ele reinicia, e a tela diz isso imediatamente acima dela. S-2 pede o **alvo** por extenso e existe para custar lê-lo; a sondagem não tem alvo — não apaga nada e não escolhe nada. Pedir a palavra `sondar` por extenso seria ecoar o comando, e uma confirmação que só ecoa ensina a digitar sem ler |

> **O que a sondagem mediu de graça, e nenhuma etapa tinha medido:** quanto custa
> o boot do Clonezilla live nesta máquina, **isolado**. Todas as execuções
> anteriores tinham uma operação longa depois dele — 39,7 GB gravados, uma
> restauração, um `ocs-chkimg` de 312 s —, e o boot ficou embutido em cada total.
>
> **1 min 40 s** do reinício ao desligamento, cronometrado à mão em 24/08/2026.
> Dois pedaços desse total vêm de outra medição:
>
> ```text
> 1 min 40 s   do reinicio ao desligamento (cronometrado)
>    − 30 s    o menu do grub, parado (o `set timeout` do grub.cfg)
>    − 20 s    o `sleep` antes do poweroff (a receita)
>    ────────
>    ≈ 50 s    POST + kernel + initrd + `toram` + o live subir + o `lsblk`
> ```
>
> **Os 50 s são aritmética sobre um número cronometrado**, e não uma terceira
> medição — valem como ordem de grandeza, e não como o `202,6 s` de V-1.
>
> **E a tela do `arca sondar` continua sem prometer tempo nenhum**, com teste
> cobrando isso: o número vive aqui, com a decomposição à vista, e não numa
> frase que promete o mesmo para qualquer máquina.

## 10. Implementação

> **A receita não é um script.** As versões anteriores deste documento
> mostravam as duas receitas como `#!/bin/bash` de várias linhas. Isso nunca
> existiu: o que roda é **uma string única**, dentro de
> `ocs_live_run="bash -c '...'"`, numa linha só do `grub.cfg` — como o
> [ADR-0002](../docs/adr/0002-receita-como-string-no-grub.md) decidiu e as
> três receitas preservadas em `recursos/capturas/` comprovam. As seções
> abaixo mostram a forma real. A indentação existe só para caber na página;
> no `grub.cfg` tudo é uma linha.

### 10.1 — Receita de backup

Gerada por `src/receita.rs`, para `NOME=2026-08-22_Apps`, `DISCO=nvme0n1` e um
selo de exemplo:

```text
mkdir -p /home/partimag/ARCA-LOGS/backup-2026-08-22_Apps;
echo ARCA_SELO=a3f1c9e07b2d4856 > /home/partimag/ARCA-LOGS/backup-2026-08-22_Apps/arca-fim.txt;
if ocs-sr -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true savedisk 2026-08-22_Apps nvme0n1;
  then echo ARCA_BACKUP=OK >> /home/partimag/ARCA-LOGS/backup-2026-08-22_Apps/arca-fim.txt;
    if ocs-chkimg -b -or /home/partimag 2026-08-22_Apps > /home/partimag/2026-08-22_Apps/arca-check.log 2>&1;
      then echo ARCA_VEREDITO=APROVADA >> /home/partimag/2026-08-22_Apps/arca-check.log;
      else echo ARCA_VEREDITO=REPROVADA >> /home/partimag/2026-08-22_Apps/arca-check.log;
    fi;
  else echo ARCA_BACKUP=FALHOU >> /home/partimag/ARCA-LOGS/backup-2026-08-22_Apps/arca-fim.txt;
fi;
echo ARCA_FIM >> /home/partimag/ARCA-LOGS/backup-2026-08-22_Apps/arca-fim.txt;
sleep 20;
poweroff
```

A verificação de B-9 mora **dentro** do ramo de êxito: com o `savedisk`
falhando, a pasta da imagem pode nem existir, e o redirecionamento do
`ocs-chkimg` falharia junto do `else` dele.

O `ARCA_VEREDITO=` é acrescentado ao `arca-check.log` porque é o marcador que
o leitor prefere ([ADR-0003](../docs/adr/0003-veredito-lido-do-arca-check-log.md)),
e escrevê-lo tira o veredito da dependência de interpretar frases em inglês do
`ocs-chkimg`.

### 10.2 — Receita de restauração

```text
mkdir -p /home/partimag/ARCA-LOGS/restauracao-2026-08-22_Apps;
echo ARCA_SELO=7e02b4d1af963c85 > /home/partimag/ARCA-LOGS/restauracao-2026-08-22_Apps/arca-fim.txt;
if ocs-sr -e1 auto -e2 -batch -j2 -k0 -iefi -p true restoredisk 2026-08-22_Apps nvme0n1 > /home/partimag/ARCA-LOGS/restauracao-2026-08-22_Apps/arca-restore.log 2>&1;
  then echo ARCA_RESTORE=OK >> /home/partimag/ARCA-LOGS/restauracao-2026-08-22_Apps/arca-fim.txt;
  else echo ARCA_RESTORE=FALHOU >> /home/partimag/ARCA-LOGS/restauracao-2026-08-22_Apps/arca-fim.txt;
fi;
echo ARCA_FIM >> /home/partimag/ARCA-LOGS/restauracao-2026-08-22_Apps/arca-fim.txt;
sleep 20;
poweroff
```

O `LOG` mora no `ARCAVAULT`, que a restauração não toca — a imagem substitui o `nvme0n1`, e o desfecho sobrevive num disco que não estava no caminho. Sem verificação: B-9 é do backup, e aqui não há imagem nova para conferir.

**A pasta do log leva a operação, e não só o nome da imagem.** Toda receita começa truncando o próprio `arca-fim.txt` com um `>`. Se as duas dividissem o caminho, um `arca restore X` rodado antes de o backup de X ser colhido apagaria o desfecho dele, e §5.5 leria um backup bem-sucedido como desfecho ausente. O selo não cobre isso: ele julga um desfecho **encontrado**, e não serve para nada quando o arquivo já foi por cima.

### 10.2.4 — Receita de verificação

Acrescentada na etapa E11, e é a menor das três. Gerada por `src/receita.rs`
para `NOME=2026-08-22_Apps` e um selo de exemplo:

```text
mkdir -p /home/partimag/ARCA-LOGS/verificacao-2026-08-22_Apps;
echo ARCA_SELO=a3f1c9e07b2d4856 > /home/partimag/ARCA-LOGS/verificacao-2026-08-22_Apps/arca-fim.txt;
if ocs-chkimg -b -or /home/partimag 2026-08-22_Apps >> /home/partimag/2026-08-22_Apps/arca-check.log 2>&1;
  then echo ARCA_VEREDITO=APROVADA >> /home/partimag/2026-08-22_Apps/arca-check.log;
    echo ARCA_VERIFY=OK >> /home/partimag/ARCA-LOGS/verificacao-2026-08-22_Apps/arca-fim.txt;
  else echo ARCA_VEREDITO=REPROVADA >> /home/partimag/2026-08-22_Apps/arca-check.log;
    echo ARCA_VERIFY=FALHOU >> /home/partimag/ARCA-LOGS/verificacao-2026-08-22_Apps/arca-fim.txt;
fi;
echo ARCA_FIM >> /home/partimag/ARCA-LOGS/verificacao-2026-08-22_Apps/arca-fim.txt;
sleep 20;
poweroff
```

**Ela não nomeia disco nenhum**, e é a única das três assim: o `ocs-chkimg`
opera sobre a **imagem**. É por isso que o campo `disco` do `estado.json`
passou a ser opcional na E11, com a string vazia dizendo "nenhum" — e o vazio
foi escolhido porque `Disco::novo("")` já recusava desde a E3, então ele nunca
poderia colidir com um nome que o Linux dê ([ADR-0016](../docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md)).

**O `>>` no `arca-check.log` é a diferença que importa, e o backup usa `>`.**
Lá a imagem acabou de nascer e o log não existe; aqui ele existe, e é o
veredito do backup que a criou. Um `>` o destruiria — e faria a linha
`Imagem de origem: APROVADA — veredito do backup que a criou` do §6.3 virar
mentira. Pior: o `>` **trunca ao abrir**, antes de o comando rodar, e um
desligamento nessa janela deixaria uma imagem **boa** aparecendo `sem veredito`
na listagem.

Com `>>`, o [ADR-0003](../docs/adr/0003-veredito-lido-do-arca-check-log.md)
vale como está escrito: as duas marcas ficam no arquivo, e o leitor lê **toda
forma de reprovar antes de toda forma de aprovar** — uma imagem que já reprovou
continua reprovada, mesmo que a verificação nova aprove. É o lado conservador
de propósito, e é o que S-5 pede.

**O que aqui era código novo**: o `ARCA_VERIFY=`, e o `ocs-chkimg` como comando
principal em vez de aninhado dentro do ramo de êxito de um `savedisk`. Tudo o
mais é transcrição — a chamada do `ocs-chkimg` vem de `ARCA-TESTE-03`, e o `if`
que escreve o `ARCA_VEREDITO=` rodou em 22/08/2026.

> **Esta receita rodou em 23/08/2026**, no marco da E11, e fecha P-24. O
> desfecho está em `recursos/capturas/arca-fim-verificacao-2026-08-22_Apps.txt`:
> `ARCA_SELO=aefa48f71fc66a46`, `ARCA_VERIFY=OK`, `ARCA_FIM` — cinquenta e um
> bytes, três linhas, o selo batendo com o do `estado.json` do mesmo job.
>
> **E uma parte dela não fez o que a receita diz.** O `>>` devia acrescentar ao
> `arca-check.log`, e o arquivo saiu com **uma** execução do `ocs-chkimg` — o
> log do backup de 22/08 sumiu. É **P-25**, a primeira vez neste projeto em que
> uma receita rodou e o rastro dela divergiu do que a string manda fazer. O
> `>>` fica assim mesmo, e a razão trocou: ele não compra a preservação, mas
> não abre a janela em que o `>` deixaria o log em zero byte
> ([ADR-0016](../docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md)).

### 10.2.5 — Receita de sondagem

Acrescentada na etapa E12, e é a menor das quatro. Gerada por `src/receita.rs`
com um selo de exemplo:

```text
mkdir -p /home/partimag/ARCA-LOGS/sondagem;
echo ARCA_SELO=a3f1c9e07b2d4856 > /home/partimag/ARCA-LOGS/sondagem/arca-fim.txt;
if lsblk -i -o KNAME,NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT,MODEL > /home/partimag/ARCA-LOGS/sondagem/blkdev.list 2>&1;
  then echo ARCA_PROBE=OK >> /home/partimag/ARCA-LOGS/sondagem/arca-fim.txt;
  else echo ARCA_PROBE=FALHOU >> /home/partimag/ARCA-LOGS/sondagem/arca-fim.txt;
fi;
echo ARCA_FIM >> /home/partimag/ARCA-LOGS/sondagem/arca-fim.txt;
sleep 20;
poweroff
```

**Ela não chama programa nenhum do Clonezilla**, e é a única das quatro assim.
Não há `ocs-sr`, logo não há `savedisk` nem `restoredisk`; não há `ocs-chkimg`,
logo não há escrita dentro de pasta de imagem. Tudo o que ela escreve mora no
`ARCAVAULT` (SD-1).

**Ela também não nomeia imagem**, e é a única das quatro assim. É por isso que a
pasta do log é fixa — `sondagem`, e não `sondagem-<nome>` — e que o campo `nome`
do `estado.json` passou a ser opcional na E12, com a string vazia dizendo
"nenhuma". O vazio foi escolhido pelo mesmo argumento do `disco` da E11:
`Nome::novo("")` recusa desde a E1, então ele nunca poderia colidir com um nome
que B-2 aceite — e `sondagem`, que seria o sentinela óbvio, **colidiria**
([ADR-0019](../docs/adr/0019-a-sondagem-e-a-quarta-operacao.md)).

**O `if` é R-5, e a primeira forma escrita desta receita não o tinha.** A
proposta encadeava com `;`, e o `;` não olha código de saída: com o `lsblk`
falhando — uma flag que esta versão do util-linux não conheça basta —, o desfecho
diria `OK` assim mesmo. Medido num bash de verdade: `recursos/ensaio-da-receita.sh`
roda as duas formas lado a lado, e a com `;` escreve `ARCA_PROBE=OK` sobre um
`lsblk` que saiu com código diferente de zero.

**O `2>&1` aponta para o próprio `blkdev.list`**, e não para um log à parte. Com
o `lsblk` falhando, o arquivo fica com a mensagem de erro dele em vez de vazio, e
a próxima sessão lê **qual** flag foi recusada. Um arquivo assim não é lido como
oráculo: o cabeçalho não bate, e o parser devolve lista vazia.

#### As flags do `lsblk` são uma terceira procedência

O §10.2.2 tem duas: **transcrito** (há captura da linha de comando) e **código
novo** (não há original nenhum). A sondagem estreia a terceira:

> **Reconstrução** — há original do **resultado**, e não da linha que o produziu.

O `blkdev.list` de dentro de cada imagem traz o cabeçalho
`KNAME NAME SIZE TYPE FSTYPE MOUNTPOINT MODEL`, e o `-o` reproduz exatamente
essas sete colunas, nessa ordem. A linha que o Clonezilla usou mora nos scripts
dele, dentro do `filesystem.squashfs`, que este repositório nunca abriu.

O `-i` (`--ascii`) é parte da reconstrução e tem razão própria: o arquivo
capturado desenha a árvore com `|-` e `` ` ``, e o `lsblk` só escolhe esses
símbolos quando o `CODESET` do locale não é UTF-8. A receita boota com
`locales=en_US.UTF-8`, que §3.2 torna obrigatório — sem `-i`, a árvore sairia em
Unicode e o arquivo deixaria de ter a forma do que ele imita.

#### O pressuposto novo já tinha original, e ninguém tinha notado

Esta receita escreve em `/home/partimag` **antes** de qualquer comando do
Clonezilla. Se o repositório não estivesse montado nesse instante, o `mkdir`
criaria a pasta no tmpfs da RAM e o `poweroff` levaria tudo embora.

Está provado, e a prova é da E11: a receita de verificação (§10.2.4) tem
exatamente esta forma, rodou em 23/08/2026 às 16:53, e os 51 bytes de
`recursos/capturas/arca-fim-verificacao-2026-08-22_Apps.txt` saíram dos dois
primeiros passos dela. **Quem monta o `/home/partimag` é o `ocs_repository=` do
boot, e não o `ocs-sr`.**

E há um segundo sinal, de graça: o `lsblk` roda com o repositório montado, então
a linha da partição do `ARCAVAULT` sai com `/home/partimag` no `MOUNTPOINT` —
como já sai nos `blkdev.list` capturados. O próprio arquivo testemunha que foi
escrito no lugar certo.

### 10.2.3 — O orçamento da linha de comando

A receita inteira vira uma linha só, e o `COMMAND_LINE_SIZE` do kernel no x86_64 é **2048 caracteres**. Estourar não dá erro: o kernel **trunca em silêncio**, e uma receita truncada é uma string inválida — o caso do §3.2, em que o Clonezilla descarta tudo e abre o menu.

O nome da imagem aparece dez vezes na receita de backup, e cada caractere a mais custa dez na linha. O orçamento:

| | |
|---|---|
| Teto do kernel | 2048 |
| Reservado para o `menuentry` base | 512 (medido nas capturas: 206, 369, 369) |
| Sobra para o que o ARCA gera | 1536 |
| Receita de backup com o nome mais longo que B-2 aceita (48) | 1271 |

A recusa acontece nos dois pontos: B-2 limita o nome a 48 caracteres, e a montagem da receita confere os cinco parâmetros que gera contra os 1536 — porque o limite do nome é uma estimativa e o tamanho da linha é o fato.

#### O orçamento medido contra a linha que rodou

O marco em hardware de 22/08/2026 rodou uma linha montada pelo ARCA, e ela pode
ser medida: `cargo run --example orcamento_da_linha_do_kernel`. O que a medição
mostra, para o backup `2026-08-22_Apps` (nome de 15 caracteres):

Medido em **bytes**, sem o recuo do bloco — que é do `grub.cfg` e não da linha
que o kernel recebe:

| | Orçado | Medido |
|---|---|---|
| `menuentry` base — a linha do `live-toram` deste dispositivo | 512 | **471** |
| Os cinco parâmetros que o ARCA gera | 1536 | **941** |
| A receita sozinha, dentro do `ocs_live_run` | — | **813** |
| **A linha inteira, como o kernel a recebeu** | 2048 | **1334** — 65% do teto |
| Os cinco parâmetros, com o nome de 48 | 1271 | **1271** |

Três coisas saem daí:

- **O `1271` estava exato**, e agora se sabe o que ele mede: os cinco
  parâmetros, e não a receita sozinha (1143) nem a linha pronta (1664).
- **A reserva de 512 estava apertada, e não folgada.** As três capturas mediam
  206, 369 e 369, e o texto acima diz que reservar 512 era "quase 40% acima do
  maior já visto". O `menuentry` base **deste** dispositivo ocupa 471: sobram 41
  bytes, e não 143. As capturas mediam um `menuentry` mais pobre do que o
  modelo de que o ARCA deriva — que é justamente o argumento do ADR-0007 para
  derivar, visto pelo outro lado.
- **A linha que rodou gastou 65% do teto**, e o pior caso que B-2 deixa passar
  gasta 1664, com 384 de folga. O orçamento inteiro cabe, e a folga é real.

> **A unidade é byte, e o ARCA confere em caracteres.** O `COMMAND_LINE_SIZE`
> conta bytes; `Receita::montar` compara `chars().count()` contra os 1536. Para
> texto ASCII os dois números são o mesmo, e hoje a receita é ASCII por
> construção — B-2 recusa nome com acento justamente porque *"o que atravessa o
> grub e o live system é ASCII"*. A diferença fica anotada porque a defesa está
> numa barreira anterior, e não na conferência: afrouxada B-2, o limite da
> linha passaria a contar errado, e a folga de 384 bytes é o que dá margem para
> descobrir isso antes de o kernel truncar.

### 10.2.1 — Como as receitas entram no `grub.cfg`

Os cinco parâmetros que a receita exige na linha `$linux_cmd`, idênticos nas
três capturas:

```text
locales=en_US.UTF-8 keyboard-layouts=NONE ocs_repository="dev:///LABEL=ARCAVAULT" ocs_live_run="bash -c '<a receita>'" ocs_live_batch="yes"
```

O resto da linha — `hostname`, `vga`, `toram`, as blacklists de driver — é do
`menuentry` base do Clonezilla, e não da receita.

> **E o `menuentry` base é o `live-toram`, não o `live-default`.** Corrigido na
> etapa E7. O `live-default` **não tem** `toram`; quem tem é o
> `menuentry --id live-toram`, e ali o
> `toram=live,syslinux,EFI,boot,.disk,utils` está exatamente onde as capturas
> armadas o mostram, logo depois do `vga=788`. **Ninguém acrescentou o
> `toram`** — ele veio junto do modelo.
>
> Medido token a token: a captura `grub-backup-arca-teste-02.cfg` é o
> `live-toram` do `grub.cfg` inerte com **exatamente as cinco substituições
> acima**, e nada mais. É por isso que o ARCA **deriva** o bloco do arquivo que
> está no dispositivo em vez de transcrever um fixo — o `grub.cfg` carrega a
> configuração daquele hardware (`hostname=cl-3.3.3-15`, as blacklists,
> `nvme.poll_queues=1`), e um bloco fixo a descartaria em silêncio.
>
> Ver [ADR-0007](../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md),
> inclusive para o que a `teste-03` — a única cópia com
> `set default="arca-backup"` — perdeu do modelo, e por que isso é argumento a
> favor de derivar.

O `live-default` continua tendo papel, e é outro: é para onde o `set default`
volta no estado inerte (§4.4). O `live-toram` é o **modelo do armar**; o
`live-default` é o **alvo do desarmar**.

### 10.2.2 — O que destas receitas nunca rodou

Distinção que custou uma etapa inteira para aparecer, e que vale mais do que
qualquer das linhas acima:

| Parte | Origem |
|---|---|
| As flags do `ocs-sr` e a ordem delas | Transcrito das três capturas |
| O `ocs-chkimg` com saída redirecionada | Transcrito de `ARCA-TESTE-03` |
| Os cinco parâmetros de boot | Transcrito das três |
| A forma `bash -c '...'` com `;` entre os passos | Transcrito das três |
| O `if/then/else` de R-5 | **Era código novo** — as três encadeiam com `;`. Rodou em 22/08/2026, no ramo do sucesso |
| O `arca-fim.txt`, o `ARCA_SELO=`, o `ARCA_FIM` | **Era código novo** — nenhuma receita real o escrevera. Rodou em 22/08/2026, e o original está em `recursos/capturas/arca-fim-2026-08-22_Apps.txt` |
| O `ARCA_VEREDITO=` no `arca-check.log` | **Era código novo** — ADR-0003. Rodou em 22/08/2026, e o original está em `recursos/capturas/arca-check-2026-08-22_Apps.log` |
| O `sleep 20` | **Era código novo** — nenhuma captura o tem. Rodou em 22/08/2026 |
| O `ARCA_VERIFY=`, e o `ocs-chkimg` fora de um `savedisk` | **Era código novo** — acrescentado na etapa E11 (§10.2.4). **Rodou em 23/08/2026**, e o original está em `recursos/capturas/arca-fim-verificacao-2026-08-22_Apps.txt`. Fecha P-24 |
| O `lsblk` como comando principal, e o `ARCA_PROBE=` | **Código novo** — acrescentado na etapa E12 (§10.2.5). Nenhuma receita deste projeto chamou o `lsblk` |
| As flags do `lsblk` | **Reconstrução** — a terceira procedência, e ela estreia aqui: há original do **resultado** (o `blkdev.list` de dentro das imagens) e não da linha que o produziu (§10.2.5) |
| O `>>` no `arca-check.log` da verificação | **Rodou, e não fez o que se esperava**: o log saiu com uma execução só, e o do backup sumiu. É P-25 — a única linha desta tabela cujo comportamento em hardware **divergiu** do que a receita diz |

Ver [ADR-0004](../docs/adr/0004-a-receita-transcreve-o-que-rodou.md).

> **As quatro linhas de "código novo" rodaram todas de uma vez, em
> 22/08/2026**, e a tabela fica como está porque a distinção que ela registra
> continua valendo: elas **eram** código sem original, e é isso que explica por
> que o marco em hardware da E7 e da E8 era um marco. O que mudou é que agora
> há original para as três primeiras, e ele está em `recursos/capturas/`.
>
> O que continua sem rodar é o **ramo de falha** do `if/then/else` — o
> `ARCA_BACKUP=FALHOU`. Uma execução bem-sucedida não o exercita, por
> definição, e é P-6.

> **Por que `if/then/else` e não `;`.** Encadear com `;` não olha código de saída: uma restauração que falhasse produziria exatamente o mesmo rastro de uma que desse certo.

### 10.3 — Restrições da receita

- **Sem pipes.** Só `>` e `>>`. Um pipe invalida a string inteira e o Clonezilla abre o menu interativo, sem executar nada e sem avisar
- **Nenhuma aspa, nem simples nem dupla.** A receita mora dentro de um `bash -c '...'` que mora dentro de um `ocs_live_run="..."`: uma aspa simples fecha o primeiro, uma dupla fecha o segundo. Não basta que estejam balanceadas — um par de aspas simples fecha a string do `bash` e abre outra, produzindo algo sintaticamente válido e semanticamente diferente
- **Nenhuma substituição de comando** (`` ` ``, `$(`, `${`, `$`). O que se validou deixaria de ser o que roda
- **Nada que não seja ASCII imprimível.** A receita é uma linha só do `grub.cfg`, e uma quebra de linha dentro dela transformaria o resto em outra diretiva do grub
- `locales=en_US.UTF-8` explícito
- `toram` mantido — evita acoplar o live system ao dispositivo que ele remonta
- Validar a string antes de gravar (C-2)

O nome da imagem é a única parte da receita que vem de fora, e ele passa por
B-2 antes: **lista de permissão** (`A-Z a-z 0-9 . _ -`), e não lista de
recusa. Uma lista de recusa só está certa enquanto ninguém esquecer um
caractere, e esquecer um caractere aqui custa uma execução real.

### 10.4 — Stack

Rust + `clap`. Sem interface gráfica, sem banco. O único estado é um arquivo por dispositivo, gravado no `ARCABOOT`.

Manifesto com `requireAdministrator`, repassando argumentos na reelevação.

## 11. Armadilhas conhecidas

Cada uma custou uma execução real para aparecer.

| Armadilha | Efeito | Defesa |
|---|---|---|
| Pipe na receita | Clonezilla ignora tudo e abre o menu — indistinguível de "o boot não funcionou" | C-2 |
| `;` em vez de `if/then/else` | Falha deixa o mesmo rastro que sucesso | R-5 — e a defesa **rodou nos dois ramos**: o do sucesso em 22/08/2026, e o **do erro em 24/08/2026**, numa sondagem armada com uma coluna inventada no `lsblk`. `ARCA_PROBE=FALHOU` é o primeiro `FALHOU` deste projeto, e as duas telas seguintes concordaram — desfecho `FALHOU` e disco `POR DETERMINAR`, que é o que o `;` teria tornado contraditório. **P-6 continua aberta**: ela pergunta pelo `ocs-sr`, e quem falhou aqui foi o `lsblk` |
| Documentar como fundação o que veio do trabalho de validação | O `ARCA_VEREDITO=`, o `arca-fim.txt` de 21/08, o `set default`, o `498,7 GB` e a ordem de boot com o dispositivo à frente pareciam medidas, e vieram do trabalho em volta | Procurar o original em `recursos/capturas/` antes de chamar qualquer coisa de medida. **Em 22/08 o padrão não se repetiu**: o `arca-fim.txt` do marco tem original, e o que atesta que a receita o escreveu é o `Info-saved-by-cmd.txt` que o Clonezilla escreve sozinho |
| Relógio do Clonezilla 3h adiantado | Ele lê o RTC (hora local do Windows) como UTC. Uma trava construída sobre comparação de datas reprovou um backup perfeito | S-6. **Confirmado outra vez em 22/08, pelo outro lado**: o `arca-fim.txt` escrito às 21:06 tem `mtime` de 18:06 lido do Windows, e parece anterior ao job que o produziu. É o mesmo instante em dois fusos, e é por isso que quem liga desfecho a job é o selo. **E de novo em 23/08, com a mesma diferença e o mesmo sinal**: o log diz `Ending /usr/sbin/ocs-sr at 2026-08-23 11:31:55 UTC`, o `mtime` visto do Windows é 08:31:55, e o job foi armado às 11:10:50 — o desfecho parece ter sido escrito quarenta minutos antes de a operação começar |
| **Medir o firmware depois do reinício e achar que se mediu o reinício** | As duas leituras do `bcdedit` de 22/08 discordam entre si, e as duas estão certas: a ordem de boot mudou no meio — e quem a mudou foi o próprio ciclo de boot. Uma leitura feita no Windows descreve o firmware **como ele ficou**. **E não é preciso bootar no dispositivo para isso acontecer**: em 24/08 um religar limpo, direto ao Windows, acrescentou três entradas à ordem ([ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md)) | Ler a NVRAM de dentro do live, que é onde o boot está acontecendo. O Clonezilla já grava `efi-nvram.dat` em toda imagem, de graça (§3.1, [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)) |
| **Contar entradas entre duas capturas e concluir que alguém mexeu** | As três classes de dispositivo do firmware vão e vêm sem causa conhecida: estavam em 20/08, não estavam em 22/08 de manhã, voltaram em 24/08 num religar limpo — e dois boots pelo dispositivo no mesmo dia não as trouxeram. Uma contagem que muda não prova intervenção humana, e este documento já atribuiu a alguém o que era do ciclo de boot | O `node` do UUID separa as origens dentro do próprio arquivo, e o padrão é total nas capturas deste projeto: `806e6f6e6963` é sempre entrada de firmware, `aa4ed9bd2b34` é sempre objeto do BCD ([ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md)) |
| Argumentos perdidos na reelevação | `--dry-run` virou execução real, sem aviso | C-7 |
| Crase como escape | O parser do Windows reparte a linha, não o do PowerShell | C-8 |
| Job fantasma | Imagem feita quando o ARCA ainda morava no `C:` carrega dentro de si um `estado.json` pendente apontando para si mesma. §4.1 elimina a causa daqui para frente; imagens antigas continuam trazendo o problema de volta | C-11 |
| ARCA dentro da imagem | Restaurar devolve versões antigas com defeitos já corrigidos | §4.1 |
| Pasta sem `MD5SUMS` | Resíduo de backup interrompido; recusar só imagem válida empurra o usuário a regravar por cima dos fragmentos | B-3 |
| Boot no removível após `poweroff` | Não reproduzido, causa não determinada | C-9 |
| `set default="0"` no `grub.cfg` | Aponta por **posição**, e o `menuentry` do ARCA entra antes do `live-default`: inserir o bloco arma sozinho, sem ninguém tocar no `set default`. Um dispositivo assim não está inerte, está parecendo inerte | Desarmar devolve o `set default` para `live-default` qualquer que seja o valor que encontrou (§4.4, ADR-0005) |
| `bcdedit /deletevalue` chamando de erro não ter o que apagar | Apagar um `bootsequence` que não existe sai com código 1 sem mudar nada. Um desarmar que propagasse isso falharia justamente no caso normal, e a idempotência de C-1 nunca passaria | Descartar o que o `bcdedit` responde e conferir com `/enum` (C-3) |
| **A ordem permanente com o dispositivo à frente** | A máquina boota no dispositivo **sem** boot único nenhum. Com o dispositivo inerte isso para no menu do Clonezilla; **com uma receita armada, ela roda** — e a janela em que o `grub.cfg` fica armado vai do fim da receita ao `arca resultado`, oito minutos em 22/08. E não é preciso ninguém para chegar nesse estado: o ciclo de boot põe a entrada de volta na ordem (ADR-0009) | **C-13**, desde 23/08/2026: ao colher, o `{bootmgr}` volta ao topo, e a partir dali toda religada sobe o Windows. `arca status` continua **avisando** e não consertando — é diagnóstico, e um comando de consulta que escreve na NVRAM seria outra coisa. `tests/e7_armar_o_dispositivo.rs` cobra a invariante que sobra: em primeiro na ordem, o dispositivo tem de estar inerte. E a defesa da **janela** continua sendo C-9 — remover o SSD antes de religar —, porque o conserto só acontece na colheita, que vem depois ([ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md)) |
| Ler duas ferramentas em momentos diferentes e chamar a diferença de discordância | O `bcdedit` de 22/08 e o `efibootmgr` de 20/08 dizem coisas diferentes sobre a ordem de boot, e as duas estão certas: a ordem mudou no meio. Datar cada captura desfaz a contradição inteira. **Em 22/08 aconteceu com a mesma ferramenta**, duas leituras separadas por trinta e seis minutos | A tabela do §3.1 traz a data — e agora a hora — de cada leitura, e `recursos/capturas/PROVENIENCIA.md` diz de onde cada arquivo veio |
| **Datar a captura e não saber de que operação ela é** | Duas leituras de NVRAM de 21/08 estavam na tabela do §3.1 como uma linha só, explicando o backup daquele dia. Uma é do backup e a outra é da **restauração**, uma hora e meia depois, com um boot no Windows no meio — e elas discordam sobre a ordem de boot justamente por isso. O que as juntou foi o nome da pasta, que a E3 escolheu por outro motivo inteiramente | Datar **e nomear a operação**. A tabela do §3.1 traz as duas separadas, e a `PROVENIENCIA.md` diz qual operação escreveu cada arquivo ([ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md)) |
| **Duas medidas do mesmo disco, em réguas diferentes** | O `MSFT_Disk` e o `Win32_DiskDrive` respondem 500.107.862.016 e 500.105.249.280 para o **mesmo** disco. O segundo é `60801 × 255 × 63 × 512` — a geometria CHS legada truncada no último cilindro. Medir a origem pela GPT de dentro da imagem e o destino pelo `Win32_DiskDrive` faz R-7 recusar um disco por não caber **nele mesmo** | R-7 mede o destino pelo `MSFT_Disk` e compara em setores; `DiscoFisico::medida` existe ao lado do `tamanho_bytes` com doc dizendo por quê, e há teste que afirma o número errado para que trocar a fonte de volta não seja silencioso ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)) |
| **Uma recusa por identidade do Windows guardando um valor do Linux** | R-8 recusa o dispositivo como destino **pela letra** (`E:`, `R:`); o nome que entra na receita é do **Linux**, e sai de um casamento por **modelo** nos `blkdev.list` (§4.5). São dois canais de identidade, e o vão entre eles apagava o dispositivo: com um segundo disco do mesmo modelo dele, `--destino <o outro>` passava pela recusa por letra e a receita saía `restoredisk <imagem> sda` — o `sda` é o dispositivo. A recusa dura tinha um contorno por acidente de modelo | Resolver o nome do Linux do **dispositivo** pelo mesmo oráculo e comparar com o do destino. Achado pela revisão de código da E9, e é o defeito mais grave que ela pegou |
| **Uma identidade vazia casando com tudo ou com nada** | O `Model:` do `<disco>-gpt.sgdisk` era opcional, e um modelo vazio viajava: a conferência de R-2 recusava uma imagem coerente por "as fontes discordam", a busca do destino dizia "nenhum disco tem o modelo ``", e a tela de confirmação imprimia `Origem da imagem:  · nvme0n1`. É o mesmo raciocínio que faz o leitor do WMI exigir o `Model` em vez de supor | Exigir o `Model:`, com recusa própria. O modelo é a identidade do disco, e o ARCA não confere um destino contra identidade nenhuma |
| **A recusa engolindo a notícia do desarmar** | C-1 desarma incondicionalmente, como primeiro passo; uma recusa posterior sobe como erro e corta a saída. Quem rodasse `arca restore --destino <errado>` num dispositivo armado veria só a recusa, e o job armado teria sumido em silêncio. **Aconteceu duas vezes**: a revisão da E7 pegou no `arca backup`, e a E9 cometeu de novo no `arca restore` — com o comentário que descreve o defeito a poucas linhas de distância | Imprimir o que já aconteceu **antes** de julgar. As duas telas saem em duas metades, e a primeira traz o desarmar. Achado rodando o comando de verdade, e não relendo o código |
| **Um par honesto respondendo por um evento maior do que ele** | O §3.4 dizia que a restauração não mexe na NVRAM, sustentado num par `antes`/`depois` real, do mesmo evento, com hora — nada dele veio de trabalho manual. Só que as duas leituras são **do mesmo boot do live**, e por isso só podiam falar do `ocs-sr`. O ciclo inteiro faz outra coisa: a ordem permanente volta ao que está dentro da imagem. Procurar o original não acharia este defeito, porque o original estava lá | Perguntar **entre que dois instantes** a evidência foi tirada, e se a pergunta cabe dentro deles. O par da E9 é do lado Windows e atravessa o reinício ([ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)) |
| **A operação apagando o registro de que foi armada** | O `%LOCALAPPDATA%\ARCA\arca.log` mora no `C:`. Numa restauração, a linha do armar é escrita minutos antes do reinício e **substituída pela imagem** junto com o resto do disco: o log que sobra salta do último comando de antes da imagem direto para a colheita. E a tela que a imprimiu morreu no reinício que ela disparou | §4.1 — o `estado.json` mora no `ARCABOOT` e sobrevive. É o único lugar que liga o selo do desfecho a um job, e a colheita de uma restauração se vira só com ele (§6.3) |
| **Um redirecionamento que o bash honra e o Clonezilla não** | A receita da verificação usa `>>` para **acrescentar** ao `arca-check.log`, e o ensaio em bash prova que `>>` acrescenta. Em hardware o arquivo saiu com **uma** execução do `ocs-chkimg`, e o log do backup que a imagem carregava sumiu. O `--dry-run` tinha impresso `>>` minutos antes; o ensaio tinha passado; a suíte estava verde. **Nada disso fala sobre o que o `ocs-chkimg` faz com o descritor que recebe** | Contar execuções no arquivo, e não confiar no redirecionamento: toda execução do `ocs-chkimg` abre com a mesma sequência de escapes de terminal, e duas execuções dariam duas. É P-25, e o `>>` fica pela razão que sobreviveu — ele não abre a janela de zero byte que o `>` abre |
| **A tela não dizer o que vai aparecer do outro lado do reinício** | O `grub.cfg` tem `set timeout="30"`, e o `set default` escolhe **qual** entrada boota sozinha **sem tirar a espera**. Todo boot armado mostra o menu do Clonezilla parado por meio minuto, depois carrega o live system para a RAM (`toram`), e só então a receita roda. Em 23/08/2026 uma verificação armada foi disparada, o menu apareceu, quem estava na frente viu que não era o Windows e **desligou a máquina** — não havia defeito nenhum, e a tela do ARCA dizia só *"vai reiniciar e desligar sozinha ao terminar"*. **E o rastro de "desliguei durante o menu" é idêntico ao de "o Clonezilla recusou a receita"**: nos dois casos não há `arca-fim.txt`, e C-12 reporta as duas causas porque não há como separá-las de fora | `armar::montar_o_que_vem_pela_frente`, nos **três** comandos que armam: nomeia o menu, os trinta segundos e o que desligar ali custa. O número sai do `set timeout` do `grub.cfg` capturado, e há teste que falha se os dois divergirem. O que separou as duas causas naquele dia foi ir ao dispositivo procurar a pasta do log: o primeiro passo de toda receita é um `mkdir -p`, e **ela não estava lá** |
| **Ler as pontas de uma lista e concluir o que há no meio** | O `MD5SUMS` de uma imagem tem 39 linhas, e a ordem **não é alfabética pura**: os catorze `nvme0n1p*` — os 39,7 GB — ficam no meio, entre o `nvme0n1-mbr` e o `nvme0n1-pt.parted`. Olhando as oito primeiras e as três últimas, a conclusão é que o `MD5SUMS` cobre só os metadados — e V-1 inteiro nasceria sobre isso, aprovando uma imagem tendo lido 2 KB de 39,7 GB | Contar. `tests/e11_verificar_a_imagem.rs` cobra que **toda** imagem do dispositivo liste arquivos de partição, e `src/md5sums.rs` fixa os catorze da captura. A pergunta que separa os dois casos é a mesma de sempre: *a evidência que olhei fala sobre a pergunta inteira?* |
| **Uma coluna que cabe no caso comum e morre no caso real** | As linhas do §5.2 têm coluna fixa em 33, e `formato::linha` deixa o rótulo **estourar** quando não cabe — o que está certo para um rótulo excepcional. No andamento de V-1 o rótulo é um nome do Clonezilla, e `nvme0n1p3.ntfs-ptcl-img.zst.aa` tem trinta caracteres: **catorze das trinta e nove linhas** saíam com um ponto só, e a coluna deixava de existir justamente na parte que demora três minutos | A coluna do andamento sai do **maior nome da lista**, e não de uma constante. Achado **rodando o comando de verdade**, com a suíte verde — como na E6, na E7, na E9 e na E10 |
| **O log do Clonezilla não é o log inteiro** | O `arca-restore.log` do marco tem o fim da operação e não tem o começo: uma passagem só do Partclone — a da última partição, 1,1 GB —, nenhuma das outras três, e um `Ending /usr/sbin/ocs-sr` sem o `Starting` correspondente. Causa não determinada. O §6.3 aponta esse arquivo a quem quer saber o que aconteceu, e o que está lá **pode não cobrir a parte que falhou** | Saber disso antes de concluir qualquer coisa por ausência. Medir de novo na próxima restauração, e perguntar se o corte cai sempre no mesmo lugar |
| **Duas ferramentas com o mesmo nome, e a errada responde primeiro** | `tar` no `PATH` **não é** o `bsdtar` numa máquina com Git para Windows: é o **GNU tar 1.35** do `/usr/bin`, que não abre zip — responde *"This does not look like a tar archive"* e sai com erro. O `curl` tem o mesmo problema, por um em `/mingw64/bin`. **O modo de falha é caro**: o `arca prepare` extrai o pacote *depois* de ter apagado o disco, e falharia com o dispositivo destruído e nada instalado. É a **segunda** vez que esta mesma ferramenta engana por homonímia — a primeira foi a versão medida no `ProductVersion` do Windows em vez do `FileVersion` do bsdtar | Chamar por **caminho absoluto** no `System32`, e nunca pelo nome. O campo que separa os dois sem ambiguidade é o `OriginalFilename` do executável: `bsdtar` num, `tar` no outro. Há teste rodando `--version` contra o binário de verdade ([ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md)) |
| **O índice do disco não é identidade** | Em 23/08/2026 o dispositivo desta mesa era o **disco 1**; horas depois, com um segundo SSD conectado, virou o **disco 2** — e o `ARCAVAULT` dele, que sempre aparecera em `E:`, veio em `D:`. Entre o `arca prepare` imprimir o plano e apagar a tabela há uma pessoa lendo e digitando, e nesse intervalo cabe trocar um cabo: o `sim` dado sobre um disco seria executado sobre outro | O terceiro tempo de PR-4 — o ARCA **relê o disco** e compara modelo e tamanho com o que imprimiu, antes da primeira escrita. E a confirmação de S-2 pede o **modelo**, e não o índice: o número é justamente o que muda |
| **Uma ferramenta que põe na ordem de boot sem ninguém pedir** | `bcdedit /copy {bootmgr}` acrescenta a entrada nova ao `displayorder` **sozinho** — medido duas vezes em 23/08/2026. É exatamente o que C-5 nomeia como perigo: um caminho permanente a mais para bootar no dispositivo, que ninguém pediu e que só se descobre num religar qualquer | Tirar da ordem logo depois de criar, com `/remove` e releitura de C-3, e conferir que nenhuma outra entrada saiu junto. Tirar não custa nada: o `bootsequence` funciona sobre entrada fora da ordem, medido na E7 (ADR-0007, [ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md)) |
| **Um artefato que se parece com o que se tem, e veio de outro lugar** | O `grub.cfg` do **zip** do Clonezilla e o do dispositivo desta mesa diferem em duas coisas: o `noeject` em treze `menuentry`, e **seis segundos** no carimbo do rodapé. Seis segundos é o `ocs-live-dev` gerando o ISO e o zip na mesma execução — o dispositivo veio do **ISO**. Sem essa comparação, um `arca prepare` instalaria um `grub.cfg` diferente do que serve de oráculo para os testes da E4, e ninguém saberia por quê | Comparar o artefato que se vai instalar com o que já roda, **linha a linha**, antes de instalar. E o que absorve a diferença já estava decidido: o estado inerte se reconstrói do arquivo corrente (ADR-0005) e o bloco do ARCA deriva dele (ADR-0007) |
| **Um pacote que extrai sem erro e não boota** | Um zip sem o `bootx64.efi` sai do `bsdtar` com código zero e produz um dispositivo que não boota — e isso só se descobre **depois** de o Windows ter sido apagado, porque é aí que alguém precisa dele. O código de saída da ferramenta de extração não fala sobre o conteúdo | Listar o pacote (`bsdtar -t`) e conferir os quatro caminhos que fazem um dispositivo bootar **antes** de escrever, normalizando `\` para `/` e ignorando a caixa — quem lista é o `bsdtar` e quem confere é o Windows |
| **Testes de hardware que descreviam um dispositivo, e não o conceito** | Cinco testes das etapas E1, E4, E7 e E11 rodavam contra o dispositivo da mesa e afirmavam coisas que eram verdade **daquele**: que o `ARCAVAULT` tem imagens, que ao lado do `grub.cfg` há três cópias armadas de agosto, e que o `grub.cfg` é byte a byte a captura do repositório. Nenhuma delas é verdade num dispositivo que o `arca prepare` acabou de fazer — ele nasce vazio, sem cópias, e com o `grub.cfg` do **zip**. Os cinco ficaram vermelhos assim que o dispositivo novo ficou sozinho na mesa | Distinguir **"não há o que conferir"** de **"conferi e reprovou"**, e dizer qual dos dois — um teste de hardware que sai calado é indistinguível de um que passou. Os que dependem de imagem saem cedo explicando; os do `grub.cfg` passaram a aceitar **os dois inertes conhecidos**, com o teste da E10 provando que são equivalentes. O que nenhum deles faz é afrouxar: o que provavam continua provado assim que houver o que provar |
| **Uma recusa rara que deixou de ser rara** | C-10 recusa dois `ARCAVAULT`, e a mensagem dizia *"Desconecte os demais"* — o que bastava enquanto ter dois dispositivos exigia **comprar** dois. Desde a E10 o ARCA **faz** o segundo: um `arca prepare` bem-sucedido deixa dois conectados por definição, e a partir dali todo comando cai na recusa — **inclusive o `arca status`**, que é o que alguém rodaria para entender o que está acontecendo | A mensagem nomeia **as letras** e a causa provável: `ha 2 volumes com o rotulo ARCAVAULT conectados (D:, E:) … Se voce acabou de preparar um dispositivo, sao os dois`. Achado **rodando o comando de verdade** depois do marco, e a lição é a de sempre: uma peça nova encaixada numa peça antiga que ninguém releu ao encaixar |

| **Uma pergunta plausível no lugar da certa** | `julgar_o_conjunto` só julga a operação pelo veredito da pasta quando aquele veredito **fala daquela operação** — a E9 achou isso na restauração, onde a pasta é a imagem de origem. Ao acrescentar a sondagem, a formulação que vem à cabeça é *"as que **produzem** imagem"* — e ela deixaria a **verificação** de fora, que é justamente uma das que devem entrar: o `arca-check.log` que ela lê é o `ocs-chkimg` daquela execução. A formulação errada passava pela suíte inteira, e o estrago era **toda sondagem bem-sucedida sair com código de erro** | Nomear a pergunta em vez do critério: `o_veredito_fala_desta_operacao`, com a tabela das quatro no doc e teste sobre as quatro. Achado **falsificando** — mutando o código de produção e vendo a suíte calar |
| **Um conselho dito duas vezes na mesma tela** | O pré-voo imprime a recusa de `SemNome` e, logo abaixo, um aviso fixo que explica por que o nome do disco importa. A E12 acrescentou `Para produzi-lo: arca sondar` **nos dois**, e a tela passou a repetir a mesma frase em quatro linhas — o começo do ruído que treina quem lê a pular o parágrafo | Cada recusa diz a saída **dela**, e o aviso fixo não repete. E a saída não é a mesma para todas: `arca sondar` resolve `SemOraculo`, `ModeloNaoCasa` e `NomeInvalido`, e **não** resolve `ModeloAmbiguo` — sondar de novo veria os dois discos outra vez. Achado **rodando o comando de verdade**, com a suíte verde |
| **Duas linhas da mesma tela afirmando fontes diferentes** | O `arca backup --dry-run` do marco da E12 imprimiu, no pré-voo, `Disco de origem ..... lido da sondagem de 24/08 11:58` — e **quatro linhas abaixo**, no ensaio, `Disco de origem: nvme0n1 · lido do blkdev.list de uma imagem`. A segunda era uma **frase fixa**, de antes de a sondagem existir: o `Ensaio` carregava um `de_exemplo: bool`, que sabia dizer *se* o nome fora determinado e nunca *por quem*. É o padrão de sempre, e a peça antiga aqui é um `bool` de duas etapas atrás | Não ter mais frase fixa: o campo virou `origem: Option<&NomeDoDisco>`, e a linha do ensaio é **literalmente** a que o pré-voo imprime — o `Display` de `NomeDoDisco`, que é quem sabe de onde o nome veio. Achado **rodando o comando de verdade**, com a suíte verde |
| **O `mtime` de um arquivo que o Linux escreveu, com o dono do relógio trocado** | O campo `quando` da sondagem sai do `mtime` do `blkdev.list`, e a doc dele dizia que vinha do relógio **do Windows, e não do live**. É o contrário: quem escreve o arquivo é o `lsblk`, do outro lado do reinício, e o valor sai **três horas atrás** do relógio daqui. A tela imprimiu `lido da sondagem de 24/08 11:58` para uma sondagem armada às **14:56:55**. Quem comparasse com o `armado_em` do `estado.json` concluiria que a sondagem é mais velha do que é — que é exatamente a conta que S-6 existe para ninguém fazer | Dizer **de quem é o carimbo** na própria linha: `(carimbo do Clonezilla, P-7)`. E **não corrigir o valor**: somar três horas fabricaria um instante que ninguém mediu. Para o que o campo existe — separar uma sondagem da anterior — o deslocamento não atrapalha, porque as duas vêm do mesmo relógio |
| **Uma ponte USB dando outro nome ao disco** | O `lsblk` chama o dispositivo desta mesa de `Maxtor Z1 SSD 480GB`; o WMI o chama de `JMicron Generic SCSI Disk Device`. A ponte USB responde ao Windows com o nome **dela**, e o Linux lê o disco atrás dela. O disco de origem casa nas duas fontes — é o que o backup precisa —, e o que **não** casa é o dispositivo: com isso a **segunda** barreira de R-8 (resolver o nome Linux do dispositivo pelo mesmo oráculo e comparar com o do destino) fica **inerte**. No dispositivo antigo os dois lados casavam (`KGSSE100 256 SCSI Disk Device` ↔ `KGSSE100256`), e ninguém tinha razão para suspeitar | Saber que ela pode ficar inerte, e por quê. Ela não falha errado — só não dispara —, e a primeira barreira, por **letra do Windows**, continua valendo; o ADR-0015 já previa que a segunda viraria redundante. O que o marco acrescentou foi a **causa**: um canal de identidade que passa por uma ponte não fala do disco |
| **Um teste que aceita mais do que devia** | O teste que guarda a reconstrução das colunas do `lsblk` procurava `-o <colunas>` como **substring**, e `-o A,B,C,D` contém `-o A,B,C`: uma coluna **a mais** passava por ele. Não é hipotético — a falha forçada de 24/08/2026 acrescentou `FLAGQUENAOEXISTE` ao fim da lista, e a asserção **passou**. O único teste que pegou a mutação foi o do ensaio em bash, e por acaso | Comparar por **igualdade** sobre a lista extraída da receita, e ter um segundo teste que exercita a extração contra uma receita adulterada — o guarda do guarda. A lição é a de sempre com o sujeito trocado: um teste que aceita mais do que devia é um teste que ninguém sabe se funciona, e a única forma de descobrir é **mutar o código de produção** |
| **Um conselho que serve a três operações e serve mal à quarta** | A colheita de um desfecho `FALHOU` dizia *"o log da operação está em `ARCA-LOGS\<pasta>\`"*. Nas três operações que gravam, o log tem centenas de linhas de progresso e "olhe a pasta" é o melhor que se pode dizer. Na sondagem há **um** arquivo com **uma** linha — `lsblk: unknown column: FLAGQUENAOEXISTE` —, e ela é a resposta: mandar procurar na pasta esconde a resposta a um `cd` de distância | Ramo próprio, apontando o `blkdev.list` pelo nome e dizendo por que a mensagem sobreviveu (o `2>&1`). Achado **rodando a falha de verdade** — a frase genérica estava certa e era inútil, que é um defeito que nenhum teste de string pega |
| **Um teste de sincronia que a forma errada satisfaz** | `recursos/ensaio-da-receita.sh` guarda a forma **proposta e não escrita** da receita de sondagem — a com `;` — para mostrar, num bash de verdade, o que ela escreveria. O teste `o_ensaio_em_bash_ensaia_a_receita_de_hoje` confere que a receita do código aparece no script; com a forma errada lá dentro, **trocar o `if` pelo `;` no código passa por ele** | Saber disso antes de confiar naquele teste, e é o que o comentário da string diz. A mutação é pega por outros três, que olham a receita e não o script. **É o custo de guardar contraexemplo junto do exemplo**, e ele vale a pena: o contraexemplo é a única coisa neste repositório que mostra o `;` mentindo |

## 12. Decisões e pendências

### Decisões fechadas

| Decisão | Motivo |
|---|---|
| Só imagens completas, nunca incrementais | Independência: corrupção não se propaga |
| Cada dispositivo é autocontido | Nada externo é necessário para restaurar |
| O ARCA e o estado moram no dispositivo | O dispositivo não entra na imagem |
| Labels fixos `ARCABOOT` / `ARCAVAULT` | Receita reprodutível, dispositivos intercambiáveis |
| Um dispositivo ARCA por vez | Elimina ambiguidade de label |
| Nome livre, nada é sobrescrito | Sem marcos, sem catálogo, sem retenção |
| Escolha da imagem no Windows, execução sem telas | A lista à vista, antes do ponto sem volta |
| `-iefi` e `-k0` sempre na restauração | Validados por medição |
| Verificação sempre, veredito em arquivo | Imagem não verificada é suposição |
| **Nunca `-scs`** — a conferência nativa do Clonezilla fica ligada, ao lado do `ocs-chkimg` explícito | `-scs` é `--skip-check-restorable`. B-8 o pedia sempre; o hardware rodou sem ele. Dois sinais independentes valem mais do que um ([ADR-0004](../docs/adr/0004-a-receita-transcreve-o-que-rodou.md)) |
| `-e1 auto -e2` sempre na restauração | Estavam na única restauração que deu certo. Inócuos no mesmo disco, e é o que faz a partição de boot bater com a geometria de outro (R-7) |
| A receita escreve `ARCA_VEREDITO=` no `arca-check.log` | Tira o veredito da dependência de interpretar frases em inglês do `ocs-chkimg` ([ADR-0003](../docs/adr/0003-veredito-lido-do-arca-check-log.md)) |
| A receita transcreve o que rodou, e o que não tem original é marcado como código novo | Duas vezes já se documentou como fundação validada o que veio do trabalho de validação em volta dela (ADR-0004) |
| ~~O ARCA não cria partições~~ **O ARCA particiona o dispositivo, e nunca escolhe o disco** | P1, revisado em 23/08/2026 ([ADR-0014](../docs/adr/0014-o-arca-particiona-o-dispositivo.md)) |
| `toram` mantido | Evita acoplar o live system ao dispositivo que ele remonta |
| Job ligado ao desfecho por selo, nunca por data | Não há relógio comum entre Windows e Clonezilla (P-7) |
| A receita continua sendo string no `grub.cfg`, não arquivo | É o mecanismo medido em hardware. Trocá-lo por um `custom-ocs` em arquivo exigiria remedir, e `toram` pode desmontar o medium |
| ~~Restaurar em disco diferente é permitido, recusado só se menor~~ **Só o disco de origem é destino válido** | Quem troca o disco reinstala o Windows, e o caso de uso não existe. R-7 troca de função: de `≥` (cabe?) para `=` (é ele mesmo?) — mais duro sem custar código ([ADR-0015](../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md), pago no código na E10) |
| **A entrada de firmware nasce de `bcdedit /copy {bootmgr}`, e sai da ordem permanente** | A entrada `ARCA` desta máquina é uma cópia do `{bootmgr}` com três campos trocados — o original estava nela mesma. E o `/copy` a põe no `displayorder` sozinho, que é o perigo de C-5: tirar é uma escrita de alvo fixo, sobre a entrada que o próprio comando acabou de criar ([ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md)) |
| **O `arca prepare` instala o zip, e desarma o `grub.cfg` que ele entrega** | O zip vem com `set default="0"`, que é "um estado que parece inerte" (ADR-0005). Desarmar o do zip produz exatamente o `grub.cfg` inerte deste dispositivo, a menos do `noeject` e de seis segundos de carimbo — a mesma build, artefatos diferentes ([ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md)) |
| **O `curl` e o `bsdtar` são chamados por caminho absoluto** | `tar` no `PATH` é o GNU tar numa máquina com Git instalado, e ele não abre zip. O modo de falha é falhar na extração **depois** de o disco ter sido apagado (ADR-0018) |
| A recusa de R-7 fica, e a razão é **onde** ela acontece — não "senão corrompe" | O Clonezilla já confere e desiste por padrão. A recusa dele custa um reinício de uma operação destrutiva; a do ARCA custa zero ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)) |
| O tamanho do destino sai do `MSFT_Disk`, e a comparação sai em setores | É a única régua que casa com a GPT de dentro da imagem. O `Win32_DiskDrive` é o CHS legado truncado, e faria um disco não caber em si mesmo (ADR-0010) |
| O próprio dispositivo ARCA nunca é destino, e nenhuma confirmação libera | Apagaria o Clonezilla que está executando a receita e a imagem que ela lê (R-8) |
| Imagem **reprovada** continua sendo oferecida para restaurar, com aviso | L-2 fala de resíduo, e não de veredito. Com o disco de origem morto, uma imagem reprovada pode ser tudo que restou — recusá-la seria o ARCA decidir por quem está na frente da tela |
| Clonezilla com versão fixada e checksum embutido | Checksum baixado do mesmo servidor que o arquivo não verifica nada |
| O binário roda de onde estiver; só o estado é obrigado a morar no `ARCABOOT` | O que a restauração não pode devolver é o julgamento, não o executável |
| O estado inerte se **reconstrói** do `grub.cfg` corrente, e não vem de cópia embutida nem guardada no dispositivo | Idempotência de graça, e funciona num dispositivo que o ARCA nunca viu. Embutir prenderia o ARCA a uma versão do Clonezilla e descartaria a configuração de hardware daquele dispositivo a cada desarmar ([ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)) |
| O `set default` volta sempre para `live-default`, nunca para `0` | `"0"` aponta por posição, e a posição muda quando o bloco do ARCA entra (§4.4) |
| O `menuentry` do ARCA **deriva** do `live-toram` do próprio dispositivo, e não é transcrito de nenhuma cópia | Mesma razão do estado inerte: o `grub.cfg` carrega a configuração daquele hardware. Medido: a `teste-02` é o `live-toram` com as cinco substituições de §10.2.1, e a `teste-03` — a única que provavelmente rodou desatendida — perdeu nove parâmetros, inclusive o de NVMe ([ADR-0007](../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md)) |
| Sem nome de disco determinado, `arca backup` **recusa**; não pergunta nem deriva | Um nome do Linux digitado do lado Windows não tem contra o que ser conferido, e a receita que o nomeia é destrutiva na E9 (§4.5) |
| Colher **marca** o `estado.json` como colhido, e nunca o apaga | O arquivo é o único registro que liga um selo a um nome. Marcar fecha o par que a E5 deixou aberto sem reabrir B-10 ([ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md)) |
| Armar não cria entrada de firmware; migra a que existe, ou recusa | Criar uma do zero é código sem original. O lugar disso é o `arca prepare` (C-4) |
| **A verificação armada é uma terceira `Operacao`**, e não um backup sem `savedisk` | A pasta do log vem do nome da operação, e toda receita trunca o próprio `arca-fim.txt` com um `>`. Dividir a pasta faria a verificação apagar o desfecho de um backup não colhido — o defeito que a revisão da E3 pegou entre backup e restauração, cometido pela terceira vez ([ADR-0016](../docs/adr/0016-a-verificacao-armada-e-a-terceira-operacao.md)) |
| **A verificação acrescenta ao `arca-check.log`; o backup o cria** | Lá a imagem acabou de nascer; aqui o log é o veredito do backup que a criou, e um `>` o destruiria — inclusive por truncar ao abrir, deixando uma imagem boa sem veredito. Com `>>`, a ordem "toda forma de reprovar antes de toda forma de aprovar" do ADR-0003 passa a valer entre **duas verificações**, que é o caso que ele previu (ADR-0016) |
| **V-1 não grava veredito na listagem** | A coluna do `arca list` é o parecer do `ocs-chkimg` — outra pergunta. Escrever ali uma reprovação de MD5 faria a listagem afirmar que ele reprovou, e ele nem rodou. A tela de V-1 diz isso quando reprova (ADR-0016) |

### Pendências

> **O que cada uma custa, e quais duas separam este app de estar fechado**, está
> em [o-que-falta-para-fechar.md](o-que-falta-para-fechar.md) — escrito depois da
> E10, quando as doze etapas do plano acabaram e a pergunta *"então está
> pronto?"* passou a merecer uma resposta que não dependa de quem responde.

| # | Questão |
|---|---|
| P-6 | O `ocs-sr` devolve código ≠ 0 quando falha? Fecha com falha forçada em VM |
| P-7 | O deslocamento de 3 h é permanente. Existe para a próxima pessoa que for comparar datas |
| ~~P-18~~ | ~~O boot único pode nunca ter sido disparado por boot único.~~ **Fechada em 22/08/2026, etapa E8.** O `efibootmgr` do live registrou `BootCurrent: 0001` com `BootOrder: 0000,0001`: a máquina bootou por uma entrada que não era a primeira da ordem, e só o `bootsequence` explica. Ver §3.1, §3.5 e [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md) |
| P-19 | **Em que condição o firmware cria uma `UEFI OS` no lugar da entrada?** — ver §3.5. Aberta na E8, estreitada na E9, e com o candidato **refutado** em 24/08/2026: não é o `bootsequence`. A leitura de graça dentro de cada imagem foi de fato quem respondeu — e respondeu contra a hipótese. Não fecha por reinício ([ADR-0023](../docs/adr/0023-o-bootsequence-nao-e-o-gatilho-da-reescrita.md)) |
| P-14 | `arca resultado` deve rodar sozinho no logon? Começar sem, decidir com uso |
| ~~P-15~~ | ~~A receita de backup publicada em §10.1 divergia da fundação §3.2 quanto ao `-batch`.~~ **Fechada em 22/08/2026, etapa E3.** `-batch` rodou, nas três receitas preservadas em `recursos/capturas/`. O help do `ocs-sr` diz por que é `-batch` e não `-b`: em parâmetro de boot, o `init` do sistema também honraria `-b` |
| ~~P-16~~ | ~~O mecanismo de desfecho nunca rodou.~~ **Fechada em 22/08/2026, etapa E8.** `arca-fim.txt`, selo na receita, `ARCA_FIM` e `if/then/else` estrearam todos de uma vez, e o selo do desfecho bate com o do `estado.json`. Ver §3.5 |
| ~~P-17~~ | ~~`-icds` contradiz R-7.~~ **Fechada em 23/08/2026, etapa E9.** O help está certo e a premissa de R-7 estava errada: o Clonezilla confere e **desiste**, não corrompe. R-7 foi reescrito — a recusa fica, e a razão passa a ser que a do Clonezilla acontece do outro lado do reinício. E resolver isso obrigou a descobrir a armadilha da régua: o `Win32_DiskDrive` e o `MSFT_Disk` dão dois tamanhos para o mesmo disco. Ver [ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md) |
| ~~P-21~~ | ~~O `ocs-sr` sai com código ≠ 0 quando desiste por destino menor?~~ **Fechada por escopo em 23/08/2026**: só o disco de origem é destino válido, e o caso não é alcançável ([ADR-0015](../docs/adr/0015-a-restauracao-so-restaura-no-disco-de-origem.md)) |
| ~~P-22~~ | **Fechada em 24/08/2026: é a NVRAM** — ver §3.5. ~~O `bcdedit /enum firmware` mostra a NVRAM, ou o BCD do disco?~~ Aberta no marco da E9. Um religar limpo acrescentou ao `displayorder` três entradas que só o firmware escreve — `UEFI:CD/DVD Drive`, `UEFI:Removable Device`, `UEFI:Network Device` —, e nada no BCD as originaria. A linha `Ordem de boot` do `arca status` lê a fonte certa, e **C-13 conserta o firmware e não um espelho dele**. Abriu P-28. Ver [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md) |
| ~~P-28~~ | **Fechada em 24/08/2026, no mesmo dia em que nasceu: ela não desvia o boot** — ver §3.5. ~~`UEFI:Removable Device` alcança o `ARCABOOT`?~~ As entradas que o firmware acrescenta não declaram alvo, e o ARCA lia a ausência de alvo como *não leva ao dispositivo*: com uma delas em primeiro, o `arca status` **engolia o aviso**; com o dispositivo fora da ordem, ele **afirmava** que só o boot único leva a ele. Consertado antes da medição (C-14, [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md)) e medido às 18:47: posta em primeiro, ela não levou ao dispositivo — e o firmware apagou as três no POST, devolvendo a ordem byte a byte ao que era |
| ~~P-20~~ | ~~O `arca resultado` deve devolver o `{bootmgr}` à frente do `displayorder`.~~ **Fechada em 23/08/2026, etapa E10.** Virou C-13, com os quatro comandos medidos à mão antes de virar código. Ver [ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md) |
| ~~P-23~~ | ~~Por que o `arca-restore.log` começa no meio?~~ **Fechada em 24/08/2026.** Ele não começa no meio: o Clonezilla reabre o próprio log com truncamento na última passagem, e o descritor da receita retoma de um offset alto — o vão vira NULs. O corte **não** cai sempre no mesmo lugar: cai onde o `ocs-sr` chegou ([ADR-0022](../docs/adr/0022-o-arca-restore-log-e-truncado-por-baixo.md)) |
| ~~P-24~~ | ~~A verificação armada (V-2) nunca rodou.~~ **Fechada em 23/08/2026, etapa E11** — armada às 16:53:30, colhida `concluida` com veredito `APROVADA`. Ver §3.5 |
| P-25 | **Por que o `arca-check.log` foi substituído, se a receita usa `>>`?** — ver §3.5. Aberta no marco da E11, e é uma previsão deste documento que a execução real desmentiu |
| ~~P-26~~ | **Fechada em 24/08/2026, no marco da E12** — ver §3.5. ~~Um dispositivo preparado pelo `arca prepare` boota?~~ Aberta no marco da E10, 23/08/2026. O comando produziu um dispositivo com o Clonezilla instalado, o `grub.cfg` inerte e a entrada de firmware apontando para ele — e **nada disso foi bootado**. O que se conferiu foi tudo o que se pode conferir sem reiniciar: a estrutura de partições relida do disco, os quatro caminhos obrigatórios dentro do pacote, o `set default` de volta em `live-default`, e a entrada de boot relida do `bcdedit`. **O que falta é o firmware honrar aquela entrada**, e isso custa um reinício — o único da E10, e ela é a primeira etapa deste projeto cujo marco não precisou de nenhum. ~~Fecha com um `arca backup` no dispositivo novo~~ — **ele recusa**, e a razão é o §4.5: não há imagem de onde ler o nome do disco. **Fecha com o marco da E12**, `arca sondar`, que responde as **duas** metades de uma vez: (a) o dispositivo boota, e (b) a entrada que o ARCA criou leva a ele — porque o boot é o **único**, disparado pelo `bootsequence`, sobre uma entrada que está fora da ordem permanente. Um F12 responderia só (a). O risco é o menor de todos os marcos deste projeto: a receita da sondagem não tem `ocs-sr`, e nada é escrito fora do `ARCAVAULT` |
| ~~P-27~~ | **Fechada em 24/08/2026, no mesmo marco** — `ARCA_PROBE=OK`, e a arvore saiu em ASCII, o que diz que o `-i` foi aceito. ~~As flags do `lsblk` da sondagem são reconstrução, e o util-linux daquele live pode recusar alguma.~~ Aberta na E12, 23/08/2026. As sete colunas saem do cabeçalho do `blkdev.list` capturado, e o `-i` sai do fato de a árvore vir em ASCII; a **linha de comando** que o Clonezilla usa mora nos scripts dentro do `filesystem.squashfs`, que este repositório nunca abriu. Fecha no primeiro `arca sondar` que rodar: `ARCA_PROBE=OK` diz que as flags passaram. Custa o mesmo reinício que já se ia gastar, e o modo de falha é barato e visível — `ARCA_PROBE=FALHOU`, com a mensagem do `lsblk` dentro do próprio `blkdev.list` (SD-2, SD-3) |

---

*Documento vivo. Atualizar após cada medição em hardware.*
