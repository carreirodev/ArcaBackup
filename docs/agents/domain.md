# Domain Docs

Como as engineering skills devem consumir a documentação de domínio deste repositório ao explorar o código.

## Antes de explorar, leia estes

- **`CONTEXT.md`** na raiz do repositório, ou
- **`CONTEXT-MAP.md`** na raiz, se existir: ele aponta para um `CONTEXT.md` por contexto. Leia cada um que seja relevante ao tema.
- **`docs/adr/`**: leia os ADRs que tocam a área em que você está prestes a trabalhar. Em repositórios multi-context, verifique também `src/<context>/docs/adr/` para decisões restritas a um contexto.

Se algum desses arquivos não existir, **siga em silêncio**. Não sinalize a ausência; não sugira criá-los antecipadamente. A skill `/domain-modeling` (alcançada via `/grill-with-docs` e `/improve-codebase-architecture`) os cria de forma preguiçosa, quando termos ou decisões efetivamente forem resolvidos.

## Estrutura de arquivos

Repositório single-context (o caso da maioria — e o deste repositório):

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-event-sourced-orders.md
│   └── 0002-postgres-for-write-model.md
└── src/
```

Repositório multi-context (indicado pela presença de `CONTEXT-MAP.md` na raiz):

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← decisões de escopo sistêmico
└── src/
    ├── ordering/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← decisões específicas do contexto
    └── billing/
        ├── CONTEXT.md
        └── docs/adr/
```

## Use o vocabulário do glossário

Quando sua saída nomear um conceito de domínio (no título de um issue, numa proposta de refatoração, numa hipótese, no nome de um teste), use o termo como definido em `CONTEXT.md`. Não derive para sinônimos que o glossário evita explicitamente.

Se o conceito de que você precisa ainda não está no glossário, isso é um sinal: ou você está inventando linguagem que o projeto não usa (reconsidere) ou existe uma lacuna real (anote-a para o `/domain-modeling`).

## Sinalize conflitos com ADRs

Se sua saída contradiz um ADR existente, exponha isso explicitamente em vez de sobrescrever em silêncio:

> _Contradiz o ADR-0007 (event-sourced orders), mas vale reabrir porque…_
