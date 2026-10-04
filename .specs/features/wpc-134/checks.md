# WPC-134 · O `arca prepare` obtém o Clonezilla no pré-voo, guarda o pacote no PC e não trava calado - checks

Profile: light (nenhum `AGENTS.md` declara outro)
Plan: o issue [WPC-134](https://linear.app/wpcsolutions/issue/WPC-134/o-arca-prepare-obtem-o-clonezilla-no-pre-voo-guarda-o-pacote-no-pc-e) - os critérios 1-26 viram C1-C26, pelo número

Base da feature: `378143b`. Os comandos rodam no Git Bash, a partir da raiz do repositório.

26 checks in 3 slices · 4 one-way doors · 0 open

Sources:

- [WPC-134](https://linear.app/wpcsolutions/issue/WPC-134/o-arca-prepare-obtem-o-clonezilla-no-pre-voo-guarda-o-pacote-no-pc-e) - os critérios 1-26, States, Out of scope, Observable, Swept, Impact, Decided e Unresolved (vazio). **É o registro da decisão**, e os textos literais dos critérios 6 e 8 estão aprovados como estão (comentário de 04/10/2026 no issue)
- `.design/pacote-guardado.md` (commits `1612944` e `378143b`) - as Key decisions 1 a 7, que o ADR-0028 registra (C23), e as tabelas de estado das duas fatias
- `PRD/PRD-ARCA-v5_1.md` §9.6 - PR-1 a PR-6 ficam como estão; PR-7 entra (C22)
- `docs/adr/0018-o-pacote-e-o-zip-e-o-prepare-desarma-o-que-instala.md` - por que a versão é fixada
- [WPC-66](https://linear.app/wpcsolutions/issue/WPC-66/sem-iso-o-pre-voo-nao-tem-o-que-conferir-cobra-menos-do-que-o-nome) - o teste `sem_iso_o_pre_voo_nao_tem_o_que_conferir`, cuja premissa ("uma vez so, depois do ponto sem volta") muda por decisão (Impact do issue), e não para ficar verde
- Documentação do `curl` (Context7, `/curl/curl`, 04/10/2026) - `--connect-timeout`, `--speed-time` ("does not affect slow connects"), `-C -`, `--fail` (código 22), `--write-out` `http_code` ("the last retrieved transfer")
- **Medição de 04/10/2026, nesta máquina**, com o `curl.exe` 8.21.0 do `System32` contra servidores locais (`scratchpad/medir-curl`, resultado transcrito no ADR-0028 por C23):
  - fonte que entrega 100 bytes e para: `--speed-time 2` abandona em 8,0 s, `6` em 12,0 s, `12` em 18,0 s — **o `curl` abandona `T + 6 s` depois do último byte**; com `60`, seriam 66 s, e o critério 18 diz 60
  - sem `-N`, o arquivo de destino fica com **0 bytes** enquanto chegam 100 bytes a cada 0,5 s, e matar o `curl` perde o que estava no buffer; com `-N`, ele cresce 100, 200, 300, 400 e fica com 400 depois de morto
  - conexão que não completa (`192.0.2.1`): código 28, `http_code` `000`; a mesma depois de um `302`: código 28, `http_code` `302`
  - 404 direto e depois de um `302`: código 22, `http_code` `404`, e nenhum arquivo criado
  - `-C -` com 100 bytes no destino: pedido com `Range: bytes=100-`, destino com 1000 bytes; `-C -` sem arquivo ou com arquivo de 0 bytes: pedido sem `Range`, destino com 1000 bytes
  - servidor que ignora o `Range`: código 33, destino intocado; porta fechada: código 7

## Out of scope

- Usar automaticamente uma versão mais nova do Clonezilla, ou avisar que ela existe - decidido no design: a versão continua fixada e muda à mão, sob pedido
- Limpar pacotes guardados de versões antigas - B-10
- O `--iso` alimentar o pacote guardado - PR-2 fica como está (C12)
- A velocidade dos servidores
- A tela de exemplo do §7.1 do PRD, que também mostra o download depois do particionamento - o critério 25 nomeia o README, e o PRD só ganha a linha PR-7 (C22). Fica como achado no relatório final
- Uma pasta que **já tem** um `.parcial` e passou a recusar escrita: o pré-voo só escreve nela quando o `.parcial` não existe ou recomeça do zero (ver Landing, linha 3), e nesse caso o `curl` falha em cada fonte com o texto dele (critério 6). C8 cobre a pasta que não se deixa criar e o `.parcial` que não se deixa criar nem zerar
- Um `.parcial` menor que o pacote, entregue com código 0 e que não bate (um servidor que responde `200` com uma página no lugar do zip): a execução seguinte o retomaria, e não recomeçaria do zero. O critério 7 fala do pacote recém-baixado, que o `curl` só dá por entregue quando chega o `Content-Length` inteiro; o caso menor fica como achado no relatório final

## Landing

Toca as portas `Sistema` (o `baixar`) e `Arquivos` (o `renomear` novo), as três implementações de cada (adaptador, `ArquivosEmMemoria`/`SistemaDeMentira`, `ArquivosQueRecusam`), `src/pacote.rs` (as fontes, o julgamento do que cada fonte respondeu, o tamanho que decide entre retomar e recomeçar, as três recusas novas), `src/registro.rs` (a pasta), o pré-voo e os passos 6 e 7 de `src/comandos/prepare.rs`, e a documentação. Reusa `conferir_o_pacote` (SHA256 e os quatro caminhos) nas duas conferências, `formato::linha` e `formato::tamanho` em toda linha nova, `escrever_atomico` para criar e zerar o `.parcial`, e a `Bancada` dos testes do `prepare`.

O pré-voo passa a escrever na tela por um `&mut dyn FnMut(&str)`, que em produção é `print!` com `flush` e nos testes junta o texto: é o que deixa C2, C3, C4, C9, C10 e C16-C19 provarem o texto da tela, e não só o comportamento.

| One-way door | Literal shape | Alternative rejected |
| --- | --- | --- |
| A porta `Arquivos` ganha `renomear` (Decided do issue) | `fn renomear(&self, origem: &Path, destino: &Path) -> Resultado<()>`, substituindo o destino se ele existir (`fs::rename`, que no Windows é `MoveFileEx` com `MOVEFILE_REPLACE_EXISTING`); a porta continua sem método de exclusão | um nome só, com o tamanho decidindo entre parcial e completo: um download pela metade ficaria com o nome do pacote |
| O contrato de `Sistema::baixar` | `fn baixar(&self, url: &str, destino: &Path, limites: Limites, andamento: &mut dyn FnMut(u64)) -> Resultado<Transferencia>`, com `Limites { conectar: Duration, parado: Duration }` e `Transferencia { codigo: i32, texto: String, http: Option<u16>, parou: bool, tamanho_no_destino: u64 }`. Sempre continua do fim de `destino` (`-C -`); o `curl` cuida da conexão (`--connect-timeout`); o adaptador lê o tamanho de `destino` (com `-N`) a cada volta, chama `andamento` ao menos a cada 1 s, e encerra o `curl` depois de `parado` sem o destino crescer, devolvendo `parou: true`. `http` sai do `-w` que o próprio adaptador pede, e `None` é o `000` | `--speed-limit 1 --speed-time 60`, que era o que o Decided supunha: medido em 04/10/2026, o `curl` abandona `T + 6 s` depois do último byte — 66 s, e não os 60 do critério 18 —, e sem `-N` o tamanho lido do destino fica em 0 enquanto os bytes chegam, o que tiraria do andamento a verdade. E ler o andamento da barra de progresso do `curl` no stderr: é texto sem contrato, e a porta inteira foi desenhada para não julgar texto |
| O pacote guardado e o `.parcial` moram na pasta do registro (Decided do issue: `%LOCALAPPDATA%\ARCA\`) | `Registro::pasta(&self) -> &Path`, a mesma pasta de `arca.log`; `pacote::guardado_em(pasta)` e `pacote::parcial_em(pasta)` dão os dois caminhos, `clonezilla-live-3.3.3-15-amd64.zip` e `clonezilla-live-3.3.3-15-amd64.zip.parcial` | um campo novo no `Contexto`, em nove construções: duas fontes para a mesma pasta, que só precisam divergir uma vez — e o issue a define como "a mesma pasta do `arca.log`", inclusive no caso sem `LOCALAPPDATA` |
| Recomeçar do zero é zerar o `.parcial` antes do `curl` | `escrever_atomico(parcial, "")` quando o `.parcial` não existe, tem `TAMANHO_BYTES` ou mais, ou o pacote guardado não bateu (só na primeira fonte); e essa escrita é a prova de escrita do critério 8, antes de qualquer download | chamar o `curl` sem `-C -`, que trunca sozinho: a falta de escrita só apareceria durante o download, com o texto do `curl` em cada uma das três fontes |

- Nothing else in this change is hard to reverse. As fontes compiladas e a linha PR-7 são do Decided do issue e entram no formato literal dele

## Checks

### S1 - Pacote guardado · 8 files · 263 KB · ~66k

**C1** (critério 1) - WHEN não há pacote guardado e `executar` roda com `--dispositivo 1`, sem `--iso`, com a primeira fonte entregando THEN o `curl` é chamado com destino `<pasta do registro>\clonezilla-live-3.3.3-15-amd64.zip.parcial`, o `.parcial` é resumido e listado antes de qualquer `enumerar` do particionador, depois renomeado para `<pasta>\clonezilla-live-3.3.3-15-amd64.zip`, e só então o comando enumera os discos e particiona; e WHEN o `.parcial` baixado não tem o `bootx64.efi` THEN a recusa é `PacoteIncompleto`, com `enumeracoes == 0`, `particionou()` falso e o pacote guardado ausente
Proof: `cargo test --lib comandos::prepare::testes::sem_pacote_guardado_ele_e_baixado_conferido_e_renomeado_antes_dos_discos -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_baixado_sem_o_que_boota_nao_chega_aos_discos -- --exact`

**C2** (critério 2) - WHEN há pacote guardado cujo SHA256 bate e o pré-voo roda sem `--iso` THEN `sistema.baixados` fica vazio e a tela do pré-voo é exatamente `  Pacote conferido ................ ok · 00cee7700433 · guardado neste PC em <guardado> · nada tocado ainda (PR-7)\n`
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_guardado_que_bate_e_usado_sem_rede -- --exact`

**C3** (critério 3) - WHEN o pacote guardado não bate THEN a tela traz `  Pacote guardado ................. nao bate com o SHA256 deste ARCA · baixando de novo, do zero\n` antes da primeira linha `Baixando Clonezilla`, o `.parcial` que já existia é zerado antes do download, e o guardado passa a ter o conteúdo do `.parcial` só depois de ele passar; e WHEN o pacote novo também não bate THEN a recusa é `ResumoDivergente` e o guardado continua com o conteúdo antigo
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_guardado_que_nao_bate_e_baixado_de_novo_do_zero -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_guardado_que_nao_bate_so_e_substituido_quando_o_novo_passa -- --exact`

**C4** (critério 4) - WHEN há um `.parcial` com 125.829.120 bytes (120,0 MB), menos que 561.478.648 THEN ele não é zerado (o guardado termina com o conteúdo dele), e a primeira escrita da linha `Baixando Clonezilla` é `  Baixando Clonezilla ............. 3.3.3-15 · retomando de 120,0 MB · free.nchc.org.tw (oldstable)`. Que o `curl` continua do byte em que o arquivo termina é C18, segunda prova
Proof: `cargo test --lib comandos::prepare::testes::um_parcial_menor_que_o_pacote_e_retomado_de_onde_parou -- --exact`

**C5** (critério 5) - O tamanho decide, sobre todo o espaço de entrada: ausente → do zero; 0 e 561.478.647 → retomar daquele byte; 561.478.648 e 561.478.649 → do zero. E WHEN o `.parcial` tem 561.478.648 bytes, e WHEN tem 561.478.649 THEN ele é zerado antes do download
Proof: `cargo test --lib pacote::testes::o_tamanho_compilado_decide_entre_retomar_e_recomecar -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::um_parcial_do_tamanho_do_pacote_ou_maior_recomeca_do_zero -- --exact`

**C6** (critério 6) - O texto da recusa é, byte a byte, o aprovado: `nao deu para baixar o Clonezilla 3.3.3-15 de nenhuma das 3 fontes:\n  free.nchc.org.tw (oldstable) — nao respondeu em 30 s\n  free.nchc.org.tw (stable) — parou de mandar por 60 s, com 120,0 MB de 535,5 MB\n  downloads.sourceforge.net — HTTP 404\nO que ja veio fica guardado, e a proxima tentativa continua de onde parou. Para nao depender do download do ARCA, baixe pelo navegador\n  https://free.nchc.org.tw/clonezilla-live/oldstable/3.3.3-15/clonezilla-live-3.3.3-15-amd64.zip\ne passe o arquivo com --iso. Nada foi apagado (PR-7)`; e WHEN as três fontes falham assim THEN `executar` devolve `PacoteRecusado(NenhumaFonteEntregou)` com essa mensagem, `enumeracoes == 0`, `particionou()` falso, o `.parcial` continua lá e o guardado não existe
Proof: `cargo test --lib pacote::testes::a_recusa_de_nenhuma_fonte_e_o_texto_aprovado -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::nenhuma_fonte_entregou_recusa_antes_dos_discos_e_guarda_o_que_veio -- --exact`

**C7** (critério 7) - WHEN a fonte entrega e o `.parcial` não bate com o SHA256 THEN `executar` devolve `PacoteRecusado(ResumoDivergente)`, com `enumeracoes == 0`, `particionou()` falso, o `.parcial` ainda lá e o guardado ausente. A execução seguinte recomeçar do zero é C5: o `curl` só sai com 0 depois do `Content-Length` inteiro, e o `.parcial` tem então o tamanho do pacote
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_recem_baixado_que_nao_bate_recusa_antes_dos_discos -- --exact`

**C8** (critério 8) - Os dois textos são, byte a byte, `nao deu para guardar o pacote em C:\Users\Ana Paula\AppData\Local\ARCA (Acesso negado. (os error 5)). Nada foi apagado (PR-7)` e `faltam 536 MB para baixar o pacote, e C:\Users\Ana Paula\AppData\Local\ARCA tem 100 MB livres. Nada foi apagado (PR-7)`; WHEN a pasta não se deixa criar, e WHEN o `.parcial` não se deixa criar THEN `NaoDeuParaGuardar`, com `sistema.baixados` vazio; WHEN o volume tem 100 MB livres e nada baixado THEN `FaltaEspaco` com 536 e 100, e `baixados` vazio; e WHEN há 500 MB no `.parcial` e 40 MB livres THEN não há recusa — o que conta é o que falta baixar
Proof: `cargo test --lib pacote::testes::as_recusas_de_onde_guardar_sao_o_texto_aprovado -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_pasta_que_nao_aceita_escrita_recusa_antes_de_baixar -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::sem_espaco_para_o_que_falta_recusa_antes_de_baixar -- --exact`

**C9** (critério 9) - WHEN `--dry-run` e nenhum pacote guardado THEN `baixados` fica vazio, nada é escrito na pasta (nem o `.parcial`), a tela do pré-voo é exatamente `  Pacote .......................... nao esta neste PC · o preparo de verdade baixa 535,5 MB antes de qualquer pergunta; o ensaio nao baixa\n`, e a linha do Clonezilla no plano é exatamente `  Clonezilla 3.3.3-15 · o preparo de verdade o baixa antes do menu (535,5 MB) e o guarda\n     neste PC, com o SHA256 conferido contra o valor compilado neste ARCA (PR-1)\n`
Proof: `cargo test --lib comandos::prepare::testes::o_ensaio_sem_pacote_guardado_nao_baixa_e_diz_que_o_preparo_baixa -- --exact`

**C10** (critério 10) - WHEN `--dry-run` e um pacote guardado que bate THEN `resumidos == [guardado]`, a tela do pré-voo é a mesma de C2, e `particionou()` é falso
Proof: `cargo test --lib comandos::prepare::testes::o_ensaio_com_pacote_guardado_confere_e_nao_particiona -- --exact`

**C11** (critério 11) - WHEN o plano é montado com o pacote guardado THEN a linha do Clonezilla é exatamente `  Clonezilla 3.3.3-15 · do pacote guardado neste PC, conferido contra o valor\n     compilado neste ARCA (PR-1)\n`, e o plano não contém `baixa`
Proof: `cargo test --lib comandos::prepare::testes::o_plano_com_pacote_guardado_nao_promete_download -- --exact`

**C12** (critério 12) - WHEN `--iso` e há pacote guardado na pasta THEN, depois do comando inteiro, nem o guardado, nem o `.parcial`, nem a pasta foram consultados (`foi_consultado` falso para os três), `baixados` está vazio, a tela do pré-voo é exatamente `  Pacote conferido ................ ok · 00cee7700433 · <iso> · nada tocado ainda (PR-2)\n` (a de `378143b`), e a linha do Clonezilla no plano é exatamente `  Clonezilla 3.3.3-15 · do arquivo local <iso> (PR-2)\n`
Proof: `cargo test --lib comandos::prepare::testes::com_iso_o_pacote_guardado_nem_e_olhado -- --exact`

**C13** (critério 13) - WHEN o comando sem `--iso` atravessa o passo 5 com o pacote guardado THEN `resumidos == [guardado, guardado]` (pré-voo e passo 7), a extração sai do guardado e a cópia no `ARCAVAULT` tem o conteúdo dele; e WHEN o guardado deixa de bater entre o pré-voo e o passo 7 THEN a recusa é `ResumoDivergente`, depois de particionar, e a cópia no `ARCAVAULT` não existe. É a reescrita, por decisão (Impact do issue, ADR-0028), de `sem_iso_o_pre_voo_nao_tem_o_que_conferir`
Proof: `cargo test --lib comandos::prepare::testes::sem_iso_o_pacote_guardado_e_conferido_antes_do_menu_e_de_novo_no_passo_7 -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_guardado_que_muda_antes_do_passo_7_nao_e_copiado -- --exact`

**C14** (critério 14) - Nenhum código fora dos testes apaga arquivo, e a porta `Arquivos` não declara exclusão; e WHEN a pasta tem `clonezilla-live-3.3.2-1-amd64.zip` e `arca.log` e o comando baixa de novo um guardado que não bateu THEN os dois continuam com o conteúdo de antes, e as únicas chaves que mudaram são o guardado e o `.parcial`
Proof: `cargo test --test b10_nada_e_apagado`
Proof: `cargo test --lib comandos::prepare::testes::o_download_nao_mexe_nos_outros_arquivos_da_pasta -- --exact`

**C15** (critério 15) - WHEN o download é interrompido (o `baixar` devolve erro, que é o que sobra de um `curl` morto para quem o chamou) THEN `executar` devolve o erro com `enumeracoes == 0`, `particionou()` falso e o `.parcial` ainda lá; e um `curl` encerrado no meio deixa o destino com os bytes que chegaram (C18, primeira prova). Que um Ctrl+C mata os dois processos do console é do Windows, e não tem prova automática aqui
Proof: `cargo test --lib comandos::prepare::testes::um_download_interrompido_nao_chega_ao_particionador -- --exact`
Proof: `cargo test --lib adaptadores::windows::sistema::testes::uma_fonte_parada_e_abandonada_e_o_parcial_fica_com_o_que_veio -- --exact`

### S2 - Fontes do pacote · 5 files · 252 KB · ~63k (6k novos sobre S1)

**C16** (critério 16) - O adaptador, contra um servidor local que entrega 1000 bytes em 100 a cada 0,5 s, chama `andamento` ao menos 3 vezes, nunca com mais de 2 s entre duas chamadas, com valores que não diminuem e terminam em 1000; e o pré-voo, recebendo `andamento` 10.485.760 e 125.829.120, escreve `\r  Baixando Clonezilla ............. 3.3.3-15 · 120,0 MB de 535,5 MB · free.nchc.org.tw (oldstable)` sem quebra de linha entre as duas reescritas
Proof: `cargo test --lib adaptadores::windows::sistema::testes::o_andamento_chega_ao_menos_a_cada_2_s -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::o_andamento_reescreve_a_linha_no_lugar -- --exact`

**C17** (critério 17) - O julgamento do que a fonte respondeu, sobre as transferências medidas em 04/10/2026: código 28 com `http` `None`, e código 28 com `http` `302`, são `nao respondeu em 30 s`; `parou` é `parou de mandar por 60 s, com <recebido> de 535,5 MB`; código 22 com `404` é `HTTP 404`; código 7 é o texto do `curl`; código 0 é `entregou`. WHEN a primeira fonte não responde THEN a tela traz `  Fonte abandonada ................ free.nchc.org.tw (oldstable) · nao respondeu em 30 s\n` e `baixados == [oldstable, stable]`; e o `curl` recebe `--connect-timeout 30`, `-C -`, `-N`, `-f`, `-L`, e não recebe `--retry` nem `--speed-time`
Proof: `cargo test --lib pacote::testes::o_que_cada_fonte_respondeu_e_julgado_pelo_codigo -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_que_nao_responde_e_abandonada_e_a_proxima_e_tentada -- --exact`
Proof: `cargo test --lib adaptadores::windows::sistema::testes::o_curl_recebe_os_limites_e_retoma -- --exact`

**C18** (critério 18) - O adaptador, com `parado` de 2 s, contra um servidor que manda 100 de 1000 bytes e para, devolve `parou: true` em menos de 5 s, com o destino em 100 bytes; contra um destino com 100 bytes e um servidor que honra `Range`, o pedido traz `Range: bytes=100-` e o destino termina com 1000; e WHEN a primeira fonte para com 120,0 MB THEN a tela traz `  Fonte abandonada ................ free.nchc.org.tw (oldstable) · parou de mandar por 60 s, com 120,0 MB de 535,5 MB\n`, as duas chamadas têm o mesmo destino `.parcial`, e o `.parcial` não é zerado entre elas
Proof: `cargo test --lib adaptadores::windows::sistema::testes::uma_fonte_parada_e_abandonada_e_o_parcial_fica_com_o_que_veio -- --exact`
Proof: `cargo test --lib adaptadores::windows::sistema::testes::o_curl_retoma_do_byte_em_que_o_parcial_termina -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_que_para_e_abandonada_e_a_proxima_continua_do_mesmo_parcial -- --exact`

**C19** (critério 19) - O adaptador, contra um servidor local que responde 404, devolve `http: Some(404)` e código 22; e WHEN a primeira fonte responde 404 THEN a tela traz `  Fonte abandonada ................ free.nchc.org.tw (oldstable) · HTTP 404\n` e a segunda fonte é tentada
Proof: `cargo test --lib adaptadores::windows::sistema::testes::um_erro_http_volta_com_o_codigo -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::uma_fonte_com_erro_http_e_abandonada_e_a_proxima_e_tentada -- --exact`

**C20** (critério 20) - `pacote::FONTES` é exatamente a lista do critério, nesta ordem, com os rótulos `free.nchc.org.tw (oldstable)`, `free.nchc.org.tw (stable)` e `downloads.sourceforge.net`; cada URL contém `VERSAO` e termina em `ARQUIVO`; e no C6 `baixados` é as três URLs, nessa ordem, uma vez cada
Proof: `cargo test --lib pacote::testes::as_fontes_sao_as_tres_na_ordem_e_carregam_a_versao -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::nenhuma_fonte_entregou_recusa_antes_dos_discos_e_guarda_o_que_veio -- --exact`

**C21** (critério 21) - WHEN as três fontes respondem (não respondeu, parou com 120,0 MB, entregou) THEN o `arca.log` tem, nessa ordem, uma linha por fonte com a URL, a frase do critério 6 (ou `entregou`) e `<n> bytes desta fonte`, com `0`, `125829120` e `435649528`; e WHEN o guardado é usado sem rede THEN o `arca.log` tem `pacote guardado usado sem rede · <guardado>`
Proof: `cargo test --lib comandos::prepare::testes::cada_fonte_tentada_fica_no_arca_log -- --exact`
Proof: `cargo test --lib comandos::prepare::testes::um_pacote_guardado_que_bate_e_usado_sem_rede -- --exact`

### S3 - A regra escrita · 5 files · 362 KB · ~91k

**C22** (critério 22) - O §9.6 do PRD, entre `### 9.6` e `### 9.7`, tem a linha que começa com o texto do Decided; e o §13 do README ganha `### Preparação`, entre `### Segurança` e `### Armadilhas conhecidas`, com PR-1 a PR-7 — o §13 não tinha tabela de preparação, e uma com só PR-7 diria que as outras seis não existem
Proof: `sed -n '/^### 9.6/,/^### 9.7/p' PRD/PRD-ARCA-v5_1.md | grep -qF '| PR-7 | Nenhum download depois do ponto sem volta. Sem --iso, o arca prepare obtém o pacote no pré-voo — do pacote guardado no PC ou baixando-o — e o confere contra o SHA256 compilado antes de listar os discos. Uma fonte que não conecta em 30 s ou fica 60 s sem entregar nada é abandonada, e a próxima continua de onde ela parou.'`
Proof: `S=$(sed -n '/^### Preparação$/,/^### Armadilhas conhecidas/p' README.md); for id in PR-1 PR-2 PR-3 PR-4 PR-5 PR-6 PR-7; do printf '%s\n' "$S" | grep -qF "| **$id** |" || exit 1; done; printf '%s\n' "$S" | grep -F '| **PR-7** |' | grep -qF 'ponto sem volta' && sed -n '/^## 13\./,/^## 14\./p' README.md | grep -qx '### Preparação'`

**C23** (critério 23) - `docs/adr/0028-nenhum-download-depois-do-ponto-sem-volta.md` existe e registra: as Key decisions 1 a 7 do design (o pré-voo, `%LOCALAPPDATA%\ARCA\`, a conferência a cada uso, o `.parcial`, B-10, 30 s e 60 s, o `--dry-run` que não baixa); as medições de 04/10/2026 (`0,9 MB/s`, `1,5 MB/s`, e o `T + 6 s` do `--speed-time`); e a mudança de premissa do teste da WPC-66, citando o texto antigo `uma vez so, depois do ponto sem volta`
Proof: `A=docs/adr/0028-nenhum-download-depois-do-ponto-sem-volta.md; [ -f "$A" ] && for t in 'PR-7' '%LOCALAPPDATA%\ARCA\' '.parcial' 'B-10' '30 s' '60 s' '--dry-run' '04/10/2026' '0,9 MB/s' '1,5 MB/s' 'T + 6 s' 'WPC-66' 'uma vez so, depois do ponto sem volta' 'SHA256'; do grep -qF -- "$t" "$A" || { echo "falta: $t"; exit 1; }; done`

**C24** (critério 24) - O `CONTEXT.md` tem a entrada `**Pacote guardado**:`, e o bloco dela (até a linha `_Evitar_` seguinte) diz `%LOCALAPPDATA%`, nomeia a cópia de `PR-3` como outra coisa e diz que ela mora no dispositivo
Proof: `E=$(sed -n '/^\*\*Pacote guardado\*\*:$/,/^_Evitar_/p' CONTEXT.md); [ -n "$E" ] && printf '%s\n' "$E" | grep -qF '%LOCALAPPDATA%' && printf '%s\n' "$E" | grep -qF 'PR-3' && printf '%s\n' "$E" | grep -qF 'dispositivo' && printf '%s\n' "$E" | grep -q '^_Evitar_'`

**C25** (critério 25) - No README: a linha do passo 6 da tabela de passos deixa de dizer `Baixar` e a do pré-voo passa a dizer `pacote guardado`; a linha `| URL |` da tabela do pacote vira `| Fontes |`, com as três URLs na ordem de C20; na tela de exemplo do §6.1, `Baixando Clonezilla` e `Pacote conferido` aparecem antes de `Podemos continuar?`, e nada de `Baixando` depois de `Particionando`; a linha `Conexão de rede` dos requisitos diz `pacote guardado`; e o diagnóstico `o SHA256 do pacote NAO bate` perde a frase `Sem `--iso` o disco já foi apagado quando ela chega` e passa a dizer `Nada foi apagado`. Além do critério, por consequência direta: o §10 lista o pacote guardado e o `.parcial` entre os arquivos do `C:`
Proof: `T=$(sed -n '/^#### Os passos, e o que fica se você parar em cada um$/,/^#### O pacote do Clonezilla$/p' README.md); printf '%s\n' "$T" | grep '^| 6 |' | grep -vqi 'baixar' && printf '%s\n' "$T" | grep '^| pré-voo |' | grep -qF 'pacote guardado'`
Proof: `P=$(sed -n '/^#### O pacote do Clonezilla$/,/^#### Instalar sem rede$/p' README.md); ! printf '%s\n' "$P" | grep -q '^| URL |' && L=$(printf '%s\n' "$P" | grep '^| Fontes |') && a=$(printf '%s' "$L" | grep -bo 'free.nchc.org.tw/clonezilla-live/oldstable' | cut -d: -f1) && b=$(printf '%s' "$L" | grep -bo 'free.nchc.org.tw/clonezilla-live/stable' | cut -d: -f1) && c=$(printf '%s' "$L" | grep -bo 'downloads.sourceforge.net' | cut -d: -f1) && [ "$a" -lt "$b" ] && [ "$b" -lt "$c" ]`
Proof: `X=$(sed -n '/^#### Exemplo real/,/^### 6.2/p' README.md); b=$(printf '%s\n' "$X" | grep -n 'Baixando Clonezilla' | head -1 | cut -d: -f1); c=$(printf '%s\n' "$X" | grep -n 'Pacote conferido' | head -1 | cut -d: -f1); q=$(printf '%s\n' "$X" | grep -n 'Podemos continuar?' | head -1 | cut -d: -f1); p=$(printf '%s\n' "$X" | grep -n '  Particionando' | head -1 | cut -d: -f1); [ -n "$b" ] && [ -n "$c" ] && [ "$b" -lt "$q" ] && [ "$c" -lt "$q" ] && ! printf '%s\n' "$X" | tail -n +"$p" | grep -q 'Baixando'`
Proof: `grep '^| Conexão de rede |' README.md | grep -qF 'pacote guardado'`
Proof: `D=$(sed -n '/^### `o SHA256 do pacote NAO bate`$/,/^### `o arquivo/p' README.md); ! printf '%s\n' "$D" | tr '\n' ' ' | grep -qiF 'o disco já foi apagado quando ela chega' && printf '%s\n' "$D" | grep -qF 'Nada foi apagado'`
Proof: `sed -n '/^### No `C:`/,/^### O que o ARCA nunca escreve/p' README.md | grep -qF 'clonezilla-live-3.3.3-15-amd64.zip.parcial'`

**C26** (critério 26) - O §12 do README tem uma entrada para cada recusa nova — `nao deu para baixar o Clonezilla`, `nao deu para guardar o pacote em` e `faltam ... MB para baixar o pacote` —, cada uma com a mensagem num bloco de código e um parágrafo que diz o que fazer em seguida (o `--iso`, a pasta, o espaço)
Proof: `for h in 'nao deu para baixar o Clonezilla' 'nao deu para guardar o pacote em' 'MB para baixar o pacote'; do E=$(awk -v h="$h" '/^### /{dentro = index($0, h) > 0} dentro' README.md); [ -n "$E" ] || { echo "sem entrada: $h"; exit 1; }; printf '%s\n' "$E" | grep -q '^```' || { echo "sem bloco: $h"; exit 1; }; printf '%s\n' "$E" | grep -qF 'Nada foi apagado' || { echo "sem a promessa: $h"; exit 1; }; done; awk '/^### /{d = index($0, "nao deu para baixar o Clonezilla") > 0} d' README.md | grep -qF -- '--iso' && awk '/^### /{d = index($0, "MB para baixar o pacote") > 0} d' README.md | grep -qiF 'espaço'`

## Swept

- validation: C1, C3, C5, C7 — o SHA256 compilado decide sobre o pacote, e o tamanho compilado decide entre retomar e recomeçar
- failure modes: C6, C7, C8, C15
- idempotency and retry: C2, C4, C15 — rodar de novo reusa o guardado que bate, ou retoma o `.parcial`
- authorization: existing - o ARCA roda elevado (`requireAdministrator`), e o pacote mora na pasta do `arca.log`, do usuário que roda (Landing, linha 3)
- concurrency and ordering: C1 (o pacote antes do primeiro `enumerar`), C13 (a cópia de PR-3 depois da segunda conferência). Dois `arca prepare` simultâneos: n/a - não é caso de uso; o `.parcial` misturado falha no SHA256 (C7)
- data lifecycle: C14 — nada é apagado; os pacotes de outras versões ficam
- external-dependency failure: C6, C17, C18, C19
- state transitions: C1, C2, C3, C4, C5, C7 — a tabela de estados do issue: Ausente→Parcial (C1), Parcial→Parcial retomando (C4, C18), Parcial→Parcial do zero (C5), Parcial→Guardado (C1), Parcial→Parcial que não bate (C7), Guardado→Guardado sem rede (C2), Guardado→Parcial do zero (C3)
- observability: C21, e C16 com C18 — a linha do andamento é reescrita ao menos a cada 2 s, e uma fonte parada é dita na tela em 60 s

## Handoff

- S1 = ~66k: `wc -c` de `src/comandos/prepare.rs` (143.225), `src/duplos.rs` (54.366), `src/pacote.rs` (23.269), `src/adaptadores/arquivos_do_sistema.rs` (14.380), `src/app.rs` (11.781), `src/registro.rs` (6.085), `src/portas/arquivos.rs` (5.433) e `tests/b10_nada_e_apagado.rs` (4.137) = 262.676 bytes, dividido por quatro
- S2 = ~63k sozinha, mas só `src/portas/sistema.rs` (9.801) e `src/adaptadores/windows/sistema.rs` (15.630) são leitura nova sobre S1: S1+S2 = 288.107 bytes = ~72k
- S3 = ~91k: `README.md` (125.990), `PRD/PRD-ARCA-v5_1.md` (209.045), `CONTEXT.md` (11.965), `docs/adr/0018-…` (8.946) e `docs/adr/0027-…` (6.043, o modelo de ADR) = 361.989 bytes
- S1+S2 = ~72k cabe; S3 levaria a ~163k, acima dos 150k. **Handoff depois de S2**, que é também onde a superfície muda: do código para a documentação. S3 vai a um agente novo, que lê este checklist e o diff de `378143b..HEAD`

- **Boundary:** C1-C21 fechados em `73e061e` (S1 e S2). A base da feature é `378143b`, e o checklist entrou sozinho em `255fa50`. S3 (C22-C26, só documentação) é do próximo agente
- **Settled mid-build:** nada foi perguntado ao usuário. Decidido no caminho, e não coberto por Landing nem por check:
  - as três recusas de PR-7 saem sem o prefixo `o pacote do Clonezilla foi recusado: ` que `Erro::PacoteRecusado` põe nas outras (`mensagem_do_pacote`, em `src/erro.rs`): o texto aprovado vai inteiro, e "recusado" seria falso para "nenhuma fonte entregou". As recusas de PR-1 e PR-2 saem byte a byte como antes
  - `--dry-run` com um pacote guardado que não bate escreve `Pacote guardado ................. nao bate com o SHA256 deste ARCA · o preparo de verdade baixa de novo, do zero; o ensaio nao baixa`, e o plano sai como sem pacote (`PorBaixar`)
  - uma pasta com o nome do pacote guardado recusa no pré-voo como `NaoDeuParaGuardar`, com o motivo `` `clonezilla-live-3.3.3-15-amd64.zip` e uma pasta ``, em vez de deixar o `renomear` falhar depois de um download inteiro
  - o adaptador avisa o andamento pela primeira vez depois de 1 s, e não ao começar: um aviso imediato apagaria na hora o `retomando de <X>` que o pré-voo acabou de escrever (C4)
  - o módulo de testes de `src/adaptadores/windows/sistema.rs` passou de `#[cfg(all(test, windows))]` a `#[cfg(test)]` + `#[cfg(windows)]`: a varredura de B-10 reconhece o módulo de testes pelo `#[cfg(test)]`, e os testes do download limpam o próprio diretório temporário
  - `ArquivosQueRecusam` passou a recusar também `criar_diretorio` e `escrever_atomico` no caminho recusado (C8); nenhum uso anterior escrevia nele
  - as bancadas de `src/comandos/prepare.rs` e `src/app.rs` nascem com um pacote guardado que bate: é o estado normal do PC depois do primeiro preparo, e mantém os testes que não são sobre o pacote no que eram, sem rede. `Bancada::sem_pacote_guardado` tira o pacote
  - `sem_iso_o_pre_voo_nao_tem_o_que_conferir` (WPC-66) foi reescrito como `sem_iso_o_pacote_guardado_e_conferido_antes_do_menu_e_de_novo_no_passo_7`, por decisão do issue (Impact), e o comentário do teste diz isso
- **Abandoned:** o `--speed-limit 1 --speed-time 60` (medido: 66 s); pôr `retomado de <X>` no fim da linha do andamento (passaria de 120 colunas, e uma linha que quebra não se reescreve no lugar); e a asserção de C15 sobre a variante `Erro::Ferramenta`, porque o `SistemaDeMentira` devolve uma cópia da recusa como `FerramentaRecusou` — a prova passou a cobrar a mensagem do `curl` e que não é recusa do pacote. Quatro defeitos foram injetados e desfeitos antes do commit, e as provas caíram: renomear o `.parcial` sem conferir (C1, C3, C7), `<=` na borda do tamanho (as duas provas de C5), e o `curl` sem `-N` (as três provas do adaptador, inclusive a da fonte parada, com o destino em 0 bytes)
