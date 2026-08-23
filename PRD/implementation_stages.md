# ARCA — Etapas de implementação

Plano de execução derivado do [PRD v5.1](PRD-ARCA-v5_1.md). O PRD diz **o que** o ARCA é; este documento diz **em que ordem** ele é construído e **como se sabe** que cada pedaço está pronto.

Vocabulário canônico em [CONTEXT.md](../CONTEXT.md).

## Progresso

| Etapa | O que entrega | Fase | Status | Concluída em |
|---|---|---|---|---|
| E0 | Fundação executável | I | ✅ | 2026-08-22 11:47 |
| E1 | Descoberta do dispositivo e das imagens | I | ✅ | 2026-08-22 13:42 |
| E2 | Leitura do firmware | I | ✅ | 2026-08-22 14:28 |
| E3 | Geração e validação da receita | II | ✅ | 2026-08-22 16:04 |
| E4 | Desarmar | II | ✅ | 2026-08-22 17:36 |
| E5 | Estado e selo | II | ✅ | 2026-08-22 18:37 |
| E6 | Pré-voo | III | ✅ | 2026-08-22 19:24 |
| E7 | Armar e disparar | III | ✅ | 2026-08-22 21:06 · marco cumprido |
| E8 | Colher o desfecho | III | ✅ | 2026-08-22 21:14 · marco cumprido |
| E9 | Restauração | IV | ✅ | 2026-08-23 11:50 · marco cumprido |
| E10 | `arca prepare` | IV | ⬜ | — · **C-13 entregue em 2026-08-23**, ver P-20 |
| E11 | `arca verify` | IV | ⬜ | — |

Uma etapa só é marcada ✅ quando o **Pronto quando** ou o **Entrega** da sua seção estiver cumprido de fato — não quando o código foi escrito. As três etapas com marco em hardware (E7, E8 e E9) exigem a execução real para fechar.

**🟨 era escrita, revisada e commitada, com o marco em hardware devendo.** O
estado nasceu na E7, e ele existia porque ✅ seria mentira e ⬜ seria pior: quem
lesse ⬜ suporia que não há código. Cada seção 🟨 terminava com **o que falta,
nomeado** — e o que faltava, nas três, custava um reinício que apagava a sessão
que o dispararia. A E4 já tinha entregue o critério dela "cumprido pela metade
verificável"; a diferença é que lá a outra metade era barata, e aqui ela era o
marco inteiro.

**Não há mais nenhum 🟨.** As três etapas com marco em hardware fecharam, e as
três em sessões à parte:

- **E7 e E8, em 22/08/2026.** O backup `2026-08-22_Apps` foi armado às
  20:53:48, a máquina bootou pelo dispositivo por boot único, gravou 39,7 GB,
  verificou, escreveu o desfecho às 21:06:02 e desligou; a colheita foi às
  21:14:49.
- **E9, em 23/08/2026** — a única cuja operação **apaga um disco**. A
  restauração daquela mesma imagem foi armada às 11:10:50, o `ocs-sr` encerrou
  às 11:31:55 do relógio do live, a máquina desligou, e a colheita foi às
  11:50:53. O Windows que colheu veio de dentro da imagem.

Os blocos **"o que faltava para o marco, e como cada coisa fechou"** das três
seções continuam lá, reescritos contra o que aconteceu — apagá-los perderia o
registro de o que estava em aberto e de como cada coisa fechou.

---

## Princípio de ordenação

**Risco crescente, e nunca antes da hora.** As etapas vão de leitura pura até a operação que apaga um disco, nesta ordem, e cada uma entrega algo executável de verdade.

Três regras que decidem a ordem toda:

1. **Só se arma o que já se sabe desarmar.** O desarmar (E4) vem antes do armar (E7). Um armar sem desarmar deixa a máquina com boot único pendente e nenhuma forma de cancelar.
2. **Só se dispara o que já se sabe colher.** O estado e o selo (E5) vêm antes do primeiro reinício.
3. **Restauração por último.** É a única operação que destrói dados. Ela só entra depois de backup e colheita rodarem ponta a ponta em hardware.

## Decisões fechadas antes de escrever código

| # | Decisão | Consequência |
|---|---|---|
| 1 | **O binário é portátil e roda de onde estiver** — dispositivo, `C:`, OneDrive. Sem instalador, sem shim, sem `PATH` | Nenhuma. Mas o **estado continua obrigatoriamente no `ARCABOOT`** (§4.1): o que muda de lugar é o executável, nunca o estado |
| 2 | **A receita continua sendo uma string no `grub.cfg`**, como no mecanismo já validado em hardware. Não vira arquivo `custom-ocs` | Nada a remedir. `toram` fica como está. C-2 valida a string; sem pipes, só `>` e `>>` |
| 3 | **Correlação por selo, nunca por data** | Fecha S-6, R-6 e o caso "não há `arca-fim.txt`" com um mecanismo só |
| 4 | **`arca verify` confere `MD5SUMS` no Windows**; `--completo` arma boot para `ocs-chkimg` | Verificação rápida sem reinício. Não substitui B-9, que continua obrigatória no backup |
| 5 | **Destino divergente é permitido**, com confirmação que nomeia o disco de destino. Recusa dura só se o destino for **menor** que a origem | ~~`-k0` num disco menor corrompe em vez de falhar.~~ **A premissa estava errada, e a E9 resolveu** (P-17, [ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md)): o help do `ocs-sr` diz que o Clonezilla confere o tamanho do destino **por padrão** e **desiste** se for menor, e que `-icds` é quem desliga isso. A recusa do ARCA fica, e a razão passa a ser **onde** ela acontece: a do Clonezilla custa um reinício de uma operação destrutiva. E resolver isso obrigou a descobrir a armadilha da régua — o `MSFT_Disk` e o `Win32_DiskDrive` dão dois tamanhos para o mesmo disco, e com o segundo o disco não cabe em si mesmo. Destino divergente ganhou `--destino <indice>`; sem ele a metade permissiva de R-7 seria inalcançável. Em disco novo, `-iefi` não acha entrada correspondente e o `bcdboot` volta a ser necessário |
| 6 | **Clonezilla com versão fixada e SHA256 embutido no binário do ARCA**, nunca baixado | Cópia do pacote usado fica no `ARCAVAULT`. `--iso <caminho>` para instalação offline |
| 7 | **`--dry-run` é flag de primeira classe** em todo comando que arma | A armadilha registrada no PRD (`--dry-run` virou execução real) é exatamente o que C-7 previne. Os dois andam juntos, na E0 |

## Correções a aplicar no PRD

Aplicar **antes** da E3, que transcreve as receitas para código. **Todas aplicadas em 22/08/2026**; a coluna diz em que etapa cada uma entrou.

| # | Correção | Aplicada |
|---|---|---|
| D1 | `-batch` aparece na fundação §3.2 mas some de B-8 e §10.1. **Adotado: `-batch` nas duas receitas**, alinhando à fundação medida. Confirmar na primeira execução real pelo ARCA | ✅ E3 — e **confirmado**: rodou nas três receitas preservadas. P-15 fechada |
| D2 | §10.2 usa `$LOG` e `$NOME` sem definir. Fixar `LOG="/home/partimag/ARCA-LOGS/$NOME"`, igual à de backup — o `ARCAVAULT` sobrevive à restauração do `nvme0n1` | ✅ E3 — inclusive o log do Clonezilla, que a captura mandava para `/home/partimag/restore.log`, um caminho fixo que a restauração seguinte sobrescreveria |
| D3 | O "princípio P1" é citado em §2 e §7.1 e **nunca enunciado**. Escrever: o ARCA não executa a operação mais destrutiva do fluxo | ✅ §7.1 |
| D4 | Job fantasma e R-6 descrevem uma ameaça que §4.1 já elimina. Reescrever como **risco herdado**: só imagens feitas antes de o ARCA sair do `C:` carregam estado dentro de si. O selo cobre de qualquer forma | ✅ §11 e R-6 |
| D5 | S-1 conflita com B-5 e B-6, que escrevem no disco de origem. Delimitar S-1 a **acesso raw ao dispositivo** | ✅ S-1 |
| D6 | `arca list` e `arca verify` não têm requisito nenhum. Ganham requisitos nas E1 e E11 | ✅ §9.5 |
| D7 | "Um dispositivo por vez" é regra sem ID. Vira requisito: **recusar se houver mais de um `ARCAVAULT` ou `ARCABOOT` conectado** | ✅ C-10 |
| D8 | Não existe requisito para `arca-fim.txt` ausente — o desfecho de toda falha silenciosa. Vira tabela de estados terminais na E8 | ✅ §5.5 e C-12 |
| D9 | Cabeçalho diz "Versão 0.5", título diz "v5", arquivo diz `v5_1`. Escolher uma | ✅ v5.1 em toda parte |
| D10 | §3.1 leva a crer que `Removable Media` e `External hard disk media` saem do `bcdedit`. **Não saem.** Procuradas no `bcdedit.exe` e nos seus recursos `pt-BR` e `en-US`: não estão lá. São valores de `MediaType` do WMI (`Win32_DiskDrive`, em `cimwin32.dll`). Reescrever C-6 pelo que é verificável: a rejeição silenciosa aparece como um `device` que **não mudou** depois da escrita, e quem a revela é a releitura de C-3. O `GetDriveType` dá o sinal antecipado, antes de qualquer tentativa | ✅ E3 — §3.1 e C-6 |

Nasceram na E3, contra as receitas preservadas em `recursos/capturas/`:

| # | Correção | Aplicada |
|---|---|---|
| D11 | §10.1 e §10.2 mostram um `#!/bin/bash` de várias linhas. **A receita nunca foi um script**: é uma string única em `ocs_live_run="bash -c '...'"`, como o ADR-0002 decidiu e as três capturas comprovam | ✅ §10 inteiro reescrito |
| D12 | B-8 pede `-scs` e não pede `-p true`. O hardware rodou o contrário, e o help explica os dois: `-scs` pula a conferência nativa (oposto de B-9) e o padrão de `-p` é `reboot` (sem `-p true`, o `ocs-chkimg` nunca rodaria) | ✅ B-8 e §3.2 |
| D13 | R-4 não lista `-e1 auto -e2`, que a restauração validada usou, e §10.2 não explica por que `-p true` em vez do `-p poweroff` que rodou | ✅ R-4 |
| D14 | O PRD trata S-4, C-11, C-12, R-5 e R-6 como fundação validada. **Nenhum deles rodou**: nenhuma receita real escreve `arca-fim.txt`. O `arca-fim.txt` do dispositivo veio do trabalho de validação, como o `ARCA_VEREDITO=` do ADR-0003 | ✅ §3.5 (P-16), S-4, R-5, §11 |

---

## Fase I — Leitura pura (nada é escrito)

### E0 · Fundação executável

Esqueleto em Rust com `clap`, manifesto `requireAdministrator` e reelevação por UAC **repassando os argumentos** (C-7), escape com barra invertida e não crase (C-8), `--dry-run` global, e log local do lado Windows.

As três fronteiras perigosas ficam atrás de portas desde o primeiro dia — firmware (`bcdedit`), enumeração de discos, sistema de arquivos — para que parser, validador e regra de espaço tenham teste sem hardware. S-1 vira propriedade da arquitetura: nenhuma porta abre o disco de origem em modo raw.

**Cobre**: C-7, C-8, S-1
**Pronto quando**: `arca --version` roda, eleva sozinho e chega do outro lado com os argumentos intactos — inclusive `--dry-run`.

### E1 · Descoberta do dispositivo e das imagens

Localizar o dispositivo pelos labels `ARCABOOT` e `ARCAVAULT`, nunca por letra ou número de série (B-1, S-3). Recusar mais de um dispositivo conectado (D7). Enumerar imagens: pasta, tamanho, presença de `MD5SUMS` — o que separa imagem de resíduo (B-3) — e veredito lido do `arca-check.log`.

**Cobre**: B-1, B-3 (detecção), B-10, S-3, D6, D7
**Entrega**: `arca list` de verdade, com a saída de §5.4.
**Ainda não**: nada escreve. Nem log no dispositivo.

### E2 · Leitura do firmware

Parser de `bcdedit /enum` **por valor, não por nome de campo** — só `identificador` sai traduzido (fundação §3.1). Localizar a entrada `ARCA`; não havendo, reconhecer a legada `Clonezilla` (C-4). Recusar `Removable Media`, que o `bcdedit` rejeita em silêncio respondendo "êxito" (C-6) — e ver D10, porque essa palavra não sai do `bcdedit`.

Testes unitários sobre saídas capturadas em português e em inglês. Este parser é o único ponto do sistema onde uma leitura errada leva a máquina a bootar no lugar errado.

**Cobre**: C-3, C-4 (detecção), C-6
**Entrega**: `arca status` — diagnóstico não destrutivo: dispositivo, imagens, entrada de firmware, estado do job. Comando novo, a acrescentar em §8.

**Medido nesta etapa, e não previsto pelo plano:**

- **O `bcdedit` não escreve UTF-8.** Ele escreve na página de código do console de quem o chama — 850 na janela que o UAC abre nesta máquina, 65001 num terminal já em UTF-8. O adaptador da E0 fazia `from_utf8_lossy`, e perdia 6 caracteres por leitura, em silêncio. Corrigido em `adaptadores::windows::texto`; medido por `examples/codificacao_do_bcdedit.rs`.
- **A fixture em inglês não precisou ser fabricada.** O `bcdedit.exe` carrega as mensagens de `System32\<idioma>\bcdedit.exe.mui`, e esta máquina tem `en-US` instalado. Copiado o executável para uma pasta onde só existe o `.mui` inglês, a mesma consulta ao mesmo BCD sai em inglês — e o par pt/en descreve a mesma configuração, lida com segundos de diferença. É o que torna `o_idioma_nao_muda_nada_do_que_o_parser_extrai` uma prova em vez de uma suposição.
- **A entrada desta máquina foi renomeada de `Clonezilla` para `ARCA` entre 20/08 e 22/08**, mantendo o GUID. Os dois lados de C-4 estão capturados: o estado legado em `bcdedit-enum-firmware-legado-pt.txt`, o migrado nas outras duas.
- **Nenhuma captura tem `bootsequence`**, porque armar é a E7. O formato do boot único está coberto por caso construído, marcado como tal, para a E7 confirmar contra hardware.

## Fase II — A receita (escreve em arquivo, não arma nada)

### E3 · Geração e validação da receita

Montar as duas receitas exatamente como as validadas em hardware, com `-batch` (D1) e o `LOG` da restauração corrigido (D2). Backup com nome e disco embutidos, **sem `ask_user`** (B-7), flags fixas de B-8, chamada explícita ao `ocs-chkimg` com saída redirecionada (B-9). Restauração com `-k0 -iefi -j2`, sem `-g auto` (R-4), e `if/then/else` — nunca `;`, que faria uma falha deixar o mesmo rastro de um sucesso (R-5).

Validador C-2 como porteiro: rejeita pipes, aspas desbalanceadas e nomes inseguros (B-2) **antes** de qualquer gravação.

Os testes desta etapa comparam a receita gerada, caractere a caractere, com a que rodou no hardware. É o ponto de verificação mais importante do projeto: daqui para frente tudo confia que esta string está certa.

**Cobre**: C-2, B-2, B-7, B-8, B-9, R-4, R-5, S-4 (a receita é quem grava o desfecho)
**Entrega**: `arca backup <nome> --dry-run` imprime a receita completa e não toca em nada.

**Executado de verdade em 22/08/2026, com o dispositivo conectado:** `arca backup 2026-08-22_Apps --dry-run` imprime as duas receitas inteiras — a de backup e a de restauração — nas duas formas, o comando e a linha do `grub.cfg`. O `grub.cfg` do dispositivo saiu com o **mesmo SHA256** de antes da execução, e nenhum `estado.json`, pasta de imagem ou `ARCA-LOGS/backup-*` foi criado. As oito recusas de B-2 foram exercitadas **pela linha de comando real**, atravessando a elevação por UAC: espaço, acento (o `ô` chegou intacto do outro lado, o que confirma C-7 e C-8 de novo), `;`, nome começando com `-`, nome reservado do Windows, pasta de serviço do dispositivo, e nome acima de 48 caracteres — cada uma com a sua mensagem. Sem `--dry-run`, o comando continua dizendo que armar é a E7.

**Medido nesta etapa, e não previsto pelo plano:**

- **A receita do §10 do PRD nunca rodou.** As três que rodaram estão preservadas em `recursos/capturas/`, copiadas do dispositivo: `grub-backup-arca-teste-02.cfg`, `grub-backup-arca-teste-03.cfg` e `grub-restauracao-arca-teste-02.cfg`. Nenhuma é um script: as três são uma string única em `ocs_live_run="bash -c '...'"`, como o ADR-0002 já dizia e o §10 contradizia na forma. Reescritos §10.1, §10.2 e mais quatro seções do PRD contra elas.
- **O "caractere a caractere" não tinha original inteiro.** Metade da receita — o `arca-fim.txt`, o selo, o `ARCA_FIM`, o `if/then/else`, o `ARCA_VEREDITO=`, o `sleep 20` — **nunca existiu em execução nenhuma**. O `arca-fim.txt` do dispositivo veio de trabalho manual de validação, o mesmo padrão que o ADR-0003 já tinha achado no `ARCA_VEREDITO=`. É o segundo caso do mesmo tipo, e virou P-16 no PRD. O que é transcrição e o que é código novo está marcado em `src/receita.rs`, com teste cobrando que nenhuma captura contenha `arca-fim.txt`, `ARCA_SELO` ou `if `.
- **As flags de B-8 estavam erradas em três pontos.** Rodou `-q2 -j2 -z9p -i 4096 -gm -sfsck -senc -batch -p true`. Com o help do `ocs-sr` desta versão na mão — capturado pela própria receita de `ARCA-TESTE-03` —, `-scs` fica **fora** (é `--skip-check-restorable`, o oposto de B-9) e `-p true` fica **dentro** (o padrão de `-p` é `reboot`, e sem ele o `ocs-chkimg` de B-9 nunca rodaria). Ver ADR-0004.
- **A restauração rodou com `-e1 auto -e2`, que R-4 não listava.** Ficam: são inócuos no mesmo disco e são o que faz a partição de boot NTFS bater com a geometria de outro. O `-p poweroff` dela vira `-p true` — com a máquina desligando dentro do `ocs-sr`, o desfecho de R-5 nunca seria escrito.
- **P-15 fechada com evidência.** `-batch` rodou, nas três.
- **A pendência do `ARCA_VEREDITO=` era desta etapa, e foi decidida: a receita passa a escrevê-lo.** É o marcador que o leitor da E1 prefere, e escrevê-lo tira o veredito da dependência de interpretar frases em inglês do `ocs-chkimg`.
- **B-9 mora dentro do ramo de êxito do `savedisk`.** Com o backup falhando, a pasta da imagem pode nem existir, e o redirecionamento do `ocs-chkimg` falharia junto do `else` dele.
- **C-2 recusa toda aspa, e não aspa desbalanceada.** Um par balanceado de aspas simples dentro do `bash -c '...'` fecha a string do `bash` e abre outra: o resultado é sintaticamente válido e semanticamente outro. Contar aspas daria só a impressão de estar conferindo. B-2 é lista de permissão (`A-Z a-z 0-9 . _ -`) pelo mesmo motivo.
- **A receita foi executada num `bash` de verdade**, com o Clonezilla substituído por comandos falsos: `recursos/ensaio-da-receita.sh`. Os cinco desfechos deixam o rastro certo, inclusive o que importa mais — com o `savedisk` falhando, o `ocs-chkimg` não é chamado e não há `arca-check.log`. Um teste em `src/receita.rs` cobra que o script não fique para trás da receita. Não substitui o marco em hardware, mas tira do caminho o modo de falha mais provável do código novo: um `fi` no lugar errado escrevendo `OK` sobre uma falha.
- **Achado fora do escopo, anotado como P-17:** o help diz que o Clonezilla confere o tamanho do disco de destino **por padrão** e desiste se for menor que a origem — `-icds` é quem desliga isso. A decisão 5 abaixo e R-7 partem da premissa contrária. A receita não usa `-icds`, e há teste cobrando. É da E9.

**O que a revisão pegou, e que os testes não pegariam:**

- **O `ARCA_VEREDITO=APROVADA` podia inverter uma reprovação.** Enquanto o marcador só existia porque alguém o escrevera depois de olhar o log, a ordem de leitura da E1 estava certa. Com a receita passando a escrevê-lo a partir do código de saída do `ocs-chkimg`, deixou de estar: um `ocs-chkimg` que saísse zero com `NOT restorable` no texto deixaria as duas marcas, e o marcador venceria. **Uma melhoria criando o defeito.** A ordem agora é toda forma de reprovar antes de toda forma de aprovar.
- **B-2 aceitava `ARCA-LOGS` como nome de imagem** — a imagem seria gravada por cima da pasta de logs e sumiria da listagem, porque `imagens::enumerar` pula esse nome. Invisível no `arca list` e invisível para o pré-voo de B-3.
- **O backup e a restauração da mesma imagem dividiam o `arca-fim.txt`.** Toda receita começa truncando o arquivo com `>`; um `arca restore X` antes de o backup de X ser colhido apagaria o desfecho dele. O selo não cobre — ele julga um desfecho encontrado, não um que foi por cima. O log passa a levar a operação no nome.
- **`COM0` e `LPT0` faltavam** na lista de reservados do Windows.
- **O nome podia estourar o `COMMAND_LINE_SIZE` do kernel** (2048 no x86_64), que trunca em silêncio — e receita truncada é o caso do §3.2. Orçamento agora explícito (§10.2.3 do PRD), recusa própria sobre a linha pronta, e o limite do nome baixou de 64 para 48.

Os três primeiros são o mesmo padrão, e é o padrão desta etapa inteira: **uma peça nova encaixada numa peça antiga que ninguém releu ao encaixar.**

**O que isto muda nas etapas seguintes:** a E7 e a E9 deixam de ser confirmações de um mecanismo pronto. O marco em hardware da E7 estreia, de uma vez, o `arca-fim.txt`, o selo dentro da receita, o `ARCA_FIM` e o `if/then/else`.

### E4 · Desarmar

Reescrever o `grub.cfg` para o estado inerte — o menu normal do Clonezilla, que é o que §6.3 pressupõe existir quando o Windows não sobe — e limpar qualquer marca de boot único residual. Incondicional, idempotente, **sem consultar estado nenhum** (C-1), e é o primeiro passo de todo comando.

**Cobre**: C-1
**Pronto quando**: rodar duas vezes seguidas dá o mesmo resultado, e o dispositivo boota no menu normal do Clonezilla depois.

**Executado de verdade em 22/08/2026, com o dispositivo conectado**, em quatro cenários, com o `grub.cfg` salvaguardado fora do dispositivo antes da primeira escrita. Todos saíram com código 0, e todos terminaram no `grub.cfg` inerte — SHA256 `4B33DA61…F947AA3D`, byte a byte:

| # | Estado de partida | O que o comando fez |
|---|---|---|
| A | o inerte, **duas vezes seguidas** | as duas saídas são idênticas linha a linha, e o SHA256 não mudou nenhuma vez (C-1) |
| A3 | o inerte, com `--dry-run` | não escreveu nada, no `grub.cfg` nem no firmware |
| B | `grub.cfg.teste01`, uma cópia armada do próprio dispositivo | tirou o bloco do ARCA; voltou ao inerte |
| C | `grub.cfg.original`, o que o **Clonezilla entrega**, com `set default="0"` | devolveu o `set default`; voltou ao inerte |
| D | `grub-backup-arca-teste-03.cfg`, armada por inteiro | desfez **as duas** mudanças; voltou ao inerte |

Nenhum `.arca-tmp` ficou para trás no diretório de que o `grub` lê. Os cenários foram escolhidos para que nenhum deles deixasse o dispositivo capaz de bootar desatendido se fosse interrompido no meio — só o D põe `set default="arca-backup"` no disco, por segundos, e não há boot único armado no firmware que fizesse a máquina chegar nele sozinha.

**Um defeito de saída só apareceu na execução real.** No cenário C — `set default="0"`, sem `menuentry` do ARCA nenhum — o comando dizia *"Havia receita armada"*. Não havia: havia um `set default` que **armaria sozinho** na próxima inserção, que é outro problema. Quem lesse aquilo acharia que a máquina estava a um reinício de rodar um backup. As duas coisas passaram a ser nomeadas separadamente, e há teste para cada uma.

**O critério de aceite foi cumprido pela metade verificável, e isso está dito de propósito.** "Rodar duas vezes seguidas dá o mesmo resultado" foi executado. "O dispositivo boota no menu normal do Clonezilla depois" **não foi observado**: custaria um reinício, e o que se pode afirmar sem ele é mais forte do que parece — o `grub.cfg` reescrito sai byte a byte igual ao que está no dispositivo hoje, que é o arquivo com que a máquina bootou todas as vezes desde 21/08. O boot fica confirmado no marco da E7, que reinicia de qualquer forma.

**Medido nesta etapa, e não previsto pelo plano:**

- **É o `set default` que faz o boot ser desatendido, e ele não estava documentado em lugar nenhum.** Passou três etapas sem ninguém perceber que existia. O `grub.cfg` inerte e a captura `grub-backup-arca-teste-03.cfg` diferem em **exatamente duas coisas**: `set default="live-default"` vira `set default="arca-backup"`, e um `menuentry --id arca-backup` de quatro linhas entra antes do `live-default`. **Inserir o bloco não arma nada** — a máquina espera os trinta segundos do `timeout` e boota no Clonezilla normal. Aplicado à §3.2 do PRD.
- **`live-default` e nunca `0`.** O `grub.cfg` que o Clonezilla entrega traz `set default="0"`, e difere do inerte deste dispositivo **só nisso**. `"0"` aponta por **posição**, e o bloco do ARCA entra antes do `live-default`: com `"0"`, inserir o bloco arma sozinho. Um dispositivo assim não está inerte, está parecendo inerte. O desarmar devolve o `set default` para `live-default` qualquer que seja o valor que encontrou — e a prova de que essa é a regra certa é que desarmar o `grub.cfg.original` do Clonezilla produz o inerte de hoje, byte a byte.
- **O PRD nunca definia o que é o estado inerte.** O §6.3 contava com ele, o §5.2 e o §5.4 mostravam `Desarmando ... ok` sem dizer o que é desarmar. Definido no §4.4, e definido de forma **verificável sem reiniciar** — o que é o que permite a etapa fechar sem marco em hardware.
- **`bcdedit /deletevalue {fwbootmgr} bootsequence` chama de erro não ter o que apagar.** Medido: sem `bootsequence`, ele sai com **código 1** e "Elemento não encontrado", e o `/enum` antes e depois é idêntico — não muda nada. O adaptador da E0 converte código ≠ 0 em erro, e com razão, porque é assim que "Acesso negado" chega. Um desarmar ingênuo **falharia justamente no caso normal**, e a segunda das duas passadas que C-1 exige nunca passaria. A saída não é ler o texto da recusa, que é frase em dois idiomas: é descartar o que o `bcdedit` responde e conferir com `/enum` (C-3).
- **C-1 e C-3 não brigam, e aqui C-3 é o que torna C-1 possível.** C-1 proíbe consultar estado *antes de decidir*; C-3 exige conferir *depois de escrever*. Como o código de saída do `bcdedit` é inútil exatamente no caso idempotente, a releitura é a única prova que existe.
- **A escrita atômica nunca tinha rodado em produção, e o `ARCABOOT` é FAT32.** Medido antes de a primeira escrita acontecer, em `examples/escrita_atomica_no_fat32.rs`, e com uma cópia do `grub.cfg` guardada fora do dispositivo: renomear por cima de arquivo existente funciona, o `sync_all` funciona, o LF é preservado, o nome longo `grub.cfg.arca-tmp` é aceito e nenhum temporário fica para trás. A conclusão **não** é que a escrita virou transacional em FAT32 — é que a sequência funciona e não deixa resto. A janela continua existindo, e é por isso que o desarmar grava o estado seguro: interrompido no meio, o dispositivo continua com o que tinha.
- **A "diferença de duas formas de inserção" entre as capturas não existe.** As quatro cópias armadas põem o bloco na mesma posição, linhas 93–97. O `diff` ancora umas depois da linha 91 e outras depois da 92 porque desambigua linhas em branco repetidas de jeitos diferentes. O desarmar tolera variação assim mesmo — nada garante que as próximas sejam idênticas —, mas a justificativa é precaução, e não observação.
- **Os blocos do ARCA não são iguais entre si.** A `teste-02` preserva o `hostname=cl-3.3.3-15` e as blacklists de driver do `menuentry` base; a `teste-03` perdeu os dois. Não há forma canônica transcrita, e é por isso que `grub::armar` **recebe** o bloco pronto em vez de montá-lo: escolher entre eles é decidir que linha de comando o kernel recebe, e é da E7.
- **Uma quarta cópia armada existia e não estava capturada**: `R:\boot\grub\grub.cfg.teste01`, de 19/08. Preservada agora, junto do inerte e do original do Clonezilla.

**O que a revisão pegou, e que os testes não pegavam:**

- **O desarmar podia engolir o `menuentry` seguinte, e o teste que existia para isso não pegava.** `achar_bloco` terminava o bloco na primeira linha `}` adiante, sem conferir se outro `menuentry` aparecia antes dela. O teste que eu tinha escrito construía um caso **sem `}` nenhum** até o fim do arquivo — e num `grub.cfg` de verdade sempre há um `}` adiante, o do próximo `menuentry`. Medido antes da correção, com um bloco do ARCA sem fechamento: o arquivo saiu **reduzido a uma linha**, com o `menuentry --id live-default` removido junto e o `set default` apontando para uma entrada que acabou de sumir — e esse arquivo iria para o dispositivo. Agora achar um abridor de bloco antes do fechamento é o mesmo que não achar fechamento: recusa, e nada é gravado. Uma segunda defesa cobra a pós-condição — tendo tirado bloco, o alvo do `set default` tem de existir no resultado.
- **A releitura de C-3 tratava "não entendi a resposta" como "a marca sumiu".** `firmware::ler` nunca falha por desenho: texto irreconhecível vira leitura vazia, e leitura vazia tem `boot_unico` vazio — indistinguível de estar inerte. Um `bcdedit` que saísse zero com a saída noutro formato faria o ARCA dizer "não havia" com o boot único ainda armado, e o próximo reinício rodaria a receita velha. Pior: a conferência de C-5 logo abaixo compararia duas listas vazias e passaria junto. `Leitura` passou a dizer se viu o `{fwbootmgr}`, e o desarmar falha alto quando não viu. **É o mesmo padrão do ADR-0004**: uma peça nova (a releitura) encaixada numa peça antiga (um parser que, para exibir, faz certo em não falhar).
- **A remoção da linha em branco adjacente podia apagar uma que o ARCA não pôs.** O ramo "senão remove a de antes" existia por causa das "duas formas de inserção" do briefing — que se revelaram artefato do `diff`. Ele saiu: agora só sai a linha em branco **de depois**, que é a que `armar` insere. Uma linha em branco a mais é inofensiva; colar duas entradas do Clonezilla uma na outra contradiria o que o módulo promete.
- **Faltava `#[cfg(windows)]` no exemplo da medição de FAT32**, e sem ele o `cargo check --all-targets` quebraria fora do Windows — uma configuração que `src/main.rs` diz explicitamente querer manter compilando.

Os dois primeiros são o mesmo padrão de sempre, e o primeiro tem um agravante que vale registrar: **eu tinha escrito um teste para exatamente aquele perigo, e ele passava.** O caso que construí era mais fácil do que o real — sem `}` nenhum, em vez de com o `}` errado logo adiante. Um teste que exercita o caso fácil de um perigo dá a impressão de cobri-lo.

**Decidido nesta etapa:**

- **O estado inerte se reconstrói do `grub.cfg` corrente** — não vem de cópia embutida no binário nem guardada no dispositivo. Idempotência de graça, funciona num dispositivo que o ARCA nunca viu, e não prende o ARCA a uma versão do Clonezilla. Os dois caminhos descartados e o que a reconstrução custa estão no [ADR-0005](../docs/adr/0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md).
- **`src/grub.rs` fica com as duas metades, e a E4 usa uma.** A função de armar é pura, não escreve em disco nem toca no firmware, e o ponto sem volta continua na E7 — a regra "só se arma o que já se sabe desarmar" não é furada. Ela existe agora por causa de um teste que só é possível com as duas juntas: tira-se o bloco de uma cópia armada, desarma-se, arma-se de volta, e o resultado tem de ser a cópia byte a byte. Com só o desarmar, a etapa testaria contra um alvo que ela mesma inventou.
- **`arca desarmar` vira comando**, acrescentado à §8. Desarmar continua sendo o primeiro passo de todo comando que arma; o comando existe porque o §5.5 descreve um caso que não tinha resposta — "o boot não aconteceu", depois do qual o dispositivo continua armado e não havia nada a rodar. E é a única forma de exercitar a idempotência de C-1 sem armar.
- **A linha do §5.2 leva o caminho**: `Desarmando receita anterior ..... ok · R:\boot\grub\grub.cfg`, com o caminho na coluna do **valor** — no rótulo ele estouraria a coluna 33 e desalinharia esta linha das que vêm depois dela. É a defesa barata contra desarmar o dispositivo errado enquanto `discos_fisicos()` não existir: com dois dispositivos na mesa, a letra errada aparece na tela. A pendência de fundo fica para a E6, como decidido.
- **Apagar o `bootsequence` não viola B-10.** B-10 fala de imagem, resíduo e log — do que o usuário perderia. A marca de boot único é uma intenção que o próprio ARCA gravou. `tests/b10_nada_e_apagado.rs` varre o código atrás de exclusão de *arquivo* e não distingue os dois casos, e por isso está escrito em `src/desarme.rs`, onde alguém vá procurar.

**Aberto nesta etapa, e não resolvido aqui:**

- **P-18 — o boot único da §3.1 pode nunca ter sido disparado por boot único.** As capturas de NVRAM mostram `BootCurrent: 0001` e `Boot0001* ARCA`: a máquina bootou pela entrada de firmware do ARCA, confirmado. Isso é **indistinguível de alguém ter escolhido essa mesma entrada com F12**. Nenhuma captura tem `BootNext`, e a ausência não prova nada — o firmware o consome ao usá-lo, e as capturas foram feitas de dentro do Clonezilla. É o terceiro candidato ao padrão de P-16. Fecha na E7.
- **Por que três das quatro cópias armadas não têm o `set default`** apontando para o ARCA. Fechada por falta de evidência, com as três vias nomeadas no ADR-0005 para o próximo não refazer o caminho: datas não (S-6 e ADR-0001), `BootNext` não, dedução não. **E não importa** — nas duas explicações possíveis o `set default` faz parte do que se arma, logo faz parte do que se desarma.
- **O `menuentry` que a E7 vai inserir de verdade.** A E4 entrega a função pura e a testa; escolher a forma do bloco é da E7.

### E5 · Estado e selo

`estado.json` no `ARCABOOT` — nunca no `C:`, que a restauração substitui (§4.1). Campos: selo, comando, nome, disco alvo, momento do armar (informativo, **nunca comparado com nada escrito pelo Linux**, S-6). Escrita atômica: arquivo temporário mais renomeação.

O selo entra na receita e volta dentro do `arca-fim.txt`. Na colheita, só é aceito o desfecho cujo selo case com o job pendente.

**Cobre**: R-6, S-6
**Pronto quando**: um `arca-fim.txt` com selo divergente é rejeitado como job fantasma, com mensagem própria.

**Executado de verdade em 22/08/2026, com o dispositivo conectado.** O critério
de aceite está cumprido nos dois níveis em que ele existe: `desfecho::julgar`
devolve `JobFantasma` nomeando o selo encontrado, e `arca status` o imprime na
tela ao lado do selo do job pendente. O `grub.cfg` saiu com o **mesmo SHA256**
de antes (`4B33DA61…F947AA3D`) — esta etapa não arma nada.

**Medido nesta etapa, e não previsto pelo plano:**

- **O único `arca-fim.txt` do dispositivo não tem linha de selo.** Vinte e
  cinco bytes, duas linhas: `ARCA_RESTORE=OK` e `ARCA_FIM`. É P-16 pela
  terceira vez — ele veio do trabalho manual de validação. E a tabela do §5.5
  **não tinha linha para ele**: tem "selo não bate" e tem "sem `ARCA_FIM`", e
  este arquivo não é nem um nem outro. Dizer "o selo não bate" seria mentira,
  porque não há selo a bater. Linha nova aplicada ao §5.5.
- **Aquele arquivo é inalcançável, e a linha vale código assim mesmo.** A E3
  decidiu que a pasta do log leva a operação no nome, e ele está em
  `ARCA-LOGS\2026-08-21_WindowsCompleto\` — o ARCA de hoje nunca olha para lá.
  Mas **"sem selo" é alcançável por outro caminho**, e ele foi medido em bash:
  toda receita começa com `echo ARCA_SELO=... > arca-fim.txt`, e o `>` **trunca
  ao abrir**, antes de o `echo` rodar. Um redirecionamento que abre e não
  escreve deixa o arquivo em zero byte. Um desligamento nessa janela produz
  exatamente o caso — com o selo sendo justamente o que foi cortado. Sem a
  linha, esse arquivo cairia no ramo que o código tomasse por descuido, e o
  ramo natural produziria a mensagem que não pode ser dada.
- **`R:\arca\` não existia**, e `criar_diretorio` nunca tinha rodado em
  produção. Medido em `examples/estado_no_arcaboot.rs`, contra o FAT32 real:
  cria, é idempotente na segunda passada, os cinco campos voltam byte a byte,
  nenhum `.arca-tmp` fica para trás, e um arquivo cortado ao meio é recusado —
  no disco, e não só na memória. A pasta fica; é onde o estado vai morar.
- **`BCryptGenRandom` já estava no `windows-sys`**, atrás da feature
  `Win32_Security_Cryptography`, desligada desde a E0. Nenhum crate novo. O
  `Cargo.lock` não mudou.
- **`ConvertTo-Json` do PowerShell 5.1 não escapa não-ASCII.** Medido, e é
  achado para a E6: a saída sai com bytes crus na página de código do console.
  A esperança de que o JSON fosse ASCII puro por construção estava errada, e
  `de_pagina_de_codigo` continua obrigatório.

**Decidido nesta etapa:**

- **O selo vem de `BCryptGenRandom`, e o `estado.json` é escrito à mão** — sem
  `rand` e sem `serde`. As três dependências do projeto continuam três. O
  raciocínio inteiro, incluindo por que o relógio ficou de fora (colide, e não
  por S-6) e por que escrever JSON à mão é seguro aqui (os cinco alfabetos não
  alcançam nada que precise de escape), está no
  [ADR-0006](../docs/adr/0006-o-selo-e-o-estado-sem-dependencia-nova.md).
- **`Entropia` é uma quarta porta**, e o `src/portas/mod.rs` dizia "são três".
  Ela entra pelo mesmo motivo das outras: sem duplo, nenhum teste sobre o
  `estado.json` saberia que selo esperar.
- **`Entropia` não entrou no `Contexto`.** Nada em produção gera selo nesta
  etapa — o selo nasce ao armar, e armar é a E7. Um campo que nenhum comando lê
  seria peso morto. O que fecha o buraco que a E4 nomeou — *o primeiro uso real
  de uma porta é onde as surpresas moram* — é a medição contra o dispositivo.
- **`MomentoDoArmar` guarda texto, e não um `DateTime`.** O plano pedia o campo
  "informativo, nunca comparado"; isso já era um comentário em
  `src/portas/relogio.rs` e comentário não impede nada — a trava que reprovou um
  backup perfeito neste projeto tinha o comentário do lado. Guardando texto não
  há o que subtrair nem o que comparar, e violar S-6 exigiria parsear a string
  de volta de propósito, num `let` que apareceria no diff.
- **`tests/s6_o_tempo_nao_decide.rs`**, na forma dos testes de arquitetura de
  S-1 e B-10. Cobra que `MomentoDoArmar` não derive ordenação, que nada em
  `src/estado.rs` devolva um tempo comparável, e — a metade que vale mais —
  que **`src/desfecho.rs` não mencione tempo em forma nenhuma**. Quem julga a
  quem um desfecho pertence não alcança o relógio.
- **O leitor do `estado.json` recusa em vez de adivinhar**: escape, chave
  desconhecida, chave repetida, chave faltando e texto depois do `}`. Chave
  desconhecida ser recusa é deliberado — agir sobre metade de um estado que
  arma uma operação destrutiva é pior do que recusar o arquivo inteiro.
- **`arca status` passa a ler o conteúdo do `estado.json`**, e não só a
  perguntar se ele existe. Mostra selo, comando, nome, disco alvo, momento e o
  que há no lugar do desfecho, já julgado pelo selo. Um `estado.json` presente e
  ilegível aparece como **ilegível**, nunca como ausência: um dispositivo com
  job armado e estado corrompido continua armado.

**O que os testes pegaram, e que eu não pegaria lendo:**

- **O teste do arquivo truncado achou uma borda no primeiro `cargo test`.** Em
  vez de escolher um ponto de corte, ele corta o `estado.json` em **todos** os
  comprimentos possíveis — e reprovou no corte 150 de 151, que tira só a quebra
  de linha final e deixa um objeto completo. O leitor aceitar isso é correto e
  deliberado (nada garante que quem gravou terminou com `\n`); o teste é que
  cobrava demais, e passou a cobrar até o fim do **conteúdo**. A borda apareceu
  porque o teste não escolheu o caso fácil — que é exatamente a lição da revisão
  da E4.

**O que a revisão pegou, e é o mesmo padrão de sempre:**

Os três achados têm **uma raiz só**, e ela é a peça antiga que ninguém releu ao
encaixar a nova: **`Arquivos::existe` devolve `bool`, e um `bool` não tem como
dizer "não sei".** `Path::exists` transforma qualquer falha de I/O em `false`.

- **A defesa contra "não consegui olhar" estava construída sobre a função que já
  confundia os dois casos.** Eu tinha acabado de separar `SemArquivo` de
  `NaoDeuParaLer` — o padrão que o ADR-0005 nomeou no firmware — e então
  perguntei `existe()` antes de ler. Um `arca-fim.txt` num volume com problema
  de leitura sairia como *"o boot não aconteceu"*, e `NaoDeuParaLer` ficava
  inalcançável por aquele caminho. **A correção que eu escrevi criou o defeito
  que ela vinha corrigir**, que é literalmente o achado da revisão da E3.
- **O mesmo no `estado.json`, com consequência pior**: um estado presente e
  não-estatável virava `EstadoDoJob::Nenhum`, e a tela dizia "não há job
  pendente" — a afirmação que o próprio comentário do enum diz que nunca pode
  sair de uma falha de leitura. Alguém reiniciaria achando que não há nada
  esperando.
- **`caminho_do_estado` falha por dois motivos, e o código dizia que era um.**
  O comentário afirmava "só há um motivo para não haver caminho: não há
  `ARCABOOT`". Falso: `Erro::VolumeSemLetra` também o produz, e nesse caso o
  `ARCABOOT` **está na mesa** e pode ter job armado. Dizer "sem ARCABOOT"
  mandaria alguém procurar um dispositivo que já está conectado.

A correção não foi a óbvia. Trocar `exists()` por `try_exists()` resolveria os
sintomas e deixaria de pé a pergunta que não deveria ser feita. **O código
deixou de perguntar "existe?" e passou a ler**, deixando o `ErrorKind` dizer
qual dos dois casos é (`Erro::e_arquivo_ausente`). É mais preciso, e não há
janela entre a pergunta e a leitura. `SemOndeOlhar` passou a carregar o motivo.

E os testes não pegavam nada disso porque **nenhum duplo sabia recusar uma
leitura**: `ArquivosEmMemoria` só sabe ter ou não ter o arquivo, e contra ele
o código errado passa sempre. Nasceu `ArquivosQueRecusam`, em que um caminho
existe e não se deixa ler — sem ele, os três testes novos não teriam como
falhar.

**E a correção foi confirmada no hardware, e não só nos testes.** Com uma ACL de
negação sobre o `arca-fim.txt` do `ARCAVAULT` — arquivo presente, leitura
recusada, ACL devolvida logo depois — o `arca status` disse *"está lá e não se
deixou ler … NÃO é o mesmo que o boot não ter acontecido"*, com o `os error 5`
junto. Antes da correção esse mesmo arquivo sairia como *"o boot não
aconteceu"*.

**Aberto nesta etapa, e não resolvido aqui:**

- **`arca status` e `arca desarmar` mostram um par que lê como contradição.**
  Depois de um `arca desarmar`, a tela diz "Boot único: não armado" ao lado de
  um job pendente — e está certa: desarmar não toca no `estado.json` (C-1), e
  quem encerra o job é a E8, ao colher. Está dito na tela e no código; quem
  fecha é a E8.
- **Nada em produção gera selo ainda.** `gerar_selo` existe, é testado e foi
  medido contra o hardware; quem o chama é a E7.

## Fase III — Backup ponta a ponta

### E6 · Pré-voo

Tudo que §5.2 mostra antes da confirmação: nome válido (B-2) e ainda não usado, inclusive contra resíduo (B-3); espaço pelo maior entre `maior imagem × 1,3` e `em uso × 0,45`, com faixa de aviso entre 1× e 1,5× disso (B-4); Inicialização Rápida, oferecendo `powercfg /h off` (B-5); `chkdsk /scan`, oferecendo agendar `/f` (B-6).

**Cobre**: B-2, B-3, B-4, B-5, B-6
**Entrega**: o diálogo de §5.2 inteiro, terminando **antes** de armar.

**Executado de verdade em 22/08/2026, com o dispositivo conectado.** `arca
backup 2026-08-22_Apps` imprime o diálogo do §5.2 inteiro e para antes da
confirmação. O `grub.cfg` saiu com o mesmo SHA256 e nenhum `estado.json` foi
criado — esta etapa não arma nada.

**O achado que mudou a etapa: o WMI resolve três coisas de uma vez.** Uma
consulta, sem elevação e sem abrir handle nenhum, e ela fecha três pendências
que o plano tratava como separadas:

1. **A pendência de `src/dispositivo.rs` fechou.** `ARCAVAULT` e `ARCABOOT`
   estão os dois no Disco #1, e agora há como provar. C-10 recusava rótulo
   **repetido** e não rótulo órfão; o pré-voo recusa o dispositivo partido.
2. **`MediaType` traz literalmente `External hard disk media`** — as palavras
   da §3.1 que o `bcdedit` não produz (D10). É o sinal antecipado de C-6, e é
   melhor que o `GetDriveType`, que classifica este mesmo SSD externo como
   disco **fixo** e não distingue nada.
3. **O tamanho e as letras por disco**, que é o que B-4 precisa.

**Medido nesta etapa, e não previsto pelo plano:**

- **O CLIXML do PowerShell vai para o stderr, e o stdout sai limpo.** Com
  `-EncodedCommand`, o PowerShell despeja 628 bytes de registros de progresso
  em CLIXML no stderr. Isso importa porque o adaptador do `bcdedit` **concatena
  stdout e stderr** — copiar aquele padrão colaria XML antes do JSON. O
  adaptador do WMI lê stdout e só, e a consulta começa com
  `$ProgressPreference='SilentlyContinue'`, que zera o stderr: medido, sai com
  zero byte. As duas coisas juntas, e não uma; a segunda é o que torna um
  stderr **não vazio** uma informação de verdade.
- **`ConvertTo-Json` do PowerShell 5.1 não escapa não-ASCII.** Medido: um valor
  acentuado sai com bytes crus na página de código do console. A esperança de
  que o JSON fosse ASCII por construção estava errada, e `de_pagina_de_codigo`
  continua obrigatório — o `Model` de um disco é texto livre do fabricante.
- **`powercfg /a` fala, sim, de Inicialização Rápida** — o briefing supunha que
  não. Ela aparece sob "estados de suspensão não disponíveis", com a frase
  *"Esta ação está desabilitada na política do sistema atual"*. Isso torna a
  leitura pior, e não melhor: é frase traduzida, e ela não separa "desativada
  pelo usuário" de "indisponível por outro motivo". O registro responde com um
  número, e número não tem idioma. `HiberbootEnabled = 0` nesta máquina.
- **`chkdsk C: /scan` elevado sai com código 0 em 16,3 s**, e o texto vem em
  **CP850 mesmo chamado de um console em UTF-8** — o mesmo caso do `bcdedit` da
  E2. Julgado pelo código de saída, nunca pelo texto.
- **O `498,7 GB` do §5.2 não era um número inventado: era `498.701.697.024`, o
  tamanho da partição `C:` em base 1000, apresentado como o do disco.** O disco
  tem `500.105.249.280` bytes — 465,8 GiB na base que `src/formato.rs` usa.
  Saber a origem importa mais do que o valor certo: quem repetir vai errar pelo
  mesmo caminho, e a diferença são as outras três partições.
- **O `Win32_DiskPartition` não enxerga a partição MSR.** O disco tem quatro
  partições pelo `Get-Partition` e três pelo WMI. Por isso `em_uso_bytes` é
  contado como `tamanho do disco menos o livre nos volumes com letra`, e não
  como a soma do que os volumes usam: assim o nome do campo passa a ser verdade
  e a conta não depende de o WMI ver toda partição.
- **As duas estimativas de B-4 caem a menos de 1% uma da outra** nesta máquina
  — 50,47 GB pela maior imagem, 50,84 GB pela compressão. É coincidência desta
  máquina, e não propriedade da regra, mas é um bom sinal sobre as duas.

**Decidido nesta etapa:**

- **O ARCA fala com o WMI por processo filho**, com `Get-CimInstance` pedindo
  JSON, e não por COM. O `Cargo.toml` tem `Win32_System_Com` desde a E0 e
  ninguém usou: COM seria centenas de linhas de `unsafe` sobre vtables cruas —
  o `windows-sys` não tem os auxiliares que o `windows` tem — para **uma**
  consulta. O terceiro caminho está fechado por S-1, e não por preferência.
- **O script vai por `-EncodedCommand`**, em UTF-16LE/base64 escrito à mão.
  Não há aspa a escapar nem linha a repartir, e C-8 deixa de ter o que morder.
- **O `DeviceID` do `Win32_DiskDrive` é descartado.** Ele é o caminho de
  dispositivo bruto do disco. Recebê-lo como dado não seria abrir o
  dispositivo — o teste de S-1 varre o **fonte**, não valores de runtime —, mas
  escrever a string no fonte para casar com ela faria o teste falhar, e falhar
  com razão. O `Index` responde tudo que ele responderia. O que não se pede não
  chega.
- **Uma porta nova, `Sistema`**, para o que não é firmware: B-5 e B-6.
  Pendurá-las na porta do firmware faria ela mentir sobre o que é; deixá-las
  num `Command::new` solto tiraria o teste sem hardware das duas.
- **B-5 lê o registro, e não o `powercfg`.** Valor ausente é `NaoSeSabe`, e
  nunca "desativada" — ausência de prova não vira o desfecho conveniente, o
  mesmo que o ADR-0003 decidiu para imagem sem veredito.
- **"Oferecer" é dizer o comando e o que ele custa, e não rodá-lo.** O §5.2
  mostra B-5 e B-6 como linha de status, sem pergunta, e o pré-voo termina
  antes da confirmação. A oferta de `powercfg /h off` diz que ele desliga a
  **hibernação inteira**, e não só a Inicialização Rápida: quem aceitasse
  perderia o "Hibernar" do menu Iniciar. Rodá-lo por conta própria seria o ARCA
  mexer em mais coisa do que anunciou.
- **O nome do disco vem do `blkdev.list` de uma imagem, e a derivação por
  índice foi recusada.** O WMI diz o modelo do disco onde o `C:` mora; o
  `blkdev.list` diz que nome o Linux dá àquele modelo. As duas pontas são
  medidas. `BusType NVMe + Index 0 → nvme0n1` é plausível e **não é medido** —
  o índice do Windows não é o do Linux por construção, e aqui coincide porque a
  máquina tem um NVMe só. Não havendo imagem de onde ler, o nome fica **por
  determinar**, e o pré-voo diz isso: é uma resposta, e a E7 herda. Aplicado ao
  PRD como §4.5.
- **O modelo é casado sem caixa e sem pontuação, tirando o sufixo
  `SCSI Disk Device`** que o Windows acrescenta a disco sem driver próprio.
  Medido: `KGSSE100 256 SCSI Disk Device` e `KGSSE100256` casam assim. Não
  casar é **recusa**, e nunca um palpite — um nome de disco errado numa receita
  destrutiva é o pior desfecho possível deste módulo.
- **A origem não é o disco 0 por suposição.** Ela é o disco que tem o `C:` e
  não é o dispositivo. Numa máquina em que o dispositivo ARCA fosse o disco 0,
  supor o índice faria a receita clonar o próprio disco de backup.

**O que a execução real pegou, e os testes não pegavam:**

- **A primeira linha do §5.2 mentia.** O pré-voo imprimia
  `Desarmando receita anterior ..... ok` e **não desarmava nada** — eu tinha
  tratado o desarmar como um passo do armar, que é a E7. C-1 não é condicional
  a chegar ao armar: desarmar é o primeiro passo de todo comando. Um
  dispositivo armado com receita velha sairia daqui com "pré-voo concluído,
  pronto para a E7" e continuaria armado. O comando passou a desarmar de
  verdade, reusando o que a E4 mediu, e a linha passou a distinguir "já estava
  inerte" de "havia receita armada". No `--dry-run` ela diz que **não**
  desarmou: um `ok` sobre ação que não aconteceu é a mesma mentira que o
  `--dry-run` deste projeto já contou uma vez (§11).
- **O teste de S-1 pegou um comentário meu.** O cabeçalho do adaptador do WMI
  explicava por que o caminho por `DeviceIoControl` está fechado — e soletrava
  o nome. A varredura é de texto e não distingue menção de uso, e **está
  certa**: o que torna essas varreduras confiáveis é serem burras demais para
  serem enganadas. Ensiná-la a ignorar comentários seria o primeiro passo para
  ela deixar passar o que importa. O comentário foi reescrito; o nome mora no
  próprio arquivo de teste.
- **Dois testes meus provavam nada.** O de nome inválido no `blkdev.list`
  montava uma tabela à mão com as colunas desalinhadas: a linha era descartada
  **antes** de chegar ao validador, e o teste passava pelo motivo errado.
  Reescrito sobre o arquivo de verdade, trocando só o nome, e com uma asserção
  a mais cobrando que a linha continue sendo lida. É a lição da revisão da E4
  outra vez — *o caso construído era mais fácil do que o real*.
- **Uma asserção de aritmética estava errada, e no lado que importa.** O teste
  de `proporcao` afirmava `(v / 1000) * 450`, que descarta o resto — exatamente
  o que a função existe para não fazer.

**O que a revisão pegou, e o mais grave é o espelho do anterior:**

- **A correção do desarmar criou o defeito seguinte.** Corrigida a linha que
  dizia `ok` sem desarmar, o desarmar passou a acontecer **antes** das recusas
  do pré-voo — e, com a recusa subindo como erro, nada era impresso. Quem
  rodasse `arca backup <nome-que-já-existe>` num dispositivo armado veria só
  *"já há uma imagem chamada…"*, e o job armado teria sumido em silêncio.
  Antes a saída mentia sobre uma ação que não aconteceu; agora a ação
  acontecia e a saída não contava. **É o mesmo eixo, invertido** — e é o
  padrão que o ADR-0004 nomeou: uma melhoria produzindo o defeito.
  Mover o desarmar para depois do julgamento fuaria C-1, que diz
  incondicionalmente; a saída foi partir o diálogo em duas metades e imprimir
  a primeira **antes** de julgar.
- **O nome do disco saía da coluna errada do `blkdev.list`.** `ler` calculava
  o deslocamento de `NAME`, avisava num comentário que confundi-lo com `KNAME`
  daria a coluna errada, e então **descartava o deslocamento** para pegar o
  primeiro campo da linha — que é o `KNAME`. Passava porque os dois coincidem
  para `sda` e `nvme0n1`. Num multipath (`NAME=mpatha`, `KNAME=dm-0`) o nome
  lido seria o do dispositivo de baixo.
- **O leitor de JSON do WMI não honrava `\"`, e falhava em silêncio.** `Model`
  é texto livre do fabricante, e `SAMSUNG 2.5" SSD` é plausível. Uma aspa
  escapada é um número **ímpar** de `"`: ignorar a barra inverte a paridade do
  resto do arquivo, o `}` de cada objeto passa a ser lido como dentro de texto,
  e **dois discos se fundem num só**. Se o que sumisse fosse o do dispositivo,
  as duas recusas que dependem de saber em que disco cada letra mora — C-6 e
  C-10 — passariam sem dizer nada.
- **Dois dos cinco campos do WMI adivinhavam** onde os outros três recusavam:
  `Model` ausente virava `""` e viajava até a linha `Origem:`; `Livre` ausente
  virava zero. O doc do módulo dizia "recusa o que não entende em vez de
  adivinhar". Os quatro obrigatórios passaram a ser obrigatórios.
- **`letra_do_sistema` era `'C'` fixo**, numa função que recebia dois
  parâmetros e os ignorava. É o mesmo erro que esta etapa combate em dois
  outros lugares — não supor que a origem é o disco 0, não derivar o nome
  Linux do índice — cometido no terceiro. Agora vem de `%SystemDrive%`.
- **`folga_em_milesimos` saiu.** Sem chamador, e com o doc prometendo saturar
  em zero enquanto o código devolvia `u64::MAX`.
- **E a revisão anotou um caso que não é achado e merece registro:** ela viu
  uma falha de teste que não reproduziu em dezoito execuções. Os `Registro::em`
  dos testes montam diretórios temporários por PID, e o Windows recicla PID.
  Conferido: `Registro::em` recebe **diretório** e `caminho()` é
  `<dir>/arca.log`, então o `remove_dir_all(parent)` do `Drop` apaga o
  diretório do próprio teste — e não `%TEMP%`, que era a suspeita ao ler o
  código de fora. Fica anotado como flakiness conhecida, fora deste diff.

**Aberto nesta etapa, e não resolvido aqui:**

- **O `nvme0n1` fica por determinar num dispositivo sem imagem.** É a herança
  explícita da E6 para a E7: quem armar o primeiro backup de um dispositivo
  novo não tem `blkdev.list` de onde ler o nome. A E7 decide o que fazer —
  pedir o nome, ou recusar. O que ela **não** pode fazer é derivar do índice,
  pelo motivo que o §4.5 do PRD registra. *(**Resolvido na E7: recusar.** Um
  nome do Linux digitado do lado Windows não tem contra o que ser conferido, e
  a receita que o nomeia é destrutiva na E9. A recusa acontece antes da
  confirmação digitada, e a saída diz que o primeiro backup de um dispositivo
  novo precisa ser feito uma vez pelo menu do Clonezilla.)*
- **`disco_de_origem` acha o disco que tem `%SystemDrive%` e não é o
  dispositivo.** Numa máquina com dois Windows a escolha ficaria ambígua e
  hoje o primeiro ganha; com nenhum, recusa alto. Não é caso deste projeto, e
  fica escrito para quem encontrar.
- **B-5 e B-6 relatam e não agem.** "Oferecer" foi implementado como dizer o
  comando e o que ele custa. Se o uso mostrar que a oferta devia ser
  interativa, ela cabe na E7 — que é onde já há confirmação digitada.

### E7 · Armar e disparar

Gravar a receita no `grub.cfg`, marcar o boot único **sem tocar na ordem permanente** (C-5), migrar a entrada de firmware (C-4), recusando `Removable Media` (C-6). Confirmação por texto digitado, nunca por `s` (S-2). Aviso de remover o SSD antes de religar, antes do reinício (C-9).

**Cobre**: C-4 (migração), C-5, C-9, S-2
**Marco em hardware**: primeiro backup completo disparado pelo ARCA, sem uma única tela.

**Escrita, testada e commitada em 22/08/2026, e o marco cumprido às 21:06 do
mesmo dia.** A escrita e o marco foram sessões diferentes de propósito: a
máquina desliga no reinício e a sessão morre junto. O que era verificável sem
reiniciar foi feito primeiro; o que só o hardware responde está no bloco do
fim, reescrito contra o que aconteceu.

**O achado da etapa, e ele é o quinto do mesmo tipo.** A ordem permanente de
boot desta máquina **foi alterada por alguém, pelo menos três vezes**. A tabela
das oito capturas de NVRAM está no §3.1 do PRD, com a data de cada leitura, e
ela desfaz o que parecia uma discordância entre o `bcdedit` e o `efibootmgr`:
as duas ferramentas concordam quando lidas no mesmo dia, e a ordem mudou entre
as leituras. O que sobra é pior do que a discordância:

- Em **20/08** a entrada do ARCA estava na ordem permanente, em segundo lugar.
- Em **21/08** — o backup que o §3.3 chama de validado — ela estava em
  **primeiro**. Uma ordem de boot com o dispositivo à frente explica o boot
  inteiro **sem passar por boot único**.
- Em **22/08** ela não está mais na ordem.

É P-18 com evidência apontando para o lado desconfortável, e é o quinto caso do
padrão que o método desta etapa nomeia: o que o documento chama de fundação
validada pode ter vindo de outra coisa. `tests/e7_armar_o_dispositivo.rs` cobra
que a entrada continue **fora** da ordem, porque é essa configuração que faz a
medição do boot único significar alguma coisa.

> **O marco corrigiu a conclusão desta seção, e não a evidência.** As três
> mudanças de ordem são reais e continuam onde estão; o que estava errado era
> supor **alguém**. O ciclo de boot pelo dispositivo é que mexe na ordem — o
> firmware reescreve a entrada ao bootar por ela, o Windows a recria no
> `displayorder` ao subir —, e isso explica os três casos sem pedir ninguém.
> Aquele teste reprovou no marco, fazendo o que foi escrito para fazer, e
> trocou de asserção: à frente da ordem, o dispositivo tem de estar **inerte**.
> Ver [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md).

**O que a etapa mediu escrevendo no `bcdedit`, e não estava medido:**

- **O `bcdedit` aceita `bootsequence` para uma entrada de fora do
  `displayorder`.** Set, releitura, e lá está ela. Sem isso, armar obrigaria a
  pôr a entrada na ordem — exatamente o que C-5 proíbe.
- **O `displayorder` não muda**, nem ao pôr nem ao tirar.
- **A forma da linha bate com o caso construído da E2, byte a byte.** O duplo
  reproduzia aquilo por suposição desde a E2; agora é transcrição.
- **Com `bootsequence` presente, o `/deletevalue` sai com código 0** — ao
  contrário do código 1 medido na E4 quando não há o que apagar. As duas
  metades estão medidas, e é sobre elas que o desarmar decide não acreditar em
  nenhuma.

**A decisão central: o bloco deriva, e o modelo é o `live-toram`.** Registrada
em [ADR-0007](../docs/adr/0007-o-bloco-do-arca-deriva-do-live-toram.md). A
captura `teste-02` é o `menuentry --id live-toram` do próprio `grub.cfg` inerte
com **exatamente cinco** substituições — as cinco de §10.2.1 — e nada mais. O
`live-default`, que era o candidato do briefing, não tem `toram`; **ninguém
acrescentou o `toram`**, ele veio junto do modelo, e o §10.2.1 do PRD o
atribuía ao `menuentry` base sem dizer qual. O oráculo é o arquivo, e ele passa
com uma única divergência de um byte — um espaço duplo que é rastro de edição à
mão, e que o teste **nomeia** em vez de copiar.

A `teste-03` é a evidência de que a derivação não foi como aquele bloco nasceu,
e é desconfortável: ela perdeu nove parâmetros do modelo, é a **única** das
quatro com `set default="arca-backup"` — a única que provavelmente rodou
desatendida —, e o que ela perdeu inclui `nvme.poll_queues=1` numa máquina cujo
disco de origem é NVMe. Isso é argumento a favor de derivar, e não contra.

**As outras decisões desta etapa:**

- **A ordem das três gravações**, escrita em `src/armar.rs`: `estado.json`,
  `grub.cfg`, `bootsequence`. O estado primeiro porque é o único lugar onde
  fica escrito **qual** job foi armado — uma receita armada sem estado gravado
  faria a máquina rodar o backup e escrever um `arca-fim.txt` com um selo que
  ninguém anotou. A marca por último porque é a única das três que muda o que
  acontece no próximo reinício sem ninguém pedir. Os dois estados
  intermediários são nomeáveis e reversíveis por `arca desarmar`.
- **O reinício é a última coisa, depois da releitura de C-3.** Um ARCA que
  reiniciasse antes de conferir dispararia o reinício sem saber se armou. Há
  teste para isso, e ele é o que separa este comando de um perigoso.
- **Reiniciar entrou atrás de porta**, em `Sistema` — a mesma que a E6 criou
  para operações do próprio sistema. `shutdown /r /t 0`, e não `ExitWindowsEx`:
  este exige habilitar `SeShutdownPrivilege` e `AdjustTokenPrivileges` **sai
  com sucesso mesmo quando não ajustou tudo**, que é o mesmo modo de falha do
  `bcdedit` que C-3 existe para desconfiar.
- **A confirmação digitada ganhou porta própria**, e a razão é que S-2 é um
  requisito de segurança: sem porta, o caminho que separa "armou" de "não
  armou" não teria teste nenhum. É a sexta porta, e ela entrou pelo mesmo
  critério das outras — quando uma etapa precisou dela.
- **Sem nome de disco determinado, o ARCA recusa** — a pendência que a E6
  deixou. Pedir o nome ao usuário parece gentil e é pior: `nvme0n1` é um nome
  do Linux, quem o digitaria está no Windows, e não há nada deste lado contra o
  que conferi-lo. A recusa acontece **antes** da confirmação, para que ninguém
  digite o nome inteiro da imagem para ouvir um não depois.
- **Não havendo entrada de firmware nenhuma, o ARCA recusa em vez de criar.**
  Criar uma do zero é código sem original, e o lugar disso é o `arca prepare`
  da E10.
- **C-6 ganhou a metade que faltava.** Até aqui a rejeição silenciosa do §3.1
  era só relatada, por duas leituras. Agora o armar **escreve** o `device` da
  entrada e relê: um `device` que não mudou é a rejeição, e o armar para ali.

**O que o `estado.json` ganhou aqui, e por quê.** Um sexto campo, `situacao`,
que a E7 escreve como `armado`. Quem o lê e o muda é a E8 — ver
[ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md) — mas
ele nasce aqui porque armar é quando o job passa a existir, e um estado que não
diz se já foi colhido não pode ser escrito depois sem reabrir o arquivo.

**O que a revisão pegou, e o que ela mostra sobre o método.** Três achados
nesta etapa, e o mais grave é sobre um **teste** e não sobre o código:

- **Dois construtores do duplo pareciam compor e não compunham.** O
  `FirmwareDeMentira` ganhou `modelando_o_fwbootmgr` nesta etapa — o duplo
  antigo, que respondia de cor, não dava conta de um comando que **desarma e
  depois arma**, porque as duas escritas caem no mesmo alvo e esperam respostas
  contrárias. O novo modelo aplicava a escrita e devolvia "deu certo" para
  **qualquer** argumento, o que matava o `recusando_o_executar` em silêncio: um
  teste escrito como `.modelando_o_fwbootmgr(...).recusando_o_executar(...)` —
  a forma natural de exercitar um `/set description` que falha — passaria verde
  sem a recusa nunca disparar. Um teste que não prova nada é pior do que um que
  falta, porque ele ocupa o lugar. Corrigido, e há agora o teste que ele teria
  escondido: `uma_migracao_que_o_bcdedit_recusa_nao_passa_por_migrada`.
- **A letra do volume podia divergir em caixa.** `Alvo::ler` normaliza para
  maiúscula o que vem do `bcdedit`, e a comparação de C-6 é por igualdade de
  `Alvo`. Um `r` minúsculo vindo da enumeração de volumes nunca casaria com o
  `R` relido, e o ARCA diria que o `bcdedit` recusou o alvo em silêncio quando
  ele o aceitou. Latente hoje — a enumeração monta as letras de `b'A'` —, e é
  por isso que depender disso sem dizer sairia caro.
- **A linha do desfecho mostrava a metade que não serve para procurar.**
  `Desfecho esperado em backup-2026-08-22_Apps` não é um lugar: nada ali diz
  que aquilo mora sob `ARCA-LOGS\` no `ARCAVAULT`. E o `caminho_do_desfecho`, o
  `PathBuf` inteiro, **não tinha chamador nenhum** — o único lugar que o
  exibiria mostrava o nome da pasta.

E um defeito que a revisão **não** pegou, e que a execução real pegou: as
frases do `--dry-run` diziam "esta é a receita que a **etapa E7** armaria" e
"armar é a etapa E7". Eram verdade até esta etapa, e viraram a pior mentira que
um `--dry-run` pode contar — uma afirmação sobre o que o comando de verdade faz
— no instante em que a E7 ficou pronta. É a lição da E6 outra vez: **depois de
corrigir, releia o que a correção encostou.** Corrigi a frase final do pré-voo e
não reli o rodapé do ensaio, que ficava vinte linhas abaixo. Agora há um teste
que reprova qualquer frase do ensaio que adie o armar para uma etapa futura.

**O que faltava para o marco, e como cada coisa fechou** — cumprido em
22/08/2026, às 21:06:

- **Se o firmware honra o `bootsequence`** sobre uma entrada de fora da ordem.
  **Honra.** E a evidência não é do lado Windows: é o `efi-nvram.dat` que o
  Clonezilla escreve dentro de cada imagem, lido **durante** aquele boot —
  `BootCurrent: 0001` com `BootOrder: 0000,0001`. A máquina bootou por uma
  entrada que não era a primeira. P-18 fechada, e C-5 provada sustentável
  ([ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)).
- **Se o Clonezilla executa a receita** que o ARCA gravou. **Executa.** O
  mecanismo de desfecho inteiro estreou de uma vez e funcionou: o
  `arca-fim.txt` tem as três linhas, o selo bate com o do `estado.json`, e o
  `if/then/else` tomou o ramo do sucesso. P-16 fechada. O ramo de falha
  continua sem rodar, e é P-6.
- **O `grub.cfg` do dispositivo foi escrito pelo ARCA pela primeira vez**, e
  voltou ao inerte na colheita: o SHA256 de agora é o mesmo de antes de armar,
  `4B33DA61…F947AA3D`, byte a byte. A ida e a volta se cancelaram sobre o
  arquivo de que a máquina depende para bootar.

**E o que o marco trouxe que ninguém tinha pedido**, e é o achado da noite: a
entrada do ARCA **voltou para a ordem permanente, em primeiro**, e uma terceira
entrada apareceu. O ARCA não fez isso — a releitura de C-5 teria falhado —, e o
que o fez foi o próprio ciclo de boot. Está no ADR-0009, e ele reinterpreta a
tabela do §3.1 inteira: o que ela atribuía a trabalho manual tem uma causa que
não pede ninguém. O teste `a_entrada_do_arca_esta_fora_da_ordem_permanente`
reprovou nesta sessão, fazendo exatamente o que foi escrito para fazer, e trocou
de asserção pela invariante que importa — **à frente da ordem, o dispositivo tem
de estar inerte.**

**O marco era seguro, e valeu saber por quê.** A receita é de backup: lê o
`nvme0n1` e escreve no `ARCAVAULT`. Não há operação destrutiva sobre o disco de
origem. O pior caso era um reinício perdido — a máquina voltar ao Windows, e o
§5.5 chama isso de "o boot não aconteceu". Destrutivo é a E9.

### E8 · Colher o desfecho

`arca resultado`: ler o `arca-fim.txt`, conferir o selo, ler o veredito do `arca-check.log`, desarmar e imprimir §5.4. Falha parcial é falha total (S-5).

A tabela de estados terminais que o PRD não tem (D8):

| O que se encontra | Significado | O que o ARCA diz |
|---|---|---|
| Selo bate, `ARCA_FIM` presente, desfecho `OK` | Operação concluída | Veredito da imagem |
| Selo bate, desfecho `FALHOU` | Clonezilla falhou e disse | Falha, com o log apontado |
| Selo bate, sem `ARCA_FIM` | Truncado — desligamento no meio | Falha, imagem é resíduo |
| Selo não bate | Job fantasma | Ignora o arquivo e avisa |
| Sem `arca-fim.txt`, job pendente | O boot não aconteceu, ou o Clonezilla abriu menu | Falha, com as duas causas nomeadas |
| Sem `arca-fim.txt`, sem job | Nada a colher | Diz isso e para |

**Cobre**: S-4, S-5, D8
**Marco em hardware**: backup e colheita, ponta a ponta, sem intervenção.

**Escrita, testada e commitada em 22/08/2026, e o marco cumprido às 21:14:49 do
mesmo dia.** Ele dependia do da E7 — não há desfecho a colher enquanto o
primeiro backup não rodar —, e os dois caíram na mesma noite.

**A etapa é mais de fiação do que de código novo, e isso é o desenho dando
certo.** A E5 construiu o julgamento, a E3 o leitor do veredito, a E4 o
desarmar, a E1 a listagem. A E8 os liga na ordem certa e não reescreve nenhum
deles — duas versões da mesma regra divergem na primeira mudança, que é o
motivo de `arca status` já reusar `list::montar`.

**A decisão da etapa: colher marca o `estado.json`, e nunca o apaga.**
Registrada em
[ADR-0008](../docs/adr/0008-colher-marca-o-estado-em-vez-de-apaga-lo.md). Das
três saídas, apagar obrigaria a refazer a discussão de B-10 com um argumento
que não se transporta — a marca de boot único é uma **intenção** do ARCA, e o
`estado.json` colhido é **registro**, o único lugar que liga um selo a um nome.
Distinguir por outro sinal falharia justamente onde mais importa: um job cujo
boot não aconteceu não tem `arca-fim.txt` nenhum, e ficaria pendente para
sempre.

E o campo **não é uma data**, de propósito: poria mais um instante ao lado do
`armado_em` num arquivo cujo tipo de tempo existe para tornar a comparação
difícil (ADR-0006, S-6). Duas datas lado a lado são um convite a subtraí-las.

**O que fecha aqui, e é o par que a E5 deixou aberto.** Depois de um
`arca desarmar`, o `arca status` mostrava "Boot único: não armado" ao lado de um
job pendente, e ninguém encerrava o job. Agora colher encerra, e o `status` diz
"já colhido, nada esperando". Ele também **para de procurar o desfecho** de um
job já colhido — ir olhar de novo reabriria uma pergunta que o `arca resultado`
fechou, e um `arca-fim.txt` truncado pela operação seguinte apareceria como "o
boot não aconteceu" para um job que aconteceu.

**A distinção que custou mais cuidado.** Encerra o job quem chegou a um
**veredito** sobre ele — inclusive "não há `arca-fim.txt`", que é C-12 na letra
e é uma resposta. **Não** encerra quando o arquivo está lá e não se deixou ler:
"não consegui olhar" não é veredito, e encerrar ali perderia o selo que liga o
desfecho ao job. É a mesma distinção que a revisão da E5 pagou caro para
existir, aplicada de novo — e desta vez ela apareceu antes do defeito, e não
depois.

**S-5 saiu em duas linhas, e não numa conclusão.** O desfecho e o veredito são
independentes, e a §5.4 mostra os dois sem que um esconda o outro. Quatro
combinações são falha com desfecho `OK`: imagem reprovada, imagem sem veredito,
pasta que é resíduo, e pasta que não existe. Todas imprimem a tela inteira e
saem com código diferente de zero — quem chamou o ARCA de um script não pode
ler um desfecho ruim como êxito.

**O que a revisão pegou, e os três são do mesmo tipo.** Uma peça nova encaixada
numa peça antiga que ninguém releu ao encaixar — o padrão que a E3 nomeou e que
já apareceu em quatro etapas:

- **O título da seção do job dizia `Job pendente` para um job colhido.** A
  linha nova saía sob ele como *"Job pendente / Estado no ARCABOOT: já colhido,
  nada esperando"* — uma versão menor exatamente da contradição que esta etapa
  existia para fechar. O título passou a variar com o estado, e ganhou um
  terceiro caso: sem `estado.json` legível ele é só `Job`, porque `pendente`
  afirmaria haver um e `último` afirmaria ter havido.
- **O relatório inteiro se perdia quando a gravação falhava.** Gravar antes de
  imprimir é o certo — uma linha `Job: encerrado` impressa antes da gravação
  seria um `ok` sobre uma ação que não aconteceu (§11) —, mas gravar com `?`
  descartava a §5.4 já computada. O ARCA teria lido o desfecho do backup e o
  jogado fora. As duas propriedades cabem juntas, e a saída é um tipo:
  grava-se antes, e **o que a gravação respondeu vai para a linha**. Há agora
  um terceiro estado, `NAO FOI POSSIVEL ENCERRAR`, distinto do
  `CONTINUA PENDENTE` — um é acidente e pede conserto, o outro é o desenho
  funcionando.
- **O sexto campo obrigatório torna ilegível todo `estado.json` anterior**, e a
  consequência não era benigna: `arca resultado` recusa antes de desarmar, então
  um dispositivo genuinamente armado pela versão anterior não podia ser nem
  colhido nem desarmado pelo comando. Não há arquivo assim em lugar nenhum
  hoje — o `R:\arca\` está vazio, e as duas etapas saem juntas —, mas a forma
  do problema é real. A saída não foi afrouxar o leitor: seria abrir mão da
  propriedade que o torna confiável, e é a mesma escolha que o ADR-0006 já fez
  para chave desconhecida. Foi **a mensagem de erro dizer o que fazer** —
  `arca desarmar` não consulta estado nenhum (C-1) e por isso funciona
  justamente ali.

**O que faltava para o marco, e como cada coisa fechou** — cumprido em
22/08/2026, às 21:14:49:

- **P-16 fechou, e fechou pelo lado bom.** O `arca-fim.txt` apareceu, com selo,
  e o selo é o do job: `7d2d2f5153625b38` nas duas pontas — no `estado.json` que
  o ARCA escreveu antes de reiniciar e na primeira linha do arquivo que o
  Clonezilla escreveu do outro lado. Conferido **a olho** antes de acreditar no
  julgamento da E5, que é o que a etapa pedia.
- **O `arca-fim.txt` está guardado**, em
  `recursos/capturas/arca-fim-2026-08-22_Apps.txt`, com o `arca-check.log` e
  mais três capturas do marco. A procedência de cada uma está no
  `PROVENIENCIA.md`, e ela diz o que cada uma prova.
- **A §5.4, a §3.5 e o §11 foram corrigidos contra a execução real.** Três
  números da §5.4 mudaram — a imagem é de 39,7 GB e não 36,2, sobraram 125 GB e
  não 164, e a listagem tem três imagens e não duas. E a §5.2 tinha um defeito
  que só a execução mostraria: o `Desfecho esperado em` ainda trazia o nome da
  pasta, quando a revisão da E7 já tinha corrigido o **código** para mostrar o
  caminho inteiro. É a lição da E6 pela terceira vez, e desta vez o que ficou
  para trás foi o documento.
- **A tabela do §5.5 não ganhou linha**, e isso é resultado. A execução produziu
  exatamente o primeiro caso, e o `arca resultado` o classificou nele. As outras
  seis continuam sem original.

**O que o §10.2.3 ganhou, e não estava previsto.** A linha que rodou pode ser
medida — `cargo run --example orcamento_da_linha_do_kernel` —, e a medição
corrige um número que parecia folgado: o `menuentry` base **deste** dispositivo
ocupa 471 bytes, e a reserva é 512. As capturas mediam 206, 369 e 369, e o
documento dizia que 512 era "quase 40% acima do maior já visto". Sobram 41
bytes, não 143. É o argumento do ADR-0007 visto pelo outro lado: as capturas
descreviam um `menuentry` mais pobre do que o modelo de que o ARCA deriva.

A primeira versão daquele exemplo contava o **recuo do bloco** junto da linha, e
a revisão pegou: o recuo é do `grub.cfg`, não do que o `grub` entrega ao kernel.
Eram dois caracteres, e os números inflados já tinham sido transcritos para o
PRD. Vale registrar porque é o erro que a §5.2 acabara de cometer na mesma
sessão — medir a coisa errada e publicar o número —, e é o mesmo do `498,7 GB`
que a E6 corrigiu.

**O que a colheita apagou, e não dá para recuperar.** O `grub.cfg` como ficou
armado não existe mais: o `arca resultado` desarma ao colher, e desarmar
reescreve o arquivo. A primeira receita que o ARCA gravou num dispositivo durou
vinte e um minutos. A reprodução é determinística e o exemplo acima a gera, mas
**reprodução não é captura** — e é por isso que ela não foi guardada em
`recursos/capturas/`, que é a pasta dos originais.

**O que a etapa prova em `tests/e8_colher_o_desfecho.rs`**: que o `arca-fim.txt`
de 21/08 continua **sem selo**, que ele continua inalcançável pelo ARCA de hoje
(a pasta do log leva a operação no nome, decisão da E3, e ninguém a tinha
conferido contra o disco), e que as duas formas de veredito do ADR-0003
continuam legíveis nas imagens que estão lá.

**E ganhou o teste que o marco tornou possível.** Um daqueles se chamava
`o_unico_arca_fim_do_dispositivo_continua_sem_selo`, e o comentário dele previa
o próprio fim: *"a partir do primeiro `arca backup` colhido, haverá um segundo
`arca-fim.txt`, esse sim com selo"*. Há. O teste virou
`o_arca_fim_de_21_08_continua_sem_selo`, e ao lado dele entrou
`o_desfecho_do_marco_e_julgado_como_operacao_concluida` — que corre contra a
captura, e não contra o dispositivo, porque o arquivo no `ARCAVAULT` será
truncado pelo próximo `arca backup 2026-08-22_Apps`: toda receita começa com
`echo ARCA_SELO=… >`, e o `>` trunca ao abrir.

Ele prova o que a etapa inteira existia para provar: **o julgamento da E5,
escrito contra texto inventado e duplos, classifica o primeiro original de
verdade no ramo certo** — `Concluida` com o selo do job, `JobFantasma` com
outro. O selo faz nas duas direções o que se esperava dele.

**O que a revisão do marco pegou, e os dois piores são o mesmo erro.** A linha
`Ordem de boot` que o ADR-0009 mandou acrescentar ao `arca status` nasceu com
dois furos, e os dois são a mesma pergunta mal feita — *onde está a entrada
chamada `ARCA`?* em vez de *alguma entrada alcança este dispositivo?*:

- **A captura desta máquina tem duas entradas para `partition=R:`**, e a linha
  só olhava uma. A `{687478f2}` `UEFI OS` — que o firmware criou, e por onde o
  `nvram-live-2026-08-22.txt` mostra a máquina tendo bootado — é invisível para
  quem procura pelo nome. Com ela em primeiro e a do ARCA atrás do Windows, o
  `status` diria *"o Windows vem antes"* e engoliria o aviso, enquanto todo
  reinício com o SSD conectado continuaria bootando no dispositivo. **A
  evidência que desmentia o código estava na captura que a mesma sessão tinha
  acabado de guardar.**
- **O `o Windows vem antes` era texto fixo**, e não o que estava de fato à
  frente. Agora a linha nomeia a entrada que vem antes, lendo a descrição dela.
- **A seção não guardava em `viu_o_gerenciador`.** `firmware::ler` nunca falha:
  um `bcdedit` que não se deixou entender devolve `ordem_permanente` vazia, e
  vazia é indistinguível de "o dispositivo está fora da ordem" — a resposta
  tranquilizadora. O `armar` e o `desarme` guardam nessa flag desde a E4; o
  caminho novo não guardava, e transformava "não entendi a resposta" numa
  afirmação de segurança. É C-3 pelo avesso, e o teste que o cobre monta o caso
  difícil: o `{fwbootmgr}` faltando **enquanto** os blocos das entradas saem
  certos, que é quando `entrada_do_arca()` responde normalmente e só a flag
  separa uma coisa da outra.
- **O teste da invariante dizia "na ordem" e devia dizer "em primeiro".** Ele
  usava `.any()` — pertinência — enquanto o ADR e o §11 falam de estar à frente.
  Consequência concreta: com o dispositivo em segundo e legitimamente armado —
  o estado **normal** da janela entre o `arca backup` e o reinício — a suíte
  ficaria vermelha acusando um perigo inexistente, e contradizendo o
  `arca status`, que declara essa mesma configuração segura.
- **O exemplo do orçamento contava o recuo do bloco**, e os números inflados já
  tinham ido para o §10.2.3. Dois caracteres, e é o mesmo erro do `498,7 GB` da
  E6: medir a coisa errada e publicar o número.
- **O `PROVENIENCIA.md` afirmava conferência por SHA256 e não registrava os
  hashes.** Era a única coisa que aquele documento existe para permitir.

**E a decisão do ADR-0008 foi exercitada em hardware, e não só em duplo.**
`arca resultado` rodado uma segunda vez sobre o mesmo job diz que ele já foi
colhido, mostra o selo, e **não desarma de novo** — conferido pelos dois lados:
o `grub.cfg` continua com o SHA256 de antes de armar e o `mtime` da colheita, e
o `estado.json` continua em `"situacao": "colhido"`, intocado. Era a
consequência que o ADR previa, e ela custava um marco para ser vista.

## Fase IV — O resto

### E9 · Restauração

Só começa depois do marco da E8. Lista no Windows, com a escolha antes do ponto sem volta (R-1); conferência do destino contra a própria imagem (R-2); nome da imagem digitado por extenso (R-3). Destino divergente segue a decisão 5: passa com confirmação que nomeia o disco, e é recusado se for menor que a origem.

**Cobre**: R-1, R-2, R-3, R-7, R-8, L-2 — e **P-17**, que era a etapa por escrito.
**Marco em hardware**: restauração completa disparada pelo ARCA. **Cumprido em 23/08/2026, às 11:50:53.** Com ele, o projeto está funcionalmente pronto: backup, colheita e restauração rodaram ponta a ponta em hardware, disparados pelo ARCA e sem uma única tela. O que resta — E10 e E11 — serve ao segundo dispositivo e à verificação rápida.

#### O que a etapa entregou, e onde

A terceira etapa seguida em que quase nada é mecanismo novo, e isso é o desenho
dando certo. A receita de restauração está montada e validada desde a E3;
`armar::executar` recebe a operação como parâmetro e não sabe qual é;
`desfecho::ler` já lê `ARCA_RESTORE=` por sufixo; `arca resultado` já colhe as
duas operações. O que a E9 escreveu foi **a escolha, a conferência e a
recusa**:

| Peça | Onde | O que responde |
|---|---|---|
| A lista numerada, sem resíduo | `src/comandos/restore.rs` | R-1, L-2 |
| A conferência da imagem contra ela mesma | idem, `conferir_a_imagem` | R-2 |
| A medida da origem, de dentro da imagem | `src/gpt.rs` — **módulo novo** | R-7 |
| A medida do destino, pelo `MSFT_Disk` | `portas::Medida`, `adaptadores/windows/wmi.rs` | R-7 |
| A escolha e o julgamento do destino | `restore.rs`, `escolher_o_destino` | R-7, R-8 |
| A confirmação digitada, compartilhada com o backup | `src/confirmacao.rs` — **módulo novo** | S-2, R-3 |
| O aviso da janela do ADR-0009 na restauração | `restore.rs`, `montar_o_armado` | C-9 |
| A colheita que não confunde o sujeito | `src/comandos/resultado.rs` | S-5 |

#### O achado que muda a etapa, e ele está medido

**O mesmo disco tem dois tamanhos conforme quem responde.** Medido nesta
máquina em 23/08/2026:

```text
Get-Disk (MSFT_Disk) ........ 500.107.862.016 bytes = 976.773.168 setores
Win32_DiskDrive.Size ........ 500.105.249.280 bytes = 976.768.065 setores
nvme0n1-gpt.sgdisk na imagem  976.773.168 setores
diferenca ................... 2.612.736 bytes = 5.103 setores
```

`60801 × 255 × 63 × 512` dá exatamente o número do `Win32_DiskDrive` — a
geometria CHS legada truncada no último cilindro inteiro; os 5.103 setores que
faltam são menos de um cilindro (16.065). O `MSFT_Disk` bate byte a byte com o
que a imagem registra.

A armadilha é de **régua**: medir a origem pela GPT de dentro da imagem e o
destino pela fonte que `Discos::discos_fisicos` usa desde a E6 faria R-7 recusar
o disco por não caber **nele mesmo**. Para B-4 a fonte antiga continua servindo
— lá ela superestima o em uso, que é o lado seguro. Ver
[ADR-0010](../docs/adr/0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md).

#### As duas coisas que a etapa achou fora do escopo

**P-19 estreitou, e não pelo caminho previsto.** O ADR-0009 apostou que um
segundo backup responderia, pelo `efi-nvram.dat` de dentro da imagem. Os dois
`efi-nvram.dat` — o de 21/08 e o de 22/08 — saíram **byte-idênticos**. Quem
respondeu foram as capturas de 20/08 que já estavam no dispositivo: em três
boots pelo dispositivo a entrada continuou na forma que o `bcdedit` escreve,
o que descarta *"o firmware reescreve em todo boot"*. E, no caminho, apareceu
que as duas leituras de NVRAM de 21/08 são de **dois boots diferentes** — uma
do backup e outra da restauração — e que a que o §3.1 usava é da restauração.
Ver [ADR-0011](../docs/adr/0011-as-capturas-de-21-08-sao-de-dois-boots.md).

**A recusa engolindo o desarmar, de novo.** A primeira versão do
`arca restore` montava a tela inteira depois de julgar imagem e destino, e com
a recusa subindo como erro nada era impresso — o desarmar de C-1, que já tinha
acontecido, sumia em silêncio. É o **mesmo defeito** que a revisão da E7 pegou
no `arca backup`, cometido de novo com o comentário que o descreve a poucas
linhas de distância. Achado **rodando o comando de verdade**, e não relendo o
código: `arca restore --destino 1` imprimiu a recusa e mais nada.

#### O que a revisão de código pegou, e o padrão é o de sempre

Cinco defeitos, e **quatro deles são a peça nova encaixada numa peça antiga que
ninguém releu ao encaixar** — o mesmo padrão que a E3 nomeou e que a E7 pagou
duas vezes.

**O mais grave: uma recusa por identidade do Windows guardando um valor do
Linux.** R-8 recusa o dispositivo como destino pela **letra**; o nome que vai
para a receita é do **Linux**, e sai de um casamento por **modelo** nos
`blkdev.list`. Com um segundo disco do mesmo modelo do dispositivo,
`--destino <o outro>` passava pela recusa por letra, passava pela medida e pelo
tamanho — e o passo que resolve o nome achava aquele modelo só sob `sda`, que é
o dispositivo. A receita sairia `restoredisk <imagem> sda`, que é exatamente o
desfecho que R-8 existe para impedir. **A recusa dura tinha um contorno por
acidente de modelo.** A defesa é resolver o nome do Linux do dispositivo pelo
mesmo oráculo e comparar.

**O `Model:` do `sgdisk` era opcional, e o vazio viajava.** Uma imagem sem
aquela linha produzia `modelo == ""`, e daí: R-2 recusava uma imagem coerente
por "as fontes discordam"; a busca do destino dizia "nenhum disco desta máquina
tem o modelo ``"; e a tela de confirmação imprimia `Origem da imagem:  ·
nvme0n1`. É o mesmo raciocínio que faz o leitor do WMI exigir o `Model`.

**`NadaAOferecer` contava só os resíduos.** Um `ARCAVAULT` com uma única pasta
de imagem cujo nome não passa por B-2 dizia "não há imagem no ARCAVAULT para
restaurar" enquanto o `arca list` mostrava a pasta — a mesma omissão que o
próprio módulo argumenta contra para o resíduo.

**A busca do destino por modelo não excluía o dispositivo.** Com ele tendo o
modelo do disco de origem, havia dois candidatos e a recusa saía
`DestinoAmbiguo` — cuja mensagem manda "nomeie o destino com `--destino`", um
caminho que ali também não leva a lugar nenhum.

O quinto — C-6 e C-10 não checados no `arca restore` — já estava corrigido
quando a revisão terminou; ela leu uma versão anterior do arquivo. Foi achado
relendo `prevoo::julgar` com a restauração na mão, que é a mesma defesa.

#### O que faltava para o marco, e como cada coisa fechou

**Cumprido em 23/08/2026.** A restauração foi armada às 11:10:50, a máquina
bootou pelo dispositivo, apagou e reescreveu o `nvme0n1`, o `ocs-sr` encerrou
às 11:31:55 do relógio do live e desligou; a colheita foi às 11:50:53. O
Windows que colheu **veio de dentro da imagem** — é a primeira vez neste
projeto em que o ARCA julga uma operação de dentro do que ela produziu.

O que este bloco dizia antes do reinício continua abaixo, reescrito contra o
que aconteceu. Tudo que era verificável sem reiniciar estava feito: 546 testes,
9 deles de integração contra o hardware desta mesa, o `--dry-run` rodado de
verdade e as recusas exercitadas no binário real.

**E o caminho sem `--dry-run` rodou inteiro, até uma linha antes do ponto sem
volta.** Em 23/08/2026, `arca restore 2026-08-22_Apps` com a confirmação
digitada **errada** de propósito:

```text
  Desarmando receita anterior ..... ok · ja estava inerte · R:\boot\grub\grub.cfg
  Imagem escolhida ................ 2026-08-22_Apps
  Origem da imagem ................ KINGSTON SNV3S500G · nvme0n1 · 976773168 setores de 512 B · 465,8 GB
  Destino ......................... KINGSTON SNV3S500G · disco 0 do Windows · nvme0n1 · 976773168 setores de 512 B · 465,8 GB
  Cabe (R-7) ...................... ok · o destino tem exatamente o tamanho da origem
  ...
ATENCAO: a restauracao APAGA o disco de destino.

erro: a confirmacao nao bate: era para digitar `2026-08-22_Apps` e veio
`nao-e-o-nome`. Nada foi armado                            (codigo de saida 1)
```

O desarmar de C-1 **aconteceu de verdade**, a conferência de R-2 leu os
arquivos de dentro da imagem, R-7 comparou as duas medidas reais, e S-2 barrou.
Conferido pelos dois lados: o `grub.cfg` e o `estado.json` saíram com o mesmo
SHA256 e o mesmo conteúdo de antes. É o análogo do que a E7 fez com o
`bootsequence` — exercitar tudo que não custa um reinício.

**O que faltava era o reinício**, e ele apagou o disco desta máquina.

**A imagem foi a `2026-08-22_Apps`**, decidida em 23/08/2026. Três razões, e a
terceira é a que importa para o projeto:

1. É a mais recente, e o que ela perde é quase nada: um atalho no Desktop
   (`Powershell Admin.lnk`, de 22/08 às 21:17) e três commits — `d45bfa7`,
   `69034b7` e o desta etapa —, **os três no `origin`**. Fora do
   `Repository` e do Desktop, nada no perfil mudou depois de 22/08 21:00; e
   dentro dele, dos 718 MB que mudaram, 717,8 MB são `target\` e `.git\`.
2. Já foi exercitada: o `--dry-run` e o caminho real até uma linha antes do
   ponto sem volta rodaram com ela hoje.
3. **É a única imagem que o ARCA gravou.** O `Info-saved-by-cmd.txt` dela traz
   o `ocs-sr` de B-8 na ordem de B-8, escrito pelo próprio Clonezilla.
   Restaurá-la fecha o ciclo com uma imagem que o próprio ARCA produziu — e é
   a diferença entre provar o mecanismo e provar o mecanismo sobre si mesmo.

E o binário do `ARCABOOT` foi atualizado para o desta etapa. Não é zelo: o
`arca.exe` de `target\release\` mora no `C:`, e a restauração o devolve à
versão de 22/08 — que colheria a restauração com a tela do backup, chamando o
veredito da imagem de origem de "Verificacao". É §4.1 sendo usada pela primeira
vez para o que ela existe. **E funcionou como escrito**: o `arca resultado` que
colheu é o `R:\arca\arca.exe`, e a tela que saiu foi a do §6.3.

Faltavam quatro coisas, e as quatro fecharam:

1. **`arca restore 2026-08-22_Apps`, a confirmação digitada, e a máquina
   reiniciando.** Aconteceu. O `estado.json` do `ARCABOOT` registra
   `armado_em: 2026-08-23T11:10:50-03:00`, selo `ce04819cf0ee96f7`, e é o
   **único registro do armar que sobreviveu** — a linha correspondente do
   `arca.log` foi apagada pela própria restauração.
2. **As cinco linhas do armar (§6.1) e o `arca-restore.log`, nenhum dos dois
   com original.** O log tem original agora: 16.600 bytes em
   `recursos/capturas/arca-restore-2026-08-22_Apps.log`, com o `restoredisk`
   chegando ao fim, o `ocs-restore-mbr`, os dois `ntfsfix` e o
   `Ending /usr/sbin/ocs-sr`. **E ele começa no meio** — uma passagem só do
   Partclone, a da última partição, sem o `Starting` correspondente ao `Ending`.
   O primeiro original de uma coisa costuma ensinar o que ela não é, e este
   ensinou: o §6.3 manda procurar ali quando algo deu errado, e o que está ali
   pode não cobrir a parte que falhou. **As cinco linhas continuam sem captura**, e
   agora se sabe por quê: elas foram impressas de verdade, e a sessão que as
   imprimiu morreu no reinício que ela mesma disparou. Está registrado no §6.1
   e na `PROVENIENCIA.md` como perda, e não como pendência — reprodução não é
   captura.
3. **`arca resultado` colhendo uma restauração: a tela do §6.3.** Saiu, e o
   §6.3 passou a ser execução real. As três diferenças que a etapa desenhou
   apareceram todas: `Imagem de origem:` em vez de `Verificacao:`, o veredito
   sem reprovar a operação, e os três conselhos. O julgamento da E5 classificou
   o desfecho em `Concluida` com o selo do job, e o ADR-0008 foi exercitado de
   novo — `"situacao": "colhido"` ficou no arquivo. **E as duas telas de
   colheita existem lado a lado**, com catorze horas e uma operação destrutiva
   entre elas: as três diferenças deixaram de ser desenho e viraram observação,
   e a §5.4 foi conferida linha a linha contra a original.
4. **As duas leituras que respondem §3.4 pelo lado do ARCA.** As duas estão
   guardadas, **e o par não fechou idêntico** — é o achado do marco, e está no
   [ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md).

#### O achado do marco: a ordem permanente estava dentro da imagem

Antes de restaurar, o `displayorder` tinha três entradas, com o dispositivo em
primeiro. Depois de restaurar e religar, tem uma: o `{bootmgr}`. E a leitura de
depois é **byte a byte** a que a E2 capturou em 22/08 de manhã — mesmo SHA256,
`d837093d…f204f15e`, idênticas linha a linha.

O ARCA não fez isso: C-5 proíbe, e o armar e o desarme releem o firmware depois
de escrever. O `ocs-sr` também não, e quem responde isso é o par de 21/08 do
§3.4 — duas leituras do mesmo boot, com o Clonezilla correndo entre elas.
**Não é o `arca-restore.log`**, e por pouco não foi: ele não tem uma linha de
`efibootmgr`, e usar essa ausência como prova seria cometer, dentro do próprio
achado, o erro que ele descreve — o log começa no meio. O que sobra é que a
partição EFI está dentro da imagem, e o BCD está dentro dela: **a ordem permanente
voltou junto com o disco.**

A entrada `{687478f2}` `UEFI OS` fecha o argumento pela data. Ela nasceu na
NVRAM durante o boot do backup de 22/08, com o Windows desligado, e nunca
chegou ao BCD que aquela imagem carrega. Restaurar a apagou porque ela nunca
esteve lá dentro.

**Isto estreita P-20, que é da E10.** A restauração não precisa do conserto que
o pedido descreve — ela já devolve a ordem. O pedido é sobre o **backup**, que
suja a ordem e não a limpa. E abre P-22, que é o outro lado da mesma moeda:
*o `bcdedit` mostra a NVRAM ou o BCD?* Se for só o BCD, a máquina pode continuar
bootando no dispositivo enquanto o `arca status` diz que está tudo bem — uma
afirmação de segurança sobre uma leitura que não fala da pergunta. **Um
reinício com o SSD conectado responde**, sem risco: não há job armado e o
`grub.cfg` está inerte, então o pior caso é um menu esperando alguém.

#### O que o marco mostrou sobre o §4.1, e não estava previsto

O `%LOCALAPPDATA%\ARCA\arca.log` foi destruído, como a etapa previa. O que ela
não previa é **onde** o buraco ficaria: a última linha do lado de lá é de 22/08
às 20:53:48 — o armar do **backup** —, e a seguinte já é a desta colheita.
Sumiram no meio a colheita do backup das 21:14, o `--dry-run` desta manhã, a
recusa da confirmação errada, e **a linha do armar desta própria restauração**,
escrita quarenta minutos antes de a imagem substituí-la.

**A operação apaga o registro de que ela foi armada.** O `estado.json` do
`ARCABOOT` é o que sobrou, e é por isso que ele não é redundância do log. O
arquivo está em `recursos/capturas/arca-log-windows-2026-08-23-pos-restauracao.txt`
— a única captura deste projeto que está lá pelo que lhe **falta**.

#### O que a etapa prova agora em `tests/e9_restaurar_o_disco.rs`

Três testes que não podiam ser escritos antes do marco, cada um contra um
original que não existia:

- **As duas pontas do selo**, uma de cada lado do reinício: o `estado.json` que
  o Windows escreveu antes de desligar e o `arca-fim.txt` que o `bash` do live
  escreveu depois de o disco ter sido apagado. O julgamento da E5 diz
  `Concluida` para o selo do job e `JobFantasma` para o do backup de 22/08 — que
  é o job fantasma mais plausível que este dispositivo tem.
- **A receita que o código monta hoje escreve as três linhas que o original
  traz**, e não escreve `ARCA_BACKUP`. Fecha, do lado da restauração, o que a
  E7 fechou do lado do backup: até aqui o `ARCA_RESTORE=` era código sem
  original (P-16).
- **A mudança da ordem permanente**, fixada contra as duas leituras do `bcdedit` e
  contra a captura da E2.

Os três passaram de primeira, o que neste projeto é motivo para desconfiar —
então dois foram **falsificados** antes de ficarem: trocando a operação do
pedido para `Backup`, o segundo reprova dizendo *"o original traz
`ARCA_RESTORE=OK`, e a receita de hoje nao a escreveria"*; apontando a captura
de depois para a de antes, o terceiro reprova nomeando as três entradas da
ordem. É a lição da revisão da E4 aplicada antes do defeito, e não depois.

### E10 · `arca prepare`

Exige a FAT32 vazia de ≥ 1 GB já criada — o ARCA não particiona (§7.1). Baixa o Clonezilla na versão fixada, confere contra o SHA256 embutido, extrai, instala o ARCA no `ARCABOOT`, migra a entrada de firmware. `--iso <caminho>` para offline, que é o que salva quando a máquina que precisa preparar o dispositivo é a que está sem Windows.

Fica tarde de propósito: o dispositivo atual já existe, preparado à mão. Esta etapa serve ao **segundo** dispositivo.

**Cobre**: §7.1 — e **P-20, que fechou antes do resto da etapa** e virou C-13.

#### O `arca resultado` devolve o Windows à frente da ordem de boot (P-20)

**Entregue em 23/08/2026, e é a única parte da E10 que existe em código.** O
`arca prepare` continua ⬜; isto saiu na frente porque o incômodo era diário e
não dependia dele. Ver
[ADR-0013](../docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md), que
supersede a decisão do ADR-0009.

O que ficou, em uma frase: **`/set {fwbootmgr} displayorder {bootmgr}
/addfirst`, incondicional, nos três caminhos do `arca resultado`, com releitura
de C-3 sobre a pós-condição e linha própria na tela.** Nada é removido — as
entradas do dispositivo ficam na ordem, atrás do Windows.

O texto abaixo é o pedido como ele foi registrado em 22/08, e o que aconteceu
com cada ponto dele está no bloco **"como cada coisa foi decidida"**, no fim
desta seção.

Pedido em 22/08/2026, depois do marco da E8, e a razão é operacional: com o
dispositivo em primeiro no `displayorder`, **ligar a máquina com o SSD
conectado boota nele**. O grub está inerte, então isso para no menu do
Clonezilla e espera alguém. A rotina vira "ligar sem o SSD, conectar depois", e
isso é fricção que o usuário paga em todo boot.

A entrega: ao colher, o `arca resultado` põe o `{bootmgr}` em primeiro no
`displayorder`, **independentemente de o dispositivo estar conectado** — o
estado ruim é da NVRAM, e não do que está na mesa.

**Isto exige revisar C-5, e a revisão é um ADR.** C-5 diz "nunca alterar a
ordem permanente", e o [ADR-0009](../docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)
decidiu, no mesmo dia, **avisar em vez de consertar**. Quem for implementar tem
de derrubar aquela decisão com argumento, e não por baixo dela — um ADR novo
que a supersede.

O argumento a favor, e ele é forte o bastante para a revisão valer a pena:
**C-5 foi escrito contra uma operação e este pedido é a oposta.** O perigo que
o §3.1, o ADR-0007 e o `src/armar.rs` nomeiam é o ARCA **acrescentar** um
caminho permanente para bootar no dispositivo — "desfeito o job, a máquina
continuaria com um caminho a mais". Pôr o Windows à frente **remove** um
caminho. As duas escrevem no mesmo `displayorder`, e C-5 não distingue as duas;
a assimetria é real e nunca foi discutida.

Três coisas contra, e nenhuma é fatal:

- **O ADR-0009 argumentou que a entrada foi posta pelo Windows**, a partir do
  objeto `{f4057bd0}` do BCD, e que desfazer isso é mexer numa decisão de outro
  dono. Repare que o pedido não pede tirar a entrada — pede **reordenar**, o
  que é menos invasivo e reversível pelo mesmo `bcdedit`.
- **É a NVRAM de boot**, onde um erro deixa a máquina sem bootar. A releitura de
  C-3 é obrigatória, e o modo de falha do `bcdedit` — responder êxito sem ter
  mudado nada — já está medido desde a E2.
- **O conserto não é permanente.** O ADR-0009 mediu que o ciclo de boot põe o
  dispositivo de volta na ordem a cada backup. Então isto é limpeza recorrente,
  e o `arca resultado` é justamente o lugar certo para ela: roda uma vez por
  job, depois do boot que sujou a ordem.

**E o marco da E9 estreitou o pedido, o que é argumento a favor e não contra.**
Medido em 23/08/2026: uma **restauração** devolve a ordem sozinha, porque a
partição EFI está dentro da imagem e o BCD está dentro dela — a leitura de
depois de religar saiu byte a byte igual à captura de 22/08 de manhã
([ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)).

Duas consequências para quem for implementar:

- **O conserto é sobre o backup, e só.** Metade dos casos já se resolve, e o
  `arca resultado` de uma restauração não tem o que arrumar. Uma linha
  `Ordem de boot: devolvida ao Windows` numa colheita de restauração seria um
  `ok` sobre ação que não aconteceu — a mesma mentira que a E6 e a E7 pegaram
  duas vezes.
- **A ordem tem um terceiro dono, e ele não estava nomeado.** O ADR-0009
  arbitrava entre o ARCA e o Windows; a imagem é o terceiro, e ela escreve sem
  perguntar a nenhum dos dois. Um conserto que rodasse no `arca resultado` de
  uma restauração estaria discutindo com um estado que acabou de ser gravado
  por cima.

**E há uma pergunta a fechar antes desta**, e ela é mais barata: **P-22 — o
`bcdedit` mostra a NVRAM ou o BCD do disco?** Se mostrar só o BCD, a leitura
que este pedido usaria para conferir o próprio conserto (C-3) não fala do que
decide o boot — e a linha `Ordem de boot` do `arca status` já estaria
tranquilizando sem base. Um reinício com o SSD conectado e o dispositivo inerte
responde, sem risco.

**O que medir antes de escrever**, e nada disto está medido: a forma exata do
comando que reordena (`/set {fwbootmgr} displayorder {bootmgr} /addfirst` é o
candidato), se ele sai com código 0, se a releitura confirma, e o que ele faz
com as outras entradas — a `{687478f2}` `UEFI OS` do firmware inclusive. Medir
à mão primeiro, como a E7 fez com o `bootsequence`, e transcrever depois.

**E há uma pergunta de desenho a decidir**: se isto entra no `arca resultado`,
ele passa a fazer duas coisas — colher e arrumar. A E8 já registrou que
misturar "colhi" com "arrumei" tira de quem lê a saída a informação de qual das
duas aconteceu, e foi por isso que `arca resultado` **não** desarma quando não
há job. A saída tem de dizer as duas coisas em linhas separadas, ou isto é um
comando próprio.

#### Como cada coisa foi decidida, em 23/08/2026

**O requisito veio mais estreito do que o pedido de 22/08, e o recorte é o que
tornou a solução pequena.** *"Depois do boot inicial após um backup ou
restauração eu não me incomodo de ter que retirar o SSD. Mas depois disso, eu
me incomodo."* Então **C-9 fica inteiro** — remover o SSD antes de religar
continua sendo o que a tela pede logo depois de armar, e continua sendo a
defesa da janela em que o `grub.cfg` está armado. O que se conserta é o estado
**permanente**, dali em diante.

**Medido à mão antes de virar código**, como a E7 fez com o `bootsequence`, e a
NVRAM conferida byte a byte contra o estado inicial no fim:

| Comando | Exit | Efeito | Releitura |
|---|---|---|---|
| `displayorder {ARCA} /addfirst` | 0 | ARCA ao topo | confirma |
| `displayorder {bootmgr} /addfirst` | 0 | Windows ao topo, **ARCA fica em segundo** | confirma |
| idem, já consertado | 0 | nada muda | confirma |
| `displayorder {ARCA} /remove` | 0 | sai da ordem, **o objeto sobrevive** | confirma |

Os quatro respondem *"A operação foi concluída com êxito"* — o texto em que
este projeto não confia desde a E2. Quem responde é a releitura, e ela pergunta
a pós-condição que importa: *o primeiro da ordem é o `{bootmgr}`?*

**`/addfirst`, e não `/remove`, e o motivo não é o óbvio.** O `/remove` faria a
ordem voltar literalmente ao que era antes de o ARCA existir, que é o que o
pedido descreve. Ficou de fora pelo modo de falha: ele precisa acertar **quais**
entradas tirar, e *"quais levam ao dispositivo"* é a pergunta que a revisão do
marco da E8 já pegou respondida errado — a linha do `arca status` procurava
pela entrada **chamada** `ARCA`, enquanto quem levava ao dispositivo era a
`{687478f2}` `UEFI OS`, que o firmware criou e que nome nenhum encontra. Um
alvo fixo não faz essa pergunta, e vale para todas as entradas de uma vez,
inclusive as que o firmware criar depois.

**As três objeções do bloco acima, uma a uma.** A do ADR-0009 — *"a entrada foi
posta pelo Windows, e desfazer isso é mexer numa decisão de outro dono"* —
deixou de morder, porque nada é desfeito: a entrada continua na ordem. A da
NVRAM de boot continua de pé e virou a releitura obrigatória de C-3. E a
terceira, *"o conserto não é permanente"*, era argumento a favor desde sempre:
é limpeza recorrente, e a colheita é onde ela cabe.

**A pergunta de desenho foi respondida pelas duas metades.** Entra no
`arca resultado`, em **linha própria** — `Ordem de boot`, com o mesmo rótulo do
`arca status` —, e o parágrafo de conselho só aparece quando houve conserto. E
acontece nos **três** caminhos do comando, inclusive os dois que não desarmam:
a ordem permanente é estado da NVRAM, e não do job. Desarmar desfaz uma
intenção do ARCA, e sem job não houve intenção; a ordem está suja ou não está.

**O que a execução real pegou, e os testes não pegavam.** Dois defeitos, numa
versão com a suíte verde:

- **A linha saía com o GUID onde promete um nome.** O código lia
  `/enum {fwbootmgr}`, como o `desarme` faz, e aquele alvo devolve o bloco do
  gerenciador **sem as entradas** — a ordem vinha certa e a descrição nunca era
  achada. A raiz estava no duplo, que respondia a mesma coisa aos dois alvos; o
  `bcdedit` não os junta.
- **O conselho não saía no caminho "já colhido".** A linha estava nos três
  caminhos e o parágrafo em dois, e o teste que devia pegar cobrava só a linha
  — o caso fácil do que ele existia para cobrir. É a lição da revisão da E4
  outra vez.

**O binário do `ARCABOOT` foi atualizado** (§4.1): é ele que se roda depois de
uma operação, e um conserto que só existisse no `target\release\` não chegaria
a quem precisa dele.

### E11 · `arca verify`

`MD5SUMS` conferido no Windows, em segundos. `--completo` arma boot único que só roda `ocs-chkimg` e desliga — mesmo mecanismo da E7, receita menor.

**Cobre**: D6

---

## Cobertura de requisitos

Nenhum requisito do PRD fica sem etapa.

| Etapa | Requisitos |
|---|---|
| E0 | C-7, C-8, S-1 |
| E1 | B-1, B-3, B-10, S-3, D7 |
| E2 | C-3, C-4, C-6 |
| E3 | C-2, B-2, B-7, B-8, B-9, R-4, R-5, S-4 |
| E4 | C-1 — e aplica C-3 (releitura depois de escrever) e defende C-5 (a ordem permanente não muda ao desarmar) |
| E5 | R-6, S-6 |
| E6 | B-2, B-3, B-4, B-5, B-6 |
| E7 | C-4, C-5, C-9, S-2 — e a segunda metade de C-6, que até aqui só era relatada por leitura |
| E8 | S-4, S-5, D8 — e C-12, que ganhou o comando que o atende |
| E9 | R-1, R-2, R-3, **R-7**, **R-8**, L-2 — e P-17. R-7 não tinha etapa nenhuma nesta tabela até aqui |
| E10 | §7.1 — e **C-13**, que fechou P-20 e saiu na frente do `arca prepare` |
| E11 | D6 |

## Riscos que atravessam o plano

**P-6 continua aberto, e sucesso não o fecha.** O ramo de falha do `ocs-sr` nunca foi observado — por definição, execuções bem-sucedidas não o exercitam. No backup existem **dois** sinais independentes do código de saída, e não um: a conferência nativa que o Clonezilla faz por padrão (e que `-scs` desligaria, razão de ele ficar de fora — ver ADR-0004) e o `ocs-chkimg` explícito de B-9. **Na restauração não há segundo juiz do resultado**: se o `ocs-sr` devolver 0 ao falhar, o `if/then/else` de R-5 escreve `OK` sobre uma restauração quebrada. O que segura esse caso hoje é o Windows subir ou não, e o `arca resultado` diz isso na tela desde a E9 (§6.3).

> **Uma correção de letra, achada na E9 lendo o help inteiro.** Este parágrafo
> dizia *"na restauração não há segundo sinal"*, e há: `-scr`,
> `--skip-check-restorable-r`, desligaria uma conferência que o Clonezilla faz
> **por padrão** antes de restaurar, e a receita não o usa — do mesmo jeito que
> não usa `-scs`. O que a conferência dele responde é *"esta imagem é
> restaurável?"*, e não *"a restauração deu certo?"*. Ela pode fazer o `ocs-sr`
> desistir antes de tocar no disco, e isso é bom; o que ela não é, é um segundo
> juiz do resultado. O espírito do parágrafo estava certo e a letra estava
> errada, e a distinção é a mesma que o método nomeia: **conferir se a
> evidência fala sobre a pergunta.**

> **O juiz que falta respondeu, e só sobre esta operação.** A tela do §6.3 manda
> religar e conferir, e em 23/08/2026 o Windows subiu — este documento está
> sendo editado nele. Isso fecha a dúvida sobre a restauração de
> `2026-08-22_Apps`, e não fecha P-6: um êxito não exercita o ramo de falha, que
> é o que a pendência pergunta. O que a execução acrescenta é que o desenho
> funciona quando dá certo, e a próxima coisa a saber é o que ele faz quando dá
> errado — em VM, com falha forçada.

~~**O mecanismo de desfecho nunca rodou**~~ (P-16, achado na E3). Nenhuma das três receitas preservadas escreve `arca-fim.txt`, grava selo ou usa `if/then/else` — o que existia no dispositivo veio de trabalho manual de validação. O plano supunha que a E7 e a E9 confirmariam um mecanismo pronto; elas foram a **primeira execução** dele. **Rodou em 22/08/2026, e o ramo do sucesso funcionou inteiro**; o de falha continua sem rodar, e é P-6. O que continua valendo é a regra que o risco produziu: antes de tratar qualquer linha do §3 do PRD como medida, procurar o original em `recursos/capturas/` — cinco vezes ele não estava lá, e uma vez estava.

**A ordem permanente de boot desta máquina foi alterada, e não pelo ARCA**
(achado na E7). Em 21/08 — o backup que o §3.3 do PRD chama de validado — o
dispositivo estava **em primeiro** na `BootOrder`, o que explica o boot inteiro
sem passar por boot único. É o **quinto** caso do mesmo padrão, depois do
`ARCA_VEREDITO=`, do `arca-fim.txt`, do `set default` e do `498,7 GB`. A tabela
das oito capturas está no §3.1 do PRD, com a data de cada leitura — e datar as
capturas foi o que desfez uma contradição aparente entre duas ferramentas que,
lidas no mesmo dia, concordam.

Daí uma regra que vale além desta etapa: **antes de chamar duas medições de
contraditórias, confira se são do mesmo momento.** Foi o que quase custou uma
etapa aqui.

**E a ordem tem um terceiro dono, achado na E9: a imagem.** Uma restauração
devolve a ordem permanente ao que estava dentro dela — o BCD mora na partição EFI,
e a partição EFI é restaurada junto. Medido em 23/08/2026, e a evidência é
forte: a leitura de depois de religar é byte a byte a que a E2 tirou em 22/08
de manhã. O ADR-0009 arbitrava entre o ARCA e o Windows; este terceiro escreve
sem perguntar a nenhum dos dois, e P-20 tem de ser decidido sabendo disso
([ADR-0012](../docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)).

Daí a regra irmã da anterior, e ela pegou uma linha do §3.4 que tinha original:
**antes de tratar um par `antes`/`depois` como resposta, pergunte entre que
dois instantes ele foi tirado, e se a pergunta cabe dentro deles.** O par que
sustentava *"a restauração não mexe na NVRAM"* é de dentro de um boot só, e por
isso só podia falar do `ocs-sr`.

**A entrada de firmware é o ponto de falha mais caro.** Um erro do parser da E2 leva a máquina a bootar no lugar errado com uma receita armada. É a única etapa cujos testes precisam cobrir os dois idiomas do `bcdedit`.

**O relógio do Clonezilla está 3 h adiantado, permanentemente** (P-7). O selo existe para que ninguém precise saber disso.

## Fora de escopo

Incremental, agendamento, retenção, catálogo, interface gráfica, particionamento, BIOS legada, BitLocker, RAID, Storage Spaces — tudo conforme §2 do PRD. E `arca resultado` no logon (P-14) fica de fora até o uso pedir.
