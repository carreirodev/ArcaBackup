# O ARCA particiona o dispositivo, e P1 fica só com o que ele protegia

**Supersede o princípio P1** na parte que proibia particionar. Decidido em
23/08/2026.

## A decisão, e o argumento que a sustenta

> *"Na realidade eu quero que ele particione sozinho, e renomeie as duas
> partições como devem ser — o máximo que você deve fazer é solicitar uma
> confirmação. Esse app é justamente uma automatização, então não entendo por
> que não fazer isso sozinho."*

O argumento é bom e o registro precisa dizer por quê, em vez de anotar a
decisão como preferência.

**P1 dizia que o ARCA não executa a operação mais destrutiva do fluxo, e isso
nunca foi verdade.** `arca restore` apaga 465 GB do disco de sistema desta
máquina, e é a razão de o projeto existir. Particionar um pen drive vazio de
4 GB não chega perto. O princípio estava classificando por *categoria da
operação* — "particionar é perigoso" — quando o que separa uma coisa da outra
é **o que se perde quando dá errado**, e por esse critério a ordem é a
inversa.

E o app é uma automação. Automatizar o backup e a restauração, e parar na
preparação para mandar o usuário ao Gerenciamento de Disco, é uma inconsistência
que só se explicava pelo princípio — e o princípio não se sustentava.

## A objeção que foi levantada, e o que ficou dela

A objeção foi: **o perigo não está em particionar, está em acertar em qual
disco.** Ela continua verdadeira, e não some com esta decisão — ela vira a
lista de defesas abaixo.

O precedente que a sustenta é concreto: a revisão da E9 achou que R-8, a recusa
que impede o dispositivo de ser destino da restauração, tinha um contorno por
acidente de modelo. Com um segundo disco do mesmo modelo do dispositivo, a
receita sairia `restoredisk <imagem> sda`, e o `sda` era o próprio dispositivo.
**Identificar disco é onde este código já errou**, e `arca prepare` roda antes
de existirem os rótulos — logo sem B-1, sem S-3 e sem o que C-10 recusaria.

A resposta não é não fazer. É fazer com as defesas que o `arca restore` levou
uma etapa inteira para ganhar.

## E há original, o que muda a natureza do trabalho

Esta foi a descoberta que mais mudou a decisão. Medido em 23/08/2026 e
preservado em
`recursos/capturas/estrutura-de-particoes-do-dispositivo-2026-08-23.txt`:

```text
Get-Disk        KGSSE100 256 · USB · PartitionStyle MBR · 256.060.514.304 bytes
particao 1      E: · MbrType 7  (IFS/NTFS)      · offset 1.048.576      · 254.379.294.720
particao 2      R: · MbrType 12 (FAT32 LBA)     · offset 254.380.343.296 · 1.677.721.600
volumes         E: ARCAVAULT NTFS 4096 · R: ARCABOOT FAT32 4096
IsActive        nenhuma das duas
MediaType       External hard disk media
```

**O dispositivo é MBR, e boota por UEFI assim mesmo.** Não é o esquema que um
manual moderno recomendaria — o canônico seria GPT com uma ESP —, e é o que
está comprovadamente bootando nesta máquina desde 19/08: o `bcdedit` aponta
`partition=R:` para `\EFI\boot\bootx64.efi`, e o `efi-nvram.dat` de dentro das
imagens registra a máquina tendo bootado por ali. `IsActive` em nenhuma das
duas confirma que o boot é UEFI puro, e não BIOS.

Isso tira a objeção mais forte que restava — *"particionar seria código novo
sem original"*. **Há original, e ele está medido.** `arca prepare` transcreve
uma estrutura que boota, em vez de inventar uma que deveria bootar, e o
precedente é o [ADR-0004](0004-a-receita-transcreve-o-que-rodou.md): a receita
transcreve o que rodou.

**A tentação a resistir é "modernizar para GPT+ESP" no caminho.** Seria trocar
um esquema medido por um suposto, num lugar onde o modo de falha é um
dispositivo que não boota **e que só se descobre depois de o Windows já ter
sido apagado** — porque é justamente aí que alguém precisa dele. Se a mudança
for desejável, ela é uma decisão própria, com o seu próprio marco em hardware.

## O que P1 vira

P1 deixa de proibir e passa a dizer **quando o ARCA pode destruir**:

> **P1 (revisado).** O ARCA destrói dados quando o usuário nomeou o alvo e
> confirmou por escrito, e nunca por dedução. O que ele não faz é agir sobre um
> disco que ele mesmo escolheu.

Isso já descrevia o `arca restore` — R-3, R-7, R-8 e S-2 são exatamente isso — e
passa a descrever o `arca prepare` também. A parte que muda é só qual operação
está na lista.

## As defesas, e nenhuma é opcional

O `arca prepare` só escreve num disco que passe por **todas**:

1. **`MediaType` removível ou externo.** O WMI responde
   `External hard disk media` para este dispositivo e
   `Fixed hard disk media` para o NVMe — medido na E6, e é o sinal que C-6 já
   usa. Disco fixo é recusa dura, sem `--force` nenhum.
2. **Não é o disco do `%SystemDrive%`**, nem o `IsSystem`, nem o `IsBoot`. A E6
   já acha o disco do sistema sem supor que ele é o 0, e o motivo está lá: numa
   máquina em que o dispositivo fosse o disco 0, supor o índice apagaria o
   disco errado.
3. **O disco é escolhido pelo usuário, e nunca deduzido.** `--dispositivo
   <índice>`, no molde do `--destino <índice>` que a E9 construiu. Havendo um
   candidato só, ele ainda é **mostrado e confirmado**, nunca assumido.
4. **A tela mostra o que será destruído, antes.** Índice, modelo, `MediaType`,
   tamanho, e **as partições que existem hoje com rótulo, sistema de arquivos e
   tamanho**. Quem vai perder dados tem de poder reconhecê-los na tela.
5. **Confirmação digitada** (S-2), nunca `s`. `src/confirmacao.rs` existe desde
   a E9.
6. **`--dry-run` de primeira classe**, como em todo comando que arma (decisão 7
   do plano). Aqui ele vale mais do que em qualquer outro: é a única forma de
   ver o plano de partições sem executá-lo.
7. **Releitura depois de escrever**, no espírito de C-3: particionou, releia o
   disco e confira que saiu o que se pediu. O `Format-Volume` e o
   `New-Partition` não são o `bcdedit`, mas a regra de não acreditar em código
   de saída já provou o seu valor três vezes neste projeto.

## O que continua fora, e agora por um motivo escrito

- **Particionar disco fixo.** Não é conservadorismo: é que o modo de falha
  apaga o Windows de alguém, e nenhuma confirmação digitada compra isso.
- **Escolher o disco sozinho.** Um candidato só continua sendo mostrado e
  confirmado. A dedução é o que P1 revisado proíbe.

## Consequências

- §7.1 do PRD deixa de se chamar *"O ARCA não cria partições"* e é reescrito.
- §2 perde a linha *"❌ Criador de partições"*.
- **PR-5** nasce: particionar e rotular, com as sete defesas.
- **PR-4** continua valendo e muda de sujeito: as instruções deixam de ser
  "como particionar" e passam a ser "o que vai acontecer com este disco".
- O marco em hardware da E10 continua exigindo um segundo dispositivo — e agora
  ele é destruído de propósito, que é o teste.
