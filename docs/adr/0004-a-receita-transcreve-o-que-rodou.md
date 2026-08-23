# A receita transcreve o que rodou, e marca o que não tem original

O plano manda a E3 comparar a receita gerada, caractere a caractere, com a que rodou em hardware — e chama isso de "o ponto de verificação mais importante do projeto". Ao buscar o original para comparar, ele não existia inteiro. As três receitas preservadas no dispositivo — `grub.cfg.backup02`, `grub.cfg.original`, `grub.cfg.teste02`, agora em `recursos/capturas/` — divergem do §10 do PRD em seis pontos, e três deles são partes que **nenhuma execução real jamais teve**.

Decidimos separar as duas coisas em vez de dissolver uma na outra. O que tem original é transcrito e testado contra o arquivo capturado. O que não tem é escrito como código novo, marcado como tal no comentário e no teste, e fica devendo confirmação em hardware — exatamente como a E2 fez com o caso construído do `bootsequence`.

## O que é transcrição

As flags do `ocs-sr` e a ordem delas, o `ocs-chkimg` com saída redirecionada, o `ocs_repository`, o `locales`, o `keyboard-layouts`, o `ocs_live_batch`, e a forma `ocs_live_run="bash -c '...'"` com `;` entre os passos. Os testes de `src/receita.rs` extraem esses trechos das capturas e comparam — nenhum deles pode ser ajustado para passar, porque o oráculo é o arquivo.

Três decisões saíram daí, todas tomadas com `ocs-sr-help.txt` na mão:

**`-scs` fica de fora, contra o que B-8 pedia.** B-8 mandava usá-lo sempre. O help diz o que ele é: `--skip-check-restorable`, *"By default Clonezilla will check the image if restorable after it is created. This option allows you to skip that."* Ele **pula** uma verificação, que é o oposto do que B-9 quer. O hardware rodou sem ele, e sem ele existem dois sinais independentes sobre a imagem: a conferência nativa, que alimenta o código de saída que o `if` de R-5 lê, e o `ocs-chkimg` explícito, que não depende dela. O custo é tempo de execução, numa operação que já leva dezenas de minutos.

**`-p true` entra, e B-8 não o listava.** Não é enfeite: o help diz que o padrão de `-p|--postaction` é **`reboot`**. Sem `-p true`, o `ocs-sr` reiniciaria a máquina assim que terminasse de gravar, e o `ocs-chkimg` obrigatório de B-9 nunca chegaria a rodar. A receita que rodou em hardware o tinha; o requisito que a descrevia, não. É o tipo de omissão que só aparece quando se compara documento com evidência.

**`-e1 auto -e2` ficam na restauração, e R-4 não os lista.** `-e1 --change-geometry` força a mudança do CHS da partição de boot NTFS depois de restaurar; `-e2 --load-geometry-from-edd` força usar o CHS do EDD ao criar a tabela por `sfdisk`. Restaurando no mesmo disco são inócuos — a geometria de destino é a de origem. Restaurando em **outro** disco, que a decisão 5 do plano permite, são o que faz a partição de boot bater com a geometria do disco novo. Não há argumento para tirar de uma receita destrutiva o que estava na única execução dela que deu certo.

## O que é código novo

**O `if/then/else` de R-5.** As três receitas encadeiam com `;`. A armadilha que R-5 descreve — uma falha deixar o mesmo rastro de um sucesso — é real e está registrada no §11 do PRD, mas a defesa contra ela nunca rodou.

**O `arca-fim.txt`, o selo e o `ARCA_FIM`.** Este é o achado mais sério da etapa. **Nenhuma receita real escreve o desfecho**, e portanto todo o mecanismo do qual a E5 e a E8 dependem nunca foi exercitado. O `arca-fim.txt` que existe no dispositivo, com `ARCA_RESTORE=OK` e `ARCA_FIM`, veio de trabalho manual de validação — exatamente o mesmo padrão que o [ADR-0003](0003-veredito-lido-do-arca-check-log.md) já tinha identificado no `ARCA_VEREDITO=`. É o segundo caso do mesmo tipo: **o PRD documenta como fundação validada coisas que na verdade vieram do trabalho de validação em volta dela.** Um teste em `src/receita.rs` cobra que nenhuma captura contenha `arca-fim.txt`, `ARCA_SELO` ou `if `, para que este achado não vire folclore.

**O `ARCA_VEREDITO=` no `arca-check.log`.** O ADR-0003 deixou a decisão para esta etapa, porque é a receita que escreveria. A receita passa a escrevê-lo: `if ocs-chkimg ... > log 2>&1; then echo ARCA_VEREDITO=APROVADA >> log; else echo ARCA_VEREDITO=REPROVADA >> log; fi`. O marcador é o caminho preferido do leitor da E1, e escrevê-lo tira o veredito da dependência de interpretar frases em inglês do `ocs-chkimg`. As imagens antigas continuam legíveis pelo caminho do resumo, como o ADR-0003 previu.

**Três mudanças menores, todas com motivo próprio.** O `-p poweroff` da restauração vira `-p true`: com a máquina desligando dentro do `ocs-sr`, o `echo` do desfecho nunca aconteceria — S-4 e R-5 seriam letra morta. O log do Clonezilla sai de `/home/partimag/restore.log`, um caminho fixo na raiz que a restauração seguinte sobrescreveria, e vai para o `ARCA-LOGS` do job (D2 do plano). E a verificação de B-9 mora **dentro** do ramo de êxito do backup: com o `savedisk` falhando, a pasta da imagem pode nem existir, e até o `else` do `ocs-chkimg` falharia ao tentar escrever nela.

> **Tudo desta seção rodou em 22/08/2026**, no marco em hardware da E7 e da E8.
> O `if/then/else` tomou o ramo do êxito, o `arca-fim.txt` foi escrito com selo
> e `ARCA_FIM`, o `ARCA_VEREDITO=APROVADA` foi acrescentado ao `arca-check.log`,
> e o `-p true` fez o que se esperava dele — o `ocs-sr` devolveu o controle à
> receita em vez de reiniciar, e os passos depois dele aconteceram. Os originais
> estão em `recursos/capturas/`, e o que atesta que foi a receita, e não uma
> pessoa depois, é o `ocs-sr-linha-de-comando-2026-08-22.txt`, escrito pelo
> próprio Clonezilla.
>
> **O que continua sem rodar é o ramo de falha** dos dois `if`: o
> `ARCA_BACKUP=FALHOU` e o `ARCA_VEREDITO=REPROVADA`. Uma execução
> bem-sucedida não os exercita, por definição, e é P-6.
>
> O achado desta seção — que o PRD documentava como fundação validada o que
> veio do trabalho de validação em volta — continua de pé; o que mudou é que o
> `arca-fim.txt` e o `ARCA_VEREDITO=` deixaram de ser exemplos dele.

## O que o `bash` diz, e os testes não podiam dizer

Os testes de `src/receita.rs` provam o que a **string contém**. Nenhum deles prova o que o **bash faz com ela** — e o `if/then/else` aninhado é justamente a parte sem original. `recursos/ensaio-da-receita.sh` fecha isso: roda as duas receitas num bash de verdade, com o Clonezilla substituído por comandos falsos que saem com o código que se pedir, e confere o rastro de cada desfecho.

Os cinco casos passam. O `savedisk` bem-sucedido com imagem aprovada deixa `ARCA_SELO` / `ARCA_BACKUP=OK` / `ARCA_FIM` e um `ARCA_VEREDITO=APROVADA`; com imagem reprovada, o mesmo desfecho e `ARCA_VEREDITO=REPROVADA`; o `savedisk` que falha deixa `ARCA_BACKUP=FALHOU`, **não chama o `ocs-chkimg`** e não deixa `arca-check.log` — que é o ponto de a verificação morar dentro do ramo de êxito. A restauração ramifica igual, e o `2>&1` captura o `stderr` do `ocs-sr`.

Isso não substitui o marco em hardware: o ensaio prova que a *string* está certa, e não que o Clonezilla a executa como se espera dentro do `grub`. Mas o modo de falha mais provável do código novo — um `fi` no lugar errado escrevendo `OK` sobre uma falha — está coberto antes do primeiro reinício.

O ensaio mora fora do `cargo test` porque precisa de bash, que nem toda máquina Windows tem. O preço é ele poder ficar para trás quando a receita mudar, e um teste em `src/receita.rs` paga esse preço: se as strings divergirem, ele falha dizendo o que fazer.

## O que a revisão pegou, e o que ela revelou sobre o método

A revisão de código desta etapa achou cinco defeitos, e o mais grave era **causado por uma melhoria**. Vale registrar cada um, porque três deles são do mesmo tipo: uma peça nova interagindo mal com uma peça antiga que ninguém releu.

**O `ARCA_VEREDITO=APROVADA` podia inverter uma reprovação.** O leitor da E1 procurava, nesta ordem: marcador de reprovação, marcador de aprovação, `not restorable` no resumo, `restorable` no resumo. Enquanto o marcador só existia porque alguém o escrevera **depois de olhar o log**, essa ordem estava certa. A partir do momento em que a receita passou a escrevê-lo a partir do código de saída do `ocs-chkimg`, deixou de estar: um `ocs-chkimg` que saísse zero com um `NOT restorable` no texto — que é P-6 aplicado a ele — deixaria as duas marcas no arquivo, e o marcador venceria. Uma imagem quebrada sairia como aprovada, **por causa** de uma mudança feita para melhorar a leitura do veredito. A ordem passou a ser: toda forma de reprovar antes de toda forma de aprovar. O comentário do código já prometia isso ("qualquer sinal de reprovação reprova"); só não era o que ele fazia.

**B-2 aceitava `ARCA-LOGS` como nome de imagem.** Um `arca backup ARCA-LOGS` gravaria a imagem por cima da pasta de logs do dispositivo — e ela sumiria da listagem, porque `imagens::enumerar` pula esse nome. Invisível no `arca list`, e invisível também para o pré-voo de B-3, que é quem recusaria o nome já usado. O que a enumeração esconde, o validador tem de impedir que exista; agora há um teste que percorre a lista inteira, e não os dois casos que me ocorreram.

**O backup e a restauração da mesma imagem dividiam o `arca-fim.txt`.** O comentário que eu escrevi dizia "uma por nome de operação, e nunca uma só compartilhada", e o caminho ignorava a operação. Toda receita começa truncando o próprio `arca-fim.txt` com um `>`: um `arca restore X` rodado antes de o backup de X ser colhido apagaria o desfecho dele para sempre, e o §5.5 leria um backup bem-sucedido como desfecho ausente. O selo não cobre esse caso — ele julga um desfecho **encontrado**, e não serve para nada quando o arquivo já foi por cima. O log agora leva a operação no nome.

**Faltavam `COM0` e `LPT0`** na lista de nomes reservados do Windows. Detalhe, mas com um detalhe dentro: quem cria a pasta é o Clonezilla, do lado Linux, onde `COM1` é um nome como outro qualquer. A recusa **tem** de acontecer do lado Windows, antes de a receita existir — do outro lado não há quem recuse.

**O nome podia estourar a linha de comando do kernel.** O `COMMAND_LINE_SIZE` do x86_64 é 2048, e estourá-lo não dá erro: o kernel trunca em silêncio, e uma receita truncada é exatamente o caso do §3.2 — o Clonezilla descarta a string e abre o menu. Medido: o nome aparece dez vezes na receita de backup, e cada caractere custa dez na linha. Com o limite antigo de 64, sobravam 105 caracteres de folga. Agora há um orçamento explícito (2048 menos 512 reservados para o `menuentry` base, medido em 206–369 nas capturas), uma recusa própria sobre a linha pronta, e o limite do nome baixou para 48 — com o que a receita mais longa fica em 1271 dos 1536.

O padrão nos três primeiros é o mesmo, e é o mesmo desta etapa inteira: **uma peça nova encaixada numa peça antiga que ninguém releu ao encaixar.** A defesa que funcionou foi ler o código antigo procurando o que a peça nova mudava nele — não ler a peça nova procurando defeitos.

## Consequências

A receita de hoje não é mais uma cópia de nenhuma que rodou. É uma transcrição fiel do núcleo, com um envoltório novo — e o envoltório é justamente onde mora tudo que faz o ARCA saber se a operação deu certo. **A E7 e a E9 deixam de ser confirmações de um mecanismo pronto e passam a ser as primeiras execuções do mecanismo de desfecho.** O marco em hardware da E7 vale mais do que o plano supunha: ele estreia o `arca-fim.txt`, o selo dentro da receita, o `ARCA_FIM` e o `if/then/else`, todos de uma vez.

O `-scs` fora e uma verificação a mais no caminho de todo backup. Se ela vier a custar caro demais na prática, a discussão volta — mas volta com medição, e não com o requisito que a omitia.

C-2 continua sendo a única barreira entre um escape errado e uma máquina parada num menu, como o [ADR-0002](0002-receita-como-string-no-grub.md) registrou. O validador recusa **toda** aspa, e não aspa desbalanceada: um par balanceado de aspas simples dentro do `bash -c '...'` fecha a string do `bash` e abre outra, produzindo algo sintaticamente válido e semanticamente diferente. Contar aspas daria só a impressão de estar conferindo.

## Fora do escopo desta decisão

O help do `ocs-sr` diz que **por padrão** o Clonezilla confere o tamanho do disco de destino e desiste se ele for menor que a origem, e que `-icds` é quem desliga essa conferência. A decisão 5 do plano e R-7 partem da premissa contrária — de que `-k0` num disco menor corromperia em vez de falhar. A receita de restauração não usa `-icds`, e um teste cobra isso; resolver a contradição é da E9.
