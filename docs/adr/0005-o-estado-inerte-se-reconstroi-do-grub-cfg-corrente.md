# O estado inerte se reconstrói do `grub.cfg` corrente

Desarmar é devolver o `grub.cfg` ao estado inerte, e o PRD nunca disse de onde
esse estado vem. Havia três caminhos: **embutir** uma cópia no binário do ARCA,
**guardá-la no dispositivo** (`ARCABOOT\arca\grub.cfg.inerte`, copiada pelo
`arca prepare` da E10), ou **reconstruí-la** do `grub.cfg` que está lá,
desfazendo o que o ARCA pôs.

Decidimos reconstruir. `src/grub.rs` remove os `menuentry` com
`--id arca-backup` e aponta o `set default` para `live-default`; tudo o mais
sai byte a byte como entrou.

## O que armar muda, medido

A decisão só ficou possível depois de olhar o diff. O `grub.cfg` inerte deste
dispositivo e a captura `grub-backup-arca-teste-03.cfg` diferem em **exatamente
duas coisas**:

```diff
-set default="live-default"
+set default="arca-backup"
+
+menuentry "ARCA - backup automatico" --id arca-backup {
+  search --set -f /live/vmlinuz
+  $linux_cmd /live/vmlinuz ... ocs_live_run="bash -c '...'" ...
+  $initrd_cmd /live/initrd.img
+}
```

Nada mais muda — nem `timeout`, nem os outros `menuentry`, nem uma linha de
comentário. Armar é uma edição pequena e localizada, e a inversa dela também é.

## O `set default` é o que faz o boot ser desatendido

Este achado não está no PRD nem no plano, e passou três etapas sem aparecer.

**Inserir o `menuentry` do ARCA não arma nada.** Ele vira mais uma linha no
menu, e a máquina continua esperando trinta segundos e bootando no Clonezilla
normal. Quem faz o boot ser desatendido é o `set default` apontar para o id do
ARCA. As capturas provam os dois lados: `grub-backup-arca-teste-02.cfg` e
`grub-restauracao-arca-teste-02.cfg` têm o `menuentry` e **não** têm o
`set default` — nesse estado nenhuma receita rodaria.

Daí uma ordem de importância que o desarmar herda: devolver o `set default` é o
que torna o dispositivo inerte; tirar o bloco é higiene. As duas acontecem na
mesma gravação, mas só a primeira separa "aparece no menu" de "roda sozinho".

## `live-default`, e nunca `0`

O `grub.cfg` que o **Clonezilla** entrega — `grub.cfg.original`, preservado em
`recursos/capturas/grub-clonezilla-original.cfg` — traz `set default="0"`, e
difere do inerte deste dispositivo **só nisso**.

`"0"` aponta por posição, e a posição muda: o bloco do ARCA entra **antes** do
`live-default` e passa a ser o índice 0. Um dispositivo com `set default="0"`
está armado no instante em que o bloco é inserido, sem que ninguém toque no
`set default`. Não é o estado inerte — é um estado que parece inerte.

`"live-default"` aponta pelo `--id` que o próprio Clonezilla dá ao seu
`menuentry` padrão (está no `grub.cfg.original`; não foi ninguém que inventou),
e continua apontando para o mesmo lugar com ou sem bloco do ARCA no meio.

Por isso o desarmar devolve o `set default` para `live-default` **qualquer que
seja o valor que encontrou** — inclusive `"0"`. E é isso que responde "qual é o
estado inerte que a E4 reproduz": o `grub.cfg` deste dispositivo, e a prova é
que desarmar o do Clonezilla produz exatamente ele, byte a byte. Há teste para
isso nas duas pontas — contra a cópia no repositório e contra o arquivo no
disco.

## Considered Options

**Embutir no binário** é simples e funciona até num dispositivo cujo `grub.cfg`
foi corrompido. O preço é que o `grub.cfg` carrega configuração *daquele*
dispositivo e *daquela* versão do Clonezilla: `hostname=cl-3.3.3-15`, as
blacklists de driver, `nvme.poll_queues=1`. Escrever 11 KB embutidos por cima
descarta tudo isso em silêncio, e desarmar acontece como primeiro passo de todo
comando — seria a operação mais frequente do sistema sendo também a mais
destrutiva. Um dispositivo preparado com outra versão do Clonezilla ficaria com
a configuração de outra máquina, e o modo de falha é não bootar.

**Guardar no dispositivo** casa com "dispositivo autocontido" e sobrevive a uma
troca de versão do Clonezilla. Não existe nos dispositivos preparados à mão —
inclusive o que está na mesa. A E4 dependeria de um artefato que a E10 cria, e
a etapa que precisa vir antes do armar ficaria esperando a penúltima do plano.

## O que a reconstrução custa

**Depende de casar texto**, que é a coisa mais frágil que existe. Três coisas
reduzem o custo:

O texto a casar é **do próprio ARCA**. O `--id arca-backup` é uma marca que o
ARCA escreve, e não uma heurística sobre o arquivo alheio — e é lida por token,
para que um `--id arca-backup-antigo` não seja arrastado junto.

**O modo de falha parcial é benigno.** Um `grub.cfg` com o bloco sobrando e o
`set default` devolvido é exatamente o estado das duas capturas `teste-02`: a
máquina espera trinta segundos e boota no menu normal do Clonezilla. O que
seria grave é o contrário, e não acontece — as duas mudanças vão na mesma
escrita atômica.

**O que não se entende é recusado, não adivinhado.** Um `menuentry` do ARCA sem
a chave que o fecha faz o desarmar parar sem gravar: remover até o fim do
arquivo deixaria o `grub.cfg` truncado, e um `grub.cfg` truncado é uma máquina
que não boota, enquanto um armado ainda boota. Vale o mesmo para um `grub.cfg`
sem linha `set default`, e para um sem `menuentry --id live-default` a que
apontar.

E a **idempotência sai de graça**, que é o que C-1 cobra: a segunda passada não
acha bloco nenhum e encontra o `set default` já no lugar.

## As duas metades no mesmo lugar, e só uma em uso

`src/grub.rs` sabe as duas operações. A E4 usa só o desarmar; quem chama o
armar é a E7.

Isso não fura "só se arma o que já se sabe desarmar": a função de armar é pura,
não escreve em disco nem toca no firmware, e o ponto sem volta continua na E7.
Ela existe agora por causa de um teste que só é possível com as duas juntas —
tira-se o bloco de uma cópia armada, desarma-se, arma-se de volta com o mesmo
bloco, e o resultado tem de ser a cópia byte a byte. Com só o desarmar, a E4
estaria testando contra um alvo que ela mesma inventou; com os dois, o oráculo
é o arquivo que saiu do dispositivo.

**O armar recebe o bloco pronto, e não o monta.** As cópias mostram blocos
diferentes entre si: a `teste-03` perdeu o `hostname` e as blacklists de driver
que a `teste-02` tem. Escolher entre eles é decidir que linha de comando o
kernel recebe, e isso é da E7 — aqui o bloco é dado, e o que se prova é que
inserir e tirar se cancelam.

## O `bcdedit` chama de erro não ter o que apagar

A outra metade do desarmar é limpar a marca de boot único. Medido em
22/08/2026, com o `{fwbootmgr}` sem `bootsequence`:

```text
> bcdedit /deletevalue {fwbootmgr} bootsequence
Erro ao tentar excluir o elemento de dados especificado.
Elemento não encontrado.
(código de saída 1)
```

O `/enum` antes e depois sai idêntico: ele **não muda nada** e ainda assim sai
com código diferente de zero. Isso importa porque o adaptador transforma
código ≠ 0 em erro — e com razão, porque é assim que "Acesso negado" chega. Um
desarmar que propagasse esse erro falharia **justamente no caso normal**, que é
o dispositivo já estar inerte, e a segunda das duas passadas que C-1 exige
nunca passaria.

A saída não é interpretar o texto da recusa para separar "não havia nada" de
"não pude olhar" — são frases, em dois idiomas, e interpretar frase é o que C-3
existe para evitar. A saída é **não acreditar no `bcdedit` em nenhum dos dois
sentidos**: manda apagar, descarta o que ele responde, e pergunta de novo. Se a
marca sumiu — ou nunca esteve lá —, desarmou. Se continua, é falha. E se foi
falta de privilégio, a releitura falha junto, porque `bcdedit /enum` sem
privilégio também sai com código 1.

**C-1 e C-3 parecem brigar e não brigam.** C-1 proíbe consultar estado *antes de
decidir*: desarmar não pergunta. C-3 exige conferir com `/enum` *depois de
escrever*: o sucesso do `bcdedit` nunca é prova. Aqui o desarmar é o caso em que
C-3 é o que torna C-1 possível, porque o código de saída da ferramenta é inútil
exatamente no caso idempotente.

**Apagar o `bootsequence` não viola B-10.** B-10 fala de imagem, resíduo e log —
do que o usuário perderia. A marca de boot único é uma intenção que o próprio
ARCA gravou, e desfazê-la é o que C-1 manda. `tests/b10_nada_e_apagado.rs` varre
o código atrás de exclusão de *arquivo* e não distingue os dois casos, daí valer
deixar isto escrito.

## Uma pergunta fechada por falta de evidência

Das quatro cópias armadas que o dispositivo guarda, **só uma** tem
`set default="arca-backup"`: a `grub-backup-arca-teste-03.cfg`, que veio do
`ARCAVAULT`. As três que estão no `ARCABOOT` — `teste01`, `teste02` e
`backup02` — têm o `menuentry` do ARCA e `set default="live-default"`, com
`timeout=30`. Nesse estado a máquina esperaria trinta segundos e bootaria no
menu normal do Clonezilla, sem executar receita nenhuma.

Ou essas cópias foram feitas depois de alguém já ter devolvido o `set default`,
ou aquelas execuções foram disparadas à mão pelo menu do GRUB. As três vias
para decidir estão fechadas:

- **Datas, não.** O `restore.log` e as imagens estão no NTFS, escritos pelo
  Clonezilla, que roda 3 h adiantado (P-7); os `grub.cfg.*` estão no FAT32,
  escritos pelo Windows. Cruzá-los é exatamente o que o
  [ADR-0001](0001-selo-liga-job-ao-desfecho.md) e S-6 proíbem, e é o erro que já
  reprovou um backup perfeito neste projeto.
- **`BootNext` na NVRAM, não.** As quatro capturas de `efibootmgr` têm zero
  ocorrências, e isso não prova nada: o firmware consome o `BootNext` ao usá-lo,
  e as capturas foram feitas já de dentro do Clonezilla.
- **Dedução, não.** Foi o que produziu os dois casos anteriores de "fundação
  validada" que não era (ADR-0003 e [ADR-0004](0004-a-receita-transcreve-o-que-rodou.md)).

**E não importa.** Nas duas explicações o `set default` faz parte do que se
arma, logo faz parte do que se desarma. Fica registrado para o próximo não
refazer o caminho — e há um teste que fixa o achado:
`as_outras_copias_estao_meio_armadas_e_a_diferenca_e_so_o_set_default`.

## Consequências

O ARCA passa a ter uma definição operacional de **estado inerte** que o PRD não
tinha: o `grub.cfg` do dispositivo sem `menuentry --id arca-backup` e com
`set default="live-default"`. Ela é verificável sem reiniciar, e é contra ela
que a E4 fecha.

O desarmar funciona num dispositivo que este binário nunca viu — armado à mão,
por uma versão antiga, ou com o `set default` do Clonezilla puro. Não podia ser
diferente: ele não consulta estado, então não tem como saber quem armou.

A E10 fica livre de guardar uma cópia do inerte no dispositivo, e o binário
fica livre de carregar uma. Se um dia a reconstrução se mostrar frágil demais na
prática, o caminho do arquivo guardado continua aberto — e volta com medição, e
não com a suposição de que casar texto é sempre pior.

**A confirmação de que o dispositivo boota depois continua devendo.** A E4 fecha
com o `grub.cfg` reescrito saindo byte a byte igual ao inerte conhecido, que é
verificável sem reiniciar. Bootar de verdade custa um reinício, e a E7 já vai
fazer um.
