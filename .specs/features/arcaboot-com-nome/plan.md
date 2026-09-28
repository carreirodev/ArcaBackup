# ARCABOOT com nome

## Problem

Todo dispositivo ARCA se apresenta igual:

- No Explorer, cada SSD aparece como `ARCAVAULT (E:)` e `ARCABOOT (R:)`.
- No menu do `arca prepare`, dois SSDs do mesmo modelo saem na mesma linha: `JMicron Generic ... JA E UM DISPOSITIVO ARCA`.
- O `arca status` imprime `ARCABOOT` para qualquer um.

Quem tem mais de um dispositivo não sabe, pela tela, qual está conectado. Pior: não sabe qual dos dois SSDs iguais na mesa guarda as imagens que ele quer manter antes de preparar por cima do outro.

O projeto já registrou a lacuna quando nasceu o aviso `ESTE DISCO JA E UM DISPOSITIVO ARCA`. A nota do PR-4 no PRD diz: *"um `ARCAVAULT` de 445 GB numa linha não é reconhecimento, é uma pista"*. A fonte não traz contagem de incidentes.

Quando isto for entregue, o usuário renomeia o `ARCABOOT` de cada dispositivo no Explorer para `ARCA-<texto>`, por exemplo `ARCA-CASA`. O nome aparece:

- no Explorer;
- no `arca status`;
- na recusa de dois dispositivos conectados;
- no menu do `arca prepare`.

O texto tem no máximo **6 caracteres**, porque o rótulo FAT32 tem 11 e `ARCA-` ocupa 5.

## Flow

Reusa o `dispositivo::encontrar` e a contagem de C-10 que já existem, sem criar um segundo caminho de identificação. Reusa também a lista de partições com rótulo que a tela do plano do `prepare` já imprime.

1. rótulos dos volumes -> `Discos::volumes()` (exists) -> `dispositivo::encontrar` (exists) - o `ARCABOOT` passa a ser o volume cujo rótulo é `ARCABOOT` ou começa com `ARCA-` (door 1). Mais de um é `Erro::DispositivosDemais` (exists), de C-10
2. `Erro::DispositivosDemais` (exists) - quando algum volume de boot tem a forma `ARCA-<texto>`, a mensagem passa a trazer a letra e o rótulo de cada volume de boot
3. `comandos::status::secao_do_dispositivo` (exists) - imprime o rótulo real do `ARCABOOT` quando ele tem a forma `ARCA-<texto>`
4. `comandos::prepare::e_um_dispositivo_arca` (exists) - reconhece o dispositivo pelo mesmo casamento do passo 1. `descrever_o_disco` (exists) acrescenta o rótulo à marca `JA E UM DISPOSITIVO ARCA`. A tela do plano (exists) já lista cada partição com o rótulo
5. `prevoo::julgar_o_dispositivo` (exists) - a recusa do dispositivo partido, que confere o mesmo disco físico, continua valendo para os quatro comandos que armam: `backup`, `restore`, `sondar` e `verify --completo`. Os dois últimos passaram a chamá-la com a WPC-53 (`10dd87c`, `630c8ef`), em `sondar.rs:88` e `verify.rs:456`
6. out: a receita (exists) continua citando só `LABEL=ARCAVAULT`, e o `arca prepare` (exists) continua criando o dispositivo com `ARCABOOT`

## Impact

| Front | What changes |
| --- | --- |
| domain | existing term: `ARCABOOT` era "a partição com o rótulo `ARCABOOT`" e passa a ser "a partição de boot, com o rótulo `ARCABOOT` ou `ARCA-<texto>`". Ramificam nele hoje: `dispositivo::encontrar`, a contagem de C-10, `prepare::e_um_dispositivo_arca` e a linha do `arca status` |
| domain | existing term: `Dispositivo` - a frase "rotuladas sempre com os mesmos nomes" deixa de valer para o `ARCABOOT`. Ela está no README §5, no `CONTEXT.md` e nos docs de módulo de `dispositivo.rs` e de `portas/discos.rs`. Para a receita, os dispositivos continuam intercambiáveis, porque ela só cita o `ARCAVAULT` |
| domain | existing rule: C-10 contava "dois `ARCABOOT`" por um rótulo só. Passa a contar os volumes que casam a forma, com rótulos iguais ou diferentes, e "rótulo repetido" deixa de descrevê-la. A redação muda no PRD §9.1 (C-10) e no README §6.3 (pré-voo, `há rótulo repetido na mesa?`), §6.7 (defesa 2, `e rótulo repetido`) e §13 (`Recusar rótulo repetido`). No código, mudam os docs de `dispositivo::encontrar`, de `Dispositivo::boot` e de `Erro::DispositivosDemais`, no commit do código. Continuam como estão: a tela final do `prepare` (`prepare.rs:1312`, copiada no README §6.1 e no PRD §7.1), porque com dois dispositivos inteiros a recusa é por dois `ARCAVAULT`, e esse rótulo se repete; e a nota histórica do PRD §5.2 |
| domain | new rule: C-16 - o `ARCABOOT` pode levar um nome, `ARCA-<texto>`. Nasce no PRD, no README §13 e no ADR-0027 |
| docs | README: a anatomia (§5), o exemplo do `arca status` (§6.8), a recusa de dois dispositivos (§5 e §12), a tabela §13, o verbete `ARCABOOT` do glossário §17, a linha do 0027 na tabela de ADRs e uma seção que diz como renomear no Explorer. `CONTEXT.md`: os verbetes `Dispositivo` e `ARCABOOT` |
| stored data | nada a migrar. Dispositivos com `ARCABOOT` continuam valendo sem mudança. Um dispositivo nomeado que passe de novo por `arca prepare` volta a `ARCABOOT` |
| tests | nenhum teste compara o rótulo real do dispositivo desta mesa com `ARCABOOT`. O `e10` lê `LABEL="ARCABOOT"` de uma captura, que é evidência de 25/08/2026 e não muda |

## Relations

`None - no stored-data shape change`

## Surface

`None - nothing consumed outside`

## Landing

| One-way door | Literal shape | Alternative rejected |
| --- | --- | --- |
| 1. forma do rótulo do `ARCABOOT` ([ADR-0027](../../../docs/adr/0027-o-arcaboot-pode-levar-um-nome.md)) | `ARCABOOT` exato, ou começando com `ARCA-`, os dois sem diferenciar caixa. O prefixo é comparado com `str::get(..5)`, nunca com fatia de bytes, que entra em pânico num rótulo que começa com acento | achar o boot pelo mesmo disco físico do `ARCAVAULT`, com qualquer rótulo: custa a consulta WMI (~2 s) em todo comando, inclusive no `arca list`, que hoje a evita de propósito. Um `arca-nome.txt` no `ARCAVAULT`: não aparece no Explorer, que é onde a igualdade é vista. Prefixo `ARCA` sem hífen: `ARCAVAULT` casaria, e só o sistema de arquivos separaria as duas partições |

- A porta 1 fecha nomes com mais de 6 caracteres. Um nome mais longo, no futuro, exige outro mecanismo, porque o rótulo FAT32 não comporta.
- Nada mais nesta mudança é difícil de reverter. A mensagem de C-10 e as linhas de tela são texto, e se reescrevem num commit.

## Criteria

### S1: Um dispositivo renomeado continua sendo o dispositivo (P1)

O ARCA acha, conta e reconhece o `ARCABOOT` renomeado exatamente como acha o que se chama `ARCABOOT`.

**Acceptance Criteria**

1. WHEN os volumes conectados trazem um `ARCAVAULT` e um volume com o rótulo `ARCA-CASA` THEN `dispositivo::encontrar` SHALL devolver esse volume como o `ARCABOOT` do dispositivo
2. WHEN o rótulo de boot é `ARCABOOT`, `arcaboot` ou `arca-Casa` THEN `dispositivo::encontrar` SHALL reconhecê-lo como `ARCABOOT`, sem diferenciar caixa
3. IF o rótulo começa com `ARCA` sem o hífen e não é `ARCABOOT` (ex.: `ARCACASA`, `ARCAVAULT`) THEN `dispositivo::encontrar` SHALL deixá-lo fora do `ARCABOOT`
4. IF há dois volumes de boot conectados, com rótulos iguais ou diferentes (`ARCABOOT` + `ARCA-CASA`, `ARCA-CASA` + `ARCA-ESCRIT`, `ARCA-CASA` + `ARCA-CASA`) THEN `dispositivo::encontrar` SHALL recusar com `DispositivosDemais` (C-10)
5. IF o rótulo tem menos de 5 caracteres ou começa com caractere não-ASCII (ex.: `ARC`, `ÉRCA-X`) THEN o casamento SHALL responder que não é `ARCABOOT`, sem entrar em pânico
6. IF o disco escolhido no `arca prepare` tem `ARCAVAULT` e uma partição `ARCA-<texto>` THEN a tela do plano SHALL mostrar o aviso `ESTE DISCO JA E UM DISPOSITIVO ARCA`
7. The receita SHALL citar o destino somente por `LABEL=ARCAVAULT`, sem nenhuma referência ao rótulo do `ARCABOOT`

**Independent test:** `DiscosDeMentira` com `ARCAVAULT` em `E:` e `ARCA-CASA` em `R:`. O `encontrar` devolve `R:` como boot, e o `caminho_do_grub` é `R:\boot\grub\grub.cfg`.

### S2: O nome aparece onde o ARCA mostra o dispositivo (P1)

Um dispositivo nomeado se identifica nas telas do ARCA pelo mesmo rótulo que o Explorer mostra.

**Acceptance Criteria**

8. WHILE o rótulo do `ARCABOOT` tem a forma `ARCA-<texto>` the `arca status` SHALL imprimir esse rótulo, como o Windows o devolve, no lugar de `ARCABOOT` na segunda linha do bloco `Dispositivo ARCA` (ex.: `ARCA-CASA ....... R: · FAT32 · 1,6 GB`)
9. IF a recusa C-10 é por mais de um volume de boot e algum deles tem a forma `ARCA-<texto>` THEN a mensagem SHALL listar a letra e o rótulo de cada um (`R: ARCA-CASA, S: ARCA-ESCRIT`) no lugar de `com o rotulo ARCABOOT`, SHALL justificar a recusa pelo que mora no volume de boot (a receita e o estado do job), e SHALL NOT conter `repetido` nem `e pelo rotulo que a receita resolve o destino`, porque os rótulos podem diferir e a receita não cita o de boot. `Desconecte os demais` e `Se voce acabou de preparar um dispositivo` continuam
10. IF a recusa C-10 é por dois `ARCAVAULT` e algum volume de boot conectado tem a forma `ARCA-<texto>` THEN a mensagem SHALL acrescentar a letra e o rótulo de cada volume de boot conectado
11. WHEN o menu do `arca prepare` lista um dispositivo ARCA cujo boot é `ARCA-CASA` THEN a linha SHALL terminar em `JA E UM DISPOSITIVO ARCA (ARCA-CASA)`
12. WHILE nenhum volume conectado tem rótulo na forma `ARCA-<texto>` the system SHALL imprimir o bloco do `arca status`, as recusas de C-10 e o menu do `arca prepare` byte a byte como hoje

**Independent test:** o `arca status` com `ARCA-CASA` em `R:` mostra `ARCA-CASA` no bloco do dispositivo. A mesma chamada com `ARCABOOT` produz a saída de hoje.

## Out of scope

| Excluded | Why |
| --- | --- |
| `arca prepare` criar o dispositivo já nomeado (`--nome`) | o nome se dá no Explorer, depois. O `prepare` continua rotulando `ARCABOOT`, e a releitura dele (`preparacao.rs:548`) continua exata |
| comando do ARCA para renomear | o "Renomear" do Explorer já faz isso, e o ARCA não ganharia nada escrevendo o rótulo |
| o nome no cabeçalho `Dispositivo ARCA: ARCAVAULT (E:)` dos comandos que armam, no `Dispositivo ARCA: ARCABOOT (R:)` do `arca desarmar --dry-run` e no `arca list` | mudaria a saída de seis comandos e a do `list`, e o Explorer e o `arca status` já mostram o nome. No ensaio do `desarmar`, `ARCABOOT` é o papel da partição, como em "Estado no ARCABOOT". Fica para quando fizer falta |
| renomear o `ARCAVAULT` | a receita resolve o destino por `LABEL=ARCAVAULT` (S-3) |
| nome com mais de 6 caracteres | o rótulo FAT32 tem 11, e `ARCA-` ocupa 5 |

## Assumptions

| Assumption | Chosen default | Rationale | Confirmed? |
| --- | --- | --- | --- |
| mecanismo do nome | rotular o `ARCABOOT` como `ARCA-<texto>` | escolhido pelo usuário em 27/09/2026 | y |
| `ARCA-` sem texto | conta como `ARCABOOT`, e aparece como `ARCA-` | é a forma. Recusar pediria uma mensagem nova para um caso sem uso | y - usuário, 28/09/2026 |
| volume que não é FAT32 com rótulo `ARCA-...`, como o `ARCAVAULT` renomeado por engano | o casamento não olha o sistema de arquivos | o caso já termina em recusa: sem `ARCAVAULT` sai `DispositivoAusente`, e com dois volumes de boot sai C-10 nomeando os dois | n |
| pendrive qualquer rotulado `ARCA-...`, conectado junto do dispositivo | conta como segundo volume de boot, e C-10 recusa nomeando-o | recusar e dizer qual é o comportamento seguro de C-10 | y - usuário, 28/09/2026 |
| dois dispositivos guardados com o mesmo nome | o ARCA não percebe | ele vê um dispositivo por vez (C-10), e a unicidade do nome fica com o usuário | n |
| o que a tela mostra | o rótulo inteiro (`ARCA-CASA`), e não só o texto | a tela mostra o mesmo que o Explorer | y - usuário, 28/09/2026 |
| recusa por dois `ARCAVAULT` | enriquecida (AC 10), em vez de deixar a identificação só para o Explorer | é nessa recusa que o ARCA manda desconectar um, e ela precisa dizer qual | y - usuário, 28/09/2026, com o escopo de exibição (`arca status`, recusas de C-10, menu do `prepare`) |

**Open questions:** none - all resolved or logged above.

## Observable

| Surface | Decision | Landing |
| --- | --- | --- |
| command `arca status` | output format | AC 8, AC 12 |
| command `arca prepare` (menu) | output format | AC 11, AC 12 |
| command `arca prepare` (plano) | what the destructive action confirms first | AC 6. A confirmação continua sendo o modelo digitado (S-2), existing |
| todo comando que chama `encontrar` | error shape (C-10) | AC 9, AC 10 |
| todo comando | exit codes | existing - C-10 sai pelo mesmo código de hoje |
| todo comando | flags and defaults | n/a - nenhuma flag nova |
| todo comando | what it prints when it fails halfway | n/a - nada novo é escrito, o nome só é lido |
| coleção de dispositivos | naming criterion | Landing, door 1 |
| coleção de dispositivos | duplicates | Assumptions - o ARCA vê um dispositivo por vez |
| documento README / CONTEXT / PRD | structure, and what the reader does next | Impact - C-16, §5, §6.8, §12, §13, os verbetes `ARCABOOT` e `Dispositivo`, e como renomear no Explorer |

## Sources

- pedido do usuário em 27/09/2026 - os dispositivos têm todos o mesmo nome, e o `ARCABOOT` passa a `ARCA-<texto>`
- PRD, nota do PR-4 - "um `ARCAVAULT` de 445 GB numa linha não é reconhecimento, é uma pista"
- `docs/adr/0027-o-arcaboot-pode-levar-um-nome.md` - a decisão, e por que B-1 e S-3 não mudam
- leitura, sem escrita, do `ARCABOOT` desta mesa (`E:`) em 28/09/2026 - nenhum `LABEL=` nem `ARCABOOT` em `boot/`, `EFI/` ou `syslinux/`, e todo `menuentry` do `grub.cfg` acha a raiz por `search --set -f`. É a evidência de que o lado do boot não depende do rótulo
- `adaptadores/windows/volumes.rs` (`letras_montadas().filter_map(ler_volume)`) e a `CONSULTA` de `adaptadores/windows/particionador.rs` - os dois devolvem todo volume e toda partição sem filtrar por rótulo, então um `ARCA-CASA` chega ao `encontrar` e ao `e_um_dispositivo_arca`
- Linear: WPC-54 (a feature), WPC-55 (S1, AC 1 a 7), WPC-56 (S2, AC 8 a 12), WPC-57 (a documentação do Impact). WPC-53, em `Done`, fechou a lacuna do `sondar` e do `verify --completo` que este plano deixava fora de escopo, e deixou o achado do "rótulo repetido" que o Impact agora absorve
