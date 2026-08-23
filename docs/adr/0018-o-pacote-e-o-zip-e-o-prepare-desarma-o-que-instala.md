# O pacote é o zip, e o `arca prepare` desarma o que acabou de instalar

Decidido em 23/08/2026, na etapa E10.

## O Clonezilla publica dois artefatos, e o dispositivo desta mesa veio do outro

PR-1 manda fixar a versão e compilar o SHA256 no binário. A versão escolhe-se
sozinha: **`3.3.3-15` é a que está bootando aqui** — o `grub.cfg` do
dispositivo traz `hostname=cl-3.3.3-15` em cada `menuentry`, e é sobre esse
ambiente que rodaram os quatro marcos em hardware deste projeto.

O que não se escolhia sozinho era o **formato**. O projeto publica
`clonezilla-live-3.3.3-15-amd64.iso` e `…-amd64.zip`, e o zip é o que se extrai
direto numa partição FAT32 — que é exatamente o que `arca prepare` faz.

Baixado o zip e comparado o `boot/grub/grub.cfg` dele com o
`grub-clonezilla-original.cfg` que a E4 preservou do dispositivo, saíram **duas
diferenças, e só duas**:

| | zip | dispositivo desta mesa |
|---|---|---|
| `noeject` | 13 ocorrências, uma por `menuentry` | **nenhuma** |
| carimbo do rodapé | `Created at time: Sun 05 Jul 2026 04:11:28 AM BST` | `… 04:11:22 AM BST` |

Tirado o `noeject`, os dois arquivos são **idênticos byte a byte** — 210 linhas
cada, e nenhuma outra divergência.

## Seis segundos, e o que eles respondem

Seis segundos é o `ocs-live-dev` gerando os dois artefatos na mesma execução.
**É a mesma build**, e o dispositivo desta mesa veio do **ISO**.

Isso responde uma pergunta que ia ficar aberta — *"de onde veio este
dispositivo?"* — sem dedução nenhuma, o que importa num projeto que já pagou
cinco vezes por deduzir a origem de um artefato. E responde pelo caminho mais
barato possível: dois números de segundo num rodapé que ninguém tinha olhado.

## O `noeject` não é argumento contra o zip; é a favor

`noeject` diz ao live system para **não ejetar a mídia** ao desligar. Para
mídia óptica, ejetar é o certo — o ISO não o tem por isso. Para um SSD por USB,
ejetar é o oposto do que se quer.

**O dispositivo desta mesa é que carrega o parâmetro de outra mídia.** Ele
funciona assim mesmo — quatro marcos o provam —, e o zip é quem traz o
parâmetro adequado ao que o ARCA prepara.

E o resto do ARCA não sente a diferença, por decisões que já estavam tomadas:

- **o estado inerte se reconstrói do `grub.cfg` corrente**
  ([ADR-0005](0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md));
- **o bloco do ARCA deriva do `live-toram` do próprio dispositivo**
  ([ADR-0007](0007-o-bloco-do-arca-deriva-do-live-toram.md)).

As duas existem para o ARCA funcionar sobre o arquivo que estiver lá. O
`noeject` viaja para dentro do bloco do ARCA de graça, sem que ninguém precise
saber que ele existe — que é o argumento do ADR-0007 visto pelo outro lado.

O custo é oito bytes na linha de comando do kernel. O orçamento do §10.2.3
reserva 512 para o `menuentry` base e o desta mesa mede 471; o do pacote mede
479, e a folga cai de 41 para 33. Há teste guardando isso.

## O zip entrega um `grub.cfg` que **não** está inerte

`set default="0"`. O
[ADR-0005](0005-o-estado-inerte-se-reconstroi-do-grub-cfg-corrente.md) tem uma
seção sobre exatamente isso, escrita quatro etapas antes de alguém saber que ela
seria necessária aqui:

> `"0"` aponta por **posição**, e a posição muda: o bloco do ARCA entra
> **antes** do `live-default` e passa a ser o índice 0. Um dispositivo com `set
> default="0"` está armado no instante em que o bloco é inserido, sem que
> ninguém toque no `set default`. **Não é o estado inerte — é um estado que
> parece inerte.**

Um dispositivo entregue assim ficaria armado no primeiro `arca backup`, e o
`arca status` diria que ele está inerte.

**Decidimos que `arca prepare` desarma o que acabou de instalar.** Extraído o
pacote, o `grub.cfg` passa por `grub::desarmar` antes de o dispositivo ser
declarado pronto. Não é zelo — é o que faz o §4.4 valer para um dispositivo
recém-preparado.

E o desarmar sobre o pacote foi conferido pelo oráculo mais forte que esta
etapa tinha: **desarmar o `grub.cfg` do zip produz exatamente o `grub.cfg`
inerte do dispositivo desta mesa**, a menos das duas diferenças de origem já
medidas. Um dispositivo preparado pelo `arca prepare` fica no mesmo estado que o
dispositivo com que este projeto rodou tudo.

> **Isto é o padrão que este projeto revisa a cada etapa**: peça nova encaixada
> em peça antiga que ninguém releu ao encaixar. O `grub::desarmar` nasceu na E4
> e nunca tinha visto o `grub.cfg` do pacote — ele só conhecia o do dispositivo,
> que já vinha com `set default="live-default"` porque **alguém o desarmara à
> mão** antes de o ARCA existir.
>
> A diferença é que desta vez a releitura aconteceu antes de o defeito rodar, e
> não depois.

## O SHA256 vem de duas fontes, e é isso que o torna verificação

PR-1 diz que o checksum é **compilado no binário** e nunca baixado junto do
arquivo — um checksum servido pelo mesmo servidor que serve o arquivo não prova
coisa alguma, porque quem pudesse trocar um trocaria o outro.

O número foi obtido de duas fontes independentes em 23/08/2026, e as duas dizem
o mesmo:

| Fonte | O que ela é |
|---|---|
| `free.nchc.org.tw/clonezilla-live/stable/CHECKSUMS.TXT` | O mirror do próprio projeto, em Taiwan, onde o Clonezilla é desenvolvido |
| `certutil -hashfile … SHA256` sobre o arquivo baixado do **SourceForge** | Outro servidor, outra rota |

`00cee7700433e63017e2ea9eb40519108829710132364a8028a6c039a6046304`, 561.478.648
bytes. Servidores diferentes, o mesmo número — o mais perto de verificação
independente que este caso admite.

O `CHECKSUMS.TXT` está preservado em `recursos/capturas/`, e há teste cobrando
que a constante do binário esteja na seção **`SHA256SUMS:`** dele. A seção
importa: o arquivo traz seis algoritmos, e o **B3SUM tem os mesmos 64 dígitos
hexadecimais**. Casar só a linha deixaria um B3SUM passar por SHA256 — e como o
`certutil` não sabe Blake3, todo download seria reprovado com uma mensagem
falando do arquivo.

## O conteúdo é conferido antes de escrever, e o `bsdtar` sair com zero não basta

Um zip que extrai sem erro e sem o `bootx64.efi` produz um dispositivo que não
boota — e isso só se descobre depois de o Windows ter sido apagado, porque é aí
que alguém precisa dele.

Por isso o comando lista o pacote (`bsdtar -t`) e confere que os quatro
caminhos que fazem um dispositivo bootar estão lá, **antes** de extrair:
`EFI/boot/bootx64.efi`, `live/vmlinuz`, `live/initrd.img`,
`boot/grub/grub.cfg`.

A comparação normaliza `\` para `/` e ignora a caixa, porque quem lista é o
`bsdtar` e quem confere é o Windows — e trocar as duas coisas produziria um
"não achei" que não quer dizer nada, na pior hora possível.

## A armadilha do `tar`, medida e cara

**`tar` no `PATH` pode não ser o `bsdtar`.** Medido em 23/08/2026 nesta
máquina: com o Git para Windows instalado, `tar` resolve para o **GNU tar
1.35** do `/usr/bin`, que **não abre zip** — ele responde *"This does not look
like a tar archive"* e sai com erro.

Quem abre zip é o `C:\Windows\System32\tar.exe`, que é o `bsdtar 3.8.8`. Os dois
se chamam `tar.exe`; o campo que os separa sem ambiguidade é o
`OriginalFilename` do executável — `bsdtar` num, `tar` no outro.

Por isso o adaptador chama `curl` e `bsdtar` **por caminho absoluto**, e nunca
pelo nome. O modo de falha é caro: o `arca prepare` extrai o pacote *depois* de
ter apagado o disco, e um `tar` que não entende zip falharia com o dispositivo
já destruído e nada instalado nele.

> **É a segunda vez que esta mesma ferramenta engana por homonímia.** O plano da
> E10 registrava `tar.exe 10.0.26100` como a versão do bsdtar, e a E11 corrigiu:
> aquilo era o `ProductVersion` do Windows, não o `FileVersion` do bsdtar — o
> sétimo caso de número medido na coisa errada. Agora não é o número que está
> errado, é o programa.
>
> Os outros três comandos que o adaptador roda — `chkdsk`, `certutil`,
> `shutdown` — continuam pelo nome: nenhum tem homônimo conhecido, e mudá-los
> agora seria alteração sem medição em caminho já exercitado em hardware.

## Consequências

- **PR-1 ganha número, e ele tem duas fontes.** §9.6 e `src/pacote.rs`.
- **PR-2 rodou**, no segundo marco de 23/08: `--iso <caminho>` conferiu o mesmo
  SHA256 sem passar pelo `curl`.
- **PR-3 rodou**: a cópia do pacote fica no `ARCAVAULT`.
- §11 ganha a armadilha do `tar` homônimo.
- **O que fica em aberto é a próxima versão.** Quando o Clonezilla publicar a
  `3.3.4`, trocar a constante `VERSAO` sem trocar a URL e o SHA256 baixaria
  outra coisa — e há três testes ligando os três valores, mais um que exige que
  a versão fixada seja a que o `grub.cfg` capturado roda. Trocar de versão vai
  quebrar esse último de propósito, e o certo é recapturar o `grub.cfg` de um
  dispositivo que rodou a versão nova.
