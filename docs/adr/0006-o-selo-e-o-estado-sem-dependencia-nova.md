# O selo vem do Windows e o `estado.json` se escreve à mão

A E5 precisava de duas coisas que o projeto não tinha: um gerador de valores
não repetidos e um serializador. O `Cargo.toml` tem três dependências —
`clap`, `chrono`, `thiserror` — e nenhuma delas faz qualquer das duas. O
caminho de sempre seria `rand` e `serde`/`serde_json`, quatro linhas e umas
quinze caixas novas na árvore.

Decidimos não trazer nenhuma. O selo sai de `BCryptGenRandom`, que já está no
`windows-sys` desde a E0 — bastou ligar a feature `Win32_Security_Cryptography`
—, e o `estado.json` é escrito e lido por código próprio, de cinco campos.

## O selo: por que não `rand`, e por que não o relógio

O selo tem **uma** propriedade obrigatória, e não é a que o nome sugere: ele
não precisa ser imprevisível, precisa ser **não repetido**. Dois jobs com o
mesmo selo seriam indistinguíveis, que é exatamente o que o mecanismo existe
para impedir (C-11, §4.3).

Isso descarta o terceiro caminho considerado — derivar do relógio e do PID.
Ele não traz dependência nenhuma e colide: duas execuções no mesmo
milissegundo produziriam o mesmo valor, e o PID do Windows é reciclado.

**Usar o tempo para gerar não fura S-6, e vale deixar isso escrito porque
parece contradição.** S-6 proíbe comparar uma data escrita pelo Windows com
outra escrita pelo Linux para **decidir** se um desfecho pertence a um job. Um
identificador derivado do relógio não decide nada — quem decide é a igualdade
entre duas cadeias de dezesseis dígitos. São coisas diferentes. O relógio
ficou de fora por colidir, e não por S-6.

Entre `rand` e a fonte do sistema, a fonte do sistema:

- **Nenhum crate novo.** `BCryptGenRandom` está em `windows-sys` 0.61.2, atrás
  de uma feature que estava desligada. Medido antes de decidir.
- **É a mesma família de tudo em `src/adaptadores/windows/`** — a mesma forma
  de chamada, o mesmo estilo de `unsafe`, a mesma disciplina de comentário de
  segurança.
- **`BCRYPT_USE_SYSTEM_PREFERRED_RNG` dispensa handle.** A alternativa é abrir
  um algoritmo com `BCryptOpenAlgorithmProvider` e fechá-lo depois; a flag
  tira as duas chamadas e não deixa nada a vazar se algo falhar no meio.

O preço é que o ARCA passa a ter uma quarta porta, e o `src/portas/mod.rs`
dizia "são três". Ela existe pelo mesmo motivo das outras: sem duplo, nenhum
teste sobre o `estado.json` saberia que selo esperar.

**E a porta falha alto.** `Entropia::preencher` ou preenche o destino inteiro
ou devolve erro — não há preenchimento parcial. Um gerador que recusasse em
silêncio deixaria zeros no fim do selo, e dezesseis zeros são exatamente o que
`Selo::de_ensaio` usa para dizer "isto não é de verdade". Há teste para o
caminho da recusa, e ele existe por causa disso.

## O `estado.json`: por que à mão

Escrever cinco campos à mão é menos código do que parece, e o motivo é uma
propriedade que já existia e ninguém tinha nomeado: **nenhum dos cinco valores
pode conter algo que o JSON precise escapar.** Não é sorte, é consequência de
validadores que a E1, a E3 e esta etapa já obrigam:

| campo | quem o julga | alfabeto |
|---|---|---|
| `selo` | `Selo::novo` | 16 dígitos hexadecimais minúsculos |
| `comando` | `Operacao` | `backup` ou `restauracao` |
| `nome` | `Nome::novo` (B-2) | `A-Z a-z 0-9 . _ -`, lista de permissão |
| `disco` | `Disco::novo` | `[a-z][a-z0-9]*` |
| `armado_em` | `MomentoDoArmar` | dígitos, `-`, `:`, `T`, `+` |

Nenhum alcança `"`, `\`, caractere de controle ou não-ASCII. **Ainda assim se
confere antes de escrever**, e essa conferência é o que torna a decisão
defensável em vez de só curta: "já foi validado antes" é a frase que produziu
os dois achados mais caros deste projeto (ADR-0003 e ADR-0004). Um valor que
precisasse de escape é erro alto, e não escape silencioso.

### O leitor recusa em vez de ler pela metade

`Estado::de_json` não é um parser de JSON — é o leitor do que
`Estado::como_json` escreve. A diferença aparece em `\`, que aqui é **recusa**
em vez de escape: honrar escapes faria este leitor aceitar textos que aquele
escritor não consegue produzir, e o que se quer é o contrário. Recusa também
chave desconhecida, chave repetida, chave faltando e qualquer coisa depois do
`}`.

Chave desconhecida ser recusa merece explicação, porque a escolha usual é
ignorá-la. Um `estado.json` com uma chave que esta versão não conhece veio de
uma versão que sabe alguma coisa que esta não sabe — e **agir sobre metade de
um estado que arma uma operação destrutiva é pior do que recusar o arquivo
inteiro.** Há exatamente um escritor, e ele está neste repositório.

### O teste é o corte em todos os comprimentos

O requisito era "um arquivo truncado no meio tem de ser recusado, não lido pela
metade". A tentação é escolher um ponto de corte e testar aquele — e é
exatamente a armadilha que a revisão da E4 nomeou: *o caso construído era mais
fácil do que o real*. O teste corta o arquivo em **todos** os comprimentos
possíveis e exige recusa em cada um.

Ele achou uma borda no primeiro `cargo test`: o corte que tira só a quebra de
linha final deixa um objeto completo, e o leitor o aceita — de propósito, porque
nada garante que quem gravou terminou com `\n`. O teste passou a cobrar até o
fim do **conteúdo**, e não do arquivo. A borda é do teste, e apareceu porque ele
não escolheu o caso fácil.

## `MomentoDoArmar` guarda texto, e não um `DateTime`

O plano pedia o momento do armar como campo "informativo, **nunca comparado**
com nada escrito pelo Linux". Isso já era um comentário em `src/portas/relogio.rs`
antes desta etapa, e comentário não impede nada: a trava que reprovou um backup
perfeito neste projeto tinha o comentário do lado.

Por isso o tipo **guarda o texto já formatado**. Não há o que subtrair, não há
o que comparar com o `modificado_em` de um arquivo, e não há acessor que
devolva algo comparável. Quem quisesse violar S-6 precisaria primeiro parsear a
string de volta, de propósito, num `let` que apareceria no diff.

Ele deriva `PartialEq` e não deriva `PartialOrd` nem `Ord`: comparar dois
momentos escritos pelo **mesmo** relógio não é o que S-6 proíbe, e o teste de
ida e volta precisa da igualdade.

`tests/s6_o_tempo_nao_decide.rs` cobra as duas metades a cada build, na forma
dos testes de arquitetura que S-1 e B-10 já usam. A segunda metade é a que vale
mais: **`src/desfecho.rs` — o módulo que decide a quem um `arca-fim.txt`
pertence — não menciona tempo em forma nenhuma.** Não é disciplina de quem
escreve; é que o tipo não está lá para ser usado.

## Consequências

O `Cargo.toml` continua com três dependências, e o binário continua sem árvore.
Se um dia o `estado.json` crescer para além de campos de alfabeto fechado — um
caminho, um texto livre —, a premissa que sustenta escrever à mão cai junto, e
a discussão volta. Volta com o campo novo na mesa, e não com a suposição de que
serializar à mão é sempre pior.

A quarta porta abre precedente, e é bom que abra com um caso pequeno: ela
mostra que "as três fronteiras perigosas" era uma descrição do que havia, e não
um limite.

**Nada em produção gera selo nesta etapa**, e por isso `Entropia` não entrou no
`Contexto` do `app.rs`: um campo que nenhum comando lê é peso morto, e quem arma
é a E7. O que fecha o buraco que a E4 nomeou — *o primeiro uso real de uma porta
é onde as surpresas moram* — é `examples/estado_no_arcaboot.rs`, que roda o
adaptador de verdade contra o `ARCABOOT` desta mesa. Foi ali que se confirmou
que `R:\arca\` não existia e que `criar_diretorio` o cria, é idempotente, e que
os cinco campos dão a volta byte a byte no FAT32.
