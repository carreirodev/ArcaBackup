# Issue tracker: GitHub

Issues e specs deste repositório vivem como GitHub issues. Use o CLI `gh` para todas as operações.

## Convenções

- **Criar um issue**: `gh issue create --title "..." --body "..."`. Use um heredoc para corpos multilinha.
- **Ler um issue**: `gh issue view <number> --comments`, filtrando comentários com `jq` e buscando também as labels.
- **Listar issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'` com os filtros `--label` e `--state` apropriados.
- **Comentar em um issue**: `gh issue comment <number> --body "..."`
- **Aplicar / remover labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Fechar**: `gh issue close <number> --comment "..."`

Infira o repositório a partir de `git remote -v`; o `gh` faz isso automaticamente quando executado dentro de um clone.

## Pull requests como superfície de triagem

**PRs as a request surface: no.** _(Defina como `yes` se este repositório tratar PRs externos como pedidos de feature; o `/triage` lê esse flag.)_

Quando definido como `yes`, PRs passam pelas mesmas labels e estados dos issues, usando os equivalentes `gh pr`:

- **Ler um PR**: `gh pr view <number> --comments` e `gh pr diff <number>` para o diff.
- **Listar PRs externos para triagem**: `gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments`, mantendo apenas `authorAssociation` igual a `CONTRIBUTOR`, `FIRST_TIME_CONTRIBUTOR` ou `NONE` (descartar `OWNER`/`MEMBER`/`COLLABORATOR`).
- **Comentar / rotular / fechar**: `gh pr comment`, `gh pr edit --add-label`/`--remove-label`, `gh pr close`.

O GitHub compartilha um único espaço de numeração entre issues e PRs, então um `#42` isolado pode ser qualquer um dos dois: resolva com `gh pr view 42` e caia de volta para `gh issue view 42`.

## Quando uma skill disser "publish to the issue tracker"

Crie um GitHub issue.

## Quando uma skill disser "fetch the relevant ticket"

Execute `gh issue view <number> --comments`.

## Operações de wayfinding

Usadas pelo `/wayfinder`. O **mapa** é um único issue com issues **filhos** como tickets.

- **Mapa**: um único issue rotulado `wayfinder:map`, contendo o corpo com Notes / Decisions-so-far / Fog. `gh issue create --label wayfinder:map`.
- **Ticket filho**: um issue vinculado ao mapa como sub-issue do GitHub (`gh api` no endpoint de sub-issues). Onde sub-issues não estiverem habilitados, adicione o filho a uma task list no corpo do mapa e coloque `Part of #<map>` no topo do corpo do filho. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Uma vez reivindicado, o ticket é atribuído ao dev responsável.
- **Bloqueio**: use as **dependências nativas de issues** do GitHub, a representação canônica e visível na UI. Adicione uma aresta com `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`, onde `<blocker-db-id>` é o **database id** numérico do bloqueador (`gh api repos/<owner>/<repo>/issues/<n> --jq .id`, _não_ o `#number` nem o `node_id`). O GitHub reporta `issue_dependencies_summary.blocked_by` (apenas bloqueadores abertos, o gate ativo). Onde dependências não estiverem disponíveis, use como fallback uma linha `Blocked by: #<n>, #<n>` no topo do corpo do filho. Um ticket está desbloqueado quando todos os bloqueadores estão fechados.
- **Consulta de fronteira**: liste os filhos abertos do mapa (`gh issue list --state open`, restrito às sub-issues / task list do mapa), descarte os que tiverem bloqueador aberto (`issue_dependencies_summary.blocked_by > 0`, ou um issue aberto na linha `Blocked by`) ou assignee; o primeiro na ordem do mapa vence.
- **Reivindicar**: `gh issue edit <n> --add-assignee @me`, a primeira escrita da sessão.
- **Resolver**: `gh issue comment <n> --body "<answer>"`, depois `gh issue close <n>`, depois anexe um ponteiro de contexto (gist + link) ao Decisions-so-far do mapa.
