# O `bootsequence` não é o gatilho da reescrita, e P-19 perde a hipótese que tinha

O [ADR-0011](0011-as-capturas-de-21-08-sao-de-dois-boots.md) estreitou P-19 até
sobrar uma pergunta com um candidato: a primeira metade fechou pela negativa — o
firmware **não** reescreve a entrada em todo boot pelo dispositivo —, e o que
restou foi *"só quando ela foi consumida por `bootsequence`?"*.

O experimento de 24/08/2026 rodou contra essa pergunta, e ela **não** fechou. O
candidato foi eliminado.

## O que se rodou, e o que veio de graça

O roteiro previa dois braços:

- **o braço de propósito** — bootar no dispositivo **sem** `bootsequence`, pela
  `displayorder`, com a entrada promovida à mão (o método do
  [ADR-0013](0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md)) e a NVRAM lida
  de dentro do Clonezilla live, com `efibootmgr -v`;
- **o braço de brinde** — o `efi-nvram.dat` que o Clonezilla grava dentro de
  **toda** imagem, e que o backup da fase 2 entregaria sem custo nenhum. Ele é o
  mesmo boot **com** `bootsequence`, no mesmo dispositivo, no mesmo dia.

**Foi o braço de brinde que respondeu, e a resposta foi contra a hipótese.** O
braço de propósito confirmou o que se esperava dele e não decidiu nada sozinho.

## O par controlado

Duas leituras, ambas escritas pelo `efibootmgr` **durante** o boot, separadas por
dois dias:

| | 22/08 ~20:57 | 24/08 ~20:35 |
|---|---|---|
| arquivo | `nvram-live-2026-08-22.txt` | `efi-nvram-2026-08-24_Ciclo.dat` |
| operação | backup `2026-08-22_Apps` | backup `2026-08-24_Ciclo` |
| `BootOrder` | `0000,0001` | `0000,0001` |
| `BootCurrent` | `0001` | `0001` |
| device path da `0001` | `HD(2,MBR,0x4049dea9,0x1d9d2000,0x320000)` | **o mesmo, byte a byte** |
| **descrição da `0001`** | **`UEFI OS`** | **`ARCA`** |
| **caminho** | **`\EFI\BOOT\BOOTX64.EFI`** | **`\EFI\boot\bootx64.efi`** |
| **dados da variável** | **`0000424f`** | **`BCDOBJECT={f4057bd3…}`** |

**O gatilho é o mesmo nas duas, e é o mesmo argumento que o prova.** P-18 fechou
com esta linha: `BootCurrent: 0001` com `BootOrder: 0000,0001` significa que a
máquina bootou pela entrada `0001` **estando a `0000` à frente** — nem F12, que
ninguém apertou, nem ordem permanente explicam isso; o `bootsequence` explica.
O de 24/08 traz exatamente os mesmos dois valores, e o `arca backup` é quem arma
`bootsequence` por desenho (C-5).

Mesmo gatilho, mesmo dispositivo, mesma posição na ordem, mesma leitura, mesma
operação. **Resultados opostos.** A hipótese previa que o segundo fosse igual ao
primeiro, e ele não é.

## As seis leituras de dentro do boot que este projeto tem

Reunidas aqui pela primeira vez, porque o padrão só aparece com todas na mesa. O
device path é `HD(2,MBR,0x4049dea9,0x1d9d2000,0x320000)` em **todas** — é sempre
o mesmo dispositivo, sempre a mesma tabela de partição.

| Quando | Operação | `BootOrder` | Descrição | Caminho | Dados |
|---|---|---|---|---|---|
| 20/08 | restauração R2 | `0003,0000` | `Clonezilla` | minúsculas | `BCDOBJECT` |
| 21/08 | backup | `0000,0001` | **`UEFI OS`** | **MAIÚSCULAS** | **`0000424f`** |
| 21/08 | restauração | `0001,0000` | `ARCA` | minúsculas | `BCDOBJECT` |
| 22/08 | backup | `0000,0001` | **`UEFI OS`** | **MAIÚSCULAS** | **`0000424f`** |
| 24/08 | backup | `0000,0001` | `ARCA` | minúsculas | `BCDOBJECT` |
| 24/08 | boot pela ordem | `0001,0000` | `ARCA` | minúsculas | `BCDOBJECT` |

Quatro das seis mostram a entrada na forma que o `bcdedit` escreve. Duas mostram
a forma do firmware — `UEFI OS`, caminho em maiúsculas, e os quatro bytes
`0000424f` no lugar do `BCDOBJECT`, que é a assinatura da entrada que o UEFI
**cria sozinho** ao descobrir um `\EFI\BOOT\BOOTX64.EFI` num dispositivo.

**Nenhuma coluna separa as duas das quatro.** A ordem não separa — a `0000,0001`
aparece dos dois lados. A operação não separa — há backup dos dois lados. O
gatilho não separa, que é o achado deste ADR. E o dispositivo é o mesmo em todas.

O que sobra é a **data**: as duas estão em 21 e 22/08, e as três de 20 e 24/08
não. **O que mudou entre 22 e 24 de agosto nesta placa não está identificado, e
esta mesa não tem como identificá-lo.**

## A correção do verbo, e ela é do enunciado de P-19

P-19 pergunta *"quando o firmware **reescreve** a entrada"*, e o verbo descreve
mal o que se viu.

Do lado do live, durante o boot de 22/08, havia **duas** entradas e a do ARCA não
estava entre elas: o slot `Boot0001` — o mesmo slot, o mesmo alvo — trazia
`UEFI OS`. Isso é substituição.

Do lado do Windows, na leitura de depois, a entrada do ARCA `{f4057bd0}`
**sobreviveu intacta**, em minúsculas, com o `BCDOBJECT` presente — e ao lado
dela havia uma `{687478f2}` `UEFI OS` recém-nascida. Isso é criação de uma
segunda entrada, e a original não perdeu um byte.

As duas leituras são do mesmo arquivo de NVRAM em momentos diferentes, e **o que
aconteceu entre elas não está medido**. É a armadilha que o §11 do PRD já nomeia:
uma leitura feita no Windows descreve o firmware **como ele ficou**, e o boot é
justamente o que mexe.

## O que P-19 ganha, mesmo sem fechar

**A consequência operacional está medida três vezes, de dentro do boot, no mesmo
dia.** Em 24/08, com os dois gatilhos — `bootsequence` no backup, `displayorder`
no boot da fase 1 —, a entrada `{f4057bd3}` está lá, intacta, e `BootCurrent:
0001` diz que a máquina bootou por ela. Nenhuma `UEFI OS` nasceu, e a leitura do
`bcdedit` depois da restauração confirma o mesmo pelo lado do Windows.

**E o que P-19 protegia já está protegido por outra coisa.** C-13 devolve o
`{bootmgr}` ao topo ao colher, com ou sem entrada reescrita; C-9 manda remover o
SSD antes de religar, que fecha a janela entre o fim da receita e a colheita. O
`arca status` de depois da restauração confirma: `dispositivo em 2o de 2 ·
Windows Boot Manager vem antes`.

## O que fica aberto, e por que não vale outro reinício

P-19 continua aberta com o enunciado corrigido: **em que condição este firmware
cria uma entrada `UEFI OS` no lugar da que o `bcdedit` escreveu?**

Não vale outro reinício. As variáveis que o experimento sabia controlar estão
controladas, e o resultado mudou assim mesmo — um quarto boot em 24/08 daria a
mesma resposta que os três primeiros deram, porque é a data que separa, e a data
não é uma variável que se possa pôr no roteiro. O que responderia é saber o que
mudou na placa entre 22 e 24 de agosto, e isso não é uma medição: é um registro
que não existe.

**Nenhuma tela do ARCA afirma nada que dependa da resposta**, e é por isso que ele
pode conviver com ela aberta — o critério do §7 continua satisfeito.

## Consequências

- **A hipótese de P-19 sai do PRD.** O §3.5 e a tabela de pendências passam a
  registrar a refutação, e não o candidato.
- **O experimento anotado no ADR-0011 fica cumprido, e com o desfecho invertido.**
  Lá se escreveu que a leitura de dentro do live era o que fecharia P-19, e que
  ela vinha de graça em cada imagem. As duas coisas se confirmaram; o que não se
  confirmou foi a resposta que se esperava dela.
- **As seis leituras ficam tabeladas neste ADR.** Elas estavam espalhadas por
  cinco arquivos e nunca tinham sido postas lado a lado — e é só lado a lado que
  se vê que nenhuma variável conhecida separa os dois casos dos quatro.
- **A armadilha do §11 ganha um caso a mais**, e é o mais claro que este projeto
  tem: em 22/08, a leitura de dentro do boot e a leitura de depois discordam sobre
  a existência da entrada do ARCA, e **as duas estão certas**.

> **O braço de brinde é a lição de método.** Ele não custou nada — o Clonezilla
> grava o `efi-nvram.dat` em toda imagem, e o passo 2b só o copiou para
> `recursos/capturas/`. Foi ele que refutou. O braço que se desenhou de propósito,
> que custou um reinício e uma sessão dentro do live, confirmou o esperado e
> sozinho não teria decidido nada.
>
> Quando um experimento tem um braço que vem de graça, ele **é** o experimento —
> e o caro serve para tirar a ressalva do barato.
