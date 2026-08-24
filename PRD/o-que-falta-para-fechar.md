# O que falta para o ARCA se considerar fechado

23/08/2026, depois da etapa E10 — revisado no mesmo dia, quando a E12 foi
planejada, e **revisado outra vez em 24/08/2026, quando ela rodou e P-26, P-25,
P-22 e P-28 fecharam** — esta última nascida e fechada no mesmo dia. Complementa
o [PRD v5.1](PRD-ARCA-v5_1.md) e o [plano de etapas](implementation_stages.md);
onde este documento divergir deles, são eles que valem.

---

## A pergunta que este documento responde

**As treze etapas do plano fecharam.** Os nove comandos do §8 fazem trabalho, a
lista de *"chega na etapa X"* esvaziou na E10, todo requisito do §9 tem código, e
a suíte tem 838 testes. O ciclo inteiro rodou em hardware: backup armado e
colhido (22/08), restauração completa com o Windows voltando de dentro da imagem
(23/08), verificação armada (23/08), um dispositivo criado do zero (23/08) e —
**em 24/08 — esse dispositivo bootando sozinho pela entrada de firmware que o
próprio ARCA criou nele**.

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


**Três pendências abertas**, e eram cinco de manhã. Em 24/08/2026 fecharam
P-26, P-25 e **P-22** — e a última abriu **P-28**, que **fechou no mesmo dia**,
sete horas depois. Estão listadas por **quanto custa a alguém se elas estiverem
erradas**, e não pela ordem em que nasceram.

> ### ~~P-26~~ — fechada em 24/08/2026, e o que ela custou foi um reinício
>
> **Aberta na E10 e fechada no marco da E12**, inteira e de uma vez. `arca
> sondar` armou às 14:56:55 no dispositivo que o `arca prepare` criou — vazio de
> imagens —, a máquina bootou, o `lsblk` rodou sozinho e ela desligou. A colheita
> saiu `concluida`, com `ARCA_PROBE=OK` e o selo `354da624e7fa0d21` batendo.
>
> **As duas metades juntas**, e o que as junta é o `arca status` de minutos
> antes: `1 entrada(s), nenhuma para o dispositivo · so o boot unico leva a ele`.
> Com a entrada fora da ordem permanente não havia outro caminho até o
> dispositivo — (a) ele boota, e (b) a entrada que o ARCA criou leva a ele. Um
> F12 teria respondido só (a), e é a distinção que a própria P-26 fazia.
>
> **As duas primeiras respostas a esta pergunta estavam erradas**, e ficam
> registradas: um `arca backup` no dispositivo novo **recusa** (§4.5), e mandar o
> usuário para o menu do Clonezilla era exatamente aquilo que este app existe
> para não precisar — dois reinícios e cerca de quarenta minutos.
>
> **E fechou P-27 junto**: as flags reconstruídas do `lsblk` foram aceitas por
> aquele util-linux, e a árvore saiu em ASCII, o que diz que o `-i` funcionou.

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

> ### O que ela **deixou** de ser, em 24/08/2026
>
> A tabela acima diz *"nenhuma execução deste projeto jamais escreveu um
> `FALHOU`"*, e isso deixou de ser verdade — **falta uma linha nela**:
>
> | Aconteceu | Onde |
> |---|---|
> | `ARCA_PROBE=FALHOU` | §10.2.5, em 24/08/2026 |
>
> Uma sondagem foi armada com uma coluna inventada no `lsblk`, e o dispositivo
> voltou com `ARCA_PROBE=FALHOU` e `lsblk: unknown column: FLAGQUENAOEXISTE`
> dentro do próprio `blkdev.list`. O `arca resultado` reportou a falha e saiu
> com código 1; o `arca backup --dry-run` seguinte disse `POR DETERMINAR`. **As
> duas concordaram**, que é exatamente o que o `;` teria tornado contraditório.
>
> **P-6 continua aberta, e o sujeito é a razão.** Ela pergunta se o **`ocs-sr`**
> devolve código diferente de zero ao falhar, e quem falhou aqui foi o `lsblk`.
> Nenhuma resposta de um fala pelo outro.
>
> **O que mudou é o que sobrou dentro dela.** Antes, P-6 carregava duas coisas
> juntas: *"a forma de R-5 funciona?"* e *"o `ocs-sr` coopera?"*. A primeira está
> respondida em hardware — o `if` toma os dois ramos, e a tela de falha existe e
> imprime. A segunda é a que continua custando uma VM.

> ### ~~P-25~~ — fechada em 24/08/2026, e o redirecionamento era inocente
>
> **Aberta no marco da E11, em 23/08/2026**, quando uma receita rodou e o
> rastro pareceu divergir do que a string mandava fazer. A segunda verificação
> armada da `2026-08-22_Apps` mediu, e a resposta é curta: **cada verificação
> substitui o `arca-check.log`, e o `>>` não tem nada com isso.**
>
> O que mudou de método foi guardar a receita **gravada** —
> `recursos/capturas/grub-verificacao-2026-08-24.cfg`, copiada do dispositivo
> antes de colher. É a primeira captura da receita de verificação que de fato
> rodou, e ela tem o `>>`.
>
> ```text
> antes   4759 bytes · SHA256 0ebf57a0…05bdf843 · mtime 23/08
> depois  4759 bytes · SHA256 0ebf57a0…05bdf843 · mtime 24/08 13:32:54
>         (append daria ~9500 bytes)
> ```
>
> **Escreveu, e escreveu por cima.** O `arca-fim.txt` da mesma receita — selo
> `b668820c0a23ab5f`, o mesmo que o `arca resultado` imprimiu — leva o
> **mesmo `mtime` ao segundo**, o que prende a escrita a esta execução e não a
> outra. E o conteúdo saiu byte a byte igual ao de 23/08: duas execuções do
> `ocs-chkimg` sobre a mesma imagem dão o mesmo arquivo, o que explica por que
> o de 23/08 parecia o antigo.
>
> **E o `>>` chega ao `ocs-chkimg`**, o que até aqui só o `--dry-run` dizia.
> Comparando os dois arquivos de 23/08 byte a byte, o bloco de relatório de
> 927 bytes cai no **meio** quando a receita usa `>` e no **fim** quando usa
> `>>` — que é o efeito de `O_APPEND`. Quem esvazia o arquivo age **antes** do
> primeiro byte, e não entre o redirecionamento e o disco.
>
> **O achado de tabela: todo `arca-check.log` de backup tem um buraco.** Com
> `>`, os 927 bytes do relatório são escritos **por cima** da saída do
> partclone — o log de 22/08 perdeu `Starting to check image`, `File system`,
> `Device size` e o resto do progresso, e sobrou o pedaço cortado no meio da
> palavra: `maining: 00:00:00Ave. Rate:`. O mesmo padrão está no fixture do
> `ARCA-TESTE-03` em `src/imagens.rs`. **O veredito sobrevive** — ele é a
> última linha, escrita pelo bash —, e é ele que o `arca list` lê. O que se
> perde é diagnóstico de uma reprovação, e nenhuma tela promete isso.
>
> **O que sobra não é pendência**: *por que* o Clonezilla esvazia o arquivo é
> curiosidade sobre o `ocs-chkimg`, e nenhuma tela do ARCA afirma nada que
> dependa da resposta. O `>>` fica, com a razão trocada — ele não compra a
> preservação, mas não abre a janela em que o `>` deixaria uma imagem boa com
> o log em zero byte.

> ### ~~P-22~~ — fechada em 24/08/2026, e quem respondeu foi o firmware
>
> **Aberta no marco da E9**, e a pergunta era se o `bcdedit /enum firmware`
> mostra a NVRAM ou o BCD do disco. **É a NVRAM.**
>
> O experimento foi o que estava escrito aqui — religar às 17:11 com o SSD
> conectado, sem job armado, `grub.cfg` conferido inerte byte a byte — e a
> máquina foi **direto ao Windows**. Isso responderia só a metade operacional,
> e ela é a que importava: a linha `Ordem de boot` do `arca status` prevê onde
> a máquina boota.
>
> **O que fechou a pergunta literal apareceu sozinho no arquivo.** Entre as
> duas leituras, sem que ninguém escrevesse nada, o `displayorder` foi de duas
> entradas para cinco:
>
> ```text
> 17:11  {bootmgr} · {f4057bd3} ARCA
> 17:26  {bootmgr} · {f4057bd3} ARCA · UEFI:CD/DVD Drive
>                                    · UEFI:Removable Device
>                                    · UEFI:Network Device
> ```
>
> As três são **classes de dispositivo que o firmware enumera no POST**. Não
> têm `device` nem `path` — só `description` —, e nada no BCD as originaria: o
> Windows não teria o que espelhar. Elas entraram na ordem por causa de um
> reinício, logo **o `bcdedit` imprime conteúdo que só existe na NVRAM**.
>
> **Cai junto a dúvida que o ADR-0013 tinha acrescentado:** C-13 conserta o
> firmware, e não um espelho dele. E a promessa da tela do `arca prepare` —
> *"ligar a máquina continua subindo o Windows"* — tem agora um religar de
> verdade por trás.
>
> **A expectativa que entrou no experimento estava errada.** A análise que o
> precedeu apostava no menu do Clonezilla, pelo argumento de que a
> `{687478f2}` sumira numa restauração com `-iefi` e uma entrada da NVRAM não
> some por causa disso. O que faltava era saber que o firmware **reconstrói**
> entradas em POST — que é o que este mesmo reinício mediu. Ver
> [ADR-0020](../docs/adr/0020-o-bcdedit-enum-firmware-le-a-nvram.md).

> ### ~~P-28~~ — aberta e fechada em 24/08/2026, e o firmware apagou a testemunha
>
> **Ela é o que P-22 achou pelo caminho, e viveu sete horas.** O código foi
> consertado sem esperar a medição
> ([ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md)),
> e a medição chegou às 18:47 do mesmo dia.
>
> As três entradas que o firmware acrescentou não declaram alvo nenhum, e o ARCA
> as lia como **não levam ao dispositivo** — a resposta tranquilizadora.
> `UEFI:Removable Device` é a classe que boota o primeiro dispositivo removível,
> e o `ARCABOOT` é um SSD USB.

> **Os três casos, medidos em duplo** sobre a captura real do religar, passada
> pelo `montar` de verdade:
>
> | | A ordem | O que a tela dizia | Aviso |
> |---|---|---|---|
> | **A** | como estava | `dispositivo em 2o de 5 · Windows Boot Manager vem antes` | não sai, e está certo |
> | **B** | `UEFI:Removable Device` no topo | `dispositivo em 3o de 5 · UEFI:Removable Device vem antes` | **não saía** |
> | **C** | entrada `ARCA` fora da ordem | `4 entrada(s), nenhuma para o dispositivo · so o boot unico leva a ele` | **não saía** |
>
> O **B** é a falha como ela nasceu descrita: correta ao pé da letra — aquela
> entrada **está** antes —, e sem o parágrafo de perigo, que morava só no ramo
> em que o dispositivo é o primeiro. **O C não estava escrito em lugar nenhum, e
> é pior:** ali a tela não engolia um aviso, ela **afirmava** — e o estado que o
> produz é o que o `arca prepare` deixa, mais um religar.
>
> **É a terceira forma da mesma falha.** O ADR-0009 pegou a que procurava a
> entrada *pelo nome* — *"diria 'o Windows vem antes' e engoliria o aviso"* —, e
> C-6 pegou a que confiava no nome que o `bcdedit` devolve. Aqui é a **ausência**
> de alvo virando segurança, que é o que `viu_o_gerenciador` existe para não
> deixar acontecer no bloco vizinho.
>
> #### O conserto veio antes da medição, e virou C-14
>
> O julgamento passou a ter **três estados** — `Leva`, `NaoLeva`, `NaoSeSabe` —,
> e `NaoSeSabe` não vale como segurança em lugar nenhum. A regra **não afirma
> nada sobre este firmware**: ela deixa de afirmar, que é a forma de
> `viu_o_gerenciador`. Por isso não precisou esperar o reinício.
>
> **O discriminante é `alvo: None`, e é ele que impede o ruído.** O `{bootmgr}`
> aponta para `partition=\Device\HarddiskVolume1`: alvo concreto, que só não dá
> para conferir por letra. Fosse a regra por letra, o aviso sairia em toda tela
> — e um aviso que dispara sempre é o mesmo que não avisar.
>
> **Pegou três telas**: o `arca status` nos dois ramos; o `arca restore`, com um
> quarto estado em `OrdemDeBoot`; e o `arca prepare`, cuja tela de fim prometia
> *"ligar a maquina continua subindo o Windows"* em **texto fixo**, derivado de
> um fato só — a entrada do ARCA saiu da ordem —, sem olhar quem ficou nela.
> Esse era o furo irmão, e ninguém o tinha listado.
>
> #### A medição, às 18:39 — e o método não foi o F12
>
> Com o `grub.cfg` conferido inerte byte a byte (`4b33da61…9f47aa3d`) e sem job
> armado, a `{6cc093dc}` foi promovida ao **topo** da ordem à mão. **O F12 ficou
> de fora de propósito**: ele mede a *classe* que o menu do firmware oferece, e a
> pergunta é sobre a *entrada na `displayorder`*, que é o objeto que a tela lê.
> Sujar a ordem e desfazer no fim é o método do ADR-0013.
>
> A tela nova saiu **em hardware** pela primeira vez — o cenário B fora do
> fixture, com o aviso inteiro. E a máquina, reiniciada com o SSD conectado,
> **subiu o Windows**.
>
> #### E o que apareceu depois do boot vale mais do que a resposta
>
> Às 18:47, **sem o ARCA ter escrito nada** — o `arca resultado` não chegou a
> rodar, e C-13 não entrou —, o `bcdedit /enum firmware` é **byte a byte** a
> captura das 17:11:50 (`89ca7ad1…7b8df3b9`):
>
> ```text
> 17:26  {bootmgr} · {f4057bd3} ARCA · UEFI:CD/DVD · UEFI:Removable · UEFI:Network
> 18:39  UEFI:Removable · {bootmgr} · {f4057bd3} ARCA · UEFI:CD/DVD · UEFI:Network
> 18:47  {bootmgr} · {f4057bd3} ARCA
> ```
>
> As três sumiram **inteiras**: nem na ordem, nem como bloco. O firmware
> reescreveu o `displayorder` no POST, removeu as três e devolveu o `{bootmgr}`
> ao topo — restaurando exatamente o estado anterior.
>
> **Duas leituras do mesmo desfecho, e esta medição não as separa**: ou a entrada
> foi *tentada* e não alcançou o `ARCABOOT`, ou foi *descartada antes de ser
> tentada*, na mesma reconstrução que apagou as três. **Para o efeito
> operacional dá no mesmo, e é o que P-28 cobrava**: ela não desvia o boot.
>
> A evidência antiga favorece a primeira: o firmware desta placa enumera este SSD
> como **disco**, não como removível — foi ele quem criou a `{687478f2}` `UEFI OS`
> apontando para `partition=R:` —, e o Windows o classifica igual: não há
> `AVISO (C-6)` em tela nenhuma, e o `bcdedit` aceitou `partition=R:`, o que C-6
> diz não acontecer com mídia removível.
>
> #### O que ela deixa para trás
>
> **No código, nada muda** — C-14 foi escrito para não depender desta resposta, e
> não dependeu. O aviso continua o brando, com uma razão a mais para existir: uma
> entrada que **some do arquivo** entre a leitura e o reinício é ainda menos base
> para afirmar segurança.
>
> **E duas defesas que já existiam ficam nomeadas**, porque eram melhores do que
> o *"as três estão em 3º, 4º e 5º"* que este documento usava: **C-13 protege por
> construção** (`/addfirst {bootmgr}` põe o Windows na frente de tudo, com ou sem
> alvo declarado), e **o `arca restore` nunca silenciou** — o ramo brando já
> mandava remover o SSD.

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

**Duas das outras seis já rodaram**, e nenhuma das duas foi planejada como
exercício da tabela — o código que distingue as quatro restantes continua sem ter
sido exercitado por um arquivo que o Clonezilla escreveu:

| Linha | Como se produziria |
|---|---|
| ~~Selo bate, desfecho `FALHOU`~~ | **rodou** em 24/08, com uma coluna inventada no `lsblk` (E12). O que continua sem original é o `FALHOU` de uma operação que **grava**, e esse depende de P-6 |
| Selo bate, sem `ARCA_FIM` | desligar a máquina no meio da receita |
| Selo não bate (job fantasma) | colher com um `estado.json` de outro job |
| Sem linha de selo / selo repetido / sem marcador | truncar o `arca-fim.txt` |
| Sem `arca-fim.txt`, com job pendente | **rodou** em 23/08, por acidente (E11) |
| Sem `arca-fim.txt`, sem job pendente | rodar `arca resultado` numa mesa limpa |

> **A quinta rodou sem que ninguém planejasse.** Na primeira tentativa do marco
> da E11, quem estava na frente da tela desligou a máquina durante o menu do
> Clonezilla — e o `arca resultado` colheu a ausência de desfecho, nomeou as duas
> causas de C-12, desarmou e encerrou o job. Era a linha que mais tinha esperado.

> **E a primeira rodou de propósito, em 24/08.** Uma sondagem com uma coluna
> inventada no `lsblk` produziu `ARCA_PROBE=FALHOU`, e o `arca resultado` o
> classificou naquela linha, reportou a falha e saiu com código 1. **Foi a
> linha mais barata de produzir da tabela** — a sondagem não grava nada, e o
> comando principal dela falha com uma flag errada.
>
> O que ela **não** produziu é o `FALHOU` de uma operação que grava, e é esse
> que P-6 pergunta: `ARCA_BACKUP=` e `ARCA_RESTORE=` dependem de o **`ocs-sr`**
> devolver código diferente de zero, e nenhuma resposta do `lsblk` fala por ele.

Nenhuma das quatro que sobram é cara de produzir à mão, e **a mais valiosa é o
`FALHOU` de uma operação que grava**, que é P-6 com outra roupa.

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

~~A dependência que sobra é outra: hoje o **primeiro backup** de um dispositivo
novo precisa do menu do Clonezilla, porque o nome do disco no Linux só existe
dentro de uma imagem (§4.5).~~ **Saiu em 24/08/2026.** A E12 foi desenhada para
tirar isso do caminho e tirou: `arca sondar` produziu o `blkdev.list` num
dispositivo sem imagem nenhuma, e o `arca backup --dry-run` seguinte nomeou
`nvme0n1` dizendo que veio da sondagem. **Nenhuma parte do ciclo de vida de um
dispositivo ARCA passa mais por fora do ARCA.**

**O que continua sem original é o ramo de falha desta operação.** Nenhum
`ARCA_PROBE=FALHOU` foi escrito, e ele é mais barato de produzir do que os
outros três — bastaria uma flag inventada no `lsblk`. É P-6 com a roupa mais
barata que ela já teve, e vale como caminho para a linha `FALHOU` do §5.5.

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

**A suíte está inteira verde**: 838 testes, zero vermelhos, com o dispositivo
que a E10 criou sozinho na mesa — e que a E12 já bootou.

Chegar aí custou uma correção que vale registrar aqui, porque ela é sobre o que
este documento mede. **Cinco testes de hardware — das etapas E1, E4, E7 e E11 —
descreviam o dispositivo antigo achando que descreviam um dispositivo**: que o
`ARCAVAULT` tem imagens, que há três cópias armadas de agosto ao lado do
`grub.cfg`, que o `grub.cfg` é byte a byte a captura do repositório.

Nada disso é verdade num dispositivo recém-nascido. Os que dependem de imagem
passaram a **sair cedo dizendo por quê**; os do `grub.cfg` passaram a aceitar os
**dois** inertes conhecidos — o do ISO e o do zip —, com o teste da E10 provando
que são equivalentes. Nenhum foi afrouxado.

**A E10 e a E12 estão commitadas.** O conserto de P-28
([ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md))
está na árvore de trabalho, verde, e ainda não.

### O dispositivo está sem oráculo agora, e isso é do teste de falha

**O `blkdev.list` do dispositivo tem a mensagem de erro do `lsblk`**, e não a
lista de discos: a falha forçada de 24/08 substituiu a sondagem boa, que é o que
SD-4 diz que a segunda sondagem faz. Enquanto ficar assim, `arca backup` recusa
com `POR DETERMINAR` — corretamente.

```text
E:\ARCA-LOGS\sondagem\blkdev.list ... lsblk: unknown column: FLAGQUENAOEXISTE
```

**Devolver o oráculo custa um reinício**: `arca sondar` com o binário normal, que
já está no `ARCABOOT`. Nada mais precisa ser feito antes.

**Os originais da sondagem que deu certo estão salvos** em `recursos/capturas/`
— `blkdev-list-da-sondagem-2026-08-24.txt`, `arca-fim-sondagem-2026-08-24.txt` e
`estado-sondagem-2026-08-24.json` —, e foi por isso que eles foram copiados antes
da falha. **Não os copie de volta para o dispositivo**: um `blkdev.list` escrito
à mão é exatamente o artefato que o §3.5 do PRD alerta, e a diferença entre "a
receita escreveu" e "alguém escreveu depois" é a que este projeto mais paga para
manter.

### O firmware, depois de tudo

`arca status` de 24/08, no fim da sessão:

```text
Entrada de firmware
  Descricao ....... ARCA
  Identificador ... {f4057bd3-65a4-11f1-b0f1-aa4ed9bd2b34}
  Aponta para ..... partition=F: · o ARCABOOT deste dispositivo
  Ordem de boot ... dispositivo em 2o de 2 · `Windows Boot Manager` vem antes

Ultimo job, ja colhido
  Boot unico ...... nao armado
  Estado .......... sondagem · ja colhido, nada esperando
```

A entrada **entrou na ordem permanente** em algum ponto dos dois boots pelo
dispositivo — ela estava fora quando o `arca prepare` terminou —, e C-13 a
empurrou para trás do Windows ao colher. Ligar a máquina sobe o Windows. **Quando
e como ela entrou não está medido**, e é o achado registrado na seção da E12 do
plano de etapas.

> **E às 17:26 daquele mesmo dia a ordem tinha cinco entradas, não duas.** O
> religar de P-22 acrescentou as três classes de dispositivo que o firmware
> enumera no POST, e a linha passou a dizer `dispositivo em 2o de 5`. O Windows
> continua em primeiro e nada disso é perigoso — **mas um religar limpo suja a
> ordem, e isso não estava medido**. As três já tinham estado lá em 20/08, não
> estavam em 22/08 de manhã nem às 17:11, e voltaram; os dois boots pelo
> dispositivo de 24/08 não as trouxeram. Por que vão e vêm é curiosidade sobre
> este firmware — nenhuma tela do ARCA depende da resposta —, e fica registrado
> para quem for comparar contagens de entradas entre capturas e achar que
> alguém mexeu.
>
> As letras também mudaram nesta sessão: o `ARCAVAULT` está em `D:` e o
> `ARCABOOT` em `R:`, e a entrada de firmware acompanhou — `partition=R:`, com
> o `arca status` confirmando `o ARCABOOT deste dispositivo`. É S-3 fazendo o
> que existe para fazer.
>
> **E às 18:47 a ordem tinha duas de novo.** O boot que fechou P-28 levou as
> três embora — não só da ordem: da enumeração inteira, nem como bloco elas
> ficaram. O `bcdedit /enum firmware` daquele momento é **byte a byte** o das
> 17:11 (`89ca7ad1…7b8df3b9`), e o `{bootmgr}` voltou ao topo **sem o ARCA ter
> escrito nada** — o `arca resultado` não chegou a rodar. Quem desfez foi o
> firmware no POST, ou o Windows ao subir; os dois já constam como donos da
> ordem, e esta medição não os separa.

---

## 6. Uma ordem que faz sentido

Do mais barato para o mais caro, e cada linha diz o que se ganha.

| # | O quê | Custa | Fecha |
|---|---|---|---|
| ~~—~~ | ~~**E12** — escrever o `arca sondar` e rodá-lo~~ | ~~1 etapa + 1 reinício~~ | **Feito em 24/08/2026: P-26 e P-27** |
| ~~—~~ | ~~Segunda verificação armada~~ | ~~1 reinício, ~5 min~~ | **Feito em 24/08/2026: P-25** |
| ~~—~~ | ~~Uma sondagem com flag inventada no `lsblk`~~ | ~~1 reinício~~ | **Feito em 24/08/2026: o primeiro `FALHOU`** |
| ~~—~~ | ~~Religar com o SSD conectado, sem job armado~~ | ~~1 reinício, risco zero~~ | **Feito em 24/08/2026: P-22**, e conferiu a promessa da tela do `arca prepare`. Abriu **P-28** |
| ~~—~~ | ~~`UEFI:Removable Device` no topo da ordem, e religar~~ | ~~1 reinício, risco zero~~ | **Feito em 24/08/2026: P-28**, e o firmware apagou as três no POST |
| 1 | `arca sondar` com o binário normal | 1 reinício | *não é pendência* — devolve o oráculo que a falha forçada apagou (§5) |
| 2 | Produzir as outras quatro linhas do §5.5 à mão | tempo, sem risco | quatro casos do §5.5 |
| 3 | Falha forçada em VM | montar uma VM | **P-6** no `ocs-sr`, que é a pergunta original |
| 4 | Próxima restauração, com o log medido | uma restauração | **P-23** |
| 5 | Backup por F12, com o `bcdedit` antes | um backup | **P-19** |

> **O passo que saiu era novo, e nasceu da E12.** Até 24/08, produzir um `FALHOU`
> exigia fazer o `ocs-sr` falhar — uma VM, um disco de teste, uma operação
> destrutiva de mentira. A sondagem tem um comando principal que **falha de
> graça**: uma flag que o `lsblk` não conheça basta, e nada é escrito fora do
> `ARCAVAULT`.
>
> Foi feito no mesmo dia, e rendeu mais do que a linha do §5.5: mostrou que o
> `if` de R-5 toma os dois ramos em hardware, que o `2>&1` guarda a causa no
> dispositivo, e **expôs um teste que aceitava mais do que devia** — o das
> colunas do `lsblk` passava com uma coluna a mais, e a mutação atravessou a
> suíte inteira.

> **O passo de P-28 não era o F12, e é por isso que ele saiu tão barato.** Duas
> versões desta tabela mandaram um F12 escolhendo `UEFI:Removable Device` — e o
> F12 mede a **classe** que o menu do firmware oferece, enquanto a pergunta era
> sobre a **entrada na ordem**, que é o que a tela lê. Pôr aquela entrada em
> primeiro com um `/addfirst` e religar mede o objeto certo, custa o mesmo
> reinício, e não depende de o menu do firmware nomear a linha do mesmo jeito.
>
> **O passo 1 vem antes de tudo por outra razão.** Sem o oráculo da sondagem, o
> `arca backup` recusa com `POR DETERMINAR` — corretamente —, e nenhum dos
> passos que precisa de um backup sai do lugar. Ele não fecha pendência nenhuma;
> só destrava a mesa.

---

## 7. O critério

Este documento existe para que a resposta a *"o app está fechado?"* deixe de
depender de quem responde. O critério proposto:

> **O ARCA se considera fechado quando nenhuma tela dele afirmar algo que este
> repositório não possa mostrar tendo acontecido.**

Por esse critério, hoje falta **uma** coisa, e eram duas:

- ~~**P-26**, porque a tela do `arca prepare` diz *"Dispositivo pronto"* e
  ninguém bootou nele.~~ **Fechada em 24/08/2026**: bootou, pela entrada que o
  próprio comando criou, e a tela deixou de afirmar o que ninguém tinha visto.
- **P-6**, porque a tela do `arca resultado` sabe dizer `FALHOU` e ~~nunca
  disse~~ — **disse em 24/08/2026**, sobre uma sondagem com flag inventada no
  `lsblk`. O que a tela afirma e o repositório ainda não pode mostrar é mais
  estreito do que era: *"o `ocs-sr` falhou e disse"*, numa operação que grava.

**E é essa metade que sobra.** A sondagem provou a **forma**: o `if/then/else`
de R-5 toma os dois ramos em hardware, o `2>&1` guarda a causa no dispositivo, o
`arca resultado` imprime a falha e sai com código 1, e a tela seguinte concorda
com ele. O que continua sem prova é o **sujeito** de P-6: que o `ocs-sr` devolve
código diferente de zero quando falha.

Vale registrar a diferença de custo, porque ela explica por que uma metade
fechou e a outra não: a sondagem falha de graça — uma flag errada, nada escrito
fora do `ARCAVAULT` —, e o `ocs-sr` só falha destruindo alguma coisa.

As outras são perguntas honestas sobre o mundo — como o firmware se comporta,
onde o log da restauração começa. **E uma delas deixou de ser em 24/08**: *de
onde o `bcdedit` lê* era a mais barata de todas, custou um reinício, e a
resposta é a NVRAM. O app não afirma nada sobre as que sobram que dependa da
resposta, e é por isso que ele pode conviver com elas abertas.

**P-28 era o caso limítrofe, e não é mais — pelas duas pontas, no mesmo dia.**
A leitura em duplo achou um ramo em que a tela não engolia um aviso e sim
**afirmava** — `so o boot unico leva a ele`, com a entrada `ARCA` fora da ordem
—, e aquilo era afirmação por ausência de resposta, que o critério acima não
tolera. **Saiu do código às 18:00 e foi medido às 18:47**: com a entrada opaca
em primeiro, a máquina subiu o Windows. Ver
[ADR-0021](../docs/adr/0021-uma-entrada-sem-alvo-na-ordem-nao-e-seguranca.md).

**Vale notar que a ordem foi a inversa da habitual, e de propósito.** O conserto
não esperou a medição porque ele **deixa de afirmar** em vez de afirmar — não há
nada nele que a medição pudesse desmentir. Foi a mesma escolha de C-3 e de
`viu_o_gerenciador`, feita antes de saber a resposta; e quando a resposta veio,
não mudou uma linha.

---

*Atualizar quando qualquer pendência fechar, e apagar quando a do §7 fechar.*
