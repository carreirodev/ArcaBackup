# R-7 recusa por medição, não por corrupção — e a régua do destino é o `MSFT_Disk`

R-7 manda recusar quando o destino for **menor** que a origem, e dá a razão:
*"`-k0` copia a tabela inteira e, num disco menor, **corrompe** em vez de
falhar"*. A decisão 5 do plano repete a mesma premissa. Nenhuma das duas foi
medida, e as duas estão erradas.

O help do `ocs-sr` **desta** versão, tirado desta máquina e preservado em
`recursos/capturas/ocs-sr-help.txt`, diz o contrário:

```text
-icds, --ignore-chk-dsk-size-pt  Skip checking destination disk size before
creating the partition table on it. By default it will be checked and if the
size is smaller than the source disk, quit.
```

**Por padrão o Clonezilla confere e desiste.** `-icds` é quem desligaria a
conferência, e a receita de restauração não o usa — há teste em
`src/receita.rs` cobrando isso desde a E3, escrito quando ele guardava o
oposto do que se sabia. Era P-17, aberta na E3 e resolvida aqui.

Escrever a E9 exigiu decidir duas coisas em cima disso.

## Decisão 1: a recusa fica, e a razão muda

R-7 continua recusando destino menor. O que muda é o porquê, e a diferença tem
consequência prática.

Não é *"senão corrompe"* — não corrompe. É **onde** a recusa acontece. A do
Clonezilla acontece do outro lado do reinício, dentro de um boot desatendido:
a máquina reinicia, o `ocs-sr` desiste, o `if/then/else` de R-5 escreve
`ARCA_RESTORE=FALHOU`, o `sleep 20` roda, a máquina desliga, e alguém religa e
roda `arca resultado` para descobrir o que já se sabia antes de sair da
cadeira. A do ARCA custa zero reinícios.

Isso é defesa em profundidade com um ganho medível, e não desconfiança do
Clonezilla. Duas notas sobre o que ela **não** é:

- **Não é a única defesa.** Se a recusa do ARCA tiver um furo, a do Clonezilla
  ainda pega — e é por isso que `-icds` continua fora da receita, e por isso
  que o teste que o proíbe continua valendo.
- **Não substitui medir.** O help fala de *"the size"*, sem dizer qual fonte, e
  a decisão 2 é sobre exatamente isso.

## Decisão 2: o destino se mede pelo `MSFT_Disk`, e a comparação sai em setores

Esta é a parte que custou a medição, e ela é o achado da etapa: **o mesmo disco
tem dois tamanhos conforme quem responde.** Medido nesta máquina em
23/08/2026, no `KINGSTON SNV3S500G`:

| Fonte | Bytes | Setores de 512 B |
|---|---|---|
| `MSFT_Disk` (`Get-Disk`) | 500.107.862.016 | 976.773.168 |
| `Win32_DiskDrive.Size` | 500.105.249.280 | 976.768.065 |
| `nvme0n1-gpt.sgdisk`, dentro da imagem | — | **976.773.168** |

A diferença é de 2.612.736 bytes, e ela não é ruído nem defeito de hardware:
`60801 × 255 × 63 × 512` dá **exatamente** o número do `Win32_DiskDrive`. É o
produto da geometria CHS legada, truncado no último cilindro inteiro — os 5.103
setores que faltam são menos de um cilindro (16.065), que é a assinatura desse
truncamento. O `MSFT_Disk` bate byte a byte com o que a imagem registra.

**A armadilha é de régua, e ela é sobre código que ainda não existia.** Medir a
origem pela GPT de dentro da imagem — que é a única medida da origem que existe
do lado Windows — e o destino pelo `Win32_DiskDrive`, que é a fonte que
`Discos::discos_fisicos` usa desde a E6, faz a comparação sair de duas réguas.
O destino aparece 2,6 MB menor **inclusive quando origem e destino são
fisicamente o mesmo disco**. O caso normal da restauração — devolver a imagem
ao disco de que ela veio — seria recusado por R-7, e a mensagem diria que o
disco não cabe nele mesmo.

Não é número inventado; é número **medido na coisa errada**, que é o nome que a
E6 deu ao `498,7 GB` do §5.2. É a segunda vez do mesmo padrão, e desta vez ele
apareceu antes de a linha ser escrita.

Então:

- **A medida de R-7 vem do `MSFT_Disk`**, num campo novo — `DiscoFisico::medida`
  — e não substituindo o `tamanho_bytes`. Para B-4 a fonte antiga continua
  servindo: lá ela superestima o em uso, e superestimar é o lado seguro de
  "cabe uma imagem?". Aqui ela para de servir.
- **A comparação é em setores**, com o tamanho do setor **lógico** lido dos dois
  lados: `LogicalSectorSize` no `MSFT_Disk`, e o primeiro número do
  `Sector size (logical/physical)` no `sgdisk`. Setores lógicos diferentes são
  **recusa**, e não conversão: a tabela de partição da imagem é escrita em
  setores da origem e `-k0` a copia inteira, e o que ela endereçaria num disco
  de outro setor não está medido neste projeto.
- **"Não consegui medir" é recusa**, e nunca cai de volta no `Win32_DiskDrive`.
  Um `MSFT_Disk` que não responda deixa `medida` em `None`, e R-7 para ali
  nomeando isso.

## Por que não as outras saídas

**Tolerar a diferença** — aceitar que o destino seja até um cilindro menor —
resolveria o caso normal e é a pior das três. Uma margem numa comparação que
decide se um disco vai ser apagado é exatamente o tipo de número sem oráculo que
este projeto passou nove etapas removendo. E ela esconderia o achado: quem
lesse `if destino + 16065 >= origem` não teria como saber por quê.

**Medir a origem também pelo `Win32_DiskDrive`** — pôr as duas pontas na régua
CHS — é impossível: a origem não está nesta máquina. Ela é uma pasta no
`ARCAVAULT`, e a única coisa que sabe de que tamanho era aquele disco é a GPT
que o Clonezilla gravou dentro dela.

**Decidir que R-7 não se responde bem do lado Windows** e delegar tudo ao
Clonezilla é defensável, e foi descartada pelo custo: a recusa dele custa um
reinício, e é um reinício de uma operação destrutiva. E ela deixaria o ARCA sem
nada a dizer na tela de confirmação sobre se aquele destino cabe — que é
justamente a informação que alguém prestes a apagar um disco quer ver.

## O que a decisão obriga a manter

**Duas medidas do mesmo disco convivendo no `DiscoFisico`.** É custo real, e
está pago com nome: o campo `medida` tem doc dizendo por que existe ao lado do
`tamanho_bytes`, e `duplos::discos_desta_mesa()` traz os **dois números
diferentes** de propósito — um duplo que repetisse o mesmo valor nos dois campos
faria todo teste de R-7 passar sem exercitar nada.

**Um teste que documenta o número errado.** `src/gpt.rs` tem
`a_medida_da_imagem_nao_bate_com_o_win32_diskdrive`, que afirma a diferença de
2.612.736 bytes e reproduz o produto CHS. Ele existe para que trocar a fonte de
volta não seja uma mudança silenciosa: com o número errado escrito e nomeado, a
troca quebra um teste que diz o que ela quebra.

**E um teste contra o hardware.**
`tests/e9_restaurar_o_disco.rs::o_msft_disk_bate_byte_a_byte_com_a_gpt_de_dentro_da_imagem`
compara, para cada imagem do dispositivo, o que o Windows diz agora com o que a
GPT registrou quando ela foi feita. É o único que pode falhar por uma mudança
fora deste repositório.

## Consequências

R-7 e a decisão 5 do plano são reescritos contra o help, e P-17 fecha.

`arca restore` ganha `--destino <indice>`, que é o que torna a metade permissiva
de R-7 alcançável — sem ela, a recusa por destino menor seria uma regra que
nunca dispara. O índice é o do **Windows**, e nunca chega à receita: o ARCA
traduz índice → modelo → nome do Linux pelo `blkdev.list` de dentro das imagens,
que é o oráculo do §4.5. Aceitar `--destino nvme0n1` seria pôr numa receita
destrutiva um nome do Linux digitado do lado Windows, que é o que a E7 recusou
por não ter contra o que conferi-lo.

E fica registrado o que continua sem medida: **o que acontece de fato quando o
`ocs-sr` desiste por destino menor.** O help diz `quit`; nenhuma execução deste
projeto o exercitou, e nesse ponto ele é P-6 com outra roupa — se o `quit` sair
com código zero, R-5 escreve `OK` sobre uma restauração que não aconteceu. A
recusa do ARCA acontecer antes é o que torna essa pergunta barata de nunca
precisar responder.
