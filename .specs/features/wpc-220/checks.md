# WPC-220 · A fonte lenta perde a vez, o download mostra velocidade e quanto falta, e o `--iso` fica guardado - checks

Profile: light (nenhum `AGENTS.md` declara outro)
Plan: o issue [WPC-220](https://linear.app/wpcsolutions/issue/WPC-220/a-fonte-lenta-perde-a-vez-o-download-mostra-velocidade-e-quanto-falta) - os critérios 1-32 viram C1-C32, pelo número

Base da feature: `c0b4824`. Branch: `carreiro/wpc-220-a-fonte-lenta-perde-a-vez-o-download-mostra-velocidade-e`. Os comandos rodam no Git Bash, a partir da raiz do repositório.

32 checks in 4 slices · 1 one-way door · 5 Unresolved do issue, nenhum decidido aqui (cada critério usa o "Until answered" dele)

Sources:

- [WPC-220](https://linear.app/wpcsolutions/issue/WPC-220/a-fonte-lenta-perde-a-vez-o-download-mostra-velocidade-e-quanto-falta) - os critérios 1-32, States, Out of scope, Observable, Swept, Impact, Decided e Unresolved. **É o registro da decisão**
- `.design/download-do-pacote.md` (commit `c0b4824`) - as Key decisions 1 a 6, confirmadas pelo usuário em 23/10/2026. **Vinculante para a tela**: as tabelas de estado de Andamento, Fontes do pacote e `--iso` trazem as linhas literais que C1-C26 transcrevem
- `docs/adr/0028-nenhum-download-depois-do-ponto-sem-volta.md` - a decisão 6 e a linha de PR-2 de "O que fica como estava", que esta feature revê (C29), e as medições do `curl` de 04/10/2026, que continuam valendo
- `PRD/PRD-ARCA-v5_1.md` §9.6 - PR-1 fica; PR-2 e PR-7 ganham o texto do Decided (C30)
- Medições de 23/10/2026 e a conversa do mesmo dia, transcritas em Sources do issue - entram no ADR-0029 (C28)

## Out of scope

- Atualizar o Clonezilla sozinho, ou avisar que há versão nova - a versão não era a causa; PR-1 fica
- Mais fontes - ganha se, com a regra nova, aquele computador ainda vir as três lentas
- Baixar por várias conexões, ou medir todas as fontes antes de escolher uma - ninguém mediu se uma fonte limita a velocidade por conexão
- Lembrar, entre execuções, qual fonte foi a mais rápida - o `.parcial` já retoma
- A barra do `curl` - continua calada (ADR-0028)
- Mover o arquivo do `--iso`, ou usar a cópia guardada nos passos 6 e 7 - o arquivo é de quem chamou, e o passo 7 já o confere
- Os cinco Unresolved do issue - cada critério segue o "Until answered" dele: a recusa sem a frase nova (1), a linha `Fonte` (2), `parada ha` a partir de 5 s (3), `█` e `░` (4), `lenta: nada veio nos ultimos 20 s` (5). Medir se o console desenha `█` e `░` (4) não foi feito aqui

## Landing

Toca a porta `Sistema` (o `baixar`), as duas implementações dela (o adaptador do `curl` e o `SistemaDeMentira`), um módulo puro novo, `src/andamento.rs` (a medida dos últimos 20 s, a linha viva e o julgamento da fonte lenta), `src/pacote.rs` (os 20 s e os 600 s ao lado de `LIMITES`, a resposta `Lenta` e a recusa que passa a listar fontes), o pré-voo de `src/comandos/prepare.rs` (as duas voltas e a cópia do `--iso`), o `ArquivosEmMemoria` e o `ArquivosQueRecusam` (a cópia anotada, e a recusa que vale também para o destino da cópia), e a documentação. Reusa `conferir_o_pacote` (o SHA256 e os quatro caminhos) na cópia do `--iso`, `formato::linha` e `formato::tamanho` em toda linha nova, a `LinhaViva` da WPC-134 para reescrever no lugar, `escrever_atomico` para zerar o `.parcial`, `copiar` e `renomear` da porta `Arquivos`, e a `Bancada` dos testes do `prepare`.

| One-way door | Literal shape | Alternative rejected |
| --- | --- | --- |
| O contrato de `Sistema::baixar` (Decided do issue) | `fn baixar(&self, url: &str, destino: &Path, limites: Limites, andamento: &mut dyn FnMut(Amostra) -> ControlFlow<()>) -> Resultado<Transferencia>`, com `Amostra { recebido: u64, decorrido: Duration }` em `crate::portas`. `recebido` é o tamanho do destino, e `decorrido` é o tempo monotônico (`Instant`) desde o começo desta chamada, medido pelo adaptador. `ControlFlow::Break(())` faz o adaptador encerrar o `curl` como já encerra a fonte parada, e `Transferencia` ganha `encerrada: bool`, verdadeiro só nesse caso. `Limites` continua `{ conectar, parado }` | **O adaptador julgar a lentidão com um terceiro limite em `Limites`**: a regra só se testaria com tempo de verdade, contra servidores locais, e a tela calcularia o mesmo número por outro caminho (Key decision 3). **O pré-voo medir o tempo pelo `Relogio`**: é hora de parede, e um ajuste de hora do Windows no meio do download daria velocidade negativa. **Um `enum` próprio no lugar de `ControlFlow`**: a biblioteca padrão já dá nome a "continuar ou parar" num callback |

- Nothing else in this change is hard to reverse. Os textos das linhas novas são os literais do issue e do desenho; o módulo `andamento` é código puro na raiz de `src/`, como os outros, e as constantes dos 20 s e dos 600 s moram com `pacote::LIMITES`, como a Key decision 1 manda

### Testes cuja premissa muda por decisão, e não para ficar verdes

- `comandos::prepare::testes::um_parcial_menor_que_o_pacote_e_retomado_de_onde_parou`, `o_andamento_reescreve_a_linha_no_lugar` e `uma_fonte_que_para_e_abandonada_e_a_proxima_continua_do_mesmo_parcial` leem a linha `Baixando Clonezilla` no formato de `c0b4824` (`3.3.3-15 · <recebido> de 535,5 MB · <rotulo>`, com `retomando de` dentro dela). A versão, o rótulo e a retomada vão para a linha `Fonte` (Impact do issue, C1-C3)
- `adaptadores::windows::sistema::testes::o_andamento_chega_ao_menos_a_cada_2_s` passa a cobrar amostras com `decorrido` crescendo (Impact, C3)
- `comandos::prepare::testes::com_iso_o_pacote_guardado_nem_e_olhado` cobrava o critério 12 da WPC-134 — o `--iso` nem olha o pacote guardado —, que C18-C20 revogam: o `--iso` tem de medir o guardado para saber se copia. Ele vira `um_iso_que_passa_com_o_guardado_que_bate_nao_escreve_nada`, a prova de C19 (Impact, domain)
- `comandos::prepare::testes::o_pacote_local_bom_e_conferido_de_novo_depois_do_ponto_sem_volta` esperava `resumidos == [iso, iso]`, e passa a esperar `[iso, .parcial, iso]`: a cópia do `--iso` é conferida no meio (C18). A igualdade continua exata
- `tests/e10_preparar_o_dispositivo.rs` e as capturas de 23/08/2026 não mudam: são evidência

### Bordas que os critérios não nomeiam, decididas aqui

São reversíveis com um commit, e por isso não são portas. Cada uma tem prova em algum teste abaixo.

1. **O literal de uma escrita da linha viva é o que o console mostra.** Uma escrita mais curta que a anterior termina com os espaços que cobrem a sobra dela — é o mecanismo da `LinhaViva` da WPC-134, sem o qual a reescrita no lugar deixaria restos na tela. Vale para C5 e C6, cuja escrita vem depois de uma mais longa; as provas comparam o literal e cobram que o resto seja só espaço, na quantidade exata da sobra
2. **`retomando de` só aparece com o `.parcial` maior que zero, também na segunda volta.** C12 escreve `retomando de <recebido>` sempre, e a segunda volta com o `.parcial` vazio (três fontes que nada mandaram em 20 s) diria `retomando de 0 B`. Rejeitado: escrever `retomando de 0 B`, que é a linha de C1 dizendo uma coisa falsa
3. **Sem velocidade medida, a linha de entrega não diz velocidade.** Uma fonte que entrega sem nenhuma amostra depois da do começo (o duplo, que entrega na hora) não tem os últimos 20 s; a linha fica `[████████████████████] 100% · 535,5 de 535,5 MB`
4. **Antes de 5 s sem crescer, nada nos últimos segundos é `medindo`, e não `0 B/s · faltam` infinito**
5. **Uma cópia do `--iso` que o `certutil` não consegue medir não é zerada** ("não consegui medir" não é "não bate", ADR-0028), e cai na linha de C22, com o motivo do `certutil`
6. **No ensaio**, um guardado que não se deixa medir mostra a linha de C21, e uma pasta com o nome do pacote mostra a de C22, sem aviso no `arca.log`: são os casos em que o preparo de verdade não copia, e a linha de C26 diria que ele copia
7. **`parou` na primeira volta não entra na segunda**: só entram as que "saíram por lentas" (C12). Na primeira volta ela é teórica — os 20 s de C11 vêm antes dos 60 s do adaptador —, mas o duplo a alcança
8. **O texto do `curl` na segunda volta abandona a fonte como os três casos de C13**, com a linha `Fonte abandonada` de hoje. `NaoConseguiuEscrever` continua sendo a recusa `nao deu para guardar o pacote` em qualquer volta (WPC-134)

## Checks

### S1 - Andamento · 6 files · 313 KB · ~78k

**C1** (critério 1) - WHEN o `.parcial` não existe e a primeira fonte começa THEN a tela recebe `  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (oldstable)\n` e, logo em seguida, a primeira escrita da linha viva, sem `\r`, é exatamente `  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   0% · 0,0 de 535,5 MB · medindo`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_que_comeca_do_zero_ganha_a_linha_fonte_e_a_barra_vazia -- --exact`

**C2** (critério 2) - WHEN o `.parcial` tem 125.829.120 bytes e a primeira fonte começa THEN a tela recebe `  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (oldstable) · retomando de 120,0 MB\n` e a primeira escrita da linha viva, sem `\r`, é exatamente `  Baixando Clonezilla ............. [████░░░░░░░░░░░░░░░░]  22% · 120,0 de 535,5 MB · medindo`; e o `.parcial` não é zerado (a única escrita nele é a renomeação)
Proof: `cargo test --lib comandos::prepare::testes::um_parcial_menor_que_o_pacote_e_retomado_de_onde_parou -- --exact`

**C3** (critério 3) - Três partes. (a) O adaptador, contra o servidor local `Devagar` (1000 bytes, 100 a cada 400 ms), chama `andamento` ao menos 3 vezes, com `decorrido` estritamente crescente, nunca mais de 2 s de `decorrido` nem de relógio entre duas chamadas, `recebido` que não diminui e a última em 1000. (b) O pré-voo, dadas as amostras (0 s, 0 B), (10 s, 10.485.760 B) e (30 s, 115.343.360 B) da primeira fonte, escreve a linha viva uma vez antes delas e uma vez por amostra, sem `\n` entre essas escritas, e a escrita da terceira amostra é exatamente `\r  Baixando Clonezilla ............. [████░░░░░░░░░░░░░░░░]  20% · 110,0 de 535,5 MB · 5,0 MB/s · faltam 2 min`. (c) Os arredondamentos: barra e porcentagem para baixo (0 → 0 casas e `  0%`; 125.829.120 → 4 e ` 22%`; 561.478.647 → 19 e ` 99%`; 561.478.648 → 20 e `100%`); quanto falta para cima, por unidade (0,4 s → `1 s`; 59 s → `59 s`; 59,5 s → `1 min`; 85,1 s → `2 min`; 3.599,5 s → `1 h 0 min`; 8.311,4 s → `2 h 19 min`); a velocidade é a do intervalo que começa na última amostra de até 20 s antes da atual, ou no começo da fonte quando ela corre há menos de 20 s
Proof: `cargo test --lib adaptadores::windows::sistema::testes::o_andamento_chega_ao_menos_a_cada_2_s -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::o_andamento_reescreve_a_linha_no_lugar -- --exact`
Proof: `cargo test --lib andamento::testes::a_barra_e_a_porcentagem_arredondam_para_baixo -- --exact`
Proof: `cargo test --lib andamento::testes::quanto_falta_arredonda_para_cima -- --exact`
Proof: `cargo test --lib andamento::testes::a_velocidade_e_a_dos_ultimos_20_s -- --exact`

**C4** (critério 4) - WHEN a medida começa com 15.466.496 bytes no `.parcial` e a amostra de 20 s traz 16.777.216 (1.310.720 nos últimos 20 s) THEN a linha viva, com o prefixo, é exatamente `  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   2% · 16,0 de 535,5 MB · 64,0 KB/s · faltam 2 h 19 min`
Proof: `cargo test --lib andamento::testes::mais_de_uma_hora_sai_em_horas_e_minutos -- --exact`

**C5** (critério 5) - WHEN o pré-voo recebe as amostras (0 s, 0 B), (10 s, 10.485.760 B) e (15 s, 10.485.760 B) THEN a escrita da terceira é `\r  Baixando Clonezilla ............. [░░░░░░░░░░░░░░░░░░░░]   1% · 10,0 de 535,5 MB · 0 B/s · parada ha 5 s` seguida só dos espaços que cobrem a sobra da escrita anterior (borda 1); e WHEN a terceira amostra é (14 s, 10.485.760 B) THEN a linha não diz `parada`
Proof: `cargo test --lib comandos::prepare::testes::a_linha_viva_diz_ha_quanto_tempo_a_fonte_parou -- --exact`
Proof: `cargo test --lib andamento::testes::a_fonte_que_nao_cresce_ha_5_s_diz_ha_quanto_tempo_parou -- --exact`

**C6** (critério 6) - WHEN a primeira fonte entrega depois das amostras (0 s, 0 B), (90 s, 456.621.048 B) e (110 s, 561.478.648 B) THEN a última escrita da linha viva é `\r  Baixando Clonezilla ............. [████████████████████] 100% · 535,5 de 535,5 MB · 5,0 MB/s` seguida só dos espaços da sobra, não contém `faltam`, e a próxima escrita da tela é `\n`; e (andamento) com 561.478.647 bytes a porcentagem é ` 99%` (C3, parte c)
Proof: `cargo test --lib comandos::prepare::testes::a_fonte_que_entrega_deixa_a_linha_em_100_por_cento -- --exact`
Proof: `cargo test --lib andamento::testes::a_barra_e_a_porcentagem_arredondam_para_baixo -- --exact`

**C7** (critério 7) - Para todo `recebido` de uma varredura de 0 a 561.478.648 (as bordas de cada escala de `tamanho` e um passo de 1/1000 do pacote) e toda velocidade de uma varredura de 1 B/s a 1 GB/s (as bordas de cada escala, 1023 e 1024 de cada uma), nos quatro estados da linha (medindo, baixando, parada até 3600 s, entregue), a linha com o prefixo tem no máximo 120 caracteres — e os quatro glifos dela (`█`, `░`, `·`, letras ASCII) ocupam uma coluna cada
Proof: `cargo test --lib andamento::testes::a_linha_viva_cabe_em_120_colunas -- --exact`

### S2 - Fontes do pacote · 4 files · 344 KB · ~86k (11k novos sobre S1)

**C8** (critério 8) - Três partes. (a) A medida que começa com o `.parcial` vazio e tem 1.310.720 bytes aos 20 s, na primeira volta, julga a fonte lenta, e o texto do julgamento é `lenta: faltariam 2 h 23 min a 64,0 KB/s`. (b) O pré-voo, com as amostras (0 s, 0 B) e (20 s, 1.310.720 B) na primeira fonte, devolve `Break` na amostra de 20 s; a tela traz `\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · lenta: faltariam 2 h 23 min a 64,0 KB/s\n`; a fonte seguinte começa com `  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (stable) · retomando de <tamanho(1.310.720)>`; os dois destinos são o mesmo `.parcial`; e as escritas são `[pasta, .parcial (zerado no começo, porque não existia), .parcial, guardado]` — nada o zera entre as duas fontes. (c) O adaptador, quando `andamento` devolve `Break`, encerra o `curl` antes de o corpo terminar: `encerrada` verdadeiro, `parou` falso, e o `.parcial` com um prefixo não vazio e incompleto do corpo
Proof: `cargo test --lib andamento::testes::a_fonte_lenta_e_julgada_pelo_que_falta -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_lenta_perde_a_vez_e_a_seguinte_continua_do_mesmo_parcial -- --exact`
Proof: `cargo test --lib adaptadores::windows::sistema::testes::pedir_para_encerrar_encerra_o_curl_e_o_parcial_fica_com_o_que_veio -- --exact`

**C9** (critério 9) - WHEN a fonte da primeira volta corre há 19,999 s sem nada recebido, e WHEN corre há 19 s com 1 byte THEN o julgamento é continuar; e WHEN o pré-voo recebe (5 s, 1 B), (10 s, 2 B) e (19 s, 3 B) e a fonte entrega THEN nenhum `Break` é devolvido, a tela não traz `Fonte abandonada`, e `baixados == [oldstable]`
Proof: `cargo test --lib andamento::testes::antes_de_20_s_nenhuma_fonte_e_lenta -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_com_menos_de_20_s_nao_sai_por_lenta -- --exact`

**C10** (critério 10) - WHEN a medida começa com 522.977.280 bytes e tem 524.288.000 aos 20 s (1.310.720 nos últimos 20 s; faltam 567 s) THEN o julgamento é continuar e a linha viva traz `64,0 KB/s · faltam 10 min`; e WHEN o pré-voo recebe essas amostras na primeira fonte e ela entrega THEN não há `Fonte abandonada` e `baixados == [oldstable]`
Proof: `cargo test --lib andamento::testes::perto_do_fim_a_fonte_lenta_termina -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_lenta_perto_do_fim_vai_ate_o_fim -- --exact`

**C11** (critério 11) - WHEN a medida começa com 1.048.576 bytes e continua com eles aos 20 s, e WHEN cresce até 1.048.576 aos 5 s e fica nisso até os 25 s THEN, na primeira volta, a fonte sai por lenta, com o texto `lenta: nada veio nos ultimos 20 s`; e o pré-voo escreve `\n  Fonte abandonada ................ free.nchc.org.tw (oldstable) · lenta: nada veio nos ultimos 20 s\n`
Proof: `cargo test --lib andamento::testes::nada_em_20_s_e_lenta -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_que_nada_mandou_em_20_s_sai_por_lenta -- --exact`

**C12** (critério 12) - WHEN, na primeira volta, oldstable sai por lenta a 64,0 KB/s, stable sai por lenta sem nada nos últimos 20 s e o SourceForge sai por lenta a 128,0 KB/s, e na segunda o SourceForge entrega com amostras que na primeira volta seriam lentas THEN `baixados == [oldstable, stable, sourceforge, sourceforge]`, nenhum `Break` é devolvido na segunda volta, e a tela traz `  Fonte ........................... 3.3.3-15 · downloads.sourceforge.net · segunda volta · retomando de <recebido>`; WHEN oldstable e o SourceForge saem por lentas à mesma velocidade e stable responde 404, e as duas da segunda volta não entregam THEN a segunda volta é `[oldstable, sourceforge]`, na ordem de `FONTES`
Proof: `cargo test --lib comandos::prepare::testes::todas_lentas_a_mais_rapida_medida_volta_na_segunda_volta -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::na_segunda_volta_o_empate_segue_a_ordem_das_fontes -- --exact`

**C13** (critério 13) - WHEN as três saem por lentas na primeira volta e, na segunda, a primeira (a mais rápida) para por 60 s, a segunda não responde em 30 s e a terceira responde 404 THEN cada uma ganha a linha `Fonte abandonada` de hoje — `· parou de mandar por 60 s, com <recebido> de 535,5 MB`, `· nao respondeu em 30 s` e `· HTTP 404` —, a seguinte da segunda volta é tentada depois de cada uma, e o pré-voo termina em `NenhumaFonteEntregou`
Proof: `cargo test --lib comandos::prepare::testes::na_segunda_volta_a_fonte_que_falha_da_a_vez_a_seguinte -- --exact`

**C14** (critério 14) - WHEN oldstable não responde, stable responde 404 e o SourceForge sai com o texto do `curl` (`curl: (6) Could not resolve host`) THEN não há segunda volta (`baixados` tem três entradas); e WHEN oldstable sai por lenta, stable responde 404 e o SourceForge não responde THEN a segunda volta é só `[oldstable]`
Proof: `cargo test --lib comandos::prepare::testes::a_fonte_que_saiu_sem_ser_lenta_nao_volta -- --exact`

**C15** (critério 15) - O texto da recusa continua o de `c0b4824` (Unresolved 1), com uma linha por fonte; e WHEN `executar` roda com as três saindo como em C13 THEN o erro é `PacoteRecusado(NenhumaFonteEntregou)`, `enumeracoes == 0`, e o texto é exatamente `nao deu para baixar o Clonezilla 3.3.3-15 de nenhuma das 3 fontes:\n` seguido de uma linha `  <rotulo> — <resposta>` por fonte, na ordem de `FONTES`, cada uma com a resposta da segunda volta, e do resto do texto de hoje
Proof: `cargo test --lib pacote::testes::a_recusa_de_nenhuma_fonte_e_o_texto_aprovado -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::a_recusa_lista_cada_fonte_com_a_ultima_resposta -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::nenhuma_fonte_entregou_recusa_antes_dos_discos_e_guarda_o_que_veio -- --exact`

**C16** (critério 16) - WHEN oldstable sai por lenta a 64,0 KB/s com 1.310.720 bytes, stable sai por lenta sem nada, o SourceForge não responde, e oldstable entrega na segunda volta THEN o `arca.log` tem, nesta ordem, `fonte <oldstable> · lenta: faltariam 2 h 23 min a 64,0 KB/s · 1310720 bytes desta fonte`, `fonte <stable> · lenta: nada veio nos ultimos 20 s · 0 bytes desta fonte`, `fonte <sourceforge> · nao respondeu em 30 s · 0 bytes desta fonte` e `fonte <oldstable> · entregou · 560167928 bytes desta fonte`; e as linhas das fontes que não saem por lentas continuam as de hoje
Proof: `cargo test --lib comandos::prepare::testes::a_fonte_lenta_fica_no_arca_log_nas_duas_voltas -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::cada_fonte_tentada_fica_no_arca_log -- --exact`

**C17** (critério 17) - `FONTES` são as três de hoje, na ordem de hoje; `LIMITES` é `{ conectar: 30 s, parado: 60 s }`, e o `curl` recebe `--connect-timeout 30`; e WHEN as três saem por lentas na primeira volta e saem com o texto do `curl` na segunda THEN `baixados` tem seis entradas, a primeira volta na ordem de `FONTES`, e cada URL aparece exatamente duas vezes
Proof: `cargo test --lib pacote::testes::as_fontes_sao_as_tres_na_ordem_e_carregam_a_versao -- --exact`
Proof: `cargo test --lib pacote::testes::os_limites_de_pr_7_continuam_30_e_60_s -- --exact`
Proof: `cargo test --lib adaptadores::windows::sistema::testes::o_curl_recebe_os_limites_e_retoma -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::nenhuma_fonte_e_tentada_mais_de_duas_vezes -- --exact`

### S3 - `--iso` · 6 files · 289 KB · ~72k (4k novos sobre S2)

**C18** (critério 18) - WHEN o `--iso` passa e não há pacote guardado THEN o pré-voo devolve `DoIso(<iso>)`, `enumeracoes == 0`, as escritas são exatamente `[pasta, .parcial, .parcial, guardado]` (criar a pasta, copiar, renomear), a cópia sai do `--iso` para o `.parcial`, `resumidos == [<iso>, .parcial]`, o guardado tem o conteúdo do `--iso`, o `.parcial` não existe mais, o `--iso` não foi escrito e tem o conteúdo de antes, e a tela é exatamente `  Pacote conferido ................ ok · 00cee7700433 · <iso> · nada tocado ainda (PR-2)\n  Pacote guardado ................. copiado do --iso para <guardado> · o proximo preparo nao baixa (PR-2)\n`
Proof: `cargo test --lib comandos::prepare::testes::um_iso_que_passa_vira_o_pacote_guardado -- --exact`

**C19** (critério 19) - WHEN o `--iso` passa e o guardado bate THEN, depois do comando inteiro, nenhum caminho dentro da pasta do pacote guardado foi escrito, `baixados` está vazio, o disco foi particionado, a tela do pré-voo é exatamente a linha `Pacote conferido` de C18, e a linha do Clonezilla no plano é exatamente `  Clonezilla 3.3.3-15 · do arquivo local <iso> (PR-2)\n`
Proof: `cargo test --lib comandos::prepare::testes::um_iso_que_passa_com_o_guardado_que_bate_nao_escreve_nada -- --exact`

**C20** (critério 20) - WHEN o `--iso` passa e o guardado não bate THEN `resumidos == [<iso>, guardado, .parcial]`, as escritas são `[pasta, .parcial, .parcial, guardado]` — o guardado só é escrito pela renomeação, depois de o `.parcial` passar —, o guardado termina com o conteúdo do `--iso`, e a segunda linha da tela é exatamente `  Pacote guardado ................. nao batia com o SHA256 deste ARCA · substituido pela copia do --iso\n`; e WHEN a cópia também não bate THEN o guardado continua com o conteúdo de antes
Proof: `cargo test --lib comandos::prepare::testes::um_iso_que_passa_substitui_o_guardado_que_nao_bate_depois_de_conferido -- --exact`

**C21** (critério 21) - WHEN o `--iso` passa e o `certutil` não resume o guardado, e WHEN o Windows não deixa olhar o guardado THEN nada é escrito na pasta, nada é copiado, e a segunda linha da tela é exatamente `  Pacote guardado ................. nao deu para medir o que ja esta guardado (<motivo>) · deixado como esta\n`, com o motivo do `certutil` no primeiro caso e o do Windows no segundo
Proof: `cargo test --lib comandos::prepare::testes::um_guardado_que_nao_se_deixa_medir_nao_e_tocado_pelo_iso -- --exact`

**C22** (critério 22) - WHEN o `--iso` passa e (1) a pasta não se deixa criar, (2) o `.parcial` não aceita a cópia, (3) o volume tem 561.478.647 bytes livres, (4) há uma pasta com o nome do pacote, ou (5) o `certutil` não resume a cópia (borda 5) THEN o pré-voo devolve `DoIso(<iso>)`, a segunda linha da tela é exatamente `  Pacote guardado ................. nao deu para guardar em <pasta> (<motivo>) · este preparo segue do --iso\n`, o `arca.log` ganha um `AVISO` com o mesmo motivo, e o guardado não existe; os motivos são `Acesso negado. (os error 5)` em (1) e (2), `precisa de 536 MB livres, e ha 535 MB` em (3) e `` `clonezilla-live-3.3.3-15-amd64.zip` e uma pasta `` em (4); em (3) e (4) nada é copiado; e WHEN o volume tem exatamente 561.478.648 bytes livres THEN a cópia acontece
Proof: `cargo test --lib comandos::prepare::testes::uma_copia_do_iso_que_nao_se_faz_vira_aviso_e_o_preparo_segue -- --exact`

**C23** (critério 23) - WHEN o `--iso` passa e a cópia no `.parcial` não bate THEN o `.parcial` termina vazio, e a escrita que o zera vem depois da conferência dele; o guardado não existe; o pré-voo devolve `DoIso(<iso>)`; e a segunda linha da tela é exatamente `  Pacote guardado ................. a copia nao bateu com o SHA256 · este preparo segue do --iso\n`
Proof: `cargo test --lib comandos::prepare::testes::uma_copia_do_iso_que_nao_bate_e_zerada_e_o_preparo_segue -- --exact`

**C24** (critério 24) - WHEN `executar` roda com um `--iso` que passa, sem pacote guardado, até depois do passo 8 THEN `resumidos == [<iso>, .parcial, <iso>]` (o pré-voo, a cópia e o passo 7), a extração sai do `--iso`, a cópia de PR-3 no `ARCAVAULT` sai do `--iso`, e `baixados` está vazio; e o teste de `c0b4824` do `--iso` bom continua cobrando as duas conferências do `--iso`, com a da cópia no meio
Proof: `cargo test --lib comandos::prepare::testes::com_iso_o_passo_7_a_extracao_e_a_copia_saem_do_iso -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::o_pacote_local_bom_e_conferido_de_novo_depois_do_ponto_sem_volta -- --exact`

**C25** (critério 25) - WHEN o `--iso` é recusado no pré-voo por `NaoEstaLa`, `EUmaPasta`, `NaoDeuParaOlhar`, `ResumoDivergente` ou `PacoteIncompleto` THEN o erro é essa variante e nenhum caminho dentro da pasta do pacote guardado foi escrito; os textos de hoje continuam provados pelos testes de cada recusa
Proof: `cargo test --lib comandos::prepare::testes::um_iso_recusado_no_pre_voo_nao_escreve_na_pasta_do_arca -- --exact`

**C26** (critério 26) - WHEN `--dry-run`, o `--iso` passa e não há guardado, e WHEN o guardado não bate THEN nada é escrito e a tela é exatamente a linha `Pacote conferido` de C18 seguida de `  Pacote guardado ................. o preparo de verdade copia o --iso para este PC; o ensaio nao escreve\n`; WHEN o guardado bate THEN a tela é só a linha `Pacote conferido`; e WHEN o guardado não se deixa medir, e WHEN há uma pasta com o nome dele THEN nada é escrito, e a segunda linha é a de C21 e a de C22, respectivamente (borda 6)
Proof: `cargo test --lib comandos::prepare::testes::o_ensaio_com_iso_nao_copia_e_diz_que_o_preparo_copia -- --exact`

**C27** (critério 27) - Nenhum código fora dos testes apaga arquivo, e a porta `Arquivos` não declara exclusão; e WHEN a pasta tem `arca.log`, `clonezilla-live-3.3.2-1-amd64.zip` e um guardado que não bate, e o `--iso` passa THEN os dois primeiros continuam com o conteúdo de antes, a pasta termina com exatamente esses três nomes, e as escritas são só a pasta, o `.parcial` e o guardado
Proof: `cargo test --test b10_nada_e_apagado`
Proof: `cargo test --lib comandos::prepare::testes::a_copia_do_iso_nao_mexe_nos_outros_arquivos_da_pasta -- --exact`

### S4 - A regra escrita · 5 files · 378 KB · ~94k

Os comandos abaixo saem com 0 quando o check passa. `A=docs/adr/0029-a-fonte-lenta-perde-a-vez-e-o-iso-fica-guardado.md`.

**C28** (critério 28) - O ADR-0029 existe, com exatamente seis decisões numeradas (`### 1.` a `### 6.`), cada uma com a Key decision de mesmo número do desenho — 1 diz `20 s` e `10 minutos`; 2 diz `segunda volta`; 3 diz que a tela mostra o mesmo número que decide; 4 diz `ControlFlow` e `monot` (monotônico); 5 diz `.parcial` e que o arquivo do `--iso` fica onde está; 6 diz `aviso` —, uma seção com as medições de 23/10/2026 (`5,1 MB/s`, `111 s`, `63,9 KB/s`, `800 Mb`, `0,4 MB/s`, `1,2 MB/s`, `3.3.3-37`) e uma seção `## O que fica do ADR-0028`
Proof: `A=docs/adr/0029-a-fonte-lenta-perde-a-vez-e-o-iso-fica-guardado.md; test "$(grep -cE '^### [1-6]\. ' $A)" = 6 && test "$(grep -cE '^### [0-9]+\. ' $A)" = 6 && awk '/^### 1\. /,/^### 2\. /' $A | grep -q '20 s' && awk '/^### 1\. /,/^### 2\. /' $A | grep -q '10 minutos' && awk '/^### 2\. /,/^### 3\. /' $A | grep -q 'segunda volta' && awk '/^### 3\. /,/^### 4\. /' $A | grep -qi 'mesmo número' && awk '/^### 4\. /,/^### 5\. /' $A | grep -q 'ControlFlow' && awk '/^### 4\. /,/^### 5\. /' $A | grep -qi 'monot' && awk '/^### 5\. /,/^### 6\. /' $A | grep -q '\.parcial' && awk '/^### 5\. /,/^### 6\. /' $A | grep -qi 'fica onde está' && awk '/^### 6\. /,/^## /' $A | grep -qi 'aviso' && for t in '5,1 MB/s' '111 s' '63,9 KB/s' '800 Mb' '0,4 MB/s' '1,2 MB/s' '3.3.3-37'; do grep -qF -- "$t" $A || exit 1; done && grep -qE '^## O que fica do ADR-0028' $A`

**C29** (critério 29) - O ADR-0028 cita o ADR-0029 dentro da decisão 6 e na linha de PR-2 de "O que fica como estava", e nenhuma linha dele é apagada, salvo a linha de PR-2 reescrita no lugar
Proof: `B=docs/adr/0028-nenhum-download-depois-do-ponto-sem-volta.md; awk '/^### 6\. /,/^## As quatro portas/' $B | grep -q '0029' && grep -E '^- \*\*PR-2\*\*' $B | grep -q '0029' && test "$(git diff -U0 c0b4824 -- $B | grep -E '^-[^-]' | grep -vcE '^-- \*\*PR-2\*\*')" = 0`

**C30** (critério 30) - A linha PR-7 do PRD §9.6 traz, literal, o texto do Decided e a anotação de 23/10/2026; a linha PR-2 traz a frase nova e a anotação dela; as anotações de hoje continuam; e as linhas **PR-7** e **PR-2** da tabela de requisitos do README dizem o mesmo
Proof: `P=PRD/PRD-ARCA-v5_1.md; L7=$(grep -E '^\| PR-7 \|' $P); L2=$(grep -E '^\| PR-2 \|' $P); grep -qF 'Uma fonte que, na velocidade dos últimos 20 s, levaria mais de 10 minutos para terminar perde a vez; se todas forem lentas, a mais rápida medida volta e vai até o fim. A tela mostra a barra, a porcentagem, quanto veio, a velocidade e quanto falta.' <<<"$L7" && grep -qF '23/10/2026: a fonte lenta e o andamento nascem de um download a 64 KB/s numa conexão de 800 Mb. Ver' <<<"$L7" && grep -qF '0029' <<<"$L7" && grep -qF '04/10/2026, WPC-134' <<<"$L7" && grep -qF 'O arquivo que passa no SHA256 é copiado para o pacote guardado deste PC, e o próximo preparo sem --iso não baixa.' <<<"$L2" && grep -qF '23/10/2026: antes, o --iso não alimentava o pacote guardado, e a cópia manual se repetia a cada preparo. Ver' <<<"$L2" && grep -qF '0029' <<<"$L2" && grep -qF 'Etapa E10' <<<"$L2"`
Proof: `R7=$(grep -E '^\| \*\*PR-7\*\* \|' README.md); R2=$(grep -E '^\| \*\*PR-2\*\* \|' README.md); for t in '20 s' '10 minutos' 'mais rápida' 'barra' 'quanto falta'; do grep -qF -- "$t" <<<"$R7" || exit 1; done && grep -qF 'pacote guardado' <<<"$R2" && grep -qF 'não baixa' <<<"$R2"`

**C31** (critério 31) - No README: (1) a segunda saída de `nao deu para baixar o Clonezilla` diz que o arquivo passado com `--iso` fica guardado e que o próximo preparo não baixa, e não diz mais que ele volta a baixar; (2) a linha do pacote guardado na tabela de arquivos do `C:` diz que o `--iso` também o alimenta; (3) as telas de exemplo do README e do PRD trazem a linha `Fonte` e a linha `Baixando Clonezilla` com a barra, no estado em que ficam no fim, e não trazem mais a linha de `c0b4824`
Proof: `S=$(awk '/^### `nao deu para baixar o Clonezilla/,/^### `nao deu para guardar/' README.md | grep -E '^2\. \*\*Baixe pelo navegador\*\*'); grep -qF -- '--iso' <<<"$S" && grep -qi 'fica guardado' <<<"$S" && grep -qi 'não baixa' <<<"$S" && ! grep -qi 'volta a baixar' <<<"$S"`
Proof: `grep -F '| `%LOCALAPPDATA%\ARCA\clonezilla-live-3.3.3-15-amd64.zip` |' README.md | grep -qF -- '--iso'`
Proof: `for f in README.md PRD/PRD-ARCA-v5_1.md; do grep -qF '  Fonte ........................... 3.3.3-15 · free.nchc.org.tw (oldstable)' $f && grep -qF '  Baixando Clonezilla ............. [████████████████████] 100% · 535,5 de 535,5 MB · ' $f && ! grep -qF '  Baixando Clonezilla ............. 3.3.3-15 · 535,5 MB de 535,5 MB' $f || exit 1; done`

**C32** (critério 32) - A entrada **Pacote guardado** do `CONTEXT.md` diz que o `--iso` que passa no SHA256 também o alimenta
Proof: `awk '/^\*\*Pacote guardado\*\*:/,/^_Evitar_/' CONTEXT.md | grep -F -- '--iso' | grep -q 'SHA256'`

## Swept

- validation: C18, C20, C23 - o SHA256 compilado decide a cópia; C9 - nenhuma fonte é julgada com menos de 20 s
- failure modes: C13, C15, C21, C22, C23; bordas 5 e 8
- idempotency and retry: C19 - um guardado que bate não é reescrito; C8 e C12 - trocar de fonte continua do byte em que o `.parcial` está
- authorization: existing - o ARCA roda elevado, e a cópia vai para a pasta do `arca.log` do usuário que roda
- concurrency and ordering: C12 - a ordem da segunda volta é a velocidade medida, e o empate segue `FONTES`; C17 - no máximo duas tentativas por fonte. Dois `arca prepare` no mesmo PC: n/a, como na WPC-134
- data lifecycle: C27
- external-dependency failure: C8, C11, C13, C14, C15
- state transitions: a fonte - C8, C11, C12, C13, C14; o arquivo do pacote - C18, C20, C23 (ver Coverage)
- observability: C16 e C22 - cada tentativa, inclusive a lenta, e cada cópia que não se fez ficam no `arca.log`; C3 e C5 - a tela diz a velocidade, quanto falta e há quanto tempo a fonte parou

## Coverage

| Set (size) | Member -> proof | Unproven |
| --- | --- | --- |
| o que a fonte responde na primeira volta (8) | `Entregou` C1, C6 · `Lenta` faltariam C8 · `Lenta` nada veio C11 · `Parou` existing (`uma_fonte_que_para_e_abandonada…`), borda 7 · `NaoRespondeu` C14 · `Http` C14 · `Outra` C14 · `NaoConseguiuEscrever` existing (`um_parcial_que_o_curl_nao_consegue_escrever…`) | - |
| o que a fonte responde na segunda volta (5) | `Entregou` C12 · `Parou` C13 · `NaoRespondeu` C13 · `Http` C13 · `Outra` C17, borda 8 | - |
| a linha viva (4 estados) | medindo C1, C2 · baixando C3, C4 · parada C5 · entregue C6 · os quatro na largura C7 | - |
| o guardado diante de um `--iso` que passa (5) | ausente C18 · bate C19 · não bate C20 · não se deixa medir C21 · pasta com o nome C22(4) | - |
| o mesmo no ensaio (5) | ausente C26 · bate C26 · não bate C26 · não se deixa medir C26 · pasta C26 | - |
| a cópia que não se faz (6) | pasta C22(1) · `.parcial` C22(2) · espaço C22(3) · pasta com o nome C22(4) · a cópia não se mede C22(5) · a cópia não bate C23 | a renomeação recusada: cai no mesmo braço de C22 pelo código, sem teste próprio |
| a recusa do `--iso` no pré-voo (5) | `NaoEstaLa` · `EUmaPasta` · `NaoDeuParaOlhar` · `ResumoDivergente` · `PacoteIncompleto`, as cinco em C25 | - |

- Nenhum check afirma código de saída, rota ou forma de resposta HTTP: as respostas das fontes cruzam a fronteira no adaptador (C3, C8, C17) e no duplo (o resto)

## Handoff

S1-S3 = ~93k, todos em `src/` (o pré-voo, a porta, o adaptador e os duplos). S4 entra na documentação (o ADR novo, o ADR-0028, o PRD, o README e o `CONTEXT.md`) com ~94k próprios, 187k no total, acima dos 150k: **handoff depois de S3**, onde a superfície muda. S1-S3 constrói quem escreveu este checklist; S4 vai para um agente novo, que lê este arquivo e o diff `c0b4824..HEAD`. O Verifier é despachado por quem orquestra, depois de S4, sobre `c0b4824..HEAD` inteiro.
