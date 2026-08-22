# Um job é ligado ao seu desfecho por selo, nunca por data

O Clonezilla lê o RTC — hora local do Windows — como se fosse UTC, e roda 3 h adiantado de forma permanente. Uma trava anterior comparava a data do `arca-fim.txt` com a do job armado para decidir se aquele desfecho era do job corrente, e reprovou um backup perfeito. Decidimos que o ARCA gera um identificador aleatório ao armar — o **selo** —, grava no `estado.json`, embute na receita, e só aceita o desfecho cujo selo case.

## Consequências

Um mecanismo só resolve quatro casos que antes eram indistinguíveis entre si: desfecho de um job anterior, desfecho vindo de dentro de uma imagem antiga (job fantasma), desfecho ausente porque o boot nunca aconteceu, e arquivo truncado por desligamento no meio.

O preço é que o selo atravessa três lugares — `estado.json`, receita e `arca-fim.txt` — e mudar seu formato obriga a mexer nos três de uma vez.

Comparar datas entre Windows e Clonezilla nunca é correto neste projeto, mesmo quando parece funcionar: o deslocamento de 3 h é permanente e continua ali para a próxima pessoa que tentar.
