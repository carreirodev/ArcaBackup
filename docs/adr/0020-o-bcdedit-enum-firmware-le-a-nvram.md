# O `bcdedit /enum firmware` lê a NVRAM, e quem provou foi o firmware

O [ADR-0012](0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)
abriu P-22 com uma pergunta que até ali não precisava de resposta: **o `bcdedit
/enum firmware` mostra a NVRAM do firmware, ou o BCD do disco?** O que dependia
dela era a linha `Ordem de boot` do `arca status` — uma afirmação de segurança
lida dali — e, desde o [ADR-0013](0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md),
também o conserto de C-13, cuja releitura de C-3 confirmaria sobre o espelho se
a resposta fosse "o BCD".

O experimento rodou em 24/08/2026, custou um reinício, e a resposta não veio de
onde se esperava.

## O que foi medido

Duas leituras do `bcdedit /enum firmware` desta máquina, separadas por um
religar limpo — SSD ARCA conectado, sem job armado, `grub.cfg` inerte conferido
byte a byte contra `grub-inerte-arcaboot.cfg` (`4b33da61…f947aa3d`).

| Quando | Arquivo | `displayorder` | SHA256 |
|---|---|---|---|
| 17:11:50 | `bcdedit-enum-firmware-2026-08-24-antes-do-religar.txt` | `{bootmgr}`, `{f4057bd3}` | `89ca7ad1…7b8df3b9` |
| 17:26:14 | `bcdedit-enum-firmware-2026-08-24-pos-religar.txt` | `{bootmgr}`, `{f4057bd3}`, **`{6cc093db}`, `{6cc093dc}`, `{6cc093dd}`** | `7ba552b5…4f0599a2` |

A máquina foi **direto ao Windows**, que é o desfecho que o ADR-0012 nomeou
como "a NVRAM acompanhou". Mas o que fecha P-22 não é onde ela parou — é o que
apareceu no arquivo.

O `diff` das duas leituras é limpo: **nada removido, nada alterado, três
entradas acrescentadas.**

```text
Aplicativo de Firmware (101fffff)
identificador           {6cc093db-9ff9-11f1-8a4e-806e6f6e6963}
description             UEFI:CD/DVD Drive

Aplicativo de Firmware (101fffff)
identificador           {6cc093dc-9ff9-11f1-8a4e-806e6f6e6963}
description             UEFI:Removable Device

Aplicativo de Firmware (101fffff)
identificador           {6cc093dd-9ff9-11f1-8a4e-806e6f6e6963}
description             UEFI:Network Device
```

## O argumento, e ele é de uma linha

`UEFI:CD/DVD Drive`, `UEFI:Removable Device` e `UEFI:Network Device` são
**classes de dispositivo que o firmware enumera no POST**. Não descrevem
arquivo, partição nem aplicação — elas não têm `device` nem `path`, só
`description`. Nada no BCD as originaria, e o Windows não tem como inventá-las:
não são objetos que descrevam qualquer coisa do disco.

Elas apareceram no `displayorder` do `{fwbootmgr}` num intervalo de quinze
minutos em que o único evento foi um reinício. **Logo o que o `bcdedit /enum
firmware` imprime contém informação que só existe na NVRAM.**

Isto é mais forte do que o experimento pedia. "Parou no Windows" mede o
**efeito** — a ordem que o `bcdedit` mostra prevê onde a máquina boota. As três
entradas medem a **fonte**, que é a pergunta literal de P-22.

## O segundo carimbo: as mesmas três já estiveram lá, com outro nome

As três descrições não são novas neste repositório. Elas estão na captura de
20/08 (`bcdedit-enum-firmware-legado-pt.txt`), no mesmo `displayorder`, com os
mesmos nomes — e com **GUIDs diferentes**:

| | `UEFI:CD/DVD Drive` | `UEFI:Removable Device` | `UEFI:Network Device` |
|---|---|---|---|
| 20/08 | `{c71136d7-9c6a-11f1-8a41-…}` | `{c71136d8-9c6a-…}` | `{c71136d9-9c6a-…}` |
| 24/08 | `{6cc093db-9ff9-11f1-8a4e-…}` | `{6cc093dc-9ff9-…}` | `{6cc093dd-9ff9-…}` |

São UUIDs versão 1 — o `1` de `11f1` é o campo de versão —, e neles o
`time_mid` avançou de `9c6a` para `9ff9`. **Foram geradas de novo**, e não
recuperadas de um cache. É o mesmo sinal que o §3.1 já tinha achado digno de
nota quando o número de uma entrada foi de `0001` para `0003`: recriada, por
ninguém.

## O terceiro carimbo, e ele dá uma regra de leitura

O `node` do UUID separa as duas origens, e o padrão é **total** em todas as
capturas de `bcdedit` deste repositório, sem exceção:

| `node` | O que sempre é |
|---|---|
| `806e6f6e6963` | Entrada `Aplicativo de Firmware (101fffff)` — `{687478f2}` `UEFI OS`, as três de 20/08, as três de 24/08 |
| `aa4ed9bd2b34` | Objeto do BCD — `{f4057bca}` `resumeobject`, `{f4057bd0}` e `{f4057bd3}` `ARCA` |

O `{f4057bd3}` do ARCA nasce de `bcdedit /copy {bootmgr}` (ADR-0017): ele é um
objeto do BCD que o `{fwbootmgr}` referencia. As entradas de firmware puras
recebem GUID sintético de outro gerador. **Quem ler uma captura futura pode
separar as duas coisas sem sair do arquivo**, e isso não estava documentado.

## O que isto fecha

**P-22 fecha, e pelas duas acepções.** A literal — de onde o `bcdedit` lê — está
respondida: a NVRAM. A operacional — se a linha `Ordem de boot` do `arca status`
prevê onde a máquina vai bootar — está respondida junto, e é a que o §7 do
`o-que-falta-para-fechar.md` cobrava.

Cai com ela a dúvida que o ADR-0013 acrescentou: **C-13 conserta o firmware, e
não um espelho dele.** A releitura de C-3 do `/addfirst {bootmgr}` confirma
sobre a coisa que o próximo POST vai obedecer.

E a promessa da tela do `arca prepare` — *"a entrada de firmware existe e está
FORA da ordem permanente — ligar a máquina continua subindo o Windows"* — tem
agora um religar de verdade por trás, com o dispositivo na mesa.

## O que isto corrige no ADR-0012, e a correção é de mecanismo

O ADR-0012 mediu que a `{687478f2}` `UEFI OS` **sumiu inteira** do `bcdedit`
depois da restauração de 23/08, e explicou o par com "o Windows a espelhou ao
subir". Aquela frase foi escrita quando não se sabia de onde o `bcdedit` lê, e
ela carregava a hipótese de um espelhamento NVRAM→BCD que ninguém mediu.

Com P-22 fechada, o sumiço é da **NVRAM**, e há candidato melhor e medido: o
firmware **reconstrói entradas a cada POST** — foi o que ele acabou de fazer com
as três `UEFI:*` —, e uma entrada apontando para um dispositivo que não está
mais conectado é podada nessa reconstrução. A `{687478f2}` apontava para
`partition=R:`, o dispositivo ARCA.

**Isto é hipótese, e fica nomeada como tal.** O que está medido é que o firmware
recria entradas em POST; que ele *poda* as ausentes não foi medido, e o
experimento que fecharia é outro. As medições do ADR-0012 continuam todas de pé
— é o mecanismo proposto para uma delas que muda.

> **A hipótese ficou meia medida às 18:47 do mesmo dia.** No boot do experimento
> de P-28, o firmware **removeu as três `UEFI:*`** da ordem e da enumeração — o
> arquivo voltou byte a byte ao das 17:11 —, com o dispositivo ainda conectado.
> **Podar ele poda.** O que continua sem medição é a poda que este ADR propôs: a
> de uma entrada cujo dispositivo não está mais lá. Ver
> [ADR-0021](0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).

## O que isto abre

**Um religar limpo suja a ordem permanente**, e isso não estava medido. Não com
o dispositivo à frente — o Windows continua em primeiro —, mas a ordem foi de
duas para cinco entradas, e o `arca status` passou de `dispositivo em 2o de 2`
para `dispositivo em 2o de 5`.

E **as três vão e vêm sem causa conhecida**: estavam em 20/08, não estavam em
22/08 de manhã nem às 17:11 de hoje, voltaram às 17:26 — **e foram embora às
18:47**, no boot do experimento de P-28. Dois boots pelo dispositivo, em 24/08,
não as trouxeram. Não há pendência aqui porque nenhuma tela do ARCA afirma nada
que dependa da resposta — é curiosidade sobre este firmware, registrada para
quem for comparar contagens de entradas entre capturas e achar que alguém mexeu.

**O que é pendência é P-28**, e ela nasceu desta leitura.

### P-28 — uma entrada da ordem que não diz para onde aponta

`alcanca_o_arcaboot` (`src/comandos/status.rs`) devolve `false` quando
`entrada.alvo` é `None`, e as três `UEFI:*` não têm `device` nenhum: o `bcdedit`
imprime delas só `identificador` e `description`. O ARCA as lê como **não levam
ao dispositivo**, e essa é a resposta tranquilizadora.

`UEFI:Removable Device` é a classe que boota o primeiro dispositivo removível, e
o `ARCABOOT` é um SSD USB removível.

**O modo de falha não é a tela mentir sobre o nome — é ela engolir o aviso.**
Com aquela entrada em primeiro e a `{f4057bd3}` `ARCA` em segundo, `posicao`
vale 1, o ramo `posicao > 0` sai, e a linha fica:

```text
Ordem de boot ... dispositivo em 2o de 5 · `UEFI:Removable Device` vem antes
```

Está correta ao pé da letra: aquela entrada **está** antes. O que não sai é o
parágrafo de perigo — *"Enquanto o SSD estiver conectado, a maquina boota nele
sem boot unico nenhum"* —, porque ele mora só no ramo `posicao == 0`. Quem abriu
a tela para saber se pode religar com o SSD na mesa lê *"vem antes"*, não recebe
aviso nenhum, e entende que está seguro.

**É a terceira forma da mesma falha.** O ADR-0009 já a pegou uma vez — a versão
que procurava a entrada *pelo nome* em vez do alvo, e cuja consequência foi
descrita ali com estas palavras: *"aquela versão diria 'o Windows vem antes' e
engoliria o aviso"*. C-6 a pegou noutra. Aqui não é nome errado nem alvo errado:
é a **ausência** de alvo virando segurança, que é exatamente o que
`viu_o_gerenciador` existe para não deixar acontecer no bloco vizinho.

**Não é urgente e a razão é medida**: as três estão em 3º, 4º e 5º, atrás do
Windows, e o que decide o boot é a primeira. O que falta saber antes de
escrever código é se `UEFI:Removable Device` de fato alcança o `ARCABOOT` nesta
máquina, e isso custa um F12 escolhendo aquela linha em vez da entrada `ARCA`.

> **Esta última frase estava errada, e o [ADR-0021](0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md)
> a corrige no mesmo dia.** A regra que faltava — *uma entrada que não diz para
> onde aponta não é segurança* — não afirma nada sobre este firmware: ela deixa
> de afirmar, e é a forma de `viu_o_gerenciador`. Foi escrita sem o F12, que
> continua valendo para calibrar a dureza do texto.
>
> E a leitura em duplo achou **um terceiro ramo**, que este ADR não tinha visto:
> com a entrada `ARCA` fora da ordem — o estado que o `arca prepare` deixa — a
> tela não engolia um aviso, ela **afirmava**: `so o boot unico leva a ele`.

## Consequências

- P-22 sai do §3.5 e do §12 do PRD como fechada, e sai do §1 e do §7 do
  `o-que-falta-para-fechar.md`.
- **P-28 entra no §3.5**, com o experimento nomeado.
- O ADR-0012 ganha nota: o mecanismo do sumiço da `{687478f2}` muda de "o
  Windows espelhou" para "o firmware reconstruiu", e a segunda é hipótese.
- O ADR-0013 ganha nota: a dúvida que ele levantou sobre C-13 está respondida
  pelo lado bom.
- Quatro capturas novas em `recursos/capturas/`, com o par de `bcdedit` e o par
  de `arca status` do mesmo reinício.
- O §11 ganha o `node` do UUID como regra de leitura: numa captura de
  `bcdedit`, ele separa o que veio do firmware do que veio do BCD.
- **P-28 vira código no mesmo dia**, e não depois do F12:
  [ADR-0021](0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md). A captura
  `bcdedit-enum-firmware-2026-08-24-pos-religar.txt` deste ADR entra na suíte,
  onde nenhum teste a lia.

> **A expectativa que entrou no experimento estava errada, e vale registrado.**
> A análise que precedeu este reinício apostava no menu do Clonezilla, e o
> argumento era a `{687478f2}` ter sumido numa restauração que rodou com
> `-iefi` — uma entrada da NVRAM não some por causa de uma restauração de
> disco. O que faltava àquele raciocínio era saber que o firmware reconstrói
> entradas em POST, que é o que este mesmo experimento mediu.
>
> É o padrão que o §11 já nomeia por outro caminho: **rodar acha o que reler
> não acha.** Desta vez o que se releu foram capturas, e a evidência de arquivo
> apontava para o lado errado porque nenhum par dela era apertado o bastante —
> todos tinham um boot no meio, e o boot é justamente o que mexe.
