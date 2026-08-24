<#
  Experimento P-19 + P-23 — 2026-08-24

  P-19  o firmware reescreve a entrada so quando ela e consumida por bootsequence?
        Passos 1a e 1b: um boot pelo dispositivo pela ORDEM PERMANENTE, sem
        bootsequence, com leitura do bcdedit imediatamente antes e depois.

  P-23  o corte do arca-restore.log cai sempre no mesmo lugar?
        Passos 2 (backup, que da a imagem) e 3 (restauracao, que da o log).

  Uso:  .\experimento-p19-p23.ps1 -Passo 1a
        .\experimento-p19-p23.ps1 -Passo 1b
        .\experimento-p19-p23.ps1 -Passo 2a
        .\experimento-p19-p23.ps1 -Passo 2b
        .\experimento-p19-p23.ps1 -Passo 3a

  Exige sessao elevada. Nao faz nada destrutivo fora do passo 3a, que APAGA
  o disco do sistema e devolve a imagem feita no passo 2a.
#>

[CmdletBinding()]
param(
  [Parameter(Mandatory=$true)]
  [ValidateSet('1a','1b','2a','2b','3a','3b')]
  [string]$Passo,

  [string]$Imagem = '2026-08-24_Ciclo'
)

$ErrorActionPreference = 'Stop'

$Repo     = 'C:\Users\Eduardo\Repository\ArcaBackup'
$Arca     = Join-Path $Repo 'target\release\arca.exe'
$Capturas = Join-Path $Repo 'recursos\capturas'
$Entrada  = '{f4057bd3-65a4-11f1-b0f1-aa4ed9bd2b34}'
$Vault    = 'D:'
$Docs     = 'D:\ARCA-DOCS'

function Exigir-Elevacao {
  $eu = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
  if (-not $eu.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Esta sessao nao esta elevada. Abra o terminal como administrador.'
  }
}

# Grava o `bcdedit /enum firmware` e devolve o texto + o SHA256 calculado pelo
# mesmo metodo das leituras de 24/08 (bytes UTF-8 do stdout, sem arquivo no meio).
function Capturar-Firmware([string]$Rotulo) {
  $texto = (bcdedit /enum firmware 2>&1 | Out-String)
  $bytes = [System.Text.Encoding]::UTF8.GetBytes($texto)
  $sha   = (Get-FileHash -InputStream ([System.IO.MemoryStream]::new($bytes)) -Algorithm SHA256).Hash
  $arq   = Join-Path $Capturas "bcdedit-enum-firmware-2026-08-24-$Rotulo.txt"
  [System.IO.File]::WriteAllText($arq, $texto, (New-Object System.Text.UTF8Encoding($false)))
  Write-Host ""
  Write-Host "  capturado ....... $arq"
  Write-Host "  SHA256 .......... $sha"
  [pscustomobject]@{ Texto = $texto; Sha = $sha; Arquivo = $arq }
}

# A entrada canonica que o firmware escreve — e a testemunha de P-19.
#
# Os dois testes sao `-cmatch`, e a razao e medida: `-match` e case-insensitive,
# e o que separa a forma do firmware da forma do bcdedit e exatamente a caixa —
# `\EFI\BOOT\BOOTX64.EFI` contra `\EFI\boot\bootx64.efi`.
#
# `Aplicativo de Firmware` sozinho NAO e rastro: as tres classes `UEFI:*` que o
# firmware enumera no POST usam o mesmo bloco, e elas vao e vem sem que ninguem
# escreva nada (24/08, P-22 e P-28). Sao contadas a parte, como ruido esperado.
function Procurar-Rastro-Do-Firmware([string]$Texto) {
  $achados = @()
  if ($Texto -cmatch 'UEFI OS')                   { $achados += 'descricao `UEFI OS`' }
  if ($Texto -cmatch '\\EFI\\BOOT\\BOOTX64\.EFI') { $achados += 'caminho `\EFI\BOOT\BOOTX64.EFI` em MAIUSCULAS' }
  $achados
}

function Contar-Classes-Do-Firmware([string]$Texto) {
  ([regex]::Matches($Texto, 'UEFI:(CD/DVD|Removable|Network)')).Count
}

function Contar-Entradas-Da-Ordem([string]$Texto) {
  if ($Texto -match '(?ms)^displayorder\s+(.*?)^timeout') {
    ([regex]::Matches($matches[1], '\{')).Count
  } else { 0 }
}

function Confirmar([string]$Palavra, [string]$Pergunta) {
  Write-Host ""
  Write-Host $Pergunta -ForegroundColor Yellow
  $lido = Read-Host "  digite $Palavra para seguir"
  if ($lido -ne $Palavra) { Write-Host "  abortado — nada foi feito."; exit 1 }
}

Exigir-Elevacao

switch ($Passo) {

  # ---------------------------------------------------------------- 1a
  '1a' {
    Write-Host "P-19 · passo 1a — boot pelo dispositivo SEM bootsequence" -ForegroundColor Cyan
    & $Arca status
    $a = Capturar-Firmware 'antes-do-boot-pela-ordem'

    if ($a.Texto -match 'bootsequence') {
      throw 'ha bootsequence no firmware. Rode `arca desarmar` antes: o experimento e sobre a ordem permanente.'
    }
    $rastro = Procurar-Rastro-Do-Firmware $a.Texto
    if ($rastro) {
      Write-Host ""
      Write-Host "  ATENCAO: ja ha rastro do firmware ANTES do experimento:" -ForegroundColor Yellow
      $rastro | ForEach-Object { Write-Host "    · $_" }
      Write-Host "  o passo 1b so pode afirmar sobre o que APARECER, e nao sobre o que ja estava."
    }

    Write-Host ""
    Write-Host "  O que vai acontecer:"
    Write-Host "    1. a entrada ARCA sobe ao topo da ordem permanente (a mao, metodo do ADR-0013)"
    Write-Host "    2. a maquina reinicia e boota no dispositivo PELA ORDEM, sem boot unico"
    Write-Host "    3. o dispositivo esta inerte: aparece o menu do Clonezilla, com timeout de 30 s"
    Write-Host '    4. DEIXE SUBIR o `Clonezilla live (VGA 800x600)` — e o default'
    Write-Host '    5. idioma, teclado (`Dont touch keymap`), e escolha `Enter_shell`'
    Write-Host "    6. la dentro, a medicao de P-19 por dentro do live:"
    Write-Host ""
    Write-Host "         sudo -i"
    Write-Host "         efibootmgr -v > /tmp/nvram.txt"
    Write-Host "         blkid | grep -i arcavault"
    Write-Host "         mkdir -p /mnt/v && mount /dev/disk/by-label/ARCAVAULT /mnt/v"
    Write-Host "         cp /tmp/nvram.txt /mnt/v/ARCA-LOGS/nvram-live-2026-08-24-sem-bootsequence.txt"
    Write-Host "         sync; umount /mnt/v; poweroff"
    Write-Host ""
    Write-Host '       se o mount recusar: `cat /tmp/nvram.txt` e fotografe — sao quatro linhas'
    Write-Host "    7. de volta no Windows, rode o passo 1b ANTES de qualquer comando do ARCA"
    Write-Host ""
    Write-Host "  Atalho, se nao quiser entrar no live: aperte uma seta para parar o timeout e"
    Write-Host "  desligue no botao. O experimento continua valido — so fica sem a leitura"
    Write-Host "  de dentro do boot, que e a que dispensa confiar no que o Windows fez depois."

    Confirmar 'BOOT' 'Isto reinicia a maquina agora. Nada e gravado em disco nenhum.'

    bcdedit /set '{fwbootmgr}' displayorder $Entrada /addfirst | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "o bcdedit recusou o /addfirst (codigo $LASTEXITCODE) — nada foi reiniciado." }
    $b = Capturar-Firmware 'ordem-suja-antes-do-boot'
    if ($b.Texto -notmatch [regex]::Escape($Entrada)) { throw 'a entrada ARCA nao aparece na ordem depois do /addfirst.' }
    if ($b.Texto -match 'bootsequence') { throw 'apareceu bootsequence depois do /addfirst — pare e investigue.' }

    Write-Host ""
    Write-Host "  ordem suja e conferida. Reiniciando..." -ForegroundColor Cyan
    shutdown /r /t 0
  }

  # ---------------------------------------------------------------- 1b
  '1b' {
    Write-Host "P-19 · passo 1b — a leitura de depois, antes de qualquer comando do ARCA" -ForegroundColor Cyan
    $d = Capturar-Firmware 'pos-boot-pela-ordem'

    $antes   = Join-Path $Capturas 'bcdedit-enum-firmware-2026-08-24-antes-do-boot-pela-ordem.txt'
    $texto_a = ''
    if (Test-Path $antes) {
      $texto_a = [System.IO.File]::ReadAllText($antes)
      $sha_a   = (Get-FileHash -InputStream ([System.IO.MemoryStream]::new([System.Text.Encoding]::UTF8.GetBytes($texto_a))) -Algorithm SHA256).Hash
      Write-Host ""
      Write-Host "  SHA de antes .... $sha_a"
      Write-Host "  SHA de depois ... $($d.Sha)"
    }

    $rastro = Procurar-Rastro-Do-Firmware $d.Texto
    Write-Host ""
    Write-Host "  entradas na ordem  antes: $(Contar-Entradas-Da-Ordem $texto_a) · depois: $(Contar-Entradas-Da-Ordem $d.Texto)"
    Write-Host "  classes UEFI:*     antes: $(Contar-Classes-Do-Firmware $texto_a) · depois: $(Contar-Classes-Do-Firmware $d.Texto)   (ruido conhecido: vao e vem sozinhas)"
    Write-Host ""
    if ($rastro) {
      Write-Host "  RASTRO DO FIRMWARE PRESENTE:" -ForegroundColor Yellow
      $rastro | ForEach-Object { Write-Host "    · $_" }
      Write-Host "  → um boot pela ordem permanente TAMBEM reescreve: P-19 fecha pela negativa."
    } else {
      Write-Host "  nenhum rastro do firmware: a entrada continua na forma que o bcdedit escreve." -ForegroundColor Green
      Write-Host "  → o boot SEM bootsequence nao reescreve. Com 22/08 do outro lado, P-19 fecha"
      Write-Host "    pela positiva — a reescrita e do consumo por bootsequence."
    }

    Write-Host ""
    Write-Host "  desfazendo a sujeira da ordem (o mesmo /addfirst {bootmgr} que C-13 faz)..."
    bcdedit /set '{fwbootmgr}' displayorder '{bootmgr}' /addfirst | Out-Null
    Capturar-Firmware 'ordem-desfeita-pos-boot' | Out-Null
    & $Arca status
  }

  # ---------------------------------------------------------------- 2a
  '2a' {
    Write-Host "P-23 · passo 2a — o backup que da a imagem a restaurar" -ForegroundColor Cyan
    Push-Location $Repo
    $sujo = (git status --porcelain)
    Pop-Location
    if ($sujo) {
      Write-Host ""
      Write-Host "  a arvore de trabalho tem mudancas nao commitadas:" -ForegroundColor Yellow
      $sujo | ForEach-Object { Write-Host "    $_" }
      Confirmar 'SEGUIR' 'O backup congela o C: como ele esta. Commitar antes deixa isto dentro da imagem.'
    }

    Capturar-Firmware 'antes-do-backup-ciclo' | Out-Null

    Write-Host ""
    Write-Host '  O `arca backup` desarma, pede confirmacao por extenso, arma e REINICIA sozinho.'
    Write-Host "  Ao terminar a maquina desliga. REMOVA O SSD antes de religar (C-9), religue,"
    Write-Host "  reconecte o SSD e rode o passo 2b."
    Confirmar 'ARMAR' "Vai armar o backup $Imagem e reiniciar."

    & $Arca backup $Imagem
  }

  # ---------------------------------------------------------------- 2b
  '2b' {
    Write-Host "P-23 · passo 2b — colher o backup e guardar as testemunhas" -ForegroundColor Cyan
    Capturar-Firmware 'pos-backup-ciclo' | Out-Null
    & $Arca resultado

    # o efi-nvram.dat e a leitura de dentro do live, COM bootsequence: o segundo
    # braco do experimento de P-19, no mesmo dispositivo e no mesmo dia.
    $nvram = Join-Path "$Vault\$Imagem" 'efi-nvram.dat'
    if (Test-Path $nvram) {
      Copy-Item $nvram (Join-Path $Capturas "efi-nvram-$Imagem.dat") -Force
      Write-Host "  efi-nvram.dat ... copiado (leitura de dentro do live, com bootsequence)"
      Write-Host "  SHA256 .......... $((Get-FileHash $nvram -Algorithm SHA256).Hash)"
      $antigo = Join-Path $Vault '2026-08-22_Apps\efi-nvram.dat'
      if (Test-Path $antigo) {
        Write-Host "  o de 22/08 ...... $((Get-FileHash $antigo -Algorithm SHA256).Hash)"
      }
    } else {
      Write-Host "  efi-nvram.dat ... NAO existe na imagem — anote, e o braco 2 de P-19 nao sai daqui" -ForegroundColor Yellow
    }

    $check = Join-Path "$Vault\$Imagem" 'arca-check.log'
    if (Test-Path $check) {
      Copy-Item $check (Join-Path $Capturas "arca-check-$Imagem.log") -Force
      Write-Host "  arca-check.log .. copiado ($((Get-Item $check).Length) bytes) — familia de P-23"
    }

    Write-Host ""
    Write-Host "  ATENCAO: tudo o que for medido daqui ate a restauracao morre nela, a menos"
    Write-Host "  que esteja no ARCAVAULT. O passo 3a copia $Capturas para $Docs."
  }

  # ---------------------------------------------------------------- 3a
  '3a' {
    Write-Host "P-23 · passo 3a — a restauracao, e ela APAGA o disco do sistema" -ForegroundColor Red
    if (-not (Test-Path (Join-Path $Vault $Imagem))) { throw "a imagem $Imagem nao esta no ARCAVAULT." }

    Push-Location $Repo
    $sujo   = (git status --porcelain)
    $status = (git status -sb | Select-Object -First 1)
    Pop-Location
    Write-Host ""
    Write-Host "  git ............. $status"
    if ($sujo) { $sujo | ForEach-Object { Write-Host "                    $_" } }

    if (-not (Test-Path $Docs)) { New-Item -ItemType Directory -Path $Docs | Out-Null }
    $destino = Join-Path $Docs 'capturas-antes-da-restauracao-2026-08-24'
    if (-not (Test-Path $destino)) { New-Item -ItemType Directory -Path $destino | Out-Null }
    Copy-Item (Join-Path $Capturas '*') $destino -Force -Recurse
    Write-Host "  capturas ........ copiadas para $destino (sobrevive a restauracao)"

    Capturar-Firmware 'antes-da-restauracao-ciclo' | Out-Null

    Write-Host ""
    Write-Host "  O disco 0 volta a ser a imagem $Imagem, feita ha minutos. O que se perde e o"
    Write-Host "  que foi feito no C: depois do backup. O ARCAVAULT nao e tocado."
    Confirmar 'RESTAURAR' 'Isto e destrutivo, e o `arca restore` ainda vai pedir o nome da imagem por extenso.'

    & $Arca restore $Imagem
  }

  # ---------------------------------------------------------------- 3b
  '3b' {
    Write-Host "P-23 · passo 3b — colher a restauracao e medir o log" -ForegroundColor Cyan
    Capturar-Firmware 'pos-restauracao-ciclo' | Out-Null
    & $Arca resultado

    $log = "$Vault\ARCA-LOGS\restauracao-$Imagem\arca-restore.log"
    if (-not (Test-Path $log)) { throw "nao ha $log — anote isto, porque e um achado por si so." }

    Copy-Item $log (Join-Path $Capturas "arca-restore-$Imagem.log") -Force
    Write-Host ""
    Write-Host "  log copiado ..... $(Join-Path $Capturas "arca-restore-$Imagem.log")"

    $bash = 'C:\Program Files\Git\bin\bash.exe'
    $medidor = Join-Path $Repo 'recursos/medir-arca-restore-log.sh'
    $emUnix = ($log -replace '^([A-Za-z]):', '/$1' -replace '\\','/')
    Write-Host ""
    & $bash $medidor $emUnix

    Write-Host ""
    Write-Host "  As capturas da sessao ficaram em $Capturas, e uma copia de antes da"
    Write-Host "  restauracao em $Docs — o C: voltou a ser a imagem, o ARCAVAULT nao."
  }
}
