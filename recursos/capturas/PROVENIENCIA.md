# De onde vieram estas capturas

O parser do `bcdedit` é o único ponto do ARCA onde uma leitura errada leva a
máquina a bootar no lugar errado com uma receita armada. Testá-lo contra
exemplos inventados provaria que ele lê o que eu imaginei que o `bcdedit`
escreve. Estes arquivos são o que ele **escreveu de verdade**.

Todos foram convertidos de CP850 para UTF-8 na gravação, e só nisso. As
quebras de linha CRLF do `bcdedit` estão preservadas — o `.gitattributes`
marca esta pasta como `-text` para que o git não as normalize.

| Arquivo | O que é |
|---|---|
| `bcdedit-enum-firmware-pt.txt` | `bcdedit /enum firmware` desta máquina, 22/08/2026, console em CP850 |
| `bcdedit-enum-firmware-en.txt` | **o mesmo BCD, no mesmo instante**, pelo mesmo `bcdedit`, com os recursos `en-US` ao lado |
| `bcdedit-enum-firmware-legado-pt.txt` | `E:\ARCA-LOGS\nvram-windows-antes.txt`, capturado em 20/08/2026, antes de a entrada ser renomeada |

## Por que o par pt/en prova alguma coisa

O plano de implementação nomeia a fixture em inglês como metade do risco desta
etapa, e com razão: um parser afinado num só idioma passa em todo teste e
falha na máquina de outra pessoa.

As duas primeiras capturas descrevem **a mesma configuração de boot**, lida com
segundos de diferença. Não são uma tradução de outra: são duas leituras do
mesmo dado. Isso permite o teste que fecha o risco — o parser tem de extrair
delas exatamente o mesmo resultado, campo a campo. Qualquer dependência de
texto traduzido aparece como diferença.

O `bcdedit.exe` do Windows carrega suas mensagens de
`System32\<idioma>\bcdedit.exe.mui`. Esta máquina tem `pt-BR` e `en-US`
instalados. Copiando o `bcdedit.exe` para uma pasta onde só existe
`en-US\bcdedit.exe.mui`, o carregador de recursos usa o que está ali — e a
mesma consulta ao mesmo BCD sai em inglês.

## O que o par confirma

- **Só `identificador` é traduzido** entre os nomes de campo. `device`, `path`,
  `description`, `locale`, `inherit`, `displayorder`, `timeout` e os demais
  saem idênticos nos dois idiomas. É a fundação §3.1 do PRD, agora com as duas
  metades medidas.
- **Os títulos de bloco também são traduzidos** — `Windows Boot Manager` /
  `Gerenciador de Inicialização do Windows`. O PRD não diz isso, e é por isso
  que o parser não pode usá-los para decidir nada.
- **A entrada legada é reconhecível pela `description`**, que não é traduzida.

## A entrada desta máquina mudou de nome entre as capturas

A captura de 20/08 traz `description Clonezilla`; a de 22/08 traz
`description ARCA`. O identificador é o mesmo nas duas —
`{f4057bd0-65a4-11f1-b0f1-aa4ed9bd2b34}`.

Não é acidente de captura: é exatamente o que C-4 descreve, dos dois lados. A
captura antiga é a única evidência real do caso "não há entrada `ARCA`, há a
legada `Clonezilla`", e é por isso que ela está aqui em vez de ter sido
descartada por estar desatualizada.

## O que nenhuma delas contém

**Nenhum `bootsequence`.** Não há job armado nesta máquina, e armar um é a
etapa E7 — a E2 não escreve no firmware. O formato do boot único está coberto
por caso construído no teste, marcado como tal, e a E7 o confirma contra
hardware quando armar pela primeira vez.

**Nenhuma menção a `Removable Media` ou `External hard disk media`.** Estas
palavras não são do `bcdedit`: são valores de `MediaType` do WMI
(`Win32_DiskDrive`, em `cimwin32.dll`). Nem o `bcdedit.exe` nem os seus
`.mui` contêm qualquer uma delas — procurado nos dois idiomas. Ver o que
`src/firmware.rs` diz sobre C-6.
