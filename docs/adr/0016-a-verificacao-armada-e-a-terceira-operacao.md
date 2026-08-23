# A verificação armada é a terceira `Operacao`, e V-1 não leva "em segundos"

Decidido em 23/08/2026, na etapa E11.

## O contexto: `arca verify` tem duas forças, e só uma reinicia

V-1 confere os `MD5SUMS` no Windows. V-2 — `--completo` — arma boot único que
só roda o `ocs-chkimg` e desliga. As duas respondem perguntas **diferentes**, e
a distinção é o que faz as duas existirem:

- **V-1**: *"os bytes que estão no dispositivo são os que o Clonezilla
  gravou?"* Pega corrupção de mídia e cópia truncada.
- **V-2**: *"esta imagem é restaurável?"* Descomprime cada partição e olha o
  que sai.

Um `.zst` intacto byte a byte que carregue dentro de si um NTFS inconsistente
**passa em V-1 e reprova em V-2**. É por isso que V-2 não substitui B-9, que
continua obrigatória em todo backup.

## Decisão 1: a verificação armada é uma terceira `Operacao`, e quem decidiu foi a pasta do log

`Operacao` tinha `Backup` e `Restauracao`, e as duas atravessam `receita.rs`,
o `estado.json`, o marcador do `arca-fim.txt`, a `pasta_do_log`, o
`arca resultado` e o `arca status`. Uma terceira toca tudo isso, e a pergunta
era se valia.

**Vale, e o argumento é um só: a pasta do log.**

Toda receita começa truncando o próprio `arca-fim.txt` com um `>`. A revisão da
E3 pegou o backup e a restauração dividindo o mesmo caminho, e o modo de falha
está registrado no [ADR-0004](0004-a-receita-transcreve-o-que-rodou.md): um
`arca restore X` rodado antes de o backup de X ser colhido apagava o desfecho
dele, e o §5.5 lia um backup bem-sucedido como desfecho ausente. **O selo não
cobre isso** — ele julga um desfecho *encontrado*, e não serve para nada quando
o arquivo já foi por cima.

Uma verificação armada que reusasse `Backup` cometeria o mesmo defeito pela
terceira vez. Pasta própria vem do nome da operação, e o nome da operação é o
enum. `verificacao-<nome>`, ao lado de `backup-<nome>` e `restauracao-<nome>`,
e há teste cobrando que os três sejam diferentes.

O marcador do desfecho é `ARCA_VERIFY=`, e ele é **código novo** — nenhuma
receita real o escreveu. A *forma* é transcrita dos dois que rodaram em
22/08/2026: `ARCA_` mais o nome da operação em inglês, maiúsculo.

## Decisão 2: o `disco` do estado passa a ser opcional, e o vazio é o "nenhum"

`Estado` tinha `disco: Disco` obrigatório, e o `ocs-chkimg` **não nomeia disco
nenhum** — ele opera sobre a imagem. O campo virou `Option<Disco>`.

No arquivo, a chave continua obrigatória — o leitor recusa chave faltando de
propósito, e afrouxar isso para um campo só tiraria a propriedade que o torna
confiável — e o valor ausente é a **string vazia**.

**A escolha do sentinela não é arbitrária.** `Disco::novo("")` já recusava,
com `RecusaDaReceita::DiscoVazio`, desde a E3: o vazio nunca foi um nome de
disco possível, então usá-lo para dizer "nenhum" **não pode colidir** com nome
nenhum que o Linux dê. Um sentinela como `nenhum` colidiria — `[a-z][a-z0-9]*`
o aceitaria, e um dia alguém teria um disco assim.

E a premissa do [ADR-0006](0006-o-selo-e-o-estado-sem-dependencia-nova.md)
continua de pé: a string vazia não alcança `"`, `\`, controle nem não-ASCII, e
por isso escrever o JSON à mão continua defensável. Aquele ADR avisava que a
discussão voltaria com o campo novo na mesa; ela voltou pela segunda vez — a
primeira foi o `situacao` da E8 — e o campo passa.

**A coerência é cobrada nos dois sentidos**, no leitor e em `Receita::montar`:
`verificacao` exige vazio, e `backup`/`restauracao` exigem nome. O segundo
sentido importa tanto quanto o primeiro — um disco carregado até uma receita
que não o usa é um valor que ninguém confere. E o primeiro é o que dói: um
`estado.json` dizendo `restauracao` com disco vazio armaria uma operação
destrutiva sem dizer sobre o quê.

> **Esta metade da decisão quase saiu sem teste.** A falsificação de rotina —
> mutar o código e conferir que a suíte fala — mostrou que a mutação *"aceita
> disco vazio para qualquer comando"* **passava despercebida**. Os testes
> nasceram daí, e não da escrita.

## Decisão 3: a receita da verificação acrescenta ao `arca-check.log`, e não o trunca

A receita de backup redireciona o `ocs-chkimg` com `>`, porque a imagem acabou
de nascer e o log não existe. **Aqui ele existe**: é o veredito do backup que
criou a imagem.

O `>` custaria duas coisas:

- O §6.3 mostra `Imagem de origem: APROVADA — veredito do backup que a criou`,
  e a frase viraria mentira depois da primeira verificação.
- O `>` **trunca ao abrir**, antes de o comando rodar — medido, e é o que o
  §5.5 registra sobre o `arca-fim.txt`. Um desligamento nessa janela deixaria
  uma imagem **boa** com o log em zero byte, e ela apareceria `sem veredito` na
  listagem. Perda de informação sobre uma imagem que não tem nada de errado.

Com `>>`, o [ADR-0003](0003-veredito-lido-do-arca-check-log.md) valeria
exatamente como está escrito: *"a receita **acrescenta** a linha ao log — uma
imagem verificada duas vezes fica com as duas marcas, e a antiga vem
primeiro"*.

> ### O marco desmentiu esta parte, e a decisão fica por outra razão
>
> **A previsão acima estava errada, e quem a desmentiu foi a execução real de
> 23/08/2026.** Depois da verificação armada, o `arca-check.log` da
> `2026-08-22_Apps` tem **uma marca só**, e não duas — e o log do backup de
> 22/08 **não está mais lá**.
>
> Medido, e cada linha é verificável nas duas capturas em `recursos/capturas/`:
>
> ```text
> antes  arca-check-2026-08-22_Apps.log ............ 3832 bytes
> depois arca-check-2026-08-22_Apps-pos-verificacao  4759 bytes
>
> ocorrências de `ARCA_VEREDITO=` ... 1  (estaria 2 se tivesse acrescentado)
> offset da marca .................. 4736, o fim — e não 3809, herdado
> os primeiros 3832 bytes .......... NÃO são os da captura anterior
> inicializações de terminal ....... 1 em cada arquivo
> ```
>
> A última é a decisiva: toda execução do `ocs-chkimg` abre com a mesma
> sequência de escapes (`ESC ) 0 ESC [ 1 ; 2 4 r`). Um arquivo com duas
> execuções teria duas; os dois arquivos têm **uma**. E o tamanho fecha o
> argumento — um append de uma execução inteira daria mais de 7600 bytes, e o
> arquivo tem 4759.
>
> **A receita tinha `>>`**: o `--dry-run` a imprimiu assim minutos antes de
> armar, e `recursos/ensaio-da-receita.sh` prova que `>>` acrescenta num bash
> de verdade. Alguma coisa entre o redirecionamento e o disco truncou o
> arquivo, e **a causa não está determinada** — é P-25.
>
> **E o `>>` fica assim mesmo, com a razão trocada.** Ele não compra a
> preservação do log antigo, que era o motivo escrito acima. O que ele compra
> continua de pé e foi medido em outro lugar: **o `>` trunca ao abrir**, antes
> de o comando rodar, e um desligamento nessa janela deixaria uma imagem boa
> com o log em zero byte. O `>>` não tem essa janela — no pior caso o arquivo
> antigo fica intacto.
>
> É o mesmo movimento do [ADR-0010](0010-r7-recusa-por-medicao-e-a-regua-e-o-msft-disk.md):
> *a defesa fica, e a razão muda* — e a razão nova é a que sobreviveu à
> medição.
>
> **O que isto custa, e é pouco:** o §6.3 mostra `Imagem de origem: APROVADA —
> veredito do backup que a criou`, e para uma imagem já verificada essa frase
> passa a nomear a verificação, e não o backup. Nesta operação os dois vereditos
> eram `APROVADA`, então nada mudou de conteúdo. Se um dia divergirem, quem lê
> a listagem verá o mais recente — que é a resposta certa —, com o rótulo
> errado.
>
> **E a previsão do ADR-0003 continua sem original.** *"Uma imagem verificada
> duas vezes fica com as duas marcas"* era o caso que ele previu em 22/08 e que
> esta etapa tentou produzir; ele não aconteceu. O caminho que lê `toda forma
> de reprovar antes de toda forma de aprovar` continua valendo — ele é barato e
> é o lado seguro —, e continua sem ter sido exercitado por duas verificações
> reais.

A ordem *"toda forma de reprovar antes de toda forma de aprovar"* decide o que
sai quando as duas marcas **estão** no mesmo arquivo: **uma imagem que já
reprovou continua reprovada**, mesmo que a verificação nova aprove. É o lado
conservador de propósito — mídia que falha de forma intermitente é o caso em
que a segunda leitura mente —, e é o que S-5 pede. Rodado num bash de verdade,
em `recursos/ensaio-da-receita.sh`.

## Decisão 4: V-1 não leva "em segundos", e o requisito é que estava errado

V-1 dizia *"confere os `MD5SUMS` no Windows, **em segundos**, sem reiniciar"*.
Medido em 23/08/2026, sobre a `2026-08-22_Apps`:

```text
42.604.877.207 bytes (39,7 GB) · 39 arquivos · 202,6 s · 200,5 MB/s
```

**São três minutos e vinte e três segundos.** O comando, rodado depois,
confirmou: 199,4 s e 202,8 s em duas execuções.

A afirmação "em segundos" era sobre 39,7 GB e ninguém a tinha medido. Corrigir
o requisito é parte da etapa, e é o mesmo movimento do P-17 na E9 — *o help
está certo e a premissa do requisito estava errada*.

**O que a etapa põe no lugar não é outro número fixo.** A tela estima a partir
do tamanho real, pela taxa medida, e diz de onde o número veio. A primeira
versão prometia `3 min 23 s` para qualquer imagem, e uma imagem de 1 GB
levaria cinco segundos.

E a comparação honesta entre as duas forças, que é o que ajuda a escolher:

| | lê | tempo | reinícios |
|---|---|---|---|
| **V-1** `arca verify` | os 39 MD5 do `MD5SUMS` | 3 min 23 s | 0 |
| **V-2** `--completo` | `ocs-chkimg` descomprime | 5 min 12 s | 1 |

O tempo de V-2 sai dos `mtime` da operação de 22/08: o `MD5SUMS`, o
`clonezilla-img` e o `Info-img-id.txt` levam 18:00:49 — o fim do `savedisk` —,
e o `arca-check.log` é de 18:06:02. Os dois instantes vieram do mesmo relógio,
e por isso a **diferença** entre eles não sofre com o deslocamento de 3 h (P-7).

**O que separa as duas na prática não são os dois minutos: é o reinício.** V-2
desliga a máquina, e quem está trabalhando nela para de trabalhar.

## Decisão 5: o veredito de V-1 não entra na listagem

A coluna `aprovada` do `arca list` sai do `arca-check.log`, que é o parecer do
`ocs-chkimg`. Escrever uma reprovação de V-1 naquele arquivo faria a listagem
afirmar que o `ocs-chkimg` reprovou, **e ele nem rodou**.

Então V-1 imprime e registra no `arca.log` — o registro do lado Windows, que
todo comando já alimenta — e não toca no `arca-check.log`. Quando **reprova**,
a tela diz que aquela reprovação não vai aparecer no `arca list`, porque quem
lê precisa saber que a listagem vai continuar dizendo outra coisa. Quando
aprova, o aviso não sai: conselho que aparece sempre vira ruído, e a E10 já
pagou por essa lição no `arca resultado`.

## O que isto não decidiu

**Se a listagem devia mostrar as duas verificações.** Um `arca list` com duas
colunas — o parecer do `ocs-chkimg` e a última conferência de bytes — é
concebível e é escopo que V-1 não pede. Fica de fora, e volta quando o uso
pedir, como P-14.

## Consequências

- V-1 perde o "em segundos" no §9.5 do PRD e ganha o que foi medido.
- `Operacao` tem três valores, e `pasta_do_log` produz três pastas distintas.
- `Estado.disco` é `Option`, com a coerência cobrada no leitor.
- O `arca status` ganha uma linha que diz "nenhum disco" em vez de deixar em
  branco.
- O `arca resultado` ganha um terceiro rótulo na segunda linha, e ele diz que
  desfecho e veredito **têm a mesma fonte** — ao contrário do backup, onde são
  independentes (§4.3, S-5). Duas linhas concordando não são duas testemunhas
  quando saem do mesmo `if`.
- `recursos/ensaio-da-receita.sh` ganha a terceira receita e três casos novos,
  inclusive o que prova o `>>` **num bash de verdade** — e que, medido em
  hardware, não descreve o que o `ocs-chkimg` faz (P-25).
- **V-2 rodou em hardware em 23/08/2026**, e passou: selo `aefa48f71fc66a46`
  batendo, `ARCA_VERIFY=OK`, `ARCA_FIM`, veredito `APROVADA`. Fecha P-24.
- **A pasta própria provou o que ela existe para provar.** O
  `backup-2026-08-22_Apps/arca-fim.txt` continua lá, intacto, com o selo
  `7d2d2f5153625b38` do marco da E7 — a verificação escreveu na pasta dela e
  não encostou nele. É a decisão 1 medida em vez de argumentada.
- **P-25 nasce**: o `arca-check.log` foi substituído apesar do `>>`, e a causa
  não está determinada.
