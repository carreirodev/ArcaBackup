# A ordem permanente muda no ciclo de boot, e não à mão

O §3.1 do PRD registrava que a ordem permanente de boot desta máquina mudou
"pelo menos três vezes", e atribuía a mudança a **alguém**: "a evidência é de
que já foi feito à mão". C-5 existe para impedir que o ARCA o faça, e a
conclusão que sobrava era que uma pessoa vinha desfazendo o que C-5 protege.

O marco em hardware de 22/08/2026 mediu os dois lados do **mesmo** reinício, e
a explicação é outra — mais simples, e ela cobre os três casos.

## O que foi medido, e cada leitura tem hora

O backup `2026-08-22_Apps` foi armado às 20:53:48 e colhido às 21:14:49. No
meio, um reinício. As quatro leituras, na ordem:

| Quando | Ferramenta | Entradas | Ordem de boot | Bootou por |
|---|---|---|---|---|
| 20:41:45 | `bcdedit` (`arca.log`) | 2 | `{bootmgr}` — **o ARCA fora da ordem** | — |
| ~20:57 | `efibootmgr` (`nvram-live-2026-08-22.txt`) | 2 | `0000,0001` — **Windows à frente** | **`0001`** |
| ~21:07 | — | — | — | Windows, com o dispositivo conectado |
| 21:17:27 | `bcdedit` (`arca.log`, e a captura) | **3** | `{f4057bd0}`, `{bootmgr}`, `{687478f2}` — **o ARCA à frente** | — |

A terceira linha não tem ferramenta porque a evidência é de outra natureza, e
ela é boa: ao religar depois do backup, com o SSD conectado, a máquina foi
**direto ao Windows**. Isso data a mudança da ordem melhor do que qualquer
captura — no instante do religar, o Windows ainda estava à frente.

## O que sai daí, e a primeira coisa fecha P-18

**`BootCurrent: 0001` com `BootOrder: 0000,0001`.** A máquina bootou pela
entrada `0001` estando a `0000` à frente da ordem. Nenhuma ordem permanente
explica isso; o `bootsequence` explica. **O firmware honra a marca sobre uma
entrada que não está à frente da ordem de boot** — a metade de P-18 que só o
hardware respondia, e a prova de que C-5 é sustentável.

E ela fecha pelo lado bom justamente onde o ADR-0007 temia o contrário: a
mesma leitura, feita em 21/08, mostrava `BootOrder: 0001,0000` — o dispositivo
à frente —, e era por isso que o backup daquele dia não provava nada. Hoje a
ordem está do outro jeito, e o resultado é o mesmo. A diferença entre as duas
leituras é a diferença entre uma coincidência e uma medição.

**A captura vale mais do que o `bcdedit` do lado Windows para esta pergunta**,
e a razão é o momento: ela foi escrita pelo Clonezilla **durante** o boot que
se quer explicar. Um `bcdedit` lido depois, no Windows, descreve o firmware
como ele ficou — e é exatamente o que muda no meio.

## A segunda coisa: quem mudou a ordem foi o ciclo de boot

Entre 20:41 e 21:17 ninguém tocou no `bcdedit` a não ser o ARCA, e o ARCA só
escreve `bootsequence`. Três coisas mudaram assim mesmo:

- A entrada `0001` foi **reescrita**. Em 21/08 ela era `ARCA`,
  `\EFI\boot\bootx64.efi`, com `data:` trazendo o `BCDOBJECT={f4057bd0-…}` que
  o `bcdedit` grava. Em 22/08, no mesmo device path, ela é `UEFI OS`,
  `\EFI\BOOT\BOOTX64.EFI` em maiúsculas, `data: 00 00 42 4f`. É a forma
  canônica que um firmware escreve ao enumerar um dispositivo bootável, e não
  a que o Windows escreve.
- Uma **terceira** entrada apareceu, `{687478f2-9e87-11f1-8a47-806e6f6e6963}`,
  também em `partition=R:`.
- O `displayorder` passou de só `{bootmgr}` para três entradas, com a do ARCA
  **em primeiro**.

**O ARCA não fez nada disso, e há mais do que a palavra dele.** `src/armar.rs`
lê a ordem permanente antes de escrever e a relê depois, e falha com
`OrdemPermanenteAlterada` se ela mudar; `src/desarme.rs` faz o mesmo. As duas
rodaram nesta operação e nenhuma acusou. Elas comparam a ordem através da
**própria escrita** — e a mudança não aconteceu numa escrita do ARCA, aconteceu
no reinício entre elas, que é o único intervalo que nenhuma das duas observa.

A reconstrução que explica as quatro leituras sem sobrar nada:

1. O firmware consome o `bootsequence`, boota o dispositivo, e **reescreve a
   entrada** na sua forma canônica — `UEFI OS`, path em maiúsculas. O
   `BCDOBJECT` que ligava aquela entrada ao objeto do BCD se perde aí.
2. O Windows sobe. Encontra no BCD o objeto `{f4057bd0}` descrevendo uma
   entrada de firmware que não tem mais correspondente na NVRAM, e a
   **recria** — pondo-a no `displayorder`, que é o que criar uma entrada faz.
3. Sobram as três entradas de agora: a recriada, o `{bootmgr}`, e a que o
   firmware escreveu.

**Isto reinterpreta a tabela do §3.1 inteira.** As três mudanças que ela
atribuía a trabalho manual têm agora uma causa que não pede ninguém: cada boot
pelo dispositivo mexe na ordem. Inclusive o detalhe que a tabela achou digno de
nota — o número da entrada indo de `0001` para `0003`, "o que só acontece
quando ela é recriada". Recriada, sim; por ninguém.

## A terceira coisa, e é a que tem consequência operacional

Depois de um backup, esta máquina fica com o dispositivo **à frente** da ordem
permanente. Enquanto ele estiver conectado, o próximo reinício boota nele.

Com o `grub.cfg` inerte, isso é o menu do Clonezilla: a máquina para e espera
alguém, que é chato e não destrói nada. **Com o `grub.cfg` ainda armado, isso é
a receita rodando de novo** — e a janela em que ele fica armado é exatamente a
que vai do fim da receita até o `arca resultado`. Em 22/08 ela durou oito
minutos, das 21:06:02 às 21:14:50.

Não foi o que aconteceu, e a razão é a segunda linha da reconstrução: no
instante do religar a ordem ainda tinha o Windows à frente, e só depois o
Windows recriou a entrada. A janela existiu e não foi exercitada. Isso é sorte
de sequência, não desenho — e o aviso de C-9, "remova o SSD antes de religar",
é a defesa que já estava lá, escrita antes de alguém saber disso.

## A decisão: o ARCA avisa, e não conserta

Três saídas, e duas se descartam rápido.

**Devolver a ordem** — o ARCA tirar a entrada do `displayorder` ao colher —
está fora. É escrever na ordem permanente, que é o que C-5 proíbe, e a
proibição não tem cláusula para o caso de o ARCA achar que está arrumando. Pior:
a entrada foi posta pelo Windows a partir do BCD, e tirá-la seria desfazer uma
decisão de outro dono, no lugar onde um erro deixa a máquina sem bootar.

**Não fazer nada** deixa o usuário descobrir sozinho, num reinício qualquer, que
a máquina agora para num menu em inglês técnico. É o modo de falha que o §3.2
descreve, chegando pelo caminho que ninguém esperava.

**Decidimos avisar.** O `arca status` passa a dizer quando o dispositivo está à
frente da ordem permanente, e o que isso significa para o próximo reinício. É
leitura, não escrita — não encosta em C-5 —, e é a informação de que alguém
precisa para não ser surpreendido. O ARCA já lê o `{fwbootmgr}` inteiro em todo
comando; o dado estava na mão e não estava sendo dito.

### A pergunta é sobre o dispositivo, e não sobre a entrada chamada `ARCA`

A primeira versão daquela linha procurava **a entrada do ARCA** na ordem, e a
revisão a derrubou com a captura desta própria máquina. Há **duas** entradas em
`partition=R:` — a `{f4057bd0}` do ARCA e a `{687478f2}` `UEFI OS` que o
firmware criou —, e foi pela segunda que a máquina bootou. Com a `{687478f2}`
em primeiro e a do ARCA atrás do Windows, aquela versão diria "o Windows vem
antes" e engoliria o aviso, enquanto todo reinício com o SSD conectado
continuaria bootando no dispositivo.

**O que decide o boot é para onde a entrada aponta, e não como ela se chama** —
a mesma lição de C-4, que procura a entrada legada pela descrição mas confere o
alvo, e de C-6, que desconfia do nome que o `bcdedit` devolve. A seção passa a
percorrer a ordem, resolver cada identificador na entrada que ele nomeia, e
perguntar se o alvo é o `ARCABOOT` que está na mesa.

E ela guarda em `viu_o_gerenciador` antes de qualquer coisa. `firmware::ler`
nunca falha: um `bcdedit` que não se deixou entender produz `ordem_permanente`
vazia, e vazia é indistinguível de "o dispositivo está fora da ordem" — que é a
resposta tranquilizadora. Sem a guarda, "não entendi a resposta" viraria uma
afirmação de segurança, que é o mesmo erro que C-3 existe para não cometer.

## O teste que guardava a premissa muda de asserção

`tests/e7_armar_o_dispositivo.rs` tinha
`a_entrada_do_arca_esta_fora_da_ordem_permanente`, e ele reprovou nesta sessão —
fazendo exatamente o que foi escrito para fazer. O que ele guardava era a
**premissa de uma medição que ainda não existia**: enquanto o boot único não
tivesse rodado, a entrada estar fora da ordem era o que faria a medição
significar alguma coisa.

A medição existe agora, e é melhor do que a premissa exigia: o
`nvram-live-2026-08-22.txt` registra a ordem **no instante do boot**, e não
depende de o firmware continuar em qualquer configuração. O teste cumpriu a
função e a premissa não é mais necessária.

Mantê-lo como está seria pedir que o hardware volte a uma configuração que o
próprio ciclo de boot desfaz a cada backup — uma suíte vermelha por uma
condição que o ARCA não controla e que é, agora, o estado normal. Apagá-lo
perderia o alarme. Ele passa então a cobrar **a invariante que importa e que o
ARCA controla**, que é a do perigo real desta seção:

> Se alguma entrada que leva ao `ARCABOOT` está em **primeiro** na ordem
> permanente, então o `grub.cfg` tem de estar inerte e não pode haver boot
> único pendente.

Essa é a combinação que roda um backup sem ninguém pedir, e é verificável a
qualquer momento, com ou sem o dispositivo à frente.

**"Em primeiro" e não "na ordem"**, e a diferença tem consequência: um
dispositivo armado atrás do Windows é o estado **normal** da janela entre o
`arca backup` e o reinício. Cobrar inércia ali deixaria a suíte vermelha
acusando um perigo que não existe — e, pior, contradizendo o `arca status`, que
declara essa mesma configuração segura. Duas versões da mesma regra divergem na
primeira mudança, e aqui elas divergiram antes de a primeira mudança chegar. E entra ao lado dela um
teste que fixa o fechamento de P-18 contra a captura nova, do mesmo jeito que a
E7 fixou o ADR-0007: `BootCurrent` fora da frente da `BootOrder` é o que prova
o `bootsequence`, e uma captura que deixasse de mostrar isso deixaria de ser a
evidência que o §3.1 diz que ela é.

## Consequências

O §3.1 perde a frase que atribui as mudanças de ordem a trabalho manual, e a
tabela ganha as leituras de 22/08 com hora. P-18 sai do §3.5 como fechada, e
com ela a linha que o §3.1 mantinha em suspenso.

O §11 ganha a armadilha: **medir o firmware depois do reinício não descreve o
reinício.** As duas leituras do `bcdedit` desta noite discordam entre si pelo
mesmo motivo que o §3.1 já tinha aprendido para o par `bcdedit`/`efibootmgr` —
momentos diferentes —, e desta vez a ferramenta é a mesma. O que datou a
mudança não foi captura nenhuma: foi alguém dizer em que a máquina bootou.

C-5 continua intacta, e agora com a razão de ser medida: o `bootsequence`
funciona sem a entrada na ordem, então não há troca a fazer entre armar e
respeitar a ordem permanente.

E fica aberto o que esta medição não alcança: **se o firmware reescreve a
entrada em todo boot pelo dispositivo, ou só quando ela foi consumida por
`bootsequence`**. Um segundo backup responde, e a leitura que responde é o
`efi-nvram.dat` que o Clonezilla escreve sozinho dentro de cada imagem — ela já
está sendo colhida, de graça, desde antes de alguém saber para que serviria.
