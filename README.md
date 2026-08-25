# ARCA

**Automatizador de Clonezilla para backup e restauração de imagem de disco.**

O ARCA transforma um SSD externo qualquer num **dispositivo** autocontido — Clonezilla e imagens juntos, no mesmo disco — e dispara operações nele com um comando e uma confirmação digitada. O procedimento manual equivalente são ~20 telas em modo texto, em inglês técnico, das quais errar em duas destrói o disco.

> **O ARCA nunca lê nem escreve disco.** Quem lê e escreve disco é o Clonezilla, do outro lado de um reinício. O ARCA prepara o ambiente, monta a **receita**, arma o **boot único**, e na volta **colhe** o que o Clonezilla deixou escrito em arquivo.

```
   Windows                          │  reinício  │        Clonezilla (Linux)
─────────────────────────────────── │            │ ───────────────────────────────
 arca backup 2026-08-24_Apps        │            │
   ├─ desarma o que houver          │            │
   ├─ pré-voo (espaço, nome, disco) │            │
   ├─ pede confirmação por extenso  │            │
   ├─ grava a receita no grub.cfg   │            │
   ├─ marca o boot único no firmware│            │
   └─ shutdown /r /t 0  ────────────┼───────────▶│  boota no dispositivo
                                    │            │  monta LABEL=ARCAVAULT
                                    │            │  executa a receita sozinho
                                    │            │  escreve arca-fim.txt
                                    │            │  poweroff
 arca resultado  ◀──────────────────┼────────────┘  (você religa a máquina)
   ├─ lê o desfecho e confere o selo
   ├─ desarma o dispositivo
   └─ devolve o Windows ao topo da ordem de boot
```

---

## Índice

1. [O que o ARCA é, e o que não é](#1-o-que-o-arca-é-e-o-que-não-é)
2. [Requisitos](#2-requisitos)
3. [Compilação — passo a passo](#3-compilação--passo-a-passo)
4. [Instalação e primeira execução](#4-instalação-e-primeira-execução)
5. [Anatomia de um dispositivo ARCA](#5-anatomia-de-um-dispositivo-arca)
6. [Os nove comandos, um por um](#6-os-nove-comandos-um-por-um)
   - [`arca prepare`](#61-arca-prepare) — prepara o dispositivo
   - [`arca sondar`](#62-arca-sondar) — descobre os discos desta máquina
   - [`arca backup`](#63-arca-backup) — arma o backup e reinicia
   - [`arca resultado`](#64-arca-resultado) — colhe o desfecho
   - [`arca list`](#65-arca-list) — lista as imagens
   - [`arca verify`](#66-arca-verify) — confere uma imagem
   - [`arca restore`](#67-arca-restore) — restaura, e apaga o disco
   - [`arca status`](#68-arca-status) — diagnóstico
   - [`arca desarmar`](#69-arca-desarmar) — devolve o estado inerte
7. [As flags globais](#7-as-flags-globais)
8. [Workflow completo — do disco virgem ao Windows restaurado](#8-workflow-completo--do-disco-virgem-ao-windows-restaurado)
9. [As quatro receitas, byte a byte](#9-as-quatro-receitas-byte-a-byte)
10. [Todos os arquivos que o ARCA lê e escreve](#10-todos-os-arquivos-que-o-arca-lê-e-escreve)
11. [Códigos de saída](#11-códigos-de-saída)
12. [Quando dá errado — diagnóstico e recuperação](#12-quando-dá-errado--diagnóstico-e-recuperação)
13. [As regras que o ARCA nunca quebra](#13-as-regras-que-o-arca-nunca-quebra)
14. [Arquitetura do código](#14-arquitetura-do-código)
15. [Testes, exemplos e medições](#15-testes-exemplos-e-medições)
16. [Documentação do projeto](#16-documentação-do-projeto)
17. [Glossário](#17-glossário)

---

## 1. O que o ARCA é, e o que não é

### É

- Uma ferramenta de **linha de comando** para Windows, escrita em Rust.
- Um **preparador de dispositivos**: `arca prepare` pega um disco externo qualquer, particiona, instala o Clonezilla e cria a entrada de boot.
- Um **disparador de operações desatendidas**: backup, restauração, verificação e sondagem — cada uma é um boot único que roda sozinho e desliga a máquina no fim.
- Um **colhedor de desfechos**: na volta, ele lê o arquivo que o Clonezilla deixou, confere o selo, e diz o que aconteceu.

### Não é

| ❌ | Por quê |
|---|---|
| Catálogo ou banco de imagens | Se a informação está na listagem de diretórios do dispositivo, não há o que armazenar |
| Backup incremental ou diferencial | O Clonezilla faz imagem de disco inteiro |
| Agendador | Não há serviço, não há tarefa agendada, não há daemon |
| Retenção automática | O ARCA **nunca apaga nada** (B-10) |
| Interface gráfica | Sem GUI, sem instalador, sem banco |
| Gerenciador de discos de uso geral | `arca prepare` particiona **o dispositivo** e só ele — disco fixo é recusa dura, sem opção de forçar |

### Não suporta

BIOS legada · BitLocker · RAID · Storage Spaces.

---

## 2. Requisitos

### Para compilar

| Item | Versão | Observação |
|---|---|---|
| **Rust** | ≥ 1.85 | `edition = "2024"`; o `rust-version` do `Cargo.toml` cobra isso |
| **Toolchain** | `x86_64-pc-windows-msvc` | O manifesto de elevação é embutido pelo linker MSVC (`/MANIFEST:EMBED`) |
| **Visual Studio Build Tools** | qualquer versão com "Desenvolvimento para desktop com C++" | Fornece `link.exe`, que o toolchain MSVC usa |
| **Git** | qualquer | **Opcional.** Sem ele o `--version` diz `sem git` e a compilação segue |

> Este repositório foi compilado e testado com `rustc 1.98.0` / `cargo 1.98.0`.

### Para executar

| Item | Observação |
|---|---|
| **Windows 10/11 x64, UEFI** | O ARCA fala com o firmware pelo `bcdedit` |
| **Privilégio administrativo** | **Todos** os comandos exigem. O manifesto embutido faz o Windows pedir o UAC sozinho |
| Ferramentas do próprio Windows | `bcdedit`, `powershell.exe`, `chkdsk`, `certutil`, `shutdown`, e — só no `prepare` — `C:\Windows\System32\curl.exe` e `C:\Windows\System32\tar.exe` (que é o `bsdtar`) |
| **Um disco externo** | **17,6 GB** no mínimo — 1,6 GB do `ARCABOOT` mais os 16 GB mínimos do `ARCAVAULT`, contados em base 1024. Na prática, grande o bastante para caber as imagens |
| Conexão de rede | Só no `arca prepare` sem `--iso`, para baixar 535,5 MB do Clonezilla |

> **Os caminhos do `curl` e do `tar` são fixos em `C:\Windows\System32\`, e não vêm do `PATH`.** O Git Bash traz homônimos — um `tar` que não entende zip falharia **depois** de o `prepare` já ter apagado o disco.

---

## 3. Compilação — passo a passo

### 3.1 — Instalar o Rust

Baixe e execute o `rustup-init.exe` de <https://rustup.rs>. Ele instala `rustc`, `cargo` e o toolchain padrão. No Windows, o padrão já é o `x86_64-pc-windows-msvc` — que é o que este projeto precisa.

Confira:

```powershell
rustc --version
cargo --version
```

Saída esperada (a sua pode ser mais nova):

```
rustc 1.98.0 (88d9e12ae 2026-08-18)
cargo 1.98.0 (797e8a9bc 2026-08-05)
```

Se o `rustup` reclamar de linker ausente, instale os **Build Tools for Visual Studio** com a carga de trabalho *"Desenvolvimento para desktop com C++"*. O toolchain MSVC não traz linker próprio: ele usa o `link.exe` da Microsoft.

### 3.2 — Clonar e compilar

```powershell
git clone https://github.com/carreirodev/ArcaBackup.git
cd ArcaBackup
cargo build --release
```

Resultado:

```
    Finished `release` profile [optimized] target(s) in <alguns minutos na primeira vez>
```

O binário fica em:

```
target\release\arca.exe        # 1.507.328 bytes neste build, com símbolos removidos (strip = true)
```

Para uma compilação de desenvolvimento (mais rápida de compilar, mais lenta de rodar, com símbolos):

```powershell
cargo build                    # → target\debug\arca.exe
```

### 3.3 — O que o `build.rs` faz, e por que você deve se importar

O `build.rs` faz **duas** coisas que nenhum outro lugar do projeto poderia fazer.

#### a) Embute o manifesto `requireAdministrator`

```
recursos/arca.manifest  →  /MANIFEST:EMBED  →  arca.exe
```

Com o manifesto dentro do executável, **é o próprio Windows quem eleva e quem repassa a linha de comando** — o caminho mais curto para o requisito C-7 (*"repassar os argumentos ao relançar com elevação"*), porque não passa por nenhuma serialização do ARCA. Dar duplo clique no `arca.exe` ou chamá-lo de um console não elevado dispara o UAC automaticamente.

Consequência prática: **o executável de teste herdaria o manifesto**, e o `cargo test` não conseguiria rodá-lo sem disparar UAC. Por isso o `Cargo.toml` traz:

```toml
[[bin]]
name = "arca"
path = "src/main.rs"
test = false
bench = false
```

Não há perda: `src/main.rs` é fino de propósito, e tudo que tem teste mora na `lib`.

#### b) Carimba o commit no `--version`

O `arca.exe` mora em **dois lugares** — o `target\release\` do `C:` e o `arca\arca.exe` dentro do `ARCABOOT` do dispositivo — e eles envelhecem em ritmos diferentes. Até 24/08/2026 os dois respondiam `arca 0.1.0`, e descobrir que o do dispositivo estava três consertos atrás exigiu procurar strings dentro do executável.

O carimbo resolve isso:

```
0.1.0 (aeef837 2026-08-24)                      ← árvore limpa
0.1.0 (aeef837 2026-08-24, arvore suja)         ← havia mudanças não commitadas
0.1.0 (aeef837 2026-08-24, arvore desconhecida) ← o `git status` não respondeu
0.1.0 (sem git)                                 ← sem git na máquina, ou fora de um clone
```

> **`arvore suja` conta arquivos não rastreados também.** Um fonte novo que ainda não entrou no git muda o que o binário faz tanto quanto um fonte editado. E o carimbo **nunca derruba um build**: um carimbo ausente é informação a menos, e parar a compilação por causa dele seria pior do que o problema que ele resolve.

O `build.rs` declara `rerun-if-changed` sobre `.git/HEAD` e `.git/index` justamente para que um commit que só toca documentação não deixe o carimbo apontando para o commit anterior.

### 3.4 — Rodar a suíte de testes

```powershell
cargo test
```

Saída resumida da execução real deste repositório:

```
     Running unittests src\lib.rs
running 731 tests
test result: ok. 731 passed; 0 failed; 0 ignored

     Running tests\b10_nada_e_apagado.rs ............ 2 passed
     Running tests\e10_preparar_o_dispositivo.rs .... 28 passed
     Running tests\e11_verificar_a_imagem.rs ........ 9 passed
     Running tests\e12_sondar_a_maquina.rs .......... 18 passed
     Running tests\e1_dispositivo_conectado.rs ...... 5 passed
     Running tests\e2_firmware_desta_maquina.rs ..... 6 passed
     Running tests\e4_desarmar_o_dispositivo.rs ..... 6 passed
     Running tests\e7_armar_o_dispositivo.rs ........ 9 passed
     Running tests\e8_colher_o_desfecho.rs .......... 6 passed
     Running tests\e9_restaurar_o_disco.rs .......... 9 passed
     Running tests\repasse_de_argumentos.rs ......... 5 passed
     Running tests\s1_nenhum_acesso_raw.rs .......... 2 passed
     Running tests\s6_o_tempo_nao_decide.rs ......... 4 passed
```

**840 testes, e nenhum deles pede UAC.**

Alguns testes de integração falam com o hardware desta mesa (o dispositivo conectado, o `bcdedit` desta máquina). Eles **se pulam sozinhos** quando o hardware não está lá, imprimindo o motivo:

```
pulado: nenhum dispositivo ARCA conectado
```

Isso é deliberado: o teste não mente dizendo que passou sobre algo que não olhou, e também não quebra o build de quem não tem o SSD na mesa.

Para rodar um arquivo só, ou um teste só:

```powershell
cargo test --test e12_sondar_a_maquina        # um arquivo de integração
cargo test montar_backup                      # todo teste cujo nome contenha isso
cargo test -- --nocapture                     # mostra o que os testes imprimem
```

### 3.5 — Compilar fora do Windows

Compila. **E não faz nada útil** — mas mantém honesto o que é portátil e o que não é.

```bash
cargo build            # em Linux/macOS: compila, com duplos no lugar dos adaptadores
```

Fora do Windows, `src/main.rs` monta o contexto com os **duplos** de `src/duplos.rs`: `PrivilegiosDeMentira` (sempre elevado), `SistemaDeMentira`, `ParticionadorDeMentira` (responde os três discos desta mesa e apenas registra o que lhe mandaram fazer) e `EntropiaDeMentira` (devolve zeros — e um selo de zeros nunca passa por selo de verdade, porque a saída o diz).

O `build.rs` detecta o alvo e **não** tenta embutir o manifesto fora do `windows-msvc`.

### 3.6 — Verificação de qualidade

```powershell
cargo fmt --check          # formatação
cargo clippy -- -D warnings
cargo doc --open           # a documentação interna, que é densa e vale a leitura
```

> Os comentários deste código não descrevem o que ele faz — descrevem **por que ele faz assim, e o que custou descobrir**. `cargo doc` é a melhor porta de entrada para o projeto depois deste README.

---

## 4. Instalação e primeira execução

Não há instalador. O `arca.exe` é autocontido — copie-o para onde quiser:

```powershell
copy target\release\arca.exe C:\Ferramentas\arca.exe
```

Para chamá-lo por nome de qualquer diretório, acrescente a pasta ao `PATH` do usuário:

```powershell
[Environment]::SetEnvironmentVariable(
    "Path",
    [Environment]::GetEnvironmentVariable("Path", "User") + ";C:\Ferramentas",
    "User")
```

Abra um **novo** console e confira:

```powershell
arca --version
```

```
arca 0.1.0 (aeef837 2026-08-24)
```

### A ajuda embutida

```powershell
arca --help                # a lista dos nove comandos e as flags
arca backup --help         # a ajuda de um comando específico
arca --version             # a versão com o commit de onde este binário veio
```

Os três são respondidos pelo próprio `clap`, saem com código `0`, e **não pedem elevação**.

> **`arca --version` é o jeito de saber qual dos dois binários você está olhando.** O `arca.exe` do `target\release\` e o `arca\arca.exe` do `ARCABOOT` envelhecem em ritmos diferentes — um dispositivo preparado hoje carrega o ARCA de hoje, e continua carregando depois de o ARCA mudar.


### O que acontece ao rodar qualquer comando

1. O ARCA **registra a invocação** em `%LOCALAPPDATA%\ARCA\arca.log` — antes de qualquer outra coisa, inclusive antes de o `clap` analisar a linha. `--version` e `--help` curto-circuitam dentro do `clap`, e sem essa anotação não haveria como provar, do lado de fora, que a linha de comando atravessou a elevação intacta.
2. **Analisa a linha de comando.** Uma linha errada recebe a mensagem do `clap` no console de onde foi digitada, **sem pedir UAC**.
3. **Garante a elevação.** Com o manifesto embutido, o Windows já elevou antes de o processo começar. Sem ele, o ARCA se relança elevado, repassando os argumentos **brutos** — não uma reconstrução a partir do que o `clap` entendeu. (Foi por reconstruí-los que um `--dry-run` virou execução real, uma vez.)
4. **Executa o comando.**
5. **Segura a janela** ao terminar, esperando Enter — porque a janela que o UAC abre não é a mesma de onde o comando foi digitado, e sem a pausa a saída piscaria e sumiria.

> Para chamar o ARCA de dentro de um script, use a flag oculta `--sem-pausa`. Ela é flag de linha de comando, e não variável de ambiente, porque **o processo que o UAC eleva não herda o ambiente de quem o chamou** — quem o cria é o serviço AppInfo. O que atravessa a elevação é a linha de comando.

---

## 5. Anatomia de um dispositivo ARCA

Um **dispositivo** é um disco externo com duas partições, rotuladas sempre com os mesmos nomes. É o rótulo — nunca a letra, nunca `sda`, nunca o número de série — que torna a receita reproduzível e os dispositivos intercambiáveis.

```
[dispositivo]  — um SSD externo, tabela MBR, duas partições
│
├── ARCABOOT  — FAT32, 1600 MiB, no fim do disco (MbrType 12)
│   ├── EFI/boot/bootx64.efi        ← para onde a entrada de firmware aponta
│   ├── live/                       ← kernel, initrd, filesystem.squashfs
│   ├── boot/grub/grub.cfg          ← A RECEITA, reescrita a cada operação
│   └── arca/
│       ├── arca.exe                ← o próprio ARCA (§4.1)
│       └── estado.json             ← o job: selo, operação, nome, disco, momento
│
└── ARCAVAULT — NTFS, todo o resto (MbrType 7)
    ├── ARCA-LOGS/
    │   ├── backup-2026-08-22_Apps/arca-fim.txt
    │   ├── restauracao-2026-08-22_Apps/
    │   │   ├── arca-fim.txt
    │   │   └── arca-restore.log
    │   ├── verificacao-2026-08-22_Apps/arca-fim.txt
    │   └── sondagem/
    │       ├── arca-fim.txt
    │       └── blkdev.list          ← o oráculo do §4.5
    ├── clonezilla-live-3.3.3-15-amd64.zip   ← cópia do pacote (PR-3)
    ├── 2026-08-21_WindowsCompleto/          ← uma imagem
    │   ├── MD5SUMS                          ← é ele que distingue imagem de resíduo
    │   ├── arca-check.log                   ← o veredito do ocs-chkimg
    │   ├── blkdev.list, disk, parts, ...
    │   └── nvme0n1p3.ntfs-ptcl-img.zst.aa, ...
    └── 2026-08-22_Apps/
```

### Por que o ARCA mora no dispositivo

A imagem captura o `nvme0n1` — o disco **interno**. O dispositivo é externo, logo **não entra na imagem**.

Consequência: uma restauração substitui o `C:` e devolve, junto, qualquer ARCA que estivesse lá dentro — inclusive versões antigas com defeitos já corrigidos. **O que julga uma restauração não pode morar no disco que a restauração substitui.** Morando no `ARCABOOT`, o `arca.exe` e o `estado.json` sobrevivem a qualquer restauração.

O corolário incômodo: o `%LOCALAPPDATA%\ARCA\arca.log` mora no `C:`, e **a restauração o destrói**. O `arca.log` que estiver lá depois veio de dentro da imagem, e as linhas dele são de outro tempo. O que sobrevive é o `estado.json`.

### Regra única de operação

**Um dispositivo ARCA conectado por vez.** Com dois, todo comando que se localiza por rótulo recusa — e a recusa nomeia as letras dos dois:

```
erro: ha 2 volumes com o rotulo ARCAVAULT conectados (D:, E:), e o ARCA opera
um dispositivo por vez: e pelo rotulo que a receita resolve o destino, e com ele
repetido nao ha o que escolher. Desconecte os demais e rode de novo. Se voce
acabou de preparar um dispositivo, sao os dois — o novo e o de antes
```

*(captura real de 23/08/2026, logo depois de um `arca prepare` bem-sucedido)*

### Os dois estados de um dispositivo

| Estado | O que é | Como se chega |
|---|---|---|
| **Inerte** | O `grub.cfg` sem nenhum `menuentry --id arca-backup` **e** com `set default="live-default"`; o `{fwbootmgr}` sem `bootsequence`. Bootar nele abre o menu do Clonezilla e espera alguém | `arca desarmar`, ou o primeiro passo de todo comando que arma |
| **Armado** | A receita está no `grub.cfg`, o `set default` aponta para ela, e o firmware tem a marca de boot único | Depois da confirmação digitada de `backup`, `restore`, `verify --completo` ou `sondar` |

> **O estado inerte não é uma cópia guardada** — é o que sai de aplicar a regra ao `grub.cfg` que está no dispositivo agora. E **não é o estado em que o Clonezilla entrega o pacote**: o zip vem com `set default="0"`, que aponta por **posição**, e a posição muda quando o bloco do ARCA entra antes do `live-default`. Um dispositivo assim está armado no instante em que alguém insere o bloco. Por isso o `arca prepare` desarma o que acabou de instalar.

---

## 6. Os nove comandos, um por um

Visão geral:

```
arca prepare --dispositivo <indice> [--iso <caminho>]
                          # particiona o disco, instala o Clonezilla e o ARCA,
                          #   cria a entrada de boot
arca sondar               # arma boot unico que so roda `lsblk` e desliga
arca backup <nome>        # monta a receita, arma o boot, reinicia
arca resultado            # le o desfecho do job pendente e desarma o dispositivo
arca list                 # imagens no dispositivo conectado
arca verify <nome> [--completo]
                          # confere os MD5SUMS sem reiniciar; --completo arma
                          #   um boot unico que so roda o ocs-chkimg
arca restore [<nome>]     # lista, confirma e reinicia para restaurar
arca status               # diagnostico: dispositivo, firmware, job pendente
arca desarmar             # devolve o dispositivo ao estado inerte
```

**Quatro deles armam.** O `prepare` não arma — ele *desarma* o que instala. E `list`, `status`, `resultado` e `verify` sem `--completo` só leem:

| Comando | Reinicia? | Destrói? | Nomeia um disco | Nomeia uma imagem |
|---|---|---|---|---|
| `prepare` | não | **sim, na hora** | sim (`--dispositivo`) | não |
| `sondar` | sim | não | não | não |
| `backup` | sim | não | sim (descoberto) | sim |
| `restore` | sim | **sim, no reinício** | sim (descoberto) | sim |
| `verify --completo` | sim | não | não | sim |
| `verify` | não | não | não | sim |
| `list` / `status` / `resultado` / `desarmar` | não | não | não | não |

**Todos exigem privilégio administrativo.**

---

### 6.1 `arca prepare`

Transforma um disco qualquer num dispositivo ARCA: apaga a tabela de partição, cria as duas partições, rotula, instala o Clonezilla, instala o ARCA e cria a entrada de boot no firmware.

> ⚠️ **É a única operação do ARCA que destrói dados sem reiniciar a máquina.** E é a única que roda antes de existirem os rótulos pelos quais todas as outras se localizam.

#### Sintaxe

```powershell
arca prepare --dispositivo <INDICE> [--iso <CAMINHO>] [--dry-run]
```

| Argumento | Obrigatório | O que é |
|---|---|---|
| `--dispositivo <INDICE>` | **sim** | O índice do disco **no Windows** — o número que o `Get-Disk` mostra. Não aceita letra, rótulo nem `sda` |
| `--iso <CAMINHO>` | não | Instala de um arquivo local em vez de baixar. Apesar do nome, o arquivo é o **zip** do Clonezilla |

#### Por que `--dispositivo` é obrigatório, mesmo havendo um candidato só

O princípio: *o ARCA destrói dados quando o usuário nomeou o alvo e confirmou por escrito, e **nunca por dedução**.* Deduzir o disco seria o ARCA escolhendo o que apagar — mesmo quando a escolha pareceria óbvia.

**E o índice não é identidade.** Medido em 23/08/2026: o dispositivo desta mesa era o disco 1 e virou o disco 2 quando um segundo SSD foi conectado. Por isso a confirmação final pede o **modelo**, que a tela acabou de imprimir — e não o número que se digitou aqui.

Para descobrir o índice:

```powershell
Get-Disk | Format-Table Number, FriendlyName, Size, BusType, IsSystem, IsBoot
```

#### As sete defesas, e nenhuma é opcional

| # | Defesa | Recusa quando |
|---|---|---|
| 1 | `MediaType` | O disco é **fixo** — ou de tipo que o Windows **não soube classificar**. "Não sei" recusa junto com "fixo": supor que o desconhecido é externo faria a defesa passar batido justamente onde ela mais importa |
| 2 | `IsSystem` / `IsBoot` | É o disco do sistema ou o disco de boot **deste** boot |
| 3 | `%SystemDrive%` | O disco carrega a letra onde o Windows que está rodando mora. É uma **segunda** pergunta, não a mesma da defesa 2: numa máquina com dois Windows as duas divergem |
| 4 | Somente-leitura | Não há o que particionar |
| 5 | Tamanho | Menor que 17,6 GB — os 1600 MiB do `ARCABOOT` mais os 16 GiB mínimos do `ARCAVAULT` |
| 6 | `--dry-run` de primeira classe | É a **única forma de ver o plano de partições sem executá-lo** |
| 7 | Releitura | O disco é relido **antes** de escrever — é o mesmo? — e **depois** de escrever — saiu o que se pediu? |

**Disco fixo é recusa dura, sem opção de forçar.** O modo de falha apaga o Windows de alguém, e nenhuma confirmação digitada compra isso.

#### Os onze passos, e o que fica se você parar em cada um

| # | Passo | Parando aqui, o que fica |
|---|---|---|
| 1 | Descrever o disco e julgar as sete defesas | nada tocado |
| 2 | Imprimir o plano inteiro | nada tocado |
| 3 | Perguntar `(s/N)` e **reler o disco** | nada tocado |
| 4 | Confirmação digitada: o **modelo** do disco | nada tocado |
| 5 | **Particionar e formatar** ← ponto sem volta | disco apagado, duas partições vazias |
| 6 | Baixar o pacote, ou usar o `--iso` | dispositivo vazio, sem Clonezilla |
| 7 | Conferir o SHA256 | idem — e **nada foi extraído** |
| 8 | Extrair | `ARCABOOT` com o Clonezilla e `set default="0"` |
| 9 | Devolver o `grub.cfg` ao estado inerte | dispositivo bootável e inerte |
| 10 | Instalar o `arca.exe` e a cópia do pacote | dispositivo completo, sem entrada de firmware |
| 11 | Criar a entrada, apontá-la e **tirá-la da ordem permanente** | pronto |

Nenhum desses estados é pior do que o anterior, e todos são reversíveis rodando o comando de novo — ele começa apagando. Do passo 8 em diante o dispositivo **já boota**: um `prepare` interrompido ali deixa um Clonezilla utilizável pelo menu.

#### O pacote do Clonezilla

| | |
|---|---|
| Versão | **3.3.3-15** — a que está bootando neste projeto, lida do `hostname=cl-3.3.3-15` do `grub.cfg` |
| Arquivo | `clonezilla-live-3.3.3-15-amd64.zip`, 561.478.648 bytes |
| URL | `https://downloads.sourceforge.net/project/clonezilla/clonezilla_live_stable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip` |
| SHA256 | `00cee7700433e63017e2ea9eb40519108829710132364a8028a6c039a6046304` |

O SHA256 é **constante de código, compilada no binário** — nunca baixada junto do arquivo, o que não verificaria nada: quem pudesse trocar um trocaria o outro. Ele tem duas fontes independentes — o `CHECKSUMS.TXT` do mirror do projeto e o `certutil` sobre o arquivo baixado do SourceForge, servidores diferentes e o mesmo número — e a conferência acontece **antes de extrair**.

#### Instalar sem rede

```powershell
arca prepare --dispositivo 1 --iso D:\clonezilla-live-3.3.3-15-amd64.zip
```

O SHA256 é conferido do mesmo jeito; só o `curl` é pulado. É o que salva quando a máquina que precisa preparar o dispositivo é justamente a que está sem Windows e sem rede.

#### Ver o plano sem executá-lo

```powershell
arca prepare --dispositivo 1 --dry-run
```

Para **antes** da pergunta, não escreve nada, e não diz que escreveu.

#### Exemplo real — a execução de 23/08/2026

*Captura preservada em `recursos/capturas/arca-prepare-2026-08-23-marco.txt`, abreviada nos comentários mais longos.*

```
> arca prepare --dispositivo 1

  Disco 1 ......................... JMicron Generic · USB · 447,1 GB
  Tipo de midia ................... External hard disk media · nao e disco fixo (PR-5)
  Sistema ......................... IsSystem false · IsBoot false · nao carrega o C:
  Tabela de particao hoje ......... MBR · vai ser reescrita como MBR

O QUE EXISTE NESTE DISCO HOJE, e vai ser APAGADO:
  1  NTFS    445,6 GB  "ARCAVAULT"                E:
  2  FAT32     1,6 GB  "ARCABOOT"                 F:

  ESTE DISCO JA E UM DISPOSITIVO ARCA. Os rotulos acima sao os dele, e o
  que esta no ARCAVAULT sao AS IMAGENS — todas. Preparar por cima apaga
  cada uma, e o ARCA nunca apaga imagem em nenhum outro caminho (B-10).

O QUE VAI FICAR NO LUGAR:
  MBR  1  NTFS   445,6 GB  ARCAVAULT   as imagens moram aqui
       2  FAT32    1,6 GB  ARCABOOT    o Clonezilla e o ARCA moram aqui

E O QUE MAIS VAI ACONTECER:
  Clonezilla 3.3.3-15 · baixado (535,5 MB), com o SHA256 conferido contra
     o valor compilado neste ARCA — e nao contra um baixado junto (PR-1)
  Uma copia do pacote fica no ARCAVAULT, para o dispositivo se reconstruir
     sozinho (PR-3)
  Uma entrada de boot chamada `ARCA` e criada no firmware, apontando para o
     ARCABOOT — e tirada da ordem permanente logo em seguida, para que
     ligar a maquina continue subindo o Windows (C-5)
  O proprio `arca.exe` e instalado no ARCABOOT, porque o que julga uma
     restauracao nao pode morar no disco que ela substitui (§4.1)

  O `grub.cfg` fica INERTE: nada roda sozinho ate um `arca backup` (§4.4)

Podemos continuar? (s/N): s
  Conferido antes de escrever ..... ok · o disco 1 continua sendo `JMicron Generic` de 447,1 GB

Digite o modelo do disco para confirmar: JMicron Generic
  Particionando ................... ok · MBR, 2 particoes · MbrType 7 e 12
  Formatando e rotulando .......... ok · ARCAVAULT (NTFS) em E: · ARCABOOT (FAT32) em F:
  Conferido apos escrever ......... ok · relido do disco · nenhuma particao ativa, unidade 4096 (C-3)
  Baixando Clonezilla ............. 3.3.3-15 · 535,5 MB · pode levar minutos
  SHA256 conferido ................ ok · 00cee7700433 · de https://downloads.sourceforge.net/...
  Copia do pacote em ARCAVAULT .... ok · E:\clonezilla-live-3.3.3-15-amd64.zip (PR-3)
  Extraindo ....................... ok · F:\
  Estado inerte ................... ok · o `set default` do pacote era "0", e voltou para `live-default`
  Instalando o ARCA em ARCABOOT ... ok · F:\arca\arca.exe (§4.1)
  Entrada de firmware ............. reusada e reapontada · ARCA · {f4057bd0-…} · partition=F:
  Ordem de boot ................... ok · a entrada ja estava fora da ordem permanente

Dispositivo pronto.

  O `grub.cfg` esta INERTE: bootar neste dispositivo abre o menu do
  Clonezilla e espera alguem (§4.4). Nada roda sozinho ate um `arca backup`.
  A entrada de firmware existe e esta FORA da ordem permanente — ligar a
  maquina continua subindo o Windows, com ou sem este dispositivo conectado.

  O ARCAVAULT esta em E: e o ARCABOOT em F:. As letras mudam de uma
  conexao para outra; os rotulos, nao — e e por rotulo que o ARCA acha o
  dispositivo (B-1, S-3).

  Primeiro backup:  arca backup <nome>
```

> **A entrada de firmware é única.** O ARCA mantém **uma** entrada chamada `ARCA`, e não uma por dispositivo: duas seriam duas formas de bootar no Clonezilla, uma delas sem ninguém olhando. Se você voltar a usar outro dispositivo ARCA, o `arca backup` reaponta a entrada para ele ao armar, e **confere** que reapontou.

---

### 6.2 `arca sondar`

Descobre **que discos há nesta máquina** pelos olhos do Linux: um boot único que roda `lsblk`, grava a saída no `ARCAVAULT` e desliga.

#### Sintaxe

```powershell
arca sondar [--dry-run]
```

**Não aceita argumento nenhum**, e a ausência é decisão. Os outros três comandos que armam nomeiam uma imagem; a sondagem pergunta *"que discos há nesta máquina?"* — uma pergunta sem sujeito a escolher, feita justamente no dispositivo que ainda não tem imagem. Um argumento aqui seria um valor que receita nenhuma usa, e a linha de comando o recusa em vez de ignorar em silêncio.

#### Por que ele existe

A receita de backup precisa nomear o disco pelo nome que o **Linux** lhe dá — `nvme0n1`. O Windows não conhece esse nome, e o ARCA **não o inventa**: ele o lê de um `blkdev.list`, o **oráculo**.

Há duas fontes para esse arquivo:

| Fonte | Descreve | Existe quando |
|---|---|---|
| O `blkdev.list` de dentro de cada imagem | a máquina de **quando o backup foi feito** | há pelo menos uma imagem no dispositivo |
| O `blkdev.list` da sondagem | a máquina de **agora** | depois de um `arca sondar` |

Havendo as duas, **a sondagem ganha**, e a divergência é dita na tela — nunca resolvida em silêncio.

Sem nenhuma das duas — que é exatamente o caso de um dispositivo recém-preparado — `arca backup` **recusa**:

```
  Disco de origem ................. POR DETERMINAR

  O NOME DO DISCO DE ORIGEM NAO FOI DETERMINADO.
  nenhuma imagem do dispositivo traz um `blkdev.list` legivel, e e dele que
  sai o nome que o Linux da ao disco. O Windows nao conhece esse nome, e o
  ARCA nao o inventa
```

Até a etapa E11, sair desse buraco custava fazer o primeiro backup pelo menu do Clonezilla — dois reinícios e ~40 minutos, exatamente aquilo que este app existe para não precisar. Hoje custa um `arca sondar`: **1 min 40 s**, sem nenhuma tela.

#### É a mais barata das quatro operações

A receita da sondagem **não chama programa nenhum do Clonezilla** — nem `ocs-sr`, nem `ocs-chkimg` — e nada é escrito fora do `ARCAVAULT`. O pior caso é a máquina parar num menu, que é chato e não destrói nada.

#### A confirmação é uma tecla

Ao contrário dos outros comandos que armam, a sondagem não pede um nome digitado por extenso: ela pergunta `Reiniciar agora e sondar? (s/N)`, com o padrão no **não**. A razão sobrevive à pergunta *"o que essa confirmação impede?"* — ela impede o **reinício** de quem digitou o comando sem saber que ele reinicia. Não há alvo destrutivo a confirmar.

#### O que aparece na tela

```
Dispositivo ARCA: ARCAVAULT (E:) · 445 GB livres

  Desarmando receita anterior ..... ok · ja estava inerte · F:\boot\grub\grub.cfg
  Sondagem de hoje ................ nenhuma · esta sera a primeira

A SONDAGEM NAO FAZ BACKUP NEM RESTAURACAO. Ela reinicia a maquina, roda o
`lsblk` no Linux do Clonezilla, grava a saida no ARCAVAULT e desliga.
Nenhum programa do Clonezilla e chamado, e nada e escrito fora do ARCAVAULT.

O QUE VOCE GANHA: o nome que o LINUX da ao disco desta maquina (`nvme0n1`), que
e o que a receita de backup e a de restauracao precisam nomear e que o Windows
nao conhece (§4.5). Sem ele, `arca backup` recusa.

O QUE ISSO CUSTA: um reinicio, e o que estiver aberto se perde. A maquina
desliga sozinha ao terminar.

Reiniciar agora e sondar? (s/N):
```

#### O que ela deixa no dispositivo

```
E:\ARCA-LOGS\sondagem\arca-fim.txt ... 50 bytes  · ARCA_SELO=… · ARCA_PROBE=OK · ARCA_FIM
E:\ARCA-LOGS\sondagem\blkdev.list .... 852 bytes · 2 discos, 7 particoes, 1 loop
```

**A sondagem anterior é substituída** — a pasta é fixa, e a tela avisa disso antes.

#### Na volta

```
> arca resultado

Sondagem
  nao opera sobre imagem nenhuma
  Desfecho: concluida — o selo bate e a receita chegou ao fim
  Discos vistos: sda (Maxtor Z1 SSD 480GB), nvme0n1 (KINGSTON SNV3S500G)
  Selo: 354da624e7fa0d21

  Desarmando SSD .................. ok · F:\boot\grub\grub.cfg
  Job ............................. encerrado · o desfecho foi lido e dito
  Ordem de boot ................... ok · o Windows ja era o primeiro

  O ORACULO DO §4.5 EXISTE AGORA. O `blkdev.list` da sondagem esta em
  ARCA-LOGS\sondagem\, e e dele que sai o nome que o LINUX da ao disco.

  Para conferir o que o `arca backup` vai nomear, sem armar nada:
    arca backup <nome> --dry-run
```

*(captura real de 24/08/2026 — `recursos/capturas/arca-sondar-marco-2026-08-24.txt`)*

> **O carimbo de tempo do `blkdev.list` é do Clonezilla, e ele sai três horas atrás do relógio do Windows.** A sondagem foi armada às 14:56:55 e o arquivo está carimbado 11:58. Nada é corrigido — somar três horas fabricaria um instante que ninguém mediu —, e a tela diz de quem é o carimbo. É a mesma razão de o selo existir: **não há relógio comum entre os dois lados do reinício.**

---

### 6.3 `arca backup`

Monta a receita de backup, arma o boot único e reinicia a máquina. Do outro lado, o Clonezilla faz um `savedisk` do disco de sistema, verifica a imagem que acabou de criar e desliga.

#### Sintaxe

```powershell
arca backup <NOME> [--dry-run]
```

#### As regras do nome (B-2)

O nome vira uma pasta no `ARCAVAULT` **e** uma palavra dentro de uma string de shell do outro lado do reinício. Ele é julgado por **lista de permissão**, e não de recusa — uma lista de recusa só está certa enquanto ninguém esquecer um caractere.

| Regra | Recusa | Por quê |
|---|---|---|
| Caracteres | Só `A-Z`, `a-z`, `0-9`, `.`, `_`, `-` | `arca backup "meu backup"` → o espaço reparte a palavra em duas dentro da receita |
| Acento | `backup_do_Antônio` | O que atravessa o grub e o live system é ASCII; um acento chega do outro lado como outra coisa |
| Começar com `-` | `-scs` | O `ocs-sr` o leria como opção, e não como nome de imagem |
| Começar com `.` | `.oculto` | No Linux é pasta oculta, e `.`/`..` são o diretório corrente e o pai |
| Terminar com `.` | `backup.` | O Windows corta o ponto final em silêncio, e a pasta criada não teria o nome pedido |
| Nome reservado do Windows | `CON`, `COM0`–`COM9`, `LPT0`–`LPT9`, `NUL`, `PRN`, `AUX` | O Clonezilla criaria a pasta do lado Linux, e do lado Windows nada dentro dela abriria |
| Pasta de serviço | `ARCA-LOGS`, `ARCA-DOCS` | A imagem seria gravada dentro dela e sumiria da listagem |
| Tamanho | mais de **48** caracteres | — |

O nome é julgado **antes de tocar no dispositivo**: um nome recusado não precisa de SSD conectado para ser recusado.

#### O pré-voo, na ordem em que acontece

1. **Julga o nome** (B-2).
2. **Acha o dispositivo** pelo rótulo `ARCAVAULT`.
3. **Desarma incondicionalmente** — sem consultar estado nenhum. Num dispositivo já inerte isso não escreve nada.
4. **Imprime o cabeçalho** — inclusive a linha do desarmar, **antes** de qualquer recusa poder cortar a saída. (Uma versão antiga recusava antes de imprimir, e quem rodasse `arca backup <nome-que-já-existe>` num dispositivo armado via só o erro: a ação acontecia e a saída não contava.)
5. **Julga o pré-voo**:
   - **B-3** — o nome já existe? Recusa mesmo se a pasta for resíduo.
   - **B-4** — cabe? O mínimo é o maior entre *maior imagem × 1,3* e *em uso × 0,45*. Entre 1× e 1,5× disso: avisa e pede confirmação.
   - **C-6** — o dispositivo é mídia removível que o `bcdedit` recusaria como alvo?
   - **C-10** — há rótulo repetido na mesa?
6. **Lê a Inicialização Rápida** (B-5) — direto do registro, em `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power` → `HiberbootEnabled`. Nunca do `powercfg /a`, que responde traduzido. Valor ausente é **"não se sabe"**, nunca "desativada".
7. **Roda `chkdsk /scan`** no volume do **sistema** (B-6) — julgado pelo **código de saída**, nunca pelo texto, que vem traduzido. Leva ~16 s nesta máquina.
8. **Descobre o disco de origem** no oráculo (§4.5).
9. **Pede a confirmação por extenso** — o nome da imagem, comparação exata, uma tentativa só.
10. **Arma**: grava a receita, reaponta a entrada de firmware, marca o boot único — e **relê cada uma** para provar que aconteceu.
11. **Avisa para remover o SSD** e reinicia.

#### Não há "digite o nome do disco"

`nvme0n1` é um nome do **Linux**, e quem o digitaria está no Windows, onde não há nada contra o que conferi-lo: um `nvme1n1` digitado por engano passaria por bom, iria para a receita, e nomearia o disco errado. Não achando o nome no oráculo, o comando **para** — e a saída é `arca sondar`.

#### Exemplo real — o backup de 22/08/2026, às 20:53:48

*Foi este comando que disparou o primeiro marco em hardware do projeto.*

```
> arca backup 2026-08-22_Apps

Dispositivo ARCA: ARCAVAULT (E:) · 164 GB livres
Origem: KINGSTON SNV3S500G · 465,8 GB · 105,9 GB em uso
Imagem estimada: ~47,7 GB · espaco suficiente
Imagem: 2026-08-22_Apps

  Desarmando receita anterior ..... ok · ja estava inerte · R:\boot\grub\grub.cfg
  Inicializacao rapida ............ desativada   ok
  chkdsk /scan .................... limpo        ok
  Nome disponivel ................. ok
  Disco de origem ................. nvme0n1 · lido de 2026-08-21_WindowsCompleto/blkdev.list, casando o modelo `KINGSTON SNV3S500G`

Pre-voo concluido, e o dispositivo esta inerte. Nada foi armado ainda —
o ponto sem volta e a confirmacao abaixo.

Digite o nome do backup para confirmar: 2026-08-22_Apps

  Entrada de firmware ............. ARCA · {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34} · partition=R:
  Receita armada .................. ok · R:\boot\grub\grub.cfg
  Boot unico ...................... ok · relido no bcdedit · {f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}
  Selo do job ..................... 7d2d2f5153625b38
  Desfecho esperado em ............ E:\ARCA-LOGS\backup-2026-08-22_Apps\arca-fim.txt

A maquina vai reiniciar agora e desligar sozinha ao terminar.
AO TERMINAR: remova o SSD antes de religar.

Reiniciando...
```

**As cinco linhas do meio são a releitura**: cada uma é algo que o ARCA mandou fazer e **conferiu perguntando de novo**, porque o sucesso do `bcdedit` nunca é prova — ele responde "êxito" e mantém o valor antigo em alguns casos.

**O `Selo do job` aparece na tela** porque é ele que a colheita vai cobrar do `arca-fim.txt`. Um selo que só existisse dentro do `estado.json` não daria a ninguém como conferir, à mão, se o desfecho que voltou é deste job. Neste marco isso foi feito: o `7d2d2f5153625b38` da tela e o da primeira linha do `arca-fim.txt` que voltou são a mesma cadeia, lida a olho antes de qualquer conclusão.

#### Ensaio: ver a receita sem armar nada

```powershell
arca backup 2026-08-24_Apps --dry-run
```

O ensaio imprime o pré-voo inteiro, a receita como o Clonezilla a executa, e a linha exata como ela entra no `grub.cfg`. **No ensaio o dispositivo não é nem desarmado** — a tela diz isso, e o selo impresso é de ensaio (dezesseis zeros), o que torna a receita inservível de propósito:

```
O selo acima e de ensaio (so zeros), e por isso esta receita nao serviria: o
de verdade nasce **ao armar**, de uma fonte de entropia do sistema. E ele que
liga o job ao desfecho que voltar.
```

#### Se o reinício falhar

O dispositivo fica **armado**, e a mensagem diz isso:

```
O dispositivo FICOU ARMADO e a maquina nao reiniciou. O proximo reinicio,
seja qual for a causa, vai bootar no dispositivo e rodar a receita.
Para desfazer:  arca desarmar
```

---

### 6.4 `arca resultado`

Lê o desfecho do job pendente, julga-o pelo **selo**, desarma o dispositivo, devolve o Windows ao topo da ordem de boot e encerra o job.

#### Sintaxe

```powershell
arca resultado
```

Sem argumentos. É o comando que se roda **depois de religar a máquina**, com o dispositivo reconectado.

#### O que ele faz, na ordem

1. **Lê o `estado.json`** — antes de desarmar. Sem job não há o que colher, e desarmar um dispositivo inerte para depois dizer "nada a colher" seria agir antes de ter o que dizer.
2. **Lê o `arca-fim.txt`** no lugar que o `estado.json` aponta.
3. **Confere o selo.** Só é aceito o desfecho cujo selo case com o do job pendente.
4. **Desarma** o dispositivo.
5. **Devolve o `{bootmgr}` ao topo** da ordem permanente — uma escrita só, incondicional, conferida por releitura. **Nada é removido**: as entradas do dispositivo continuam na ordem, atrás do Windows.
6. **Lista as imagens** — antes de encerrar o job, para que uma falha de leitura não encerre um job cujo desfecho ninguém viu.
7. **Marca o job como colhido** no `estado.json` — e nunca o apaga.
8. **Sai com código diferente de zero** se o conjunto não estiver bom.

> **Sem job nenhum a colher, ele não desarma** — e a diferença importa: misturar *"colhi"* com *"arrumei"* tiraria de quem lê a saída a informação de qual das duas aconteceu. Para desarmar sem colher existe o `arca desarmar`.

#### Exemplo real — a colheita de 22/08/2026, às 21:14:49

```
> arca resultado

Backup 2026-08-22_Apps
  22/08 · 39,7 GB
  Desfecho: concluida — o selo bate e a receita chegou ao fim
  Verificacao: APROVADA
  Selo: 7d2d2f5153625b38

  Desarmando SSD .................. ok · R:\boot\grub\grub.cfg
  Job ............................. encerrado · o desfecho foi lido e dito

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

125 GB livres
```

Hoje o comando imprime também a linha da ordem de boot, logo abaixo de `Job`:

```
  Ordem de boot ................... devolvida · o Windows voltou ao topo, na frente de ARCA · {f4057bd0-…}
```

ou, quando ele já estava lá:

```
  Ordem de boot ................... ok · o Windows ja era o primeiro
```

#### Desfecho e veredito são duas linhas, e nenhuma esconde a outra

| Linha | Responde | Quem escreve |
|---|---|---|
| **Desfecho** | *A operação terminou?* — `ARCA_BACKUP=OK`, `ARCA_RESTORE=FALHOU`, `ARCA_VERIFY=OK`, `ARCA_PROBE=OK` | a receita, no `arca-fim.txt` |
| **Veredito** | *A imagem é restaurável?* — `APROVADA` / `REPROVADA` | o `ocs-chkimg`, no `arca-check.log` |

São independentes: **um backup pode terminar e a imagem ser reprovada.** Mostrar só a verificação faria isso parecer um problema de verificação, quando é falha da operação inteira. Quando os dois não estão bons, o comando imprime a tela inteira **e sai com código diferente de zero** — falha parcial é falha total.

#### Todos os desfechos possíveis

| O que se encontra | Significado | O que o ARCA faz |
|---|---|---|
| Selo bate, `ARCA_FIM` presente, desfecho `OK` | Operação concluída | Mostra o veredito da imagem |
| Selo bate, desfecho `FALHOU` | O Clonezilla falhou e disse | Reporta falha e aponta o log |
| Selo bate, sem `ARCA_FIM` | Truncado — desligamento no meio | Falha; a pasta é resíduo |
| Selo não bate | **Job fantasma** | Ignora o arquivo e avisa |
| Sem linha de selo, selo repetido, ou sem marcador de desfecho | Não é desfecho de job nenhum do ARCA | Recusa nomeando qual dos três. **Nunca diz "o selo não bate": não há selo a bater** |
| Sem `arca-fim.txt`, com job pendente | O boot não aconteceu, **ou** o Clonezilla abriu menu | Falha, **nomeando as duas causas** |
| `arca-fim.txt` presente e ilegível | "Não consegui olhar" | **Não encerra o job** — a linha diz `CONTINUA PENDENTE` |

**Nenhuma linha desta tabela é silêncio.** Ausência de desfecho é falha, nunca "deu certo".

> **Colher marca o `estado.json` como colhido, em vez de apagá-lo.** O ARCA não apaga nada, e o arquivo é o único registro que liga aquele selo àquele nome. Depois da colheita, `arca status` diz *"já colhido, nada esperando"*.

---

### 6.5 `arca list`

Lista as imagens do dispositivo conectado. **Lê e nada mais**: não escreve no dispositivo.

#### Sintaxe

```powershell
arca list
```

#### Exemplo real

```
> arca list

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

125 GB livres
```

Com o dispositivo vazio — o caso de um recém-preparado:

```
Nenhuma imagem em ARCAVAULT.

445 GB livres
```

#### A última coluna

| Valor | Significa |
|---|---|
| `aprovada` | O `arca-check.log` da imagem termina em `ARCA_VEREDITO=APROVADA` |
| `reprovada` | Idem, com `REPROVADA`. **Uma imagem que já reprovou uma vez continua reprovada** |
| `sem veredito` | A pasta é imagem, mas não há `arca-check.log` legível. Dizer isso é melhor do que deixar a coluna vazia |
| `residuo` | **A pasta não tem `MD5SUMS`** — é rastro de um backup interrompido |

#### Resíduo não é imagem

Uma pasta sem `MD5SUMS` aparece marcada como resíduo, **nunca é oferecida para restaurar**, e nunca é sobrescrita: `arca backup` recusa um nome cuja pasta já exista, mesmo sendo resíduo. **Quem apaga resíduo é você, à mão** — o ARCA nunca apaga nada.

```
  2026-08-22_Interrompido      22/08 · 512 B · residuo
```

#### O que a listagem não é

Não há catálogo, não há banco, não há índice. `arca list` lê o dispositivo: se a informação já está na listagem de diretórios, não há o que armazenar. A pasta `ARCA-LOGS` fica de fora — ela não é imagem nem resíduo.

---

### 6.6 `arca verify`

Duas verificações que respondem **perguntas diferentes**. É isso que faz as duas existirem.

#### Sintaxe

```powershell
arca verify <NOME> [--completo] [--dry-run]
```

| | `arca verify <nome>` | `arca verify <nome> --completo` |
|---|---|---|
| Pergunta | *Os bytes que estão aqui são os que o Clonezilla gravou?* | *Esta imagem é restaurável?* |
| Como | Soma cada arquivo listado no `MD5SUMS` e compara, **no Windows** | Arma um boot único que só roda o `ocs-chkimg` |
| Reinicia | **não** | **sim** — e a máquina desliga sozinha |
| Escreve | nada | o `arca-check.log` da imagem |
| Muda a coluna do `arca list` | **não** | **sim** |
| Custo medido | 202,8 s em 39,7 GB (≈ 200 MB/s) | 5 min 12 s na mesma imagem, mais o boot |
| Pega | corrupção de mídia, cópia truncada | inconsistência dentro da imagem |

> **Um `.zst` intacto byte a byte que carregue dentro de si um NTFS inconsistente passa na conferência e reprova no veredito.** As duas palavras nomeiam julgamentos sobre perguntas diferentes, e misturá-las faria a listagem afirmar algo que o `ocs-chkimg` não disse. Por isso a conferência de `verify` sem `--completo` **não** escreve no `arca-check.log`.

E nenhuma das duas substitui a verificação automática de todo backup: a receita de backup já chama o `ocs-chkimg` dentro do ramo de êxito do `savedisk`.

#### Exemplo real — `arca verify` sem reiniciar

*Rodado em 23/08/2026 sobre a `2026-08-22_Apps` — 202,8 s, e a estimativa da terceira linha acertou o segundo. Abreviado nas trinta e sete linhas do meio.*

```
> arca verify 2026-08-22_Apps

Dispositivo ARCA: ARCAVAULT (D:) · 125 GB livres
Imagem: 2026-08-22_Apps · 22/08 · 39,7 GB

  MD5SUMS lido .................... 39 arquivos · D:\2026-08-22_Apps\MD5SUMS
  A conferir ...................... 39,7 GB

Conferindo 39 arquivos · 39,7 GB. Estimativa: 3 min 23 s.
A tela vai andando um arquivo por vez — parada nao e travamento.

  [ 1/39] blkdev.json ..................... ok
  [ 2/39] blkdev.list ..................... ok
  ...
  [24/39] nvme0n1p3.ntfs-ptcl-img.zst.aa .. ok
  ...
  [39/39] parts ........................... ok

  Conferidos ...................... 39 de 39 · 39,7 GB lidos
  Fora do MD5SUMS ................. 4 arquivos · normal — o proprio MD5SUMS e o que nasce depois dele
  Veredito ........................ APROVADA — os bytes sao os que o Clonezilla gravou

  Isto conferiu que os bytes nao mudaram desde o backup. NAO conferiu que a
  imagem e restauravel — quem responde isso e o `ocs-chkimg`, e para isso ha
  `arca verify <nome> --completo`, que custa um reinicio.
```

> **`D:` e não `E:` não é erro de transcrição.** As outras telas deste documento mostram o `ARCAVAULT` em `E:`; nesta sessão ele veio em `D:`. **A letra muda de uma conexão para outra e o rótulo não** — que é exatamente o motivo de o ARCA se localizar por rótulo.

> **A linha `Fora do MD5SUMS` nunca é problema.** A pasta tem 43 arquivos e o `MD5SUMS` lista 39. Os quatro que sobram têm hora: o próprio `MD5SUMS`, o `clonezilla-img` e o `Info-img-id.txt` nascem no fim do `savedisk`, e o `arca-check.log` cinco minutos depois. Chamar isso de falta reprovaria toda imagem que o Clonezilla já fez.

Quando alguma coisa não bate, a tela sai inteira antes do erro, com cada falha nomeada:

```
  [1/3] disk .... ok
  [2/3] parts ... NAO BATE · o MD5SUMS diz 000000000000 e o arquivo soma b9c383232530
  [3/3] sumido .. AUSENTE · o MD5SUMS o lista e ele nao esta na pasta da imagem
```

**`AUSENTE` e `NAO DEU PARA LER` são linhas diferentes**, e a distinção é a regra da casa: *"não consegui olhar" nunca vira "não há nada lá"*. O `certutil` responde `0x80070002` para arquivo ausente, e cair nesse ramo faria as duas chegarem iguais — por isso quem responde sobre existência é o sistema de arquivos, antes de o `certutil` ser chamado.

#### `--completo`: a verificação armada

```powershell
arca verify 2026-08-22_Apps --completo
```

Desarma primeiro, pede a confirmação digitada e reinicia — como os outros comandos que armam. E **pede a confirmação mesmo não destruindo nada**, pelo mesmo motivo que o `arca backup` a pede: a máquina vai reiniciar e desligar sozinha, e quem digitou `--completo` sem ler está a um Enter de perder o que estiver aberto.

```
A verificacao completa reinicia a maquina, roda o `ocs-chkimg` e desliga.
Ela NAO substitui a verificacao de todo backup (B-9), e nao destroi nada —
mas a maquina desliga, e o que estiver aberto se perde.
Na imagem de 39,7 GB desta mesa o `ocs-chkimg` levou 5 min 12 s.

Sem reiniciar, `arca verify 2026-08-22_Apps` confere os MD5SUMS em 3 min 23 s.

Digite o nome da imagem para confirmar:
```

Sem `--completo`, o comando **não desarma** — desarmar é obrigação dos comandos que armam, e este não arma.

#### Recusas

Os dois caminhos recusam **antes** de conferir ou armar qualquer coisa: imagem inexistente, pasta que é resíduo, ou `MD5SUMS` que não serve.

---

### 6.7 `arca restore`

Lista as imagens, confirma e reinicia para restaurar. Do outro lado, o Clonezilla faz um `restoredisk` **e apaga o disco de destino**.

> ⚠️ **É a operação destrutiva do ARCA.** Ela não destrói na hora: destrói no reinício.

#### Sintaxe

```powershell
arca restore                      # lista e pergunta o número
arca restore 2026-08-22_Apps      # pula a lista
arca restore 2026-08-22_Apps --dry-run
```

#### Não há flag de destino, e a ausência é decisão

O `--destino <indice>` existiu até 23/08/2026 e saiu: **o único destino válido é o disco de que a imagem veio**. Sem destino divergente, a flag passaria a ser um jeito de apontar um disco para apagar — que é exatamente o que o princípio proíbe. O ARCA acha o disco de origem pelo modelo e **prova que é ele pelos setores**; não achando, ou achando dois, ele **para**.

Um script antigo que passe a flag recebe erro de uso, e não um argumento ignorado em silêncio.

#### As defesas, na ordem

| # | Defesa | O que impede |
|---|---|---|
| 1 | Desarma incondicionalmente e **imprime que desarmou** antes de qualquer recusa | Um job sumir em silêncio |
| 2 | Recusa mídia removível como alvo de entrada de boot, e rótulo repetido | Armar num dispositivo errado, ou num partido entre dois |
| 3 | Resíduo nunca ganha número na lista | Digitar um número que não se pode restaurar |
| 4 | Confere a imagem contra o que há dentro dela — `disk`, `<disco>-gpt.sgdisk`, `blkdev.list` | Restaurar uma imagem incompleta |
| 5 | **R-7: os setores têm de bater exatamente** com o que o `MSFT_Disk` responde para o disco na mesa | Restaurar no disco errado. A medição prova **identidade**, não capacidade: para mais **ou** para menos, recusa |
| 6 | **R-8: recusa o próprio dispositivo ARCA como destino, sempre** | Apagar o Clonezilla que está executando a receita, e as imagens que ela lê — inclusive a que está sendo restaurada |
| 7 | Confirmação: **o nome da imagem por extenso** | Um `2` apagar um disco |

**São duas leituras do usuário, e elas não são redundantes.** O índice **escolhe** — apontar numa lista, e um número é a forma mais curta de apontar. O nome por extenso **confirma**, e existe justamente para custar o trabalho de ler e digitar.

#### Exemplo real — 23/08/2026

*Tudo até a confirmação é execução real. As linhas depois dela são reprodução determinística do que o código monta: a sessão que as imprimiu morreu no reinício que ela mesma disparou, e o `arca.log` que teria o registro mora no `C:` — que a restauração substituiu.*

```
> arca restore

Dispositivo ARCA: ARCAVAULT (E:) · 125 GB livres

  Desarmando receita anterior ..... ok · ja estava inerte · R:\boot\grub\grub.cfg

Imagens em ARCAVAULT:
  [1] 2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  [2] 2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  [3] ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

Qual restaurar? 2

  Imagem escolhida ................ 2026-08-22_Apps
  Origem da imagem ................ KINGSTON SNV3S500G · nvme0n1 · 976773168 setores de 512 B · 465,8 GB
  Destino ......................... KINGSTON SNV3S500G · disco 0 do Windows · nvme0n1 · 976773168 setores de 512 B · 465,8 GB
  Cabe (R-7) ...................... ok · o destino tem exatamente o tamanho da origem
  Conferido contra a imagem ....... ok · `disk`, `nvme0n1-gpt.sgdisk` e `blkdev.list`
  Imagem criada por ............... /usr/sbin/ocs-sr -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true savedisk 2026-08-22_Apps nvme0n1

ATENCAO: a restauracao APAGA o disco de destino.
Tudo que estiver nele sera perdido.

Digite o nome da imagem para confirmar: 2026-08-22_Apps

  Entrada de firmware ............. ARCA · {f4057bd0-…} · partition=R:
  Receita armada .................. ok · R:\boot\grub\grub.cfg
  Boot unico ...................... ok · relido no bcdedit · {f4057bd0-…}
  Selo do job ..................... <16 digitos hexadecimais>
  Desfecho esperado em ............ E:\ARCA-LOGS\restauracao-2026-08-22_Apps\arca-fim.txt

A maquina vai reiniciar agora e desligar sozinha ao terminar.
AO TERMINAR: remova o SSD antes de religar.

  E REMOVER O SSD NAO E ZELO NESTA OPERACAO. O dispositivo esta em
  PRIMEIRO na ordem permanente de boot: enquanto ele estiver conectado,
  todo reinicio boota nele — sem boot unico nenhum. Entre o fim da
  restauracao e o `arca resultado` o `grub.cfg` continua armado, e um
  reinicio nessa janela RESTAURA DE NOVO, por cima do Windows que acabou
  de voltar.
  Remova o SSD ao desligar, religue, e so entao reconecte para
  `arca resultado`.

Reiniciando...
```

> **A tela mostra os dois discos em setores, e não só em GB.** É a comparação de R-7 impressa em vez de resumida: quem está prestes a apagar um disco tem de poder refazer a conta — e os dois números saírem da **mesma régua** é o achado que custou a etapa.

#### O que a restauração faz com a ordem de boot

Uma restauração **devolve a ordem permanente que está dentro da imagem** — o BCD mora na partição EFI, e ela é restaurada junto. Depois de uma restauração, a ordem de boot é a de quando o backup foi feito. Por isso o `arca resultado` põe o `{bootmgr}` de volta no topo ao colher.

#### E se o Windows não bootar mais?

O dispositivo é autocontido: boote nele pelo menu de boot do firmware (F12, ou a tecla da sua placa) e use o menu do Clonezilla à mão. Todas as imagens estão ali dentro, e o `arca.exe` também.

---

### 6.8 `arca status`

Diagnóstico completo: dispositivo, imagens, entrada de firmware, ordem de boot e job pendente. **Só lê.**

#### Sintaxe

```powershell
arca status
```

#### Exemplo real — 24/08/2026

```
> arca status

Dispositivo ARCA
  ARCAVAULT ....................... D: · NTFS · 236,9 GB
  ARCABOOT ........................ R: · FAT32 · 1,6 GB

Imagens em ARCAVAULT:
  2026-08-21_WindowsCompleto   21/08 · 36,2 GB · aprovada
  2026-08-22_Apps              22/08 · 39,7 GB · aprovada
  ARCA-TESTE-03                22/08 · 32,9 GB · aprovada

125 GB livres

Entrada de firmware
  Descricao ....................... ARCA
  Identificador ................... {f4057bd3-65a4-11f1-b0f1-aa4ed9bd2b34}
  Aponta para ..................... partition=R: · o ARCABOOT deste dispositivo
  Carrega ......................... \EFI\boot\bootx64.efi
  Ordem de boot ................... dispositivo em 2o de 5 · `Windows Boot Manager` vem antes

Ultimo job, ja colhido
  Boot unico ...................... nao armado
  Estado no ARCABOOT .............. verificacao `2026-08-22_Apps` · ja colhido, nada esperando
  Selo ............................ b668820c0a23ab5f
  Disco alvo ...................... nenhum · `verificacao` lê a imagem, e nao um disco
  Armado em ....................... 2026-08-24T16:28:48-03:00 · informativo, nunca comparado

  Este job ja foi colhido: o `arca resultado` leu o desfecho dele e disse
  o que era. O `estado.json` fica no dispositivo de proposito — e o unico
  registro que liga este selo a este nome, e o ARCA nao apaga nada (B-10).
```

#### As três leituras que ele faz

| Bloco | De onde vem |
|---|---|
| Dispositivo e imagens | Enumeração de volumes por rótulo + listagem de diretórios |
| Entrada de firmware e ordem de boot | `bcdedit /enum firmware`, parseado **por valor** e nunca pelo texto — que vem traduzido |
| Job | O `estado.json` do `ARCABOOT`, mais o `arca-fim.txt` se houver |

#### `Armado em` é informativo, e nunca comparado

O relógio do Clonezilla lê o RTC como UTC e fica 3 h adiantado, permanentemente. **Não há relógio comum entre os dois lados do reinício** — quem liga um job ao seu desfecho é o selo, nunca o tempo. Uma trava construída sobre comparação de datas já reprovou um backup perfeito neste projeto.

#### O que a linha `Ordem de boot` não afirma

Nem toda entrada da ordem permanente diz para onde aponta. As que o firmware acrescenta sozinho no POST — `UEFI:CD/DVD Drive`, `UEFI:Removable Device`, `UEFI:Network Device` — trazem só identificador e descrição, sem `device` nem `path`.

O ARCA lia essa ausência como *"não leva ao dispositivo"*. **Desde 24/08/2026 ele lê como "não sei":** o julgamento tem **três** estados — leva, não leva, e não se sabe — e *não se sabe* não autoriza afirmação de segurança em nenhuma tela.

---

### 6.9 `arca desarmar`

Devolve o dispositivo ao estado inerte: tira a receita do `grub.cfg`, devolve o `set default` e limpa a marca de boot único.

#### Sintaxe

```powershell
arca desarmar [--dry-run]
```

#### Ele não substitui o desarmar automático

Desarmar continua sendo o **primeiro passo, incondicional**, de todo comando que arma — não é algo que você precise lembrar de fazer. O comando existe para o caso em que **o boot não aconteceu**: o dispositivo ficou armado e não há nada a colher. É também a única forma de exercitar a idempotência do desarmar sem armar.

#### Saída — dispositivo que já estava inerte

```
Desarmando o dispositivo
  Desarmando receita anterior ..... ok · R:\boot\grub\grub.cfg
  Marca de boot unico ............. nao havia

Nao havia nada armado. O dispositivo ja estava inerte, e continua.
```

#### Saída — dispositivo armado

```
Desarmando o dispositivo
  Desarmando receita anterior ..... ok · R:\boot\grub\grub.cfg
  Marca de boot unico ............. removida · apontava para {f4057bd0-…}

Havia receita armada no grub.cfg, e ela foi tirada.
O `set default` do grub.cfg apontava para outra entrada e voltou para
`live-default` — o menu normal do Clonezilla. E ele que faz o boot ser
desatendido: sem isso, a receita so apareceria no menu.
O dispositivo boota no menu normal do Clonezilla.
```

> **A frase final diz o que de fato havia, e não só que havia algo.** Com o `grub.cfg` que o Clonezilla entrega — `set default="0"`, apontando por posição, sem nenhum `menuentry` do ARCA — dizer "havia receita armada" seria falso: **havia um `set default` que armaria sozinho na próxima inserção**, que é um problema diferente e merece ser nomeado.

#### Ele funciona mesmo com o `estado.json` ilegível

`arca desarmar` **não consulta estado nenhum**. É por isso que ele é a saída recomendada quando o `estado.json` não pode ser lido: o dispositivo pode estar armado, e este comando o desarma sem precisar saber qual job era.

---

## 7. As flags globais

```
--dry-run       # imprime o que faria; nao arma e nao escreve nada
--sem-pausa     # nao segura a janela ao terminar (oculta; para scripts)
```

As duas são **globais**: valem antes ou depois do subcomando.

```powershell
arca --dry-run backup 2026-08-24_Apps      # funciona
arca backup 2026-08-24_Apps --dry-run      # idêntico
```

E as três específicas de comando:

```
--completo              # em verify: arma boot unico para o ocs-chkimg
--dispositivo <indice>  # em prepare: o disco a preparar — obrigatorio
--iso <caminho>         # em prepare: instala de arquivo local
```

### `--dry-run` faz coisas diferentes em cada comando

| Comando | O que o ensaio faz |
|---|---|
| `backup`, `restore`, `verify --completo`, `sondar` | Imprime a receita inteira e a linha exata como ela entra no `grub.cfg`. **Não desarma, não arma, não pede confirmação, não reinicia** |
| `prepare` | Imprime o plano de partições e **para antes da pergunta**. É a única forma de ver o plano sem executá-lo |
| `desarmar` | Diz o que reescreveria, e não escreve |
| `list`, `status`, `resultado` | Não têm o que ensaiar — eles já só leem |

O selo impresso num ensaio é **de ensaio**: dezesseis zeros. A receita que o carrega não serviria, e a tela diz isso. O selo de verdade nasce **ao armar**, de uma fonte de entropia do sistema.

> **`--dry-run` já mentiu uma vez neste projeto**, dizendo `ok` sobre um desarmar que não aconteceu. Hoje a linha do desarme, no ensaio, diz literalmente `nao, e ensaio`.

### `--sem-pausa`

Sem ela, o ARCA segura a janela esperando Enter ao terminar — porque a janela que o UAC abre não é a mesma de onde o comando foi digitado, e sem a pausa a saída de um `arca list` piscaria e sumiria.

Ela é lida dos argumentos **brutos**, e não do que o `clap` entendeu: a decisão vale também quando a linha de comando foi recusada — que é exatamente quando há uma mensagem que alguém precisa ler antes de a janela sumir.

---

## 8. Workflow completo — do disco virgem ao Windows restaurado

O caminho inteiro, na ordem, com o que esperar de cada passo.

### Passo 0 — Compilar

```powershell
git clone https://github.com/carreirodev/ArcaBackup.git
cd ArcaBackup
cargo build --release
copy target\release\arca.exe C:\Ferramentas\arca.exe
```

### Passo 1 — Descobrir o índice do disco a preparar

```powershell
Get-Disk | Format-Table Number, FriendlyName, Size, BusType, IsSystem, IsBoot
```

```
Number FriendlyName        Size BusType IsSystem IsBoot
------ ------------        ---- ------- -------- ------
     0 KINGSTON SNV3S500G 465GB NVMe    True     True
     1 JMicron Generic    447GB USB     False    False
```

O disco 1 é o externo. **Confira o modelo e o tamanho** — é o modelo que você vai digitar daqui a pouco.

### Passo 2 — Ver o plano antes de executá-lo

```powershell
arca prepare --dispositivo 1 --dry-run
```

Leia a tela inteira. Ela diz o que existe no disco hoje, o que vai ficar no lugar, e o que mais vai acontecer — o download, a entrada de firmware, o `arca.exe`.

### Passo 3 — Preparar o dispositivo

```powershell
arca prepare --dispositivo 1
```

Responde `s`, depois digita o **modelo do disco**. Leva alguns minutos — a maior parte é o download de 535,5 MB. No fim, o dispositivo está pronto e **inerte**.

> Se você já tinha outro dispositivo ARCA conectado, **desconecte um dos dois agora**. O ARCA opera um por vez.

### Passo 4 — Sondar a máquina

```powershell
arca sondar
```

Responde `s`. A máquina reinicia, roda o `lsblk` sozinha e **desliga**. Leva ~1 min 40 s.

**Religue a máquina** e rode:

```powershell
arca resultado
```

Agora o oráculo existe: o ARCA sabe que o disco de sistema se chama `nvme0n1` para o Linux.

> Este passo só é necessário **num dispositivo sem imagem nenhuma**. Com pelo menos uma imagem no `ARCAVAULT`, o nome sai do `blkdev.list` de dentro dela.

### Passo 5 — Conferir o que o backup vai fazer

```powershell
arca backup 2026-08-24_Primeiro --dry-run
```

Confira a linha `Disco de origem`: ela tem de nomear um disco e dizer **de onde o nome veio**.

### Passo 6 — Fazer o backup

```powershell
arca backup 2026-08-24_Primeiro
```

Leia o pré-voo. Se algo estiver amarelo — Inicialização Rápida ligada, `chkdsk` acusando —, o ARCA diz o comando que resolve. Digite o nome da imagem por extenso.

A máquina reinicia. **Não há mais nenhuma tela**: o Clonezilla monta o `ARCAVAULT`, faz o `savedisk`, roda o `ocs-chkimg` sobre a imagem que acabou de criar, escreve o desfecho e **desliga**.

> ⚠️ **Ao desligar, remova o SSD antes de religar.** O ciclo de boot pelo dispositivo põe a entrada dele na frente da ordem permanente — enquanto ele estiver conectado, todo reinício boota nele.

### Passo 7 — Colher

Religue a máquina (sem o SSD), reconecte o dispositivo e rode:

```powershell
arca resultado
```

Você lê o desfecho, o veredito, o selo, e a listagem atualizada. O dispositivo é desarmado e o Windows volta ao topo da ordem de boot.

### Passo 8 — Conferir a imagem quando quiser

```powershell
arca list                                  # o que há no dispositivo
arca verify 2026-08-24_Primeiro            # os bytes ainda são os mesmos? (~3,5 min)
arca verify 2026-08-24_Primeiro --completo # a imagem é restaurável? (reinicia)
```

### Passo 9 — Restaurar

```powershell
arca restore
```

Escolhe o número, lê a conferência de setores, digita o nome por extenso. A máquina reinicia e restaura.

> ⚠️ **Aqui remover o SSD não é zelo.** Entre o fim da restauração e o `arca resultado`, o `grub.cfg` continua armado — e um reinício nessa janela **restaura de novo, por cima do Windows que acabou de voltar**.

Religue sem o SSD, reconecte, e:

```powershell
arca resultado
```

### O ciclo, resumido

```
prepare  ──▶  sondar  ──▶  resultado  ──▶  backup  ──▶  resultado  ──▶  list
   │           (só na primeira vez)            ▲                         │
   │                                           └───── repita ────────────┘
   │
   └──▶ verify [--completo]  ·  status  ·  desarmar  ·  restore ──▶ resultado
```

**Toda operação que reinicia termina com um `arca resultado`.** Um job armado e não colhido aparece no `arca status` como algo esperando.

---

## 9. As quatro receitas, byte a byte

A **receita** é a string que o Clonezilla executa sozinho no boot desatendido. Ela é gravada no `grub.cfg` a cada operação, dentro de um `menuentry` derivado do próprio `live-toram` do dispositivo.

**Ela não é um script.** É uma linha, dentro de `ocs_live_run="bash -c '...'"`, e a linha inteira tem de caber no `COMMAND_LINE_SIZE` do kernel — 2048 caracteres, dos quais 512 ficam reservados para o resto do `menuentry`. Um pipe, uma aspa, uma substituição de comando ou um caractere não-ASCII **invalidam a receita inteira** antes de qualquer gravação.

### Como ela entra no `grub.cfg`

```
locales=en_US.UTF-8 keyboard-layouts=NONE ocs_repository="dev:///LABEL=ARCAVAULT"
ocs_live_run="bash -c '<A RECEITA>'" ocs_live_batch="yes"
```

Cinco parâmetros. Note o `dev:///LABEL=ARCAVAULT` — **o destino é sempre por rótulo**, nunca por letra, `sda` ou número de série.

### Receita de backup

```bash
mkdir -p /home/partimag/ARCA-LOGS/backup-<nome>
echo ARCA_SELO=<selo> > /home/partimag/ARCA-LOGS/backup-<nome>/arca-fim.txt
if ocs-sr -q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true savedisk <nome> <disco>
then
   echo ARCA_BACKUP=OK >> .../arca-fim.txt
   if ocs-chkimg -b -or /home/partimag <nome> > /home/partimag/<nome>/arca-check.log 2>&1
   then echo ARCA_VEREDITO=APROVADA >> .../arca-check.log
   else echo ARCA_VEREDITO=REPROVADA >> .../arca-check.log
   fi
else
   echo ARCA_BACKUP=FALHOU >> .../arca-fim.txt
fi
echo ARCA_FIM >> .../arca-fim.txt
sleep 20
poweroff
```

*(quebrada em linhas para leitura — na realidade é uma linha só, com `;`)*

Cada passo tem uma razão:

| Passo | Por quê |
|---|---|
| `mkdir -p` primeiro | Sem ele o primeiro `>` falha e o desfecho inteiro se perde |
| O selo com `>` | Garante que ele seja a **primeira** linha do `arca-fim.txt` |
| O `savedisk` dentro de um `if` | `;` não olha código de saída |
| O `ocs-chkimg` **dentro do ramo de êxito** | Com o backup falhando, a pasta da imagem pode nem existir |
| `ARCA_FIM` no fim | É o que separa um desfecho completo de um truncado por desligamento no meio |
| `sleep 20` | Dá tempo de o `echo` chegar ao disco antes de o `poweroff` cortar |

**As flags nunca mudam de ordem**: `-q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true` é a sequência exata que rodou nos backups validados. `-batch` é o que suprime as perguntas; `-p true` é o que impede o `ocs-sr` de reiniciar antes de o `ocs-chkimg` rodar. **Nunca `-scs`** — ele pula a conferência nativa, o oposto do que se quer.

### Receita de restauração

```bash
mkdir -p /home/partimag/ARCA-LOGS/restauracao-<nome>
echo ARCA_SELO=<selo> > .../arca-fim.txt
if ocs-sr -e1 auto -e2 -batch -j2 -k0 -iefi -p true restoredisk <nome> <disco> > .../arca-restore.log 2>&1
then echo ARCA_RESTORE=OK >> .../arca-fim.txt
else echo ARCA_RESTORE=FALHOU >> .../arca-fim.txt
fi
echo ARCA_FIM >> .../arca-fim.txt
sleep 20
poweroff
```

Sem verificação: aqui não há imagem nova para conferir. O log do Clonezilla vai para dentro do `ARCA-LOGS`, no `ARCAVAULT` — **que a restauração não toca**. A imagem substitui o disco interno, e o desfecho sobrevive num disco que não estava no caminho.

### Receita de verificação (`verify --completo`)

```bash
mkdir -p /home/partimag/ARCA-LOGS/verificacao-<nome>
echo ARCA_SELO=<selo> > .../arca-fim.txt
if ocs-chkimg -b -or /home/partimag <nome> >> /home/partimag/<nome>/arca-check.log 2>&1
then echo ARCA_VEREDITO=APROVADA >> .../arca-check.log; echo ARCA_VERIFY=OK   >> .../arca-fim.txt
else echo ARCA_VEREDITO=REPROVADA >> .../arca-check.log; echo ARCA_VERIFY=FALHOU >> .../arca-fim.txt
fi
echo ARCA_FIM >> .../arca-fim.txt
sleep 20
poweroff
```

Note o `>>` no `arca-check.log`, onde o backup usa `>`: lá a imagem acabou de nascer e o log não existe; aqui ele existe, e o `>` **trunca ao abrir** — um desligamento nessa janela deixaria uma imagem boa com o log em zero byte.

### Receita de sondagem

```bash
mkdir -p /home/partimag/ARCA-LOGS/sondagem
echo ARCA_SELO=<selo> > .../arca-fim.txt
if lsblk -i -o KNAME,NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT,MODEL > .../blkdev.list 2>&1
then echo ARCA_PROBE=OK >> .../arca-fim.txt
else echo ARCA_PROBE=FALHOU >> .../arca-fim.txt
fi
echo ARCA_FIM >> .../arca-fim.txt
sleep 20
poweroff
```

A única das quatro que **não chama programa nenhum do Clonezilla**. O `2>&1` aponta para o próprio `blkdev.list`: falhando o `lsblk`, o arquivo fica com a mensagem dele em vez de vazio, e a próxima sessão lê **qual** flag foi recusada em vez de deduzir.

### O selo

Dezesseis dígitos hexadecimais minúsculos, gerados **ao armar** a partir de 8 bytes de entropia do sistema (`BCryptGenRandom`). Ele passa por três lugares:

```
estado.json          "selo": "7d2d2f5153625b38"
    ↓ embutido na receita
arca-fim.txt         ARCA_SELO=7d2d2f5153625b38     ← primeira linha, escrita com `>`
    ↓ lido de volta
arca resultado       Selo: 7d2d2f5153625b38
```

Ele resolve quatro casos com um mecanismo só: desfecho de um job anterior; desfecho vindo de dentro de uma imagem antiga (**job fantasma**); desfecho ausente porque o boot nunca aconteceu; e arquivo truncado por desligamento no meio.

**Sem selo, o ARCA não arma.** Um job sem selo é um job cujo desfecho ninguém consegue reclamar.

### Ver as receitas sem armar nada

```powershell
arca backup <nome> --dry-run
arca restore <nome> --dry-run
arca verify <nome> --completo --dry-run
arca sondar --dry-run
```

Ou, sem dispositivo nem elevação:

```powershell
cargo run --example receita_ao_lado_da_que_rodou
```

Que imprime a receita gerada ao lado da que rodou em hardware, para comparação a olho.

---

## 10. Todos os arquivos que o ARCA lê e escreve

### No dispositivo — `ARCABOOT` (FAT32)

| Caminho | Quem escreve | Quem lê | O que é |
|---|---|---|---|
| `boot\grub\grub.cfg` | ARCA (armar/desarmar) e o pacote do Clonezilla | o grub, no boot | **A receita.** Único arquivo de que a máquina depende para bootar |
| `arca\estado.json` | ARCA, ao armar e ao colher | ARCA (`resultado`, `status`) | O job: selo, operação, nome, disco, momento, situação |
| `arca\arca.exe` | ARCA (`prepare`) | você, quando o Windows não boota | O próprio ARCA, fora do disco que a restauração substitui |
| `EFI\boot\bootx64.efi`, `live\` | o pacote do Clonezilla | o firmware | O ambiente de boot |

O `estado.json` é um JSON escrito e lido à mão, sem dependência nova:

```json
{
  "selo": "354da624e7fa0d21",
  "comando": "sondagem",
  "nome": "",
  "disco": "",
  "armado_em": "2026-08-24T14:56:55-03:00",
  "situacao": "colhido"
}
```

`nome` e `disco` vazios são sentinela: a sondagem não opera sobre imagem nenhuma e não nomeia disco.

### No dispositivo — `ARCAVAULT` (NTFS)

| Caminho | Quem escreve | O que é |
|---|---|---|
| `<nome>\` | o Clonezilla (`savedisk`) | Uma **imagem**. Nomeada por você, nunca sobrescrita |
| `<nome>\MD5SUMS` | o Clonezilla | **É o que distingue imagem de resíduo** |
| `<nome>\arca-check.log` | o `ocs-chkimg`, pela receita | O **veredito**, terminando em `ARCA_VEREDITO=` |
| `<nome>\blkdev.list` | o Clonezilla | Uma das duas fontes do **oráculo** |
| `ARCA-LOGS\backup-<nome>\arca-fim.txt` | a receita | O **desfecho** do job |
| `ARCA-LOGS\restauracao-<nome>\arca-fim.txt` | a receita | idem |
| `ARCA-LOGS\restauracao-<nome>\arca-restore.log` | o `ocs-sr`, pela receita | A saída da restauração |
| `ARCA-LOGS\verificacao-<nome>\arca-fim.txt` | a receita | idem |
| `ARCA-LOGS\sondagem\arca-fim.txt` | a receita | idem — pasta **fixa**, substituída a cada sondagem |
| `ARCA-LOGS\sondagem\blkdev.list` | o `lsblk`, pela receita | A outra fonte do **oráculo** |
| `clonezilla-live-*.zip` | ARCA (`prepare`) | Cópia do pacote, para o dispositivo se reconstruir sozinho |

**Cada operação escreve numa pasta de log própria**, e o motivo é concreto: toda receita começa truncando o próprio `arca-fim.txt`, e duas operações que dividissem a pasta apagariam o desfecho uma da outra.

### No `C:` — e ele não sobrevive a uma restauração

| Caminho | O que é |
|---|---|
| `%LOCALAPPDATA%\ARCA\arca.log` | O registro local. **Descartável** |
| `%LOCALAPPDATA%\ARCA\arca.log.anterior` | A geração anterior, rotacionada a 1 MB |

O registro anota a linha de comando **bruta** de cada invocação — que é o que denuncia um argumento perdido na elevação. Trecho real do `arca.log` de 22/08/2026, preservado em `recursos/capturas/arca-log-windows-2026-08-23-pos-restauracao.txt` (a palavra `suposto` é daquela versão do código, de antes de o oráculo do §4.5 existir):

```
2026-08-22 15:42:24.995 INFO  [16744] arca 0.1.0 · elevado=sim · linha=["backup", "2026-08-22_Apps", "--dry-run", "--sem-pausa"]
2026-08-22 15:42:24.996 INFO  [16744] comando `backup` (ensaio)
2026-08-22 15:42:24.996 INFO  [16744] ensaio de backup `2026-08-22_Apps` · disco nvme0n1 (suposto) · receita de 813 caracteres · validada por C-2
```

Nenhuma falha de escrita aqui interrompe uma operação: perder o registro é ruim, parar um backup por causa dele seria pior. Sem `%LOCALAPPDATA%`, ele cai no diretório temporário.

### O que o ARCA nunca escreve

- **Nunca o disco em acesso raw.** Nenhuma assinatura das portas entrega um handle de dispositivo, um caminho de dispositivo bruto ou um deslocamento em setores. O que elas oferecem é metadado — rótulo, tamanho, modelo, espaço livre — e conversa com ferramentas do próprio Windows. Há um teste de arquitetura que cobra isso a cada build.
- **Nunca apaga nada.** Nem imagem, nem resíduo, nem `estado.json`, nem log. A única exceção é o `arca prepare`, que apaga o disco inteiro que você nomeou e confirmou por escrito.

---

## 11. Códigos de saída

| Código | Quando |
|---|---|
| `0` | Sucesso |
| `1` | Falha — infraestrutura, recusa de pré-voo, desfecho ruim, imagem reprovada |
| `2` | Uso incorreto: linha de comando recusada pelo `clap`, **nome de imagem inválido**, elevação recusada |

O `2` separa *"você digitou um nome inválido"* de *"alguma coisa falhou"* — é o mesmo código que o `clap` usa para uso incorreto.

**Um desfecho ruim sai com código diferente de zero depois de a tela inteira ter sido impressa.** Quem chama o ARCA de um script não pode ler uma imagem reprovada, ou um backup truncado, como êxito.

```powershell
arca resultado --sem-pausa
if ($LASTEXITCODE -ne 0) { Write-Host "o backup nao ficou bom" }
```

Quando o ARCA se relança elevado, o código do processo elevado é propagado. O que não cabe num byte vira `1`, **nunca `0`** — as terminações anormais do Windows são códigos negativos, e reduzi-las a zero diria "deu certo" sobre um processo que morreu no meio.

---

## 12. Quando dá errado — diagnóstico e recuperação

### `nenhum dispositivo ARCA conectado`

```
erro: nenhum dispositivo ARCA conectado: nao ha volume que responda pelo rotulo
ARCAVAULT. Ou o dispositivo nao esta conectado, ou o volume dele nao respondeu —
um volume bloqueado pelo BitLocker ou ainda montando nao aparece
```

**Duas causas, e a mensagem nomeia as duas.** Um volume que existe mas não responde à consulta some da enumeração do mesmo jeito que um dispositivo desconectado. Espere alguns segundos e tente de novo; confira no Explorador se o volume aparece.

### `ha 2 volumes com o rotulo ARCAVAULT conectados (D:, E:)`

Desconecte um dos dois. Se você acabou de rodar `arca prepare`, são o novo e o de antes.

### `o volume ARCAVAULT nao tem letra atribuida`

Atribua uma no Gerenciamento de Disco. Sem letra não há caminho por onde lê-lo do lado Windows.

### `o nome que o Linux da ao disco de origem nao foi determinado`

Rode `arca sondar`, religue, e `arca resultado`. Depois disso o `arca backup` acha o disco sozinho.

### `o estado do job nao pode ser lido`

```
Isto NAO quer dizer que nao ha job: o dispositivo pode estar armado, e o que
dizia qual job era este se perdeu.
```

1. `arca status` — para ver se há boot único armado.
2. `arca desarmar` — ele **não consulta estado nenhum**, e por isso funciona justamente aqui.

O ARCA não apaga o arquivo.

### `O dispositivo FICOU ARMADO e a maquina nao reiniciou`

O `shutdown` falhou depois de o dispositivo já estar armado. **O próximo reinício, venha de onde vier, boota no dispositivo e roda a receita.** Para desfazer:

```powershell
arca desarmar
```

### O boot não aconteceu: `arca resultado` diz que não há desfecho

```
Sem `arca-fim.txt`, com job pendente: o boot nao aconteceu, ou o Clonezilla
abriu menu
```

**São as duas causas, e o ARCA não escolhe uma.** O que investigar:

- O SSD estava conectado no momento do reinício?
- A entrada de firmware ainda aponta para o `ARCABOOT` deste dispositivo? (`arca status`)
- O `grub.cfg` do dispositivo ainda tem a receita? Se o Clonezilla abriu o menu e alguém desligou a máquina, o dispositivo continua armado.

O comando desarma e encerra o job de qualquer forma — ausência de desfecho é um veredito.

### `a marca de boot unico continua no firmware depois de mandada apagar`

O `bcdedit` respondeu "êxito" sem ter feito nada — é o modo de falha medido desde o começo deste projeto, e é por isso que **toda escrita no firmware é conferida com uma releitura**. Rode `arca status` e confira o firmware antes de reiniciar.

### `a entrada de firmware … devia apontar para … e a releitura mostra …`

A rejeição silenciosa: o `bcdedit` aceita o comando, responde "êxito" e mantém o valor antigo quando o alvo é mídia removível. Um dispositivo assim **boota por F12, nunca por entrada de firmware**.

### `o SHA256 do pacote NAO bate`

```
O ARCA para aqui e nao extrai nada. O numero esperado esta compilado neste
binario e nao veio junto do download.
```

Ou o download veio corrompido — rode de novo —, ou o arquivo do outro lado não é o que este ARCA conhece. **Neste ponto o disco já foi apagado**: o dispositivo fica vazio, com as duas partições prontas e sem Clonezilla. Rodar o `prepare` de novo resolve.

### `o disco N NAO e mais o que estava no plano`

Entre imprimir o plano e escrever a tabela houve uma pessoa lendo e digitando, e nesse intervalo alguém mexeu num cabo. **Nada foi apagado.** Rode `arca prepare --dispositivo <indice>` de novo e confira o modelo e o tamanho na tela.

### A máquina passou a bootar no Clonezilla sozinha

O ciclo de boot pelo dispositivo põe a entrada dele na frente da ordem permanente. É o que o aviso de C-9 previne — remover o SSD antes de religar. Para consertar:

```powershell
arca resultado      # ele devolve o {bootmgr} ao topo da ordem
```

ou, se não houver job a colher, desconecte o dispositivo e religue: `arca status` mostra a ordem atual.

### Elevação recusada

```
erro: elevacao recusada: o ARCA escreve no grub.cfg e fala com o bcdedit, e
nenhuma das duas coisas roda sem privilegio administrativo
```

Não é falha do ARCA — é uma decisão sua no diálogo do UAC. Sai com código `2`.

### Onde olhar quando nada disso explica

```powershell
notepad "$env:LOCALAPPDATA\ARCA\arca.log"
arca status
```

E, no dispositivo:

```
<ARCABOOT>\arca\estado.json                        ← qual job estava armado
<ARCABOOT>\boot\grub\grub.cfg                      ← a receita, se ainda estiver lá
<ARCAVAULT>\ARCA-LOGS\<operacao>-<nome>\arca-fim.txt
<ARCAVAULT>\ARCA-LOGS\restauracao-<nome>\arca-restore.log
<ARCAVAULT>\<nome>\arca-check.log
```

---

## 13. As regras que o ARCA nunca quebra

Cada uma custou uma execução real para existir. Elas têm identificadores no código e nos comentários — `C-1`, `B-4`, `R-7`… — e é assim que os testes as cobram.

### Comuns a toda operação

| ID | Regra |
|---|---|
| **C-1** | Desarmar a receita anterior **incondicionalmente**, como primeiro passo, sem consultar estado nenhum |
| **C-2** | Validar a receita **antes de gravar**: sem pipe, sem aspa, sem substituição de comando, sem caractere de controle, sem não-ASCII, e cabendo no `COMMAND_LINE_SIZE` |
| **C-3** | **Nunca confiar no retorno do `bcdedit`** — sempre conferir com `/enum` e parsear **por valor** |
| **C-4** | Manter **uma** entrada de firmware chamada `ARCA`; não havendo, migrar a legada em vez de criar outra |
| **C-5** | Boot **único** — armar e desarmar nunca alteram a ordem permanente, e releem para provar que não a alteraram |
| **C-6** | Recusar mídia removível como alvo de entrada de boot, e orientar F12 |
| **C-7** | Repassar os argumentos **brutos** ao relançar com elevação |
| **C-8** | Escapar aspas com **barra invertida**, não crase — quem reparte a linha é o parser do Windows |
| **C-9** | Avisar para remover o SSD **depois de armado e antes de reiniciar** — é a última coisa que alguém lê antes de a tela apagar |
| **C-10** | Recusar rótulo repetido |
| **C-11** | Gerar um **selo** ao armar, gravá-lo no `estado.json` e embuti-lo na receita |
| **C-12** | **Ausência de desfecho é falha, nunca silêncio** — e reporta as duas causas possíveis |
| **C-13** | Ao colher, devolver o `{bootmgr}` ao topo da ordem permanente — sem remover nada |
| **C-14** | **Ausência de resposta do firmware nunca vira segurança.** Três estados: leva, não leva, não se sabe |

### Backup

| ID | Regra |
|---|---|
| **B-1** | Localizar o dispositivo pela partição `ARCAVAULT` |
| **B-2** | Recusar nome inválido — por **lista de permissão** |
| **B-3** | Recusar nome cuja pasta já exista, **mesmo sem `MD5SUMS`** |
| **B-4** | Espaço mínimo: o maior entre *maior imagem × 1,3* e *em uso × 0,45* |
| **B-5** | Verificar a Inicialização Rápida **no registro**, nunca no `powercfg /a` — que responde traduzido |
| **B-6** | Rodar `chkdsk /scan` no volume do sistema, julgado **pelo código de saída** |
| **B-7** | Receita com nome e disco embutidos, **sem `ask_user`** |
| **B-8** | Sempre as mesmas flags, **na mesma ordem**. Nunca `-scs` |
| **B-9** | Sempre chamar o `ocs-chkimg` explicitamente, **dentro do ramo de êxito** do `savedisk` |
| **B-10** | **Nunca apagar nada** |

### Restauração

| ID | Regra |
|---|---|
| **R-1** | Listar as imagens **no Windows** — a escolha acontece antes do reinício |
| **R-2** | Conferir o destino contra o `disk`/`blkdev.list` de dentro da imagem |
| **R-3** | Exigir o nome da imagem digitado **por extenso** |
| **R-4** | Sempre as mesmas flags, nesta ordem, sempre **sem** `-g auto` |
| **R-5** | Receita com `if/then/else`, escrevendo `OK` ou `FALHOU` |
| **R-6** | Ler o desfecho na volta e **conferir o selo antes de acreditar nele** |
| **R-7** | **O único destino válido é o disco de origem**, e a medição prova **identidade**: os setores têm de bater exatamente |
| **R-8** | **Recusar o próprio dispositivo ARCA como destino, sempre**, e sem confirmação que libere |

### Segurança

| ID | Regra |
|---|---|
| **S-1** | O ARCA **nunca abre o disco de origem em acesso raw** de escrita |
| **S-2** | Operação destrutiva exige **texto digitado**, nunca só `s`. Comparação exata, uma tentativa só |
| **S-3** | Destino sempre por LABEL — nunca por letra, `sda` ou número de série |
| **S-4** | Veredito e desfecho sempre gravados **em arquivo**, nunca só em tela |
| **S-5** | **Falha parcial é falha total** |
| **S-6** | **Nunca comparar uma data escrita pelo Windows com outra escrita pelo Linux.** O que liga um job ao desfecho é o selo |

### Armadilhas conhecidas, e quem as defende

| Armadilha | Efeito | Defesa |
|---|---|---|
| Pipe na receita | O Clonezilla ignora tudo e abre o menu — indistinguível de "o boot não funcionou" | C-2 |
| `bcdedit` respondendo "êxito" sem escrever | Uma tela que afirma o que não aconteceu | C-3 |
| Mídia removível como alvo de entrada de boot | Rejeição **silenciosa** | C-6 |
| Relógio do Clonezilla 3 h adiantado | Uma trava por data reprova um backup perfeito | C-11, S-6 |
| Job fantasma vindo de dentro de uma imagem antiga | Colher o desfecho errado | C-11 |
| `set default="0"` do pacote | Um dispositivo que *parece* inerte e arma sozinho | O `prepare` desarma o que instala |
| Índice de disco mudando entre o plano e o "sim" | Apagar o disco errado | A releitura de PR-4 |

---

## 14. Arquitetura do código

### O desenho em uma frase

**Toda conversa do ARCA com o mundo passa por uma porta**, e cada porta tem um adaptador de verdade e um duplo de mentira. É o que permite que o parser do `bcdedit`, o validador da receita e a regra de espaço tenham teste **sem hardware**.

```
src/
├── main.rs          # fino de propósito: colhe argumentos, registra, eleva, despacha
├── app.rs           # o Contexto (as portas + --dry-run) e o match dos comandos
├── cli.rs           # a superfície de linha de comando (clap derive)
│
├── comandos/        # um arquivo por comando — falam com o mundo e montam telas
│   ├── prepare.rs   restore.rs   backup.rs   resultado.rs
│   ├── verify.rs    sondar.rs    status.rs   list.rs    desarmar.rs
│
├── portas/          # as fronteiras perigosas, cada uma atrás de um trait
│   ├── firmware.rs      # o bcdedit
│   ├── discos.rs        # enumeração de volumes e discos físicos
│   ├── arquivos.rs      # leitura, escrita atômica, listagem
│   ├── sistema.rs       # chkdsk, registro, certutil, curl, bsdtar, shutdown
│   ├── particionador.rs # a única porta cuja operação destrói um disco
│   ├── entropia.rs      # de onde sai o selo
│   ├── console.rs       # o que o usuário digita
│   ├── privilegios.rs   # elevado? relançar elevado
│   └── relogio.rs
│
├── adaptadores/     # as implementações de verdade
│   ├── windows/     # bcdedit, WMI/CIM, PowerShell, registro, BCryptGenRandom…
│   └── ...
│
├── duplos.rs        # as implementações de mentira, para os testes
│
└── (código puro — sem I/O, tudo testável)
    receita.rs      # monta e valida as quatro receitas (C-2)
    grub.rs         # as duas operações inversas sobre o texto do grub.cfg
    menuentry.rs    # deriva o bloco do ARCA do live-toram do próprio dispositivo
    armar.rs        # a ordem das três gravações, com releitura de cada uma
    desarme.rs      # o caminho de volta ao estado inerte
    estado.rs       # o estado.json, escrito e lido à mão
    desfecho.rs     # julga o arca-fim.txt pelo selo
    prevoo.rs       # B-3, B-4, C-6, C-10, B-5, B-6
    preparacao.rs   # as sete defesas e o plano de partições
    blkdev.rs       # o parser do oráculo (§4.5)
    gpt.rs          # o parser do sgdisk de dentro da imagem (R-7)
    md5sums.rs      # o parser do MD5SUMS
    verificacao.rs  # a conferência de V-1
    nome.rs         # B-2
    imagens.rs      # imagem × resíduo × veredito
    espaco.rs, formato.rs, ordem.rs, firmware.rs, sondagem.rs,
    pacote.rs, resumo.rs, dispositivo.rs, confirmacao.rs,
    elevacao.rs, registro.rs, erro.rs
```

### As nove portas

| Porta | O que atravessa | Adaptador no Windows |
|---|---|---|
| `Firmware` | `bcdedit /enum`, `/set`, `/copy`, `/deletevalue` | `Bcdedit` |
| `Discos` | volumes por rótulo, discos físicos, `MediaType` | WMI/CIM via PowerShell |
| `Arquivos` | ler, escrever **atomicamente**, listar, criar diretório | `ArquivosDoSistema` |
| `Sistema` | Inicialização Rápida (registro), `chkdsk`, `certutil`, `curl`, `bsdtar`, `shutdown` | `SistemaDoWindows` |
| `Particionador` | `Clear-Disk`, `New-Partition`, `Format-Volume` | `ParticionadorDoWindows` |
| `Entropia` | `BCryptGenRandom` | `EntropiaDoWindows` |
| `Console` | o que o usuário digita | `ConsoleDoUsuario` |
| `Privilegios` | elevado? relançar elevado | `PrivilegiosDoWindows` |
| `Relogio` | agora | `RelogioDoSistema` |

**`S-1` é uma propriedade destas assinaturas.** Nenhuma delas entrega um handle de dispositivo, um caminho bruto ou um deslocamento em setores. Uma porta que precisasse abrir o disco em modo raw não teria como ser acrescentada sem que a assinatura denunciasse — e há um teste que cobra isso.

### Duas decisões que se sentem em todo o código

**Nomes em português.** O vocabulário do código é o do `CONTEXT.md`: *dispositivo*, *receita*, *job*, *armar*, *desarmar*, *selo*, *desfecho*, *veredito*, *resíduo*. Onde o código diverge do glossário, é o código que está errado.

**Comentários que dizem o porquê, e o que custou.** Os comentários deste projeto não repetem o que o código faz — eles registram a medição que motivou aquela linha, o erro que ela previne, e às vezes a data em que isso foi descoberto. É por isso que `cargo doc` vale a leitura.

---

## 15. Testes, exemplos e medições

### A suíte

```powershell
cargo test                                # 840 testes
cargo test --test e12_sondar_a_maquina    # um arquivo
cargo test -- --nocapture                 # mostrando o que os testes imprimem
```

| Arquivo | O que prova |
|---|---|
| `tests/e1_dispositivo_conectado.rs` | A descoberta acha **o hardware desta mesa** |
| `tests/e2_firmware_desta_maquina.rs` | O parser lê o que o `bcdedit` **de verdade** escreve |
| `tests/e4_desarmar_o_dispositivo.rs` | O desarmar devolve o inerte no dispositivo real |
| `tests/e7_armar_o_dispositivo.rs` | A ordem das três gravações e a releitura de cada uma |
| `tests/e8_colher_o_desfecho.rs` | A colheita contra o desfecho real |
| `tests/e9_restaurar_o_disco.rs` | R-7 contra o `MSFT_Disk` e o `sgdisk` da imagem |
| `tests/e10_preparar_o_dispositivo.rs` | As sete defesas contra a estrutura medida |
| `tests/e11_verificar_a_imagem.rs` | O `MD5SUMS` e o `certutil` |
| `tests/e12_sondar_a_maquina.rs` | SD-1 a SD-6 |
| `tests/repasse_de_argumentos.rs` | **C-7 e C-8 contra o Windows de verdade** — quem julga se o escape está certo é o parser do Windows, não uma reimplementação dele |
| `tests/s1_nenhum_acesso_raw.rs` | S-1 como propriedade da arquitetura |
| `tests/s6_o_tempo_nao_decide.rs` | S-6: nenhum módulo que julga desfecho alcança o tempo |
| `tests/b10_nada_e_apagado.rs` | B-10 como propriedade do código |

Os testes que precisam do hardware **se pulam sozinhos**, dizendo por quê.

### Os exemplos — diagnósticos que responderam perguntas caras

```powershell
cargo run --example <nome>
```

| Exemplo | A pergunta que ele respondeu |
|---|---|
| `receita_ao_lado_da_que_rodou` | Como a receita gerada se parece com a que rodou em hardware, quando alguém olha |
| `codificacao_do_bcdedit` | Em que página de código o `bcdedit` escreve quando o ARCA o chama |
| `ponto_no_console` | O `·` das telas sobrevive ao console sem mexer na página de código |
| `escrita_atomica_no_fat32` | A escrita atômica se comporta em FAT32 como em NTFS |
| `estado_no_arcaboot` | O `estado.json` no `ARCABOOT` de verdade, que é FAT32 |
| `orcamento_da_linha_do_kernel` | Quanto da linha de comando do kernel o marco de 22/08 gastou |
| `eco_argumentos` | Imprime cada palavra que o Windows entregou — usado pelos testes de C-7 |

### Os scripts de medição

```bash
bash recursos/ensaio-da-receita.sh                  # roda as receitas num bash de verdade
bash recursos/medir-arca-restore-log.sh <caminho>   # mede um arca-restore.log
```

O primeiro existe porque os testes provam o que a **string** contém, e não o que o **bash** faz com ela. Ele substitui o Clonezilla por comandos falsos que saem com o código que se pedir, e confere se cada desfecho escreve o rastro certo.

---

## 16. Documentação do projeto

| Onde | O que é |
|---|---|
| **`CONTEXT.md`** | O **glossário do domínio**. Cada termo com a definição, o que evitar, e por que a distinção existe. Comece por aqui |
| **`PRD/PRD-ARCA-v5_1.md`** | O documento de requisitos, com as telas de execução real, as medições e os requisitos identificados (`C-*`, `B-*`, `R-*`, `S-*`, `L-*`, `V-*`, `PR-*`, `SD-*`) |
| **`PRD/implementation_stages.md`** | O plano de etapas — E0 a E12 —, o que cada uma entregou e o que aprendeu |
| **`PRD/o-que-falta-para-fechar.md`** | As pendências abertas |
| **`docs/adr/`** | 23 decisões de arquitetura, cada uma com o contexto que a forçou |
| **`recursos/capturas/`** | **A evidência.** Saídas reais de ferramentas, `grub.cfg` preservados, logs de execução, o `estado.json` de cada marco. Ver `PROVENIENCIA.md` |
| **`docs/agents/`** | Convenções para agentes que trabalham neste repositório: issue tracker, labels de triagem, docs de domínio |
| `cargo doc --open` | A documentação interna — densa, e onde as razões estão |

### As decisões de arquitetura

| ADR | Decisão |
|---|---|
| 0001 | Um job é ligado ao seu desfecho por **selo**, nunca por data |
| 0002 | A receita é uma **string no `grub.cfg`**, não um script em arquivo |
| 0003 | O veredito é lido do `arca-check.log`, com marcador explícito |
| 0004 | A receita **transcreve o que rodou**, e marca o que não tem original |
| 0005 | O estado inerte se **reconstrói** do `grub.cfg` corrente |
| 0006 | O selo vem do Windows e o `estado.json` se escreve à mão |
| 0007 | O bloco do ARCA deriva do `menuentry --id live-toram` do próprio dispositivo |
| 0008 | Colher **marca** o `estado.json` como colhido, em vez de apagá-lo |
| 0009 | A ordem permanente muda **no ciclo de boot**, e não à mão |
| 0010 | R-7 recusa por medição — e a régua do destino é o `MSFT_Disk` |
| 0011 | As capturas de 21/08 são de dois boots |
| 0012 | A restauração devolve a ordem permanente **que está dentro da imagem** |
| 0013 | Colher devolve o `{bootmgr}` ao topo da ordem |
| 0014 | **O ARCA particiona o dispositivo** |
| 0015 | A restauração só restaura **no disco de origem** |
| 0016 | A verificação armada é a terceira operação |
| 0017 | A entrada de firmware nasce de uma **cópia do `{bootmgr}`**, e sai da ordem |
| 0018 | O pacote é o zip, e o `prepare` **desarma o que acabou de instalar** |
| 0019 | A sondagem é a quarta operação, e é a segunda fonte do oráculo |
| 0020 | O `bcdedit /enum firmware` **lê a NVRAM** |
| 0021 | Uma entrada sem alvo na ordem **não é segurança** |
| 0022 | O `arca-restore.log` é truncado **por baixo** |
| 0023 | O `bootsequence` não é o gatilho da reescrita |

---

## 17. Glossário

| Termo | O que é | Evitar dizer |
|---|---|---|
| **Dispositivo** | O SSD externo que carrega o Clonezilla e as imagens juntos, com as partições `ARCABOOT` e `ARCAVAULT`. O que separa um disco de um dispositivo é o `arca prepare` — e não haver sido comprado como tal | pendrive, mídia, unidade, drive |
| **Preparar** | Transformar um disco num dispositivo. É a **única** operação que destrói dados sem reiniciar | formatar, instalar, inicializar |
| **`ARCABOOT`** | A partição FAT32 de onde a máquina boota. Guarda o Clonezilla, o `grub.cfg` e o estado do job. **Sempre fora da imagem** | partição de boot, EFI |
| **`ARCAVAULT`** | A partição NTFS onde as imagens e os logs ficam. É o que o Clonezilla monta como `/home/partimag` | repositório, storage, cofre |
| **Imagem** | Uma pasta no `ARCAVAULT` com o resultado de um `savedisk`. Nomeada por você, **nunca sobrescrita** | backup, snapshot, ponto de restauração |
| **Resíduo** | Pasta de imagem **sem `MD5SUMS`** — rastro de um backup interrompido. Não é imagem, e o ARCA nunca escreve por cima de uma | imagem corrompida, imagem parcial |
| **Receita** | A string que o Clonezilla executa sozinho no boot desatendido, gravada no `grub.cfg` a cada operação. Um pipe dentro dela invalida a string inteira | script, comando, configuração |
| **Job** | Uma operação armada e ainda não colhida. Existe entre o reinício e a leitura do desfecho | tarefa, execução |
| **Operação** | O que a receita executa, e o que dá nome à pasta do desfecho: **backup**, **restauração**, **verificação** ou **sondagem** | comando, ação |
| **Sondagem** | A operação que descobre os discos desta máquina. Não chama programa nenhum do Clonezilla, e é a única cujo pior caso não envolve gravação | scan, detecção, inventário |
| **Oráculo** | O `blkdev.list` de onde sai o nome que o **Linux** dá ao disco. Duas fontes: o de dentro de cada imagem e o da sondagem. Havendo as duas, a sondagem ganha | mapa de discos, cache |
| **Colher** | Ler o que há no lugar do desfecho, julgá-lo pelo selo e dizer o que era. Encerra o job — **inclusive quando o que se encontra é nada**, que é uma resposta | verificar, conferir, finalizar |
| **Armar** | Gravar a receita no `grub.cfg` e marcar o boot único no firmware. **É o ponto sem volta** | agendar, disparar |
| **Desarmar** | Devolver o `grub.cfg` ao estado inerte e limpar a marca de boot único. Acontece **incondicionalmente** como primeiro passo de todo comando que arma | cancelar, limpar, resetar |
| **Boot único** | O `bootsequence` do `{fwbootmgr}`: a marca que manda o firmware bootar por uma entrada **no próximo reinício, uma vez só**. O firmware a consome ao usá-la | BootNext, boot temporário |
| **Ordem permanente** | O `displayorder` do `{fwbootmgr}`: por onde a máquina boota quando ninguém pediu nada. Ela **é** a `BootOrder` da NVRAM, e tem quatro donos — nenhum deles é o ARCA armando | BootOrder, boot padrão |
| **Estado inerte** | O `grub.cfg` sem `menuentry` do ARCA e com `set default="live-default"`, e o firmware sem `bootsequence`. Um dispositivo inerte boota no menu do Clonezilla e espera alguém | estado limpo, estado original |
| **Selo** | Identificador aleatório gerado ao armar, embutido na receita e devolvido pelo Clonezilla junto do desfecho. **É o que liga um desfecho ao job que o produziu** — o relógio do Clonezilla não serve para isso | id, timestamp |
| **Job fantasma** | Um desfecho encontrado no dispositivo que não pertence ao job pendente. Reconhecível porque o selo não bate | job órfão, estado sujo |
| **Desfecho** | Se a operação terminou ou não: `ARCA_BACKUP=OK`, `ARCA_RESTORE=FALHOU`… Escrito **em arquivo**, nunca em tela | resultado, status, saída |
| **Veredito** | O parecer do `ocs-chkimg` sobre a integridade de uma imagem: aprovada ou reprovada. **É independente do desfecho** | verificação, validação |
| **Conferência** | O que `arca verify` faz sem reiniciar: somar cada arquivo do `MD5SUMS` e comparar. Responde *"os bytes são os que o Clonezilla gravou?"* — e nunca *"esta imagem é restaurável?"* | verificação, checksum |

---

## Licença

MIT. Ver `Cargo.toml`.

---

<sub>As telas marcadas como **execução real** ou **captura** neste documento estão preservadas em `recursos/capturas/`, com procedência registrada. As demais são reprodução determinística do que o código monta — e **reprodução não é captura**, distinção que este projeto paga para manter.</sub>
