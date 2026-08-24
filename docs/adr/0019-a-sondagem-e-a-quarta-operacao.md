# A sondagem é a quarta `Operacao`, e ela é a segunda fonte do §4.5

Decidido em 23/08/2026, na etapa E12. **Rodou em hardware em 24/08/2026** duas
vezes — a segunda com uma coluna inventada, para exercitar o ramo do erro. A
primeira fechou P-26 e P-27; a segunda escreveu o **primeiro `FALHOU` deste
projeto**. Ver *"O marco"* e *"A falha forçada"*, no fim.

## O contexto: três comandos que armam, e nenhum funciona num dispositivo novo

O §4.5 diz que o nome do disco no Linux sai do `blkdev.list` de dentro de uma
imagem. Um dispositivo recém-preparado **não tem imagem**, logo não tem o nome,
logo `arca backup` recusa — e `arca restore` e `arca verify --completo` também,
porque os dois precisam de uma imagem que não existe.

A E10 fechou com essa consequência escrita na tela do `arca prepare` e com P-26
aberta. **A resposta que aquela tela dava era o menu do Clonezilla** — F12,
primeiro backup à mão pelo §6.4, e daí em diante o ARCA. Ela não estava errada
sobre os fatos, e continua sendo o caminho manual quando tudo o mais falhar. O
que ela era: exatamente aquilo que este app existe para não precisar, cobrado
logo na primeira vez que alguém usa um dispositivo novo. Custava dois reinícios
e cerca de quarenta minutos.

`arca sondar` custa um reinício e nenhuma tela do Clonezilla.

---

## Decisão 1: a sondagem é uma quarta `Operacao`, e quem decidiu foi a mesma coisa da E11

`Operacao` tinha três valores, e cada um atravessa `receita.rs`, o
`estado.json`, o marcador do `arca-fim.txt`, a `pasta_do_log`, o `arca
resultado` e o `arca status`. O [ADR-0016](0016-a-verificacao-armada-e-a-terceira-operacao.md)
já respondeu a pergunta "vale uma a mais?" para a verificação, e o argumento
dele vale aqui sem mudar uma vírgula:

> Toda receita começa truncando o próprio `arca-fim.txt` com um `>`. […] Uma
> verificação armada que reusasse `Backup` cometeria o mesmo defeito pela
> terceira vez.

Pasta própria vem do nome da operação, e o nome da operação é o enum.

O marcador do desfecho é `ARCA_PROBE=`, e ele é **código novo** — nenhuma
receita real o escreveu. A *forma* é transcrita dos três que rodaram: `ARCA_`
mais o nome da operação em inglês, maiúsculo. `PROBE` e não `SONDAR` porque os
outros três estão em inglês, e um marcador em português no meio de três em
inglês seria o começo de duas convenções.

---

## Decisão 2: a pasta do log é **fixa**, e a sondagem anterior é substituída

`pasta_do_log` produz `"{operacao}-{nome}"`, e a sondagem não tem nome de
imagem. Com pasta fixa, duas sondagens colidem e a segunda trunca o
`arca-fim.txt` da primeira — que é literalmente o defeito que a revisão da E3
pegou entre o backup e a restauração.

**Aqui é o comportamento certo, e a diferença é o que se perde.**

Lá o que se perdia era o desfecho de **outro job**, sobre outra pergunta, que
ninguém mais ia reproduzir: um `arca backup X` colhido tarde demais perdia o
resultado de quarenta minutos de gravação. Aqui o que se perde é a **medição
anterior da mesma pergunta** — *que discos há nesta máquina?* —, e a resposta
mais recente é a que vale. Uma sondagem velha descrevendo uma máquina que mudou
é pior do que nenhuma, e refazê-la custa um reinício.

O que isso custa está dito e é o mesmo das outras três com o mesmo nome: uma
sondagem armada por cima de outra ainda **não colhida** trunca o `arca-fim.txt`
dela. Há um `estado.json` por dispositivo, então aquele job já estava perdido
antes de a pasta ser tocada.

`sondagem` não colide com nenhuma das outras três: as delas são
`{operacao}-{nome}`, e `Nome` nunca é vazio. **Inclusive com uma imagem chamada
`sondagem`**, que B-2 aceita — `backup-sondagem` e `sondagem` são pastas
diferentes, e há teste cobrando.

E os **dois** arquivos da sondagem moram nessa pasta: o `arca-fim.txt` e o
`blkdev.list`. `ARCA-LOGS` está fora da listagem de imagens desde a E1, com
teste — sem isso, `ARCA-LOGS\sondagem\` apareceria no `arca list` como resíduo
(não tem `MD5SUMS`) e B-3 passaria a recusar o nome `sondagem` para um backup.

---

## Decisão 3: o `nome` do `Pedido` e do `estado.json` vira opcional, com o vazio como sentinela

O precedente é da E11 e está no ADR-0016: o `disco` virou `Option<Disco>` com a
**string vazia** no arquivo, porque `Disco::novo("")` já recusava desde a E3 —
o vazio nunca foi um nome de disco possível, então usá-lo para dizer "nenhum"
não pode colidir.

**Conferido antes de reusar**, que é o que o método deste projeto manda:
`Nome::novo("")` recusa com `Recusa::Vazio` desde a E1. O mesmo argumento vale,
e a decisão está tomada por precedente.

O sentinela óbvio seria `sondagem`, e ele **colidiria**: B-2 aceita `sondagem`
como nome de imagem, e um `estado.json` de um backup chamado `sondagem` seria
lido como uma sondagem.

A coerência é cobrada nos **dois** sentidos, no leitor e em `Receita::montar`:
a sondagem exige nome vazio, e as outras três exigem nome. O segundo sentido
dói mais aqui do que doía no disco — **a pasta do desfecho sai do nome**, então
um `backup` com nome vazio procuraria o desfecho na pasta `backup-`, que não é
a de ninguém.

### E os dois eixos são independentes, o que quase se perdeu ao escrever

`nomeia_disco` e `nomeia_imagem` separam coisas diferentes, e a verificação é a
prova: ela **não** nomeia disco e **nomeia** imagem. Um campo só faria a
coerência do `estado.json` parar de cobrir metade das combinações.

| Operação | disco | imagem |
|---|---|---|
| backup | sim | sim |
| restauração | sim | sim |
| verificação | **não** | sim |
| sondagem | não | **não** |

---

## Decisão 4: `blkdev::Origem` ganha uma segunda variante, e ela leva a hora

`Origem` só sabia dizer `LidoDaImagem { imagem, modelo }`, e o pré-voo imprime
isso literalmente: `nvme0n1 · lido de 2026-08-21_WindowsCompleto/blkdev.list`.

Uma sondagem que se apresentasse como imagem seria a mesma falha que o `arca
prepare` acabou de pagar na E10 — uma tela afirmando o que não aconteceu —, e
com um agravante: **não há imagem nenhuma no dispositivo em que a sondagem mais
importa**.

A variante leva **quando** a sondagem foi feita, e o campo não é enfeite: uma
sondagem descreve a máquina do instante em que rodou, e sem a data `lido da
sondagem` não distingue a de cinco minutos atrás da de um mês — e a segunda pode
estar descrevendo um disco que não está mais na máquina.

O valor sai do `mtime` do arquivo, que é o relógio **do Windows**, e não o do
live que roda 3 h adiantado (P-7). É **informativo e nunca comparado** (S-6):
quem julga se o disco achado é o certo continua sendo o modelo. Ele é impresso,
como o `dia_e_mes` das imagens no `arca list` — com a hora junto, porque a pasta
da sondagem é fixa e duas sondagens do mesmo dia não têm nome que as separe.

---

## Decisão 5: a sondagem ganha das imagens, e a divergência é dita

As duas fontes respondem a mesma pergunta sobre instantes diferentes: a sondagem
descreve a máquina de **agora**, e a imagem descreve a de quando o backup foi
feito. Um disco trocado entre as duas faz a imagem nomear um disco que não está
mais lá.

Então a sondagem é consultada primeiro, sozinha. Respondendo, é a resposta — e o
que as imagens dizem do mesmo modelo entra como `divergencia`, que sai na linha
do disco (`DIVERGE de …`) e ganha um aviso próprio no pré-voo, explicando que
há duas fontes, que elas falam de instantes diferentes, e qual delas o ARCA
usou. **Nunca resolvida em silêncio.**

Vale registrar que a defesa velha continua embaixo desta: o casamento é por
**modelo**, e uma sondagem obsoleta que descrevesse outro disco cai em
`ModeloNaoCasa`, que é recusa e não palpite.

### `SemOraculo` é a única recusa da sondagem que deixa as imagens falar

As outras três são afirmações sobre a máquina de agora, e não a ausência de uma.
`ModeloAmbiguo` diz *"há dois discos deste modelo aqui, neste instante"* —
resolver isso por um `blkdev.list` de um backup antigo é exatamente o chute que
aquela recusa existe para não dar. Então ela vence, e o comando para.

`SemOraculo` é outra coisa: é *"não há sondagem"*, e é o caminho de antes da
E12, em que as imagens sempre responderam.

### E o `arca restore` usa a mesma lista, e não uma parecida

Ele resolve **dois** nomes pelo oráculo: o do disco de destino (R-2, passo 5) e
o do próprio dispositivo, na recusa que a revisão da E9 achou. Os dois falam do
hardware que está na mesa **agora**.

Deixar a sondagem de fora dali teria um custo concreto e assimétrico: o `arca
backup` acharia o disco por ela e o `arca restore` não acharia, sobre a mesma
máquina e no mesmo minuto. Por isso a lista é montada por
`backup::fontes_do_oraculo`, e não por uma cópia.

---

## Decisão 6: a confirmação é uma tecla, e ela não finge ser S-2

A pergunta que a decisão tinha de sobreviver é *"o que essa confirmação
impede?"*, porque uma confirmação que não impede nada ensina o usuário a digitar
sem ler.

S-2 pede o **alvo** por extenso, e existe para custar lê-lo: o nome da imagem
que vai ser gravada (§5.2), o nome da que vai ser restaurada (R-3), o modelo do
disco que vai ser apagado (PR-4). O que ela impede é **agir sobre a coisa
errada**.

A sondagem não tem alvo. Ela não apaga nada, não escolhe nada, e o único
irreversível dela é **reiniciar a máquina** — que não é pouco para quem está
trabalhando.

**Pedir a palavra `sondar` por extenso seria ruído.** Quem acabou de digitar
`arca sondar` a ecoaria sem ler nada: é o exemplo canônico da confirmação que só
prova que há alguém no teclado. Copiaria a forma de S-2 sem a razão dela, e
gastaria o único recurso que S-2 tem — a disposição de quem lê para levar uma
confirmação a sério.

O que fica é a pergunta de uma tecla com o padrão no **não** — a mesma do
primeiro tempo de PR-4, reusada e não copiada (ela saiu de
`src/comandos/prepare.rs` para `src/confirmacao.rs`). O que ela impede está dito
na tela imediatamente acima dela: o reinício de quem digitou o comando sem saber
que ele reinicia.

O `arca verify --completo` **continua** pedindo o nome por extenso, e a
diferença não é inconsistência: lá há uma imagem escolhida, e digitá-la confirma
que é a certa.

---

## O `if` não é enfeite, e a primeira forma escrita desta receita não o tinha

A forma proposta encadeava com `;`:

```text
lsblk -o ... > .../blkdev.list; echo ARCA_PROBE=OK >> .../arca-fim.txt;
```

O `;` não olha código de saída. Com o `lsblk` falhando — uma flag que esta
versão do util-linux não conheça basta —, o desfecho diria **`OK`** assim mesmo.
É R-5, e é o passo 3 de `montar_backup` desde a E3.

O estrago não é abstrato: `blkdev::ler` devolveria lista vazia, o disco de origem
sairia `POR DETERMINAR`, e a tela diria isso **logo depois** de o `arca
resultado` ter dito que a sondagem concluiu com sucesso. Duas afirmações
contraditórias, as duas do ARCA, na mesma sessão.

**Medido num bash de verdade**, e as duas formas rodam lado a lado em
`recursos/ensaio-da-receita.sh`:

```text
Sondagem: o lsblk falhou — o if diz FALHOU e o ; diria OK
  ok   desfecho com o if ......... ARCA_SELO=… · ARCA_PROBE=FALHOU · ARCA_FIM
  ok   o erro do lsblk ficou no arquivo

Sondagem: a forma com ; escreveria OK sobre um lsblk que falhou (R-5)
  ok   a forma proposta na mesa mente ... ARCA_PROBE=OK
```

Foi pego **antes** de escrever, o que é a primeira vez neste projeto.

---

## As flags do `lsblk` são reconstrução, e há uma terceira procedência

Das outras três receitas temos a **linha de comando** que rodou: ela está dentro
do `ocs_live_run` das capturas de `grub.cfg`, e o código a copia caractere a
caractere. Da sondagem temos o **resultado** — o `blkdev.list` de dentro das
imagens, com o cabeçalho `KNAME NAME SIZE TYPE FSTYPE MOUNTPOINT MODEL` — e não
temos a linha: ela mora nos scripts do Clonezilla, dentro do
`filesystem.squashfs`, que este repositório nunca abriu.

Reconstruir as colunas a partir do cabeçalho é honesto. Chamar isso de
transcrição não seria, e o §3.5 do PRD conta cinco vezes em que esse segundo
movimento custou caro. Então a tabela de procedências de `src/receita.rs` ganha
uma **terceira** coluna de resposta, e o `--dry-run` diz isso na tela.

**O `-i` é parte da reconstrução, e tem razão própria.** O arquivo capturado
desenha a árvore com `|-` e `` ` `` — ASCII. O `lsblk` só escolhe esses símbolos
quando o `CODESET` do locale não é UTF-8, e a receita boota com
`locales=en_US.UTF-8`, que §3.2 torna obrigatório. Sem `-i`, a árvore sairia com
os símbolos de caixa do Unicode e o arquivo deixaria de ter a forma do que ele
imita.

### O que torna a reconstrução aceitável é o modo de falha

Uma flag que o util-linux daquele live não conheça faz o `lsblk` sair com código
diferente de zero, e o `if` escreve `ARCA_PROBE=FALHOU`. O `2>&1` aponta para o
**próprio `blkdev.list`**: a mensagem de erro fica no dispositivo em vez de sumir
com o `poweroff`, e a próxima sessão lê **qual** flag foi recusada em vez de
achar um arquivo vazio.

Um arquivo com mensagem de erro não é lido como oráculo: o cabeçalho não bate, e
`blkdev::ler` devolve lista vazia — que é o que ela devolve para tudo o que não
entende.

Custa um reinício, e diz o que consertar.

---

## O pressuposto genuinamente novo já tinha original, e ninguém tinha notado

Esta receita escreve em `/home/partimag` **antes** de qualquer comando do
Clonezilla. As outras três só escrevem ali depois de o `ocs-sr` ou o `ocs-chkimg`
terem rodado. Se o repositório não estivesse montado nesse instante, o `mkdir`
criaria a pasta no tmpfs da RAM e o `poweroff` levaria tudo embora — falha
silenciosa, sem nada no dispositivo para investigar depois.

Está provado, e a prova é da E11. `montar_verificacao` tem exatamente esta forma
— passo 1 `mkdir -p`, passo 2 `echo ARCA_SELO= >`, e só no passo 3 o
`ocs-chkimg` —, rodou em 23/08/2026 às 16:53, e o resultado está em
`recursos/capturas/arca-fim-verificacao-2026-08-22_Apps.txt`: cinquenta e um
bytes que saíram daqueles dois primeiros passos.

**Quem monta o `/home/partimag` é o `ocs_repository="dev:///LABEL=ARCAVAULT"` do
boot, e não o `ocs-sr`.** É o único pressuposto genuinamente novo da sondagem, e
ele foi pago por uma etapa que não sabia estar pagando.

E há um segundo sinal, de graça: o `lsblk` roda com o repositório montado, então
a linha da partição do `ARCAVAULT` sai com `/home/partimag` no `MOUNTPOINT` —
como já sai nos `blkdev.list` capturados. O próprio arquivo testemunha que foi
escrito no lugar certo.

---

## O que isto não decidiu

**Se a sondagem devia gravar mais do que o `lsblk`.** Um `efibootmgr -v`, um
`sgdisk` de cada disco, o `dmesg` — tudo isso é concebível e nada disso SD-1
pede. Fica de fora: a receita mais barata deste projeto é barata porque faz uma
coisa, e cada comando a mais é uma coisa a mais que pode falhar num boot em que
ninguém está olhando.

**Se `arca list` devia mostrar a sondagem.** Ela não é imagem nem resíduo, e
`ARCA-LOGS` está fora da listagem de propósito. Quem quiser vê-la roda `arca
resultado`, que a imprime ao colher.

---

---

## O marco, em 24/08/2026

Armado às **14:56:55**, no dispositivo que o `arca prepare` criou em 23/08 e que
estava **vazio de imagens**. A máquina reiniciou, bootou pelo dispositivo, rodou
o `lsblk` e desligou sozinha — **1 min 40 s** de relógio de parede.

```text
E:\ARCA-LOGS\sondagem\arca-fim.txt ... 50 bytes
  ARCA_SELO=354da624e7fa0d21
  ARCA_PROBE=OK
  ARCA_FIM

E:\ARCA-LOGS\sondagem\blkdev.list .... 852 bytes
  sda       sda      447.1G disk           Maxtor Z1 SSD 480GB
  sda1      |-sda1   445.6G part ntfs  /home/partimag
  nvme0n1   nvme0n1  465.8G disk           KINGSTON SNV3S500G
```

**P-26 fechou inteira**, e o que junta as duas metades é a leitura de `arca
status` de minutos antes: `1 entrada(s), nenhuma para o dispositivo · so o boot
unico leva a ele`. Com a entrada fora da ordem permanente não havia outro
caminho — um F12 teria respondido só (a).

**P-27 fechou junto.** `ARCA_PROBE=OK` diz que o `if` tomou o ramo do êxito, e a
**forma** do arquivo diz o resto: a árvore saiu em ASCII, que é o que o `-i`
compra sobre o `locales=en_US.UTF-8` do boot. A reconstrução funcionou, e agora
a terceira procedência tem um caso.

E o **segundo sinal de graça** apareceu: `/home/partimag` no `MOUNTPOINT` do
`sda1`. O próprio arquivo testemunha que o repositório estava montado quando o
`mkdir` rodou.

### Dois defeitos que só a execução real mostrou

**1. Duas linhas da mesma tela afirmando fontes diferentes.** O `arca backup
--dry-run` disse, no pré-voo, `lido da sondagem de 24/08 11:58`, e quatro linhas
abaixo, no ensaio, `· lido do blkdev.list de uma imagem`. A segunda era uma
frase fixa, de antes de a sondagem existir: o `Ensaio` carregava um
`de_exemplo: bool`, que sabia dizer *se* o nome fora determinado e nunca *por
quem*.

O campo virou `origem: Option<&NomeDoDisco>`, e a linha do ensaio passou a ser
literalmente a que o pré-voo imprime. **É a mesma classe de defeito que esta
etapa existe para não cometer** — uma tela afirmando o que não aconteceu —, e
foi achada rodando o comando de verdade, com a suíte verde.

**2. A data da sondagem tinha o dono do relógio trocado, e o erro estava na
doc.** O campo `quando` sai do `mtime` do `blkdev.list`, e a decisão 4 acima
dizia que ele vinha do relógio **do Windows, e não do live**. É o contrário:
quem escreve o arquivo é o `lsblk`, do outro lado do reinício.

O marco desmentiu em uma linha — armado às `14:56:55`, arquivo carimbado
`11:58`. Três horas atrás, que é P-7 pelo lado de sempre.

**O valor fica como está**: somar três horas fabricaria um instante que ninguém
mediu. O que mudou foi a tela, que passou a dizer de quem é o carimbo. Para o
que o campo existe — separar uma sondagem da anterior — o deslocamento não
atrapalha: as duas vêm do mesmo relógio, e a distância entre elas é real.

---

## A falha forçada, no mesmo dia — e é o primeiro `FALHOU` deste projeto

Armada às **15:32:25**, com `FLAGS_DE_SONDAGEM` mutada para incluir uma coluna
inventada (`FLAGQUENAOEXISTE`). O binário foi compilado, copiado para o
`ARCABOOT` e usado para **armar**; a mutação foi revertida antes de colher, e
quem colheu foi o binário normal.

É o mesmo movimento do [ADR-0017](0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md)
— a entrada de medição foi apagada no fim — e da segunda execução do marco da
E10: exercitar o caminho que nenhuma execução normal exercita, e desfazer o que
foi montado para isso.

```text
E:\ARCA-LOGS\sondagem\arca-fim.txt ... 54 bytes
  ARCA_SELO=95772dae07463701
  ARCA_PROBE=FALHOU
  ARCA_FIM

E:\ARCA-LOGS\sondagem\blkdev.list .... 40 bytes
  lsblk: unknown column: FLAGQUENAOEXISTE
```

**Três coisas que nenhuma execução deste projeto tinha mostrado:**

- **O `if/then/else` de R-5 tomou o ramo do erro em hardware.** Ele existe desde
  a E3 e só tinha rodado no ramo do êxito, em cinco execuções. `ARCA_PROBE=FALHOU`
  é o primeiro `FALHOU` que o ARCA escreveu.
- **O `2>&1` guardou a causa no dispositivo.** A mensagem do `lsblk` ficou no
  próprio `blkdev.list` em vez de sumir com o `poweroff` — quarenta bytes que
  dizem exatamente qual coluna foi recusada.
- **E P-27 respondeu pelo outro lado**: aquele util-linux **valida as colunas** e
  sai com código diferente de zero quando não conhece uma. Se a reconstrução das
  sete estivesse errada, teríamos sabido exatamente assim.

**E as duas telas seguintes concordaram**, que é o que o `if` compra: o `arca
resultado` disse `Desfecho: o Clonezilla falhou e disse` e saiu com código 1
(S-5); o `arca backup --dry-run` disse `Disco de origem ..... POR DETERMINAR`.
Com o `;` da forma proposta na mesa, a primeira teria dito `OK` e a segunda
continuaria dizendo `POR DETERMINAR` — as duas do ARCA, na mesma sessão.

### O que ela **não** fecha, e vale dizer

**P-6 continua aberta.** A pergunta de lá é sobre o `ocs-sr`, e nenhuma resposta
do `lsblk` fala por ele. O que esta medição fecha é o outro lado, que vale por
si: a **forma** de R-5 funciona em hardware nos dois ramos, e o `arca resultado`
sabe imprimir um desfecho ruim.

### E ela expôs um buraco no teste que guardava a reconstrução

`as_colunas_do_lsblk_sao_as_do_cabecalho_capturado` procurava `-o <colunas>` como
**substring**, e `-o A,B,C,D` contém `-o A,B,C`: a mutação **passou por ele**. O
único teste que a pegou foi o do ensaio em bash, e por acaso — porque a string do
script não bate.

A comparação passou a ser por **igualdade**, sobre a lista extraída da receita, e
há um segundo teste que exercita a extração contra uma receita adulterada — o
guarda do guarda. É a lição de sempre com o sujeito trocado: **um teste que
aceita mais do que devia é um teste que ninguém sabe se funciona**, e a única
forma de descobrir é mutar o código de produção.

### O conselho da colheita ganhou um ramo próprio

O genérico dizia *"o log da operação está em `ARCA-LOGS\sondagem\`"*. Ali há
**um** arquivo com **uma** linha, e ela é a resposta — mandar procurar na pasta
esconderia a resposta a um `cd` de distância. Nas outras três o log tem centenas
de linhas de progresso, e "olhe a pasta" é o melhor que se pode dizer.

### Um achado que ninguém pediu, e ele fala de uma defesa da E9

O `blkdev.list` trouxe o dispositivo como `Maxtor Z1 SSD 480GB`, e o WMI o chama
de `JMicron Generic SCSI Disk Device`. A ponte USB responde ao Windows com o
nome **dela**; o Linux lê o disco atrás dela.

O disco de **origem** casa nas duas fontes, que é o que o backup precisa. O que
não casa é o **dispositivo** — e com isso a segunda barreira de R-8
(`DestinoResolveNoDispositivo`) fica **inerte** aqui: `nome_do_disco` não acha
`JMicron Generic` em `blkdev.list` nenhum.

Ela não falha errado, só não dispara, e a primeira barreira — por letra do
Windows — continua valendo. O [ADR-0015](0015-a-restauracao-so-restaura-no-disco-de-origem.md)
já previa que a segunda viraria redundante. **O que o marco acrescentou foi a
causa**: um canal de identidade que passa por uma ponte não fala do disco. No
dispositivo antigo os dois lados casavam (`KGSSE100 256 SCSI Disk Device` ↔
`KGSSE100256`), e ninguém tinha razão para suspeitar.

---

## Consequências

- `Operacao` tem quatro valores, e `pasta_do_log` produz quatro pastas
  distintas — a quarta sem nome.
- `Estado.nome` é `Option`, com a coerência cobrada no leitor e em
  `Receita::montar`, nos dois sentidos e nos dois eixos.
- `blkdev::Origem` tem duas variantes, e `nome_do_disco` passa a receber
  `Lista { fonte, texto }` em vez de pares anônimos.
- A recusa `SemOraculo` ganha saída, e ela é um comando: `arca sondar`. Até a
  E11 a saída era um backup pelo menu do Clonezilla.
- A tela do `arca prepare` muda pela **terceira** vez, e agora manda sondar. A
  segunda versão foi registrada no plano de etapas junto com a data de validade
  dela, para que não sobrevivesse ao motivo — a primeira quase sobreviveu.
- `recursos/ensaio-da-receita.sh` ganha a quarta receita e cinco casos novos,
  inclusive o que roda a forma com `;` para mostrar o que ela escreveria.
- `src/confirmacao.rs` ganha `perguntar_se_pode`, que saiu do `arca prepare`.
- **P-26 e P-27 fecharam em 24/08/2026**, no mesmo reinício.
- **O boot do Clonezilla isolado passou a ter número**: 1 min 40 s do reinício ao
  desligamento, com ≈ 50 s de boot propriamente dito depois de tirar os 30 s do
  menu e os 20 s do `sleep`. Nenhuma execução anterior deste projeto podia
  medi-lo, e a tela do `arca sondar` continua sem prometê-lo.
- `Ensaio::de_exemplo` virou `Ensaio::origem`, e a linha do ensaio deixou de ter
  frase fixa sobre a fonte do nome.
