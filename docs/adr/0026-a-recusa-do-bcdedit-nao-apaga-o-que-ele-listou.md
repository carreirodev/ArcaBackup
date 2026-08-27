# A recusa do `bcdedit` não apaga o que ele listou — e o `prepare` lê o firmware antes de apagar

Decidido em 27/08/2026, no mesmo dia da medição. Nasce C-15 e PR-6.

## O que aconteceu

Às 19:03 o `arca prepare` rodou por cima de um dispositivo ARCA existente — o
KGSSE100 preparado em 26/08 — e morreu às 19:07 na linha seguinte a
*"Instalando o ARCA em ARCABOOT … ok"*:

```text
erro: bcdedit recusou (codigo 1): Gerenciador de Inicialização de Firmware
----------------------------------------
identificador           {fwbootmgr}
displayorder            {bootmgr}
                        {d13126af-a1f1-11f1-bb13-806e6f6e6963}
                        {ffe63caa-a25d-11f1-bb16-806e6f6e6963}
                        {ffe63cab-a25d-11f1-bb16-806e6f6e6963}
                        {ffe63cac-a25d-11f1-bb16-806e6f6e6963}
timeout                 1
Foi especificado um dispositivo inexistente.
```

Cinco `prepare` seguintes morreram no mesmo lugar, e um `arca sondar` às
19:30 morreu em **27 ms**, na primeira leitura — antes de tocar em qualquer
coisa. Não era o passo 11: era o `bcdedit`.

## O que foi medido, e a ordem importa

Numa janela elevada, **todo** `bcdedit /enum` desta máquina — `{fwbootmgr}`,
`firmware`, `all /v`, `{bootmgr}`, e o de cada uma das quatro `UEFI:*`, que não
têm `device` nenhum — imprimia a listagem **inteira** e terminava com *"Foi
especificado um dispositivo inexistente."*, código 1. A mensagem vem depois
da última linha, em todos os alvos: a recusa é sobre o estado do repositório,
não sobre o que foi pedido.

Na listagem, uma coisa fora do lugar:

```text
identificador           {8a1c6901-a179-11f1-be2c-cbfb5c43df57}
device                  unknown
path                    \EFI\boot\bootx64.efi
description             ARCA
```

E a NVRAM, lida direto pelas variáveis `Boot####` (com
`SeSystemEnvironmentPrivilege`, sem `bcdedit` no meio):

```text
Boot0000 "ARCA" → HD(part=2, sig=a022ea07-307a-415f-813a-9c47192360ec) / \EFI\boot\bootx64.efi
```

`a022ea07-…` era o GUID da `ARCABOOT` do layout **anterior** — o que o passo 5
das 19:03 apagou. A partição nova, `E:`, nasceu `a75737e3-…`. A entrada
apontava para o nada, e era o próprio `prepare` que a tinha deixado assim.

O conserto, feito à mão às 19:52 sobre o estado quebrado:

```text
> bcdedit /set {8a1c6901-…} device partition=E:
A operação foi concluída com êxito.
> bcdedit /enum {fwbootmgr}
… (a mesma listagem, sem a mensagem)
código 0
```

Relido: `Boot0000` → `HD(part=2, sig=a75737e3-…)`, `device partition=E:`,
e todo `/enum` de volta ao código 0. As duas leituras quebradas estão em
`recursos/capturas/bcdedit-enum-{fwbootmgr,firmware}-2026-08-27-dispositivo-inexistente.txt`.

## Por que o ARCA inteiro parou

O adaptador do `bcdedit` trata código diferente de zero como recusa, e a
razão dele está escrita e continua certa: sem privilégio, o `bcdedit`
escreve *"Acesso negado"* **na saída padrão** e sai com 1 — quem lesse só o
texto concluiria que não há entrada `ARCA` onde não houve permissão para
olhar, e criaria uma duplicata.

Mas todo comando que fala com o firmware **começa lendo** — o `{fwbootmgr}`
antes de escrever, para C-5 ter contra o que comparar —, e a leitura é a
mesma chamada que a recusa. Com a entrada pendurada, `prepare`, `sondar`,
`backup`, `restore`, `verify`, `status` e `desarmar` morriam na primeira
linha. **E o comando que conserta o estado — `/set device` — é o que o
próprio `prepare` executaria três linhas depois da leitura que o recusava.**

## As duas decisões

### C-15 — a recusa do `bcdedit` não apaga o que ele listou

Uma resposta que traz o gerenciador de firmware (ou, num `/enum {guid}`, a
entrada pedida) é uma **leitura**, qualquer que seja o código. O código fica
guardado na leitura — `Leitura::codigo_da_recusa` — como informação a mais,
e nunca vira "não li". Uma resposta que não traz nada continua sendo a
recusa que era, com o texto do `bcdedit` inteiro: o *"Acesso negado"*
continua recusa.

É `firmware::enumerar`, código puro sobre a porta, e é por ele que passam
todas as leituras do firmware desde este dia. O discriminante já existia:
`viu_o_gerenciador`, a mesma guarda que o desarmar usa para não confundir
"não entendi a resposta" com "desarmou". C-15 é a forma dela aplicada ao
código de saída.

**O que C-15 não faz**: decidir por ninguém se a leitura basta para
**escrever**. Reusar o que se leu é o que C-3 sempre mandou; criar é outra
coisa.

### PR-6 — o `prepare` lê o firmware antes de apagar, e o `device` é a primeira escrita depois

A ordem permanente e a entrada a reusar são lidas **antes do passo 5**, com o
firmware ainda coerente — uma chamada só, ao `firmware`, que traz o
gerenciador e as entradas juntos. Dali saem três coisas:

- **o plano diz qual dos dois vai acontecer** — *reapontada* ou *criada* —,
  em vez de prometer "criada" sobre uma entrada que existe;
- **C-4 decide ali se pode haver `/copy`**: uma leitura que veio com código é
  aceita para reusar — a entrada está na listagem, com identificador — e
  recusada para criar, porque um `/copy` sobre uma listagem que o `bcdedit`
  disse ter problema é apostar que ela está completa, e a aposta errada é
  uma segunda entrada. A recusa (`EntradaNaoNasceDeLeituraRecusada`)
  acontece **antes do plano**, com o disco intacto;
- **sem conseguir ler o firmware, o `prepare` não apaga nada.** Antes, um
  `bcdedit` que recusasse era descoberto no passo 11, com o disco já apagado.

E no passo 11 o primeiro `bcdedit` depois do apagar é o **`/set device`**
para o `ARCABOOT` novo — antes da descrição, antes do `path`, antes de
qualquer releitura. É o comando medido como o que devolve o código 0. As
releituras de C-3 continuam as mesmas, e continuam sendo o que responde.

## O que isto não explica, e fica nomeado

**Em 26/08 o mesmo cenário passou.** O `prepare` das 18:17 apagou o
dispositivo preparado às 18:14, a entrada ficou pendurada do mesmo jeito, e
o passo 11 reapontou sem que nenhum `/enum` saísse com código. O que difere
em 27/08, datado pelos GUIDs v1 do BCD: às **18:26** o firmware acrescentou
as três `UEFI:*` (CD/DVD, Removable, Network) à ordem, e às **05:32** uma
`UEFI:  USB, Partition 1` apontando para um pendrive MBR que não estava
conectado. A hipótese é que a sincronização NVRAM↔BCD que o `bcdedit` faz ao
abrir o repositório só tropeça na entrada pendurada quando há mais alguma
coisa para sincronizar. **Não foi medido**, é P-29, e C-15 e PR-6 foram
escritos para não depender da resposta: qualquer que seja o gatilho, a
listagem veio inteira e o `/set device` conserta.

**A forma medida da recusa é a mensagem depois da listagem completa.** A
leitura não tem como saber se um `bcdedit` futuro parar no meio; é por isso
que a guarda é o gerenciador, que sai primeiro, e por isso que nenhuma
leitura com código cria entrada.

## Consequências

- **C-15 e PR-6 entram no PRD e no README §13.** `FerramentaRecusou` do
  `bcdedit` passa a significar "não listou"; o código de uma listagem que
  veio inteira mora em `Leitura::codigo_da_recusa`.
- O `arca prepare` ganha o passo **1b**, e o plano passa a dizer *reapontada*
  quando a entrada existe. O erro novo, `EntradaNaoNasceDeLeituraRecusada`,
  só é alcançável antes do ponto sem volta.
- `FirmwareDeMentira` ganha `listando_e_recusando_no_fim`, e `prepare`,
  `armar` e `desarme` têm cada um o teste do `bcdedit` de 27/08.
- Duas capturas novas em `recursos/capturas/`, e a proveniência delas diz
  por qual caminho os bytes passaram.
- **P-29 abre** no §3.5 do PRD.
- O que **não** muda: C-3 continua inteiro — o sucesso do `bcdedit` nunca é
  prova, e cada escrita continua conferida por releitura. C-15 é a outra
  metade da mesma frase: a recusa dele também não é.
