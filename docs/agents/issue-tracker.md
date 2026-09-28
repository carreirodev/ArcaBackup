# Issue tracker: Linear

Issues e specs deste repositório vivem no Linear, no projeto **ArcaBackup** (`P-WPC-7`) do time **WPC Solutions**. A chave do time é `WPC`, então os issues são `WPC-<n>`.

Use as ferramentas do MCP do Linear (plugin `linear`, prefixo `mcp__plugin_linear_linear__`) para todas as operações. Os pull requests continuam no GitHub (`carreirodev/ArcaBackup`), operados pelo `gh`.

## Convenções

- **Criar um issue**: `save_issue` com `team: "WPC Solutions"`, `project: "ArcaBackup"`, `title` e `description`. A descrição é Markdown com quebras de linha reais, sem `\n` escapado. Sem `id`: com `id`, o `save_issue` atualiza em vez de criar.
- **Ler um issue**: `get_issue` com o identificador (`WPC-53`) e `includeRelations: true`, para ver bloqueios e relacionados. Os comentários vêm de `list_comments` com o mesmo `issueId`.
- **Listar issues**: `list_issues` com `project: "ArcaBackup"` e os filtros `state`, `label`, `assignee` e `parentId` apropriados.
- **Comentar em um issue**: `save_comment` com `issueId` e `body`.
- **Aplicar / remover labels**: `save_issue` com `id` e `addLabels` / `removeLabels`. Não use `labels` para isso, porque ele substitui o conjunto inteiro. Uma label que ainda não existe se cria antes com `save_issue_label`. As que existem vêm de `list_issue_labels`.
- **Fechar**: comente o porquê com `save_comment` e depois chame `save_issue` com `id` e o estado. Use `state: "Done"` quando foi feito, `"Canceled"` quando não será, e `"Duplicate"` junto de `duplicateOf` quando repete outro issue. Os nomes dos estados vêm de `list_issue_statuses`.

## Pull requests como superfície de triagem

**PRs as a request surface: no.** _(Defina como `yes` se este repositório tratar PRs externos como pedidos de feature; o `/triage` lê esse flag.)_

Quando definido como `yes`, os PRs passam pelas mesmas labels e estados dos issues, usando os equivalentes `gh pr`. As labels ficam no GitHub, e não no Linear, porque o Linear não guarda PRs.

- **Ler um PR**: `gh pr view <number> --comments` e `gh pr diff <number>` para o diff.
- **Listar PRs externos para triagem**: `gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments`, mantendo apenas `authorAssociation` igual a `CONTRIBUTOR`, `FIRST_TIME_CONTRIBUTOR` ou `NONE` (descartar `OWNER`/`MEMBER`/`COLLABORATOR`).
- **Comentar / rotular / fechar**: `gh pr comment`, `gh pr edit --add-label`/`--remove-label`, `gh pr close`.

Os números não se confundem: `WPC-42` é issue do Linear, e `#42` é PR do GitHub.

## Quando uma skill disser "publish to the issue tracker"

Crie um issue no Linear: `save_issue` com `team: "WPC Solutions"` e `project: "ArcaBackup"`.

## Quando uma skill disser "fetch the relevant ticket"

Chame `get_issue` com o identificador (`WPC-<n>`) e `includeRelations: true`, e `list_comments` com o mesmo `issueId`.

## Operações de wayfinding

Usadas pelo `/wayfinder`. O **mapa** é um único issue com issues **filhos** como tickets.

- **Mapa**: um único issue rotulado `wayfinder:map`, contendo o corpo com Notes / Decisions-so-far / Fog.
- **Ticket filho**: um sub-issue nativo do Linear, criado com `save_issue` e `parentId: "<mapa>"`. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Uma vez reivindicado, o ticket é atribuído ao dev responsável.
- **Bloqueio**: use a **relação nativa de bloqueio** do Linear, a representação canônica e visível na UI. Adicione uma aresta com `save_issue`, `id: "<filho>"` e `blockedBy: ["<bloqueador>"]`, que só acrescenta. Para tirar uma aresta, use `removeBlockedBy`. Um ticket está desbloqueado quando todos os bloqueadores estão com `statusType` `completed` ou `canceled`.
- **Consulta de fronteira**: liste os filhos do mapa (`list_issues` com `parentId: "<mapa>"`). Fique com os de `statusType` `backlog` ou `unstarted` e sem assignee. Descarte os que tiverem bloqueador aberto (`get_issue` com `includeRelations: true`). O primeiro na ordem do mapa vence.
- **Reivindicar**: `save_issue` com `id` e `assignee: "me"`, a primeira escrita da sessão.
- **Resolver**: em três passos.
  1. `save_comment` com a resposta.
  2. `save_issue` com `state: "Done"`.
  3. Anexe um ponteiro de contexto (gist + link) ao Decisions-so-far do mapa, com `save_issue` no mapa e um `patch` `insert_after` na seção. Não reenvie a descrição inteira.
