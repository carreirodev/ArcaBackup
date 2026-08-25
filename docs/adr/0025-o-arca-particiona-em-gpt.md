# O ARCA particiona em GPT, e o marco em hardware que o ADR-0014 pediu aconteceu

**Supersede o [ADR-0014](0014-o-arca-particiona-o-dispositivo.md) na parte que
escolhia MBR.** Decidido em 25/08/2026. O resto do ADR-0014 — P1 revisado, as
sete defesas, o que continua fora — continua valendo inteiro.

## O que o ADR-0014 pediu, com essas palavras

> *"A tentação a resistir é 'modernizar para GPT+ESP' no caminho. Seria trocar
> um esquema medido por um suposto, num lugar onde o modo de falha é um
> dispositivo que não boota **e que só se descobre depois de o Windows já ter
> sido apagado**. Se a mudança for desejável, ela é uma decisão própria, com o
> seu próprio marco em hardware."*

O argumento estava certo na primeira metade e **errado na segunda**. Trocar um
esquema medido por um suposto seria mesmo indefensável — e é por isso que este
ADR não chega por argumento, chega por medição. Mas a premissa de que a falha
"só se descobre depois de o Windows já ter sido apagado" é falsa: dá para montar
um dispositivo GPT num disco descartável, bootá-lo, ver o menu do Clonezilla
subir e ler a NVRAM de dentro dele — com o dispositivo de produção desconectado
e o Windows intacto o tempo todo.

Foi o que se fez. O roteiro está em `PRD/marco-em-hardware-gpt-2026-08-25.md`, e
a medição em `recursos/capturas/medicao-gpt-2026-08-25.txt` e
`recursos/capturas/efibootmgr-gpt-2026-08-25.txt`.

## O que decide: o dispositivo GPT bootou

Em 25/08/2026, num KGSSE100 256 de 238,5 GB, com a ARCABOOT em FAT32 marcada
como *Basic data partition* e apontada por `partition=E:` e
`\EFI\boot\bootx64.efi`. O menu do Clonezilla subiu. Sem tela preta, sem erro de
firmware, sem volta direta para o Windows.

E o device path foi lido **de dentro do boot**, pelo `efibootmgr`, que é a mesma
qualidade de evidência das seis leituras de NVRAM do
[ADR-0023](0023-o-bootsequence-nao-e-o-gatilho-da-reescrita.md):

```text
GPT:  HD(2,GPT,9c86b84a-596f-47e6-b92a-cd5b84b4a1fe,0x1d9d3000,0x320000)/\EFI\BOOT\BOOTX64.EFI
MBR:  HD(2,MBR,0x4049dea9,0x1d9d2000,0x320000)
```

O número da partição continua **2**; `MBR` vira `GPT`; a assinatura do **disco**
dá lugar ao PARTUUID da **partição**, que o `blkid` confirma ser o da ARCABOOT;
e o tamanho `0x320000` é idêntico — 3 276 800 setores, os 1600 MiB fixos. O
offset difere, mas os discos são diferentes: essa linha não compara.

**E bootou com o Windows à frente da ordem permanente.** `BootCurrent: 0001` com
`BootOrder: 0000,0001` — os mesmos dois números que `armar.rs:441` registra para
o marco em MBR. C-5 não custa nada em GPT, como não custava em MBR.

## Os três ganhos, e agora eles são o argumento inteiro

1. **O limite de 2 TiB some.** O MBR endereça 2³² setores de 512 bytes. Hoje o
   dispositivo tem 238 GB e a questão é teórica; num disco de 4 TB ela é o
   comando recusando ou, pior, particionando errado em silêncio. Não há defesa
   nem menção a esse limite em lugar nenhum do código — ver *Consequências*.
2. **A tabela ganha cópia secundária com CRC32.** O MBR tem 64 bytes num setor
   só, sem soma de verificação. A GPT tem a tabela primária no começo e uma
   cópia no fim, as duas com CRC32.
3. **O esquema deixa de ser legado.** Não é argumento sozinho, e não foi tratado
   como tal: ele só entra depois de os dois primeiros e o boot medido.

## As três perguntas que o marco existia para responder

### 1. O device path é qual

Respondida acima. **O GUID é o da partição, e não o do disco** — é a diferença
que mais importa para quem for ler esse caminho um dia.

### 2. Houve MSR, e o que se faz com ela

Houve, **nos dois dispositivos testados**, com os três mesmos números:
`GptType {e3c9e316-0b5c-4db8-817d-f92df00215ae}`, offset 17 408, 16 759 808
bytes. Em MBR o `Initialize-Disk` deixa o disco vazio; em GPT ele cria sozinho
uma *Microsoft Reserved*.

Ela não serve para nada num dispositivo de dados, e deixá-la em pé teria três
efeitos, todos ruins: a ARCAVAULT nasceria partição **2** e a ARCABOOT **3**; o
device path viraria `HD(3,GPT,…)`; e a releitura, que confere a ordem das duas
partições no disco, passaria a ver três.

**O `arca prepare` a remove sempre, e não "se houver".** Duas medições
independentes bastam para tratar a criação como comportamento do
`Initialize-Disk`, e não como acidente de dispositivo. Um `prepare` que só
removesse "se houver" teria o mesmo efeito prático e uma condicional a mais para
alguém desconfiar depois.

### 3. O `GptType` sai do `New-Partition` ou do `Format-Volume`

**Do `New-Partition`.** As duas nascem `{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}`
— *Basic data partition* — e o `Format-Volume` **não encosta nele**. Medido nos
dois dispositivos.

É o contrário do MBR, onde as duas nascem com `MbrType 6` e só chegam a 7 e a 12
depois de formatar — o achado que o ADR-0014 registrou e que a releitura de
`conferir_o_que_saiu` existia em parte para pegar.

## O achado que ninguém tinha pensado em perguntar, e é o que mais muda código

**Em GPT o tipo não distingue as duas partições.**

Em MBR, `7` (IFS) e `12` (FAT32 LBA) separavam a ARCAVAULT da ARCABOOT, e as
constantes `TIPO_MBR_IFS` e `TIPO_MBR_FAT32_LBA` de `preparacao.rs` viviam
disso. Em GPT as duas têm **o mesmo** `GptType`, e o `MbrType` sai **vazio** —
não zero, ausente.

Isso não é uma peculiaridade do PowerShell: de dentro do live, o `lsblk` dá
`PARTTYPE ebd0a0a2-…` para as duas, o `parted` chama as duas de *Basic data
partition* com flag `msftdata`, e o `gdisk` dá código `0700` para as duas. É a
tabela de partição.

**A releitura não perde a conferência; perde o critério.** Ela continua podendo
afirmar que o tipo é o de dados básicos — o que descarta uma ESP, uma MSR ou
qualquer coisa que o Windows tivesse criado por conta —, e passa a distinguir
uma partição da outra pelo que já conferia de qualquer jeito: **o rótulo, o
sistema de arquivos e a ordem no disco**.

Vale dizer o que se perde, porque não é nada: o `MbrType` nunca foi a defesa
contra trocar as duas de lugar — o `ARCAVAULT` vir antes do `ARCABOOT` já era
conferido pelos offsets, e o rótulo já era conferido campo a campo. O que o
`MbrType` acrescentava era um segundo testemunho do sistema de arquivos, e o
`FileSystem` continua ali.

## A Variante B, e por que não a ESP canônica

O roteiro tinha duas variantes, e a escolhida deliberadamente **não** marca a
ARCABOOT como *EFI System* (`{c12a7328-…}`):

| | **Variante B — FAT32 Basic Data** | Variante A — ESP de verdade |
|---|---|---|
| `GptType` da ARCABOOT | `{ebd0a0a2-…}` | `{c12a7328-…}` |
| Letra de unidade | **mantém** | Windows esconde a ESP |
| `bcdedit` | `device partition=R:` | precisa de `\Device\HarddiskVolumeN` |
| Instalar o Clonezilla | copiar para `R:\` | precisa de `mountvol` antes |
| Superfície de mudança | tabela de partição, e só | tabela + `bcdedit` + instalação + releitura |

A Variante B entrega os três ganhos sem tocar em mais nada, **e bootou**. A
Variante A acrescentaria o tipo ESP canônico, que rende em firmwares muito
estritos, ao custo de três superfícies novas. Ela fica registrada como o
primeiro lugar a olhar se algum dia um firmware recusar — e não como dívida.

## Três coisas que o marco mediu de raspão, e que não são desta decisão

Estão aqui porque foram medidas na mesma noite e o registro é o lugar delas. Não
mudam código neste ADR.

**O `bcdedit` recusa em silêncio o Kingston DataTraveler Max.** O
`/set <id> device partition=E:` responde *"A operação foi concluída com êxito"*,
código 0, e a releitura traz o device antigo — por letra e por caminho de
dispositivo. Quatro alvos isolaram a causa: `partition=C:` (NVMe, GPT) pega;
`partition=D:` e `partition=E:` (DataTraveler, GPT) não; `partition=F:`
(KGSSE100, MBR) pega; e `partition=E:` no **mesmo KGSSE100 convertido para GPT**
pega. Não é o GPT e não é o USB. É o C-6 que `prepare.rs:678` descrevia sem ter
um caso, e a releitura de C-3 é o que separa esse silêncio de um `arca prepare`
que diria ter preparado um dispositivo que não boota.

**O identificador que o `bcdedit /enum firmware` devolve não é identidade.** O
`{31cc955f-a0ae-11f1-8a54-806e6f6e6963}` era `UEFI:CD/DVD Drive` sem `device`
antes de um boot e `ARCA GPT TESTE` com `device partition=E:` depois dele. Ele
nomeia o *slot* `Boot####` da NVRAM, e não a entrada que está nele. Dentro de
uma mesma sessão, sem reinício, nada mudou nas seis releituras do roteiro; entre
boots, mudou. A distinção não foi medida de propósito, e é uma pergunta aberta
para todo lugar que guarda um identificador de firmware e o usa depois.

**E o `displayorder` do `bcdedit` não previu o comportamento do firmware.** Ele
trazia a entrada de teste em primeiro, na frente do `{bootmgr}`, com `timeout 1`;
mesmo assim a máquina bootou no Windows com o dispositivo conectado. O
`efibootmgr`, lido de dentro do boot, media `BootOrder: 0000,0001`. Encosta no
[ADR-0020](0020-o-bcdedit-enum-firmware-le-a-nvram.md), e fica como pergunta.

## O que muda no código

| Onde | O que muda |
|---|---|
| `adaptadores/windows/particionador.rs` | `-PartitionStyle MBR` → `GPT`, e a remoção da MSR logo depois do `Initialize-Disk` |
| `portas/particionador.rs` | `ParticaoFeita::tipo_mbr: u32` → `tipo_gpt: String` |
| `preparacao.rs` | `TIPO_MBR_IFS`/`TIPO_MBR_FAT32_LBA` saem; entra `TIPO_GPT_DADOS_BASICOS`, **um** para as duas |
| `preparacao.rs::conferir_o_que_saiu` | confere o tipo comum, e distingue as duas por rótulo, sistema de arquivos e ordem |
| `comandos/prepare.rs` | o parágrafo *"A estrutura e MBR, e nao GPT"* sai da tela |
| `duplos.rs`, `tests/e10_*` | os fixtures e os asserts passam a citar a captura nova |

## O que **não** muda, e vale dizer

- **As sete defesas de PR-5.** O ADR-0014 as estabeleceu e elas não dependem do
  esquema.
- **A ordem e os tamanhos.** ARCAVAULT primeiro, ARCABOOT de 1600 MiB no fim,
  unidade de alocação 4096 nas duas. Medido igual nos dois esquemas.
- **`IsActive` sai `False` nas duas**, e continua sendo conferido. O boot é UEFI
  puro, e `particionador.rs` continua certo em não passar `-IsActive`.
- **A conta do tamanho sai do `LargestFreeExtent` lido na hora**, e não de
  constante. Isso já era assim, e em GPT importa mais: a cópia secundária da
  tabela ocupa o fim do disco. Num disco de 256 060 514 304 bytes, o extent
  livre depois de remover a MSR é 256 059 113 472 — a GPT cobra 1 400 832.

## Consequências

- O ADR-0014 fica válido menos na escolha do esquema, e ganha um link para cá.
- **O limite de 2 TiB deixa de existir**, e vale registrar que deixou: era a
  pendência que este marco abriria de qualquer jeito, e o único desfecho em que
  ela viraria código era o de continuar em MBR.
- A releitura de `conferir_o_que_saiu` fica **mais** rigorosa, e não menos: ela
  passa a recusar uma terceira partição que antes teria virado uma mensagem
  confusa sobre letra faltando.
- Fica aberto, e nomeado: **se o identificador de firmware é estável dentro de
  uma sessão.** Todo lugar do código que guarda um e o usa depois depende disso,
  e não foi medido de propósito.
