# A restauração só restaura no disco de origem

**Supersede a decisão 5 do plano de etapas** — *"destino divergente é permitido,
com confirmação que nomeia o disco de destino"*. Decidido em 23/08/2026.

## A decisão

> *"Não existe a possibilidade de restaurar um backup para um disco incorreto —
> se eu eventualmente trocar o SSD, prefiro reinstalar o Windows do zero.
> Então essa questão não está em aberto."*

É decisão de escopo, e ela fecha uma pendência em vez de abrir uma. O caso de
uso *"restaurar numa máquina com disco novo"* deixa de existir: quem troca o
disco reinstala.

## O que isso torna mais simples, e mais duro

A decisão 5 tratava destino divergente como **permitido com salvaguardas**, e a
E9 construiu as salvaguardas: `--destino <indice>`, a recusa por medição (R-7),
a recusa do dispositivo (R-8) e a confirmação digitada. O que sobra agora é
mais curto: **o único destino válido é o disco de onde a imagem veio.**

**E a medição de R-7 troca de função, o que é o ganho menos óbvio desta
decisão.** Ela existia para responder *"o destino cabe?"* — uma comparação `≥`.
Passa a responder *"é ele mesmo?"* — uma comparação `=`, entre os setores que a
GPT de dentro da imagem registra e os que o `MSFT_Disk` responde para o disco
que está na mesa.

Igualdade exata é mais difícil de satisfazer por acidente do que "maior ou
igual", então a defesa fica mais forte sem ganhar código. O
[ADR-0010](0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md) continua
valendo inteiro — inclusive a armadilha da régua, que é o que faz os dois
números serem comparáveis. O que muda é o que a comparação prova.

Na execução real de 23/08 a linha já dizia exatamente isso:

```text
  Cabe (R-7) ...................... ok · o destino tem exatamente o tamanho da origem
```

## O que sai

**`--destino <indice>` perde a razão de existir.** Ele foi criado na E9 para
alcançar a metade permissiva de R-7 — sem ele, o destino divergente era
inalcançável. Sem destino divergente, ele passa a ser um jeito de apontar um
disco para apagar, e é isso que P1 revisado proíbe: *o ARCA não age sobre um
disco que ele mesmo escolheu*, e também não age sobre um que lhe apontaram sem
poder conferir.

**Isso muda o que fazer com `DestinoAmbiguo`**, e para melhor. Hoje a mensagem
manda *"nomeie o destino com `--destino <indice>`"*. Com dois discos do mesmo
modelo na mesa, o ARCA **não consegue saber qual é o de origem** — e pedir que
alguém aponte é transformar uma dúvida do ARCA numa afirmação do usuário sobre
a qual não há como conferir nada. A resposta certa passa a ser **recusar e
parar**, dizendo o que está ambíguo.

É o mesmo raciocínio que a E7 usou para não pedir o nome do disco do Linux ao
usuário: *"pedir o nome parece gentil e é pior — não há nada deste lado contra
o que conferi-lo"*.

## O que fica, e fica por defesa em profundidade

- **R-8** — recusar o dispositivo ARCA como destino — continua. Com o destino
  amarrado ao disco de origem ela vira redundante no caminho normal, e é
  exatamente por isso que fica: a revisão da E9 mostrou que a recusa por letra
  tinha um contorno por acidente de modelo, e uma segunda barreira custa nada.
- **A conferência da imagem contra ela mesma (R-2)** continua.
- **A confirmação digitada (R-3, S-2)** continua. O disco ser o de origem não
  torna a operação menos destrutiva — ela apaga o Windows que está rodando.

## O que fecha junto

- **P-21** — *"o `ocs-sr` sai com código ≠ 0 quando desiste por destino
  menor?"* — deixa de importar. Ela já não era urgente porque R-7 recusava
  antes; agora o caso que ela descreve não é alcançável pelo ARCA.
- **A nota da decisão 5 sobre `bcdboot` em disco novo** sai do escopo: em disco
  novo não há restauração.

## O que isto custa no código, e não foi feito aqui

Este ADR registra a decisão. A mudança em `src/comandos/restore.rs` e
`src/cli.rs` fica para quando for pedida:

- `escolher_o_destino` deixa de aceitar `pedido: Option<u32>`;
- `DestinoAmbiguo` passa a ser recusa terminal, com mensagem nova;
- `--destino` sai do `cli.rs`, e com ele o teste `restore_aceita_nome_e_destino`;
- R-7 passa de `>=` para `==`, e os testes `um_destino_maior_passa_e_a_sobra_aparece`
  e `um_destino_de_verdade_menor_e_recusado` mudam de sentido — o primeiro passa
  a **recusar**;
- a tela do §6.1 perde a linha do disco escolhido e ganha a de identidade.

Nada disso é grande, e todo ele mexe em código que passou por revisão. Fazer
junto com a próxima etapa que tocar o `restore` é mais barato do que fazer
agora e reabrir os mesmos arquivos depois.
