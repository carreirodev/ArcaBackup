# Marco em hardware: o dispositivo em GPT

**Objetivo:** produzir uma captura medida de um dispositivo **GPT que boota**,
para que o `arca prepare` possa transcrevê-la — em vez de supor o que o Windows
escreveria.

**Este roteiro não muda código.** Ele produz a evidência sem a qual mudar o
código seria trocar um esquema medido por um suposto, que é exatamente o que o
[ADR-0014](../docs/adr/0014-o-arca-particiona-o-dispositivo.md) manda resistir. O
código vem depois, na Etapa 9.

---

## Progresso

| Etapa | O que faz | Status | Feita em |
|---|---|---|---|
| 1 | Registrar o estado antes de tocar em nada | ✅ | 2026-08-25 · NVRAM com **uma** entrada só |
| 2 | Escolher o alvo e conferir as quatro defesas | ✅ | 2026-08-25 · disco 2, as quatro passaram |
| 3 | Apagar e inicializar em GPT | ✅ | 2026-08-25 · **houve MSR**, removida |
| 4 | Criar as duas partições | — | |
| 5 | Instalar o Clonezilla na ARCABOOT | — | |
| 6 | Criar a entrada de firmware de teste | — | |
| 7 | **O boot, que é o que decide** | — | |
| 8 | Capturar a NVRAM de dentro do boot | — | |
| 9 | Voltar ao normal | — | |

**O disco está parado em GPT com zero partições**, `LargestFreeExtent`
256 059 113 472, esperando o `New-Partition` da Etapa 4. A captura viva é
[`recursos/capturas/medicao-gpt-2026-08-25.txt`](../recursos/capturas/medicao-gpt-2026-08-25.txt),
e ela cresce a cada etapa — está registrada em `PROVENIENCIA.md` com o SHA256 do
arquivo **parado na Etapa 3**, que muda quando as seguintes escreverem nele.

> **Nada foi decidido ainda.** Três das nove etapas são preparação, e quem decide
> é a **Etapa 7**. Das três perguntas que o ADR novo precisa responder — no fim
> deste documento —, só a segunda está respondida: houve MSR, e ela foi removida.

---

## Antes de começar: o que este roteiro decide, e o que não

O ADR-0014 tem um argumento e uma fraqueza:

- **O argumento:** há um original medido em MBR que comprovadamente boota, com
  seis leituras de NVRAM feitas de dentro do boot (ADR-0023). GPT não tem
  nenhuma.
- **A fraqueza:** ele diz que a falha "só se descobre depois de o Windows já
  ter sido apagado". Isso é falso — dá para bootar um dispositivo GPT e ver o
  menu do Clonezilla **sem apagar nada**. É o que este roteiro faz.

O que decide a mudança é a **Etapa 7**: bootar. Tudo antes é preparação, tudo
depois é registro.

### As duas variantes, e qual seguir

| | **Variante B — FAT32 Basic Data** *(seguir esta)* | Variante A — ESP de verdade |
|---|---|---|
| `GptType` da ARCABOOT | `{ebd0a0a2-…}` (dados básicos) | `{c12a7328-…}` (EFI System) |
| Letra de unidade | **mantém o `R:`** | não recebe letra — Windows esconde a ESP |
| `bcdedit` | `device partition=R:` **inalterado** | precisa apontar por `\Device\HarddiskVolumeN` |
| Extrair o Clonezilla | copiar para `R:\` como hoje | precisa de `mountvol` primeiro |
| Superfície de mudança | tabela de partição, e só | tabela + `bcdedit` + instalação + releitura |

**Siga a Variante B.** Ela entrega os três ganhos reais que motivam a mudança —
o limite de 2 TiB some, a tabela ganha cópia secundária com CRC32, e o esquema
deixa de ser legado — sem tocar em mais nada. A Variante A só acrescenta o tipo
ESP canônico, que rende em firmwares muito estritos e custa três superfícies
novas. Ela está em *Se não bootar*, no fim, e só se justifica se a B **não
bootar**.

### Regras que não se negociam

1. **Use um segundo dispositivo, não o de produção.** O dispositivo que boota
   hoje é a única coisa que devolve o Windows se algo der errado. Ele não entra
   neste roteiro.
2. **Não restaure nada.** O teste é ver o menu do Clonezilla subir. Restaurar
   não faz parte, e não há motivo para arriscar.
3. **Confira o índice do disco em toda etapa que escreve.** O índice do Windows
   não é identidade e muda entre boots — é a medição que motiva a releitura de
   PR-4, e agora ela vale para você também.

---

## Pré-requisitos

- [x] Um **segundo** SSD/HD externo, cujo conteúdo você pode perder — o Kingston DataTraveler Max de 238,5 GB
- [x] O dispositivo ARCA de produção **desconectado** durante todo o roteiro
- [x] `clonezilla-live-3.3.3-15-amd64.zip` — a mesma versão do
      [ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md);
      o `.zip`, não o `.iso`
- [x] PowerShell **como Administrador** — e que seja o **7**, não o 5.1; ver a nota na Etapa 3
- [ ] Saber entrar no menu de boot da máquina (F12 ou equivalente)

Abra o PowerShell elevado e prepare o arquivo de captura:

```powershell
cd C:\Users\Eduardo\Repository\ArcaBackup
$CAP = "recursos\capturas\medicao-gpt-2026-08-25.txt"
"# O particionamento do dispositivo em GPT, medido a mao" | Out-File $CAP -Encoding utf8
"# Marco em hardware, $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" | Add-Content $CAP
function Cap($titulo) { "`n### $titulo" | Add-Content $CAP; Write-Host "`n### $titulo" -ForegroundColor Cyan }
function CapOut($obj) { $obj | Out-String | Add-Content $CAP; $obj }
```

> As funções `Cap` e `CapOut` só existem nesta janela do PowerShell. Fechando a
> janela, redefina-as antes de continuar.

---

## Etapa 1 — Registrar o estado antes de tocar em nada ✅

> **Feita em 25/08/2026.** A NVRAM tinha **uma entrada só**: `{fwbootmgr}` com
> `displayorder` apontando para `{bootmgr}`, e nada mais. A entrada `ARCA` que as
> capturas de 22 a 24/08 mediram não sobreviveu à reinstalação do Windows. É esse
> o número de referência: ao final da Etapa 6 devem ser **duas**, e a Etapa 9
> volta a uma.

Isto é o que permite voltar. Não pule.

```powershell
Cap "NVRAM antes de tudo"
CapOut (bcdedit /enum firmware)
```

```powershell
Cap "Os discos desta mesa, antes"
CapOut (Get-Disk | Select-Object Number,FriendlyName,BusType,PartitionStyle,Size,IsSystem,IsBoot)
CapOut (Get-CimInstance Win32_DiskDrive | Select-Object Index,Model,MediaType,Size)
```

**Anote:** quantas entradas de firmware existem hoje e quais são. Ao final da
Etapa 6 deve haver **uma a mais** — a de teste —, e a Etapa 9 a remove.

---

## Etapa 2 — Escolher o alvo e conferir as quatro defesas ✅

> **Feita em 25/08/2026.** O alvo é `$n = 2` — Kingston DataTraveler Max,
> 256 060 514 304 bytes, `External hard disk media`, MBR com uma única partição
> NTFS rotulada `AMD Backups` em `E:` e 238,3 GB livres de 238,5. As quatro
> conferências passaram, e o disco de produção não estava na mesa.

Descubra o índice do disco novo:

```powershell
Get-Disk | Format-Table Number,FriendlyName,BusType,PartitionStyle,@{n='GB';e={[math]::Round($_.Size/1GB,1)}},IsSystem,IsBoot
```

Ponha o índice na variável e **confira** — este bloco não escreve nada, e é o
que separa apagar o disco certo de apagar o errado:

```powershell
$n = 9   # <<< TROQUE pelo indice do disco novo, e confira a linha abaixo

$d = Get-Disk -Number $n
$w = Get-CimInstance Win32_DiskDrive | Where-Object Index -eq $n
$sysLetra = $env:SystemDrive.TrimEnd(':')
$discoDoSistema = (Get-Partition -DriveLetter $sysLetra).DiskNumber

Cap "O alvo, conferido antes de tocar"
CapOut ([pscustomobject]@{
  indice           = $n
  modelo           = $w.Model
  tamanho          = $d.Size
  BusType          = $d.BusType
  MediaType_WMI    = $w.MediaType
  IsSystem         = $d.IsSystem
  IsBoot           = $d.IsBoot
  PartitionStyle   = $d.PartitionStyle
  disco_do_sistema = $discoDoSistema
})

if ($d.IsSystem -or $d.IsBoot -or $n -eq $discoDoSistema) { throw "PARE: este e o disco do sistema" }
if ($w.MediaType -ne 'External hard disk media' -and $w.MediaType -ne 'Removable Media') { throw "PARE: MediaType nao e externo/removivel -> $($w.MediaType)" }
Write-Host "as quatro conferencias passaram" -ForegroundColor Green
```

**Pare se o `throw` disparar.** São as mesmas quatro defesas que o `arca
prepare` aplica (PR-5), e elas não são opinião.

Registre também o que existe hoje no disco, para reconhecer o que se perde:

```powershell
Cap "O QUE EXISTIA — particoes e volumes"
CapOut (Get-Partition -DiskNumber $n | Select-Object PartitionNumber,DriveLetter,Type,MbrType,GptType,Offset,Size,IsActive)
CapOut (Get-Partition -DiskNumber $n | Get-Volume | Select-Object DriveLetter,FileSystemLabel,FileSystem,Size)
```

---

## Etapa 3 — Apagar e inicializar em GPT ✅

> **Feita em 25/08/2026, e a resposta é a que o roteiro tratava como
> possibilidade: houve MSR.** O `Initialize-Disk -PartitionStyle GPT` criou
> sozinho uma partição `Reserved` de 16 759 808 bytes no offset 17 408, com
> `GptType {e3c9e316-0b5c-4db8-817d-f92df00215ae}`. Foi removida, e o disco
> voltou a zero partições — que é como o MBR sai do `Initialize-Disk`. Isto
> responde a **segunda** das três perguntas do fim do documento, e deixa de ser
> hipótese para virar requisito de `particionador.rs`.
>
> **A GPT cobra 1 400 832 bytes.** Num disco de 256 060 514 304, o
> `LargestFreeExtent` depois de remover a MSR é **256 059 113 472** — a tabela
> primária no começo e a cópia secundária no fim. É esse o número que a Etapa 4
> usa nas contas, e a razão de elas não saírem de constante.
>
> Um tropeço registrado: a primeira tentativa desta etapa rodou sob
> `powershell.exe` 5.1, que leu o `.ps1` UTF-8 como ANSI — o `—` virou `â€"`, a
> aspa tipográfica fechou as strings cedo, e o script parou entre o `Clear-Disk`
> e o `Initialize-Disk`. A segunda rodou sob `pwsh` 7.6.5. **Rode os blocos
> destas etapas em PowerShell 7**, não no 5.1.

> **Ponto sem volta.** O `Clear-Disk` abaixo é irreversível. Confira `$n` mais
> uma vez — `(Get-Disk -Number $n).FriendlyName` — antes de dar Enter.

```powershell
Cap "Clear-Disk"
Clear-Disk -Number $n -RemoveData -RemoveOEM -Confirm:$false
CapOut (Get-Disk -Number $n | Select-Object PartitionStyle,Size,LargestFreeExtent)
```

```powershell
Cap "Initialize-Disk -PartitionStyle GPT"
Initialize-Disk -Number $n -PartitionStyle GPT
CapOut (Get-Disk -Number $n | Select-Object PartitionStyle,Size,LargestFreeExtent)
```

### O ponto de medição mais importante desta etapa

```powershell
Cap "Houve MSR? (o Windows cria uma Microsoft Reserved sozinho em GPT)"
CapOut (Get-Partition -DiskNumber $n -ErrorAction SilentlyContinue | Select-Object PartitionNumber,Type,GptType,Offset,Size)
```

**Olhe a saída com atenção.** Em MBR o `Initialize-Disk` deixa o disco vazio.
Em GPT o Windows costuma criar sozinho uma partição **MSR (Microsoft
Reserved)** de 16 ou 128 MB. Se ela existir:

- a ARCAVAULT nasceria como partição **2** e a ARCABOOT como **3**;
- o device path da entrada de firmware viraria `HD(3,GPT,…)` em vez de
  `HD(2,…)`;
- a releitura do `arca prepare`, que confere **a ordem das duas partições no
  disco**, passaria a ver três.

Havendo MSR, remova-a antes de seguir — ela não serve para nada num dispositivo
de dados:

```powershell
# So rode se a saida acima mostrou uma particao do tipo Reserved
Get-Partition -DiskNumber $n | Where-Object { $_.Type -eq 'Reserved' } | Remove-Partition -Confirm:$false

Cap "Depois de remover a MSR"
CapOut (Get-Partition -DiskNumber $n -ErrorAction SilentlyContinue | Select-Object PartitionNumber,Type,GptType,Offset,Size)
CapOut (Get-Disk -Number $n | Select-Object PartitionStyle,LargestFreeExtent)
```

**Anote no log se havia MSR e se ela foi removida.** É a diferença de desenho
mais provável entre MBR e GPT neste projeto, e o código vai precisar dela.

---

## Etapa 4 — Criar as duas partições

As contas saem do `LargestFreeExtent` lido agora, e **não** de constante — a
GPT secundária ocupa espaço no fim do disco que o MBR não ocupava:

```powershell
$boot  = 1677721600                                    # ARCABOOT: 1600 MiB, fixo
$livre = (Get-Disk -Number $n).LargestFreeExtent
$vault = $livre - $boot

Cap "As contas"
CapOut ([pscustomobject]@{ LargestFreeExtent = $livre; ARCABOOT_fixo = $boot; ARCAVAULT = $vault })
if ($vault -le 0) { throw "PARE: disco pequeno demais" }
```

```powershell
Cap "New-Partition 1 (ARCAVAULT)"
$p1 = New-Partition -DiskNumber $n -Size $vault
CapOut (Get-Partition -DiskNumber $n -PartitionNumber $p1.PartitionNumber | Select-Object PartitionNumber,DriveLetter,Type,GptType,Offset,Size)

Cap "New-Partition 2 (ARCABOOT)"
$p2 = New-Partition -DiskNumber $n -UseMaximumSize
CapOut (Get-Partition -DiskNumber $n -PartitionNumber $p2.PartitionNumber | Select-Object PartitionNumber,DriveLetter,Type,GptType,Offset,Size)
```

**Anote o `GptType` das duas recém-criadas, antes de formatar.** Em MBR foi
medido que nascem com `MbrType 6` e quem as leva a 7 e 12 é o `Format-Volume`.
A pergunta equivalente aqui é: **o `GptType` sai certo do `New-Partition`, ou o
`Format-Volume` também mexe nele?** A resposta é dado novo, e o código depende
dela.

```powershell
Cap "Format-Volume 1 — NTFS 4096 ARCAVAULT"
Format-Volume -Partition $p1 -FileSystem NTFS  -NewFileSystemLabel 'ARCAVAULT' -AllocationUnitSize 4096 -Force -Confirm:$false | Out-Null

Cap "Format-Volume 2 — FAT32 4096 ARCABOOT"
Format-Volume -Partition $p2 -FileSystem FAT32 -NewFileSystemLabel 'ARCABOOT'  -AllocationUnitSize 4096 -Force -Confirm:$false | Out-Null

Cap "GptType DEPOIS de formatar"
CapOut (Get-Partition -DiskNumber $n | Select-Object PartitionNumber,Type,GptType,Offset,Size)
```

```powershell
Cap "Atribuindo letras"
foreach ($num in @($p1.PartitionNumber, $p2.PartitionNumber)) {
  try { Add-PartitionAccessPath -DiskNumber $n -PartitionNumber $num -AssignDriveLetter -ErrorAction Stop }
  catch { "particao ${num} : $($_.Exception.Message)" | Add-Content $CAP }
}
CapOut (Get-Partition -DiskNumber $n | Select-Object PartitionNumber,DriveLetter,Type,GptType,Offset,Size)
```

### A releitura, que é o que o `arca prepare` faz e o que o código vai transcrever

```powershell
Cap "RELEITURA — Get-Disk"
CapOut (Get-Disk -Number $n | Select-Object Number,FriendlyName,BusType,PartitionStyle,Size,IsSystem,IsBoot,LogicalSectorSize)

Cap "RELEITURA — Get-Partition"
CapOut (Get-Partition -DiskNumber $n | Select-Object PartitionNumber,DriveLetter,Type,MbrType,GptType,Offset,Size,IsActive,IsHidden)

Cap "RELEITURA — Get-Volume"
CapOut (Get-Partition -DiskNumber $n | Get-Volume | Select-Object DriveLetter,FileSystemLabel,FileSystem,AllocationUnitSize,Size,SizeRemaining)

Cap "RELEITURA — Win32_DiskDrive"
CapOut (Get-CimInstance Win32_DiskDrive | Where-Object Index -eq $n | Select-Object Index,Model,MediaType,InterfaceType,Size)
```

**Guarde as letras que saíram:**

```powershell
$LV = (Get-Partition -DiskNumber $n | Get-Volume | Where-Object FileSystemLabel -eq 'ARCAVAULT').DriveLetter
$LB = (Get-Partition -DiskNumber $n | Get-Volume | Where-Object FileSystemLabel -eq 'ARCABOOT').DriveLetter
"ARCAVAULT em ${LV}:  ·  ARCABOOT em ${LB}:" | Tee-Object -Append $CAP
```

---

## Etapa 5 — Instalar o Clonezilla na ARCABOOT

Extraia o **zip** (não o ISO) para a raiz da ARCABOOT:

```powershell
Expand-Archive -Path "$HOME\Downloads\clonezilla-live-3.3.3-15-amd64.zip" -DestinationPath "${LB}:\" -Force

Cap "O que ficou na ARCABOOT"
CapOut (Get-ChildItem "${LB}:\" | Select-Object Name,Length)
CapOut (Test-Path "${LB}:\EFI\boot\bootx64.efi")
```

O `Test-Path` acima **precisa** responder `True`. É o arquivo que o firmware
procura; sem ele não há o que bootar, e o resto do roteiro não faz sentido.

Deixe o menu do Clonezilla parando sozinho, para você ver que subiu:

```powershell
$cfg = "${LB}:\boot\grub\grub.cfg"
(Get-Content $cfg -Raw) -replace 'set timeout=\d+', 'set timeout=-1' | Set-Content $cfg -NoNewline

Cap "grub.cfg — timeout"
CapOut (Select-String -Path $cfg -Pattern 'set timeout' | Select-Object -First 3)
```

---

## Etapa 6 — Criar a entrada de firmware de teste

Ela nasce de uma cópia do `{bootmgr}`, como manda o
[ADR-0017](../docs/adr/0017-a-entrada-de-firmware-nasce-de-uma-copia-do-bootmgr.md):

```powershell
$saida = bcdedit /copy "{bootmgr}" /d "ARCA GPT TESTE"
$saida
$id = [regex]::Match($saida, '\{[0-9a-fA-F-]+\}').Value
"id da entrada de teste: $id" | Tee-Object -Append $CAP

bcdedit /set $id device "partition=${LB}:"
bcdedit /set $id path \EFI\boot\bootx64.efi

Cap "A entrada de teste, criada"
CapOut (bcdedit /enum $id)
```

**Anote o `$id`** — você vai precisar dele na Etapa 9 para remover a entrada.

Tire-a da ordem permanente e arme só o próximo boot — o mesmo desenho do
`bootsequence` que o ADR-0023 mediu:

```powershell
bcdedit /displayorder $id /remove
bcdedit /bootsequence $id

Cap "NVRAM com a entrada de teste, antes de reiniciar"
CapOut (bcdedit /enum firmware)
```

---

## Etapa 7 — O boot, que é o que decide

> Confira que `$CAP` está salvo antes de reiniciar — o arquivo está no
> repositório e sobrevive, mas as variáveis da sessão não.

Reinicie:

```powershell
Restart-Computer
```

**O que observar, e é só isto:**

| Resultado | O que significa |
|---|---|
| **O menu do Clonezilla aparece** | GPT boota nesta máquina. A mudança está provada, e a Etapa 9 é escrever o código. |
| Volta direto para o Windows | O firmware não aceitou a entrada. Vá para *Se não bootar*. |
| Erro de firmware / tela preta | Idem. Nada foi perdido — o Windows está intacto. |

**Não escolha nenhuma opção do menu.** Ver o menu é o teste inteiro.

Estando no menu, siga para a Etapa 8. Não querendo fazê-la agora, desligue pelo
botão e vá para a Etapa 9 — o teste principal já passou.

---

## Etapa 8 — Capturar a NVRAM de dentro do boot

É o que dá à mudança a mesma qualidade de evidência que as seis leituras do
ADR-0023. Vale o esforço: é a diferença entre "bootou uma vez" e "está medido".

No menu do Clonezilla, escolha a entrada padrão, e no modo escolha
**`Enter_shell`** (ou pressione `Ctrl+Alt+F2` para um terminal). Então:

```bash
sudo efibootmgr -v
sudo parted -l
sudo blkid
```

**Fotografe a tela** ou anote à mão o device path da entrada de teste. É a
linha que hoje, em MBR, lê:

```text
HD(2,MBR,0x4049dea9,0x1d9d2000,0x320000)
```

e que em GPT deve ler `HD(2,GPT,<guid>,...)`. Confirmar essa forma é o que
fecha o marco.

Querendo salvar em arquivo, monte a ARCAVAULT — o `blkid` acima mostra qual é,
pelo rótulo:

```bash
sudo mkdir -p /mnt/vault && sudo mount /dev/sdX1 /mnt/vault
sudo efibootmgr -v  > /mnt/vault/efibootmgr-gpt-2026-08-25.txt
sudo parted -l     >> /mnt/vault/efibootmgr-gpt-2026-08-25.txt
sudo umount /mnt/vault
```

Desligue pelo botão e volte ao Windows.

---

## Etapa 9 — Voltar ao normal

**Faça isto mesmo que o teste tenha falhado.** Deixar uma entrada morta na
NVRAM é o que o
[ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md) diz
não ser segurança.

```powershell
bcdedit /delete $id /f          # o $id anotado na Etapa 6
bcdedit /enum firmware          # confira: a entrada de teste sumiu
```

Perdido o `$id`, ache pela descrição:

```powershell
bcdedit /enum firmware | Select-String -Context 3,10 'ARCA GPT TESTE'
```

Depois: copie a captura para o repositório, registre-a em
`recursos/capturas/PROVENIENCIA.md` e reconecte o dispositivo de produção.

---

## Se não bootar

Nada foi perdido — o Windows está intacto e o dispositivo de produção nem foi
conectado. As causas, na ordem em que valem a pena investigar:

1. **A ARCABOOT não é uma ESP.** É a hipótese mais provável, e é a **Variante
   A**: refaça a Etapa 4 e, **depois** de formatar e de extrair o Clonezilla
   (Etapa 5), marque a partição como EFI System:

   ```powershell
   Set-Partition -DiskNumber $n -PartitionNumber $p2.PartitionNumber -GptType '{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}'
   ```

   Feito isso o Windows esconde a partição e tira a letra, então o
   `bcdedit /set $id device "partition=${LB}:"` da Etapa 6 deixa de valer — use
   `mountvol` para descobrir o `\Device\HarddiskVolumeN` e aponte por ele.
   **Marque só depois de extrair o Clonezilla**, ou não haverá como copiar nada
   para dentro dela.

2. **Secure Boot.** O `bootx64.efi` do Clonezilla é assinado, mas vale conferir
   no firmware se o teste falhou logo no início.

3. **O firmware não enumera o dispositivo.** Entre no menu de boot (F12) e veja
   se ele aparece como opção. Não aparecendo, o problema é anterior à tabela de
   partição.

Falhando também a Variante A, **o ADR-0014 estava certo** — e agora com medição
em vez de suposição. Registre a medição negativa: ela vale tanto quanto a
positiva, e é mais barata de guardar do que de refazer.

---

## O que fazer com o resultado

**Bootando**, o caminho no código é curto e mecânico, e cada item tem endereço:

| Onde | O que muda |
|---|---|
| `src/adaptadores/windows/particionador.rs:157` | `-PartitionStyle MBR` → `GPT`, e a remoção da MSR se a Etapa 3 mostrou uma |
| `src/preparacao.rs:573,576` | `TIPO_MBR_IFS`/`TIPO_MBR_FAT32_LBA` → as constantes de `GptType` medidas |
| `src/preparacao.rs:510-540` | a releitura passa a conferir `GptType`, e a MSR se ela existir |
| `src/comandos/prepare.rs:981` | o parágrafo "A estrutura e MBR, e nao GPT" sai da tela |
| `src/comandos/prepare.rs:1525` | o teste `o_plano_diz_por_que_e_mbr_e_nao_gpt` sai junto |
| `tests/e10_preparar_o_dispositivo.rs` | os 17 asserts passam a citar a captura nova |
| `src/duplos.rs:1120,1139` | os fixtures |
| `docs/adr/` | **ADR novo** que supersede o ADR-0014, com a captura como evidência |

O ADR novo precisa responder **três coisas** que só este roteiro mede:

1. o dispositivo GPT bootou, e o device path lido de dentro do boot é *qual*;
2. houve MSR, e o que foi feito com ela;
3. o `GptType` sai do `New-Partition` ou do `Format-Volume` — o análogo do
   achado do `MbrType 6`.

E há uma pendência que vale abrir como issue **independente do resultado**: o
limite de 2 TiB do MBR não tem defesa nem menção em lugar nenhum do código.
Ficando em MBR, ele precisa de uma recusa explícita no `prepare`; indo para
GPT, ele simplesmente deixa de existir — e vale registrar que deixou.
