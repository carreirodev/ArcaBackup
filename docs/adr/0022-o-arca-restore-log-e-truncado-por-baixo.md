# O `arca-restore.log` é truncado por baixo, e o que sobra é a última passagem

P-23 nasceu no marco da E9 com uma observação e uma suspeita. A observação: o
`arca-restore.log` daquela restauração tinha 16.600 bytes e **começava no meio**
— uma passagem só do Partclone, a da última das quatro partições, e um `Ending
/usr/sbin/ocs-sr` sem o `Starting` correspondente. A suspeita: que o corte fosse
do ARCA, ou do `>` da receita.

Não era nem um nem outro. O log não começa no meio: **ele é escrito inteiro e
depois esvaziado por baixo**, e o que sobrevive é a última coisa que o Clonezilla
escreveu.

A resposta foi **prevista antes de medir** — o mecanismo e as cinco consequências
observáveis entraram no repositório em `0e83b3f`, às 19:48 de 24/08/2026, uma hora
e quarenta antes da restauração que as testaria.

## O que foi medido

Duas restaurações completas desta máquina, a mesma imagem de quatro partições, o
mesmo disco alvo `nvme0n1`:

| | 22/08 · `2026-08-22_Apps` | 24/08 · `2026-08-24_Ciclo` |
|---|---|---|
| tamanho | 16.600 bytes | 16.641 bytes |
| SHA256 | `e4cba0de…faffaa8e` | `1414fc4f…3090cc7e` |
| tela do Partclone | bytes 0 – 4.084 | bytes 0 – 4.084 |
| buraco de NULs | 4.085 – **12.890** (8.806 bytes) | 4.085 – **12.924** (8.840 bytes) |
| texto final | 3.709 bytes | 3.716 bytes |
| conteúdo real | 7.794 bytes | 7.801 bytes |
| inicializações de terminal | 1, no offset 0 | 1, no offset 0 |
| `Starting to restore image` | offset 2.583, `nvme0n1p4` | offset 2.583, `nvme0n1p4` |
| `Starting /usr/sbin/ocs-sr` | ausente | ausente |
| `Ending /usr/sbin/ocs-sr` | offset 16.546 | offset 16.587 |

**Mais da metade de cada arquivo é zero** — 53% nos dois.

E o dado que a previsão não pedia: **os primeiros 4.085 bytes dos dois logs são
byte a byte idênticos.** É a tela do Partclone da `nvme0n1p4`, a partição de
recuperação NTFS de 1,1 GB, desenhada num terminal de 24 linhas. Mesma partição,
mesma tela, mesmo número de bytes.

## O mecanismo, e ele não deixa sobra

Um buraco de NULs no meio de um arquivo só aparece de um jeito: alguém o
**encurtou** enquanto um descritor com offset alto continuava aberto. Escrever
naquele descritor depois disso reabre o arquivo até o offset antigo, e o
sistema de arquivos preenche o vão com zeros.

Aplicado a esta receita, em três tempos:

1. o `>` da receita abre o `arca-restore.log` e o `ocs-sr` escreve por ele — o
   `Starting`, as três primeiras partições, o começo da quarta. Chega a **12.891
   bytes** em 22/08, a **12.925** em 24/08;
2. na última passagem, o Clonezilla **reabre o mesmo arquivo com truncamento**, e
   o Partclone escreve a tela dele a partir do byte 0. São 4.085 bytes;
3. o descritor da receita, com o offset intacto, retoma onde estava. O intervalo
   entre 4.085 e o offset antigo vira zeros, e o texto final — `Cloned
   successfully`, `ocs-restore-mbr`, `ntfsfix`, `Ending` — é escrito depois dele.

**É a mesma família de P-25, pelo outro lado.** Lá, dois descritores sobre o
`arca-check.log` fizeram o relatório do `ocs-chkimg` cair por cima da tela do
Partclone; o segundo escritor estava atrás. Aqui ele está à frente, e o que
aparece entre os dois não é texto sobrescrito: é o vão.

## As cinco previsões, e a quinta estava mal escrita

O medidor `recursos/medir-arca-restore-log.sh` julgou o log de 24/08 contra as
cinco consequências registradas antes:

| # | O que se previu | Saiu |
|---|---|---|
| 1 | uma única inicialização de terminal, no offset 0 | bate |
| 2 | a tela é a da última partição (`nvme0n1p4`) | bate |
| 3 | um bloco de NULs entre o fim da tela e o texto final | bate |
| 4 | `Ending` sem o `Starting` correspondente | bate |
| 5 | o buraco não começa em 4.085 **nem** termina em 12.890 | **metade** |

**A quinta cobrava demais, e o script deixou passar.** Ela foi escrita com um
"nem" — pedindo que as duas pontas do buraco mudassem — e implementada com um
`||`, que se satisfaz com uma. O fim mudou (12.890 → 12.924). **O início não: é
4.085 nos dois.**

Isso não contraria a hipótese; é a hipótese sendo mais precisa do que a previsão.
O início do buraco **é** o tamanho da tela do Partclone, porque é a tela que
reabre o arquivo. Uma tela de terminal de 24 linhas sobre a mesma partição tem
sempre o mesmo número de bytes — e a comparação byte a byte confirma que tem.
Prever que ela mudaria era prever contra o próprio mecanismo.

Quem carrega a informação que P-23 pedia é o **fim**, e só ele: é onde o `ocs-sr`
tinha chegado quando o truncamento veio. Ele mudou 34 bytes entre uma restauração
e outra — o bastante para dizer que **o corte não é fixo**, que era a pergunta.

O medidor foi corrigido para julgar só o fim, e o início saiu da contagem e virou
reforço: um valor constante ali **confirma** a hipótese, e é o oposto do que a
previsão original fazia com ele.

## O que isto fecha

**P-23 fecha, e fecha pela positiva.** A pergunta era *"o corte cai sempre no
mesmo lugar?"*, e a resposta é **não**: ele cai onde o `ocs-sr` tinha chegado.
O corte não é do ARCA, não é do `>` da receita, e não é do redirecionamento —
é o Clonezilla reabrindo o próprio arquivo de log na última passagem.

**E há uma consequência que o §6.3 não diz.** Aquela tela aponta o
`arca-restore.log` a quem colheu uma restauração e quer saber o que aconteceu.
O arquivo está lá, e o que ele promete é verdadeiro. O que ele não avisa é que
**traz uma passagem só** — e que a passagem que sobrevive é a última, que numa
falha é justamente aquela em que a operação parou.

Para o caso de falha isso é melhor do que parece: a partição que interessa é a
que quebrou, e é ela que sobra. Para o caso de sucesso é o que se viu — três
partições invisíveis e 53% de zeros.

## O que isto não conserta, e por quê

**Trocar o `>` da restauração por `>>` faria o buraco sumir.** Sem truncamento
do lado da receita, o texto final ficaria colado na tela do Partclone e os 8,8 KB
de zeros não existiriam.

**Não recupera o que se perdeu.** Quem esvazia o arquivo é o Clonezilla, ao
reabri-lo — age antes do primeiro byte, e não entre o redirecionamento e o disco.
É a mesma razão pela qual o `>>` sobreviveu a P-25 com a justificativa trocada:
ele não compra a preservação, mas também não abre a janela em que o `>` deixaria
uma operação boa com o log em zero byte.

É **decisão, não pendência** — e não foi tomada aqui.

## Consequências

- **P-23 sai da lista de pendências.** O `o-que-falta-para-fechar.md` a registra
  fechada, com o mecanismo e a data.
- **O §6.3 continua verdadeiro** no que promete. O que se ganhou é saber o que ele
  não promete, e isso vale para quem for ler um log de restauração que falhou.
- **A previsão 5 fica registrada como escrita, e como corrigida.** Este projeto já
  pagou cinco vezes por confundir o que se mediu com o que se explicou depois
  (§3.5 do PRD); uma previsão que o script deu por satisfeita com metade da
  condição é a mesma armadilha, em escala menor. A correção está no medidor, com
  a razão no comentário.
- **O mecanismo é do Clonezilla, e vale para os dois logs.** `arca-check.log` e
  `arca-restore.log` sofrem o mesmo dois-descritores-um-arquivo, e a diferença
  entre eles é só qual dos dois escritores está à frente.

> **A ordem foi a certa, e é a razão de este ADR poder afirmar o que afirma.** O
> mecanismo e as cinco consequências entraram no repositório **antes** da
> restauração — `0e83b3f`, 19:48 — e a restauração aconteceu às 20:51. O que se
> lê acima não é explicação retroativa de um arquivo já visto: é uma previsão
> datada, que depois foi medida. Inclusive na parte em que ela errou.
