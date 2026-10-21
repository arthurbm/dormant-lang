# Issue tracker: GitHub

As issues e specs deste repositório ficam no GitHub Issues de `arthurbm/dormant-lang`. Use o `gh` CLI em todas as operações.

## Convenções

- **Criar uma issue**: `gh issue create --title "..." --body "..."`. Use um heredoc para corpos com várias linhas.
- **Ler uma issue**: `gh issue view <number> --json number,title,body,labels,comments`.
- **Listar issues**: `gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`, com os filtros `--label` e `--state` adequados.
- **Tornar uma issue sub-issue de outra**: `gh issue create --parent <parent> ...`, ou `gh issue edit <parent> --add-sub-issue <child>` depois (`gh` 2.94+). Com `gh` mais antigo: `gh api --method POST repos/<owner>/<repo>/issues/<parent>/sub_issues -F sub_issue_id=<child-db-id>` (database id, como em **Bloqueio** abaixo). Sem sub-issues, ponha `Part of #<parent>` no topo do corpo da filha.
- **Comentar numa issue**: `gh issue comment <number> --body "..."`
- **Aplicar / remover labels**: `gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **Fechar**: `gh issue close <number> --comment "..."`

O `gh` deduz o repositório do `git remote -v` quando roda dentro do clone.

## Pull requests como superfície de triage

**PRs como superfície de pedidos: não.** _(Troque para `sim` se este repositório passar a tratar PRs externos como pedidos de funcionalidade; o `/triage` lê esta flag.)_

Com a flag em `sim`, PRs passam pelas mesmas labels e estados das issues, usando os equivalentes `gh pr`:

- **Ler um PR**: `gh pr view <number> --comments` e `gh pr diff <number>` para o diff.
- **Listar PRs externos para triage**: `gh api --paginate 'repos/{owner}/{repo}/pulls?state=open' --jq '.[] | select(.author_association | IN("OWNER","MEMBER","COLLABORATOR") | not) | {number, title, author: .user.login, author_association, labels: [.labels[].name]}'`.
- **Comentar / aplicar labels / fechar**: `gh pr comment`, `gh pr edit --add-label`/`--remove-label`, `gh pr close`.

O GitHub usa uma única numeração para issues e PRs, então um `#42` solto pode ser qualquer um dos dois: resolva com `gh pr view 42` e, se falhar, `gh issue view 42`.

## Quando uma skill disser "publish to the issue tracker"

Crie uma issue no GitHub.

## Quando uma skill disser "fetch the relevant ticket"

Leia como em **Ler uma issue** acima.

## Operações de wayfinding

Usadas pelo `/wayfinder`. O **mapa** é uma única issue, e os tickets são issues **filhas** dele.

- **Mapa**: uma issue com a label `wayfinder:map`, cujo corpo traz Notes / Decisions-so-far / Fog. `gh issue create --label wayfinder:map`.
- **Ticket filho**: uma issue ligada ao mapa como sub-issue do GitHub (veja **Tornar uma issue sub-issue de outra**). Onde não houver sub-issues, adicione a filha a uma task list no corpo do mapa e ponha `Part of #<map>` no topo do corpo da filha. Labels: `wayfinder:<type>` (`research`/`prototype`/`grilling`/`task`). Ao ser reivindicado, o ticket é atribuído a quem conduz a sessão.
- **Bloqueio**: as **dependências nativas de issues** do GitHub, a representação canônica e visível na interface. Adicione uma aresta com `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`, em que `<blocker-db-id>` é o **database id** numérico da issue bloqueadora (`gh api repos/<owner>/<repo>/issues/<n> --jq .id`; _não_ é o `#number` nem o `node_id`). O GitHub informa `issue_dependencies_summary.blocked_by` (só bloqueadores abertos, que é o que vale). Onde não houver dependências, use uma linha `Blocked by: #<n>, #<n>` no topo do corpo da filha. Um ticket fica desbloqueado quando todos os bloqueadores estão fechados.
- **Consulta da fronteira**: liste as filhas abertas do mapa (`gh issue list --state open`, restrito às sub-issues / task list do mapa) e descarte as que têm bloqueador aberto (`issue_dependencies_summary.blocked_by > 0`, ou uma issue aberta na linha `Blocked by`) ou responsável atribuído; vence a primeira na ordem do mapa.
- **Reivindicar**: `gh issue edit <n> --add-assignee @me`, a primeira escrita da sessão.
- **Resolver**: `gh issue comment <n> --body "<answer>"`, depois `gh issue close <n>`, depois acrescente um ponteiro de contexto (resumo + link) em Decisions-so-far no mapa.
