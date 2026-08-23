# O bloco do ARCA deriva do `menuentry --id live-toram` do próprio dispositivo

A E4 deixou esta decisão escrita e sem tomar: `grub::armar` **recebe** o bloco
pronto porque as quatro cópias armadas do dispositivo divergem entre si, e
escolher entre elas é decidir a linha de comando que o kernel recebe. Havia
duas saídas — transcrever uma delas, ou derivar do `grub.cfg` que está no
dispositivo.

Decidimos **derivar**. `src/menuentry.rs` copia o `menuentry --id live-toram`
do arquivo corrente, substitui os cinco parâmetros que §10.2.1 do PRD lista, e
troca o título e o `--id`. Tudo o mais sai byte a byte como estava.

## O modelo é o `live-toram`, e não o `live-default`

**Isto é medição, e ela corrige o §10.2.1.** O documento lista o `toram` entre
"o resto da linha, que é do `menuentry` base do Clonezilla". O `live-default`
**não tem** `toram`. Quem tem é o `live-toram`, e ali o
`toram=live,syslinux,EFI,boot,.disk,utils` está exatamente onde as capturas
armadas o mostram — logo depois do `vga=788`.

Comparadas as duas linhas token a token, a captura
`grub-backup-arca-teste-02.cfg` é o `live-toram` do `grub.cfg` inerte com
**exatamente cinco** substituições:

```text
locales=                        →  locales=en_US.UTF-8
keyboard-layouts=               →  keyboard-layouts=NONE
(ausente)                       →  ocs_repository="dev:///LABEL=ARCAVAULT"
ocs_live_run="ocs-live-general" →  ocs_live_run="bash -c '<a receita>'"
ocs_live_batch="no"             →  ocs_live_batch="yes"
```

Nada mais. **O `toram` nunca foi acrescentado por ninguém**: ele veio junto do
modelo. É a explicação mais simples possível, e ela estava escondida atrás de
um `menuentry` a seis linhas de distância do que se estava olhando.

E é o modelo certo por mais do que casar: é o `toram` que evita acoplar o live
system ao dispositivo que ele vai remontar como `/home/partimag`, que é a
decisão registrada no §10.3 e no [ADR-0002](0002-receita-como-string-no-grub.md).

O `live-default` continua tendo um papel, e é **outro**: é para onde o
`set default` volta no estado inerte ([ADR-0005](0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)).
O `live-toram` é o **modelo** do armar; o `live-default` é o **alvo** do
desarmar. Duas entradas, dois papéis, e nenhum deles substitui o outro.

## O oráculo, e o único byte que não bate

Derivar do `grub.cfg` inerte deste dispositivo, com os parâmetros da
`teste-02`, produz o bloco da `teste-02` — e o teste não pode ser ajustado
para passar, porque o alvo é o arquivo que rodou em hardware.

Ele bate em tudo menos num byte: a `teste-02` tem **dois** espaços entre
`locales=en_US.UTF-8` e `keyboard-layouts=NONE`. É a impressão digital de uma
edição à mão — quem trocou `locales=` por `locales=en_US.UTF-8 ` deixou o
espaço que já separava os dois parâmetros. O ARCA escreve um espaço só, e o
teste **nomeia** essa diferença em vez de copiá-la: reproduzir um artefato de
edição seria confundir o que rodou com o que se quis. Um segundo teste cobra
que essa seja a única divergência de espaçamento, para que a normalização não
possa ser alargada até a comparação não provar nada.

## Por que não transcrever, e o que a `teste-03` mostra

É a mesma razão do ADR-0005 aplicada ao armar: o `grub.cfg` carrega a
configuração **daquele** dispositivo e **daquela** versão do Clonezilla, e
escrever por cima um bloco fixo descartaria tudo isso em silêncio. O modo de
falha é o Clonezilla não subir na máquina de quem trocar de dispositivo.

A `teste-03` é a evidência de que a derivação **não foi** como aquele bloco
nasceu, e é a evidência mais desconfortável da etapa. Ela perdeu **nove**
coisas que o modelo tem:

`hostname=cl-3.3.3-15`, `ocs_live_extra_param=""`, `i915.blacklist=yes`,
`radeonhd.blacklist=yes`, `nouveau.blacklist=yes`, `vmwgfx.enable_fbdev=1`,
`ocs_1_cpu_udev`, `scsi_mod.use_blk_mq=0`, `nvme.poll_queues=1`.

E a `teste-03` é a **única das quatro** com `set default="arca-backup"` — a
única que, pelo ADR-0005, teria rodado desatendida. A única cópia que
provavelmente rodou é a que perdeu o parâmetro de NVMe, numa máquina cujo disco
de origem é NVMe.

Isso não é argumento para transcrevê-la; é o contrário. Ela mostra o que
acontece quando o bloco é montado de memória em vez de derivado do arquivo, e
há teste fixando cada uma das nove perdas.

## O título é constante, e transcrito

As quatro cópias armadas — inclusive a de **restauração** — escrevem
`menuentry "ARCA - backup automatico" --id arca-backup`. O título nunca nomeou
a operação. Inventar agora um `ARCA - restauracao automatica` para a E9 seria
acrescentar uma diferença que nunca rodou, e o que decide o que executa é o
`--id`, para onde o `set default` aponta. O título só apareceria num menu que
o boot desatendido não chega a mostrar.

## O `bootsequence` sobre uma entrada fora da ordem de boot

Medido em 22/08/2026, com a entrada do ARCA **fora** do `displayorder` do
`{fwbootmgr}` — que é a configuração desta máquina hoje:

```text
> bcdedit /set {fwbootmgr} bootsequence {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
A operação foi concluída com êxito.                             (código 0)

> bcdedit /enum {fwbootmgr}
identificador           {fwbootmgr}
displayorder            {bootmgr}
bootsequence            {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
timeout                 1

> bcdedit /deletevalue {fwbootmgr} bootsequence
A operação foi concluída com êxito.                             (código 0)

> bcdedit /enum {fwbootmgr}
identificador           {fwbootmgr}
displayorder            {bootmgr}
timeout                 1
```

Quatro coisas saem daí, e três eram desconhecidas:

1. **O `bcdedit` aceita `bootsequence` para uma entrada que não está no
   `displayorder`.** Não estava medido em lugar nenhum, e C-5 depende disso: se
   exigisse a entrada na ordem, armar obrigaria a violar C-5.
2. **O `displayorder` não muda** — nem ao pôr, nem ao tirar. C-5 está a salvo
   nas duas operações.
3. **A forma da linha bate com o caso construído da E2**, byte a byte:
   `bootsequence` com o mesmo recuo dos outros campos, entre `displayorder` e
   `timeout`. O duplo de `src/duplos.rs` reproduzia isso por suposição desde a
   E2; agora é transcrição.
4. **Com `bootsequence` presente, o `/deletevalue` sai com código 0** — ao
   contrário do código 1 medido na E4 quando não há o que apagar. As duas
   metades do comportamento estão medidas, e é sobre elas que o desarmar
   decide não acreditar em nenhuma.

**O que isto ainda não provava é que o firmware honra a marca.** Custava um
reinício, e o reinício veio na mesma noite — o marco em hardware da E7, em
22/08/2026. **O firmware honra.** A prova não é do lado Windows: é o
`efibootmgr` lido de dentro do live, `BootCurrent: 0001` com
`BootOrder: 0000,0001`, a máquina bootando por uma entrada que não era a
primeira da ordem. P-18 fechada.

E o mesmo marco desmentiu algo que esta seção supunha sem dizer: que a
configuração medida acima — a entrada do ARCA **fora** do `displayorder` —
fosse estável. Não é. O ciclo de boot pelo dispositivo a põe de volta, e depois
daquele backup ela está na ordem, e em primeiro. Isso não desfaz nada do que
está medido aqui; muda o que o teste que guardava essa configuração podia
cobrar. Ver [ADR-0009](0009-a-ordem-permanente-muda-no-ciclo-de-boot.md).

## Consequências

O `arca backup` deriva o bloco do arquivo que vai gravar, e não de uma cópia
embutida. Um dispositivo com outra versão do Clonezilla continua funcionando
desde que tenha um `menuentry --id live-toram`; não tendo, o ARCA **recusa** em
vez de montar a linha de comando do kernel por conta própria.

O §10.2.1 do PRD ganha a correção: o `toram` é do `menuentry` base, e o
`menuentry` base é o `live-toram`.

Se um dia a derivação se mostrar frágil demais na prática, transcrever continua
aberto — e volta com medição, e não com a suposição de que casar texto é sempre
pior.
