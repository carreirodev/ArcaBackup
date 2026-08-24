# Passo a passo — P-19 e P-23, 24/08/2026

Três reinícios, cerca de 1 h. Guarde isto aberto no celular: **a máquina reinicia
três vezes**, e nos intervalos você não tem o terminal.

Todos os comandos são rodados de `C:\Users\Eduardo\Repository\ArcaBackup`, num
PowerShell **elevado**. O script pede confirmação digitada antes de cada ponto
sem volta e recusa sozinho se achar o firmware fora do esperado.

| Fase | O que fecha | Custo | Destrutivo? |
|---|---|---|---|
| 1 | **P-19** | 1 reinício, ~6 min | não |
| 2 | (paga a fase 3, e dá o segundo braço de P-19) | 1 reinício, ~21 min | não |
| 3 | **P-23** | 1 reinício, ~25 min | **sim — apaga o disco do sistema** |

**Você pode parar depois da fase 1.** P-19 fecha lá, e as fases 2 e 3 existem só
por causa de P-23.

---

## Antes de começar

1. Confira que o dispositivo está conectado e que a mesa está limpa:

   ```powershell
   cd C:\Users\Eduardo\Repository\ArcaBackup
   .\target\release\arca.exe status
   ```

   Tem de sair: `ARCAVAULT D:`, `ARCABOOT R:`, `Boot unico ... nao armado`,
   `Estado ... ja colhido, nada esperando`, e `dispositivo em 2o de 2`.

2. Commite o que quiser levar dentro da imagem — inclusive estes dois scripts,
   que ainda estão fora do git. O que não estiver commitado continua no disco;
   o ponto do commit é outro: a fase 3 devolve o `C:` ao estado da fase 2.

---

## Fase 1 — P-19 · um boot pelo dispositivo sem `bootsequence`

**A pergunta:** o firmware reescreve a entrada de boot só quando ela é consumida
por `bootsequence`, ou qualquer boot pelo dispositivo basta?

**O método:** subir a entrada `ARCA` ao topo da ordem permanente à mão e
reiniciar sem job armado. A máquina boota pelo dispositivo **pela ordem**, e não
por boot único — que é exatamente o discriminante. Sujar a ordem e desfazer no
fim é o método do ADR-0013, o mesmo que fechou P-28.

### 1. Disparar

```powershell
.\recursos\experimento-p19-p23.ps1 -Passo 1a
```

Ele imprime o `arca status`, grava a leitura de antes, confere que **não há
`bootsequence`**, pede `BOOT`, sobe a entrada `ARCA` ao topo, confere de novo e
reinicia. Se qualquer conferência falhar, ele para **antes** de reiniciar.

### 2. No POST — o menu do Clonezilla, com 30 s de timeout

O menu tem `set timeout="30"` e `set default="live-default"`: se você não fizer
nada, ele sobe o Clonezilla live sozinho. **Deixe subir** — é lá dentro que está
a medição que vale mais.

- idioma → o que preferir
- teclado → `Don't touch keymap`
- no menu do Clonezilla → **`Enter_shell`** (é a última opção, "Command line prompt")

### 3. No shell do live — a leitura por dentro do boot

```sh
sudo -i
efibootmgr -v > /tmp/nvram.txt
blkid | grep -i arcavault
mkdir -p /mnt/v && mount /dev/disk/by-label/ARCAVAULT /mnt/v
cp /tmp/nvram.txt /mnt/v/ARCA-LOGS/nvram-live-2026-08-24-sem-bootsequence.txt
sync; umount /mnt/v; poweroff
```

Se o `mount` recusar, use o `/dev/sdXN` que o `blkid` mostrou. Se nada montar:
`cat /tmp/nvram.txt` e **fotografe a tela** — o que importa são as duas linhas
`Boot####` e a forma da entrada do dispositivo. Depois `poweroff`.

> **Por que esta leitura vale mais.** Ela é escrita **durante** o boot que se
> quer explicar. Uma leitura só do lado Windows não separa *"o firmware não
> reescreveu"* de *"reescreveu, e o Windows recriou ao subir"* — que é a
> armadilha do §11 e a razão de o ADR-0011 não ter fechado P-19 com as capturas
> que já existiam.

**Atalho, se não quiser entrar no live:** aperte uma seta para parar o timeout e
desligue no botão. O experimento continua válido, só mais fraco.

### 4. De volta no Windows — antes de qualquer comando do ARCA

```powershell
.\recursos\experimento-p19-p23.ps1 -Passo 1b
```

Ele grava a leitura de depois, compara os SHA256, conta as entradas da ordem e as
classes `UEFI:*`, **diz o veredito** e desfaz a sujeira da ordem
(`/addfirst {bootmgr}`, o mesmo que C-13 faria ao colher).

### Como ler o resultado

| O que sai | O que significa |
|---|---|
| `nenhum rastro do firmware` | o boot **sem** `bootsequence` não reescreve. Com 22/08 do outro lado — onde o boot **com** `bootsequence` reescreveu —, **P-19 fecha pela positiva** |
| `RASTRO DO FIRMWARE PRESENTE` | qualquer boot pelo dispositivo reescreve, e **P-19 fecha pela negativa** |

O rastro é a forma canônica que só o firmware escreve: descrição `UEFI OS` ou o
caminho `\EFI\BOOT\BOOTX64.EFI` **em maiúsculas** — contra o
`\EFI\boot\bootx64.efi` que o `bcdedit` grava. A caixa é o discriminante.

As classes `UEFI:CD/DVD Drive`, `UEFI:Removable Device` e `UEFI:Network Device`
**não** são rastro: elas vão e vêm sozinhas neste firmware (medido em 24/08, P-22
e P-28) e são contadas à parte.

Se o resultado for ambíguo — nenhum rastro, e a leitura do live também sem
mudança —, o achado é esse mesmo: **este dispositivo não reproduz o que o antigo
fez em 22/08**, e aí a pergunta muda de sujeito. Anote e pare; não force.

---

## Fase 2 — o backup

Não fecha pendência nenhuma sozinho. Existe por duas razões: é ele que cria a
imagem que a fase 3 vai restaurar — restaurar uma imagem de minutos antes é a
restauração de menor risco possível —, e o `efi-nvram.dat` que ele grava é uma
leitura de dentro do live **com** `bootsequence`, no mesmo dispositivo e no mesmo
dia: o segundo braço do experimento de P-19, de graça.

### 5. Armar

```powershell
.\recursos\experimento-p19-p23.ps1 -Passo 2a
```

Avisa se a árvore do git estiver suja, grava a leitura do firmware, pede `ARMAR`,
e chama `arca backup 2026-08-24_Ciclo`. O ARCA ainda vai pedir o **nome da imagem
por extenso** (S-2) e então reinicia sozinho.

### 6. Esperar, e o ato físico

- ~21 min sem nenhuma tela; a máquina **desliga sozinha**
- **remova o SSD antes de religar** (C-9)
- ligue a máquina — ela sobe o Windows
- reconecte o SSD

### 7. Colher

```powershell
.\recursos\experimento-p19-p23.ps1 -Passo 2b
```

Grava a leitura do firmware, roda `arca resultado`, e copia para
`recursos/capturas/` o `efi-nvram.dat` da imagem nova (com o SHA256 ao lado do de
22/08, para comparação direta) e o `arca-check.log` do backup.

**Anote o desfecho.** Se o backup não sair `concluida` com `APROVADA`, **pare
aqui**: a fase 3 restaura uma imagem, e uma imagem reprovada não é candidata.

---

## Fase 3 — P-23 · a restauração

**A pergunta:** o `arca-restore.log` começa no meio — o corte cai sempre no mesmo
lugar?

**A previsão, registrada antes de medir:** o log não começa no meio, ele foi
**truncado por baixo**. O `>` da receita abre o arquivo e o `ocs-sr` escreve por
ele; na última passagem o Clonezilla reabre o mesmo arquivo com truncamento e o
partclone escreve a tela dele a partir do byte 0; o descritor da receita, com o
offset intacto lá em cima, retoma dali — e o intervalo vira zeros. No log de
22/08 são **8.806 NULs entre os offsets 4.085 e 12.890, 53% do arquivo**.

### 8. Antes de disparar — o que morre e o que sobrevive

A restauração devolve o `C:` ao estado da fase 2. **Tudo o que você medir entre o
backup e agora se perde**, a menos que esteja no `ARCAVAULT`. O passo `3a` copia
`recursos/capturas/` inteiro para `D:\ARCA-DOCS\` antes de armar, justamente por
isso.

O `ARCAVAULT` não é tocado — a restauração de 23/08 leu 39,7 GB dele e escreveu
16 KB.

### 9. Armar

```powershell
.\recursos\experimento-p19-p23.ps1 -Passo 3a
```

Mostra o estado do git, copia as capturas para o `ARCAVAULT`, grava a leitura do
firmware, pede `RESTAURAR` — e o `arca restore` ainda vai pedir o **nome da
imagem por extenso** antes de armar.

### 10. Esperar, e o ato físico

- ~21 min de operação; a máquina **desliga sozinha**
- **remova o SSD antes de religar**
- ligue — o Windows sobe **de dentro da imagem**, e esse é o juiz que P-6 diz que
  falta
- reconecte o SSD

### 11. Colher e medir

```powershell
.\recursos\experimento-p19-p23.ps1 -Passo 3b
```

Grava a leitura do firmware, roda `arca resultado`, copia o `arca-restore.log`
para `recursos/capturas/` e roda o medidor contra as cinco previsões:

| # | O que deve sair |
|---|---|
| 1 | **uma** inicialização de terminal — só a última passagem do partclone sobreviveu |
| 2 | a tela do partclone abre o arquivo, e é a da **última** partição |
| 3 | há um bloco de NULs — prova de truncamento com descritor aberto atrás |
| 4 | há `Ending /usr/sbin/ocs-sr` e **não** há o `Starting` correspondente |
| 5 | o buraco **não** cai em 4.085–12.890 — ele cai onde o `ocs-sr` chegou |

**As cinco batendo, P-23 fecha:** o corte não é do ARCA nem do redirecionamento,
e a resposta a *"cai sempre no mesmo lugar?"* é **não — cai onde o `ocs-sr`
chegou**. O que sobrevive é sempre a última passagem, que numa falha é a
partição em que a operação parou.

**Qualquer uma não batendo, não feche.** O que não bate é o achado, e vale mais
do que a previsão.

---

## Se der errado

| Sintoma | O que fazer |
|---|---|
| O script para dizendo que há `bootsequence` | `arca desarmar`, depois recomece o passo |
| A máquina boota no Windows em vez do dispositivo (fase 1) | o `/addfirst` não pegou. Rode `arca status` e confira a linha `Ordem de boot`; não repita às cegas |
| A máquina boota no dispositivo quando você não quer | ela para no menu, e em 30 s sobe o live — nada é escrito sem receita armada. Desligue e rode `-Passo 1b`, que desfaz a ordem |
| O `arca resultado` diz que não há job | o desfecho não voltou. **Não rearme**: leia `D:\ARCA-LOGS\<pasta>\arca-fim.txt` e o log ao lado antes de qualquer coisa |
| Faltou espaço no backup | o pré-voo já disse que cabe (~51,6 GB contra 125 GB livres). Se mudou, apague uma imagem velha **conscientemente** — o ARCA não apaga nada (B-10) |

**A ordem permanente fica suja entre 1a e 1b.** Se você parar no meio, a máquina
vai bootar no dispositivo a cada religada (e parar no menu, sem risco). O
`-Passo 1b` desfaz; um `arca resultado` também desfaria, por C-13.
