# Colher marca o `estado.json` como colhido, em vez de apagá-lo

A E5 deixou um par que lê como contradição: depois de um `arca desarmar`, o
`arca status` mostrava **"Boot único: não armado"** ao lado de um **job
pendente**. Não era contradição — o dispositivo estava inerte e o job continuava
registrado —, mas ninguém fechava o par. O plano diz que quem fecha é a E8:
colher o desfecho encerra o job.

Encerrar como? Havia três saídas, e cada uma muda o que `arca status` mostra
depois: **apagar** o `estado.json`, **marcá-lo** como colhido, ou **deixá-lo** e
distinguir por outro sinal.

Decidimos **marcar**. O `estado.json` ganha um sexto campo, `situacao`, com dois
valores: `armado` e `colhido`.

## Por que não apagar

Apagar obrigaria a refazer a discussão de B-10, e o argumento não se transporta.
`src/desarme.rs` tem uma seção inteira defendendo que apagar o `bootsequence`
não fura B-10, e ela se apoia numa distinção precisa: a marca de boot único é
uma **intenção** que o próprio ARCA gravou, e desfazê-la é o que C-1 manda.

O `estado.json` colhido não é intenção. É **registro** — o único lugar que liga
um selo a um nome de imagem, e o único lado do reinício que o ARCA escreve.
Apagado ele, um `arca-fim.txt` que aparecesse depois não teria a quem pertencer,
e "job fantasma" viraria a resposta para tudo. O mecanismo do §4.3 existe para
distinguir quatro casos entre si; apagar o registro reduz três deles a um.

## Por que não deixar e distinguir por outro sinal

O sinal natural seria a existência do `arca-fim.txt`: há desfecho, logo o job
foi colhido. Ele falha exatamente onde mais importa. Um job cujo **boot não
aconteceu** não tem `arca-fim.txt` nenhum — e é justamente o caso do §5.5 que
deixou o dispositivo armado sem nada a colher. Ele ficaria pendente para
sempre, e a contradicão que se queria fechar voltaria pior.

Além disso, esse sinal mora do outro lado do reinício, no `ARCAVAULT` que o
Clonezilla escreve. Decidir se um job está encerrado a partir de um arquivo que
o ARCA não escreveu é a mesma família de erro que o ADR-0001 registra sobre
datas.

## Por que uma situação, e não uma data

A forma óbvia seria `colhido_em`. Duas razões contra, e a segunda é a que pesa.

A primeira é mecânica: enquanto o job não foi colhido, o campo precisaria de um
valor sentinela, e `MomentoDoArmar` exige vinte e cinco caracteres. Sentinela
dentro de um leitor que recusa tudo que não reconhece é uma exceção esperando
para ser esquecida.

A segunda: poria **mais um instante ao lado do `armado_em`** num arquivo cujo
tipo de tempo existe justamente para tornar a comparação difícil
([ADR-0006](0006-o-selo-e-o-estado-sem-dependencia-nova.md), S-6). Duas datas
lado a lado são um convite a subtraí-las, e a trava que reprovou um backup
perfeito neste projeto nasceu de uma subtração que parecia inofensiva.

Um estado com alfabeto fechado — `armado` ou `colhido` — não se subtrai. E ele
mantém de pé a premissa que sustenta escrever o JSON à mão: **nenhum dos seis
valores alcança `"`, `\`, controle ou não-ASCII**. O ADR-0006 avisava que a
discussão voltaria com o campo novo na mesa; ela voltou, e o campo passa.

## O sexto campo é obrigatório, e o formato muda de forma visível

O leitor recusa chave faltando, e `situacao` não é exceção. Um `estado.json` de
cinco campos — escrito por um ARCA anterior a esta etapa — é **recusado
nomeando a chave que falta**, e não lido como "armado por suposição".

É a mesma escolha que o ADR-0006 fez para chave desconhecida, pelo mesmo
motivo: agir sobre metade de um estado que arma uma operação destrutiva é pior
do que recusar o arquivo inteiro. Há exatamente um escritor, e ele está neste
repositório.

## O que encerra o job, e o que não encerra

A parte que exigiu mais cuidado, e ela é uma distinção que a revisão da E5 já
tinha pago caro para existir.

**Encerra** quando o ARCA chegou a um veredito sobre o job: achou o
`arca-fim.txt` e o julgou — qualquer das cinco linhas do §5.5, inclusive job
fantasma —, ou **não achou arquivo nenhum**. O segundo caso é C-12 na letra: "o
boot não aconteceu, ou o Clonezilla abriu menu" é uma resposta, e é reportada
como falha. Deixá-lo pendente faria o `arca status` dizer "job por colher" para
sempre, sem nada que pudesse colhê-lo.

**Não encerra** quando o arquivo está lá e não se deixou ler. "Não consegui
olhar" não é veredito. Encerrar aqui transformaria um backup possivelmente
bem-sucedido num job fechado como se nunca tivesse rodado, e perderia o selo que
liga o desfecho ao job. Resolvido o problema de leitura, `arca resultado` roda
de novo e o selo continua lá.

## Consequências

`arca status` ganha uma sexta linha para o estado do job, e o par que a E5
deixou aberto fecha: um job colhido não aparece como pendente, e o boot único
não armado ao lado dele deixa de parecer contradição.

`arca status` também **para de procurar o desfecho** de um job já colhido.
Ir olhar de novo reabriria uma pergunta que `arca resultado` fechou — e, pior,
um `arca-fim.txt` truncado pela operação seguinte apareceria como "o boot não
aconteceu" para um job que aconteceu.

`arca resultado` rodado duas vezes não colhe duas vezes: a segunda diz que o job
já foi colhido e **não desarma de novo**. Para desarmar há `arca desarmar`.

Nada é apagado, e B-10 não precisa ser discutido. O `estado.json` de um job
colhido fica no dispositivo até o próximo `arca backup` gravar por cima dele —
que é substituição, e não exclusão.
