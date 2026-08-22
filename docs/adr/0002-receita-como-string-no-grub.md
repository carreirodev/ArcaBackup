# A receita é uma string no grub.cfg, não um script em arquivo

A receita que o Clonezilla executa sozinho é montada como string única e gravada no `grub.cfg`. Isso obriga o ARCA a validar escapes antes de gravar e proíbe pipes: um `|` invalida a string inteira, e o Clonezilla abre o menu interativo sem executar nada e sem avisar. A alternativa natural — gravar um `custom-ocs` em arquivo e apontar para ele — devolveria os pipes e deixaria o `if/then/else` legível.

Ficamos com a string porque **é o mecanismo medido em hardware**: backup e restauração completos rodaram assim, ponta a ponta, sem intervenção.

## Considered Options

O caminho por arquivo carrega dois riscos não medidos. O `toram` — mantido por decisão, para não acoplar o live system ao dispositivo que ele remonta — pode desmontar o medium antes de o script ser lido. E a ordem entre a montagem do `ocs_repository` e a execução do `ocs_live_run` é o que decidiria se um script no `ARCAVAULT` está alcançável no momento da chamada. Trocar exigiria remedir em hardware tudo o que já está validado, para ganhar legibilidade num arquivo que o ARCA gera e ninguém lê à mão.

## Consequências

C-2 — validar a receita antes de gravar — deixa de ser zelo e vira requisito estrutural. É a única barreira entre um escape errado e uma máquina que reinicia, abre um menu em inglês técnico e fica parada esperando alguém que já saiu de perto.
