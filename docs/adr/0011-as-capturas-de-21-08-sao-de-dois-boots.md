# As capturas de 21/08 são de dois boots, e P-19 estreita em vez de fechar

O [ADR-0009](0009-a-ordem-permanente-muda-no-ciclo-de-boot.md) deixou P-19
aberta — *"o firmware reescreve a entrada em todo boot pelo dispositivo, ou só
quando ela foi consumida por `bootsequence`?"* — e disse onde estaria a
resposta: no `efi-nvram.dat` que o Clonezilla grava dentro de cada imagem.
*"Um segundo backup responde, e a leitura que responde já está sendo colhida,
de graça, desde antes de alguém saber para que serviria."*

O segundo backup aconteceu em 22/08. Abrindo as duas imagens lado a lado, o
`efi-nvram.dat` de `2026-08-21_WindowsCompleto` e o de `2026-08-22_Apps` são
**byte-idênticos** — mesmo SHA256, `44345e21…ac114a83`. A comparação que
deveria responder não distingue nada.

O que respondeu foram outras capturas, e elas estavam no dispositivo desde
20/08.

## As dez leituras, com hora e com a forma da entrada

Todas de dentro do live, pelo `efibootmgr -v`, com a hora do relógio do
Clonezilla — três horas adiantado, permanentemente (P-7). A última coluna é a
que ninguém tinha olhado:

| Quando | Arquivo | `BootCurrent` | `BootOrder` | A entrada do dispositivo |
|---|---|---|---|---|
| 20/08 02:33 | `ARCA-LOGS/nvram-original.txt` | `0001` | `0000,0001` | `Clonezilla`, `\EFI\boot\bootx64.efi`, **com `BCDOBJECT`** |
| 20/08 02:35 | `ARCA-TESTE-03/efi-nvram.dat` | `0001` | `0000,0001` | idêntica — mesmo SHA256 |
| 20/08 02:52 e 03:08 | `ARCA-LOGS/R1/nvram-antes` e `-depois` | `0001` | `0000,0001` | idênticas |
| 20/08 12:13 e 12:30 | `ARCA-LOGS/R2/nvram-antes` e `-depois` | **`0003`** | **`0003,0000`** | `Clonezilla`, **com `BCDOBJECT`** |
| 21/08 12:51 | `2026-08-21_WindowsCompleto/efi-nvram.dat` | `0001` | **`0000,0001`** | **`UEFI OS`**, `\EFI\BOOT\BOOTX64.EFI`, `data: 00 00 42 4f` |
| 21/08 14:28 e 14:46 | `ARCA-LOGS/2026-08-21_WindowsCompleto/nvram-antes` e `-depois` | `0001` | **`0001,0000`** | **`ARCA`**, `\EFI\boot\bootx64.efi`, **com `BCDOBJECT`** |
| 22/08 ~20:57 | `2026-08-22_Apps/efi-nvram.dat` | `0001` | `0000,0001` | **`UEFI OS`** — byte-idêntica à de 21/08 12:51 |

## Primeiro achado: as duas leituras de 21/08 são de operações diferentes

O §3.1 do PRD e a `PROVENIENCIA.md` trazem uma linha só para 21/08 —
*"`2026-08-21_WindowsCompleto/nvram-antes.txt` e `-depois.txt`, `0001,0000`"* —
e a usam para explicar o **backup** daquele dia. Aqueles dois arquivos não são
do backup.

Eles estão em `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\`, e não na pasta da
imagem. Ao lado deles está o `arca-fim.txt` de 25 bytes com `ARCA_RESTORE=OK` —
o mesmo que a E5 usou para escrever a linha "sem selo" do §5.5. Os `mtime`
datam a coisa: o `savedisk` gravou de 12:45:39 a 12:54:08 e o `arca-check.log`
saiu às 12:58:50; o `nvram-antes.txt` é de **14:28:36** e o `nvram-depois.txt`
e o `arca-fim.txt` são de **14:46:51**, os dois no mesmo segundo.

É a **restauração** de 21/08 — a que o §3.4 do PRD chama de validada, e cujos
`nvram-antes` e `-depois` byte-idênticos são justamente a evidência de que
`-iefi` não toca na NVRAM.

A NVRAM do boot do **backup** de 21/08 é outra: é o `efi-nvram.dat` de dentro
da imagem, escrito às 12:51:25, no meio da gravação. E ele diz `0000,0001` —
**Windows à frente**, exatamente como em 22/08.

Isto é a armadilha do §11 outra vez, e desta vez ela não separou duas
ferramentas nem dois dias: separou **dois boots do mesmo dia**, com uma hora e
meia entre eles e um boot no Windows no meio.

### O que isso desfaz, e o que não desfaz

**Desfaz** a frase do §3.1: *"No backup de 21/08 o dispositivo estava em
primeiro, e é por isso que aquele backup não provava nada."* Ele estava em
segundo, e a leitura que dizia o contrário é da restauração.

**Não desfaz P-18.** O backup de 21/08 continua não provando boot único, e o de
22/08 continua provando — mas o que os separa é outra coisa. Em 21/08 não
existia ARCA: o `git log` deste repositório começa em 22/08 às 11:47, e nada
podia ter escrito `bootsequence`. Com o Windows à frente da ordem, aquele boot
pelo dispositivo só pode ter vindo de alguém — F12, ou um `BootNext` posto à
mão. Em 22/08 havia `bootsequence` gravado pelo ARCA às 20:53:48 e **ninguém
tocou na máquina**.

A diferença entre as duas medições não é a ordem de boot; é quem apertou o
botão. O argumento fica mais forte, e não mais fraco: um `BootCurrent` fora da
frente da `BootOrder` é explicado por F12 tão bem quanto por `bootsequence`, e
o que o marco de 22/08 tem é a ausência de qualquer mão.

## Segundo achado: a primeira metade de P-19 está descartada

**O firmware não reescreve a entrada em todo boot pelo dispositivo.**

Em 20/08 houve pelo menos três boots pelo dispositivo — a sessão das 02:33 às
03:08 e a restauração R2, das 12:13 às 12:30 —, e em todas as capturas a
entrada permanece na forma que o `bcdedit` escreve: descrição `Clonezilla`,
caminho `\EFI\boot\bootx64.efi` em minúsculas, e o `BCDOBJECT={f4057bd0-…}`
dentro do `data:`. Dois desses boots aconteceram com a entrada **fora da frente
da ordem** (`BootCurrent: 0001` com `BootOrder: 0000,0001`), que é a mesma
configuração do marco de 22/08.

Se bootar pelo dispositivo bastasse, aquelas entradas teriam voltado como
`UEFI OS`. Não voltaram.

## O que continua aberto, e por quê

A segunda metade — *"só quando ela foi consumida por `bootsequence`"* — **não
fecha**, e a razão é de régua outra vez.

Uma captura feita durante o boot N mostra a NVRAM **como ela está**, e não qual
boot a deixou assim. A forma canônica do firmware aparece pela primeira vez em
21/08 12:51, num boot que não podia ter `bootsequence` do ARCA — o que
*parece* fechar pelo lado do "não é só o `bootsequence`". Mas entre a última
captura que mostra a forma antiga (R2, 20/08 12:30) e essa há um intervalo em
que aconteceram, pelo menos, um boot no Windows e a renomeação manual de
`Clonezilla` para `ARCA` — que está evidenciada pela captura de 21/08 14:28, e
que não tem data. Se ela veio depois de 12:51, o `UEFI OS` daquela captura foi
escrito por um boot anterior, e a pergunta continua de pé.

Não há captura do `bcdedit` entre 20/08 05:14 e 22/08 para datar a renomeação, e
inventar uma ordem para os eventos desse intervalo seria exatamente o que este
projeto já pagou cinco vezes para não fazer.

**O que responde é um backup disparado por F12**, com a entrada na forma do
`bcdedit` imediatamente antes — as duas leituras no mesmo dia, e a segunda de
dentro do live. Fica anotado como o experimento que fecha P-19, e ele não é
urgente: a consequência operacional que importa — a entrada volta para a ordem
permanente depois de um boot pelo dispositivo — está medida e não depende disso.

## Terceiro achado: o `efi-nvram.dat` não é uma sonda tão boa quanto parecia

O ADR-0009 tratou o `efi-nvram.dat` como a leitura que responderia P-19, porque
ela é escrita **durante** o boot que se quer explicar. Ela é isso, e é por isso
que fecha P-18. Mas para P-19 ela tem um limite que não estava dito: **ela
mostra o estado, e não a transição.**

Duas capturas byte-idênticas de dois boots diferentes não provam que nada mudou
entre eles — provam que o estado no instante da leitura era o mesmo. Foi o que
aconteceu: entre 21/08 12:51 e 22/08 20:57 a entrada virou `ARCA`, voltou a
`UEFI OS`, apareceu uma terceira entrada, e o `displayorder` mudou três vezes.
As duas leituras não registram nada disso.

Para transição é preciso um par — antes e depois, do mesmo evento —, e o
dispositivo tem exatamente dois pares assim: os `nvram-antes`/`-depois` das
restaurações R1, R2 e de 21/08. Eles foram escritos à mão, dos dois lados de
uma operação, e é por isso que respondem sobre `-iefi` (§3.4) enquanto o
`efi-nvram.dat` não responde sobre o ciclo de boot.

## Consequências

O §3.1 do PRD ganha as duas linhas de 21/08 separadas, com a operação de cada
uma, e perde a frase que atribuía ao backup daquele dia uma ordem de boot que
era da restauração. A `PROVENIENCIA.md` ganha a mesma correção, e as quatro
capturas que a sustentam entram em `recursos/capturas/`.

P-19 sai do §3.5 **estreitada**: a primeira metade está descartada por medição,
e o que fecha a segunda é um experimento nomeado.

E o §11 ganha uma variante da armadilha que já tem: **não basta datar cada
captura — é preciso saber de que operação ela é.** Duas leituras do mesmo dia,
na pasta que leva o mesmo nome de imagem, eram de dois boots com uma hora e
meia de distância; e quem as juntou foi o nome da pasta, que a E3 escolheu por
outro motivo inteiramente.
