# Uma entrada sem alvo na ordem não é segurança

O [ADR-0020](0020-o-bcdedit-enum-firmware-le-a-nvram.md) fechou P-22 e abriu
**P-28** no mesmo parágrafo: as três entradas que o firmware acrescenta ao
`displayorder` no POST — `UEFI:CD/DVD Drive`, `UEFI:Removable Device`,
`UEFI:Network Device` — **não declaram alvo**, e o ARCA lia a ausência de alvo
como *não leva ao dispositivo*, que é a resposta tranquilizadora.

Aquele ADR terminou dizendo que *"o que falta saber antes de escrever código é
se `UEFI:Removable Device` de fato alcança o `ARCABOOT` nesta máquina"*.

**Este ADR discorda dessa frase, e escreve o código antes.** O motivo é curto:
a regra que faltava não afirma nada sobre este firmware — ela **deixa de
afirmar**, e é a mesma forma da guarda de `viu_o_gerenciador`. O que o F12
responde calibra o texto da tela, não a decisão de ter a regra.

## O que foi medido, e são três telas

Em 24/08/2026, com a captura real do religar
(`bcdedit-enum-firmware-2026-08-24-pos-religar.txt`) reordenada e passada pelo
`montar` de verdade — não é leitura de código, é a tela:

| | A ordem | O que a linha `Ordem de boot` dizia | Aviso |
|---|---|---|---|
| **A** | como está hoje | `dispositivo em 2o de 5 · Windows Boot Manager vem antes` | não sai, e está certo |
| **B** | `UEFI:Removable Device` no topo | `dispositivo em 3o de 5 · UEFI:Removable Device vem antes` | **não sai** |
| **C** | entrada `ARCA` fora da ordem | `4 entrada(s), nenhuma para o dispositivo · so o boot unico leva a ele` | **não sai** |

O **B** é P-28 como o ADR-0020 a descreveu, confirmada ao pé da letra. Ele
previu `2o de 5` e saiu `3o de 5` porque aqui o `{bootmgr}` ficou onde estava,
entre as duas — a forma é a mesma, e o que importa nela é o que **não** vem
depois.

## O achado: existe um terceiro ramo, e ele afirma em vez de calar

O **C** não estava em lugar nenhum — nem no ADR-0020, nem no §3.5 do PRD, nem
no `o-que-falta-para-fechar.md` —, e é o pior dos dois:

- No **B** a tela **omite** um aviso. É o que o §7 daquele documento classifica
  como *"não é uma afirmação errada — é um aviso que deixaria de sair"*.
- No **C** a tela **afirma**: `so o boot unico leva a ele`. Se
  `UEFI:Removable Device` alcança o `ARCABOOT`, essa frase é falsa, e não
  incompleta.

**E o estado que produz o C é o mais banal da lista:** é o que o `arca prepare`
deixa — a entrada de firmware fora da ordem permanente — mais um religar, que é
justamente o evento que traz as três. Foi o estado desta máquina às 14:56 de
24/08, a menos das três.

## A decisão

O julgamento de alcance passa a ter **três estados**, e não dois:

```rust
pub enum Alcance {
    Leva,       // o alvo é o ARCABOOT desta mesa
    NaoLeva,    // dá para conferir que é outra coisa
    NaoSeSabe,  // a entrada não diz para onde aponta
}
```

`NaoSeSabe` **não** conta como segurança em lugar nenhum. Onde a tela afirmava
por ausência, ela passa a dizer que não sabe.

## O discriminante é `alvo: None`, e é ele que impede o ruído

O risco óbvio de qualquer conserto está escrito na doc da função que ele
substitui: *"supor que ela alcança o dispositivo faria o aviso disparar sempre,
que é o mesmo que não avisar"*. O `{bootmgr}` cai nessa armadilha se a regra for
por letra. Os alvos, lidos da captura de 24/08:

```text
{bootmgr}                 alvo Some("partition=\Device\HarddiskVolume1")  letra None
{f4057bd3}  ARCA          alvo Some("partition=R:")                       letra Some('R')
{6cc093db}  UEFI:CD/DVD   alvo None                                       letra None
{6cc093dc}  UEFI:Removable Device   alvo None                             letra None
{6cc093dd}  UEFI:Network Device     alvo None                             letra None
```

**`alvo.is_none()` separa exatamente as três do `{bootmgr}`.** Uma entrada com
`device` aponta para uma partição concreta, e o ARCA só não consegue conferi-la
por letra; uma entrada sem `device` não aponta para coisa nenhuma — quem a
resolve é o firmware, no próximo POST, pelo que estiver conectado.

Com esse teste, a regra **não produz um único falso positivo na ordem que
existe**: o cenário A sai limpo, e há teste que falharia se saísse.

**E isso foi conferido fora dos testes.** O binário novo rodou nesta máquina em
24/08/2026, com o SSD conectado e as três `UEFI:*` na ordem, e a saída do
`arca status` é **byte a byte** a captura
`arca-status-2026-08-24-pos-religar.txt`, feita com o binário anterior:

```text
Ordem de boot ................... dispositivo em 2o de 5 · `Windows Boot Manager` vem antes
```

O conserto muda o que a tela diz **só** onde ela dizia demais.

Um identificador da ordem **sem bloco** na leitura conta junto: a leitura que o
deixou de fora também não diz para onde ele aponta.

## Por que a regra não depende do F12

*"Uma entrada que não diz para onde aponta, à frente do dispositivo, não é
segurança"* não afirma nada sobre este firmware. É a forma de C-3 e de
`viu_o_gerenciador`, aplicada ao vizinho de baixo: **não entendi a resposta,
então não há o que garantir.**

E há um segundo motivo, mais forte: **mesmo com o F12 respondendo "alcança", o
código não poderia escrever `Leva`.** `UEFI:Removable Device` boota o *primeiro*
dispositivo removível, e qual é o primeiro depende do que está plugado —
informação que não existe no `bcdedit`. A resposta honesta continuaria sendo
`NaoSeSabe` depois da medição, e o que mudaria é só a dureza da frase.

**O F12 continua valendo**, e é o passo que fecha P-28: escolher aquela linha em
vez da entrada `ARCA` e ver onde a máquina para. Se parar no menu do Clonezilla,
a classe alcança o `ARCABOOT` desta máquina, e o texto do aviso endurece.

## Onde o conserto pega

- **`arca status`**, nos dois ramos que P-28 nomeia: com o dispositivo atrás de
  uma entrada opaca, e com o dispositivo fora da ordem. A linha do segundo caso
  deixa de dizer `nenhuma para o dispositivo · so o boot unico leva a ele` e
  passa a dizer `nenhuma que se saiba levar ao dispositivo`.
- **`arca restore`** ganha um quarto estado em `OrdemDeBoot`. O ramo brando
  **afirma** — *"a ordem permanente hoje nao leva ao dispositivo em primeiro"* —,
  e essa afirmação não se sustenta sobre uma entrada opaca. É o mesmo degrau que
  separa `NaoDeuParaLer` de `OutraCoisaAntes`, um passo adiante: aqui a ordem foi
  lida, e o que está à frente é que não se deixa ler.
- **`arca prepare`**, e este é o furo irmão que ninguém tinha listado. A tela de
  fim dizia *"ligar a maquina continua subindo o Windows, com ou sem este
  dispositivo conectado"* — **texto fixo**, derivado de um fato só (a entrada do
  ARCA saiu da ordem), sem olhar quem ficou nela. A promessa é sobre o que
  **restou** na ordem, e passa a ser condicionada a isso. Custa uma leitura de
  `firmware` a mais no fim de `criar_a_entrada`, e ela **recusa** se o
  `{fwbootmgr}` não se deixar ler: um `None` ali seria a tela prometendo o boot
  sem ter lido a ordem.

## Duas defesas que já existiam, e nenhuma estava escrita como argumento

Elas são a razão de P-28 não ser urgente, e são melhores do que o *"as três
estão em 3º, 4º e 5º"* que os documentos usavam — porque uma delas é ativa:

- **C-13 protege por construção.** `ordem::devolver_o_windows` faz
  `/set {fwbootmgr} displayorder {bootmgr} /addfirst`, que põe o Windows na
  frente de tudo, **com ou sem alvo declarado**. Toda colheita restaura a
  condição em que P-28 é inofensiva, e isso não depende de o ARCA entender
  aquelas três entradas. É o mesmo argumento do ADR-0013 — *"o alvo é um
  identificador fixo, e o resultado vale para todas as entradas do dispositivo
  de uma vez, inclusive as que o firmware ainda não criou"* — cobrando agora um
  juro que ninguém tinha notado que existia.
- **O `arca restore` nunca silenciou.** O ramo `OutraCoisaAntes` já mandava
  remover o SSD e já dizia que religar restauraria por cima. Na operação
  destrutiva, P-28 degradava a **intensidade** do aviso, não a existência dele.
  O silêncio total era só do `arca status`.

## A medição chegou no mesmo dia, e o método não foi o F12

Às 18:39 de 24/08/2026, com o `grub.cfg` conferido inerte byte a byte
(`4b33da61…9f47aa3d`) e sem job armado, a `{6cc093dc}` `UEFI:Removable Device`
foi promovida ao **topo** da ordem permanente à mão:

```text
bcdedit /set {fwbootmgr} displayorder {6cc093dc-…} /addfirst   → exit 0
displayorder  {6cc093dc} · {bootmgr} · {f4057bd3} ARCA · {6cc093db} · {6cc093dd}
```

**O F12 ficou de fora de propósito**: ele mede a *classe* que o menu do firmware
oferece, e a pergunta é sobre a *entrada na `displayorder`*, que é o objeto que a
tela lê. Sujar a ordem à mão e desfazer no fim é o método do
[ADR-0013](0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md).

**A tela nova saiu em hardware pela primeira vez**, e é o cenário B fora do
fixture:

```text
Ordem de boot ... dispositivo em 3o de 5 · `UEFI:Removable Device` vem antes

  A entrada `UEFI:Removable Device` esta na frente do dispositivo na
  ordem permanente e NAO DIZ para onde aponta: […]
```

**A máquina reiniciou com o SSD conectado e subiu o Windows.**

### E o que apareceu depois do boot vale mais do que a resposta

Às 18:47, sem que o ARCA escrevesse nada — `arca resultado` não chegou a rodar,
e C-13 não entrou —, o `bcdedit /enum firmware` é **byte a byte** a captura das
17:11:50 (`89ca7ad1…7b8df3b9`), de antes do religar que trouxe as três:

```text
displayorder  {bootmgr} · {f4057bd3} ARCA
```

As três `UEFI:*` sumiram **inteiras**: não estão na ordem e não existem mais nem
como bloco. **O firmware reescreveu o `displayorder` no POST**, removeu as três e
devolveu o `{bootmgr}` ao topo, restaurando exatamente o estado anterior.

### O que fica medido, e o que fica como duas leituras

**Medido:** com aquela entrada em primeiro e o SSD na mesa, a máquina não bootou
no dispositivo. **Para o efeito operacional, P-28 fecha**: ela não desvia o boot.

**O que esta medição não separa** são duas explicações para o mesmo desfecho:

1. a entrada foi **tentada** e não alcançou o `ARCABOOT` — a leitura literal de
   P-28, e a que a evidência antiga favorece: o firmware desta placa enumera este
   SSD como **disco**, não como removível (foi ele quem criou a `{687478f2}`
   `UEFI OS` apontando para `partition=R:`, ADR-0012), e o Windows o classifica
   igual — não há `AVISO (C-6)` em tela nenhuma, e o `bcdedit` aceitou
   `partition=R:`, o que C-6 diz não acontecer com mídia removível;
2. a entrada foi **descartada antes de ser tentada**, na mesma reconstrução que
   apagou as três — e aí a máquina bootou pela primeira sobrevivente, que era o
   `{bootmgr}`.

Quem devolveu a ordem também não está separado: o firmware, ao reconstruir no
POST, ou o Windows, ao subir. As duas já constam como donas da ordem no
`CONTEXT.md`, e separá-las exigiria uma leitura de dentro do Linux antes de o
Windows subir.

**Isto corrige uma hipótese do ADR-0020 pela metade.** Ele nomeou como *não
medido* que o firmware **poda** entradas ao reconstruir. Poda ele poda — três,
neste POST. O que continua sem medição é a poda que ele propôs: a de uma entrada
cujo dispositivo não está mais conectado.

**No código, nada muda.** C-14 foi escrito para não depender desta resposta, e
não dependeu; o texto do aviso continua o brando, e agora com uma razão a mais
para existir: uma entrada que **some do arquivo** entre a leitura e o reinício é
ainda menos base para afirmar segurança.

## O que isto não responde

- **Se as três chegam a existir como variável `Boot####` na NVRAM.** Nenhuma
  captura do `efibootmgr` deste repositório as tem: as de 20, 21 e 22/08 mostram
  **duas** entradas na NVRAM viva, vista de dentro do Linux. Não há contradição
  medida — nenhuma captura tem as duas leituras do mesmo instante, e as três vão
  e vêm —, mas também não há como afirmar que o `efibootmgr` e o `bcdedit` veem
  a mesma lista. Fica registrado para quem tiver as duas na mão no mesmo boot.
- **Por que elas vão e vêm.** Continua sendo curiosidade sobre este firmware, e
  nenhuma tela depende da resposta.

## Consequências

- **C-14 nasce**: *ausência de resposta do firmware nunca vira segurança*. Ele
  entra no §9.1 do PRD ao lado de C-3 e C-5, que são de onde ele sai.
- `Alcance` nasce em `src/comandos/status.rs`; `Leitura::ordem_resolvida` e
  `Leitura::primeira_sem_alvo` nascem em `src/firmware.rs`, onde a pergunta *"o
  que há na ordem?"* já morava — o `arca prepare` precisa dela sem ter um
  `Dispositivo` em mãos.
- `LugarNaOrdem` ganha `sem_alvo_a_frente`, e é ele que o `arca restore` lê.
- `EntradaCriada` ganha `ordem_sem_alvo`, e `montar_o_fim` deixa de ser texto
  fixo.
- **Nove testes novos**, e o que eles cobram é a distinção: as três atrás do
  Windows **não** produzem aviso; uma delas à frente **produz**; sem `ARCABOOT`
  conectado não há dúvida a levantar; e o `{bootmgr}` nunca é uma entrada opaca.
  A suíte vai a 838, verde.
- **A captura de 24/08 entra na suíte.** Ela estava em `recursos/capturas/` pelo
  ADR-0020 e nenhum teste a lia.
- **P-28 fecha no mesmo dia**, pela medição das 18:39–18:47: aquela entrada não
  desvia o boot. Quatro capturas novas em `recursos/capturas/`, e a última é byte
  a byte a das 17:11.
- **O ADR-0020 ganha nota**: a poda que ele chamou de hipótese está medida — para
  as três entradas que o próprio firmware criou, e não para a que ele propôs.
