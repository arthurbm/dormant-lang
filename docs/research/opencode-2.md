# OpenCode 2: durabilidade de sessões de agente e lições para o Dormant

Pesquisa sobre o repositório [anomalyco/opencode](https://github.com/anomalyco/opencode) no commit `a22754f761f868a80231859c4b8be8887f81187c` (versão 2.0.27).

Caminhos como `packages/core/src/bus.ts:400` apontam para esse commit. Para abrir no GitHub, use `https://github.com/anomalyco/opencode/blob/a22754f761f868a80231859c4b8be8887f81187c/<caminho>#L<linha>`. Itens tirados de memória ou de fonte secundária estão marcados como **(não verificado)**. As seções 4 e 5 são inferência, e o texto deixa isso explícito.

## 1. O que é o OpenCode 2 e o que mudou

O OpenCode é um agente de programação open source da Anomaly que roda como TUI, app desktop e app web ([docs v2](https://opencode.ai/v2/docs)). A 2.0 foi anunciada em [anoma.ly/notes/opencode-2-0](https://anoma.ly/notes/opencode-2-0/), com `published_time` 2026-10-12 no HTML da página. O `CHANGELOG.md` do commit inspecionado já registra a v2.0.26 em 2026-10-08 e a v2.0.27 em 2026-10-12 (`CHANGELOG.md:5`, `CHANGELOG.md:83`).

**Linguagem e runtime.** É um monorepo TypeScript sobre Bun (`"packageManager": "bun@1.4.2"`, `package.json:8`). O [Effect](https://effect.website) 4.0.1 é usado em todo o código, inclusive o driver `@effect/sql-sqlite-bun` (`package.json:49`, `package.json:82`). A TUI usa [OpenTUI](https://github.com/sst/opentui) com Solid (`package.json:56`, `packages/tui/package.json:61-65`). A TUI da 1.x teria sido escrita em Go no início **(não verificado)**.

**Serviço único, vários clientes.** Segundo o post da 2.0, o servidor "now automatically runs as a single background process on your machine", e a TUI, a web e o desktop se conectam a ele. A API do servidor "grew haphazardly" na v1 e foi reescrita: "We started from scratch". O guia de migração confirma: "OpenCode 2 has a revised, more ergonomic server API and a new set of clients" ([migrate-v1](https://opencode.ai/v2/docs/migrate-v1/)).

O contrato HTTP é um `HttpApi` do Effect. A partir dele, um gerador produz os clientes Promise e Effect publicados em `@opencode/client` (`packages/httpapi-codegen/README.md:9`). A direção das dependências é Schema → Core/Protocol → Server, e o cliente nunca depende de Core nem de Server (`AGENTS.md:2`).

**SDK.** O `@opencode/sdk` é "the exact same code that powers our CLI" e pode rodar em ambientes como Durable Objects ([post 2.0](https://anoma.ly/notes/opencode-2-0/)). O repositório tem um driver SQLite para workerd, o runtime do Cloudflare Workers (`packages/core/src/database/sqlite.workerd.ts`).

**Plugins e recarga a quente.** A 2.0 implementa "nearly every OpenCode 2.0 feature as an internal plugin" (são "80+"), e tudo recarrega a quente. Plugins da 1.0 não funcionam na 2.0 ([post 2.0](https://anoma.ly/notes/opencode-2-0/)).

O post [OpenCode Reloaded](https://anoma.ly/notes/opencode-reloaded) (2026-09-21) explica como a recarga fica determinística. Cada plugin "hands OpenCode a catalog transformation function". Na recarga, `ctx.catalog.reload()` "creates a new empty catalog" e reaplica as transformações em ordem. A comparação de alternativas que levou a esse desenho está em `specs/v2/catalog-config-plugin-lifecycle.md:18` (rejeitada) e `:191` (selecionada).

**Outras novidades** ([post 2.0](https://anoma.ly/notes/opencode-2-0/)):
- subagentes e comandos shell em background;
- o atalho Ctrl+B, que manda trabalho em andamento para o background;
- o Code Mode, ativado por padrão para servidores MCP (ver seção 3).

## 2. Persistência e durabilidade de sessões

Esta seção é o foco da pesquisa. A fonte principal é o spec `specs/v2/session.md`, que se declara "Current semantic overview" (`specs/v2/session.md:3`), conferido contra o código.

### 2.1 O modelo é event sourcing com projeções em SQLite

*Event sourcing* é o padrão em que a fonte da verdade é um log append-only de eventos. As tabelas que o resto do sistema consulta (*projeções*) são derivadas desse log. A divisão de responsabilidades está declarada: "Core owns event meaning, publication, persistence, typed observation, durable logs, replay, and transactional projection" (`specs/v2/event-stream-architecture.md:25`).

**Tabelas** (`packages/core/src/event/sql.ts:4-26`):
- `event_sequence(aggregate_id PK, seq, owner_id)` guarda o último `seq` de cada agregado. Um *agregado* é a unidade de consistência; para sessões, é o `sessionID`.
- `event(id PK, aggregate_id, seq, created, type, data json)` tem índice único `(aggregate_id, seq)` (`:23`).

**Versionamento.** Cada definição de evento durável declara `version` e `aggregate` (`packages/schema/src/event.ts:90`). O tipo é gravado como `"<type>.<version>"`, por exemplo `session.tool.success.2` (`versionedType`, `packages/schema/src/event.ts:174`). O registro de definições é indexado por esse tipo versionado (`durableMap`, `:178`).

**Commit atômico.** A publicação de um evento durável (`packages/core/src/bus.ts`) faz tudo dentro de uma única transação SQLite `BEGIN IMMEDIATE` (`{ behavior: "immediate" }`, `:433`):
1. calcula `seq = latest + 1` (`:371`);
2. roda os *projetores* registrados para o tipo versionado (`:400`);
3. roda um `commit(seq)` opcional, uma "local operational projection committed atomically with a new durable event" (`:97`);
4. grava o evento.

Um `KeyedMutex` por agregado serializa as escritas concorrentes (`:200`).

**Replay de eventos serializados.** É o que se usa em sync e importação (`function replay`, `:671`). A operação é idempotente: um evento já presente com o mesmo `seq` e conteúdo idêntico é ignorado. Conteúdo divergente aborta com `"Replay diverged at aggregate … sequence …"` (`:364`). Esse replay **não** é reexecução de código, como no Temporal: é a reaplicação de fatos já gravados.

**Leitura do log por clientes.** `sessions.log({ sessionID, after?, follow? })` lê os eventos duráveis depois de um `seq` exclusivo. Com `follow: true`, "subscribes before replay and emits one synchronization marker at the captured watermark before live durable events continue" (`specs/v2/session.md:108`). Deltas de streaming (texto, reasoning, input de tool) "are intentionally absent from replay" (`:110`). Ou seja, só os fatos finais são duráveis.

### 2.2 Entrada idempotente: o inbox

`Session.prompt(...)` publica o fato durável `session.inbox.enqueued`, cuja projeção insere uma linha em `session_inbox` antes de qualquer execução (`specs/v2/session.md:7`). Reusar o ID de um item é idempotente: "the first admission wins, and retried payload, metadata, and delivery are ignored" (`:9`).

O parâmetro `resume` controla o agendamento, não a durabilidade. Com `false`, a entrada é gravada sem acordar a execução (`:11-14`). Pedidos de compactação e de movimentação de sessão também entram pelo inbox, como itens de controle (`:23`).

### 2.3 Retomada após crash: write-ahead claim

O mecanismo central de recuperação é este:

> "Execution commits a write-ahead claim when a process-local busy period starts. Success, failure, and user interruption release the claim; shutdown interruption and unclean process death preserve it. On startup, managed Node and fetch runtimes resume claimed top-level Sessions, append a durable continuation instruction, and count recovery attempts. Recovery is bounded per claimed execution but does not guarantee exactly-once provider requests or tool effects." (`specs/v2/session.md:68`)

No código:
- A claim fica nas colunas `time_suspended` e `resume_attempts` da sessão, com um índice parcial para achar sessões com claim pendente (`packages/core/src/session/sql.ts:66-75`).
- O sweep de boot fica em `packages/core/src/session/execution/restart.ts`. O limite padrão é de 10 tentativas por turno (`:33`), e o contador é gravado "Durable before the resume runs, so a crash inside the resumed turn is counted by the next sweep" (`:78`). Ao estourar o limite, o turno é terminalizado com `Execution.Failed` (`:82`).
- A retomada injeta a mensagem sintética "The server restarted while you were working. Continue from where you left off without repeating completed work." (`:15`, `:94`).
- A semântica é declarada abertamente: "Recovery is at-least-once: local coordination prevents concurrent drains, not repeated external side effects after a crash" (`:52`).

### 2.4 Tool calls: durável antes do efeito, e órfãs falham em vez de reexecutar

- "Each complete local tool call is durable before side effects begin." (`specs/v2/session.md:74`)
- "At drain start, orphan reconciliation fails tool calls still projected as streaming or running from an earlier process before further model work. It preserves the original assistant attribution and never directly replays ambiguous side effects." (`:78`)
- Sobre o recovery em geral: "it cannot prove whether an interrupted external operation already took effect and does not guarantee exactly-once provider or tool behavior" (`:116`).

### 2.5 Nem replay de código nem snapshot de variáveis

O loop do agente não guarda estado em memória entre Steps: "After a local outcome, continuation reloads projected history and begins a new Step. The runner never delegates orchestration to an in-memory tool loop." (`specs/v2/session.md:80`). Antes de cada Step, o runner recarrega o histórico, resolve agente e modelo e materializa as tools (`:72`).

**Inferência:** o estado vivo de um agente de LLM é o próprio transcript, que já é dado serializável. Por isso o OpenCode não precisa de replay determinístico nem de snapshot de variáveis locais. Retomar é só reler a projeção e chamar o modelo de novo.

### 2.6 Subagentes, jobs em background e compactação

**Subagentes** são sessões filhas identificadas por `childSessionID`. O resultado é entregue à sessão pai como entrada sintética (`packages/core/src/session/subagent-job.ts:31-35`). Jobs em background recuperáveis guardam "a durable notification marker until its notification is admitted" (`packages/core/src/job.ts:171-177`). No restart, shells ainda em execução são reportados como cancelados (`packages/core/src/session/execution/restart.ts`, função `recoverShell`).

**Compactação.** "The full transcript remains durable." O histórico ativo passa a ser um resumo estruturado mais o contexto recente (`specs/v2/session.md:102`). A doc de [compaction](https://opencode.ai/v2/docs/compaction) acrescenta:
- "Earlier messages remain stored even when they are no longer sent to the model";
- por padrão são mantidos cerca de 15.000 tokens recentes (`keep.tokens`);
- compactações posteriores atualizam o mesmo resumo.

Há ainda um "native provider mode", em que o provedor devolve "an opaque, encrypted item that stands in for the older conversation".

### 2.7 Undo e revert de arquivos: git-sombra

`Snapshot.capture` grava o estado do diretório como uma árvore *content-addressed* (endereçada pelo hash do conteúdo), e `restore` restaura caminhos selecionados (`packages/core/src/snapshot.ts:49`, `:67`). O repositório-sombra fica em `<data>/snapshot/<projectID>/<hash(worktree)>` (`:96`).

A doc de [snapshots](https://opencode.ai/v2/docs/snapshots) diz que o OpenCode tira um snapshot antes e outro depois de cada passo do modelo, em modo best-effort, e que "OpenCode stores snapshots locally in a separate internal Git object database under its data directory". Os snapshots não desfazem efeitos de comandos shell.

### 2.8 Migrações de schema e de dados

**Migrações.** São 50 migrações em TypeScript no formato `{ id, up(tx) }` (`packages/core/src/database/migration.ts:14`, diretório `packages/core/src/database/migration/`). A tabela `migration(id, time_completed)` registra o que já rodou (`:57`), e cada migração pendente roda na sua própria transação (`:107`). Um banco vazio faz bootstrap direto do schema gerado e marca todas as migrações como aplicadas (`:22-36`).

**Conexão.** É `bun:sqlite` com `PRAGMA journal_mode = WAL` por padrão (`packages/core/src/database/sqlite.bun.ts:87-93`).

**Durante a beta, o histórico foi descartado em vez de migrado.** A migração `20260622170816_reset_v2_session_state.ts:8-12` faz `DELETE FROM event` e `event_sequence`, além das projeções.

**Dados da v1.** `packages/core/src/database/v1-migration.bun.ts` converte mensagens e partes da v1 para o formato da v2. Também avisa o modelo sobre tools da v1 cujos nomes ou argumentos mudaram (`:903-907`).

Não encontrei *upcasters*, isto é, funções que convertem eventos de uma versão antiga do schema para a atual na leitura (ver seção 6).

## 3. Técnicas de compilador e uso de Effect

**Code Mode: um interpretador de subconjunto de JS.** O pacote `packages/codemode` é "a lightweight, pure interpreter for a JavaScript-like language built around calling tools" (`packages/codemode/README.md:3`). A ideia de code mode é creditada à Cloudflare (`:11`). Como funciona:
- O *parser* é o [acorn](https://github.com/acornjs/acorn) 8.15.0 (`packages/codemode/package.json:28`), que transforma o código numa *AST* (árvore sintática abstrata).
- O interpretador é *tree-walking*: percorre a AST e avalia cada nó diretamente, sem gerar código intermediário. Cada avaliação é um `Effect.gen` (por exemplo `packages/codemode/src/interpreter/interpreter.ts:369`, e o `AwaitExpression` em `:1343`).
- Só as construções implementadas são aceitas (uma *allowlist*). O resto retorna o diagnóstico `UnsupportedSyntax` com localização no fonte (`README.md:16-24`). "TypeScript-only syntax is rejected rather than stripped before execution" (`interpreter-support.md:18`).
- O timeout é cooperativo, "fires between interpreter steps", e há limites de tamanho por built-in (`interpreter-support.md:36`, `src/interpreter/limits.ts`).
- O código **não é durável**: "Unfinished work is interrupted" (`README.md:18`).

**Codegen de clientes.** `packages/httpapi-codegen` faz "Reflect one authoritative `HttpApi` into a shared contract with `compile(Api)`" e emite clientes Promise e Effect. O código gerado é commitado, e "CI regenerates and fails when the worktree changes" (`packages/httpapi-codegen/README.md:9`, `:29`).

**Lint estrutural.** O lint usa regras [ast-grep](https://ast-grep.github.io), que fazem busca por padrões na AST, para impor padrões de Effect (`package.json`, scripts `lint:effect-patterns` e `lint:effect-simplifications`).

**Padrões de Effect usados:**
- `Context.Service` e `Layer` para injeção de dependências;
- `Scope` para o tempo de vida de recursos, por exemplo para fechar o SQLite no finalizer (`sqlite.bun.ts:92`);
- Effect Schema para eventos, com a versão codificada como literal no envelope (`packages/schema/src/event.ts:90-120`);
- `KeyedMutex` por agregado.

## 4. O que aproveitar no Dormant (inferência, em ordem de utilidade)

**Runtime (SQLite no Bun)**
1. **Schema do log.** Copiar o par `event(id, aggregate_id, seq, type.version, data)` + `event_sequence`, com índice único `(aggregate_id, seq)`. No Dormant, o agregado é o ID da execução do Workflow. Evento e projeção devem ser gravados na mesma transação `BEGIN IMMEDIATE`, com WAL ligado (seções 2.1 e 2.8).
2. **Write-ahead claim com contador de retomadas durável.** É assim que o runtime descobre execuções órfãs após um crash sem cooperação do processo morto. O contador gravado antes de retomar evita crash-loop (seção 2.3).
3. **"Durável antes do efeito".** Gravar `step.started` antes de executar o Step, e definir explicitamente o que acontece com um Step que tem `started` mas não tem `completed`: falhar, como o OpenCode, ou reexecutar em modo at-least-once (seção 2.4).
4. **Migrações `{ id, up(tx) }`** com tabela `migration` e uma transação por migração. É simples e suficiente para a v1 (seção 2.8).
5. **Idempotência por ID de entrada** ("first admission wins"), útil para iniciar Workflows e entregar sinais (seção 2.2).

**Interpretador com replay (Entrega 3)**
1. **Tree-walker sobre Effect.** O Code Mode mostra que um interpretador desse tipo é viável e testável. A fronteira com o host, onde ficam as chamadas de tool, é o ponto onde o interpretador do Dormant consultaria o log de eventos para decidir entre reexecutar um Step ou devolver o resultado gravado (seção 3).
2. **Allowlist com diagnóstico `UnsupportedSyntax` e localização.** É um bom modelo para rejeitar o que o subconjunto do Dormant não cobre.
3. **Determinismo por transformações reaplicadas** (post Reloaded). É uma analogia didática para explicar replay: o estado é sempre derivado de uma lista ordenada de funções aplicadas a um ponto inicial vazio.
4. **Detecção de divergência.** O `"Replay diverged at … sequence …"` do OpenCode compara fatos; o Dormant pode usar a mesma ideia para detectar não determinismo, comparando o Step esperado com o gravado na mesma posição do log.

**Compilador com snapshot (Entrega 4)**
1. **Contraste (seção 2.5).** O OpenCode não precisa de análise de *liveness* (descobrir quais variáveis ainda serão lidas mais adiante) porque o estado do agente já é o transcript. O Dormant ataca o caso em que o estado vive em variáveis locais de código arbitrário.
2. **Contraexemplo de versionamento.** O reset da beta e a ausência de upcasters visíveis reforçam por que o snapshot do Dormant deve ter `version` e `migrate` desde o início (seção 2.8).
3. Pouco além disso: o OpenCode não gera máquinas de estados.

**Testes**
1. O Code Mode roda arquivos do [test262](https://github.com/tc39/test262), a suíte oficial de conformidade do ECMAScript, "verbatim", com uma lista de arquivos pulados (`packages/codemode/interpreter-support.md:12`). Um subconjunto do test262 restrito à sintaxe aceita pelo Dormant serviria de bateria para o parser e o interpretador.
2. `packages/http-recorder` grava e reproduz HTTP em cassetes JSON determinísticos (`packages/http-recorder/README.md`). É útil se os exemplos do Dormant chamarem APIs reais.
3. Os testes do commit atômico listados em `specs/v2/event-stream-architecture.md` (seção "Verification") servem de checklist de casos de borda.

**Posicionamento e trabalhos relacionados**
- O OpenCode 2 é um exemplo de durabilidade escrita à mão para agentes: event sourcing, recuperação at-least-once e estado explícito. Isso posiciona o Dormant como complementar. O Dormant cobre o código que orquestra agentes e etapas, cujo estado não está pronto num transcript.

## 5. O OpenCode como consumidor do Dormant (inferência)

- **Workflows Dormant como plugin dentro do OpenCode: pouco provável.** O OpenCode já tem durabilidade própria, plugins em Effect/TS recarregáveis a quente e um Code Mode não durável.
- **A direção inversa é mais plausível e serve de boa demo.** Um Workflow Dormant orquestraria sessões do OpenCode via `@opencode/client`:
  - cada Step faz um `prompt` com ID fixo, idempotente pelo inbox (seção 2.2);
  - o Workflow aguarda o término via `sessions.log({ after })`.
  
  Assim os Steps ficam seguros para replay sem trabalho extra.
- Um caminho intermediário seria um plugin que dispara e acompanha execuções do runtime do Dormant. Não avaliei a API de plugins da v2 a fundo.

## 6. Itens não verificados

- **Data do GA.** A data exata em que a 2.0 virou estável. No commit inspecionado, as tags v2 não aparecem nos GitHub Releases do repositório: a mais recente é a v1.19.0, de 2026-10-11. As únicas evidências são o post e o CHANGELOG.
- **TUI da 1.x em Go:** a afirmação de que a TUI da 1.x era originalmente em Go (memória).
- **Eventos antigos.** Não encontrei upcasters nem leitura de eventos com versão anterior. Uma busca por "upcast" no Core e no Schema não retornou nada, mas posso ter deixado algo passar.
- **Versão da doc de snapshots.** A página de [snapshots](https://opencode.ai/v2/docs/snapshots) não diz explicitamente a que versão se refere, embora esteja sob `/v2/`.
- **Tabelas de sessão.** Não li a lista completa: as definições de `session_message`, `session_inbox` etc. estão espalhadas em `packages/core/src/session/sql.ts` e não foram inventariadas uma a uma.
- **Runtime.** Nada foi compilado nem executado. Todas as afirmações sobre comportamento vêm de specs, docs e leitura de código.
