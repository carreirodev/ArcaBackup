# Nenhum download depois do ponto sem volta, e o pacote fica guardado no PC

Decidido em 04/10/2026 (WPC-134). Nasce PR-7; PR-1, PR-2 e PR-3 ficam como estão. Revisto no mesmo dia, depois da verificação, em dois casos que tinham ficado de fora: ver [as duas lacunas](#duas-lacunas-nomeadas-e-decididas-no-mesmo-dia).

## O que aconteceu

Até esta decisão, o `arca prepare` sem `--iso` baixava o Clonezilla no passo 6, depois de o passo 5 ter apagado o disco. Em 03/10/2026, o dia inteiro, e na manhã de 04/10/2026, o download ficou parado sem dizer nada até o usuário cancelar. Não era a primeira vez, e cada tentativa custou o disco antes de falhar.

Três coisas se somavam:

- **O download vinha depois do ponto sem volta.** Um download que falhasse no passo 6 deixava um dispositivo vazio, e a tentativa seguinte começava apagando de novo.
- **O download não tinha limite para uma transferência parada, nem andamento na tela.** A FAQ do `curl` diz que, sem um limite próprio, uma conexão que morre no meio não é detectada.
- **Cada preparo baixava 535,5 MB de novo**, porque o passo 5 apaga também a cópia de PR-3 que estava no `ARCAVAULT`.

A versão fixada não é a causa. A `3.3.3-15` continua no mirror do projeto, em `oldstable/3.3.3-15/`, com o mesmo SHA256 e o mesmo tamanho, e a versão mais nova é servida pelo mesmo SourceForge. O travamento não se reproduziu em 04/10/2026: o link fixado respondeu.

## O que foi medido em 04/10/2026

### As fontes

Uma amostra de 20 MB de cada fonte: o SourceForge entregou a **0,9 MB/s** e o mirror do projeto a **1,5 MB/s**. São de 6 a 10 minutos por download quando ele funciona — estimativa de uma amostra, e não medição de um download inteiro.

### O `curl` 8.21.0 do `System32`

A documentação do `curl` (`--connect-timeout`, `--speed-time`, `-C -`, `--fail`, `--write-out`) foi lida no mesmo dia, e não respondia a pergunta que decidia o desenho: quanto tempo uma fonte parada leva para ser abandonada. A resposta veio de uma medição nesta máquina, contra servidores locais escritos para isso, que não ficaram no repositório:

| Situação | O que o `curl` fez |
|---|---|
| Fonte que entrega 100 bytes e para | `--speed-time 2` abandona em 8,0 s, `6` em 12,0 s, `12` em 18,0 s: **o `curl` abandona `T + 6 s` depois do último byte**. Com `60`, seriam 66 s |
| Fonte que entrega 100 bytes a cada 0,5 s, **sem `-N`** | o arquivo de destino fica com **0 bytes** enquanto os bytes chegam, e encerrar o `curl` perde o que estava no buffer |
| A mesma fonte, **com `-N`** | o destino cresce 100, 200, 300, 400 bytes, e fica com 400 depois de o `curl` ser encerrado |
| Conexão que não completa (`192.0.2.1`) | código 28, `http_code` `000`. A mesma depois de um `302`: código 28, `http_code` `302` |
| 404, direto e depois de um `302` | código 22, `http_code` `404`, e nenhum arquivo criado |
| `-C -` com 100 bytes no destino | o pedido leva `Range: bytes=100-`, e o destino termina com 1000 bytes |
| `-C -` sem arquivo, ou com um de 0 bytes | o pedido vai sem `Range`, e o destino termina com 1000 bytes |
| Servidor que ignora o `Range` | código 33, destino intocado |
| Porta fechada | código 7 |
| `.parcial` somente leitura, com bytes para retomar | código 23, sem ter pedido nada à fonte: com o que retomar, o `curl` abre o destino antes de conectar. Vazio, ele conecta e sai com 23 no primeiro byte. O stderr traz o caminho do arquivo quebrado em três linhas, começando por `curl: cannot open`, antes da frase `curl: (23) Failed writing received data to disk/application` |

Parte disso volta a ser medida a cada `cargo test`, contra servidores locais que os próprios testes do adaptador sobem: a fonte parada que deixa o `.parcial` com o que veio (`uma_fonte_parada_e_abandonada_e_o_parcial_fica_com_o_que_veio`), a retomada com `Range` (`o_curl_retoma_do_byte_em_que_o_parcial_termina`), o 404 com código 22 (`um_erro_http_volta_com_o_codigo`), o andamento (`o_andamento_chega_ao_menos_a_cada_2_s`) e o `.parcial` somente leitura (`um_parcial_somente_leitura_volta_com_o_codigo_23_sem_conectar`).

Duas linhas da tabela mudaram o mecanismo, e não a decisão:

- **O limite da fonte parada é do ARCA, e não do `--speed-time`.** O `curl` mede velocidade por média, e abandonaria uma fonte parada em 66 s, e não nos 60 s de PR-7. O adaptador lê o tamanho do `.parcial` e encerra o `curl` depois de 60 s sem ele crescer.
- **O `-N` é obrigatório.** Sem ele, o tamanho lido do destino fica em 0 enquanto os bytes chegam: o andamento na tela mentiria, e encerrar o `curl` perderia o que a próxima fonte deveria retomar.

## As sete decisões

### 1. Nenhum download acontece depois do ponto sem volta

Sem `--iso`, o pacote é obtido e conferido no **pré-voo**, antes do passo 0 — antes até de listar os discos —, e o resto do comando é o caminho que o `--iso` já seguia. É PR-7. Um preparo que não consegue o pacote recusa com o disco intacto, e as três recusas novas terminam em `Nada foi apagado (PR-7)`.

O passo 6 deixa de baixar: quando ele chega, o pacote já é local, o do `--iso` ou o guardado.

**Rejeitado: baixar depois da confirmação digitada**, e não antes do menu. A espera do primeiro preparo de cada versão sairia da frente da pergunta, mas a releitura do disco de PR-4 ficaria até 10 minutos velha quando o passo 5 chegasse. Ganha se essa espera incomodar mais do que isso, e hoje não incomoda.

### 2. O pacote guardado mora no PC, em `%LOCALAPPDATA%\ARCA\`

Na mesma pasta do `arca.log`, nomeado pela versão: `clonezilla-live-3.3.3-15-amd64.zip`. Sem `LOCALAPPDATA`, a pasta cai no diretório temporário, como o `arca.log` já caía.

Fora do dispositivo de propósito: o passo 5 apaga o dispositivo. Uma restauração do `C:` que leve o pacote embora custa um download, e nada mais.

**Rejeitados:** o `ARCAVAULT`, que o passo 5 apaga a cada preparo — é exatamente onde a cópia de PR-3 mora, e é por isso que ela não serve de pacote guardado; e a pasta do `arca.exe`, que muda conforme de onde ele é rodado.

### 3. Guardado não é confiado

O SHA256 compilado é conferido a cada uso: no pré-voo e de novo no passo 7, como o do `--iso` já era. Um arquivo que passou meses no `%LOCALAPPDATA%` pode ter sido trocado, truncado por um disco cheio ou restaurado de outro lugar, e a conferência custa segundos.

Nenhuma fonte é confiável, e por isso elas são intercambiáveis: quem diz se o que veio é o pacote é o SHA256 compilado. **PR-1 não muda.**

### 4. Um pacote só existe sob o nome final depois de passar no SHA256

O download em andamento mora em `clonezilla-live-3.3.3-15-amd64.zip.parcial`, ao lado do pacote guardado, e é ele que se retoma. Só depois de passar na conferência — o SHA256 e os quatro caminhos obrigatórios — o `.parcial` é renomeado para o pacote guardado.

O tamanho compilado, 561.478.648 bytes, decide entre retomar e recomeçar. Um `.parcial` menor que isso é um download interrompido, e se retoma do byte em que termina. Um do tamanho do pacote, ou maior, já veio inteiro e não bateu, ou não é o pacote, e recomeça do zero: retomá-lo pediria um byte que o pacote não tem. E desde a revisão do mesmo dia, o `.parcial` que chega e não bate é zerado na hora, qualquer que seja o tamanho dele ("Zerar o que não bate", abaixo).

### 5. O ARCA nunca apaga um pacote guardado (B-10)

Nem o de uma versão antiga: quem limpa a pasta é uma pessoa, olhando. Os únicos arquivos substituídos são o pacote guardado que não bateu com o SHA256 — um arquivo que não bate não é o pacote, e é a mesma exceção que a escrita atômica já tem — e o `.parcial` que recomeça do zero ou que chegou e não bateu.

A porta `Arquivos` ganha `renomear`, que substitui o destino, e continua sem método de exclusão.

### 6. Nenhum download fica mudo

Uma fonte que não conecta em 30 s, ou que fica 60 s sem entregar nada, é abandonada, e a próxima continua do byte em que a anterior parou: o `.parcial` é um só. A linha `Baixando Clonezilla` é reescrita no lugar com quanto já veio, a cada segundo, e cada fonte abandonada ganha uma linha que diz o que ela respondeu. Uma fonte de verdade entrega centenas de quilobytes por segundo; um minuto sem um byte é uma fonte parada, e não uma lenta.

As fontes, em ordem, cada uma tentada uma vez por execução:

1. o mirror do projeto, em `oldstable/3.3.3-15/`;
2. o mesmo mirror, em `stable/`;
3. o SourceForge, em `clonezilla_live_stable/3.3.3-15/`.

O mirror vem primeiro porque foi cerca de 1,7 vez mais rápido na única medição, e porque foi o SourceForge que travou. Ele entra com as duas pastas porque move a versão de `stable/` para `oldstable/` quando sai a seguinte — e a `3.3.3-15` já estava em `oldstable/` em 04/10/2026 —, e um 404 custa um segundo. Os 30 s e os 60 s são padrões, ajustáveis se um mirror lento os disparar sem estar parado.

**Rejeitado: o SourceForge primeiro.** Ganha se o mirror do projeto se mostrar mais lento ou mais instável do que ele nos primeiros preparos.

### 7. O `--dry-run` não baixa

Sem pacote guardado, o ensaio diz que o preparo de verdade vai baixar 535,5 MB antes de qualquer pergunta, e não escreve nada na pasta — nem o `.parcial`. Com o pacote guardado, ele confere o SHA256, que é leitura.

## As quatro portas de mão única

São as escolhas desta decisão que custam caro desfazer, porque o resto do código passa a contar com elas.

| Porta | A forma | Rejeitado |
|---|---|---|
| A porta `Arquivos` ganha `renomear` | `fn renomear(&self, origem: &Path, destino: &Path) -> Resultado<()>`, substituindo o destino se ele existir (`fs::rename`, que no Windows é `MoveFileEx` com `MOVEFILE_REPLACE_EXISTING`). A porta continua sem método de exclusão | **Um nome só, com o tamanho decidindo entre parcial e completo.** Um download pela metade ficaria com o nome do pacote, e quem o visse na pasta o tomaria por um |
| O contrato de `Sistema::baixar` | `fn baixar(&self, url: &str, destino: &Path, limites: Limites, andamento: &mut dyn FnMut(u64)) -> Resultado<Transferencia>`. Sempre continua do fim do destino (`-C -`); o `curl` cuida da conexão (`--connect-timeout`); o adaptador lê o tamanho do destino (com `-N`), chama `andamento` ao menos a cada segundo e encerra o `curl` depois de `parado` sem o destino crescer, devolvendo `parou: true`. Quem julga o que a fonte respondeu é código puro, pelo código de saída e pelo `http_code`, e nunca pela frase | **`--speed-limit 1 --speed-time 60`**, que era o que o issue supunha: medido, o `curl` abandona `T + 6 s` depois do último byte — 66 s —, e sem `-N` o tamanho lido do destino fica em 0 enquanto os bytes chegam. E **ler o andamento da barra de progresso do `curl` no stderr**: é texto sem contrato, e a porta inteira foi desenhada para não julgar texto |
| O pacote guardado e o `.parcial` moram na pasta do registro | `Registro::pasta()`, a mesma pasta de `arca.log`; `pacote::guardado_em(pasta)` e `pacote::parcial_em(pasta)` dão os dois caminhos | **Um campo novo no `Contexto`**, em nove construções: duas fontes para a mesma pasta, que só precisariam divergir uma vez — e a pasta é, por definição, a do `arca.log`, inclusive sem `LOCALAPPDATA` |
| Recomeçar do zero é zerar o `.parcial` antes do `curl` | `escrever_atomico(parcial, "")` quando o `.parcial` não existe, tem o tamanho do pacote ou mais, ou o pacote guardado não bateu — e, desde a revisão, depois de um `.parcial` entregue que não bateu. Essa escrita é também a prova de que a pasta aceita escrita, antes de qualquer download; quando não há o que zerar, quem descobre a falta de escrita é o `curl`, com o código 23 | **Chamar o `curl` sem `-C -`**, que trunca sozinho: a falta de escrita só apareceria durante o download, com o texto do `curl` em cada uma das três fontes |

## Um teste mudou de premissa, e não para ficar verde

O teste do pré-voo sem `--iso`, da WPC-66, se chamava `sem_iso_o_pre_voo_nao_tem_o_que_conferir` e exigia que o pacote fosse conferido `uma vez so, depois do ponto sem volta` — baixado no passo 6, com o disco já apagado. Era a regra de antes escrita como teste, e esta decisão a revoga.

Ele foi reescrito como `sem_iso_o_pacote_guardado_e_conferido_antes_do_menu_e_de_novo_no_passo_7`, e cobra o contrário: o pacote guardado conferido no pré-voo e de novo no passo 7, a extração saindo dele e a cópia de PR-3 feita a partir dele. A igualdade exata da lista de arquivos conferidos pega uma conferência a mais ou a menos, e o comentário do teste diz o nome antigo e por que ele mudou.

## O que foi rejeitado, além das portas

- **Um comando `arca baixar` separado**, que gerenciasse pacotes de várias versões. Só ganha se houver mais de uma versão do Clonezilla em uso ao mesmo tempo, e a versão continua uma só.
- **Um crate de HTTP.** A porta `Sistema` já o recusa — TLS, resolução de nome e dezenas de caixas por uma transferência —, e o `curl` do `System32` faz o que a decisão 6 pede, sem dependência nova.
- **O `--retry 2` que o adaptador passava ao `curl`.** Ele repetiria a mesma fonte, e uma fonte parada esperaria o limite três vezes. Trocar de fonte continuando do mesmo `.parcial` faz o que ele comprava.

## O que fica como estava

- **PR-1**: a versão `3.3.3-15` e o SHA256 compilado. Ao trocar de versão, as três fontes mudam junto com `VERSAO`, `ARQUIVO`, `SHA256` e `TAMANHO_BYTES`, e o resto é o que o [ADR-0018](0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md) descreve.
- **PR-2**: o `--iso` não lê nem escreve o pacote guardado, e a tela dele é a de antes. Ele também não alimenta o pacote guardado: o próximo preparo sem `--iso` baixa.
- **PR-3**: a cópia no `ARCAVAULT` continua sendo feita depois do passo 5 e da segunda conferência — agora a partir do pacote guardado.
- O passo 7, PR-4 e as sete defesas de PR-5.
- Usar automaticamente uma versão mais nova do Clonezilla, ou avisar que ela existe, continua fora: instalaria um Clonezilla que nunca rodou as receitas do ARCA, e revogaria PR-1.

## Duas lacunas nomeadas, e decididas no mesmo dia

Quando este ADR foi escrito, dois casos ficaram nomeados aqui como lacunas: o checklist da construção os tinha deixado fora do escopo. A verificação da feature mostrou, em 04/10/2026, que isso estreitava os critérios 7 e 8 do issue sem que o usuário tivesse decidido, e ele decidiu os dois no mesmo dia, com as palavras que dão título a cada um. Os dois estão resolvidos no código desde então.

### "Zerar o que não bate"

**A lacuna:** um `.parcial` menor que o pacote, entregue com código 0 e que não bate — um servidor que responde `200` com uma página no lugar do zip, como o Wi-Fi de um hotel que devolve a página de login — seria retomado na execução seguinte, e não recomeçado do zero. A decisão 4 só cobria o caminho comum: um download que chega inteiro e corrompido tem o tamanho do pacote, e recomeça do zero sozinho. A página seria retomada em toda execução, e o ARCA nunca mais sairia dela sem alguém apagar o arquivo à mão.

**Como foi resolvida:** o `.parcial` que chega e não bate com o SHA256 é **zerado na hora**, e a próxima execução baixa do zero, qualquer que seja o tamanho dele. Zerar é a mesma substituição do recomeço, e por isso B-10 continua valendo. "Não consegui medir" não zera: um `.parcial` que o `certutil` não conseguiu resumir fica como está, porque não ter conseguido medir não é o mesmo que não bater.

### "Reconhecer a falha de escrita"

**A lacuna:** uma pasta que já tem um `.parcial` e passou a recusar escrita não era pega no pré-voo, que só escreve nela para zerar. O `curl` falhava em cada uma das três fontes, e a recusa era a de nenhuma fonte ter entregado, dizendo que o que já veio fica guardado e que a próxima tentativa continua de onde parou — o que ali não era verdade.

**Como foi resolvida:** o código 23 do `curl`, a escrita no destino que falhou, deixou de ser uma resposta da fonte. Ele vira a recusa `nao deu para guardar o pacote em <pasta> (curl: (23) ...)` já na primeira fonte, e as outras não são tentadas, porque cairiam no mesmo `.parcial`. A frase entre parênteses é a última linha `curl: (N) ...` do stderr, e não a primeira, pelo que a medição desse dia mostrou: antes dela o `curl` escreve o caminho do arquivo quebrado em três linhas.

A recusa não fica "antes de qualquer download" ao pé da letra, porque uma tentativa começa. Mas, com bytes para retomar, o `curl` falha antes de conectar, o disco continua intacto, e a mensagem diz o que houve.

## Consequências

- **PR-7 entra no §9.6 do PRD e no §13 do README**, que ganha a tabela de preparação inteira, de PR-1 a PR-7.
- O `CONTEXT.md` ganha **Pacote guardado**, distinto da cópia de PR-3.
- `pacote::URL` vira `pacote::FONTES`, e `pacote::LIMITES` guarda os 30 s e os 60 s.
- Três recusas novas, todas no pré-voo e todas terminando em `Nada foi apagado (PR-7)`: nenhuma fonte entregou, a pasta não aceita escrita — inclusive quando quem descobre é o `curl`, com o código 23 —, e falta espaço para o que falta baixar. Elas saem sem o prefixo `o pacote do Clonezilla foi recusado: ` das outras recusas do pacote: "recusado" seria falso para "nenhuma fonte entregou".
- O `arca.log` ganha uma linha por fonte tentada — a URL, o que ela respondeu e quantos bytes vieram dela — e uma quando o pacote guardado é usado sem rede.
- O `.parcial` que chega e não bate com o SHA256 é zerado na hora, antes de a recusa de divergência aparecer.
- O passo 6 da tabela de passos deixa de baixar: o pacote já é local.
