# De onde vieram estas capturas

Duas coisas do ARCA não podem ser testadas contra exemplo inventado: o parser
do `bcdedit`, único ponto onde uma leitura errada leva a máquina a bootar no
lugar errado com uma receita armada; e a **receita**, que é a string que o
Clonezilla executa quando não há mais ninguém olhando. Testar as duas contra o
que eu imaginei provaria que eu sei imaginar. Estes arquivos são o que o
hardware **escreveu e executou de verdade**.

O `.gitattributes` marca esta pasta como `-text` para que o git não normalize
nada — nem as quebras de linha CRLF do `bcdedit`, nem as LF dos `grub.cfg`.

## As leituras do `bcdedit` (etapa E2)

Convertidas de CP850 para UTF-8 na gravação, e só nisso.

| Arquivo | O que é |
|---|---|
| `bcdedit-enum-firmware-pt.txt` | `bcdedit /enum firmware` desta máquina, 22/08/2026, console em CP850 |
| `bcdedit-enum-firmware-en.txt` | **o mesmo BCD, no mesmo instante**, pelo mesmo `bcdedit`, com os recursos `en-US` ao lado |
| `bcdedit-enum-firmware-legado-pt.txt` | `E:\ARCA-LOGS\nvram-windows-antes.txt`, capturado em 20/08/2026, antes de a entrada ser renomeada |

## As receitas que rodaram em hardware (etapa E3)

Cópias byte a byte, sem conversão nenhuma. Cada uma é um `grub.cfg` como
estava no dispositivo no momento em que a máquina bootou nele e executou a
receita sozinha.

| Arquivo | O que é |
|---|---|
| `grub-backup-arca-teste-02.cfg` | `R:\boot\grub\grub.cfg.backup02` — o backup de `ARCA-TESTE-02`, 19/08/2026 |
| `grub-backup-arca-teste-03.cfg` | `E:\ARCA-LOGS\grub.cfg.original` — o backup de `ARCA-TESTE-03`, 20/08/2026 |
| `grub-restauracao-arca-teste-02.cfg` | `R:\boot\grub\grub.cfg.teste02` — a restauração de `ARCA-TESTE-02`, 19/08/2026 |
| `ocs-sr-help.txt` | `E:\ARCA-LOGS\ocs-sr-help.txt` — o `--help` do `ocs-sr` **desta versão** do Clonezilla |

## O estado inerte e a quarta cópia armada (etapa E4)

Cópias byte a byte, conferidas por SHA256 contra o dispositivo depois de
gravadas.

| Arquivo | O que é | SHA256 |
|---|---|---|
| `grub-inerte-arcaboot.cfg` | `R:\boot\grub\grub.cfg` — o **estado inerte** deste dispositivo, 11069 bytes | `4b33da61…f947aa3d` |
| `grub-clonezilla-original.cfg` | `R:\boot\grub\grub.cfg.original` — o que o **Clonezilla instalou**, 05/07/2026, 11058 bytes | `9ebfa1eb…068d331b` |
| `grub-backup-arca-teste-01.cfg` | `R:\boot\grub\grub.cfg.teste01` — uma **quarta** cópia armada, 19/08/2026, não usada na E3 | `cbbe6d5a…63c3f762` |

O `grub-inerte-arcaboot.cfg` é o alvo do desarmar, e o oráculo da etapa E4
inteira. Não é um arquivo montado por teste: é o que está no dispositivo agora.
`tests/e4_desarmar_o_dispositivo.rs` compara os dois a cada execução com o SSD
conectado — uma cópia que divergiu do que documenta deixou de ser evidência.

O `grub-clonezilla-original.cfg` responde de onde vem o estado inerte. Ele
difere do inerte em **uma linha**: traz `set default="0"` onde o inerte traz
`set default="live-default"`. Desarmar o dele produz o inerte byte a byte, e é
isso que torna a regra do [ADR-0005](../../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md)
verificável em vez de arbitrária.

### O que estas três mostram e a E3 não tinha visto

- **Armar são duas mudanças, e uma delas não estava documentada.** O inerte e a
  `teste-03` diferem em exatamente duas coisas: o `set default` e o bloco de
  quatro linhas. **É o `set default` que faz o boot ser desatendido** — o
  `menuentry` sozinho só põe mais uma linha no menu. Ver §3.2 do PRD.
- **As quatro cópias armadas põem o bloco na mesma posição**, linhas 93–97,
  precedido de duas linhas em branco e seguido de uma. Um `diff` contra o
  inerte ancora umas depois da linha 91 e outras depois da 92, o que sugere
  duas formas de inserção — mas é artefato do algoritmo desambiguando linhas
  em branco repetidas. Os arquivos são iguais nessa região.
- **Só uma das quatro tem `set default="arca-backup"`**: a `teste-03`, que veio
  do `ARCAVAULT`. As três que estavam no `ARCABOOT` têm o bloco e
  `set default="live-default"` — o estado em que a máquina esperaria trinta
  segundos e bootaria no menu normal. Por quê é pergunta **fechada por falta de
  evidência**, com as três vias nomeadas no ADR-0005: datas não (S-6),
  `BootNext` não (o firmware o consome), dedução não (foi o que produziu os
  dois casos anteriores de fundação que não era).
- **Os blocos do ARCA não são iguais entre si.** A `teste-02` preserva o
  `hostname=cl-3.3.3-15` e as blacklists de driver do `menuentry` base; a
  `teste-03` perdeu os dois. Não há forma canônica transcrita — escolher qual a
  E7 vai inserir é decidir que linha de comando o kernel recebe, e é da E7.

### O que continua sem estar aqui

**Nenhum `bootsequence`.** Continua valendo o que a E2 registrou: não há job
armado nesta máquina, e armar é a E7.

A E4 é a **primeira etapa que escreve no firmware**, e escreve sem original
nenhum de onde transcrever. O `bcdedit /deletevalue {fwbootmgr} bootsequence` é
**código novo**, do mesmo jeito que o `arca-fim.txt` do ADR-0004 — marcado como
tal em `src/desarme.rs` e nos testes. O que dele foi medido em hardware, em
22/08/2026, é o comportamento **sem** `bootsequence`: código de saída 1, texto
"Elemento não encontrado", e nada muda. O caso com `bootsequence` presente está
coberto por caso construído no duplo, e a E7 o confirma.

A receita está numa linha só de cada arquivo: a `$linux_cmd` do `menuentry`
com `--id arca-backup`. `src/receita.rs` a extrai de lá nos testes, em vez de
repetir a string a mão — uma string repetida a mão prova que eu sei copiar; o
arquivo prova o que o hardware executou.

### O help se capturou sozinho

O `ocs-sr-help.txt` não foi digitado por ninguém. Ele saiu da própria receita
de `ARCA-TESTE-03`, que começa com
`ocs-sr --help > /home/partimag/ARCA-LOGS/ocs-sr-help.txt 2>&1`. A primeira
linha do arquivo é `/usr/sbin/ocs-sr: --help: invalid option` — o `ocs-sr`
desta versão não conhece `--help` e responde com o *usage* completo, que é o
que se queria. É o help **desta** versão, tirado **desta** execução, e é com
ele na mão que as decisões sobre `-scs`, `-p` e `-batch` foram tomadas.

### O que estas capturas mostram e o PRD não dizia

- **A receita nunca foi um script.** §10.1 e §10.2 do PRD mostravam um
  `#!/bin/bash` de várias linhas. O que rodou foi sempre uma string única
  dentro de `ocs_live_run="bash -c '...'"`. O ADR-0002 já havia decidido a
  forma; era o §10 que contradizia.
- **As três encadeiam com `;`, nunca com `if/then/else`.** A armadilha que
  R-5 descreve é real, mas a defesa contra ela é código novo — não há original
  de onde transcrevê-la.
- **Nenhuma das três escreve `arca-fim.txt`.** O `E:\ARCA-LOGS\2026-08-21_WindowsCompleto\arca-fim.txt`
  que existe no dispositivo (`ARCA_RESTORE=OK` / `ARCA_FIM`) veio de trabalho
  manual de validação — o mesmo padrão que o ADR-0003 registrou para o
  `ARCA_VEREDITO=`. Todo o mecanismo de desfecho, do qual a E5 e a E8
  dependem, **nunca foi exercitado em hardware**.
- **As flags de backup não eram as de B-8.** Rodou
  `-q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true`: `-batch` no fim,
  `-p true` presente e não listado, `-scs` ausente.
- **A restauração usou `-e1 auto -e2`**, que R-4 não lista, e `-p poweroff`.

Ver `docs/adr/0004-a-receita-transcreve-o-que-rodou.md` para o que foi feito
com cada uma dessas divergências.

## Por que o par pt/en prova alguma coisa

O plano de implementação nomeia a fixture em inglês como metade do risco desta
etapa, e com razão: um parser afinado num só idioma passa em todo teste e
falha na máquina de outra pessoa.

As duas primeiras capturas descrevem **a mesma configuração de boot**, lida com
segundos de diferença. Não são uma tradução de outra: são duas leituras do
mesmo dado. Isso permite o teste que fecha o risco — o parser tem de extrair
delas exatamente o mesmo resultado, campo a campo. Qualquer dependência de
texto traduzido aparece como diferença.

O `bcdedit.exe` do Windows carrega suas mensagens de
`System32\<idioma>\bcdedit.exe.mui`. Esta máquina tem `pt-BR` e `en-US`
instalados. Copiando o `bcdedit.exe` para uma pasta onde só existe
`en-US\bcdedit.exe.mui`, o carregador de recursos usa o que está ali — e a
mesma consulta ao mesmo BCD sai em inglês.

## O que o par confirma

- **Só `identificador` é traduzido** entre os nomes de campo. `device`, `path`,
  `description`, `locale`, `inherit`, `displayorder`, `timeout` e os demais
  saem idênticos nos dois idiomas. É a fundação §3.1 do PRD, agora com as duas
  metades medidas.
- **Os títulos de bloco também são traduzidos** — `Windows Boot Manager` /
  `Gerenciador de Inicialização do Windows`. O PRD não diz isso, e é por isso
  que o parser não pode usá-los para decidir nada.
- **A entrada legada é reconhecível pela `description`**, que não é traduzida.

## A entrada desta máquina mudou de nome entre as capturas

A captura de 20/08 traz `description Clonezilla`; a de 22/08 traz
`description ARCA`. O identificador é o mesmo nas duas —
`{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}`.

Não é acidente de captura: é exatamente o que C-4 descreve, dos dois lados. A
captura antiga é a única evidência real do caso "não há entrada `ARCA`, há a
legada `Clonezilla`", e é por isso que ela está aqui em vez de ter sido
descartada por estar desatualizada.

## As capturas de NVRAM que ficaram no dispositivo (etapa E7)

Não estão neste diretório — estão em `E:\ARCA-LOGS\`, no dispositivo. Ficam
onde estão de propósito: são oito arquivos grandes de `efibootmgr -v`, e o que
importa deles cabe numa tabela. Ela está no **§3.1 do PRD**, e é lá que a
próxima pessoa deve procurar.

O resumo do que elas mostram, para quem chegar aqui primeiro:

| Quando | Arquivo | `BootOrder` | `BootCurrent` |
|---|---|---|---|
| 20/08 | `nvram-original.txt` | `0000,0001` | `0001` |
| 20/08 | `nvram-windows-antes.txt` (é `bcdedit`, não `efibootmgr`) | `{bootmgr}`, `{f4057bd0}`, +3 | — |
| 20/08 | `R1/nvram-antes.txt`, `R1/nvram-depois.txt` | `0000,0001` | `0001` |
| 20/08 | `R2/nvram-antes.txt`, `R2/nvram-depois.txt` | `0003,0000` | `0003` |
| 21/08 | `2026-08-21_WindowsCompleto/nvram-antes.txt` e `-depois.txt` | `0001,0000` | `0001` |
| 22/08 | `bcdedit /enum {fwbootmgr}` desta máquina | `{bootmgr}` | — |

Três coisas que só aparecem lendo os arquivos inteiros, e não a tabela:

- **`Boot0001` é a mesma entrada que o `{f4057bd0-…}` do `bcdedit`.** O
  `efibootmgr -v` imprime os dados da variável, e lá está
  `BCDOBJECT={f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}` em UTF-16 hexadecimal. É o
  que liga as duas ferramentas, e o que dispensa deduzir a correspondência.
- **O número da entrada mudou de `0001` para `0003`** entre as capturas de
  20/08, o que só acontece quando ela é recriada.
- **Nenhuma das oito tem `BootNext`**, e isso continua não provando nada: o
  firmware o consome ao usá-lo, e todas foram feitas de dentro do Clonezilla.

**A ordem permanente desta máquina foi alterada por alguém, mais de uma vez.**
C-5 existe para impedir que o ARCA o faça, e a evidência é de que já foi feito à
mão. Em 21/08 — o backup que o §3.3 chama de validado — o dispositivo estava em
**primeiro**, o que explica o boot inteiro sem passar por boot único (P-18).

## O `bootsequence`, medido pela primeira vez (etapa E7)

A E2 e a E4 registraram aqui que **nenhuma captura tem `bootsequence`**, e que
o formato estava coberto por caso construído. A E7 mediu, em 22/08/2026, com a
entrada do ARCA **fora** do `displayorder`:

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
```

O caso construído da E2 estava certo, byte a byte: `bootsequence` com o mesmo
recuo dos outros campos, entre `displayorder` e `timeout`. E três coisas que
não estavam medidas: o `bcdedit` **aceita** a marca para uma entrada de fora da
ordem, o `displayorder` **não muda** nem ao pôr nem ao tirar, e com
`bootsequence` presente o `/deletevalue` sai com **código 0** — ao contrário do
código 1 medido na E4 quando não há o que apagar.

**O que continua sem medição é se o firmware honra a marca.** Isso custa um
reinício, e é o marco em hardware da E7. Ver
[ADR-0007](../../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md).

## O modelo do bloco do ARCA é o `live-toram` (etapa E7)

Achado ao decidir a forma do `menuentry` que a E7 insere. A captura
`grub-backup-arca-teste-02.cfg` é o `menuentry --id live-toram` do
`grub-inerte-arcaboot.cfg` com **exatamente cinco** substituições — as cinco de
§10.2.1 do PRD — e nada mais.

O `live-default`, que era o candidato óbvio, **não tem** `toram`. O
`live-toram` tem, e exatamente na posição em que as capturas armadas o mostram.
Ninguém acrescentou o `toram`: ele veio junto do modelo, e o §10.2.1 do PRD o
atribuía ao `menuentry` base sem dizer qual.

Duas coisas mais, das mesmas comparações:

- **O único byte em que a derivação e a `teste-02` divergem** é um espaço: a
  captura tem dois entre `locales=en_US.UTF-8` e `keyboard-layouts=NONE`. É
  rastro de edição à mão. O ARCA escreve um, e o teste **nomeia** a diferença
  em vez de copiá-la — reproduzir um artefato de edição seria confundir o que
  rodou com o que se quis.
- **A `teste-03` perdeu nove parâmetros** que o modelo tem: `hostname`,
  `ocs_live_extra_param`, as três `*.blacklist`, `vmwgfx.enable_fbdev`,
  `ocs_1_cpu_udev`, `scsi_mod.use_blk_mq` e `nvme.poll_queues`. É a única das
  quatro com `set default="arca-backup"` — a única que provavelmente rodou
  desatendida —, e perdeu o parâmetro de NVMe numa máquina cujo disco de origem
  é NVMe.

## O que nenhuma delas contém

**Nenhum `bootsequence`.** As capturas de `bcdedit` deste diretório continuam
sem ele, e continuam certas: não há job armado nesta máquina. A medição do boot
único da E7 está transcrita acima e não foi guardada em arquivo, porque o
comando que a produziu desfez o que fez.

**Nenhuma menção a `Removable Media` ou `External hard disk media`.** Estas
palavras não são do `bcdedit`: são valores de `MediaType` do WMI
(`Win32_DiskDrive`, em `cimwin32.dll`). Nem o `bcdedit.exe` nem os seus
`.mui` contêm qualquer uma delas — procurado nos dois idiomas. Ver o que
`src/firmware.rs` diz sobre C-6.
