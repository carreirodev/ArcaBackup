# O `ARCABOOT` pode levar um nome, e o `ARCAVAULT` não

Decidido em 27/09/2026. Nasce C-16, e C-10 muda de redação.

## O que estava errado

Todo dispositivo ARCA se apresenta com os mesmos dois rótulos. No Explorer, cada SSD é `ARCAVAULT (E:)` e `ARCABOOT (R:)`. No menu do `arca prepare`, dois SSDs do mesmo modelo saem na mesma linha. E o `arca status` imprime `ARCABOOT` para qualquer um.

O PRD escreveu a lacuna no dia em que nasceu o aviso `ESTE DISCO JA E UM DISPOSITIVO ARCA`: *"um `ARCAVAULT` de 445 GB numa linha não é reconhecimento, é uma pista"*. Quem tem dois SSDs iguais na mesa não tem como saber qual é o velho.

## Por que o rótulo do `ARCABOOT` está livre

Nada do lado do boot depende dele, e as três razões estão no código:

- **A receita só cita o `ARCAVAULT`.** O destino é `ocs_repository="dev:///LABEL=ARCAVAULT"` (`receita.rs`), e é isso que S-3 governa.
- **O GRUB acha a própria raiz por arquivo, e não por rótulo.** Todo `menuentry`, inclusive o do ARCA, faz `search --set -f /live/vmlinuz` (`grub.rs`). Lido em 28/09/2026 no `ARCABOOT` do dispositivo desta mesa: nenhum `LABEL=` nem `ARCABOOT` em `boot/`, `EFI/` ou `syslinux/`.
- **A entrada de firmware aponta para a partição.** O device path dela carrega o PARTUUID da `ARCABOOT` ([ADR-0025](0025-o-arca-particiona-em-gpt.md)), e não o rótulo.

Quem depende do rótulo é o lado Windows do próprio ARCA, em três pontos:

- `dispositivo::encontrar`, que acha a partição onde gravar a receita e o `estado.json`;
- a contagem de C-10;
- o reconhecimento de dispositivo existente no `arca prepare`.

Os três passam a casar a forma nova. **B-1** (localizar o dispositivo pelo `ARCAVAULT`) e **S-3** (destino sempre por LABEL) não mudam.

## A forma: `ARCA-` e até seis caracteres

- **`ARCABOOT` continua valendo.** Vale para todo dispositivo preparado até hoje, e para todo que o `arca prepare` fizer: ele rotula `ARCABOOT`, e a releitura dele confere esse rótulo exato.
- **`ARCA-<texto>`**, sem diferenciar caixa, como o resto do ARCA compara rótulo. O rótulo de um volume FAT32 tem 11 caracteres, e `ARCA-` ocupa 5, então o texto tem até 6: `ARCA-CASA` e `ARCA-ESCRIT` cabem, `ARCA-ESCRITORIO` não.
- **O hífen é o que separa.** Sem ele, o prefixo `ARCA` casaria o `ARCAVAULT`, e só o sistema de arquivos distinguiria as duas partições.
- **O prefixo se compara com `str::get(..5)`, e não com fatia de bytes.** Num rótulo como `ARCAÉ`, o `É` ocupa o quinto e o sexto byte, e `&rotulo[..5]` pararia entre os dois. Num rótulo mais curto que o prefixo, como `ARC`, cairia fora do fim. Em Rust, os dois casos são pânico, e não truncamento. É o mesmo cuidado que `imagens::cortar` documenta.

Quem dá o nome é o usuário, pelo "Renomear" do Explorer. O ARCA só lê.

## O que foi rejeitado

- **Achar o `ARCABOOT` pelo mesmo disco físico do `ARCAVAULT`, com qualquer rótulo.** Fecharia de graça a pendência do rótulo órfão, mas custa a consulta WMI de uns 2 s em todo comando, inclusive no `arca list`, que a evita de propósito (`portas/discos.rs`).
- **Um `arca-nome.txt` na raiz do `ARCAVAULT`, como a descrição de L-3.** O nome apareceria nas telas do ARCA e em nenhum outro lugar, e é no Explorer que os dois SSDs aparecem iguais.
- **Um comando do ARCA para renomear.** O Explorer já faz isso.

## O que C-10 passa a contar

Dois volumes de boot continuam sendo recusa, com rótulos iguais ou diferentes: `ARCABOOT` com `ARCA-CASA`, ou `ARCA-CASA` com `ARCA-ESCRIT`.

Duas coisas mudam na mensagem:

- **A recusa por dois volumes de boot deixa de dizer que eles têm o mesmo rótulo.** Ela dizia `com o rotulo ARCABOOT`. Quando algum dos volumes tem nome, isso deixa de ser verdade, e a mensagem passa a listar a letra e o rótulo de cada um. Perde também a justificativa de que *"é pelo rótulo que a receita resolve o destino"*: a receita nunca citou o rótulo de boot, e o que torna dois volumes de boot ambíguos é que a receita e o estado do job são gravados neles.
- **A recusa por dois `ARCAVAULT` acrescenta os volumes de boot conectados.** É a que aparece quando há dois dispositivos inteiros na mesa, porque o `ARCAVAULT` é contado primeiro. É nela que o ARCA manda desconectar um, e ela precisa dizer qual.

Sem nenhum volume nomeado, as duas saem byte a byte como hoje.

## O que fica como estava

- O `arca prepare` rotula `ARCABOOT`. Preparar por cima de um dispositivo nomeado devolve o `ARCABOOT`, e apaga as imagens como sempre. A novidade é que, antes de apagar, a tela diz o nome.
- A recusa do dispositivo partido, que confere o mesmo disco físico, continua valendo para os quatro comandos que armam: `backup`, `restore`, `sondar` e `verify --completo` (`prevoo::julgar_o_dispositivo`).
- Os textos que usam `ARCABOOT` como nome da partição continuam como estão, porque ali a palavra é o papel da partição, e não o rótulo dela. É o caso de "o ARCABOOT deste dispositivo" e "Estado no ARCABOOT".

## Uma lacuna que o prefixo alargaria, e que fechou antes dele

O prefixo alarga o conjunto de volumes que podem ocupar o papel de boot: um pendrive qualquer rotulado `ARCA-...`, conectado ao lado do `ARCAVAULT` de um dispositivo, seria achado como o `ARCABOOT` dele. Quem pega esse caso é a recusa do dispositivo partido, que confere se os dois estão no mesmo disco físico.

Quando esta decisão foi escrita, `sondar` e `verify --completo` armavam sem essa recusa. A lacuna era anterior ao prefixo, e foi fechada à parte em 28/09/2026 (WPC-53, commits `10dd87c` e `630c8ef`). Hoje os quatro comandos que armam a chamam antes da pergunta ou da confirmação, inclusive no `--dry-run`.
