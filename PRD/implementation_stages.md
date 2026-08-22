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
| E5 | Estado e selo | II | ⬜ | — |
| E6 | Pré-voo | III | ⬜ | — |
| E7 | Armar e disparar | III | ⬜ | — |
| E8 | Colher o desfecho | III | ⬜ | — |
| E9 | Restauração | IV | ⬜ | — |
| E10 | `arca prepare` | IV | ⬜ | — |
| E11 | `arca verify` | IV | ⬜ | — |

Uma etapa só é marcada ✅ quando o **Pronto quando** ou o **Entrega** da sua seção estiver cumprido de fato — não quando o código foi escrito. As duas etapas com marco em hardware (E7 e E9) exigem a execução real para fechar.

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
| 5 | **Destino divergente é permitido**, com confirmação que nomeia o disco de destino. Recusa dura só se o destino for **menor** que a origem | ~~`-k0` num disco menor corrompe em vez de falhar.~~ **A premissa está errada** — ver P-17: o help do `ocs-sr` diz que o Clonezilla confere o tamanho do destino **por padrão** e desiste se for menor, e que `-icds` é quem desliga isso. A recusa do ARCA continua valendo como defesa em profundidade, mas não é a única. A E9 resolve. Em disco novo, `-iefi` não acha entrada correspondente e o `bcdboot` volta a ser necessário |
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

## Fase III — Backup ponta a ponta

### E6 · Pré-voo

Tudo que §5.2 mostra antes da confirmação: nome válido (B-2) e ainda não usado, inclusive contra resíduo (B-3); espaço pelo maior entre `maior imagem × 1,3` e `em uso × 0,45`, com faixa de aviso entre 1× e 1,5× disso (B-4); Inicialização Rápida, oferecendo `powercfg /h off` (B-5); `chkdsk /scan`, oferecendo agendar `/f` (B-6).

**Cobre**: B-2, B-3, B-4, B-5, B-6
**Entrega**: o diálogo de §5.2 inteiro, terminando **antes** de armar.

### E7 · Armar e disparar

Gravar a receita no `grub.cfg`, marcar o boot único **sem tocar na ordem permanente** (C-5), criar ou migrar a entrada de firmware (C-4), recusando `Removable Media` (C-6). Confirmação por texto digitado, nunca por `s` (S-2). Aviso de remover o SSD antes de religar, antes do reinício (C-9).

**Cobre**: C-4 (migração), C-5, C-9, S-2
**Marco em hardware**: primeiro backup completo disparado pelo ARCA, sem uma única tela.

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

## Fase IV — O resto

### E9 · Restauração

Só começa depois do marco da E8. Lista no Windows, com a escolha antes do ponto sem volta (R-1); conferência do destino contra `disk` e `blkdev.list` da própria imagem (R-2); nome da imagem digitado por extenso (R-3). Destino divergente segue a decisão 5: passa com confirmação que nomeia o disco, e é recusado se for menor que a origem.

**Cobre**: R-1, R-2, R-3
**Marco em hardware**: restauração completa disparada pelo ARCA. Depois disto, o projeto está funcionalmente pronto.

### E10 · `arca prepare`

Exige a FAT32 vazia de ≥ 1 GB já criada — o ARCA não particiona (§7.1). Baixa o Clonezilla na versão fixada, confere contra o SHA256 embutido, extrai, instala o ARCA no `ARCABOOT`, migra a entrada de firmware. `--iso <caminho>` para offline, que é o que salva quando a máquina que precisa preparar o dispositivo é a que está sem Windows.

Fica tarde de propósito: o dispositivo atual já existe, preparado à mão. Esta etapa serve ao **segundo** dispositivo.

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
| E7 | C-4, C-5, C-9, S-2 |
| E8 | S-4, S-5, D8 |
| E9 | R-1, R-2, R-3 |
| E10 | §7.1 |
| E11 | D6 |

## Riscos que atravessam o plano

**P-6 continua aberto, e sucesso não o fecha.** O ramo de falha do `ocs-sr` nunca foi observado — por definição, execuções bem-sucedidas não o exercitam. No backup existem **dois** sinais independentes do código de saída, e não um: a conferência nativa que o Clonezilla faz por padrão (e que `-scs` desligaria, razão de ele ficar de fora — ver ADR-0004) e o `ocs-chkimg` explícito de B-9. **Na restauração não há segundo sinal**: se o `ocs-sr` devolver 0 ao falhar, o `if/then/else` de R-5 escreve `OK` sobre uma restauração quebrada. O que segura esse caso hoje é o Windows subir ou não.

**O mecanismo de desfecho nunca rodou** (P-16, achado na E3). Nenhuma das três receitas preservadas escreve `arca-fim.txt`, grava selo ou usa `if/then/else` — o que existe no dispositivo veio de trabalho manual de validação. O plano supunha que a E7 e a E9 confirmariam um mecanismo pronto; elas são a **primeira execução** dele. E o padrão já se repetiu duas vezes: antes de tratar qualquer linha do §3 do PRD como medida, procurar o original em `recursos/capturas/`.

**A entrada de firmware é o ponto de falha mais caro.** Um erro do parser da E2 leva a máquina a bootar no lugar errado com uma receita armada. É a única etapa cujos testes precisam cobrir os dois idiomas do `bcdedit`.

**O relógio do Clonezilla está 3 h adiantado, permanentemente** (P-7). O selo existe para que ninguém precise saber disso.

## Fora de escopo

Incremental, agendamento, retenção, catálogo, interface gráfica, particionamento, BIOS legada, BitLocker, RAID, Storage Spaces — tudo conforme §2 do PRD. E `arca resultado` no logon (P-14) fica de fora até o uso pedir.
