# O `arca prepare` oferece a lista, e continua não deduzindo o disco

**Toca o princípio escrito no §6.1** — *"`--dispositivo` é obrigatório, mesmo
havendo um candidato só"* — sem revogá-lo. Decidido em 25/08/2026.

## O que mudou

`--dispositivo <indice>` deixa de ser obrigatório na **superfície**. Omitido, o
comando lista os discos desta máquina e pergunta o número — exatamente como
`arca restore` sem nome faz desde a E9.

```text
Discos desta maquina:

  [1]  disco 1   JMicron Generic      447,1 GB · USB · MBR · 1 particao (E:)
  [2]  disco 2   KGSSE100 256         238,5 GB · USB · RAW · sem particao nenhuma

  Sem numero, e o `arca prepare` nao prepara:
       disco 0   KINGSTON SNV3S500G   465,8 GB · NVMe · GPT · 1 particao (C:)
                 e o disco do sistema E o disco de boot desta maquina (PR-5)

  O numero entre colchetes e o que se digita; o `disco N` e o indice do
  Windows, que e o que o `--dispositivo` recebe. Escolher um numero so
  mostra o plano — nada e apagado antes da confirmacao digitada.

Qual preparar?
```

## Por que isto não afrouxa P1 revisado

O princípio é *"o ARCA destrói dados quando o usuário nomeou o alvo e confirmou
por escrito, e **nunca por dedução**"*. Um menu é o ARCA **oferecendo**, e
oferecer não é deduzir — desde que três coisas continuem valendo, e as três
têm teste:

1. **Com um candidato só, ele não auto-seleciona.** Uma lista de um item que se
   aceita com Enter é exatamente o ARCA escolhendo o que apagar, com outro
   nome. O `1` continua sendo digitado.
   *(`com_um_candidato_so_o_menu_nao_auto_seleciona`,
   `com_um_candidato_so_o_enter_vazio_nao_escolhe_nada`.)*

2. **Não há padrão, e o Enter vazio não escolhe nada.** Um padrão é uma dedução
   com outro nome. *(`sem_digitar_nada_no_menu_nada_e_apagado`.)*

3. **O número não vira alvo direto.** Ele resolve para um índice e cai no
   caminho que já existia: julgar pelas sete defesas, imprimir o plano,
   perguntar `(s/N)`, **reler o disco** e pedir o **modelo** digitado (S-2). O
   menu troca só a descoberta do número; o portão continua sendo o modelo.
   *(`o_numero_do_menu_nao_dispensa_a_confirmacao_do_modelo`.)*

A distinção que sustenta as três está escrita desde a E9, em
`restore::escolher_a_imagem`: **escolher é apontar; confirmar é
comprometer-se.** Trocar a segunda pela primeira faria um `2` apagar um disco.

## Por que era barato, e por que não custou medição nenhuma

O comando **já enumerava todos os discos** antes de qualquer coisa — precisava
da lista para a recusa de índice inexistente poder dizer quais existem. E o
`preparacao::julgar` **já devolvia, por disco, um veredito tipado** com o motivo
da recusa. O menu é rodar o julgamento em cada disco em vez de num só:

- nenhuma consulta nova ao WMI;
- nenhuma medição nova;
- nenhuma segunda lista de defesas — o oráculo do que entra na lista é o
  próprio `julgar`, e há teste que cobra isso disco a disco
  (`a_oferta_julga_cada_disco_pelas_mesmas_sete_defesas`).

## Os discos recusados aparecem, e a decisão é do `arca restore`

`restore::montar_a_lista` enfrentou esta escolha e a resolveu: **mostrar sem
número**. A doutrina vale inteira aqui, e o pior caso é pior do que lá.

Omitir os recusados faria a lista parecer incompleta para quem sabe que há
outro disco na mesa. E o caso caro é a **defesa 1**: ela recusa o disco de mídia
que o Windows não soube classificar junto com o disco fixo. Um HD externo que
caia nesse caso simplesmente sumiria da tela — e a pessoa concluiria que o ARCA
não enxerga o HD dela, e iria procurar como forçar. **Escondido, o motivo vira
ausência; listado sem número, ele vira uma frase.**

E a numeração sai **só dos candidatos**, que é a outra metade da mesma doutrina:
um número ao lado de um item não escolhível ocuparia um índice, e aí os números
passariam a depender de coisas que não se pode digitar.

## Duas colunas de número, porque eles não são o mesmo número

`[1]` é o que se digita; `disco 1` é o índice do Windows — o que o `Get-Disk`
mostra e o que o `--dispositivo` recebe. Nesta mesa os dois batem por acidente,
e deixar a coincidência ensinar seria preparar o erro do dia em que ela acabar:
com o disco 1 desconectado, o `[1]` passa a ser o `disco 2`.

É a mesma família da medição que motiva a releitura de PR-4 — *o índice do
Windows não é identidade* —, agora dentro da própria tela.

## O que o menu cobre e um rótulo não cobriria

**Disco cru.** Um disco RAW, ou um meio-apagado por um `prepare` que morreu no
`Clear-Disk`, não tem nome nenhum para se anunciar. No menu ele aparece como
qualquer outro — número, modelo, tamanho, `sem particao nenhuma`. É a lista, e
não um rótulo, que o descreve.

**Um dispositivo ARCA que já existe.** A linha ganha `JA E UM DISPOSITIVO ARCA`.
A tela do plano já avisava que preparar por cima apaga **as imagens** — mas
dizer só lá é tarde para quem tem dois SSDs iguais na mesa e está escolhendo
qual dos dois é o velho.

## O que **não** entrou, e por quê

**Detecção de terminal.** A proposta original pedia "recusa clara quando não há
terminal para perguntar — um script não pode travar esperando um número".

A recusa já existe, e de graça, pelo mesmo caminho do `arca restore`: um `stdin`
fechado devolve **linha vazia** (`portas::Console`), e linha vazia nunca escolhe
nada. Não há laço e não há espera — o `read_line` sobre um `stdin` fechado
retorna na hora.

E `--sem-pausa` **não** serve de sinal para isso. Ela diz *"não segure a janela
ao terminar"*, e não *"não há ninguém aqui"*. Usá-la como proxy de terminal seria
dar-lhe um significado que ela não tem, e uma flag com dois significados diverge
na primeira mudança.

## O que fica exatamente como estava

- As **sete defesas** de PR-5, e nenhuma tem opção de forçar.
- Os **onze passos**, o ponto sem volta no passo 5 e a releitura do terceiro
  tempo de PR-4.
- A **confirmação digitada pelo modelo** (S-2), nos dois caminhos.
- `--dispositivo <indice>` como atalho de quem já sabe o número — e com ele o
  menu **não aparece**, o que mantém `arca prepare --dispositivo 1 --dry-run`
  rodando sem console (`com_dispositivo_na_linha_o_menu_nao_aparece`).

## O que saiu do código

`cli::testes::prepare_exige_o_dispositivo` — que cobrava do `clap` a recusa de
`arca prepare` sem argumento — foi substituído por
`prepare_sem_dispositivo_e_o_caminho_da_lista_numerada`. **O que ele defendia
não saiu**: a superfície recusar a linha nunca foi a defesa. A defesa é não
haver caminho por onde o ARCA escolha o disco sozinho, e agora quem a cumpre é
o comando — com os três testes listados lá em cima.
