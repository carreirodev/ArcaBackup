# Colher devolve o `{bootmgr}` ao topo da ordem permanente

**Supersede a decisão do [ADR-0009](0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)**
— *"o ARCA avisa, e não conserta"*. As medições daquele ADR continuam todas de
pé; o que muda é o que se faz com elas.

O ADR-0009 mediu que o ciclo de boot pelo dispositivo põe a entrada dele na
ordem permanente, registrou a fricção que isso causa, e escolheu avisar. No
mesmo dia recebeu pedido de revisão (P-20), e o próprio ADR deixou escrito o
que faltava: *"se a revisão passar, ela vem como ADR novo que supersede este —
e com a forma do comando medida à mão antes de virar código, como a E7 fez com
o `bootsequence`."*

Foi o que aconteceu, em 23/08/2026.

## O requisito, na forma em que ele foi pedido

> *"Depois de retornar do backup (ou da restauração) eu posso ligar o
> computador e voltar ao Windows SEMPRE, mesmo com qualquer SSD conectado."*

E, na mesma conversa, o limite dele:

> *"Depois do boot inicial após um backup/restauração eu não me incomodo de ter
> que retirar o SSD. Mas depois disso, eu me incomodo, pois não é assim que
> funcionava antes de iniciar esse app."*

Isso é mais estreito do que parece, e o recorte é o que torna a solução
pequena. **C-9 continua inteiro**: remover o SSD antes de religar segue sendo o
que a tela pede logo depois de armar, e é a defesa contra a janela em que o
`grub.cfg` está armado. O que se pede é o estado **permanente**, dali em
diante — e o `arca resultado` é onde ele se conserta, porque é o único comando
que se roda depois de uma operação, e roda depois do boot que sujou a ordem.

## O que separa isto de violar C-5, e a assimetria é real

C-5 diz *"boot único — nunca alterar a ordem permanente"*, e foi escrito contra
um perigo nomeado no §3.1 e em `src/armar.rs`: o ARCA **acrescentar** um
caminho permanente para o dispositivo. Desfeito o job, a máquina continuaria
com um caminho a mais para bootar no Clonezilla, e ninguém teria pedido isso.

`/addfirst {bootmgr}` não acrescenta caminho nenhum. Ele põe o Windows na
frente dos caminhos que já existem, e **não remove nada**. A ordem depois do
conserto contém exatamente as mesmas entradas de antes; o que muda é quem está
em primeiro.

C-5 continua valendo inteiro onde foi escrito para valer — no armar e no
desarme, que releem a ordem e falham se ela mudou. O que ele ganha é um limite
explícito: **ele fala das operações que armam.** O conserto vira C-13, um
requisito próprio, para que ninguém precise ler uma exceção dentro de uma
proibição.

## O que foi medido à mão, antes de virar código

Em 23/08/2026, com a ordem sujada e desfeita de propósito, e a NVRAM conferida
byte a byte contra o estado inicial no fim:

```text
/set {fwbootmgr} displayorder {ARCA}    /addfirst  → exit 0 · ARCA ao topo
/set {fwbootmgr} displayorder {bootmgr} /addfirst  → exit 0 · Windows ao topo, ARCA em segundo
/set {fwbootmgr} displayorder {bootmgr} /addfirst  → exit 0 · nada muda (idempotente)
/set {fwbootmgr} displayorder {ARCA}    /remove    → exit 0 · sai da ordem, o objeto sobrevive
```

Três coisas saem daí:

1. **`/addfirst` move, e não duplica.** O help do `bcdedit` diz o mesmo: *"se o
   identificador especificado já estiver na lista, ele será movido para o topo
   da lista"*. Idempotência de graça, e a segunda passada não é caso especial.
2. **`/remove` tira da ordem sem apagar o objeto.** A entrada `ARCA` continua
   existindo depois dele — o que o boot único precisa que continue.
3. **Os quatro respondem *"A operação foi concluída com êxito"* e saem com
   código 0**, inclusive o que não muda nada. É o texto em que este projeto não
   confia desde a E2, e é por isso que quem responde é a releitura de C-3 sobre
   a pós-condição: *o primeiro da ordem é o `{bootmgr}`?*

## `/addfirst`, e não `/remove` — e o motivo não é o óbvio

`/remove` faria a ordem voltar **literalmente** ao que era antes de o ARCA
existir, que é o que o pedido descreve. Ficou de fora assim mesmo.

A razão é o modo de falha. `/remove` precisa acertar **quais** entradas tirar,
e *"quais levam ao dispositivo"* é uma pergunta que esta máquina já respondeu
errado uma vez: a revisão do marco da E8 pegou a linha `Ordem de boot` do
`arca status` procurando pela entrada **chamada** `ARCA`, enquanto quem levava
ao dispositivo — e por onde a máquina de fato bootou em 22/08 — era a
`{687478f2}` `UEFI OS`, que o firmware criou e que nome nenhum encontra.

`/addfirst {bootmgr}` não faz essa pergunta. O alvo é um identificador fixo, o
resultado vale para **todas** as entradas do dispositivo de uma vez — inclusive
as que o firmware ainda não criou —, e uma escrita com alvo constante tem menos
como errar do que N escritas com alvos deduzidos. Numa NVRAM de boot, onde um
erro deixa a máquina sem bootar, essa diferença vale mais do que a limpeza.

## Onde o conserto acontece, e por que nos três caminhos

No `arca resultado`, **depois** do desarme. A ordem importa: o desarme relê a
ordem permanente para conferir que ele próprio não a tocou (C-5), e consertá-la
antes faria aquela conferência correr sobre um valor que o mesmo comando acabou
de mudar — a checagem que existe para pegar um `bcdedit` que mexeu no que não
devia passaria a comparar duas coisas nossas.

E acontece nos **três** caminhos do comando — colheu, não havia job, já estava
colhido. **C-13 não fala de job.** A ordem permanente está suja ou não está,
tenha alguém armado alguma coisa ou não; recusar-se a arrumá-la porque o
desfecho já foi lido deixaria a máquina bootando no dispositivo por um motivo
que não tem nada que ver. É a diferença para o desarmar, que a E8 manteve fora
desses caminhos com razão: desarmar desfaz uma **intenção do ARCA**, e sem job
não houve intenção nenhuma.

## A saída diz as duas coisas em linhas separadas

A E8 registrou que misturar *"colhi"* com *"arrumei"* tira de quem lê a
informação de qual das duas aconteceu. São duas linhas:

```text
  Desarmando SSD .................. ok · R:\boot\grub\grub.cfg
  Ordem de boot ................... devolvida · o Windows voltou ao topo, na frente de ARCA · {f4057bd0-…}
```

O rótulo é o mesmo que o `arca status` usa desde o ADR-0009, e de propósito: é
a mesma coisa, e quem viu o aviso lá tem de reconhecer o conserto aqui.

A linha existe sempre; o que ela **diz** muda. Com o Windows já em primeiro ela
sai `ok · o Windows ja era o primeiro`, e o parágrafo de conselho não aparece —
um `ok` sobre ação que não aconteceu é a mentira que este projeto já contou
duas vezes (§11), e a E4 pegou a versão dela no desarmar.

## O que a execução real pegou, e os testes não pegavam

Rodado o comando de verdade, com a ordem sujada à mão, **dois defeitos
apareceram numa versão que tinha suíte verde**:

- **A linha saía com o GUID onde promete um nome.** O código lia
  `/enum {fwbootmgr}`, como `src/desarme.rs` faz — e aquele alvo devolve o bloco
  do gerenciador **sozinho**, sem as entradas. A ordem vinha certa, a descrição
  nunca era encontrada, e a tela dizia
  `na frente de {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}`. Quem abriu a tela
  querendo saber se pode religar com o SSD na mesa recebia um GUID.
  A raiz estava no duplo: **ele respondia a mesma coisa aos dois alvos**, e o
  `bcdedit` não os junta. Corrigido nos dois lugares, e agora há teste que
  falharia.
- **O conselho não saía no caminho `já colhido`.** Eu tinha posto a linha nos
  três caminhos e o parágrafo em dois. O teste que devia pegar cobrava só a
  linha — o caso fácil do que ele existia para cobrir, que é a lição da revisão
  da E4 pela terceira vez.

Os dois são o padrão que este projeto já nomeou: **rodar o comando de verdade
acha o que reler o código não acha.** Foi assim na E6, na E7 e na E9.

## Consequências

- **C-5 ganha limite explícito** — fala do armar e do desarme — e **C-13
  nasce**: ao colher, o `{bootmgr}` volta ao topo da ordem permanente.
- **P-20 fecha.** O pedido era exatamente isto.
- `src/ordem.rs` é módulo novo, com 11 testes; `src/comandos/resultado.rs`
  ganha a linha e o conselho nos três caminhos.
- O `arca status` continua **avisando** e não consertando: ele é diagnóstico, e
  um comando de consulta que escreve na NVRAM seria outra coisa. Quem conserta
  é quem já estava escrevendo.
- ~~**P-22 continua aberta, e este ADR aumenta o que ela vale.** Se o `bcdedit`
  mostrar o BCD e não a NVRAM, a releitura de C-3 aqui confirma um conserto
  sobre o espelho, e a máquina continuaria bootando no dispositivo. O
  experimento é o mesmo e continua custando um reinício: religar com o SSD
  conectado, sem job armado e com o `grub.cfg` inerte.~~
  **Fechada em 24/08/2026, e pelo lado bom: é a NVRAM.** O experimento foi o
  que está escrito aí, e o que o fechou não foi onde a máquina parou — foram
  três entradas que o firmware acrescentou ao `displayorder` no meio do
  reinício, e que nada no BCD originaria. **C-13 conserta o firmware, e não um
  espelho dele**: a releitura de C-3 do `/addfirst {bootmgr}` confirma sobre a
  coisa que o próximo POST vai obedecer. Ver
  [ADR-0020](0020-o-bcdedit-enum-firmware-le-a-nvram.md).
