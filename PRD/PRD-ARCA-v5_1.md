# PRD — ARCA v5.1

**Automatizador de Clonezilla para backup e restauração de imagem de disco.**

Versão 5.1 · 22/08/2026 · Substitui a v4
Última revisão: 23/08/2026, **etapa E9, escrita** — R-7 reescrito contra o help do `ocs-sr` e contra a medição das duas réguas do mesmo disco ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)), **P-17 fecha**; §6.1 ganha a tela real e perde o `498,7 GB` — a **sexta** vez do mesmo número medido na coisa errada; §6.2 ganha o que a imagem de fato carrega; §3.1 corrigido — as duas leituras de NVRAM de 21/08 são de **dois boots diferentes**, e a que o documento usava é da restauração ([ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md)); P-19 **estreita**: a primeira metade está descartada por medição; §8 ganha `--destino`; §11 ganha a armadilha de datar a captura e não saber de que operação ela é
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
- ❌ Criador de partições (ver [P1](#71--o-arca-não-cria-partições))
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
> [ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md),
> e P-22 para o que isso deixa em aberto.

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
> pergunta. Ver o ADR-0012, e P-22 em §3.5.

> **As três restaurações à mão continuam sendo as três.** R1 e R2 em 20/08, e a
> de 21/08, feitas pelo menu do Clonezilla. O que a E9 acrescentou é a quarta,
> e a diferença dela não é o `ocs-sr` — é o envoltório que diz se deu certo
> (§10.2.2), e ele estreou nesta operação do lado da restauração.

### 3.5 — Ainda não medido

| # | Pendência |
|---|---|
| P-6 | **O `ocs-sr` devolve código diferente de zero quando falha?** O ramo de sucesso foi medido; o de falha não. Uma restauração bem-sucedida não fecha isso, por definição. Fecha com falha forçada, provavelmente em VM |
| P-19 | **Só quando ela foi consumida por `bootsequence`?** — a metade que sobrou. **A primeira metade fechou na E9, pela negativa: o firmware NÃO reescreve a entrada em todo boot pelo dispositivo.** Em 20/08 houve pelo menos três boots pelo dispositivo, e em todas as capturas a entrada continua na forma que o `bcdedit` escreve — `Clonezilla`, caminho em minúsculas, `BCDOBJECT` presente —, inclusive em dois deles com a entrada fora da frente da ordem. O que **não** fecha é datar a reescrita: uma captura feita durante o boot N mostra a NVRAM como ela está, e não qual boot a deixou assim. O experimento que fecha é **um backup disparado por F12**, com o `bcdedit` lido imediatamente antes. Ver [ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md) |
| P-21 | **O `ocs-sr` sai com código diferente de zero quando desiste por destino menor?** Aberta na E9, e é P-6 com outra roupa: o help diz que ele *"quit"*, e se esse `quit` sair com zero o `if/then/else` de R-5 escreve `ARCA_RESTORE=OK` sobre uma restauração que não aconteceu. **Não é urgente**, e a razão é o desenho: R-7 recusa antes, do lado Windows, e essa pergunta só chega a importar se a recusa do ARCA tiver um furo ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)) |
| P-20 | **O `arca resultado` deve devolver o `{bootmgr}` à frente do `displayorder`, esteja o dispositivo conectado ou não.** Pedido em 22/08/2026, pela fricção que o ADR-0009 mediu: com o dispositivo em primeiro, ligar a máquina com o SSD conectado boota nele. **Exige revisar C-5 e superseder o ADR-0009**, que decidiu avisar em vez de consertar. O argumento a favor é que C-5 foi escrito contra **acrescentar** um caminho para o dispositivo, e isto **remove** um — assimetria que o requisito não distingue e que nunca foi discutida. Decidir e medir na E10. **O alcance estreitou no marco da E9**: a restauração já devolve a ordem sozinha, porque ela está dentro da imagem ([ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)). O pedido é sobre o **backup**, que suja a ordem e não a limpa |
| P-23 | **Por que o `arca-restore.log` começa no meio?** Aberta no marco da E9, medindo o primeiro original que ele teve. Ele traz uma passagem só do Partclone — a da última das quatro partições — e um `Ending /usr/sbin/ocs-sr` sem o `Starting`. A receita redireciona com `> … 2>&1`, e o `arca-check.log` do backup não tem esse corte. **Importa porque o §6.3 aponta esse arquivo a quem quer saber o que aconteceu.** Fecha na próxima restauração, e a pergunta é se o corte cai sempre no mesmo lugar |
| P-22 | **O `bcdedit /enum firmware` mostra a NVRAM do firmware, ou o BCD do disco?** Aberta no marco da E9. Nunca precisou de resposta até a restauração devolver a ordem permanente de dentro da imagem: **se é o BCD, a NVRAM pode continuar com o dispositivo à frente e a máquina continuaria bootando nele — enquanto a linha `Ordem de boot` do `arca status` diria que está tudo bem.** Seria uma afirmação de segurança feita sobre uma leitura que não fala da pergunta, que é o defeito que a revisão do marco da E8 já pegou naquela mesma linha. **O experimento custa um reinício e nenhum risco**: religar com o SSD conectado, sem job armado e com o `grub.cfg` inerte. Parando no Windows, a NVRAM acompanhou; parando no menu do Clonezilla, não acompanhou. Ver [ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md) |

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
nome inteiro da imagem para ouvir um não depois. E o custo é conhecido e
limitado: num dispositivo sem imagem alguma, o primeiro backup precisa ser
feito uma vez pelo menu do Clonezilla (§6.4). Dali em diante o `blkdev.list`
dele responde para sempre.

**A saída sempre diz de onde o nome veio** — `nvme0n1 · lido de
2026-08-21_WindowsCompleto/blkdev.list, casando o modelo …`. Uma receita
destrutiva que nomeie um disco sem dizer a origem do nome é pior do que não
imprimir nada.

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
> mexer na tabela é o resultado que se queria, e vale dizer que foi conferido:
> as outras seis linhas continuam sem original, e a que mais importa fechar é a
> do desfecho `FALHOU`, que depende de P-6.

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

### 7.1 — O ARCA não cria partições

> **Princípio P1.** O ARCA não executa a operação mais destrutiva do fluxo. O que se faz uma vez por dispositivo, e destrói tudo quando sai errado, fica com o usuário e com a ferramenta do sistema.

Particionar um disco é exatamente isso: a operação mais destrutiva do fluxo, feita **uma vez por dispositivo**.

`arca prepare` **exige** uma partição FAT32 vazia de ≥ 1 GB. Não havendo, imprime as instruções para criá-la no Gerenciamento de Disco e para.

```
> arca prepare

Dispositivo: KGSSE100 256GB
  sda1  NTFS   236,9 GB  ARCAVAULT   ok
  sda2  FAT32    1,6 GB  ARCABOOT    ok

  Baixando Clonezilla ............. ok  (checksum conferido)
  Extraindo ....................... ok
  Instalando o ARCA em ARCABOOT ... ok
  Entrada de firmware ............. migrada de "Clonezilla"

Dispositivo pronto.
```

## 8. Comandos

```
arca prepare              # instala o Clonezilla e o ARCA num dispositivo pronto
arca backup <nome>        # monta a receita, arma o boot, reinicia
arca resultado            # le o veredito e desarma o SSD
arca list                 # imagens no dispositivo conectado
arca restore [<nome>]     # lista, confirma e reinicia para restaurar
                          #   --destino <indice>  restaura em outro disco (R-7)
arca verify <nome>        # confere os MD5SUMS, sem reiniciar
arca status               # diagnostico: dispositivo, firmware, job pendente
arca desarmar             # devolve o dispositivo ao estado inerte (§4.4)
```

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

Três flags:

```
--dry-run                 # imprime a receita e o que faria; nao arma nada
--completo                # em verify: arma boot unico para o ocs-chkimg
--destino <indice>        # em restore: outro disco de destino (R-7)
```

> **`--destino` é o índice do Windows, e não o nome do Linux.** Acrescentado na
> etapa E9. Sem ele, a metade permissiva de R-7 — *"destino diferente é
> permitido"* — seria inalcançável, e a recusa por destino menor seria uma
> regra que nunca dispara. Aceitar `--destino nvme0n1` seria pôr numa receita
> destrutiva um nome do **Linux** digitado do lado Windows, que é exatamente o
> que a E7 recusou por não ter contra o que conferi-lo (§4.5). O ARCA traduz:
> índice → modelo, pelo WMI; modelo → nome do Linux, pelo `blkdev.list` de
> dentro das imagens. Um disco que nenhuma imagem viu não tem nome, e não entra
> numa receita.
>
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
| C-4 | Procurar a entrada `ARCA`; não havendo, migrar a legada `Clonezilla` em vez de criar outra. **Migrar é renomear a `description`** — o GUID, o `device` e o `path` já são os certos, e criar uma segunda entrada deixaria a máquina com duas formas de bootar no Clonezilla. *(Etapa E7: **não havendo nenhuma das duas, o ARCA recusa em vez de criar.** Criar uma entrada de firmware do zero é código sem original — nenhuma captura mostra a forma —, e o lugar disso é o `arca prepare` da E10. Armar não é a hora de estrear a criação de entrada de boot.)* |
| C-5 | Boot único — nunca alterar a ordem permanente. *(Etapa E7: medido que o `bcdedit` **aceita** `bootsequence` para uma entrada de fora do `displayorder`, e que o `displayorder` não muda nem ao pôr nem ao tirar. Sem isso, armar obrigaria a violar este requisito. A ordem permanente é lida antes de escrever e comparada depois — em `armar` como em `desarme` —, e uma divergência é falha ainda que a marca tenha pegado.)* **Com pedido de revisão em aberto — ver P-20.** O perigo que este requisito nomeia é o ARCA **acrescentar** um caminho permanente para o dispositivo; pôr o `{bootmgr}` à frente **remove** um. A redação atual não distingue as duas, e a distinção nunca foi discutida |
| C-6 | **Recusar mídia removível como alvo de entrada de boot; orientar F12.** A recusa não se lê numa etiqueta do `bcdedit` — essas palavras não saem dele (§3.1). Verifica-se de dois jeitos: o **`MediaType` do WMI** dá o sinal antecipado, e a releitura de C-3 revela a rejeição como um `device` que não mudou. *(Etapa E6: o sinal antecipado era o `GetDriveType`, que classifica o SSD externo desta mesa como disco **fixo** e não distingue nada. O `MediaType` responde literalmente `External hard disk media` e `Removable Media` — são as palavras da §3.1, e é de lá que elas saem.)* *(Etapa E7: a **segunda** metade passa a existir. Ao armar, o ARCA escreve o `device` da entrada apontando para o `ARCABOOT` que está na mesa e relê; um `device` que não mudou é a rejeição silenciosa, e o armar para ali. Escreve **sempre**, mesmo quando o valor já está certo — é a releitura que responde, e pular a escrita no caso normal deixaria justamente o caminho normal sem exercício, que é o mesmo raciocínio de `desarme` sobre o `deletevalue`.)* |
| C-7 | Repassar os argumentos ao relançar com elevação por UAC |
| C-8 | Escapar aspas com **barra invertida**, não crase — quem reparte a linha é o parser do Windows |
| C-9 | Avisar, antes de reiniciar, para remover o SSD ao terminar. **Depois de armado e antes do reinício** — é a última coisa que alguém lê antes de a tela apagar, e não há tela do outro lado (§5.2) |
| C-10 | *(Etapa E9: as duas recusas que falam do **dispositivo** — esta e C-6 — passam a valer também para o `arca restore`, e antes da confirmação digitada. O `armar` pegaria a rejeição silenciosa de C-6 na releitura, mas depois de a pessoa ter digitado o nome de uma imagem que vai apagar um disco; e o dispositivo partido levaria o `estado.json` para o `ARCABOOT` de um dispositivo com o desfecho indo para o `ARCAVAULT` do outro.)* **Recusar mais de um dispositivo ARCA conectado.** Dois `ARCAVAULT` ou dois `ARCABOOT` tornam o destino ambíguo, e é por LABEL que a receita resolve (S-3). **E recusar também o dispositivo partido**: os dois rótulos em discos físicos diferentes são dois dispositivos meio prontos, e não um — cada rótulo aparece uma vez, a contagem passa, e a receita iria para um enquanto as imagens estão no outro. *(A brecha do rótulo órfão ficou aberta da E1 à E5, com a letra impressa na tela como única defesa; a enumeração de discos da E6 a fecha.)* |
| C-11 | **Gerar um selo ao armar**, gravá-lo no `estado.json` e embuti-lo na receita; aceitar como desfecho apenas o `arca-fim.txt` cujo selo case (§4.3) |
| C-12 | **Ausência de desfecho é falha, nunca silêncio.** Havendo job pendente e nenhum `arca-fim.txt`, reportar as duas causas possíveis: o boot não ocorreu, ou o Clonezilla abriu menu (§5.5). *(Etapa E8: ausência de desfecho **encerra** o job, porque é um veredito. O que não encerra é o `arca-fim.txt` que está lá e não se deixou ler — "não consegui olhar" não é veredito, e encerrar ali perderia o selo. Ver [ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md).)* |

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
| R-7 | Destino diferente do disco de origem é **permitido**, e nomeado por `--destino <indice>` — o índice do **Windows**, que o ARCA traduz para o nome do Linux pelo `blkdev.list` (§4.5) e que nunca chega à receita. Recusar sempre que o destino for **menor** que a origem. *(Etapa E9: ~~`-k0` copia a tabela inteira e, num disco menor, corrompe em vez de falhar~~ — **a premissa estava errada, e P-17 é isso**. O help do `ocs-sr` desta versão diz que o Clonezilla **confere o tamanho do destino por padrão e desiste** se for menor; `-icds` é quem desligaria a conferência, e a receita não o usa. A recusa do ARCA fica, e a razão passa a ser **onde** ela acontece: a do Clonezilla custa um reinício de uma operação destrutiva, e a do ARCA custa zero. A comparação sai em **setores**, com o destino medido pelo `MSFT_Disk` e a origem lida do `<disco>-gpt.sgdisk` de dentro da imagem — as duas na mesma régua. Setor lógico diferente entre origem e destino é recusa, e não conversão. Ver [ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md).)* Em disco novo, `-iefi` não encontra entrada correspondente e o `bcdboot` volta a ser necessário — ao contrário do que §3.4 mediu no disco original |
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
| V-1 | `arca verify <nome>` confere os `MD5SUMS` no Windows, em segundos, sem reiniciar. Pega corrupção de mídia e cópia truncada |
| V-2 | `arca verify <nome> --completo` arma boot único que só roda `ocs-chkimg`. É outra força de verificação: **não substitui B-9**, que continua obrigatória em todo backup |

### 9.6 — Preparação de dispositivo

| ID | Requisito |
|---|---|
| PR-1 | Versão do Clonezilla **fixada**, com o SHA256 esperado **compilado no binário do ARCA** — nunca baixado junto do arquivo, o que não verificaria nada. Não batendo, recusar e parar |
| PR-2 | `arca prepare --iso <caminho>` instala de arquivo local. É o que salva quando a máquina que precisa preparar o dispositivo é justamente a que está sem Windows |
| PR-3 | Guardar no `ARCAVAULT` uma cópia do pacote usado. Dispositivo autocontido inclui poder reconstruir o dispositivo |

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
| `;` em vez de `if/then/else` | Falha deixa o mesmo rastro que sucesso | R-5 — e a defesa **rodou em 22/08/2026**, pela primeira vez, tomando o ramo do sucesso. O ramo da falha continua sem rodar (P-6) |
| Documentar como fundação o que veio do trabalho de validação | O `ARCA_VEREDITO=`, o `arca-fim.txt` de 21/08, o `set default`, o `498,7 GB` e a ordem de boot com o dispositivo à frente pareciam medidas, e vieram do trabalho em volta | Procurar o original em `recursos/capturas/` antes de chamar qualquer coisa de medida. **Em 22/08 o padrão não se repetiu**: o `arca-fim.txt` do marco tem original, e o que atesta que a receita o escreveu é o `Info-saved-by-cmd.txt` que o Clonezilla escreve sozinho |
| Relógio do Clonezilla 3h adiantado | Ele lê o RTC (hora local do Windows) como UTC. Uma trava construída sobre comparação de datas reprovou um backup perfeito | S-6. **Confirmado outra vez em 22/08, pelo outro lado**: o `arca-fim.txt` escrito às 21:06 tem `mtime` de 18:06 lido do Windows, e parece anterior ao job que o produziu. É o mesmo instante em dois fusos, e é por isso que quem liga desfecho a job é o selo. **E de novo em 23/08, com a mesma diferença e o mesmo sinal**: o log diz `Ending /usr/sbin/ocs-sr at 2026-08-23 11:31:55 UTC`, o `mtime` visto do Windows é 08:31:55, e o job foi armado às 11:10:50 — o desfecho parece ter sido escrito quarenta minutos antes de a operação começar |
| **Medir o firmware depois do reinício e achar que se mediu o reinício** | As duas leituras do `bcdedit` de 22/08 discordam entre si, e as duas estão certas: a ordem de boot mudou no meio — e quem a mudou foi o próprio ciclo de boot. Uma leitura feita no Windows descreve o firmware **como ele ficou** | Ler a NVRAM de dentro do live, que é onde o boot está acontecendo. O Clonezilla já grava `efi-nvram.dat` em toda imagem, de graça (§3.1, [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)) |
| Argumentos perdidos na reelevação | `--dry-run` virou execução real, sem aviso | C-7 |
| Crase como escape | O parser do Windows reparte a linha, não o do PowerShell | C-8 |
| Job fantasma | Imagem feita quando o ARCA ainda morava no `C:` carrega dentro de si um `estado.json` pendente apontando para si mesma. §4.1 elimina a causa daqui para frente; imagens antigas continuam trazendo o problema de volta | C-11 |
| ARCA dentro da imagem | Restaurar devolve versões antigas com defeitos já corrigidos | §4.1 |
| Pasta sem `MD5SUMS` | Resíduo de backup interrompido; recusar só imagem válida empurra o usuário a regravar por cima dos fragmentos | B-3 |
| Boot no removível após `poweroff` | Não reproduzido, causa não determinada | C-9 |
| `set default="0"` no `grub.cfg` | Aponta por **posição**, e o `menuentry` do ARCA entra antes do `live-default`: inserir o bloco arma sozinho, sem ninguém tocar no `set default`. Um dispositivo assim não está inerte, está parecendo inerte | Desarmar devolve o `set default` para `live-default` qualquer que seja o valor que encontrou (§4.4, ADR-0005) |
| `bcdedit /deletevalue` chamando de erro não ter o que apagar | Apagar um `bootsequence` que não existe sai com código 1 sem mudar nada. Um desarmar que propagasse isso falharia justamente no caso normal, e a idempotência de C-1 nunca passaria | Descartar o que o `bcdedit` responde e conferir com `/enum` (C-3) |
| **A ordem permanente com o dispositivo à frente** | A máquina boota no dispositivo **sem** boot único nenhum. Com o dispositivo inerte isso para no menu do Clonezilla; **com uma receita armada, ela roda** — e a janela em que o `grub.cfg` fica armado vai do fim da receita ao `arca resultado`, oito minutos em 22/08. E não é preciso ninguém para chegar nesse estado: o ciclo de boot põe a entrada de volta na ordem (ADR-0009) | C-5 impede o ARCA de pôr e de tirar. `arca status` **avisa** quando alguma entrada que leva ao `ARCABOOT` está em primeiro — por alvo, nunca por nome, porque desde o marco há **duas** apontando para lá —, e `tests/e7_armar_o_dispositivo.rs` cobra a invariante: em primeiro na ordem, o dispositivo tem de estar inerte. A defesa de sempre é o aviso de C-9 — remover o SSD antes de religar |
| Ler duas ferramentas em momentos diferentes e chamar a diferença de discordância | O `bcdedit` de 22/08 e o `efibootmgr` de 20/08 dizem coisas diferentes sobre a ordem de boot, e as duas estão certas: a ordem mudou no meio. Datar cada captura desfaz a contradição inteira. **Em 22/08 aconteceu com a mesma ferramenta**, duas leituras separadas por trinta e seis minutos | A tabela do §3.1 traz a data — e agora a hora — de cada leitura, e `recursos/capturas/PROVENIENCIA.md` diz de onde cada arquivo veio |
| **Datar a captura e não saber de que operação ela é** | Duas leituras de NVRAM de 21/08 estavam na tabela do §3.1 como uma linha só, explicando o backup daquele dia. Uma é do backup e a outra é da **restauração**, uma hora e meia depois, com um boot no Windows no meio — e elas discordam sobre a ordem de boot justamente por isso. O que as juntou foi o nome da pasta, que a E3 escolheu por outro motivo inteiramente | Datar **e nomear a operação**. A tabela do §3.1 traz as duas separadas, e a `PROVENIENCIA.md` diz qual operação escreveu cada arquivo ([ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md)) |
| **Duas medidas do mesmo disco, em réguas diferentes** | O `MSFT_Disk` e o `Win32_DiskDrive` respondem 500.107.862.016 e 500.105.249.280 para o **mesmo** disco. O segundo é `60801 × 255 × 63 × 512` — a geometria CHS legada truncada no último cilindro. Medir a origem pela GPT de dentro da imagem e o destino pelo `Win32_DiskDrive` faz R-7 recusar um disco por não caber **nele mesmo** | R-7 mede o destino pelo `MSFT_Disk` e compara em setores; `DiscoFisico::medida` existe ao lado do `tamanho_bytes` com doc dizendo por quê, e há teste que afirma o número errado para que trocar a fonte de volta não seja silencioso ([ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)) |
| **Uma recusa por identidade do Windows guardando um valor do Linux** | R-8 recusa o dispositivo como destino **pela letra** (`E:`, `R:`); o nome que entra na receita é do **Linux**, e sai de um casamento por **modelo** nos `blkdev.list` (§4.5). São dois canais de identidade, e o vão entre eles apagava o dispositivo: com um segundo disco do mesmo modelo dele, `--destino <o outro>` passava pela recusa por letra e a receita saía `restoredisk <imagem> sda` — o `sda` é o dispositivo. A recusa dura tinha um contorno por acidente de modelo | Resolver o nome do Linux do **dispositivo** pelo mesmo oráculo e comparar com o do destino. Achado pela revisão de código da E9, e é o defeito mais grave que ela pegou |
| **Uma identidade vazia casando com tudo ou com nada** | O `Model:` do `<disco>-gpt.sgdisk` era opcional, e um modelo vazio viajava: a conferência de R-2 recusava uma imagem coerente por "as fontes discordam", a busca do destino dizia "nenhum disco tem o modelo ``", e a tela de confirmação imprimia `Origem da imagem:  · nvme0n1`. É o mesmo raciocínio que faz o leitor do WMI exigir o `Model` em vez de supor | Exigir o `Model:`, com recusa própria. O modelo é a identidade do disco, e o ARCA não confere um destino contra identidade nenhuma |
| **A recusa engolindo a notícia do desarmar** | C-1 desarma incondicionalmente, como primeiro passo; uma recusa posterior sobe como erro e corta a saída. Quem rodasse `arca restore --destino <errado>` num dispositivo armado veria só a recusa, e o job armado teria sumido em silêncio. **Aconteceu duas vezes**: a revisão da E7 pegou no `arca backup`, e a E9 cometeu de novo no `arca restore` — com o comentário que descreve o defeito a poucas linhas de distância | Imprimir o que já aconteceu **antes** de julgar. As duas telas saem em duas metades, e a primeira traz o desarmar. Achado rodando o comando de verdade, e não relendo o código |
| **Um par honesto respondendo por um evento maior do que ele** | O §3.4 dizia que a restauração não mexe na NVRAM, sustentado num par `antes`/`depois` real, do mesmo evento, com hora — nada dele veio de trabalho manual. Só que as duas leituras são **do mesmo boot do live**, e por isso só podiam falar do `ocs-sr`. O ciclo inteiro faz outra coisa: a ordem permanente volta ao que está dentro da imagem. Procurar o original não acharia este defeito, porque o original estava lá | Perguntar **entre que dois instantes** a evidência foi tirada, e se a pergunta cabe dentro deles. O par da E9 é do lado Windows e atravessa o reinício ([ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)) |
| **A operação apagando o registro de que foi armada** | O `%LOCALAPPDATA%\ARCA\arca.log` mora no `C:`. Numa restauração, a linha do armar é escrita minutos antes do reinício e **substituída pela imagem** junto com o resto do disco: o log que sobra salta do último comando de antes da imagem direto para a colheita. E a tela que a imprimiu morreu no reinício que ela disparou | §4.1 — o `estado.json` mora no `ARCABOOT` e sobrevive. É o único lugar que liga o selo do desfecho a um job, e a colheita de uma restauração se vira só com ele (§6.3) |
| **O log do Clonezilla não é o log inteiro** | O `arca-restore.log` do marco tem o fim da operação e não tem o começo: uma passagem só do Partclone — a da última partição, 1,1 GB —, nenhuma das outras três, e um `Ending /usr/sbin/ocs-sr` sem o `Starting` correspondente. Causa não determinada. O §6.3 aponta esse arquivo a quem quer saber o que aconteceu, e o que está lá **pode não cobrir a parte que falhou** | Saber disso antes de concluir qualquer coisa por ausência. Medir de novo na próxima restauração, e perguntar se o corte cai sempre no mesmo lugar |

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
| O ARCA não cria partições | Princípio P1 |
| `toram` mantido | Evita acoplar o live system ao dispositivo que ele remonta |
| Job ligado ao desfecho por selo, nunca por data | Não há relógio comum entre Windows e Clonezilla (P-7) |
| A receita continua sendo string no `grub.cfg`, não arquivo | É o mecanismo medido em hardware. Trocá-lo por um `custom-ocs` em arquivo exigiria remedir, e `toram` pode desmontar o medium |
| Restaurar em disco diferente é permitido, recusado só se menor | O disco de origem morrer é o motivo de existir backup de imagem |
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

### Pendências

| # | Questão |
|---|---|
| P-6 | O `ocs-sr` devolve código ≠ 0 quando falha? Fecha com falha forçada em VM |
| P-7 | O deslocamento de 3 h é permanente. Existe para a próxima pessoa que for comparar datas |
| ~~P-18~~ | ~~O boot único pode nunca ter sido disparado por boot único.~~ **Fechada em 22/08/2026, etapa E8.** O `efibootmgr` do live registrou `BootCurrent: 0001` com `BootOrder: 0000,0001`: a máquina bootou por uma entrada que não era a primeira da ordem, e só o `bootsequence` explica. Ver §3.1, §3.5 e [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md) |
| P-19 | **O firmware reescreve a entrada em todo boot pelo dispositivo, ou só quando ela foi consumida por `bootsequence`?** — ver §3.5. Aberta na E8. Um segundo backup responde, e a leitura que responde já vem de graça dentro de cada imagem |
| P-14 | `arca resultado` deve rodar sozinho no logon? Começar sem, decidir com uso |
| ~~P-15~~ | ~~A receita de backup publicada em §10.1 divergia da fundação §3.2 quanto ao `-batch`.~~ **Fechada em 22/08/2026, etapa E3.** `-batch` rodou, nas três receitas preservadas em `recursos/capturas/`. O help do `ocs-sr` diz por que é `-batch` e não `-b`: em parâmetro de boot, o `init` do sistema também honraria `-b` |
| ~~P-16~~ | ~~O mecanismo de desfecho nunca rodou.~~ **Fechada em 22/08/2026, etapa E8.** `arca-fim.txt`, selo na receita, `ARCA_FIM` e `if/then/else` estrearam todos de uma vez, e o selo do desfecho bate com o do `estado.json`. Ver §3.5 |
| ~~P-17~~ | ~~`-icds` contradiz R-7.~~ **Fechada em 23/08/2026, etapa E9.** O help está certo e a premissa de R-7 estava errada: o Clonezilla confere e **desiste**, não corrompe. R-7 foi reescrito — a recusa fica, e a razão passa a ser que a do Clonezilla acontece do outro lado do reinício. E resolver isso obrigou a descobrir a armadilha da régua: o `Win32_DiskDrive` e o `MSFT_Disk` dão dois tamanhos para o mesmo disco. Ver [ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md) |
| P-21 | **O `ocs-sr` sai com código ≠ 0 quando desiste por destino menor?** — ver §3.5. É P-6 com outra roupa, e não é urgente: R-7 recusa antes, do lado Windows |
| P-22 | **O `bcdedit /enum firmware` mostra a NVRAM, ou o BCD do disco?** — ver §3.5. Aberta no marco da E9, e importa porque a linha `Ordem de boot` do `arca status` é uma afirmação de segurança lida dali. Fecha com um reinício com o SSD conectado, sem job armado |
| P-23 | **Por que o `arca-restore.log` começa no meio?** — ver §3.5. Aberta no marco da E9. O §6.3 aponta esse arquivo a quem quer saber o que aconteceu, e ele não traz a operação inteira. Fecha na próxima restauração |

---

*Documento vivo. Atualizar após cada medição em hardware.*
