# A fonte lenta perde a vez, e o `--iso` fica guardado no PC

Decidido em 23/10/2026 (WPC-220), com o veredito confirmado pelo usuário no mesmo dia. PR-7 ganha a fonte lenta e o andamento com barra, velocidade e quanto falta; PR-2 passa a alimentar o pacote guardado; PR-1 fica como está. Revê duas partes do [ADR-0028](0028-nenhum-download-depois-do-ponto-sem-volta.md): a da decisão 6 pela qual uma fonte lenta nunca era largada, e a linha de PR-2 em "O que fica como estava". O resto dele continua valendo, e está listado em [O que fica do ADR-0028](#o-que-fica-do-adr-0028).

## O que aconteceu

Em 22/10/2026, o `arca prepare` sem `--iso` não terminou o download do Clonezilla num computador que não é esta mesa. No dia seguinte, o usuário contou assim:

> "rodei ontem em outro computador - Toda hora tem que ficar copiando o arquivo, Por que ele nunca completa o download. O ideal é que ele baixasse automaticamente."

Duas coisas se somavam, e nenhuma delas era a versão:

- **O 2.1.0 só largava uma fonte parada.** A decisão 6 do ADR-0028 abandona a fonte que fica 60 s sem mandar um byte, e diz que "um minuto sem um byte é uma fonte parada, e não uma lenta". Uma fonte a 64 KB/s manda bytes. O ARCA ficaria nela mais de duas horas, com a fonte seguinte, que talvez entregasse em dois minutos, esperando.
- **A saída que sobrava não ficava.** Quando nenhuma fonte entrega, o README manda baixar pelo navegador e passar o arquivo com `--iso`. Só que a linha de PR-2 do ADR-0028 deixava o `--iso` fora do pacote guardado, e o preparo seguinte daquele computador tentava baixar de novo. A cópia manual se repetia a cada preparo.

E, enquanto o download corria, a tela dizia quanto já veio e mais nada:

> "quero que adicione feedback para a velocidade / e quanto falta (com uma barra de preenchimento) do download. Hoje só nos resta esperar um tempao para cancelar. E pode mudar os ADRs"

O pedido de partida era outro: "Esse app, baixando uma versão fixa do clonezilla, não está funcionando. [...] Precisamos arrumar um jeito de ele atualizar o clonezilla para a versão mais atual." Ele foi trocado por este porque a versão não era a causa. As medições abaixo mostram as mesmas fontes servindo as duas versões, e a fixada chegando inteira a 5,1 MB/s. PR-1 fica.

O desenho (`.design/download-do-pacote.md`) levou duas recomendações ao usuário: uma fonte lenta perde a vez quando, na velocidade dos últimos 20 s, o que falta levaria mais de 10 minutos; e, se todas forem lentas, a mais rápida medida volta e vai até o fim. A resposta foi "pode seguir com as duas recomendações". O padrão do `--iso` foi anunciado na mesma mensagem, "salvo se você disser o contrário", e não foi contestado: copiar para o pacote guardado, deixar o original onde está, e o `--dry-run` não escreve.

## O que foi medido em 23/10/2026

- **Numa captura de tela do usuário**, o navegador baixava o `clonezilla-live-3.3.3-15-amd64.iso` a "63,9 KB/s - 256 KB de 546 MB, 2 horas restantes", numa conexão de 800 Mb. É o ISO da mesma versão, e não o zip que o ARCA usa.
- **Desta mesa, o mirror do projeto entregou `oldstable/3.3.3-15/` inteiro a 5,1 MB/s, em 111 s**, com o SHA256 compilado, `00cee770…6046304`.
- `stable/` respondeu 404. A estável corrente é a `3.3.3-37`, e a `3.3.3-15` mora em `oldstable/`, que guarda todas as versões desde a `1.0.7-18`. É o caso que o ADR-0028 previu ao pôr as duas pastas na lista: um 404 custa um segundo.
- **O SourceForge** redirecionou para `flylife.dl.sourceforge.net` e, em amostras de 20 MB, deu **0,4 MB/s na `3.3.3-15` e 1,2 MB/s na `3.3.3-37`**.

O que não se mediu: o `arca.log` do computador em que o preparo falhou não está disponível ("eu nao tenho o arca log pois foi feito em outro computador"). Não se sabe se o ARCA ficou numa fonte lenta até o cancelamento ou se largou as três. As decisões abaixo cobrem os dois casos. Se ele ficou numa lenta, a decisão 1 o tira de lá em 20 s. Se largou as três, a decisão 5 faz a cópia manual valer uma vez por computador.

As mesmas fontes, medidas desta mesa em 04/10/2026 (ADR-0028), deram 1,5 MB/s no mirror e 0,9 MB/s no SourceForge, em amostras de 20 MB. Dezenove dias depois, o mirror estava mais de três vezes mais rápido, e o SourceForge, a menos da metade. A velocidade de uma fonte não é uma propriedade dela, e por isso a regra a mede a cada execução, em vez de confiar num número de outro dia.

## As seis decisões

### 1. Uma fonte lenta é julgada pelo que falta, e não por uma velocidade mínima

Na primeira volta, uma fonte perde a vez quando, na velocidade dos últimos 20 s, o que falta do pacote levaria mais de 10 minutos, e nunca antes de 20 s de medição naquela fonte. Perto do fim, uma fonte lenta termina: com 524.288.000 bytes no `.parcial` e 64 KB/s, faltam 567 s, e ela fica. No começo, à mesma velocidade, faltariam 2 h 23 min, e ela sai aos 20 s. Uma velocidade mínima julgaria igual o começo e o fim do pacote.

A janela nunca é mais curta que 20 s. A velocidade é medida desde a última amostra de até 20 s antes da atual, e uma fonte que corre há 20 s ou mais é julgada por 20 s de medida, nunca por 19,8.

Uma fonte que nada entregou nos últimos 20 s nunca terminaria, e por isso, na primeira volta, ela também sai por lenta, com `lenta: nada veio nos ultimos 20 s`. A fonte parada, a de 60 s sem um byte, continua valendo, e é ela que tira uma fonte na segunda volta.

Os dois números, `JANELA_DA_VELOCIDADE` (20 s) e `LENTA_SE_FALTAREM_MAIS_DE` (600 s), moram em `src/pacote.rs`, ao lado dos 30 s e dos 60 s de `LIMITES`, e são ajustáveis do mesmo jeito. Ficam fora de `Limites` de propósito (decisão 4).

Isso revoga a parte da decisão 6 do ADR-0028 pela qual uma fonte lenta nunca era largada.

### 2. Nada é recusado por ser lento: as fontes vão em duas voltas

Na primeira volta, cada fonte vai uma vez, na ordem de `FONTES`, sujeita à decisão 1. Na segunda volta, voltam só as que saíram por lentas, da mais rápida medida no momento em que saiu para a mais lenta, e as empatadas seguem a ordem de `FONTES`. Nenhuma é julgada lenta na segunda volta: a primeira dela vai até o fim. Só a parada de 60 s, a falta de conexão em 30 s, um erro HTTP ou o texto do `curl` a tiram, e então entra a seguinte da segunda volta. Nenhuma fonte é tentada mais de duas vezes por execução.

Uma fonte que saiu da primeira volta por 404, por não conectar ou com o texto do `curl` não volta. Ela não saiu por ser lenta, e a segunda volta existe para que nada seja recusado por isso.

Toda troca, nas duas voltas, continua do byte em que o `.parcial` está, e ele continua um só. A linha `Fonte` da segunda volta diz isso: `Fonte ........................... 3.3.3-15 · downloads.sourceforge.net · segunda volta · retomando de <recebido>`.

Recusar uma fonte lenta mandaria o usuário ao navegador, que baixava a mesma versão a 63,9 KB/s. Uma fonte lenta que termina é melhor do que isso. Se nenhuma entrega, a recusa é a de antes, `nao deu para baixar o Clonezilla 3.3.3-15 de nenhuma das 3 fontes:`, mas com uma linha por fonte, e não por tentativa, na ordem de `FONTES`, cada uma com o que a fonte respondeu na última vez em que foi tentada. Até aqui as duas coisas eram a mesma, porque cada fonte ia uma vez só.

### 3. A tela mostra o mesmo número que decide

A linha `Baixando Clonezilla` passa a ter uma barra de 20 casas, a porcentagem, quanto veio na unidade do total, a velocidade dos últimos 20 s e quanto falta:

```text
  Baixando Clonezilla ............. [████░░░░░░░░░░░░░░░░]  20% · 110,0 de 535,5 MB · 5,0 MB/s · faltam 2 min
```

A velocidade e o "faltam" são a mesma medida que julga a fonte lenta, calculada pelo mesmo caminho: a `Medicao` de `src/andamento.rs`. Uma fonte nunca sai por lenta com a tela dizendo que falta pouco, e a linha que diz que ela saiu traz os dois números: `Fonte abandonada ................ free.nchc.org.tw (oldstable) · lenta: faltariam 2 h 23 min a 64,0 KB/s`.

A barra e a porcentagem arredondam para baixo, para 100% só aparecer com o pacote inteiro. Quanto falta arredonda para cima, na unidade em que sai (segundos abaixo de 1 min, minutos abaixo de 1 h, horas e minutos daí em diante), para a tela nunca mostrar 10 minutos ou menos numa fonte que sai por lenta. O julgamento usa o tempo exato, e não o arredondado. Um `.parcial` que não cresce há 5 s troca a estimativa por `0 B/s · parada ha <N> s`, porque a fonte vai sair. Antes de haver o que medir, a linha diz `medindo`.

A linha inteira cabe em 120 colunas com qualquer valor, e é isso que a deixa ser reescrita no lugar: uma linha que quebra não se reescreve, ela se repete. Por isso a versão, o rótulo da fonte e a retomada saíram dela e foram para uma linha `Fonte`, escrita uma vez por fonte tentada. Como foi mostrada ao usuário em 23/10/2026, com eles dentro, a linha passava de 120 colunas.

### 4. Quem julga a lentidão é código puro, e o adaptador só mede e encerra

O `andamento` de `Sistema::baixar` passa a receber uma `Amostra` (o tamanho do `.parcial` e o tempo desde o começo daquela chamada) e a responder com um `ControlFlow`. Com `Continue` o download segue. Com `Break` o adaptador encerra o `curl` como já encerrava a fonte parada, o `.parcial` fica com o que veio, e a `Transferencia` volta com `encerrada`. Quem pediu para encerrar não recebe a amostra final: uma amostra depois da decisão mudaria os números da linha que diz por que a fonte saiu.

É o princípio do ADR-0028, pelo qual quem julga o que a fonte respondeu é código puro, e é o que deixa a regra ser testada com amostras inventadas, sem esperar 20 s. A parada de 60 s continua no adaptador, onde foi medida.

O tempo é monotônico, um `Instant` do adaptador, e nunca o `Relogio`. O `Relogio` é hora de parede e existe para exibir data, e um ajuste de hora do Windows no meio do download faria a velocidade negativa. S-6 não é tocado, porque governa desfecho, e o download não julga desfecho. A varredura de `tests/s6_o_tempo_nao_decide.rs` passa sem mudança.

### 5. O `--iso` que passa no SHA256 alimenta o pacote guardado, por cópia, no pré-voo

O arquivo de quem chamou fica onde está. Depois de o `--iso` passar na conferência, ele é copiado para o `.parcial`, que é conferido (o SHA256 compilado e os quatro caminhos) e só então renomeado para o pacote guardado. Por isso o pacote guardado continua só existindo sob o nome final depois de passar no SHA256, como manda a decisão 4 do ADR-0028. A cópia de PR-3 vai direto para o nome final no `ARCAVAULT`, e esse precedente não serve aqui. Ali o disco acabou de ser particionado; aqui mora um pacote que o próximo preparo usa sem rede.

A tela diz `Pacote guardado ................. copiado do --iso para <guardado> · o proximo preparo nao baixa (PR-2)`. Um guardado que já bate não é reescrito, e a tela é a de antes. Um que não bate é substituído pela cópia, e só pela renomeação, depois de a cópia passar, com `nao batia com o SHA256 deste ARCA · substituido pela copia do --iso`. O ensaio não escreve nada, e diz `o preparo de verdade copia o --iso para este PC; o ensaio nao escreve`.

O preparo continua saindo do arquivo do `--iso`: o passo 7, a extração e a cópia de PR-3 o usam, como antes. A cópia recém-guardada não o substitui nos passos 6 e 7, porque o passo 7 já confere o `--iso` de novo e recusa se ele mudou.

Isso revoga a linha de PR-2 do ADR-0028 ("Ele também não alimenta o pacote guardado: o próximo preparo sem `--iso` baixa") e o conselho do README que decorria dela.

### 6. Uma cópia que falha não recusa o preparo

O preparo com `--iso` nunca dependeu do pacote guardado, e continua saindo do arquivo do `--iso`. Uma cópia que não se faz vira uma linha de aviso na tela, `nao deu para guardar em <pasta> (<motivo>) · este preparo segue do --iso`, e um `AVISO` no `arca.log` com o mesmo motivo. Ela pode não se fazer porque a pasta ou o `.parcial` não aceitam escrita, porque o volume não tem 561.478.648 bytes livres, porque há uma pasta com o nome do pacote, porque o `certutil` não consegue medir a cópia ou porque a renomeação foi recusada. A cópia que não bate é zerada, como o download que não bate, e a linha é `a copia nao bateu com o SHA256 · este preparo segue do --iso`.

Um pacote guardado que não se deixou medir não é tocado, e a linha diz `nao deu para medir o que ja esta guardado (<motivo>) · deixado como esta`. Não ter conseguido medir não é o mesmo que não bater, e é a mesma regra que o ADR-0028 já aplicava ao `.parcial`.

Recusar aqui custaria o preparo inteiro por uma conveniência do próximo, e quem passou o `--iso` tem o pacote na mão.

## A porta de mão única

É a escolha desta decisão que custa caro desfazer: todo adaptador e todo duplo da porta `Sistema` a seguem, e desfazê-la é refazer os dois.

| Porta | A forma | Rejeitado |
|---|---|---|
| O contrato de `Sistema::baixar` | `fn baixar(&self, url: &str, destino: &Path, limites: Limites, andamento: &mut dyn FnMut(Amostra) -> ControlFlow<()>) -> Resultado<Transferencia>`, com `Amostra { recebido: u64, decorrido: Duration }` em `crate::portas`. `recebido` é o tamanho do destino, e `decorrido` é o tempo monotônico (`Instant`) desde o começo desta chamada, medido pelo adaptador. `ControlFlow::Break(())` faz o adaptador encerrar o `curl` como já encerra a fonte parada, e `Transferencia` ganha `encerrada: bool`, verdadeiro só nesse caso. `Limites` continua `{ conectar, parado }` | **O adaptador julgar a lentidão com um terceiro limite em `Limites`**: a regra só se testaria com tempo de verdade, contra servidores locais, e a tela calcularia o mesmo número por outro caminho (decisão 3). **O pré-voo medir o tempo pelo `Relogio`**: é hora de parede, existe para exibir data e não para medir intervalo, e um ajuste de hora do Windows no meio do download daria velocidade negativa. **Um `enum` próprio no lugar de `ControlFlow`**: a biblioteca padrão já dá nome a "continuar ou parar" num callback |

## O que foi rejeitado, além da porta

- **Uma velocidade mínima, no lugar do que falta.** Julgaria igual o começo e o fim do pacote. Ganha se o pacote passar a ter tamanhos muito diferentes entre versões, e hoje ele é um só, de 535,5 MB.
- **Deixar a fonte lenta correr**, como mandava a decisão 6 do ADR-0028. A 64 KB/s, são mais de duas horas na mesma fonte, com outra a 5 MB/s esperando.
- **Manter o `--iso` fora do pacote guardado**, como mandava a linha de PR-2 do ADR-0028. Cada preparo sem `--iso` naquele computador baixaria de novo e falharia de novo, e a cópia manual se repetiria.
- **Mover o arquivo do `--iso`, em vez de copiá-lo.** O arquivo é de quem chamou. Ganha se o computador não comportar duas cópias de 535,5 MB.
- **Usar a cópia recém-guardada nos passos 6 e 7, no lugar do arquivo do `--iso`.** Ganha se o `--iso` costumar vir de um lugar que some no meio do preparo. O passo 7 já o confere de novo e recusa.
- **Medir todas as fontes antes de escolher uma, ou baixar por várias conexões.** Só ganha se uma fonte limitar a velocidade por conexão, e ninguém mediu isso. As duas voltas já cobrem uma fonte lenta por inteiro.
- **Lembrar, entre execuções, qual fonte foi a mais rápida.** O `.parcial` já retoma, e a primeira volta custa no máximo cerca de um minuto. E a velocidade de cada fonte mudou mais de duas vezes entre 04/10/2026 e 23/10/2026: o que se lembrasse seria um número de outro dia.
- **Mais fontes**, como mirrors do SourceForge escolhidos por nome. Ganha se, com a regra nova, aquele computador ainda vir as três lentas.
- **Atualizar o Clonezilla sozinho, ou avisar que há versão nova**, que era o pedido de partida. Os mesmos mirrors servem as duas versões, e a versão não era a causa. PR-1 fica. Reabre se a `3.3.3-15` sumir do `oldstable/` do mirror do projeto e do SourceForge.
- **A barra do `curl`.** Continua calada. O andamento é do ARCA, e o stderr do `curl` é texto sem contrato (ADR-0028).

## Bordas que os critérios não nomeiam

A construção as decidiu em 23/10/2026, e cada uma tem prova num teste do pré-voo ou de `src/andamento.rs`. São reversíveis com um commit. Mudam o que a tela mostra:

- **`retomando de` só aparece com o `.parcial` maior que zero, também na segunda volta.** Três fontes que nada mandaram em 20 s deixam o `.parcial` vazio, e a segunda volta diria `retomando de 0 B`: a linha `Fonte` dizendo uma coisa falsa.
- **Sem velocidade medida, a linha de entrega não diz velocidade.** Uma fonte que entrega sem nenhuma amostra depois da do começo não tem os últimos 20 s, e a linha fica `[████████████████████] 100% · 535,5 de 535,5 MB`.
- **Antes de 5 s sem crescer, uma janela sem nada é `medindo`, e não `0 B/s · faltam` infinito.** Um "faltam" sem velocidade não é uma estimativa.
- **Uma cópia do `--iso` que o `certutil` não consegue medir não é zerada.** "Não consegui medir" não é "não bate" (ADR-0028). Ela cai na linha de aviso da decisão 6, com o motivo do `certutil`.
- **No ensaio, um guardado que não se deixa medir e uma pasta com o nome do pacote mostram as linhas que o preparo de verdade mostraria**, `nao deu para medir o que ja esta guardado (<motivo>) · deixado como esta` e `nao deu para guardar em <pasta> (<motivo>) · este preparo segue do --iso`, sem aviso no `arca.log`. Nos dois casos o preparo de verdade não copia, e a linha do ensaio que diz que ele copia seria falsa. O aviso fica de fora porque o ensaio só diz o que o preparo faria.
- **Uma fonte que parou na primeira volta não entra na segunda.** Só voltam as que saíram por lentas. Na primeira volta, a parada é teórica, porque os 20 s sem nada da decisão 1 vêm antes dos 60 s do adaptador. Mas o duplo a alcança, e a regra tinha de dizer alguma coisa.
- **Na segunda volta, o texto do `curl` abandona a fonte como a parada, a falta de conexão e o erro HTTP**, com a linha `Fonte abandonada` de antes. O código 23 do `curl` continua sendo a recusa `nao deu para guardar o pacote` em qualquer volta ("Reconhecer a falha de escrita", ADR-0028).

## O que ficou em aberto

Cinco perguntas do WPC-220 ficaram sem resposta, e o código segue a resposta provisória de cada uma até o usuário dizer outra:

1. **A recusa `nao deu para baixar o Clonezilla` passa a dizer que o arquivo passado com `--iso` fica guardado?** O texto dela foi aprovado na WPC-134, e continua o mesmo. A frase recomendada é `Ele fica guardado neste PC, e o proximo preparo nao baixa.`
2. **A versão e o rótulo da fonte vão para a linha `Fonte`, fora da linha viva?** É o que o código faz, pela razão da decisão 3.
3. **`parada ha <N> s` aparece a partir de quantos segundos sem o `.parcial` crescer?** O código usa 5 s.
4. **O console desenha `█` e `░`?** Medido em 23/10/2026, nesta mesa, com a CP 850, por `examples/barra_no_console.rs`, como `examples/ponto_no_console.rs` mediu o `·`: desenha, com uma coluna cada, e uma linha viva de 109 caracteres ocupou 109 colunas. A barra fica com `█` e `░`; se outro console não os desenhar, ela passa a `#` e `-`. Fechar a pergunta continua sendo do usuário.
5. **A fonte que nada mandou em 20 s sai por lenta na primeira volta, com `lenta: nada veio nos ultimos 20 s`?** É o que o código faz. Decorre da decisão 1, e o desenho foi emendado para dizê-lo.

## Testes que mudaram de premissa, e não para ficar verdes

- `com_iso_o_pacote_guardado_nem_e_olhado` cobrava o critério 12 da WPC-134: o `--iso` nem olha o pacote guardado. A decisão 5 revoga isso, porque o `--iso` tem de medir o guardado para saber se copia. O teste virou `um_iso_que_passa_com_o_guardado_que_bate_nao_escreve_nada`, que cobra que um guardado que bate não é reescrito.
- `o_pacote_local_bom_e_conferido_de_novo_depois_do_ponto_sem_volta` esperava as conferências `[iso, iso]` e passa a esperar `[iso, .parcial, iso]`, porque a cópia é conferida no meio. A igualdade continua exata.
- Os testes do pré-voo que liam a linha `Baixando Clonezilla` no formato antigo (`3.3.3-15 · <recebido> de 535,5 MB · <rotulo>`, com `retomando de` dentro dela) passam a ler a linha `Fonte` e a barra.
- `o_andamento_chega_ao_menos_a_cada_2_s`, do adaptador, passa a cobrar amostras com o `decorrido` crescendo.

`tests/e10_preparar_o_dispositivo.rs` e as capturas de 23/08/2026 não mudam, porque são evidência.

## O que fica do ADR-0028

Continua valendo:

- **As três fontes e a ordem delas**: o mirror do projeto em `oldstable/3.3.3-15/`, o mesmo mirror em `stable/` e o SourceForge. A ordem agora é a da primeira volta.
- **Os 30 s para conectar e os 60 s de fonte parada**, em `pacote::LIMITES`. O limite da fonte parada continua sendo do adaptador, e não do `--speed-time`.
- **O `.parcial` único que se retoma.** Toda troca de fonte, nas duas voltas, continua do byte em que ele está. O que chega e não bate é zerado, e o do tamanho do pacote ou maior recomeça do zero.
- **PR-1**: a `3.3.3-15` e o SHA256 compilado. A versão não era a causa.
- **PR-3**: a cópia no `ARCAVAULT`, depois do passo 5 e da segunda conferência. Com `--iso`, ela continua saindo do arquivo do `--iso`.
- **O `--dry-run` não baixa** (decisão 7 do ADR-0028), e, com `--iso`, também não copia.
- **B-10**: o ARCA não apaga nada. A cópia do `--iso` usa `copiar` e `renomear`, e a porta `Arquivos` continua sem método de exclusão. Os únicos arquivos substituídos continuam sendo o `.parcial` e o pacote guardado que não bateu.
- **As medições do `curl` de 04/10/2026**: o `-N` obrigatório, o `T + 6 s` do `--speed-time` e os códigos 7, 22, 23, 28 e 33. Também as decisões 1 a 5 e 7, e as duas lacunas decididas no mesmo dia.

Foi revisto:

- **A parte da decisão 6 pela qual a fonte lenta nunca era largada**, com "cada uma tentada uma vez por execução" e "um minuto sem um byte é uma fonte parada, e não uma lenta". A fonte parada continua sendo a de 60 s sem um byte. A lenta passa a perder a vez na primeira volta (decisões 1 e 2), e uma fonte vai no máximo duas vezes. A linha `Baixando Clonezilla`, que a decisão 6 descrevia como "reescrita no lugar com quanto já veio", passa a ser a da decisão 3.
- **A linha de PR-2 em "O que fica como estava".** O `--iso` passa a ler o pacote guardado, para saber se copia, e a escrevê-lo quando passa no SHA256 (decisões 5 e 6).
- **A forma do `andamento` na porta `Sistema::baixar`** da tabela de portas do ADR-0028. Ele deixa de ser `FnMut(u64)` e passa a ser o da porta acima. O resto daquela linha fica: o `-C -`, o `--connect-timeout`, o `-N` e o `parado` medido pelo adaptador.

## Consequências

- **PR-7 e PR-2 ganham texto novo no §9.6 do PRD e no §13 do README**, com a anotação de 23/10/2026. PR-1 não muda.
- O `CONTEXT.md` passa a dizer, em **Pacote guardado**, que o `--iso` que passa no SHA256 também o alimenta. **Segunda volta** não entra no glossário, porque é vocabulário do download, e não do domínio do ARCA.
- `src/andamento.rs` nasce como código puro, com a medida dos últimos 20 s, a linha viva e o julgamento da fonte lenta. `pacote::Resposta` ganha `Lenta`, a única resposta que leva uma fonte à segunda volta.
- A tela do download ganha a linha `Fonte`, uma por fonte tentada, e a linha `Baixando Clonezilla` com a barra. As telas de exemplo do README e do PRD passam a mostrá-las, e as capturas de 23/08/2026 ficam como estão.
- A recusa de nenhuma fonte ter entregado passa a listar fontes, e não tentativas. Tem uma linha por fonte, com a última resposta, e o texto aprovado na WPC-134.
- O `arca.log` ganha a linha da fonte lenta no formato das outras, como `fonte <url> · lenta: faltariam 2 h 23 min a 64,0 KB/s · 1310720 bytes desta fonte`. A cópia do `--iso` deixa `pacote guardado copiado do --iso · <iso> · <guardado>`, e a cópia que não se faz deixa um `AVISO`.
- Um computador só ganha a mudança com o `arca.exe` novo, e a cópia que está no OneDrive é de 04/10/2026. O `.parcial` e o pacote guardado do 2.1.0 continuam valendo, com o mesmo nome e a mesma retomada.
- Funcionou se o próximo `arca prepare` sem `--iso`, no computador em que falhou, terminar o download sozinho, e se uma cópia manual, quando ainda for preciso, acontecer uma vez por computador. Vai mal se a linha `Baixando Clonezilla` mostrar mais de 10 minutos para terminar, por mais de 20 s, sem a fonte mudar na primeira volta, ou se o mesmo computador voltar a pedir o zip pelo navegador depois de um `--iso`.
