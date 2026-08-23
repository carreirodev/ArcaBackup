# ARCA

Automatizador de Clonezilla para backup e restauração de imagem de disco, de uso pessoal. O ARCA nunca lê nem escreve disco: ele prepara o ambiente, monta a receita, dispara o boot único e colhe o que o Clonezilla deixou escrito.

## Language

### O dispositivo

**Dispositivo**:
O SSD externo que carrega o Clonezilla e as imagens juntos, com as partições `ARCABOOT` e `ARCAVAULT`.
_Evitar_: pendrive, mídia, unidade, drive

**ARCABOOT**:
A partição FAT32 do dispositivo, de onde a máquina boota. Guarda o Clonezilla, o `grub.cfg` e o estado do job. Está sempre fora da imagem.
_Evitar_: partição de boot, EFI

**ARCAVAULT**:
A partição NTFS do dispositivo, onde as imagens e os logs ficam. É o que o Clonezilla monta como `/home/partimag`.
_Evitar_: repositório, storage, cofre

**Imagem**:
Uma pasta no `ARCAVAULT` contendo o resultado de um `savedisk`. Nomeada pelo usuário, nunca sobrescrita.
_Evitar_: backup, snapshot, ponto de restauração

**Resíduo**:
Pasta de imagem sem `MD5SUMS` — rastro de um backup interrompido. Não é imagem, e o ARCA nunca escreve por cima de uma.
_Evitar_: imagem corrompida, imagem parcial

### A operação

**Receita**:
A string que o Clonezilla executa sozinho no boot desatendido, gravada no `grub.cfg` a cada operação. Um pipe dentro dela invalida a string inteira.
_Evitar_: script, comando, configuração

**Job**:
Uma operação armada e ainda não colhida. Existe entre o reinício e a leitura do desfecho.
_Evitar_: tarefa, execução, operação pendente

**Colher**:
Ler o que há no lugar do desfecho, julgá-lo pelo selo e dizer o que era. Encerra o job — inclusive quando o que se encontra é nada, que é uma resposta. O que não encerra é não ter conseguido ler.
_Evitar_: verificar, conferir, finalizar

**Armar**:
Gravar a receita no `grub.cfg` e marcar o boot único no firmware. É o ponto sem volta.
_Evitar_: agendar, disparar, configurar

**Desarmar**:
Devolver o `grub.cfg` ao estado inerte e limpar a marca de boot único. Acontece incondicionalmente como primeiro passo de todo comando.
_Evitar_: cancelar, limpar, resetar

**Boot único**:
O `bootsequence` do `{fwbootmgr}`: a marca que manda o firmware bootar por uma entrada **no próximo reinício, uma vez só**. O firmware a consome ao usá-la. É o único jeito pelo qual o ARCA leva a máquina ao dispositivo, e funciona sobre uma entrada que não está na ordem permanente — medido em 22/08/2026.
_Evitar_: BootNext, boot temporário, agendamento de boot

**Ordem permanente**:
O `displayorder` do `{fwbootmgr}`: por onde a máquina boota quando ninguém pediu nada. Ela tem **três donos**, e nenhum deles é quem se supunha: o ciclo de boot pelo dispositivo põe a entrada dele na frente ([ADR-0009](docs/adr/0009-a-ordem-permanente-muda-no-ciclo-de-boot.md)); o Windows a espelha do BCD ao subir; e uma **restauração a devolve ao que está dentro da imagem**, porque o BCD mora na partição EFI e ela é restaurada junto ([ADR-0012](docs/adr/0012-a-restauracao-devolve-a-ordem-permanente-de-dentro-da-imagem.md)).

**O ARCA a toca em um lugar só, e nunca para acrescentar caminho**: ao colher, ele põe o `{bootmgr}` no topo (C-13, [ADR-0013](docs/adr/0013-colher-devolve-o-bootmgr-ao-topo-da-ordem.md)) — sem remover nada, e conferindo com uma releitura. Armar e desarmar continuam proibidos de mexer nela (C-5), e releem para provar que não mexeram. `arca status` só lê e avisa.
_Evitar_: BootOrder, ordem de boot, boot padrão

**Estado inerte**:
O `grub.cfg` sem nenhum `menuentry --id arca-backup` e com `set default="live-default"`, e o `{fwbootmgr}` sem `bootsequence`. Um dispositivo inerte boota no menu do Clonezilla e espera alguém. Não é uma cópia guardada: é o que sai de aplicar a regra ao `grub.cfg` que está no dispositivo.
_Evitar_: estado limpo, estado original, estado padrão

**`set default`**:
A diretiva do `grub.cfg` que decide em que entrada a máquina boota sozinha. É ela que faz o boot ser desatendido — o `menuentry` da receita, sozinho, só põe mais uma linha no menu.
_Evitar_: entrada padrão, boot padrão

**Selo**:
Identificador aleatório gerado ao armar, embutido na receita e devolvido pelo Clonezilla junto do desfecho. É o que liga um desfecho ao job que o produziu — o relógio do Clonezilla não serve para isso.
_Evitar_: id, timestamp, marca de tempo

**Job fantasma**:
Um desfecho encontrado no dispositivo que não pertence ao job pendente. Reconhecível porque o selo não bate.
_Evitar_: job órfão, estado sujo

### O que se colhe

**Desfecho**:
Se a operação terminou ou não: `ARCA_BACKUP=OK`, `ARCA_RESTORE=FALHOU`. Escrito pelo Clonezilla em arquivo, nunca em tela.
_Evitar_: resultado, status, saída

**Veredito**:
O parecer do `ocs-chkimg` sobre a integridade de uma imagem: aprovada ou reprovada. É independente do desfecho — um backup pode terminar e a imagem ser reprovada.
_Evitar_: verificação, checagem, validação
