# PRD — ARCA v5.1

**Automatizador de Clonezilla para backup e restauração de imagem de disco.**

Versão 5.1 · 22/08/2026 · Substitui a v4
Uso pessoal · Um usuário · Sem distribuição

> **As fundações não são hipótese.** O mecanismo descrito neste documento foi
> executado em hardware real: backup completo validado e restauração completa
> bem-sucedida. Este PRD especifica o **aplicativo** a ser construído sobre um
> mecanismo já provado — não um experimento a validar.

---

## Índice

1. [O que é](#1-o-que-é)
2. [O que não é](#2-o-que-não-é)
3. [Fundações validadas](#3-fundações-validadas)
4. [Estrutura de um dispositivo](#4-estrutura-de-um-dispositivo)
5. [Fluxo: backup](#5-fluxo-backup)
6. [Fluxo: restauração](#6-fluxo-restauração)
7. [Fluxo: preparar dispositivo](#7-fluxo-preparar-dispositivo)
8. [Comandos](#8-comandos)
9. [Requisitos](#9-requisitos)
10. [Implementação](#10-implementação)
11. [Armadilhas conhecidas](#11-armadilhas-conhecidas)
12. [Decisões e pendências](#12-decisões-e-pendências)

---

## 1. O que é

Uma ferramenta de linha de comando que prepara dispositivos de backup autocontidos e dispara operações neles.

**Cada dispositivo carrega o Clonezilla e as imagens juntos.** Boota nele e escolhe: fazer um backup, ou restaurar um dos que estão ali. O dispositivo é tudo que você precisa — não há nada externo a consultar.

**O ARCA não lê nem escreve disco.** Quem faz isso é o Clonezilla. O ARCA prepara o ambiente, monta a receita, dispara o boot único e lê o resultado.

### O problema que resolve

O procedimento manual exige ~20 telas em modo texto, em inglês técnico, sendo que errar em duas delas destrói o disco. É longo demais para ser feito com a frequência devida — e foi na ausência dele que duas reinstalações de Windows aconteceram em agosto/2026.

**Pelo ARCA:** um comando e uma confirmação digitada. Nenhuma tela, nenhuma decisão técnica.

## 2. O que não é

- ❌ Catálogo ou banco de dados de imagens
- ❌ Rastreamento de número de série ou de qual imagem está em qual disco
- ❌ Backup incremental ou diferencial
- ❌ Agendamento
- ❌ Retenção automática
- ❌ Interface gráfica
- ❌ Criador de partições (ver [P1](#71--o-arca-não-cria-partições))
- ❌ Suporte a BIOS legada, BitLocker, RAID, Storage Spaces

**Princípio:** se a informação já existe na listagem de diretórios do dispositivo, não há o que armazenar.

## 3. Fundações validadas

Tudo abaixo foi medido em hardware, não projetado.

### 3.1 — Mecanismo de boot único

| Fato | Evidência |
|---|---|
| Entrada de firmware apontando para SSD externo funciona | Boot único disparado e executado, múltiplas vezes |
| `bcdedit` **rejeita `Removable Media` em silêncio** — responde "êxito" e mantém o valor antigo | Pendrive testado e recusado; SSD (`External hard disk media`) aceito |
| Partição primária comum basta — não precisa marcar tipo EFI | SSD preparado assim boota normalmente |
| O `bcdedit` **não traduz** os nomes de campo: só `identificador` sai em português | Parser por valor é o correto |
| A entrada legada desta máquina chama-se **`Clonezilla`**, GUID `{f4057bd0-…}` | Procurar só por `ARCA` criaria entrada órfã |

### 3.2 — Receita desatendida

| Fato | Evidência |
|---|---|
| `ocs_repository="dev:///LABEL=..."` funciona e elimina a ambiguidade `sda`/`sdb` | Backup real gravado no destino certo |
| `locales=` vazio abre tela de idioma mesmo em batch — fixar `locales=en_US.UTF-8` | Observado |
| `-batch -sfsck -senc` suprimem todas as perguntas | Backup real sem uma única tela |
| `ask_user` é válido para imagem e dispositivo, salvando e restaurando | Documentação oficial + uso |
| **Verificação não roda sozinha em batch** — `ocs-chkimg` tem que ser chamado | Primeiro backup gerou checksum que ninguém conferiu |
| **Pipe (`\|`, `tee`) invalida a string inteira**: o Clonezilla descarta a receita e abre o menu interativo, sem executar nada | Medido. Só redirecionamento simples (`>`, `>>`) é permitido |

### 3.3 — Backup validado

Backup real executado ponta a ponta: gravado, verificado e aprovado, sem nenhuma intervenção.

| Fato | Evidência |
|---|---|
| A receita desatendida grava a imagem completa | Imagem real com as 4 partições do `nvme0n1` |
| `ocs-chkimg` aprova a imagem e grava o veredito em arquivo | `arca-check.log` lido na volta |
| Compressão com `-z9p` | ~39% do volume em uso |

### 3.4 — Restauração validada

Restauração real sobre o `nvme0n1`. Do comando ao Windows restaurado, **sem intervenção, na primeira tentativa**.

| Fato | Evidência |
|---|---|
| `-iefi` funciona | NVRAM byte-idêntica antes e depois |
| `-k0` preserva os PARTUUIDs **mesmo com a GPT zerada** | A entrada de boot preexistente continua resolvendo |
| `bcdboot` não é necessário neste hardware | Consequência do anterior |
| O Windows da imagem sobe normalmente | Máquina restaurada e em uso |

> **O `-iefi` era a pergunta que originou o projeto.** Está respondida: a restauração não toca na NVRAM e o Windows sobe.

### 3.5 — Ainda não medido

| # | Pendência |
|---|---|
| P-6 | **O `ocs-sr` devolve código diferente de zero quando falha?** O ramo de sucesso foi medido; o de falha não. Uma restauração bem-sucedida não fecha isso, por definição. Fecha com falha forçada, provavelmente em VM |

## 4. Estrutura de um dispositivo

```
[dispositivo]  — um SSD externo, duas partições
├── sda2 — FAT32, ~1,5 GB, label ARCABOOT
│     ├── EFI/boot/bootx64.efi
│     ├── live/  (kernel, initrd, filesystem.squashfs)
│     ├── boot/grub/grub.cfg      ← receita, reescrita a cada operação
│     └── arca/                   ← o próprio ARCA e o estado do job
└── sda1 — NTFS, resto, label ARCAVAULT
      ├── ARCA-LOGS/
      ├── 2026-08-21_WindowsCompleto/
      └── ...
```

Os rótulos são **sempre os mesmos** em todo dispositivo. É o que torna a receita reprodutível e os dispositivos intercambiáveis.

**Regra única de operação:** um dispositivo ARCA conectado por vez.

### 4.1 — O ARCA e o estado moram no dispositivo

Não é preferência. A imagem captura o `nvme0n1` — o disco **interno**. O dispositivo é externo, logo **não entra na imagem**.

Consequência: uma restauração substitui o `C:` e devolve, junto, qualquer ARCA que estivesse lá dentro — inclusive versões antigas com defeitos já corrigidos. **O que julga a restauração não pode morar no disco que a restauração substitui.**

Morando no `ARCABOOT`, o ARCA e o `estado.json` sobrevivem a qualquer restauração.

### 4.2 — O ambiente precisa estar fora da imagem

A máquina boota nele **antes** de a imagem ser restaurada. Um ambiente que só existisse dentro da imagem seria inalcançável no momento em que é necessário.

Custo de mantê-lo fora: **zero** — a imagem não engorda por causa dele.

### 4.3 — O selo liga o job ao desfecho

Ao armar, o ARCA gera um identificador aleatório — o **selo** — grava no `estado.json` e o embute na receita. O Clonezilla o devolve na primeira linha do `arca-fim.txt`. Na volta, só é aceito o desfecho cujo selo case com o job pendente.

Isso existe porque **não há relógio comum**: o Clonezilla lê o RTC (hora local do Windows) como UTC e fica 3 h adiantado, permanentemente. Uma trava construída sobre comparação de datas já reprovou um backup perfeito.

O selo resolve quatro casos com um mecanismo só: desfecho de um job anterior, desfecho vindo de dentro de uma imagem antiga (§11, job fantasma), desfecho ausente porque o boot nunca aconteceu, e arquivo truncado por desligamento no meio.

## 5. Fluxo: backup

### 5.1 — O que o usuário faz

| # | Ação |
|---|---|
| 1 | Conectar o dispositivo |
| 2 | `arca backup <nome>` |
| 3 | Confirmar digitando |
| 4 | *(esperar — pode sair de perto)* |
| 5 | **Remover o SSD antes de religar** |
| 6 | Ligar a máquina |
| 7 | `arca resultado` (com o SSD reconectado) |

> **O passo 5 não é zelo.** Após restauração seguida de `poweroff`, o boot seguinte foi para o dispositivo removível, apesar de não haver `bootsequence` pendente. Causa não determinada, não reproduzido. Remover o SSD elimina o cenário.

### 5.2 — Diálogo

```
> arca backup 2026-08-22_Apps

Dispositivo ARCA: ARCAVAULT (E:) · 183 GB livres
Origem: KINGSTON SNV3S500G · 498,7 GB · 92 GB em uso
Imagem estimada: ~36 GB · espaco suficiente


  Desarmando receita anterior ..... ok
  Inicializacao rapida ............ desativada   ok
  chkdsk /scan .................... limpo        ok
  Nome disponivel ................. ok
  Receita validada ................ ok

A maquina vai reiniciar agora e desligar sozinha ao terminar.
AO TERMINAR: remova o SSD antes de religar.

Digite o nome do backup para confirmar: 2026-08-22_Apps

Reiniciando...
```

### 5.3 — O que acontece sem intervenção

Firmware carrega o Clonezilla → monta `LABEL=ARCAVAULT` em `/home/partimag` → executa a receita → grava → verifica → escreve o veredito em arquivo → desliga.

**Zero telas.**

### 5.4 — Ao voltar

```
> arca resultado

Backup 2026-08-22_Apps
  22/08/2026 · 36,2 GB
  Verificacao: APROVADA

  Desarmando SSD .................. ok

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 36,2 GB · aprovada

183 GB livres
```

### 5.5 — Desfechos possíveis

Vale para backup e para restauração. Nenhuma linha desta tabela é silêncio: toda combinação tem mensagem própria.

| O que se encontra | Significado | O que o ARCA faz |
|---|---|---|
| Selo bate, `ARCA_FIM` presente, desfecho `OK` | Operação concluída | Mostra o veredito da imagem |
| Selo bate, desfecho `FALHOU` | O Clonezilla falhou e disse | Reporta falha e aponta o log |
| Selo bate, sem `ARCA_FIM` | Truncado — desligamento no meio | Falha; a pasta é resíduo (B-3) |
| Selo não bate | Job fantasma | Ignora o arquivo e avisa |
| Sem `arca-fim.txt`, com job pendente | O boot não aconteceu, ou o Clonezilla abriu menu | Falha, nomeando as duas causas |
| Sem `arca-fim.txt`, sem job pendente | Não há nada a colher | Diz isso e para |

## 6. Fluxo: restauração

### 6.1 — Windows funcionando

```
> arca restore

Imagens em ARCAVAULT:
  [1] 2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  [2] 2026-08-22_Apps              22/08 · 36,2 GB · aprovada

Qual restaurar? 2

Origem da imagem: KINGSTON SNV3S500G (conferido contra blkdev.list)
Destino:          KINGSTON SNV3S500G · 498,7 GB

ATENCAO: a restauracao APAGA o disco de destino.
Tudo que estiver nele sera perdido.

Digite o nome da imagem para confirmar: 2026-08-22_Apps

A maquina vai reiniciar e restaurar sem intervencao.
AO TERMINAR: remova o SSD antes de religar.

Reiniciando...
```

A escolha acontece **no Windows**, com a lista à vista. O Clonezilla executa sem perguntar nada.

### 6.2 — Verificação do alvo

Cada pasta de imagem carrega a identidade do disco de origem em `disk` e `blkdev.list`. O ARCA confere o destino contra o conteúdo da própria imagem — não confia na suposição de disco único.

### 6.3 — Windows não boota

Não há `arca restore` a rodar. Boote pelo dispositivo com F12 e use o menu do Clonezilla. O ambiente está lá, íntegro, porque nunca esteve dentro da imagem.

## 7. Fluxo: preparar dispositivo

### 7.1 — O ARCA não cria partições

> **Princípio P1.** O ARCA não executa a operação mais destrutiva do fluxo. O que se faz uma vez por dispositivo, e destrói tudo quando sai errado, fica com o usuário e com a ferramenta do sistema.

Particionar um disco é exatamente isso: a operação mais destrutiva do fluxo, feita **uma vez por dispositivo**.

`arca prepare` **exige** uma partição FAT32 vazia de ≥ 1 GB. Não havendo, imprime as instruções para criá-la no Gerenciamento de Disco e para.

```
> arca prepare

Dispositivo: KGSSE100 256GB
  sda1  NTFS   236,9 GB  ARCAVAULT   ok
  sda2  FAT32    1,6 GB  ARCABOOT    ok

  Baixando Clonezilla ............. ok  (checksum conferido)
  Extraindo ....................... ok
  Instalando o ARCA em ARCABOOT ... ok
  Entrada de firmware ............. migrada de "Clonezilla"

Dispositivo pronto.
```

## 8. Comandos

```
arca prepare              # instala o Clonezilla e o ARCA num dispositivo pronto
arca backup <nome>        # monta a receita, arma o boot, reinicia
arca resultado            # le o veredito e desarma o SSD
arca list                 # imagens no dispositivo conectado
arca restore              # lista, confirma e reinicia para restaurar
arca verify <nome>        # confere os MD5SUMS, sem reiniciar
arca status               # diagnostico: dispositivo, firmware, job pendente
```

Duas flags:

```
--dry-run                 # imprime a receita e o que faria; nao arma nada
--completo                # em verify: arma boot unico para o ocs-chkimg
```

Todos exigem privilégio administrativo.

## 9. Requisitos

### 9.1 — Comuns a toda operação

| ID | Requisito |
|---|---|
| C-1 | **Desarmar a receita anterior incondicionalmente**, como primeiro passo, sem consultar estado nenhum |
| C-2 | **Validar a receita antes de gravar** no `grub.cfg`: rejeitar pipes, aspas desbalanceadas e nomes inseguros |
| C-3 | Nunca confiar no retorno do `bcdedit`; sempre conferir com `/enum` e parsear **por valor** |
| C-4 | Procurar a entrada `ARCA`; não havendo, migrar a legada `Clonezilla` em vez de criar outra |
| C-5 | Boot único — nunca alterar a ordem permanente |
| C-6 | Recusar `Removable Media` como alvo de entrada de boot; orientar F12 |
| C-7 | Repassar os argumentos ao relançar com elevação por UAC |
| C-8 | Escapar aspas com **barra invertida**, não crase — quem reparte a linha é o parser do Windows |
| C-9 | Avisar, antes de reiniciar, para remover o SSD ao terminar |
| C-10 | **Recusar mais de um dispositivo ARCA conectado.** Dois `ARCAVAULT` ou dois `ARCABOOT` tornam o destino ambíguo, e é por LABEL que a receita resolve (S-3) |
| C-11 | **Gerar um selo ao armar**, gravá-lo no `estado.json` e embuti-lo na receita; aceitar como desfecho apenas o `arca-fim.txt` cujo selo case (§4.3) |
| C-12 | **Ausência de desfecho é falha, nunca silêncio.** Havendo job pendente e nenhum `arca-fim.txt`, reportar as duas causas possíveis: o boot não ocorreu, ou o Clonezilla abriu menu (§5.5) |

### 9.2 — Backup

| ID | Requisito |
|---|---|
| B-1 | Localizar o dispositivo pela partição `ARCAVAULT` |
| B-2 | Recusar nome com espaço, acento ou caractere inválido para nome de pasta |
| B-3 | **Recusar nome cuja pasta já exista** — mesmo sem `MD5SUMS`. Pasta sem `MD5SUMS` é resíduo de backup interrompido; o usuário apaga à mão |
| B-4 | Espaço mínimo: o maior entre `maior imagem do dispositivo × 1,3` e `em uso × 0,45`. Entre 1× e 1,5× disso: avisar e pedir confirmação digitada |
| B-5 | Verificar Inicialização Rápida; oferecer `powercfg /h off` |
| B-6 | Rodar `chkdsk /scan`; oferecer agendar `/f` se acusar erro |
| B-7 | Receita com nome e disco embutidos — **sem `ask_user`** |
| B-8 | Sempre `-batch -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -scs`. O `-batch` é o que suprime as perguntas (§3.2) — sem ele, a receita abre tela |
| B-9 | Sempre chamar `ocs-chkimg` explicitamente, com saída redirecionada para arquivo |
| B-10 | Nunca apagar nada |

### 9.3 — Restauração

| ID | Requisito |
|---|---|
| R-1 | Listar as imagens **no Windows**; a escolha acontece antes do reinício |
| R-2 | Conferir o destino contra `disk`/`blkdev.list` da imagem |
| R-3 | Exigir o nome da imagem digitado por extenso |
| R-4 | Sempre `-batch -k0 -iefi -j2`, sempre **sem** `-g auto` |
| R-5 | Receita com `if/then/else`: escrever `ARCA_RESTORE=OK` ou `ARCA_RESTORE=FALHOU` |
| R-6 | Ler esse arquivo na volta e **conferir o selo antes de acreditar nele** (C-11) |
| R-7 | Destino diferente do disco de origem é **permitido**, com confirmação que nomeia o disco de destino. Recusar sempre que o destino for **menor** que a origem: `-k0` copia a tabela inteira e, num disco menor, corrompe em vez de falhar. Em disco novo, `-iefi` não encontra entrada correspondente e o `bcdboot` volta a ser necessário — ao contrário do que §3.4 mediu no disco original |

### 9.4 — Segurança

| ID | Requisito |
|---|---|
| S-1 | O ARCA nunca abre o disco de origem em **acesso raw** de escrita. Chamar `powercfg` ou `chkdsk` (B-5, B-6) não é isso: são operações do próprio sistema, pelas quais o Windows responde |
| S-2 | Operação destrutiva exige texto digitado, nunca só `s` |
| S-3 | Destino sempre por LABEL — nunca por letra, `sda` ou número de série |
| S-4 | Veredito sempre gravado em arquivo, nunca só em tela |
| S-5 | Falha parcial é tratada como falha total |
| S-6 | **Nunca comparar uma data escrita pelo Windows com outra escrita pelo Linux.** O que liga um job ao seu desfecho é o selo (C-11), nunca o tempo |

### 9.5 — Consulta e verificação

| ID | Requisito |
|---|---|
| L-1 | `arca list` lê o dispositivo, nunca um catálogo — se a informação está na listagem de diretórios, não há o que armazenar |
| L-2 | Pasta sem `MD5SUMS` aparece como **resíduo**, não como imagem, e nunca é oferecida para restaurar |
| V-1 | `arca verify <nome>` confere os `MD5SUMS` no Windows, em segundos, sem reiniciar. Pega corrupção de mídia e cópia truncada |
| V-2 | `arca verify <nome> --completo` arma boot único que só roda `ocs-chkimg`. É outra força de verificação: **não substitui B-9**, que continua obrigatória em todo backup |

### 9.6 — Preparação de dispositivo

| ID | Requisito |
|---|---|
| PR-1 | Versão do Clonezilla **fixada**, com o SHA256 esperado **compilado no binário do ARCA** — nunca baixado junto do arquivo, o que não verificaria nada. Não batendo, recusar e parar |
| PR-2 | `arca prepare --iso <caminho>` instala de arquivo local. É o que salva quando a máquina que precisa preparar o dispositivo é justamente a que está sem Windows |
| PR-3 | Guardar no `ARCAVAULT` uma cópia do pacote usado. Dispositivo autocontido inclui poder reconstruir o dispositivo |

## 10. Implementação

### 10.1 — Receita de backup

```bash
#!/bin/bash
NOME="2026-08-22_Apps"
DISCO="nvme0n1"
SELO="a3f1c9e07b2d4856"
LOG="/home/partimag/ARCA-LOGS/$NOME"
mkdir -p "$LOG"

echo "ARCA_SELO=$SELO" > "$LOG/arca-fim.txt"

if ocs-sr -batch -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -scs -p true \
          savedisk "$NOME" "$DISCO"; then
  echo "ARCA_BACKUP=OK" >> "$LOG/arca-fim.txt"
else
  echo "ARCA_BACKUP=FALHOU" >> "$LOG/arca-fim.txt"
fi

ocs-chkimg -b -or /home/partimag "$NOME" \
  > "/home/partimag/$NOME/arca-check.log" 2>&1

echo "ARCA_FIM" >> "$LOG/arca-fim.txt"
sleep 20
poweroff
```

### 10.2 — Receita de restauração

```bash
#!/bin/bash
NOME="2026-08-22_Apps"
SELO="7e02b4d1af963c85"
LOG="/home/partimag/ARCA-LOGS/$NOME"
mkdir -p "$LOG"

echo "ARCA_SELO=$SELO" > "$LOG/arca-fim.txt"

if ocs-sr -batch -j2 -k0 -iefi -p true restoredisk "$NOME" nvme0n1; then
  echo "ARCA_RESTORE=OK" >> "$LOG/arca-fim.txt"
else
  echo "ARCA_RESTORE=FALHOU" >> "$LOG/arca-fim.txt"
fi
echo "ARCA_FIM" >> "$LOG/arca-fim.txt"
sleep 20
poweroff
```

O `LOG` mora no `ARCAVAULT`, que a restauração não toca — a imagem substitui o `nvme0n1`, e o desfecho sobrevive num disco que não estava no caminho.

> **Por que `if/then/else` e não `;`.** Encadear com `;` não olha código de saída: uma restauração que falhasse produziria exatamente o mesmo rastro de uma que desse certo.

### 10.3 — Restrições da receita

- **Sem pipes.** Só `>` e `>>`. Um pipe invalida a string inteira e o Clonezilla abre o menu interativo, sem executar nada e sem avisar
- `locales=en_US.UTF-8` explícito
- `toram` mantido — evita acoplar o live system ao dispositivo que ele remonta
- Validar a string antes de gravar (C-2)

### 10.4 — Stack

Rust + `clap`. Sem interface gráfica, sem banco. O único estado é um arquivo por dispositivo, gravado no `ARCABOOT`.

Manifesto com `requireAdministrator`, repassando argumentos na reelevação.

## 11. Armadilhas conhecidas

Cada uma custou uma execução real para aparecer.

| Armadilha | Efeito | Defesa |
|---|---|---|
| Pipe na receita | Clonezilla ignora tudo e abre o menu — indistinguível de "o boot não funcionou" | C-2 |
| `;` em vez de `if/then/else` | Falha deixa o mesmo rastro que sucesso | R-5 |
| Relógio do Clonezilla 3h adiantado | Ele lê o RTC (hora local do Windows) como UTC. Uma trava construída sobre comparação de datas reprovou um backup perfeito | S-6 |
| Argumentos perdidos na reelevação | `--dry-run` virou execução real, sem aviso | C-7 |
| Crase como escape | O parser do Windows reparte a linha, não o do PowerShell | C-8 |
| Job fantasma | Imagem feita quando o ARCA ainda morava no `C:` carrega dentro de si um `estado.json` pendente apontando para si mesma. §4.1 elimina a causa daqui para frente; imagens antigas continuam trazendo o problema de volta | C-11 |
| ARCA dentro da imagem | Restaurar devolve versões antigas com defeitos já corrigidos | §4.1 |
| Pasta sem `MD5SUMS` | Resíduo de backup interrompido; recusar só imagem válida empurra o usuário a regravar por cima dos fragmentos | B-3 |
| Boot no removível após `poweroff` | Não reproduzido, causa não determinada | C-9 |

## 12. Decisões e pendências

### Decisões fechadas

| Decisão | Motivo |
|---|---|
| Só imagens completas, nunca incrementais | Independência: corrupção não se propaga |
| Cada dispositivo é autocontido | Nada externo é necessário para restaurar |
| O ARCA e o estado moram no dispositivo | O dispositivo não entra na imagem |
| Labels fixos `ARCABOOT` / `ARCAVAULT` | Receita reprodutível, dispositivos intercambiáveis |
| Um dispositivo ARCA por vez | Elimina ambiguidade de label |
| Nome livre, nada é sobrescrito | Sem marcos, sem catálogo, sem retenção |
| Escolha da imagem no Windows, execução sem telas | A lista à vista, antes do ponto sem volta |
| `-iefi` e `-k0` sempre na restauração | Validados por medição |
| Verificação sempre, veredito em arquivo | Imagem não verificada é suposição |
| O ARCA não cria partições | Princípio P1 |
| `toram` mantido | Evita acoplar o live system ao dispositivo que ele remonta |
| Job ligado ao desfecho por selo, nunca por data | Não há relógio comum entre Windows e Clonezilla (P-7) |
| A receita continua sendo string no `grub.cfg`, não arquivo | É o mecanismo medido em hardware. Trocá-lo por um `custom-ocs` em arquivo exigiria remedir, e `toram` pode desmontar o medium |
| Restaurar em disco diferente é permitido, recusado só se menor | O disco de origem morrer é o motivo de existir backup de imagem |
| Clonezilla com versão fixada e checksum embutido | Checksum baixado do mesmo servidor que o arquivo não verifica nada |
| O binário roda de onde estiver; só o estado é obrigado a morar no `ARCABOOT` | O que a restauração não pode devolver é o julgamento, não o executável |

### Pendências

| # | Questão |
|---|---|
| P-6 | O `ocs-sr` devolve código ≠ 0 quando falha? Fecha com falha forçada em VM |
| P-7 | O deslocamento de 3 h é permanente. Existe para a próxima pessoa que for comparar datas |
| P-14 | `arca resultado` deve rodar sozinho no logon? Começar sem, decidir com uso |
| P-15 | A receita de backup publicada em §10.1 divergia da fundação §3.2 quanto ao `-batch`. Adotado o `-batch`, alinhando à fundação medida — mas **qual das duas rodou no hardware não está registrado**. Confirmar na primeira execução pelo ARCA |

---

*Documento vivo. Atualizar após cada medição em hardware.*
