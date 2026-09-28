# Triage Labels

As skills falam em termos de cinco papéis canônicos de triagem. Este arquivo mapeia esses papéis para o que o Linear usa neste repositório: o **estado** do issue, que é sempre um dos estados do Linear, mais uma **label** quando o estado sozinho não diz o papel.

| Papel em mattpocock/skills | No Linear                                  | Significado                                        |
| -------------------------- | ------------------------------------------ | -------------------------------------------------- |
| `needs-triage`             | estado `Backlog`                           | O mantenedor precisa avaliar este issue            |
| `needs-info`               | estado `Backlog` + label `needs-info`      | Aguardando mais informações de quem reportou       |
| `ready-for-agent`          | estado `Todo` + label `ready-for-agent`    | Totalmente especificado, pronto para um agente AFK |
| `ready-for-human`          | estado `Todo` + label `ready-for-human`    | Exige implementação humana                         |
| `wontfix`                  | estado `Canceled`                          | Não será tratado                                   |

As labels `ready-for-agent` e `ready-for-human` já existem, como labels do time **WPC Solutions**. A `needs-info` ainda não. Na primeira vez que for usada, crie-a no mesmo time com `save_issue_label` (`teamId` do WPC Solutions), e não como label do workspace.

## Os estados são os do Linear

A triagem não inventa estado. O issue anda pelo fluxo do time:

| Estado        | Quando                                                             |
| ------------- | ------------------------------------------------------------------ |
| `Backlog`     | Chegou e ainda não foi triado, ou espera informação (`needs-info`) |
| `Todo`        | Triado e pronto para começar: é onde moram os dois `ready-for-*`   |
| `In Progress` | Alguém, humano ou agente, pegou o issue                            |
| `In Review`   | O trabalho terminou e espera revisão, como um PR aberto            |
| `Done`        | Feito                                                              |
| `Canceled`    | Não será feito: é o `wontfix`                                      |
| `Duplicate`   | Repete outro issue, apontado em `duplicateOf`                      |

A label de triagem fica no issue enquanto ele anda. Um `ready-for-agent` que vai para `In Progress` continua dizendo quem devia implementá-lo.

## Como aplicar um papel

Quando uma skill mencionar um papel (ex.: "apply the AFK-ready triage label"), aplique a linha correspondente da primeira tabela:

1. Ponha o issue no estado: `save_issue` com `id` e `state`.
2. Acrescente a label quando a linha pede uma: `addLabels`.
3. Ao trocar de papel, tire a label do papel anterior com `removeLabels`. Por exemplo, de `needs-info` para `ready-for-agent`: vai para `Todo`, perde `needs-info` e ganha `ready-for-agent`.

Edite a coluna do meio para refletir o vocabulário que você de fato usa.
