# A restauração devolve a ordem permanente que está dentro da imagem

O [ADR-0009](0009-a-ordem-permanente-muda-no-ciclo-de-boot.md) mediu que o
ciclo de boot **põe** o dispositivo na ordem permanente, e decidiu que o ARCA
avisa em vez de consertar. O marco da E9 mediu o outro sentido, e ninguém o
tinha previsto: **uma restauração tira o dispositivo da ordem** — e não porque
alguém a arrumou, mas porque a ordem permanente estava dentro da imagem, junto
com o disco.

## O que foi medido

Quatro leituras do `bcdedit /enum firmware` desta máquina, todas preservadas em
`recursos/capturas/`:

| Quando | Arquivo | `displayorder` | SHA256 |
|---|---|---|---|
| 22/08 manhã | `bcdedit-enum-firmware-pt.txt` | `{bootmgr}` | `d837093d…f204f15e` |
| 22/08 21:17 · depois do backup | `…-2026-08-22-pos-marco.txt` | `{f4057bd0}`, `{bootmgr}`, `{687478f2}` | `3cd147f5…6ab02e56` |
| 23/08 · antes da restauração | `…-2026-08-23-antes-da-restauracao.txt` | `{f4057bd0}`, `{687478f2}`, `{bootmgr}` | `7bdae900…dfa87800` |
| 23/08 · depois de restaurar e religar | `…-2026-08-23-pos-restauracao.txt` | `{bootmgr}` | **`d837093d…f204f15e`** |

A última linha é o achado, e ela não é "parecida": a leitura de depois da
restauração é **byte a byte** a de 22/08 de manhã. Mesmo SHA256, e idênticas
linha a linha. `tests/e9_restaurar_o_disco.rs` as fixa.

A entrada `{687478f2}` `UEFI OS` não saiu da ordem — ela **sumiu inteira**. É a
entrada que o firmware criou durante o boot do backup de 22/08, e foi por ela
que a máquina bootou naquela noite (§3.1 e o `nvram-live-2026-08-22.txt`).

## O que não explica

**Não foi o ARCA.** C-5 proíbe escrever na ordem permanente, e o armar e o
desarme releem o firmware depois de escrever (C-3): uma escrita dessas teria
falhado alto, e a única que houve — o `bootsequence` — foi apagada na colheita.

**Não foi o `ocs-sr`.** Quem responde isso é o §3.4, e pelo lado certo: o par
`nvram-antes`/`nvram-depois` de 21/08, lido **de dentro do mesmo boot** e com
o Clonezilla correndo entre as duas leituras, é byte-idêntico. Aquele par é
estreito para a pergunta deste ADR e é exatamente do tamanho certo para esta.

> **E o log desta operação não serve como segunda evidência disto, embora
> pareça servir.** O `arca-restore-2026-08-22_Apps.log` não tem uma linha de
> `efibootmgr`, e eu quase escrevi isso aqui como prova. **Ele começa no meio:**
> traz uma única passagem do Partclone — a da `nvme0n1p4`, de 1,1 GB, a última
> das quatro — e não traz nem o `Starting /usr/sbin/ocs-sr` nem a restauração da
> `nvme0n1p1`, `p2` e `p3`. Ausência num log truncado não é ausência.
>
> Seria o mesmo erro que este ADR descreve, cometido dentro dele: usar uma
> evidência que não cobre o intervalo da pergunta. Ver a nota sobre o log logo
> abaixo.

## Uma nota sobre o log, porque ela vale além deste ADR

O `arca-restore.log` que a receita redireciona (D2) **não é o log inteiro da
operação**. Medido no do marco, 16.600 bytes:

- Uma única passagem do Partclone, a da `nvme0n1p4` — 1,1 GB, restaurada em
  8,64 s. As da `p1` (a ESP), `p2` e `p3` (o `C:`, a maior das quatro) não estão
  lá.
- Não há `Starting /usr/sbin/ocs-sr`, e há `Ending /usr/sbin/ocs-sr at
  2026-08-23 11:31:55 UTC`. O fim está inteiro; o começo, não.
- O arquivo abre com sequências de limpeza de tela (`ESC[H ESC[J`), que é como
  cada passagem do Partclone começa.

A causa não está determinada, e não se determina relendo o arquivo. **A
consequência prática importa mais**: o §6.3 manda quem colheu uma restauração
procurar ali quando quiser saber o que aconteceu, e o que está ali pode não
cobrir a parte que falhou. Vale medir de novo na próxima restauração, e a
pergunta certa é se o corte é sempre no mesmo lugar.

## O que explica, e a datação sustenta

A restauração é `restoredisk`, e a imagem carrega as quatro partições — o
`ls` da pasta traz `nvme0n1p1.vfat-ptcl-img.zst.aa` ao lado das outras três, e a
`p1` é a **ESP**. O pós-processamento do log confirma que ela estava no job:
`ocs-tux-postprocess nvme0n1p1 nvme0n1p2 nvme0n1p3 nvme0n1p4`, com
`Skip /dev/nvme0n1p1 (vfat)`. A partição EFI foi reescrita, e com ela o
`\EFI\Microsoft\Boot\BCD`.

**O que o `bcdedit` mostra hoje é o que estava dentro da imagem.** A imagem foi
selada em 22/08 às 21:06, e o Windows estava desligado desde 20:53 — logo o
estado que ela carrega é o da manhã daquele dia, que é exatamente o arquivo da
primeira linha da tabela.

E a `{687478f2}` fecha o argumento pela data: ela nasceu **durante** o boot de
20:53–21:06, na NVRAM, com o Windows desligado. Nunca chegou ao BCD que a
imagem carrega. A captura das 21:17 já a mostra, porque a essa altura o Windows
tinha subido e a espelhado. Restaurar a apagou porque ela nunca esteve lá
dentro.

## O que isto corrige no §3.4, e a correção é de alcance e não de fato

O §3.4 afirma *"`-iefi` funciona — NVRAM byte-idêntica antes e depois"*. **Isso
continua verdade, e fala do `ocs-sr`**: as duas leituras daquele par são do
mesmo boot, separadas por dezoito minutos, e entre elas só correu o Clonezilla.

O par que a E9 acrescenta é outro. Ele atravessa o reinício, é lido do lado
Windows pelo `bcdedit`, e responde uma pergunta que aquele não alcançava: *o
que a máquina tem na ordem permanente depois de voltar?* A resposta é **o que a
imagem carregava**, e não o que havia antes de restaurar.

Vale o mesmo cuidado que este projeto já pagou cinco vezes: **conferir se a
evidência fala sobre a pergunta.** Uma leitura de dentro do live, durante a
operação, não responde nada sobre o que sobra depois de o Windows subir.

## A consequência operacional, e ela é boa

O ADR-0009 registrou a fricção: com o dispositivo à frente da ordem, ligar a
máquina com o SSD conectado boota nele — e isso é o que P-20 pede para
consertar na E10.

**Depois de uma restauração não há o que consertar.** A operação já devolveu a
ordem ao estado de dentro da imagem, e nesta máquina esse estado é o Windows
sozinho. O pedido de P-20 continua de pé, e o que muda é o alcance dele: ele é
sobre o **backup**, que é a operação que suja a ordem e não a limpa.

Isto não decide P-20 nem supersede o ADR-0009 — decidir aquilo continua sendo
da E10, e continua exigindo medir a forma do comando antes de escrever código.
O que este ADR entrega para lá é que **metade dos casos não precisa do
conserto**, e que a ordem tem um **terceiro** dono que ninguém tinha nomeado. O
ADR-0009 arbitrava entre o ARCA e o Windows; a imagem escreve sem perguntar a
nenhum dos dois, e um conserto que rodasse ao colher uma restauração estaria
discutindo com um estado gravado por cima segundos antes.

## O que fica aberto, e é P-22

**O `bcdedit /enum firmware` mostra a NVRAM do firmware, ou o BCD do disco?**
A pergunta nunca precisou de resposta até aqui, e agora precisa, porque as duas
possibilidades levam a mundos diferentes:

- **Se é a NVRAM**, ela acompanhou o BCD restaurado — provavelmente pelo
  Windows, que a reescreve ao subir —, a ordem está limpa de verdade, e ligar
  com o SSD conectado sobe o Windows.
- **Se é só o BCD**, a NVRAM pode continuar com as entradas do dispositivo à
  frente. A máquina continuaria bootando nele a cada reinício, **enquanto o
  `arca status` diria que está tudo bem** — porque a linha `Ordem de boot` que
  o ADR-0009 mandou acrescentar lê justamente o `bcdedit`.

O segundo caso é uma afirmação de segurança feita sobre uma leitura que não
fala da pergunta, que é o defeito que a revisão do marco da E8 já pegou uma vez
naquela mesma linha.

**O experimento que separa os dois custa um reinício e nenhum risco:** religar
com o SSD conectado, sem job armado e com o `grub.cfg` inerte, e ver onde a
máquina para. No Windows, a NVRAM acompanhou. No menu do Clonezilla, não
acompanhou — e o `arca status` está tranquilizando sobre um estado que não leu.
O grub inerte garante que o pior caso é um menu esperando alguém.

## Consequências

- `tests/e9_restaurar_o_disco.rs` fixa as duas leituras e a identidade com a
  captura da E2. Recapturar qualquer uma delas faz a suíte falar.
- O §3.4 do PRD passa a distinguir o que o par de 21/08 mede do que o par da E9
  mede.
- P-20 continua na E10, com o alcance corrigido.
- P-22 entra em §3.5 e §12, com o experimento nomeado.
