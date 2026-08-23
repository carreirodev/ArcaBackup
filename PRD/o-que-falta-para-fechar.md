# O que falta para o ARCA se considerar fechado

23/08/2026, depois da etapa E10 — e **revisado no mesmo dia**, quando a E12
foi planejada e P-26 mudou de caminho. Complementa o
[PRD v5.1](PRD-ARCA-v5_1.md) e o [plano de etapas](implementation_stages.md);
onde este documento divergir deles, são eles que valem.

---

## A pergunta que este documento responde

**As doze etapas do plano fecharam.** Os oito comandos do §8 fazem trabalho, a
lista de *"chega na etapa X"* esvaziou na E10, todo requisito do §9 tem código, e
a suíte tem 760 testes. O ciclo inteiro rodou em hardware: backup armado e
colhido (22/08), restauração completa com o Windows voltando de dentro da imagem
(23/08), verificação armada (23/08) e um dispositivo criado do zero (23/08).

Isso responde *"foi construído?"*. Não responde *"está provado?"* — e é essa a
diferença que este documento existe para não deixar dissolver.

O §3.5 do PRD tem uma advertência que vale reler antes de qualquer coisa:

> Cinco vezes se descobriu que algo documentado como **fundação validada** tinha
> vindo do **trabalho de validação em volta dela**. O padrão se repete porque a
> evidência que sobra no dispositivo não distingue o que a receita escreveu do
> que uma pessoa escreveu depois.

Um app que se declara fechado porque as etapas acabaram é a sexta vez do mesmo
padrão, com outro nome.

---

## As três categorias, e só a primeira é falta

O que está em aberto se divide em três coisas, e misturá-las é o que faz uma
lista de pendências parecer maior ou menor do que é.

| | O que é | Conta como "falta"? |
|---|---|---|
| **Não medido** | O app faz, e ninguém viu fazer. São as pendências `P-*` | **Sim** |
| **Fora de escopo** | Decisões registradas de não fazer | Não |
| **Estado da mesa** | O que está conectado, o que foi commitado | Não é do app |

---

## 1. O que não foi medido

Seis pendências abertas. Estão listadas por **quanto custa a alguém se elas
estiverem erradas**, e não pela ordem em que nasceram.

### P-26 — um dispositivo preparado pelo ARCA nunca bootou

**Aberta na E10, em 23/08/2026. É a mais nova e a mais direta.**

O `arca prepare` produziu um dispositivo com o Clonezilla instalado, o `grub.cfg`
inerte e a entrada de firmware apontando para ele. Conferiu-se tudo o que se
confere sem reiniciar:

- a estrutura de partições **relida do disco** — `MbrType 7` e `12`, offsets
  idênticos aos da medição à mão, nenhuma partição ativa, unidade 4096;
- os quatro caminhos obrigatórios **dentro do pacote**, antes de extrair;
- o `set default` de volta em `live-default`;
- a entrada de boot **relida do `bcdedit`**, apontando para `partition=F:` e
  `\EFI\boot\bootx64.efi`.

**O que falta é o firmware honrar aquela entrada**, e nada do lado Windows
responde isso.

É a mesma forma de P-18, que a E4 abriu e o marco da E8 fechou: *o lado Windows
prova o que escreveu, e só o hardware prova que o firmware obedeceu.*

**Por que incomoda mais do que incomodava lá.** O ADR-0014 nomeia o modo de
falha deste comando: um dispositivo que não boota **só se descobre depois de o
Windows ter sido apagado**, porque é aí que alguém precisa dele. O `arca prepare`
existe para produzir algo que boota; enquanto P-26 estiver aberta, ele entrega
uma promessa conferida por leitura.

**Como fecha — e as duas primeiras respostas a esta pergunta estavam erradas.**

A resposta óbvia seria *"um `arca backup` no dispositivo novo"*. **Ele recusa**,
e a razão é o §4.5: a receita nomeia o disco pelo nome que o **Linux** lhe dá, o
ARCA o descobre lendo o `blkdev.list` de dentro de uma imagem, e um dispositivo
recém-preparado **não tem imagem nenhuma**.

O mesmo vale para os outros dois comandos que armam: `arca restore` e `arca
verify --completo` precisam de uma imagem que também não existe. **Nenhum dos
três funciona num dispositivo recém-nascido.**

Isso parte P-26 em duas metades:

| | O que prova |
|---|---|
| **(a)** o dispositivo boota | o `.efi`, o `grub.cfg` e o pacote instalado prestam |
| **(b)** a entrada de firmware que o ARCA criou leva a ele | o `/copy` e os dois `/set` produziram algo que o firmware honra |

**A segunda resposta foi mandar o usuário para o menu do Clonezilla** — F12,
primeiro backup pelo menu (§6.4), e daí em diante o ARCA. Ela não está errada
sobre os fatos, e continua sendo o caminho manual quando tudo o mais falhar. O
que ela é: **exatamente aquilo que este app existe para não precisar**, cobrado
logo na primeira vez que alguém usa um dispositivo novo. Custava dois reinícios e
cerca de quarenta minutos.

**A resposta certa é a E12.** `arca sondar` arma um boot único que não faz backup
nem restauração: roda `lsblk`, grava a saída no `ARCAVAULT` no mesmo formato do
`blkdev.list` que o §4.5 lê, e desliga. Um reinício, nenhuma tela do Clonezilla,
e o dispositivo passa a ter o oráculo que lhe faltava.

**E ela fecha as duas metades de uma vez.** O boot é o **único**, disparado pela
entrada que o `arca prepare` criou: se a máquina voltar com o desfecho escrito,
(a) e (b) estão respondidas juntas. Um F12 responderia só (a).

**Custa:** um reinício e o tempo de um boot do Clonezilla — que, aliás, **não
está medido neste repositório**, porque toda execução anterior tinha uma operação
longa depois dele. A E12 mede isso de graça.

**Risco: o menor de todos os marcos deste projeto.** A receita não tem `ocs-sr`,
logo não há `savedisk` nem `restoredisk`, e nada é escrito fora do `ARCAVAULT`.
O pior caso é a máquina parar num menu (§3.2, §4.4), que é chato e não destrói
nada.

Ver a seção **E12** do [plano de etapas](implementation_stages.md).

> **A tela do `arca prepare` dizia `Primeiro backup: arca backup <nome>`**, e
> isso era exatamente o que o §7 deste documento proíbe: uma tela afirmando o
> que o repositório não pode mostrar tendo acontecido. Corrigida em 23/08/2026 —
> ela passou a dizer que o primeiro backup é pelo menu, **por quê**, e que o F12
> daquele passo é o que responde P-26.
>
> **Essa correção vence quando a E12 fechar**, e a tela terá de dizer `arca
> sondar`. Fica registrado aqui para que a segunda versão não sobreviva ao
> motivo dela — a primeira quase sobreviveu.
>
> O defeito original é da própria E10 e tem a forma de sempre: peça nova
> encaixada em peça antiga que ninguém releu ao encaixar. A peça antiga ali é o
> §4.5, decidido na E6 e na E7.

### P-6 — o ramo de falha nunca rodou, em nenhuma das três receitas

**Aberta na E3. É a mais antiga e a mais grave.**

A pergunta literal é *"o `ocs-sr` devolve código diferente de zero quando
falha?"*, e o que depende dela é maior do que parece. **Nenhuma execução deste
projeto jamais escreveu um `FALHOU`:**

| Nunca aconteceu | Onde estaria |
|---|---|
| `ARCA_BACKUP=FALHOU` | §10.1 |
| `ARCA_RESTORE=FALHOU` | §10.2 |
| `ARCA_VERIFY=FALHOU` | §10.2.4 |
| `ARCA_VEREDITO=REPROVADA` | §10.1 e §10.2.4 |

Uma execução bem-sucedida não exercita o ramo de falha, **por definição** — e as
cinco que houve deram certo.

**O que acontece se estiver errado.** Se o `ocs-sr` sair com zero ao falhar, o
`if/then/else` de R-5 escreve `OK` sobre uma operação que não aconteceu. No
backup há **dois** sinais independentes (a conferência nativa do Clonezilla e o
`ocs-chkimg` de B-9), e o segundo pegaria. **Na restauração não há segundo juiz
do resultado**: o que segura o caso hoje é o Windows subir ou não, e o §6.3 diz
isso na tela desde a E9.

**Como fecha:** falha forçada, provavelmente em VM — um `restoredisk` apontado
para um disco que não serve, com o código de saída observado.

**Custa:** montar uma VM com Clonezilla e um disco de teste. É a pendência mais
cara desta lista, e a única cujo fechamento não acontece nesta mesa.

### P-25 — uma receita rodou e o rastro divergiu do que a string manda fazer

**Aberta no marco da E11, em 23/08/2026.** É a única vez neste projeto em que
isso aconteceu; todos os achados anteriores foram de documentação descrevendo o
que não tinha rodado.

A receita da verificação usa `>>` para **acrescentar** ao `arca-check.log`. O
`--dry-run` a imprimiu assim minutos antes de armar, e `recursos/ensaio-da-receita.sh`
prova que `>>` acrescenta num bash de verdade. Em hardware o arquivo saiu com
**uma** execução do `ocs-chkimg`, e o log do backup de 22/08 sumiu.

```text
antes  arca-check-2026-08-22_Apps.log ............ 3832 bytes · 1 marca · 1 abertura
depois arca-check-…-pos-verificacao.log .......... 4759 bytes · 1 marca · 1 abertura
                                        (append daria >7600 bytes e 2 de cada)
```

**Alguma coisa entre o redirecionamento e o disco truncou o arquivo, e não se
sabe o quê.**

**O que acontece se estiver errado.** Já aconteceu: perdeu-se o veredito que o
backup de 22/08 escreveu. O `>>` fica assim mesmo, com a razão trocada — ele não
compra a preservação, mas não abre a janela em que o `>` deixaria uma imagem boa
com o log em zero byte.

**Como fecha:** uma segunda verificação armada, comparada com esta.
**Custa:** um reinício e ~5 minutos.

### P-22 — uma afirmação de segurança lida da fonte possivelmente errada

**Aberta no marco da E9.** O `bcdedit /enum firmware` mostra a NVRAM do firmware,
ou o BCD do disco?

A pergunta nunca precisou de resposta até a restauração devolver a ordem
permanente de dentro da imagem (ADR-0012). Agora as duas possibilidades levam a
mundos diferentes:

- **se é a NVRAM**, a ordem está limpa de verdade e ligar com o SSD conectado
  sobe o Windows;
- **se é só o BCD**, a NVRAM pode continuar com o dispositivo à frente, a máquina
  continuaria bootando nele — **e o `arca status` diria que está tudo bem**.

O segundo caso é uma afirmação de segurança feita sobre uma leitura que não fala
da pergunta, que é o defeito que a revisão do marco da E8 já pegou uma vez
naquela mesma linha. **E a E10 aumentou o que ela vale**: se for o BCD, a
releitura de C-3 do `/remove` do `arca prepare` confirma um conserto feito sobre
o espelho.

**Como fecha:** religar com o SSD conectado, sem job armado e com o `grub.cfg`
inerte. Parando no Windows, a NVRAM acompanhou; parando no menu do Clonezilla,
não acompanhou.

**Custa:** um reinício. **Risco: nenhum** — o grub inerte garante que o pior caso
é um menu esperando alguém.

> **É o mais barato da lista, e hoje ele responde duas coisas de uma vez.** Com o
> dispositivo recém-preparado na mesa, esse mesmo reinício confere a promessa que
> a tela do `arca prepare` fez: *"a entrada de firmware existe e está FORA da
> ordem permanente — ligar a máquina continua subindo o Windows"*.

### P-23 — o log da restauração não cobre a operação inteira

**Aberta no marco da E9.** O `arca-restore.log` do marco tem 16.600 bytes e
começa no meio: uma passagem só do Partclone — a da última das quatro partições
—, nenhuma das outras três, e um `Ending /usr/sbin/ocs-sr` **sem** o `Starting`
correspondente.

**Por que importa:** o §6.3 aponta esse arquivo a quem colheu uma restauração e
quer saber o que aconteceu, e o que está lá **pode não cobrir a parte que
falhou**.

**Como fecha:** medir de novo na próxima restauração, e perguntar se o corte cai
sempre no mesmo lugar.
**Custa:** uma restauração, que é destrutiva. Não vale disparar só por isso.

### P-19 — quando o firmware reescreve a entrada

**Aberta na E8, estreitada na E9.** A primeira metade fechou pela negativa: o
firmware **não** reescreve a entrada em todo boot pelo dispositivo. O que não
fecha é datar a reescrita.

**Por que importa pouco:** a consequência operacional que interessa — a entrada
volta para a ordem permanente depois de um boot pelo dispositivo — está medida, e
C-13 a conserta desde 23/08.

**Como fecha:** um backup disparado por F12, com o `bcdedit` lido imediatamente
antes.
**Custa:** um backup. É a pendência menos urgente da lista.

---

## 2. As seis linhas do §5.5 sem original

Caso à parte, ligado a P-6 e mais concreto do que ela.

A tabela de desfechos possíveis do §5.5 tem **sete linhas**, e o marco da E8
produziu exatamente **uma**: selo batendo, `ARCA_FIM` presente, `ARCA_BACKUP=OK`,
veredito `APROVADA`.

As outras seis continuam sem original — o que quer dizer que o código que as
distingue nunca foi exercitado por um arquivo que o Clonezilla escreveu:

| Linha | Como se produziria |
|---|---|
| Selo bate, desfecho `FALHOU` | depende de P-6 |
| Selo bate, sem `ARCA_FIM` | desligar a máquina no meio da receita |
| Selo não bate (job fantasma) | colher com um `estado.json` de outro job |
| Sem linha de selo / selo repetido / sem marcador | truncar o `arca-fim.txt` |
| Sem `arca-fim.txt`, com job pendente | **rodou** em 23/08, por acidente (E11) |
| Sem `arca-fim.txt`, sem job pendente | rodar `arca resultado` numa mesa limpa |

> **A quinta rodou sem que ninguém planejasse.** Na primeira tentativa do marco
> da E11, quem estava na frente da tela desligou a máquina durante o menu do
> Clonezilla — e o `arca resultado` colheu a ausência de desfecho, nomeou as duas
> causas de C-12, desarmou e encerrou o job. Era a linha que mais tinha esperado.

Nenhuma delas é cara de produzir à mão, e **a mais valiosa é a do desfecho
`FALHOU`**, que é P-6 com outra roupa.

---

## 3. O que o código faz e nunca aconteceu de verdade

Nem tudo aqui é pendência numerada. São caminhos construídos, testados com
duplos, e que nenhuma execução real percorreu.

**As sete defesas de PR-5.** Nenhuma recusa do `arca prepare` disparou em
hardware: o disco fixo, o disco do sistema, o `%SystemDrive%`, a mídia
desconhecida, o disco pequeno demais, o somente-leitura e o índice inexistente.
São recusas — recusar não precisa de hardware para estar certo —, mas a mensagem
que sai na tela de cada uma nunca foi lida por ninguém.

**A recusa de PR-1 por SHA256 divergente.** O caso que a defesa existe para pegar
não aconteceu: o download bateu na primeira. Vale dizer que **o valor conferido
tem duas fontes independentes**, o que é mais do que a maioria das afirmações
deste projeto tem.

**O `arca prepare` num disco em branco.** As três execuções foram sobre um disco
com partições — a linha `(nenhuma particao — o disco esta em branco)` nunca
apareceu numa tela de verdade.

**O `arca prepare` numa máquina que nunca teve ARCA.** A segunda execução do
marco chegou perto: a entrada `ARCA` foi apagada antes, e o comando a criou do
zero. O que não se testou é uma máquina sem **nenhum** rastro do projeto.

**Um dispositivo ARCA que nunca dependeu de outro.** Vale registrar o que este
não é: o dispositivo da E10 foi feito do **zip baixado do SourceForge**, com o
SHA256 conferido contra o mirror do projeto — nada foi copiado do dispositivo
antigo, e o `arca prepare` não sabe que ele existe. O antigo serviu só como
**oráculo de comparação**, para provar que o que se instala é equivalente ao que
já funciona.

A dependência que sobra é outra e está acima, em P-26: hoje o **primeiro
backup** de um dispositivo novo precisa do menu do Clonezilla, porque o nome do
disco no Linux só existe dentro de uma imagem (§4.5). Ela não é do dispositivo
antigo — é da natureza do oráculo, e é exatamente o que a **E12** foi desenhada
para tirar do caminho: `arca sondar` produz o mesmo arquivo sem que exista imagem
nenhuma.

---

## 4. O que está fora de escopo, e não é falta

Registrado para que ninguém confunda decisão com pendência.

**Do §2 do PRD:** catálogo de imagens, rastreamento de série, backup incremental,
agendamento, retenção automática, interface gráfica, gerenciador de discos de uso
geral, BIOS legada, BitLocker, RAID, Storage Spaces.

**Decidido e adiado:**

- **P-14** — `arca resultado` rodando sozinho no logon. *"Começar sem, decidir com
  uso."*
- **`arca atualizar`** — o `arca prepare` instala **o executável que está
  rodando**, e isso congela o ARCA do dispositivo no momento em que ele foi
  preparado. É o que §4.1 quer (o julgamento não vem de dentro da imagem), e tem
  consequência: **copiar o binário para o `ARCABOOT` é pré-requisito de todo
  marco que mude o formato do `estado.json`**, e não há comando que faça isso
  sozinho. A E11 pagou por essa lição uma vez.
- **Reinstalar o Clonezilla sem apagar as imagens** — o `arca prepare` começa
  reescrevendo a tabela de partição, e a tela diz isso a quem aponta um
  dispositivo que já existe.
- **GPT+ESP** — o ADR-0014 manda resistir, e a razão não mudou: trocar um esquema
  medido por um suposto, num lugar cujo modo de falha só aparece depois de o
  Windows ter sido apagado.
- **Uma segunda coluna no `arca list`**, separando o veredito do `ocs-chkimg` da
  conferência de V-1 (ADR-0016).

**P-7 não é pendência**: é um fato registrado — o relógio do Clonezilla roda 3 h
adiantado, permanentemente. Existe na lista para a próxima pessoa que for
comparar datas.

---

## 5. Estado da mesa, e não do app

**A suíte está inteira verde**: 760 testes, zero vermelhos, com o dispositivo
que a E10 criou sozinho na mesa.

Chegar aí custou uma correção que vale registrar aqui, porque ela é sobre o que
este documento mede. **Cinco testes de hardware — das etapas E1, E4, E7 e E11 —
descreviam o dispositivo antigo achando que descreviam um dispositivo**: que o
`ARCAVAULT` tem imagens, que há três cópias armadas de agosto ao lado do
`grub.cfg`, que o `grub.cfg` é byte a byte a captura do repositório.

Nada disso é verdade num dispositivo recém-nascido. Os que dependem de imagem
passaram a **sair cedo dizendo por quê**; os do `grub.cfg` passaram a aceitar os
**dois** inertes conhecidos — o do ISO e o do zip —, com o teste da E10 provando
que são equivalentes. Nenhum foi afrouxado.

**Nada da E10 foi commitado.**

---

## 6. Uma ordem que faz sentido

Do mais barato para o mais caro, e cada linha diz o que se ganha.

| # | O quê | Custa | Fecha |
|---|---|---|---|
| 1 | Religar com o SSD conectado, sem job armado | 1 reinício, risco zero | **P-22**, e confere a promessa da tela do `arca prepare` |
| 2 | **F12** no dispositivo novo, até o menu do Clonezilla | 1 reinício, risco zero | **P-26 (a)** — o dispositivo boota |
| 3 | **E12** — escrever o `arca sondar` e rodá-lo | 1 etapa + 1 reinício, risco quase zero | **P-26 inteira**, e põe o dispositivo em uso |
| 4 | Segunda verificação armada | 1 reinício, ~5 min | **P-25** |
| 5 | Produzir as seis linhas do §5.5 à mão | tempo, sem risco | seis casos do §5.5 |
| 6 | Falha forçada em VM | montar uma VM | **P-6**, e com ele a linha `FALHOU` |
| 7 | Próxima restauração, com o log medido | uma restauração | **P-23** |
| 8 | Backup por F12, com o `bcdedit` antes | um backup | **P-19** |

**Os três primeiros valem mais do que os cinco últimos juntos.** O **1** e o
**2** são reinícios de risco zero que respondem afirmações que o ARCA já imprime
na tela; o **3** é a única coisa que separa a E10 de estar provada — e é o que o
dispositivo novo precisa para servir para alguma coisa.

> **O passo 2 continua valendo mesmo com o 3 na fila, e por uma razão de ordem.**
> A E12 fecha (a) e (b) de uma vez, mas ela custa uma etapa **escrita antes** de
> alguém saber se o dispositivo boota. Um F12 responde (a) por um reinício: se o
> menu do Clonezilla não aparecer, o problema é do dispositivo e não da receita
> nova — e descobrir isso depois de escrever a etapa é a ordem cara.
>
> **O passo 2 e o passo 8 são o mesmo reinício, se você quiser.** P-19 pede um
> backup disparado por F12 com o `bcdedit` lido imediatamente antes; o passo 2
> pede um F12. Lendo o `bcdedit` antes de apertar F12, um reinício responde as
> duas.

---

## 7. O critério

Este documento existe para que a resposta a *"o app está fechado?"* deixe de
depender de quem responde. O critério proposto:

> **O ARCA se considera fechado quando nenhuma tela dele afirmar algo que este
> repositório não possa mostrar tendo acontecido.**

Por esse critério, hoje faltam **duas** coisas, e não sete:

- **P-26**, porque a tela do `arca prepare` diz *"Dispositivo pronto"* e ninguém
  bootou nele — e ela ganhou caminho barato em 23/08/2026: a **E12**
  (`arca sondar`) a fecha inteira com um reinício, sem tocar em disco nenhum.
  Deixou de ser a pendência que custa quarenta minutos e passou a ser a que custa
  uma etapa escrita;
- **P-6**, porque a tela do `arca resultado` sabe dizer `FALHOU` e nunca disse.

As outras quatro são perguntas honestas sobre o mundo — como o firmware se
comporta, o que o `ocs-chkimg` faz com um descritor, de onde o `bcdedit` lê. O
app não afirma nada sobre elas que dependa da resposta, e é por isso que ele pode
conviver com elas abertas.

---

*Atualizar quando qualquer pendência fechar, e apagar quando as duas do §7
fecharem.*
