# A entrada de firmware nasce de uma cópia do `{bootmgr}`, e sai da ordem permanente

Decidido em 23/08/2026, na etapa E10.

## O que estava em aberto, e há quanto tempo

C-4 diz que **armar não cria entrada de firmware**: procura a `ARCA`, não
havendo migra a legada `Clonezilla`, e não havendo nenhuma das duas **recusa**.
A E7 escreveu essa recusa de propósito e disse por quê:

> *Criar uma entrada de firmware do zero é código sem original — nenhuma
> captura mostra a forma —, e o lugar disso é o `arca prepare` da E10. Armar
> não é a hora de estrear a criação de entrada de boot.*

O `arca prepare` é essa hora. E a primeira coisa a fazer era procurar o
original, como o método deste projeto manda — porque cinco vezes ele não estava
lá, e uma vez estava.

## Desta vez ele estava, e a resposta é o próprio `bcdedit`

A entrada `ARCA` desta máquina, que boota o dispositivo desde 19/08, tem
**doze** campos. Nove deles não têm nada que ver com o Clonezilla:

```text
identificador           {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
device                  partition=R:
path                    \EFI\boot\bootx64.efi
description             ARCA
locale                  pt-BR
inherit                 {globalsettings}
flightsigning           Yes
default                 {current}
resumeobject            {f4057bca-65a4-11f1-b0f1-aa4ed9bd2b34}
displayorder            {current}
toolsdisplayorder       {memdiag}
timeout                 30
```

`resumeobject`, `toolsdisplayorder {memdiag}`, `flightsigning` — isso é o
gerenciador de boot do Windows. `src/firmware.rs` já tinha notado a
consequência disso e a usava para outra coisa: *"o título do bloco não
distingue a entrada do ARCA da do Windows: as duas aparecem como `Windows Boot
Manager`, porque a do ARCA nasceu de um `bcdedit /copy`"*.

Nasceu de um `/copy` do `{bootmgr}`. **A entrada é o original da criação dela
mesma**, e o que faltava era medir o comando que a produz.

## O que foi medido, e a entrada foi apagada no fim

Em 23/08/2026, com o firmware lido antes e depois e conferido byte a byte:

```text
> bcdedit /copy {bootmgr} /d ARCA-MEDICAO-E10
A entrada foi copiada com sucesso para {f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}.
                                                              (código 0)

> bcdedit /enum {f4057bd1-…}                     ← recém-criada, crua
device                  partition=\Device\HarddiskVolume1
path                    \EFI\Microsoft\Boot\bootmgfw.efi
description             ARCA-MEDICAO-E10
  … os nove outros campos, idênticos aos da entrada ARCA

> bcdedit /set {f4057bd1-…} device partition=R:              (código 0)
> bcdedit /set {f4057bd1-…} path \EFI\boot\bootx64.efi       (código 0)

> bcdedit /enum {f4057bd1-…}                     ← relida (C-3)
  … idêntica à entrada ARCA, campo a campo, menos identificador e description

> bcdedit /delete {f4057bd1-…} /f                            (código 0)
```

Quatro coisas saem daí:

**`/copy` responde com o identificador novo, e a frase é traduzida.** *"A
entrada foi copiada com sucesso para {…}"* — o mesmo caso do `chkdsk` de B-6 e
do `certutil` de V-1. O identificador se acha **pela forma**: trinta e seis
caracteres entre chaves, hexadecimais e hifens. Nunca pela posição nem pelo
texto. Havendo mais de um, isto recusa em vez de escolher o primeiro — mesmo
raciocínio do selo repetido e do `ResumoAmbiguo`, e aqui a escolha errada
apontaria o boot da máquina para outro lugar.

**`/copy` sozinho não basta.** A entrada nasce apontando para o Windows —
`\Device\HarddiskVolume1` e `bootmgfw.efi`, herdados do `{bootmgr}` — e são os
dois `/set` que a levam ao dispositivo. Nesse estado intermediário ela é
inofensiva: bootar por ela sobe o Windows.

**A entrada criada sai idêntica à que já existia.** Conferido lado a lado, na
mesma sessão: os doze campos batem, a menos do `identificador` (diferente por
definição) e da `description` (que é o que se pediu no `/d`). Isso é o que
transforma "código sem original" em transcrição.

**O firmware voltou ao que era.** A entrada de medição foi apagada e o `/enum
firmware` do fim é o mesmo do começo.

## O achado que ninguém tinha previsto

**`bcdedit /copy` põe a entrada nova no `displayorder` sozinho.** Medido duas
vezes, nas duas metades da medição:

```text
antes do /copy   displayorder  {bootmgr}
depois do /copy  displayorder  {bootmgr}
                               {f4057bd1-65a4-11f1-b0f1-aa4ed9bd2b34}
```

Ninguém pediu. O `/copy` faz.

**Isso é exatamente o perigo que C-5 nomeia.** O §3.1, o ADR-0007 e o
`src/armar.rs` dizem a mesma coisa com palavras diferentes: o que C-5 proíbe é
o ARCA **acrescentar um caminho permanente para bootar no dispositivo** —
*"desfeito o job, a máquina continuaria com um caminho a mais"*. Um `arca
prepare` que deixasse a entrada na ordem faria isso, e o usuário descobriria
num religar qualquer.

## A decisão: tirar da ordem, com alvo que o próprio comando acabou de criar

`/set {fwbootmgr} displayorder {novo} /remove`, logo depois de criar, com
releitura de C-3 sobre a pós-condição: *a entrada está fora da ordem?*

Medido: código 0, a entrada **sai da ordem e o objeto sobrevive**, e a segunda
passada não muda nada. O objeto sobreviver é o que importa — é ele que o boot
único precisa que continue existindo.

**Tirar não quebra o armar, e isso está medido desde a E7.** O `bcdedit` aceita
`bootsequence` para uma entrada que não está no `displayorder` (ADR-0007), e o
marco de 22/08 rodou exatamente assim: `BootCurrent: 0001` com `BootOrder:
0000,0001`. Não há troca a fazer entre estar fora da ordem e ser bootável por
boot único.

### E não é o `/remove` que o ADR-0013 descartou

O [ADR-0013](0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md) considerou
`/remove` para consertar a ordem ao colher, e o descartou pelo modo de falha:

> *Ele precisa acertar **quais** entradas tirar, e "quais levam ao dispositivo"
> é uma pergunta que esta máquina já respondeu errado uma vez.*

Aqui essa pergunta não existe. O alvo é a entrada que **o próprio comando
acabou de criar**, com o identificador em mãos, na mesma execução, sem
dedução nenhuma. O que tornava `/remove` perigoso lá é precisamente o que não
se aplica aqui — e o ADR-0013 nomeou o perigo com clareza suficiente para que
dê para saber disso.

**E há uma segunda barreira**, no espírito de C-5: depois do `/remove`, o
comando confere que **nenhuma outra entrada sumiu da ordem**. Tirar uma entrada
que não é do ARCA seria desfazer uma decisão de outro dono, no lugar onde um
erro deixa a máquina sem bootar.

## Reusar, e não criar uma segunda

A primeira coisa que `criar_a_entrada` faz é procurar. Havendo `ARCA` ou a
legada `Clonezilla`, ele **reusa** — é C-4 na letra, e pelo mesmo motivo: duas
entradas seriam duas formas de bootar no Clonezilla, uma delas sem ninguém
olhando.

Isso tem uma consequência que a tela precisa dizer, e ela apareceu **no marco**:
com dois dispositivos na gaveta, a entrada deixa de apontar para o anterior e
passa a apontar para o recém-preparado. Não é perda — o `arca backup` reescreve
o `device` a cada armar e relê (C-6) —, mas quem lê a tela merece saber o que
mudou em vez de descobrir num F12.

E é o que torna `arca prepare` rodável duas vezes sem sujar o firmware: a mesma
idempotência que o desarmar ganhou de graça no
[ADR-0005](0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md).

## O que rodou em hardware, e por que foram duas execuções

**O primeiro `arca prepare` reusou a entrada**, porque esta máquina já tinha
uma. Com isso, o caminho da criação — `/copy`, o identificador achado pela
forma, o `/remove` sobre uma entrada recém-nascida — **não foi exercitado pelo
código**, só pela medição à mão.

Por isso houve uma segunda: a entrada `ARCA` foi apagada e o comando rodou de
novo. Ele criou a `{f4057bd3-…}`, apontou-a para `partition=F:`, tirou-a da
ordem, e o `bcdedit` lido depois mostra o `displayorder` com **só o
`{bootmgr}`**. Os dois originais estão em `recursos/capturas/`.

> **Vale registrar o que essa segunda execução é**: não é zelo, é a diferença
> entre *"o comando funcionou"* e *"o caminho que a etapa existe para escrever
> funcionou"*. Um marco que só exercita o ramo fácil é o que o §11 chama de
> caso construído mais fácil do que o real.

## Consequências

- **C-4 ganha a outra metade.** Ele continua dizendo que armar não cria
  entrada; o que muda é que agora há quem crie, e a criação tem original.
- **C-5 ganha uma segunda aplicação, e ela é do `arca prepare`.** A do ADR-0013
  é sobre colher; esta é sobre nascer. As duas escrevem no `displayorder` e
  nenhuma acrescenta caminho para o dispositivo — uma põe o Windows na frente,
  a outra tira a entrada nova de lá.
- **A recusa `SemEntradaDeFirmware` do armar continua valendo**, e agora com
  saída: quem cai nela roda `arca prepare`.
- O que **não** muda: `arca status`, `arca backup` e `arca resultado` continuam
  sem criar entrada nenhuma.
