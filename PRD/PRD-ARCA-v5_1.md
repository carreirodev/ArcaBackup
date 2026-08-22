# PRD — ARCA v5.1

**Automatizador de Clonezilla para backup e restauração de imagem de disco.**

Versão 5.1 · 22/08/2026 · Substitui a v4
Última revisão: 22/08/2026, etapas E7 e E8 — §3.1 ganhou a **tabela de ordem de boot** desta máquina, que mostra a ordem permanente alterada pelo menos três vezes e dá a P-18 uma terceira explicação; §10.2.1 corrigido (o `menuentry` base é o **`live-toram`**, e o `toram` nunca foi acrescentado); §5.2 ganhou as cinco linhas do armar e a ordem certa entre confirmação, aviso e reinício; §4.5 decide o que fazer sem nome de disco (**recusar**); §4.3 e §5.4 ganharam o `estado.json` de seis campos e a linha `Job: encerrado`
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
| Entrada de firmware apontando para SSD externo funciona | A máquina bootou pela entrada de firmware do ARCA, múltiplas vezes. **Que o disparo tenha sido por boot único, e não por F12 nem pela ordem permanente, é o que a evidência não separa** — ver P-18 e a tabela abaixo |
| `bcdedit` **rejeita mídia removível em silêncio** — responde "êxito" e mantém o valor antigo | Pendrive testado e recusado; SSD aceito |
| Partição primária comum basta — não precisa marcar tipo EFI | SSD preparado assim boota normalmente |
| O `bcdedit` **não traduz** os nomes de campo: só `identificador` sai em português | Parser por valor é o correto |
| A entrada legada desta máquina chama-se **`Clonezilla`**, GUID `{f4057bd0-…}` | Procurar só por `ARCA` criaria entrada órfã |
| **O `bcdedit` aceita `bootsequence` para uma entrada que não está no `displayorder`**, e o `displayorder` não muda ao pôr nem ao tirar | Medido na etapa E7, 22/08/2026, com a entrada do ARCA fora da ordem. É o que torna C-5 possível: se o boot único exigisse a entrada na ordem, armar obrigaria a violá-lo ([ADR-0007](../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md)) |

#### A ordem permanente desta máquina mudou pelo menos três vezes

Medido na etapa E7. Todas as capturas de `efibootmgr` do dispositivo, mais as
duas leituras do `bcdedit`, na ordem em que foram feitas:

| Quando | Ferramenta | Ordem de boot | Bootou por | `BootNext` |
|---|---|---|---|---|
| 20/08 | `efibootmgr` (`nvram-original.txt`) | `0000,0001` — Windows, ARCA | `0001` (ARCA) | nenhum |
| 20/08 | `bcdedit` (`nvram-windows-antes.txt`) | `{bootmgr}`, **`{f4057bd0}`**, +3 pseudo-entradas | — | — |
| 20/08 | `efibootmgr` (`R1/nvram-antes.txt` e `-depois`) | `0000,0001` | `0001` (ARCA) | nenhum |
| 20/08 | `efibootmgr` (`R2/nvram-antes.txt` e `-depois`) | `0003,0000` — **ARCA, Windows** | `0003` (ARCA) | nenhum |
| 21/08 | `efibootmgr` (`2026-08-21_WindowsCompleto/`) | `0001,0000` — **ARCA, Windows** | `0001` (ARCA) | nenhum |
| 22/08 | `bcdedit` | `{bootmgr}` — **a entrada do ARCA saiu da ordem** | — | — |

**As duas ferramentas não discordam: elas foram lidas em momentos
diferentes.** Em 20/08 as duas dizem a mesma coisa — a entrada do ARCA estava
na ordem permanente, em segundo lugar. O `bcdedit` mostra três pseudo-entradas
a mais (`UEFI:CD/DVD Drive`, `UEFI:Removable Device`, `UEFI:Network Device`)
que o `efibootmgr` não vê de dentro do Clonezilla; nas entradas reais os dois
concordam. O número da entrada do ARCA também mudou — `0001` virou `0003` —, o
que só acontece quando ela é recriada.

Três coisas saem daí, e a terceira é a que importa:

- **A ordem permanente foi alterada por alguém, mais de uma vez.** C-5 existe
  para impedir que o ARCA faça isso, e a evidência é de que já foi feito à mão.
- **Hoje a entrada do ARCA não está na ordem**, e é sobre essa configuração
  que o boot único tem de funcionar. `tests/e7_armar_o_dispositivo.rs` cobra
  isso a cada execução: se ela voltar para a ordem, a medição do ADR-0007 deixa
  de significar o que significa.
- **No backup validado de 21/08 o dispositivo estava em primeiro na ordem
  permanente.** Isso é uma explicação completa para a máquina ter bootado nele,
  e ela **não passa por boot único**. É P-18 com evidência apontando para o
  lado desconfortável: o mecanismo que este documento chamava de fundação
  validada pode nunca ter rodado, enquanto o que de fato rodou é exatamente o
  que C-5 proíbe.

O `BootNext` ausente em todas as oito capturas continua não provando nada — o
firmware o consome ao usá-lo, e todas foram feitas já de dentro do Clonezilla.

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

### 3.4 — Restauração validada

Restauração real sobre o `nvme0n1`. Do comando ao Windows restaurado, **sem intervenção, na primeira tentativa**.

| Fato | Evidência |
|---|---|
| `-iefi` funciona | NVRAM byte-idêntica antes e depois |
| `-k0` preserva os PARTUUIDs **mesmo com a GPT zerada** | A entrada de boot preexistente continua resolvendo |
| `bcdboot` não é necessário neste hardware | Consequência do anterior |
| O Windows da imagem sobe normalmente | Máquina restaurada e em uso |

> **O `-iefi` era a pergunta que originou o projeto.** Está respondida: a restauração não toca na NVRAM e o Windows sobe.

### 3.5 — Ainda não medido

| # | Pendência |
|---|---|
| P-6 | **O `ocs-sr` devolve código diferente de zero quando falha?** O ramo de sucesso foi medido; o de falha não. Uma restauração bem-sucedida não fecha isso, por definição. Fecha com falha forçada, provavelmente em VM |
| P-16 | **O mecanismo de desfecho nunca rodou.** Nenhuma das três receitas preservadas escreve `arca-fim.txt`, grava selo ou usa `if/then/else`. O `arca-fim.txt` que existe no dispositivo veio de trabalho manual de validação, como o `ARCA_VEREDITO=` do ADR-0003. **S-4, C-11, C-12, R-5 e R-6 são código novo**, e a E7 é a primeira execução de todos eles ao mesmo tempo |
| P-18 | **O boot único da §3.1 pode nunca ter sido disparado por boot único**, e a etapa E7 estreitou a pendência sem fechá-la. As capturas de NVRAM mostram `BootCurrent: 0001` e `Boot0001* ARCA`: a máquina bootou pela entrada de firmware do ARCA, confirmado. Isso continua indistinguível de um F12 na mesma entrada — e a E7 acrescentou uma **terceira** explicação, com evidência: no backup validado de 21/08 o dispositivo estava **em primeiro na ordem permanente** (§3.1). Uma ordem de boot com o dispositivo à frente explica o boot inteiro sem passar por `bootsequence`. O que a E7 mediu foi a metade que se pode medir do lado Windows: o `bcdedit` **aceita** a marca sobre uma entrada fora da ordem, e a releitura a confirma (ADR-0007). Se o firmware a **honra** é o que só o marco em hardware responde. Aberta na E4, estreitada na E7 |

> **Uma advertência sobre esta seção inteira.** Duas vezes já se descobriu que
> algo documentado como fundação validada na verdade veio do **trabalho de
> validação em volta dela**, e não da receita: o `ARCA_VEREDITO=` (ADR-0003) e
> agora o `arca-fim.txt` (P-16). O padrão se repete porque a evidência que
> sobra no dispositivo não distingue o que a receita escreveu do que uma
> pessoa escreveu depois. Antes de tratar qualquer linha desta seção como
> medida, vale procurar o original em `recursos/capturas/`.

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

O documento pressupunha este estado sem nunca defini-lo. O §6.3 conta com ele
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
contornar: este documento já registrou três vezes (P-16 e os ADRs 0003, 0004 e
0005) que chamar de fundação validada o que veio do trabalho de validação em
volta dela é o erro que mais custou neste projeto. Inventar uma derivação e
documentá-la como descoberta seria a quarta.

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
feito uma vez pelo menu do Clonezilla (§6.3). Dali em diante o `blkdev.list`
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

Executado de verdade em 22/08/2026, até a linha antes da confirmação:

```
> arca backup 2026-08-22_Apps

Dispositivo ARCA: ARCAVAULT (E:) · 164 GB livres
Origem: KINGSTON SNV3S500G · 465,8 GB · 105,6 GB em uso
Imagem estimada: ~47,5 GB · espaco suficiente
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
  Selo do job ..................... a3f1c9e07b2d4856
  Desfecho esperado em ............ backup-2026-08-22_Apps

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
> ninguém como conferir, à mão, se o desfecho que voltou é deste job.

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

```
> arca resultado

Backup 2026-08-22_Apps
  22/08 · 36,2 GB
  Desfecho: concluida — o selo bate e a receita chegou ao fim
  Verificacao: APROVADA
  Selo: a3f1c9e07b2d4856

  Desarmando SSD .................. ok · R:\boot\grub\grub.cfg
  Job ............................. encerrado · o desfecho foi lido e dito

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 36,2 GB · aprovada

164 GB livres
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
> **Os números desta tela ainda não são de uma execução real do ARCA.** A tela
> do §5.2 foi corrigida contra medição na etapa E6; esta espera o marco em
> hardware da E8. O que já está medido é o `164 GB livres`, do dispositivo
> desta mesa em 22/08/2026.

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

```
> arca restore

Imagens em ARCAVAULT:
  [1] 2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  [2] 2026-08-22_Apps              22/08 · 36,2 GB · aprovada

Qual restaurar? 2

Origem da imagem: KINGSTON SNV3S500G (conferido contra blkdev.list)
Destino:          KINGSTON SNV3S500G · 498,7 GB

ATENCAO: a restauracao APAGA o disco de destino.
Tudo que estiver nele sera perdido.

Digite o nome da imagem para confirmar: 2026-08-22_Apps

A maquina vai reiniciar e restaurar sem intervencao.
AO TERMINAR: remova o SSD antes de religar.

Reiniciando...
```

A escolha acontece **no Windows**, com a lista à vista. O Clonezilla executa sem perguntar nada.

### 6.2 — Verificação do alvo

Cada pasta de imagem carrega a identidade do disco de origem em `disk` e `blkdev.list`. O ARCA confere o destino contra o conteúdo da própria imagem — não confia na suposição de disco único.

### 6.3 — Windows não boota

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
arca restore              # lista, confirma e reinicia para restaurar
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

Duas flags:

```
--dry-run                 # imprime a receita e o que faria; nao arma nada
--completo                # em verify: arma boot unico para o ocs-chkimg
```

Todos exigem privilégio administrativo.

## 9. Requisitos

### 9.1 — Comuns a toda operação

| ID | Requisito |
|---|---|
| C-1 | **Desarmar a receita anterior incondicionalmente**, como primeiro passo, sem consultar estado nenhum. O estado a que se volta está definido no §4.4, e é reconstruído do `grub.cfg` corrente — o que torna a operação idempotente sem que ninguém precise garanti-lo |
| C-2 | **Validar a receita antes de gravar** no `grub.cfg`: rejeitar pipes, **toda** aspa (não só as desbalanceadas — um par de aspas simples fecha o `bash -c` e abre outra string), substituição de comando, caractere de controle, não-ASCII, e a linha que não coubesse no `COMMAND_LINE_SIZE` do kernel (§10.2.3). Nomes inseguros já param antes, em B-2 |
| C-3 | Nunca confiar no retorno do `bcdedit`; sempre conferir com `/enum` e parsear **por valor** |
| C-4 | Procurar a entrada `ARCA`; não havendo, migrar a legada `Clonezilla` em vez de criar outra. **Migrar é renomear a `description`** — o GUID, o `device` e o `path` já são os certos, e criar uma segunda entrada deixaria a máquina com duas formas de bootar no Clonezilla. *(Etapa E7: **não havendo nenhuma das duas, o ARCA recusa em vez de criar.** Criar uma entrada de firmware do zero é código sem original — nenhuma captura mostra a forma —, e o lugar disso é o `arca prepare` da E10. Armar não é a hora de estrear a criação de entrada de boot.)* |
| C-5 | Boot único — nunca alterar a ordem permanente. *(Etapa E7: medido que o `bcdedit` **aceita** `bootsequence` para uma entrada de fora do `displayorder`, e que o `displayorder` não muda nem ao pôr nem ao tirar. Sem isso, armar obrigaria a violar este requisito. A ordem permanente é lida antes de escrever e comparada depois — em `armar` como em `desarme` —, e uma divergência é falha ainda que a marca tenha pegado.)* |
| C-6 | **Recusar mídia removível como alvo de entrada de boot; orientar F12.** A recusa não se lê numa etiqueta do `bcdedit` — essas palavras não saem dele (§3.1). Verifica-se de dois jeitos: o **`MediaType` do WMI** dá o sinal antecipado, e a releitura de C-3 revela a rejeição como um `device` que não mudou. *(Etapa E6: o sinal antecipado era o `GetDriveType`, que classifica o SSD externo desta mesa como disco **fixo** e não distingue nada. O `MediaType` responde literalmente `External hard disk media` e `Removable Media` — são as palavras da §3.1, e é de lá que elas saem.)* *(Etapa E7: a **segunda** metade passa a existir. Ao armar, o ARCA escreve o `device` da entrada apontando para o `ARCABOOT` que está na mesa e relê; um `device` que não mudou é a rejeição silenciosa, e o armar para ali. Escreve **sempre**, mesmo quando o valor já está certo — é a releitura que responde, e pular a escrita no caso normal deixaria justamente o caminho normal sem exercício, que é o mesmo raciocínio de `desarme` sobre o `deletevalue`.)* |
| C-7 | Repassar os argumentos ao relançar com elevação por UAC |
| C-8 | Escapar aspas com **barra invertida**, não crase — quem reparte a linha é o parser do Windows |
| C-9 | Avisar, antes de reiniciar, para remover o SSD ao terminar. **Depois de armado e antes do reinício** — é a última coisa que alguém lê antes de a tela apagar, e não há tela do outro lado (§5.2) |
| C-10 | **Recusar mais de um dispositivo ARCA conectado.** Dois `ARCAVAULT` ou dois `ARCABOOT` tornam o destino ambíguo, e é por LABEL que a receita resolve (S-3). **E recusar também o dispositivo partido**: os dois rótulos em discos físicos diferentes são dois dispositivos meio prontos, e não um — cada rótulo aparece uma vez, a contagem passa, e a receita iria para um enquanto as imagens estão no outro. *(A brecha do rótulo órfão ficou aberta da E1 à E5, com a letra impressa na tela como única defesa; a enumeração de discos da E6 a fecha.)* |
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
| R-5 | Receita com `if/then/else`: escrever `ARCA_RESTORE=OK` ou `ARCA_RESTORE=FALHOU`. **Código novo** — as três receitas preservadas encadeiam com `;` (P-16) |
| R-6 | Ler esse arquivo na volta e **conferir o selo antes de acreditar nele** (C-11). O job fantasma que isto previne é **risco herdado**, e não corrente: §4.1 eliminou a causa ao tirar o ARCA do `C:`, e só imagens feitas antes disso carregam estado dentro de si. O selo cobre de qualquer forma, e é o mesmo mecanismo dos outros três casos (§4.3) |
| R-7 | Destino diferente do disco de origem é **permitido**, com confirmação que nomeia o disco de destino. Recusar sempre que o destino for **menor** que a origem: `-k0` copia a tabela inteira e, num disco menor, corrompe em vez de falhar. Em disco novo, `-iefi` não encontra entrada correspondente e o `bcdboot` volta a ser necessário — ao contrário do que §3.4 mediu no disco original |

### 9.4 — Segurança

| ID | Requisito |
|---|---|
| S-1 | O ARCA nunca abre o disco de origem em **acesso raw** de escrita. Chamar `powercfg` ou `chkdsk` (B-5, B-6) não é isso: são operações do próprio sistema, pelas quais o Windows responde |
| S-2 | Operação destrutiva exige texto digitado, nunca só `s`. **Comparação exata**, sem ignorar caixa e sem aceitar prefixo: B-2 permite maiúscula e minúscula, e `2026-08-22_apps` é uma imagem diferente de `2026-08-22_Apps`. Uma tentativa só — quem digitou errado repete o comando, que até ali não armou nada. `--dry-run` pula a confirmação **e** o armar, e não diz que armou |
| S-3 | Destino sempre por LABEL — nunca por letra, `sda` ou número de série |
| S-4 | Veredito e desfecho sempre gravados em arquivo, nunca só em tela — o `arca-check.log` e o `arca-fim.txt`, ambos escritos pela receita. **Código novo**: nenhuma receita real chegou a escrever `arca-fim.txt` (P-16) |
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

A recusa acontece nos dois pontos: B-2 limita o nome a 48 caracteres, e a montagem da receita confere a linha pronta contra os 1536 — porque o limite do nome é uma estimativa e o tamanho da linha é o fato.

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
| O `if/then/else` de R-5 | **Código novo** — as três encadeiam com `;` |
| O `arca-fim.txt`, o `ARCA_SELO=`, o `ARCA_FIM` | **Código novo** — nenhuma receita real o escreveu (P-16) |
| O `ARCA_VEREDITO=` no `arca-check.log` | **Código novo** — ADR-0003 |
| O `sleep 20` | **Código novo** — nenhuma captura o tem |

Ver [ADR-0004](../docs/adr/0004-a-receita-transcreve-o-que-rodou.md).

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
| `;` em vez de `if/then/else` | Falha deixa o mesmo rastro que sucesso | R-5 — e a defesa nunca rodou: as três receitas preservadas usam `;` (P-16) |
| Documentar como fundação o que veio do trabalho de validação | O `ARCA_VEREDITO=` e o `arca-fim.txt` do dispositivo pareciam prova de que a receita os escrevia. Nenhuma escreve | Procurar o original em `recursos/capturas/` antes de chamar qualquer coisa de medida |
| Relógio do Clonezilla 3h adiantado | Ele lê o RTC (hora local do Windows) como UTC. Uma trava construída sobre comparação de datas reprovou um backup perfeito | S-6 |
| Argumentos perdidos na reelevação | `--dry-run` virou execução real, sem aviso | C-7 |
| Crase como escape | O parser do Windows reparte a linha, não o do PowerShell | C-8 |
| Job fantasma | Imagem feita quando o ARCA ainda morava no `C:` carrega dentro de si um `estado.json` pendente apontando para si mesma. §4.1 elimina a causa daqui para frente; imagens antigas continuam trazendo o problema de volta | C-11 |
| ARCA dentro da imagem | Restaurar devolve versões antigas com defeitos já corrigidos | §4.1 |
| Pasta sem `MD5SUMS` | Resíduo de backup interrompido; recusar só imagem válida empurra o usuário a regravar por cima dos fragmentos | B-3 |
| Boot no removível após `poweroff` | Não reproduzido, causa não determinada | C-9 |
| `set default="0"` no `grub.cfg` | Aponta por **posição**, e o `menuentry` do ARCA entra antes do `live-default`: inserir o bloco arma sozinho, sem ninguém tocar no `set default`. Um dispositivo assim não está inerte, está parecendo inerte | Desarmar devolve o `set default` para `live-default` qualquer que seja o valor que encontrou (§4.4, ADR-0005) |
| `bcdedit /deletevalue` chamando de erro não ter o que apagar | Apagar um `bootsequence` que não existe sai com código 1 sem mudar nada. Um desarmar que propagasse isso falharia justamente no caso normal, e a idempotência de C-1 nunca passaria | Descartar o que o `bcdedit` responde e conferir com `/enum` (C-3) |
| **A ordem permanente com o dispositivo à frente** | A máquina boota no dispositivo **sem** boot único nenhum, e o rastro é idêntico ao de um `bootsequence` que funcionou. Foi o estado desta máquina em 21/08, no backup que o §3.3 chama de validado | C-5 impede o ARCA de pôr; `tests/e7_armar_o_dispositivo.rs` reprova se a entrada do ARCA aparecer no `displayorder`, porque ali a medição do boot único deixa de significar o que significa (§3.1, P-18) |
| Ler duas ferramentas em momentos diferentes e chamar a diferença de discordância | O `bcdedit` de 22/08 e o `efibootmgr` de 20/08 dizem coisas diferentes sobre a ordem de boot, e as duas estão certas: a ordem mudou no meio. Datar cada captura desfaz a contradição inteira | A tabela do §3.1 traz a data de cada leitura, e `recursos/capturas/PROVENIENCIA.md` diz de onde cada arquivo veio |

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
| P-18 | **O boot único pode nunca ter sido disparado por boot único** — ver §3.5. `BootCurrent: 0001` prova que a máquina bootou pela entrada do ARCA, e não separa isso de um F12 nem da **ordem permanente**, que em 21/08 tinha o dispositivo em primeiro (§3.1). A E7 mediu que o `bcdedit` aceita a marca sobre uma entrada fora da ordem; se o firmware a honra, só o marco em hardware diz |
| P-14 | `arca resultado` deve rodar sozinho no logon? Começar sem, decidir com uso |
| ~~P-15~~ | ~~A receita de backup publicada em §10.1 divergia da fundação §3.2 quanto ao `-batch`.~~ **Fechada em 22/08/2026, etapa E3.** `-batch` rodou, nas três receitas preservadas em `recursos/capturas/`. O help do `ocs-sr` diz por que é `-batch` e não `-b`: em parâmetro de boot, o `init` do sistema também honraria `-b` |
| P-16 | **O mecanismo de desfecho nunca rodou** — ver §3.5. Fecha no marco em hardware da E7, que estreia `arca-fim.txt`, selo na receita, `ARCA_FIM` e `if/then/else` de uma vez |
| P-17 | **`-icds` contradiz R-7.** O help diz que o Clonezilla confere o tamanho do disco de destino **por padrão** e desiste se for menor que a origem; `-icds` é quem desliga essa conferência. R-7 e a decisão 5 do plano supõem o contrário — que `-k0` num disco menor corromperia em vez de falhar. A receita não usa `-icds`, e há teste cobrando isso. Resolver é da E9 |

---

*Documento vivo. Atualizar após cada medição em hardware.*
