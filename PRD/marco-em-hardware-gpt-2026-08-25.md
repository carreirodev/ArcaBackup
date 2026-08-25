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
| 2 | Escolher o alvo e conferir as quatro defesas | ✅ | 2026-08-25 · **duas vezes**: disco 2 e depois disco 1 |
| 3 | Apagar e inicializar em GPT | ✅ | 2026-08-25 · **houve MSR nos dois**, removida nos dois |
| 4 | Criar as duas partições | ✅ | 2026-08-25 · o `GptType` **não** muda ao formatar, e **não** distingue as duas |
| 5 | Instalar o Clonezilla na ARCABOOT | ✅ | 2026-08-25 · `bootx64.efi` no lugar; o `grub.cfg` tinha aspas |
| 6 | Criar a entrada de firmware de teste | ✅ | 2026-08-25 · `partition=E:` pegou no SSD, `bootsequence` armado |
| 7 | **O boot, que é o que decide** | ✅ | 2026-08-25 · **o menu do Clonezilla subiu** |
| 8 | Capturar a NVRAM de dentro do boot | ✅ | 2026-08-25 · `HD(2,GPT,9c86b84a-…,0x1d9d3000,0x320000)` |
| 9 | Voltar ao normal | ✅ | 2026-08-25 · **três** entradas removidas, NVRAM como na Etapa 1 |

> # O dispositivo GPT bootou, e o marco está fechado.
>
> Em 25/08/2026 o boot único levou o firmware ao dispositivo e **o menu do
> Clonezilla subiu**. Sem tela preta, sem erro de firmware, sem volta direta
> para o Windows. Um dispositivo GPT com ARCABOOT FAT32 *Basic Data*, apontado
> por `partition=E:` e `\EFI\boot\bootx64.efi`, boota nesta máquina.
>
> **E o device path foi lido de dentro do boot**, num segundo reinício:
>
> ```text
> HD(2,GPT,9c86b84a-596f-47e6-b92a-cd5b84b4a1fe,0x1d9d3000,0x320000)/\EFI\BOOT\BOOTX64.EFI
> ```
>
> contra o `HD(2,MBR,0x4049dea9,0x1d9d2000,0x320000)` que o ADR-0023 mediu. A
> partição continua a 2, `MBR` vira `GPT`, a assinatura do **disco** dá lugar ao
> PARTUUID da **partição**, e o tamanho `0x320000` é idêntico — os 1600 MiB
> fixos da ARCABOOT.
>
> **O ADR-0014 não tinha razão, e agora isso está medido em vez de suposto.** Ele
> dizia que a falha "só se descobre depois de o Windows já ter sido apagado" —
> não só se descobre antes como não houve falha. As **três** perguntas estão
> respondidas, e a Etapa 9 devolveu a NVRAM ao estado da Etapa 1.

**O alvo é o disco 1, KGSSE100 256** — SSD externo USB, GPT, ARCAVAULT em `D:` (NTFS 4096,
254 381 391 872) e ARCABOOT em `E:` (FAT32 4096, 1 677 721 600), com o Clonezilla
extraído, o `E:\EFI\boot\bootx64.efi` no lugar e o `grub.cfg` em
`set timeout="-1"`. A NVRAM tem o `{fwbootmgr}` com `displayorder` inalterado —
só o `{bootmgr}` — e `bootsequence` apontando para a entrada de teste. A captura
viva é
[`recursos/capturas/medicao-gpt-2026-08-25.txt`](../recursos/capturas/medicao-gpt-2026-08-25.txt),
registrada em `PROVENIENCIA.md` com o SHA256 do arquivo **parado na Etapa 6**.

> **Trocou-se de dispositivo no meio, e a troca é que produziu o melhor achado
> do dia.** O roteiro correu inteiro no Kingston DataTraveler Max e emperrou na
> Etapa 6: o `bcdedit /set device` responde *"A operação foi concluída com
> êxito"* e **não escreve**, para qualquer partição daquele disco e por qualquer
> forma. Três controles depois — o `C:` pega, o `F:` de um SSD externo em MBR
> pega, e o **mesmo SSD convertido para GPT também pega** — a causa está isolada:
> **não é o GPT, é aquele pendrive.** É o C-6 que `prepare.rs:678` já previa, e
> agora ele tem um caso concreto com nome e modelo.
>
> Das três perguntas que o ADR novo precisa responder — no fim deste documento —,
> **duas estão respondidas, e cada uma foi medida duas vezes**, em dois
> dispositivos, com números idênticos. A primeira é da Etapa 7.

### A suíte foi o oitavo instrumento de medição, e pegou coisa que ninguém viu

Enquanto o marco esteve montado, **oito** testes ficaram vermelhos — e nenhum
por regressão. Os oito leem a máquina de verdade, e ela estava, de propósito, no
estado que eles existem para acusar. O interessante é o que cada grupo custou
para apagar:

| Teste | Por quê | O que o apagou |
|---|---|---|
| `e2:a_leitura_do_firmware_nao_arma_nada` | havia `bootsequence` armado | a Etapa 9 |
| `e7:nao_ha_boot_unico_pendente_nesta_maquina` | o mesmo `bootsequence` | a Etapa 9 |
| `e4:o_grub_cfg_do_dispositivo_e_um_inerte_conhecido` | o `grub.cfg` era o do pacote **cru** | o `desarmar` que a Etapa 5 esquecia |
| `e4:o_dispositivo_esta_inerte_agora` | idem | idem |
| `e4:desarmar_o_grub_cfg_do_dispositivo_nao_mudaria_um_byte` | idem | idem |
| `e7:o_grub_cfg_do_dispositivo_continua_inerte_e_e_um_dos_conhecidos` | idem | idem |
| `e7:armar_e_desarmar_o_dispositivo_de_verdade_se_cancelam` | idem | idem |
| `e7:a_entrada_do_arca_existe_nesta_maquina_e_e_a_propria` | **anterior a este roteiro**: a entrada `ARCA` não sobreviveu à reinstalação do Windows | *continua vermelho* |

Hoje passam **861 de 862**, e o que sobra é de outra história.

**Os cinco do meio é que valem.** Eles não estavam acusando o `timeout=-1`, como
parecia: acusavam que o `grub.cfg` do dispositivo era o do zip **cru**, byte a
byte, e não o que o `arca prepare` instalaria. Ver a Etapa 5.

**E há um detalhe que vale saber:** os testes de E4 e E7 acham "o dispositivo"
pelo rótulo, e o disco de teste deste roteiro tem uma partição `ARCABOOT` e uma
`ARCAVAULT`. Eles não estavam lendo o dispositivo de produção — que nem foi
conectado —, estavam lendo **este**. O rótulo não distingue um do outro, e isso
é uma pergunta em aberto para o código, não para o roteiro.

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

**A Variante B se confirmou em 25/08/2026, nas Etapas 4 a 6.** A ARCABOOT nasceu
`{ebd0a0a2-…}` como previsto, o Windows lhe deu letra, e o
`bcdedit /set device partition=E:` foi aceito e relido — as três linhas da coluna
da esquerda são medição agora, e não expectativa. Falta só a linha que nenhuma
das duas colunas antecipa: se o firmware boota.

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

- [x] Um **segundo** SSD/HD externo, cujo conteúdo você pode perder — começou no
      Kingston DataTraveler Max de 238,5 GB e terminou no **KGSSE100 256**, do
      mesmo tamanho; o porquê da troca está na Etapa 6
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
> **E ela se repetiu.** Horas depois, no KGSSE100 256 — outro fabricante de
> controladora, outra porta —, o `Initialize-Disk -PartitionStyle GPT` criou uma
> MSR **de novo**, com o mesmo `GptType`, o mesmo offset 17 408 e os mesmos
> 16 759 808 bytes. Duas medições independentes e idênticas: a MSR não é acidente
> de dispositivo, é o que o Windows faz em GPT, e `particionador.rs` tem de
> removê-la sempre — não "se houver".
>
> **A GPT cobra 1 400 832 bytes.** Num disco de 256 060 514 304, o
> `LargestFreeExtent` depois de remover a MSR é **256 059 113 472** — a tabela
> primária no começo e a cópia secundária no fim. É esse o número que a Etapa 4
> usa nas contas, e a razão de elas não saírem de constante. Os dois discos
> mediram o mesmo, o que é esperado: eles têm o mesmo tamanho.
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

## Etapa 4 — Criar as duas partições ✅

> **Feita em 25/08/2026, e ela responde a terceira pergunta: o `GptType` sai
> pronto do `New-Partition`.** As duas partições nasceram com
> `{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}` — *Basic Data* —, e o `Format-Volume`
> **não mexeu nele**. É o contrário do MBR, onde as duas nascem com `MbrType 6` e
> só chegam a 7 e a 12 depois de formatar. Em GPT não há esse segundo passo, e a
> releitura pode conferir o tipo logo depois de criar.
>
> **E há um achado que ninguém tinha pensado em perguntar: o `GptType` não
> distingue as duas.** Em MBR, `7` (IFS) e `12` (FAT32 LBA) separavam a ARCAVAULT
> da ARCABOOT, e é disso que vivem as constantes de `preparacao.rs:573,576`. Em
> GPT as duas têm **o mesmo tipo**, e a releitura deixa de poder dizer qual é qual
> por aí — sobram o rótulo, o sistema de arquivos e a ordem no disco. Isso muda a
> linha `preparacao.rs:573,576` da tabela do fim: não é trocar duas constantes por
> outras duas, é trocar o critério.
>
> Os números medidos, e são eles que o código transcreve:
>
> | | ARCAVAULT | ARCABOOT |
> |---|---|---|
> | partição | 1 | 2 |
> | offset | 1 048 576 | 254 382 440 448 |
> | tamanho | 254 381 391 872 | 1 677 721 600 |
> | `GptType` | `{ebd0a0a2-…}` | `{ebd0a0a2-…}` |
> | `MbrType` | *vazio* | *vazio* |
> | `IsActive` | `False` | `False` |
> | letra | `D:` | `E:` |
>
> `IsActive` sai `False` nas duas, como saía em MBR — o boot continua UEFI puro, e
> `particionador.rs:659` continua certo em não passar `-IsActive`. O `MbrType` sai
> **vazio**, e não zero: quem o lê em GPT lê ausência.
>
> **Esta tabela foi medida duas vezes, e as duas bateram em tudo** — nos dois
> discos de 256 060 514 304 bytes, os mesmos offsets, os mesmos tamanhos, o mesmo
> `GptType`, as mesmas letras. Os números acima são do KGSSE100, e os do
> DataTraveler estão na captura, idênticos.

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

## Etapa 5 — Instalar o Clonezilla na ARCABOOT ✅

> **Feita em 25/08/2026, nos dois dispositivos.** O `Expand-Archive` levou 3,1 s
> num e 3,7 s no outro, o `E:\EFI\boot\bootx64.efi` existe com 1 088 816 bytes, e
> sobraram 1 101 430 784 dos 1 673 527 296 da ARCABOOT — os mesmos bytes nos dois.
> O Clonezilla cabe folgado nos 1600 MiB, e a conta da Etapa 4 não precisa mudar.
>
> **O `-replace` do `grub.cfg` abaixo não funciona, e é um bug do roteiro.** A
> linha real é `set timeout="30"`, com o número **entre aspas**, e o padrão
> `set timeout=\d+` não casa com aspas. O comando respondeu sem erro e não mudou
> nada — quem pegou foi a releitura. O que casa é `set timeout="?-?\d+"?`, e é o
> que está no bloco abaixo.
>
> **E faltava uma coisa maior, que a suíte é que pegou: o `desarmar`.** Esta
> etapa extrai o zip e para aí. O `arca prepare` não: `prepare.rs:228` faz
> `grub::desarmar(&do_pacote)` e instala o **resultado** — é literalmente o nome
> do [ADR-0018](../docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md),
> *"o pacote é o zip e o prepare desarma o que instala"*. Sem esse passo, o
> dispositivo fica com o `grub.cfg` cru, que **não é** nenhum dos dois inertes
> que o projeto conhece, e cinco testes de E4 e E7 acusam.
>
> A diferença é uma linha: o `desarmar` aponta o `set default` para o `menuentry`
> inerte. Depois de extrair, o roteiro tem de fazer o que o `prepare` faz —
> `set default="0"` → `set default="live-default"`. Feito isso, os cinco passam.

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
(Get-Content $cfg -Raw) -replace 'set timeout="?-?\d+"?', 'set timeout="-1"' | Set-Content $cfg -NoNewline

Cap "grub.cfg — timeout"
CapOut (Select-String -Path $cfg -Pattern 'set timeout' | Select-Object -First 3)
```

---

## Etapa 6 — Criar a entrada de firmware de teste ✅

> **Feita em 25/08/2026, na segunda tentativa e noutro dispositivo — e o caminho
> até ela vale mais do que o resultado.**
>
> No DataTraveler Max, o `bcdedit /set <id> device partition=E:` respondeu *"A
> operação foi concluída com êxito"*, código 0, e a releitura mostrou o device
> **antigo**: `partition=\Device\HarddiskVolume1`, a ESP do Windows. Repetido,
> mesma coisa. Por `\Device\HarddiskVolume9` — o caminho real daquela partição —,
> mesma coisa. O `path`, no mesmo comando e na mesma entrada, pegou de primeira:
> não era a entrada estar travada, era o elemento `device`.
>
> **Isto é o C-6, e `prepare.rs:678` já o tinha escrito** — *"o sucesso do
> `bcdedit` nunca é prova, e com mídia removível ele responde êxito mantendo o
> valor antigo"*. O que faltava era um caso concreto. Três controles o cercaram:
>
> | Alvo | Disco | Esquema | Pegou? |
> |---|---|---|---|
> | `partition=C:` | 0, NVMe interno | GPT | **sim** |
> | `partition=D:` / `partition=E:` | 2, DataTraveler Max | GPT | não |
> | `partition=F:` | 1, KGSSE100 256 | MBR | **sim** |
> | `partition=E:` | 1, KGSSE100 256 | **GPT** | **sim** |
>
> A primeira linha mata "é o GPT". A terceira mata "é o USB". **A quarta é a que
> decide**: mesmo disco, mesma porta, mesma FAT32 com o mesmo Clonezilla dentro,
> só o esquema mudou de MBR para GPT — e pegou. Sobra um culpado só, e ele tem
> nome: o **Kingston DataTraveler Max**. O ADR-0014 não tinha razão nisto.
>
> **De quebra, uma ambiguidade fechou.** No primeiro controle, apontar por
> `\Device\HarddiskVolumeN` deu "não pegou" sem dar para dizer se era recusa ou
> se o `bcdedit` normalizava o caminho de volta para a letra. No SSD deu para
> dizer: o `/set` do `\Device\…` do `F:` foi relido como `partition=F:`. É
> **normalização**, e não recusa — o que confirma o `Alvo::ParticaoSemLetra` de
> `firmware.rs:79` como forma de escrita válida.
>
> **E o roteiro tinha um bug de verdade nos dois comandos do fim.** O
> `bcdedit /displayorder <id> /remove` e o `bcdedit /bootsequence <id>` sem alvo
> operam sobre o **`{bootmgr}`**, o gerenciador do Windows — não sobre o
> `{fwbootmgr}`, que é o do firmware. O desenho medido do projeto é outro:
> `armar.rs:459` faz `/set {fwbootmgr} bootsequence <id>` e `prepare.rs:835` faz
> `/set {fwbootmgr} displayorder <id> /remove`. Deixado como o roteiro escrevia,
> o próximo boot mandaria o **Windows Boot Manager** para um `bootx64.efi` que a
> ESP do sistema não tem. Os blocos abaixo já estão corrigidos.

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
bcdedit /set "{fwbootmgr}" displayorder $id /remove
bcdedit /set "{fwbootmgr}" bootsequence $id

Cap "NVRAM com a entrada de teste, antes de reiniciar"
CapOut (bcdedit /enum "{fwbootmgr}")
CapOut (bcdedit /enum firmware)
```

O `{fwbootmgr}` tem de sair com `displayorder` trazendo **só** o `{bootmgr}` — é
C-5, a ordem permanente não muda — e `bootsequence` apontando para o `$id`. Foi
assim que ele saiu em 25/08/2026.

---

## Etapa 7 — O boot, que é o que decide

> **Feita em 25/08/2026, e passou.** O menu do Clonezilla subiu. A Etapa 8 não
> foi feita nessa passada — quem operava desligou antes de entrar no shell —, e
> por isso o boot foi rearmado.
>
> **O ciclo de boot mexeu na ordem permanente, e isso é o ADR-0009 medido com o
> antes e o depois na mesma noite.** O `bootsequence` foi consumido, como boot
> único deve ser. Mas o `displayorder` do `{fwbootmgr}` voltou com **três
> entradas a mais**, que não estavam lá antes de reiniciar:
>
> | Identificador | `description` | `device` |
> |---|---|---|
> | `{31cc955f-…}` | `UEFI:CD/DVD Drive` | *nenhum* |
> | `{31cc9560-…}` | `UEFI:Removable Device` | *nenhum* |
> | `{31cc9561-…}` | `UEFI:Network Device` | *nenhum* |
>
> As três são `Aplicativo de Firmware (101fffff)`, têm `description` e **não têm
> `device`** — são exatamente as entradas sem alvo do
> [ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md),
> que o firmware resolve no POST pelo que estiver conectado. Quem as pôs na ordem
> foi o ciclo de boot, e não uma pessoa. **Nada disso foi desfeito**, nem para
> rearmar: C-5 vale para o ARCA, que lê a ordem permanente e não mexe nela.
>
> A entrada de teste **não** voltou para o `displayorder` — some da ordem e fica
> só no store, intacta. Rearmar foi só o `bootsequence`, com a ordem permanente
> conferida igual antes e depois.

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

> **Feita em 25/08/2026 às 15:06 UTC**, no segundo boot, pelo script deixado em
> `D:\etapa8.sh`. A saída crua está em
> [`recursos/capturas/efibootmgr-gpt-2026-08-25.txt`](../recursos/capturas/efibootmgr-gpt-2026-08-25.txt).
>
> **O device path é o do topo deste documento**, e responde a primeira das três
> perguntas na forma que o roteiro previa. Mas a etapa rendeu mais três coisas
> que ninguém tinha pensado em perguntar:
>
> **1. Bootou pela entrada do dispositivo com o Windows à frente da ordem.**
> `BootCurrent: 0001`, `BootOrder: 0000,0001` — os **mesmos dois números** que
> `armar.rs:441` registra para o marco em MBR. O `bootsequence` pega em GPT do
> mesmo jeito, e C-5 continua sustentável: não há troca a fazer entre armar e
> não mexer na ordem permanente.
>
> **2. A entrada pela qual se bootou não é a que o `bcdedit` criou.** O
> `efibootmgr` viu duas variáveis `Boot####`, e só duas: `Boot0000 Windows Boot
> Manager` e `Boot0001 UEFI OS` — esta apontando para o mesmo
> `HD(2,GPT,…)/\EFI\BOOT\BOOTX64.EFI`, com o caminho em **maiúsculas**. A entrada
> `ARCA GPT TESTE`, que o `bcdedit` criou com o caminho em minúsculas, **não
> estava na NVRAM** naquele momento. Quem bootou foi uma entrada que o firmware
> fez sozinho.
>
> **3. O Linux também não distingue as duas partições pelo tipo.** O `lsblk` dá
> `PARTTYPE ebd0a0a2-…` para as duas, o `parted` chama as duas de *Basic data
> partition* com flag `msftdata`, o `gdisk` dá código `0700` para as duas. O
> achado da Etapa 4 não é peculiaridade do PowerShell — é a tabela de partição.

O roteiro da etapa fica no próprio dispositivo, em `D:\etapa8.sh`, para não
depender de digitar comando nenhum no live: uma linha monta a ARCAVAULT pelo
rótulo, roda o script, e ele salva a saída ao lado.

No menu do Clonezilla, escolha a entrada padrão, aceite idioma e teclado, e no
menu de modo escolha **`Enter_shell`** — *Enter command line prompt*. Serve
também `Ctrl+Alt+F2` a qualquer momento depois do boot. Então, **uma linha só**:

```bash
sudo mkdir -p /mnt/vault && sudo mount -L ARCAVAULT /mnt/vault && sudo bash /mnt/vault/etapa8.sh
```

Ele imprime na tela o `BootCurrent`, o `BootOrder` e a linha da entrada de
teste, e grava `efibootmgr -v`, `parted -l`, `blkid`, `lsblk` e `sgdisk -p` em
`/mnt/vault/etapa8-saida.txt`. Ao terminar:

```bash
sudo umount /mnt/vault
```

Não havendo o script — outro dispositivo, outra passada —, os comandos crus são:

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

> **Feita em 25/08/2026, e ela não foi o `/delete` de uma linha que este bloco
> promete.** Eram **três** entradas apontando para `partition=E:`, não uma, e o
> caminho até descobrir isso é o achado mais desconfortável do roteiro.
>
> **O identificador que o `bcdedit /enum firmware` mostra não é identidade.** O
> `{31cc955f-a0ae-11f1-8a54-806e6f6e6963}` era, antes do segundo boot,
> `UEFI:CD/DVD Drive` **sem `device`**. Depois do segundo boot, o **mesmo GUID**
> era `ARCA GPT TESTE`, `device partition=E:`, `path \EFI\boot\bootx64.efi`. Ele
> nomeia o *slot* `Boot####` da NVRAM, e não a entrada que está nele — e o
> firmware reescreveu os slots entre um boot e outro.
>
> A consequência prática é direta: **deletar por lista guardada acertaria o slot
> errado**. A limpeza foi feita relendo o firmware a cada passo e escolhendo pelo
> que a leitura corrente dizia, com uma recusa de segurança contra qualquer alvo
> que se parecesse com o do sistema. Três voltas, três removidas, e a NVRAM
> voltou aos dois blocos que a Etapa 1 mediu, com o `{bootmgr}` conferido campo a
> campo.
>
> **E o `displayorder` do `bcdedit` não previu o comportamento do firmware.** Ele
> trazia a entrada de teste em **primeiro**, na frente do `{bootmgr}`, com
> `timeout 1` — pela leitura do `bcdedit`, o religamento seguinte iria para o
> dispositivo sozinho. Foi medido o contrário: com o SSD conectado, a máquina
> entrou no Windows. E o `efibootmgr`, lido de dentro do boot, media
> `BootOrder: 0000,0001` — o Windows primeiro. Quem acertou o comportamento foi o
> `efibootmgr`. Isso encosta no
> [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md), e é
> pergunta em aberto, não conclusão.

**Faça isto mesmo que o teste tenha falhado.** Deixar uma entrada morta na
NVRAM é o que o
[ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md) diz
não ser segurança.

```powershell
$id = "{f4057bd6-65a4-11f1-b0f1-aa4ed9bd2b34}"   # o anotado na Etapa 6
bcdedit /set "{fwbootmgr}" displayorder $id /remove
bcdedit /delete $id /f
bcdedit /enum firmware          # confira: a entrada de teste sumiu
```

O `/remove` antes do `/delete` não é supérfluo: o ciclo de boot põe a entrada de
volta no `displayorder` do `{fwbootmgr}` por conta própria — é o
[ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md) —, e
deletar sem tirar de lá deixaria um identificador órfão na ordem. A NVRAM tem de
voltar a **duas** entradas: o `{fwbootmgr}` e o `{bootmgr}`, como a Etapa 1 mediu.

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
| `src/adaptadores/windows/particionador.rs:157` | `-PartitionStyle MBR` → `GPT`, e a remoção da MSR **sempre** — ela apareceu nos dois dispositivos |
| `src/preparacao.rs:573,576` | **troca de critério, não de constante**: o `GptType` é o mesmo nas duas, e não separa a ARCAVAULT da ARCABOOT |
| `src/preparacao.rs:510-540` | a releitura confere `GptType` como tipo *comum*, e passa a distinguir as duas por rótulo/sistema de arquivos/ordem |
| `src/comandos/prepare.rs:981` | o parágrafo "A estrutura e MBR, e nao GPT" sai da tela |
| `src/comandos/prepare.rs:1525` | o teste `o_plano_diz_por_que_e_mbr_e_nao_gpt` sai junto |
| `tests/e10_preparar_o_dispositivo.rs` | os 17 asserts passam a citar a captura nova |
| `src/duplos.rs:1120,1139` | os fixtures |
| `docs/adr/` | **ADR novo** que supersede o ADR-0014, com a captura como evidência |

O ADR novo precisa responder **três coisas** que só este roteiro mede:

1. o dispositivo GPT bootou, e o device path lido de dentro do boot é *qual* —
   **respondida**: bootou, o menu do Clonezilla subiu, e o device path é
   `HD(2,GPT,9c86b84a-596f-47e6-b92a-cd5b84b4a1fe,0x1d9d3000,0x320000)`. O GUID
   é o **PARTUUID da partição**, e não a assinatura do disco que o MBR usava;
2. houve MSR, e o que foi feito com ela — **respondida duas vezes**: houve nos
   dois dispositivos, `{e3c9e316-…}`, offset 17 408, 16 759 808 bytes, e foi
   removida nos dois. O código a remove sempre, não "se houver";
3. o `GptType` sai do `New-Partition` ou do `Format-Volume` — **respondida duas
   vezes**: sai do `New-Partition`, e o `Format-Volume` não encosta nele. E a
   resposta traz uma pergunta que ninguém tinha feito: em GPT o tipo **não
   distingue** a ARCAVAULT da ARCABOOT, e é isso que muda a linha
   `preparacao.rs:573,576` da tabela acima.

E o ADR precisa registrar uma quarta coisa, que não estava prevista e que este
roteiro mediu por acidente: **o `bcdedit` recusa em silêncio o Kingston
DataTraveler Max**, e o C-6 deixou de ser uma precaução abstrata para virar um
caso com nome, modelo e quatro controles. A defesa que já existe —
`Erro::AlvoDoFirmwareRecusado`, em `prepare.rs:694` — é a única coisa entre esse
silêncio e um dispositivo que o `arca prepare` diria ter preparado.

E uma quinta, que é a mais incômoda das cinco: **o identificador que o
`bcdedit /enum firmware` devolve não é identidade.** O mesmo GUID trocou de
descrição e de `device` entre dois boots, porque nomeia o slot `Boot####` e não
a entrada. Todo lugar do código que guarda um identificador de firmware e o usa
depois — `armar.rs` entre o `/set` e a releitura, `prepare.rs` entre o `/copy` e
os três `/set`, o `$id` anotado para a Etapa 9 — está apostando numa estabilidade
que **não foi medida**. Dentro de uma mesma sessão, sem reinício, nada disso
mudou nas seis vezes em que este roteiro releu; entre boots, mudou. É a diferença
que o ADR novo precisa registrar, e a que separa uma releitura de C-3 que
protege de uma que só parece proteger.

E há duas pendências que valem issues **independentes do resultado**:

- **O limite de 2 TiB do MBR** não tem defesa nem menção em lugar nenhum do
  código. Ficando em MBR, ele precisa de uma recusa explícita no `prepare`; indo
  para GPT, ele simplesmente deixa de existir — e vale registrar que deixou.
- **A recusa silenciosa do `bcdedit`** merece uma mensagem que diga o que fazer.
  Hoje o `AlvoDoFirmwareRecusado` diz *esperado* e *tem*; quem lê a tela com um
  DataTraveler na mão não tem como saber que o problema é o dispositivo.
