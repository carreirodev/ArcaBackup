# ARCA — Etapas de implementação

Plano de execução derivado do [PRD v5.1](PRD-ARCA-v5_1.md). O PRD diz **o que** o ARCA é; este documento diz **em que ordem** ele é construído e **como se sabe** que cada pedaço está pronto.

Vocabulário canônico em [CONTEXT.md](../CONTEXT.md).

## Progresso

| Etapa | O que entrega | Fase | Status | Concluída em |
|---|---|---|---|---|
| E0 | Fundação executável | I | ⬜ | — |
| E1 | Descoberta do dispositivo e das imagens | I | ⬜ | — |
| E2 | Leitura do firmware | I | ⬜ | — |
| E3 | Geração e validação da receita | II | ⬜ | — |
| E4 | Desarmar | II | ⬜ | — |
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
| 5 | **Destino divergente é permitido**, com confirmação que nomeia o disco de destino. Recusa dura só se o destino for **menor** que a origem | `-k0` num disco menor corrompe em vez de falhar. Em disco novo, `-iefi` não acha entrada correspondente e o `bcdboot` volta a ser necessário |
| 6 | **Clonezilla com versão fixada e SHA256 embutido no binário do ARCA**, nunca baixado | Cópia do pacote usado fica no `ARCAVAULT`. `--iso <caminho>` para instalação offline |
| 7 | **`--dry-run` é flag de primeira classe** em todo comando que arma | A armadilha registrada no PRD (`--dry-run` virou execução real) é exatamente o que C-7 previne. Os dois andam juntos, na E0 |

## Correções a aplicar no PRD

Aplicar **antes** da E3, que transcreve as receitas para código.

| # | Correção |
|---|---|
| D1 | `-batch` aparece na fundação §3.2 mas some de B-8 e §10.1. **Adotado: `-batch` nas duas receitas**, alinhando à fundação medida. Confirmar na primeira execução real pelo ARCA |
| D2 | §10.2 usa `$LOG` e `$NOME` sem definir. Fixar `LOG="/home/partimag/ARCA-LOGS/$NOME"`, igual à de backup — o `ARCAVAULT` sobrevive à restauração do `nvme0n1` |
| D3 | O "princípio P1" é citado em §2 e §7.1 e **nunca enunciado**. Escrever: o ARCA não executa a operação mais destrutiva do fluxo |
| D4 | Job fantasma e R-6 descrevem uma ameaça que §4.1 já elimina. Reescrever como **risco herdado**: só imagens feitas antes de o ARCA sair do `C:` carregam estado dentro de si. O selo cobre de qualquer forma |
| D5 | S-1 conflita com B-5 e B-6, que escrevem no disco de origem. Delimitar S-1 a **acesso raw ao dispositivo** |
| D6 | `arca list` e `arca verify` não têm requisito nenhum. Ganham requisitos nas E1 e E11 |
| D7 | "Um dispositivo por vez" é regra sem ID. Vira requisito: **recusar se houver mais de um `ARCAVAULT` ou `ARCABOOT` conectado** |
| D8 | Não existe requisito para `arca-fim.txt` ausente — o desfecho de toda falha silenciosa. Vira tabela de estados terminais na E8 |
| D9 | Cabeçalho diz "Versão 0.5", título diz "v5", arquivo diz `v5_1`. Escolher uma |

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

Parser de `bcdedit /enum` **por valor, não por nome de campo** — só `identificador` sai traduzido (fundação §3.1). Localizar a entrada `ARCA`; não havendo, reconhecer a legada `Clonezilla` (C-4). Distinguir `External hard disk media` de `Removable Media`, que o `bcdedit` rejeita em silêncio respondendo "êxito" (C-6).

Testes unitários sobre saídas capturadas em português e em inglês. Este parser é o único ponto do sistema onde uma leitura errada leva a máquina a bootar no lugar errado.

**Cobre**: C-3, C-4 (detecção), C-6
**Entrega**: `arca status` — diagnóstico não destrutivo: dispositivo, imagens, entrada de firmware, estado do job. Comando novo, a acrescentar em §8.

## Fase II — A receita (escreve em arquivo, não arma nada)

### E3 · Geração e validação da receita

Montar as duas receitas exatamente como as validadas em hardware, com `-batch` (D1) e o `LOG` da restauração corrigido (D2). Backup com nome e disco embutidos, **sem `ask_user`** (B-7), flags fixas de B-8, chamada explícita ao `ocs-chkimg` com saída redirecionada (B-9). Restauração com `-k0 -iefi -j2`, sem `-g auto` (R-4), e `if/then/else` — nunca `;`, que faria uma falha deixar o mesmo rastro de um sucesso (R-5).

Validador C-2 como porteiro: rejeita pipes, aspas desbalanceadas e nomes inseguros (B-2) **antes** de qualquer gravação.

Os testes desta etapa comparam a receita gerada, caractere a caractere, com a que rodou no hardware. É o ponto de verificação mais importante do projeto: daqui para frente tudo confia que esta string está certa.

**Cobre**: C-2, B-2, B-7, B-8, B-9, R-4, R-5, S-4 (a receita é quem grava o desfecho)
**Entrega**: `arca backup <nome> --dry-run` imprime a receita completa e não toca em nada.

### E4 · Desarmar

Reescrever o `grub.cfg` para o estado inerte — o menu normal do Clonezilla, que é o que §6.3 pressupõe existir quando o Windows não sobe — e limpar qualquer marca de boot único residual. Incondicional, idempotente, **sem consultar estado nenhum** (C-1), e é o primeiro passo de todo comando.

**Cobre**: C-1
**Pronto quando**: rodar duas vezes seguidas dá o mesmo resultado, e o dispositivo boota no menu normal do Clonezilla depois.

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
| E4 | C-1 |
| E5 | R-6, S-6 |
| E6 | B-2, B-3, B-4, B-5, B-6 |
| E7 | C-4, C-5, C-9, S-2 |
| E8 | S-4, S-5, D8 |
| E9 | R-1, R-2, R-3 |
| E10 | §7.1 |
| E11 | D6 |

## Riscos que atravessam o plano

**P-6 continua aberto, e sucesso não o fecha.** O ramo de falha do `ocs-sr` nunca foi observado — por definição, execuções bem-sucedidas não o exercitam. No backup existe segundo sinal independente: o `ocs-chkimg` examina a imagem gravada e não depende do código de saída. **Na restauração não há segundo sinal**: se o `ocs-sr` devolver 0 ao falhar, o `if/then/else` de R-5 escreve `OK` sobre uma restauração quebrada. O que segura esse caso hoje é o Windows subir ou não.

**A entrada de firmware é o ponto de falha mais caro.** Um erro do parser da E2 leva a máquina a bootar no lugar errado com uma receita armada. É a única etapa cujos testes precisam cobrir os dois idiomas do `bcdedit`.

**O relógio do Clonezilla está 3 h adiantado, permanentemente** (P-7). O selo existe para que ninguém precise saber disso.

## Fora de escopo

Incremental, agendamento, retenção, catálogo, interface gráfica, particionamento, BIOS legada, BitLocker, RAID, Storage Spaces — tudo conforme §2 do PRD. E `arca resultado` no logon (P-14) fica de fora até o uso pedir.
