# Vercel Workflow SDK: Worlds, arquitetura de runtime e lições para o Dormant

Pesquisa sobre o repositório [vercel/workflow](https://github.com/vercel/workflow) no commit `02993a26cbabec8f00df1f8a3a758158d095008a` (pacote `workflow` 5.2.1; `@workflow/world` 5.0.4; `@workflow/swc-plugin` 5.0.1).

Caminhos como `packages/core/src/workflow.ts:441` apontam para esse commit. Para abrir no GitHub, use `https://github.com/vercel/workflow/blob/02993a26cbabec8f00df1f8a3a758158d095008a/<caminho>#L<linha>`. A documentação do site [workflow-sdk.dev](https://workflow-sdk.dev/docs) é gerada a partir do próprio repositório: `docs/content/docs/v5/<x>.mdx` corresponde a `https://workflow-sdk.dev/docs/<x>` e `docs/content/worlds/v5/<x>.mdx` a `https://workflow-sdk.dev/worlds/<x>`. Esse mapeamento foi inferido da estrutura de rotas e conferido só na página inicial **(não verificado página a página)**. Por isso cito o `.mdx` com número de linha. Itens tirados de memória ou de fonte secundária estão marcados como **(não verificado)**. A seção 6 é inferência, e o texto deixa isso explícito.

## 1. Contexto: o Workflow SDK e o lugar dos Worlds

O Workflow SDK é a biblioteca de workflows duráveis da Vercel. O programador marca funções com as diretivas `"use workflow"` (orquestrador) e `"use step"` (unidade de trabalho com retry), e um compilador reescreve esse código (`docs/content/docs/v5/how-it-works/code-transform.mdx:16-44`). O modelo é o mesmo do Temporal: a própria doc diz que os dois "share the same core model of durable orchestration through event-sourced replay", com `"use workflow"` ≈ Workflow e `"use step"` ≈ Activity (`docs/content/docs/v5/comparisons/workflow-sdk-vs-temporal.mdx:15`, `:26`).

A v5.0.0 saiu em 2026-10-01 e a v5.2.1 em 2026-10-12, segundo os [GitHub Releases](https://github.com/vercel/workflow/releases) do repositório (tags `workflow@5.0.0` e `workflow@5.2.1`). A v4 continua instalável como `workflow@4` (`docs/content/docs/v5/whats-new.mdx`, callout inicial).

**O que é um World.** É "the abstraction that allows workflows to run on any infrastructure. It handles workflow storage, step execution queuing, and data streaming" (`docs/content/worlds/v5/building-a-world.mdx:16`). No código, a interface tem o comentário "The 'World' interface represents how Workflows are able to communicate with the outside world" (`packages/world/src/interfaces.ts:738-740`). É isso que permite rodar o mesmo código localmente, na Vercel ou em qualquer infraestrutura (`docs/content/docs/v5/deploying.mdx:13`).

## 2. A interface World

### 2.1 Responsabilidades

A interface junta três responsabilidades (`docs/content/worlds/v5/building-a-world.mdx:32-36`):

1. **Storage:** persiste runs, steps, hooks e o log de eventos.
2. **Queue:** enfileira e entrega invocações de workflow e de step.
3. **Streamer:** gerencia streams de dados em tempo real entre workflows e clientes.

Em TypeScript, é `interface World extends Queue, Streamer, Storage` (`packages/world/src/interfaces.ts:740`), com membros extras (`building-a-world.mdx:40-61`):

- `specVersion` (obrigatório). É a versão do protocolo; o runtime recusa um World fora da faixa que entende (`:81-97`; `interfaces.ts:762`). A versão atual é 8 (`SPEC_VERSION_CURRENT = SPEC_VERSION_SUPPORTS_HOOK_FORCE_CLAIM`, `packages/world/src/spec-version.ts:94-123`).
- `capabilities`: flags opcionais que "fail closed". Se a flag está ausente, o runtime mantém o comportamento conservador (`building-a-world.mdx:66`). Uma delas é `deploymentAffinity`, que significa "deployments are atomic and immutable" e só a Vercel declara (`interfaces.ts:707-720`; `packages/world-vercel/src/index.ts:71`).
- Opcionais: `start()` e `close()` (ciclo de vida, por exemplo polling da fila), `getEncryptionKeyForRun()`, `createRunId()`, `analytics` (só leitura de metadados, implementada apenas pela Vercel, `:530`), `resolveLatestDeploymentId()` (só Vercel, `interfaces.ts:801-811`).

Não existe um método "timer" ou "sleep" no World. O `sleep()` é implementado com o `delaySeconds` da fila: "Waits (`sleep()`) are delivered as ordinary delayed continuations on the workflow queue, so a World that ignores `delaySeconds` redelivers immediately and causes every sleeping run to enter a busy loop" (`building-a-world.mdx:443`). Hooks também não têm interface própria: são entidades do Storage alimentadas por eventos (seção 2.2).

### 2.2 Storage: log de eventos append-only com entidades materializadas

"Workflow storage uses an **append-only event log**. All state changes happen through events" (`building-a-world.mdx:101`). *Event sourcing* é o padrão em que a fonte da verdade é esse log; as tabelas de runs, steps etc. são *projeções* derivadas dele (`docs/content/docs/v5/how-it-works/event-sourcing.mdx:16-34`).

**Superfície** (`building-a-world.mdx:111-149`; `interfaces.ts:185`):
- `runs.get/list`, mais o opcional `waitForTerminalStatus` (long poll);
- `steps.get/list`;
- `events.create`, o opcional `events.createBatch`, `events.list` e `events.listByCorrelationId`;
- `hooks.get/getByToken/list`;
- o opcional e experimental `experimental_snapshots` (seção 3.6).

**Tipos de evento** (`event-sourcing.mdx:166-204`):
- run: `run_created`, `run_started`, `run_completed`, `run_failed`, `run_cancelled`;
- step: `step_created`, `step_started`, `step_completed`, `step_failed` e o opcional `step_retrying`;
- hook: `hook_created`, `hook_conflict`, `hook_received`, `hook_disposed`;
- wait: `wait_created`, `wait_completed`;
- sistema: `noop`.

Cada evento tem um `correlationId` igual ao ID da entidade (step, hook ou wait) (`:157-162`). Os diagramas de ciclo de vida de cada entidade e os estados terminais estão em `:44-152`. Sair de um estado terminal é erro (`:236`).

**Contrato de escrita.** `events.create()` precisa (`building-a-world.mdx:154-158`):
1. persistir o evento;
2. atualizar atomicamente a entidade afetada;
3. devolver as duas coisas.

Outras regras:
- Tokens de hook são únicos. Um conflito vira um evento `hook_conflict` com o `conflictingRunId` (`:170`).
- O World rejeita `step_started` em run terminal. Essa é a checagem de liveness do caminho de step enfileirado (`:176-183`).

**IDs de evento são posições.** Esta é a exigência mais forte do contrato: "Your World assigns every event ID, and every ID must be a position in its run's log [...] A World whose IDs use another format, such as a ULID, UUID, or database sequence shared across runs, cannot replay a workflow" (`building-a-world.mdx:213-215`). O formato é `evnt_` mais a posição com zeros à esquerda (`:217`). As posições precisam ser:
- únicas, resolvidas por constraint `(runId, eventId)` e não por "ler o máximo e somar um" no processo;
- densas, sem buracos, porque é pelo comprimento que o leitor distingue um log completo de um truncado (`:219-224`).

Um buraco faz a run falhar com `CORRUPTED_EVENT_LOG` (`event-sourcing.mdx:373`). Na spec 7 existe a alternativa de pré-alocar posições e selar as abandonadas com `noop` (`building-a-world.mdx:228-232`; `event-sourcing.mdx:305-315`). O World Postgres aloca no commit, com `MAX(slot) + 1` dentro do próprio INSERT (`packages/world-postgres/src/drizzle/schema.ts:198-208`).

**Detalhes que só fazem sentido com replay:**
- `resolveData: 'skip-step-inputs'`: o World pode omitir o `input` dos steps no `events.list`, porque o replay recalcula os argumentos reexecutando o código do workflow (`building-a-world.mdx:166`; `docs/content/docs/v5/whats-new.mdx:47`).
- `eventCount` e `sinceCursor`: o escritor diz quanto do log já carregou, e o World devolve os eventos que ele não viu para que o replay os incorpore (`building-a-world.mdx:234-236`; `event-sourcing.mdx:276-301`).
- O *in-band writer fence*, opcional, garante um único escritor orquestrador por run (`building-a-world.mdx:257-278`).

**Esquema concreto no Postgres** (`packages/world-postgres/src/drizzle/schema.ts:145-193`): `workflow_events(id, type, correlation_id, created_at, run_id, payload_cbor, spec_version, resume_id, …)`.
- A chave primária é `(run_id, id)`.
- Um índice único parcial `(run_id, correlation_id, type)` vale para `step_created`, `hook_created`, `wait_created` e `attr_set`. Ele deduplica escritas concorrentes que geram o mesmo `correlationId` determinístico (`:181-192`).
- Outro índice único `(run_id, resume_id)` vale para `hook_received` (`:176-180`).

As demais tabelas são runs, steps, hooks, waits, invocations, `workflow_snapshots` e streams (`:66`, `:221`, `:264`, `:299`, `:324`, `:357`, `:364`).

### 2.3 Queue

Superfície (`building-a-world.mdx:286-317`):
- `getDeploymentId()`;
- `queue(queueName, message, opts)`;
- `queueBatch()` e `invoke()`, opcionais;
- `createQueueHandler(prefix, handler)`, que devolve um handler `(req: Request) => Promise<Response>`.

Os opcionais `registerRoute` e `registerHandler` decidem para onde vão mensagens de outros prefixos (`:326-342`).

**Uma fila só, um prefixo só.** O nome é `__wkf_workflow_<name>` e serve "for workflow orchestration and queued step invocations" (`:406`). O payload é o mesmo para os dois casos: `{ runId, stepId?, stepName?, runContext?, traceCarrier?, requestedAt? }`. A presença de `stepId` marca uma invocação de step (`:410-422`).

**Requisitos** (`:439-444`):
- entrega at-least-once;
- políticas de retry configuráveis;
- contagem de tentativas;
- idempotência por `idempotencyKey`;
- respeito a `delaySeconds`.

Com `invoke`, o World deve garantir "at most one active workflow runner per `runId` across all worker processes" (`:354`).

**Namespaces para outros runtimes.** "A library that keeps its own runs in a World and consumes its own messages, such as a durable agent harness, uses a namespace of its own (`__agent_wkf_workflow_session`) so that its messages never reach the Workflow SDK's handler" (`:324`). Ou seja, o contrato já prevê um runtime de terceiros usando o mesmo World. Isso é central para a seção 6.

### 2.4 Streamer

Streams são identificados por `(runId, name)` e oferecem `write`, `writeMulti?`, `close`, `get` (um `ReadableStream` ao vivo a partir de um índice), `list`, `getChunks` (snapshot paginado) e `getInfo` (`building-a-world.mdx:452-506`). Existe também uma sessão de escrita com estado, opcional (`:509-522`).

### 2.5 Implementações

O manifesto `worlds-manifest.json` lista as implementações oficiais e as da comunidade.

| World | Pacote | Armazenamento / fila | Observações |
|---|---|---|---|
| Local | `@workflow/world-local` | arquivos JSON em `.workflow-data/`; fila em memória que faz POST HTTP para o próprio app | padrão fora da Vercel; não persiste a fila entre restarts, instância única, sem autenticação (`docs/content/worlds/v5/local.mdx:13`, `:150-155`); reenfileira runs ativas ao iniciar (`docs/content/docs/v5/configuration/worlds.mdx:103-107`) |
| Postgres | `@workflow/world-postgres` | PostgreSQL + [graphile-worker](https://github.com/graphile/worker); `LISTEN/NOTIFY` para streams | "reference implementation [...] not optimized for scale, speed, or security"; exige processo de longa duração, não serve para serverless (`docs/content/worlds/v5/postgres.mdx:13-15`, `:213`, `:349-351`, `:405-412`); fila nomeada por run, `workflow_flows:<runId>:executor`, com um job ativo por vez (`:87-93`) |
| Vercel | `@workflow/world-vercel` | backend gerenciado da Vercel | único com `analytics`, criptografia e `deploymentAffinity` (`building-a-world.mdx:530`; `worlds-manifest.json`, `"features": ["encryption"]`) |
| Sim | `@workflow/world-sim` | em memória, determinístico | serve para encenar corridas ("What happens if the approval webhook arrives *after* `step_started` is durable but *before* the workflow gets control back?") e checar o contrato (`packages/world-sim/README.md:1-12`) |
| Comunidade | Turso/libSQL, MongoDB, Redis (BullMQ), SurrealDB, Jazz, Cloudflare Durable Objects, MySQL, Azure, NATS JetStream, Upstash, Platformatic, Eveland | variados | listados em `worlds-manifest.json` (por exemplo `:62`, `:78`, `:106`, `:245`, `:328`); **não inspecionados** |

### 2.6 Seleção e configuração

- A variável `WORKFLOW_TARGET_WORLD` escolhe o World **em tempo de execução**, não de build. O padrão é `local` fora da Vercel. Na Vercel, a presença de `VERCEL_DEPLOYMENT_ID` seleciona o World Vercel automaticamente (`docs/content/docs/v5/configuration/worlds.mdx:16-24`).
- Valores aceitos: `local`, `@workflow/world-postgres`, um caminho como `./my-world.ts` (que exporte um World, um `createWorld()` ou uma fábrica default) ou qualquer pacote (`:34-37`, `:43-54`).
- No código, `createWorld()` trata `vercel` e `local` como casos especiais. Para qualquer outro valor, tenta `require()` e cai para `import()` dinâmico (`packages/core/src/runtime/world.ts:141-200`).

### 2.7 Como escrever um World

1. Implementar a interface e declarar `specVersion: SPEC_VERSION_CURRENT`, em vez de um número literal, para acompanhar as versões do runtime (`building-a-world.mdx:81-97`).
2. Guardar estado de processo em `globalThis` sob um `Symbol.for()` versionado, ou na instância do World. A razão: bundlers podem duplicar o módulo em várias camadas, como as camadas do Next (`:582-635`).
3. Estudar o World Local e o Postgres como referência (`:641-642`).
4. Publicar no npm e abrir um PR adicionando o pacote ao `worlds-manifest.json`. A CI roda a suíte E2E de `@workflow/world-testing` contra ele (`:646-659`).

## 3. Arquitetura ponta a ponta

### 3.1 Tempo de compilação: o plugin SWC

O [SWC](https://swc.rs) é um compilador de JS/TS escrito em Rust. O `@workflow/swc-plugin` é um plugin dele compilado para WebAssembly (alvo `wasm32-unknown-unknown`, `packages/swc-plugin-workflow/README.md`). O plugin percorre a *AST* (árvore sintática abstrata) e tem três modos (`packages/swc-plugin-workflow/transform/src/lib.rs:476-483`; `code-transform.mdx:48-69`).

**Step mode.** Gera o módulo de registro de steps e é também a transformação aplicada ao código da aplicação (`code-transform.mdx:81-128`):
- remove a diretiva e mantém o corpo do step intacto;
- registra o step num `Map` global por meio de uma *IIFE* (função anônima invocada imediatamente): `__wf_reg.set("step//workflows/user.js//createUser", createUser)`;
- substitui o corpo de cada workflow por um `throw` e anexa `workflowId`, para que só `start()` consiga iniciá-lo;
- roda um passo de *DCE* (dead-code elimination, a remoção de código inalcançável) sobre o que só era usado pelos corpos de workflow removidos.

**O antigo modo "client" foi fundido no step mode na 5.0:** "Build integrations that passed `mode: "client"` now pass `mode: "step"`" (`code-transform.mdx:72`). O enum do plugin só tem `Step`, `Workflow` e `Detect`, com `deny_unknown_fields` (`lib.rs:474-483`).

**Workflow mode.** Gera o bundle executado pela rota `flow` (`code-transform.mdx:134-177`):
- **o corpo do workflow fica intacto**;
- o corpo de cada step vira `return globalThis[Symbol.for("WORKFLOW_USE_STEP")]("step//…//createUser")(email)`, um proxy que consulta o log e devolve o resultado gravado ou suspende.

**Detect mode.** Não transforma nada. Só percorre a AST e emite um manifesto JSON num comentário `/**__internal_workflows{…}*/`. O build primeiro faz um pré-filtro por expressão regular e depois usa este modo para descartar falsos positivos, como a diretiva dentro de um template literal (`code-transform.mdx:185-216`).

**IDs.** O formato é `{type}//{modulePath}//{identifier}`, com o caminho relativo ou, opcionalmente, um especificador de pacote com versão (`packages/swc-plugin-workflow/spec.md:33-40`). Os IDs mudam se o arquivo ou a função for renomeado (`code-transform.mdx:327-345`).

**Variáveis de closure.** Um step aninhado dentro de um workflow que lê variáveis externas é içado (*hoisted*) para fora. No workflow mode, o proxy recebe uma função que captura essas variáveis no momento da chamada: `globalThis[Symbol.for("WORKFLOW_USE_STEP")]("step//./input//myWorkflow/increment", () => ({ count }))` (`spec.md:536-569`). A detecção é uma análise de *variáveis livres*: coleta identificadores que não são parâmetros, declarações locais, globais conhecidos nem declarações de módulo (`spec.md:358-395`).

**Validações** feitas pelo plugin: workflow precisa ser `async`; só métodos estáticos podem ser workflow; diretiva fora do lugar; diretivas conflitantes; erro de digitação como `"use steps"`; e outras (`spec.md:949-965`). O build também rejeita módulos Node como `fs` e `http` no bundle de workflow (`code-transform.mdx:244-249`).

**Arquivos gerados** (`code-transform.mdx:228-288`; `docs/content/docs/v5/how-it-works/framework-integrations.mdx:61-65`):
- `flow.js`: "All workflow code is bundled together and embedded as a string inside `flow.js`". É executado numa **VM do Node** (`node:vm`) a cada requisição (`code-transform.mdx:236-240`).
- `__step_registrations.js`: importado pelo `flow.js` por efeito colateral. Não é rota HTTP. Na v4, havia `POST /.well-known/workflow/v1/step` e uma fila `__wkf_step_*`; a v5 juntou tudo no handler combinado (`:262-275`).
- `webhook.js`: rota `/.well-known/workflow/v1/webhook/:token` (`:278-288`).

### 3.2 Tempo de execução

**Início.** `start(workflow, args)` grava `run_created` e enfileira uma mensagem no prefixo `__wkf_workflow_` (`event-sourcing.mdx:287`; `building-a-world.mdx:406`). O World entrega a mensagem em `POST /.well-known/workflow/v1/flow`, a única rota consumidora (`framework-integrations.mdx:146-155`).

**O laço do handler.** `workflowEntrypoint()` "handles workflow execution requests, executing steps inline when possible [...] The handler loops: replay workflow → execute step inline → replay → ... until the workflow completes, times out, or encounters non-step suspensions" (`packages/core/src/runtime.ts:803-813`). Na prática, a v5 cria e executa vários steps por suspensão, em paralelo, e "only uses the queue for a wait, a hook, or when the function approaches its timeout" (`whats-new.mdx:41`). A VM é mantida viva entre steps inline, então cada volta do laço só acrescenta os eventos novos (`whats-new.mdx:45`).

**Replay** ("reexecução"). A cada invocação, a função de workflow roda desde o início dentro da VM. O proxy `useStep`:
1. gera um `correlationId = step_<ULID>` (`packages/core/src/step.ts:28`);
2. assina o consumidor de eventos;
3. se encontra o evento terminal com aquele `correlationId`, resolve a promise com o resultado gravado;
4. se o log acaba antes, agenda a suspensão e deixa a promise pendente, "so that the user workflow code does not proceed any further" (`step.ts:53-62`).

Um nome de step diferente do gravado na mesma correlação gera `ReplayDivergenceError` (`step.ts:87-97`).

**Determinismo dentro da VM** (`packages/core/src/vm/index.ts:52-200`):
- `Math.random` vem de `seedrandom(seed)` (`:55`, `:61`). A seed é `${runId}:${workflowName}:${deploymentId}` (`packages/core/src/workflow.ts:441-444`).
- `Date` e `Date.now()` devolvem um relógio fixo (`vm/index.ts:66-83`). O relógio começa no instante de criação da run (`workflow.ts:416-417`) e só avança quando uma entrega que decide um ramo chega ao workflow (resultado de step, payload de hook, wait completado), nunca quando o evento é apenas lido (`workflow.ts:508-523`).
- `crypto.getRandomValues` e `randomUUID` usam o mesmo PRNG (gerador pseudoaleatório). `crypto.subtle.digest` é síncrono, e os demais métodos de `subtle` lançam erro.
- `WeakRef`, `FinalizationRegistry`, `Atomics.waitAsync` e as entradas assíncronas de `WebAssembly` são removidos (`vm/index.ts:98-120`).

**IDs por invocação.** O `correlationId` de cada step vem de uma fábrica ULID monotônica alimentada pelo PRNG semeado (`workflow.ts:479-493`). Por isso, a identidade de uma chamada de step no log é determinada **pela ordem das chamadas no replay**, e não por algo fixado no fonte. Os IDs de compilação (`step//arquivo//nome`) identificam a função; os `step_<ULID>` identificam a invocação.

**Execução de step.** Steps rodam no runtime hospedeiro (Node, Bun ou Deno), sem sandbox (`code-transform.mdx:122`). Inline, o executor grava `step_started`, roda o corpo e grava `step_completed` ou `step_failed`. Se o step vai para a fila, a mensagem leva `stepId` e `stepName`, e o mesmo handler `flow` executa o step e depois retoma o replay (`framework-integrations.mdx:45`). Argumentos são passados por valor (serializados), não por referência (`docs/content/docs/v5/foundations/workflows-and-steps.mdx:86`).

**Retries.**
- Padrão: 3 retries (4 tentativas), configurável pela propriedade `fn.maxRetries` (`docs/content/docs/v5/foundations/errors-and-retries.mdx:17`, `:33`, `:137-139`; `packages/core/src/runtime/step-executor.ts:74`).
- `FatalError` pula os retries; `RetryableError` aceita `retryAfter` como duração, milissegundos ou `Date` (`errors-and-retries.mdx:47-127`). Um erro de serialização falha como `FatalError` (`:144`).
- O evento `step_retrying` é opcional. Um crash durante o step aparece como vários `step_started` seguidos (`event-sourcing.mdx:95`, `:182`; `docs/content/docs/v5/errors/step-executed-multiple-times.mdx:13`). O corpo do step é, portanto, at-least-once.

**Sleep.** Grava `wait_created` com o horário de despertar, e o runtime enfileira uma continuação com `delaySeconds` (`event-sourcing.mdx:136-152`; `packages/core/src/runtime/wait-continuation.ts:7`, `:108-139`). Ao acordar, um replay grava `wait_completed`, e a transição atômica garante que um wait só complete uma vez (`event-sourcing.mdx:152`).

**Hooks e webhooks.**
- `createHook()` só registra o hook quando o workflow suspende (`docs/content/docs/v5/foundations/hooks.mdx:115`).
- De fora, `resumeHook(token, payload)` grava `hook_received` e despacha a mensagem de fila **concorrentemente**, com uma constraint `(runId, resumeId)` que faz os dois escritores convergirem para um único evento (`whats-new.mdx:49`; índice em `schema.ts:176-180`).
- Webhooks são hooks com tratamento HTTP automático, servidos pela rota `webhook/:token` (`event-sourcing.mdx:105`).

**Duplicatas e divergência.** Invocações concorrentes da mesma run compartilham o log e podem gravar `step_created` duplicados. O replay agrupa os tipos em **classes** (por exemplo, `step_completed` e `step_failed` formam a classe `step_terminal`), e o primeiro evento de cada classe vence (`event-sourcing.mdx:242-260`). Uma divergência real dispara outro replay. Esgotado o orçamento, a run falha com `CORRUPTED_EVENT_LOG` (`docs/content/docs/v5/errors/replay-divergence.mdx:21-23`).

### 3.3 Quem faz o quê

| Camada | Responsabilidade | Onde |
|---|---|---|
| Compilação (plugin SWC + builders) | separar workflow/step, IDs estáveis, captura de closure, DCE, validação, manifesto, bundles `flow`/steps/webhook | `packages/swc-plugin-workflow`, `packages/builders` |
| Runtime central (`@workflow/core`) | VM determinística, replay, consumo de eventos, suspensão, execução inline de step, retries, serialização, criptografia, guarda de deployment, snapshots QuickJS | `packages/core/src` |
| World | log de eventos e entidades, alocação de posições, unicidade de tokens, fila com atraso e at-least-once, streams, chaves de criptografia, armazenamento de snapshot, ID de deployment | `packages/world*` |
| Integração de framework | aplicar o step mode ao código da app, gerar e expor as rotas | `packages/next`, `vite`, `nitro` etc. |

### 3.4 Workflows dinâmicos

Existe um modo experimental em que `start()` recebe uma **string** de código de workflow, sem build. Só a orquestração é dinâmica: "There is no way to define a new step from source" (`docs/content/docs/v5/advanced/dynamic-workflows.mdx:25`, `:53`). O World precisa da capability `dynamicWorkflowCode` para guardar o código junto com a run (`interfaces.ts:722-734`).

### 3.5 Dois motores de VM

`WORKFLOW_VM=node|quickjs`. O padrão é `node:vm`. O QuickJS, compilado para WebAssembly via `quickjs-wasi`, é experimental e serve para plataformas sem `node:vm`, como Cloudflare Workers. Os dois motores implementam o mesmo modelo de replay (`packages/core/src/runtime/vm-mode.ts:1-14`; `docs/content/docs/v5/configuration/runtime-tuning.mdx:217-229`). O motor escolhido é gravado no `executionContext` da run (`runtime-tuning.mdx:229`).

### 3.6 Snapshot de memória da VM (QuickJS, experimental)

Este é o achado mais relevante para a tese do Dormant. Com o motor QuickJS, depois de `WORKFLOW_SNAPSHOT_THRESHOLD` eventos (padrão 1000), o runtime grava um **snapshot da memória da VM** (o heap inteiro) numa suspensão. As invocações seguintes restauram a VM e fazem replay só dos eventos posteriores (`vm-mode.ts:79-98`; `runtime-tuning.mdx:240-246`).

- O World guarda o snapshot pelo opcional `experimental_snapshots.save/load/delete` (`packages/world/src/interfaces.ts:500-560`). Local, Postgres e Vercel implementam (`runtime-tuning.mdx:244`; `packages/world-postgres/src/index.ts:29`).
- **O log continua sendo a fonte da verdade:** "Snapshots are an optimization, not a source of truth [...] a missing, corrupt, oversized, or incompatible snapshot automatically falls back to a full replay" (`runtime-tuning.mdx:248`). Snapshots acima de 32 MB não são salvos (mesma linha).
- **Metadados** (`packages/world/src/snapshots.ts:12-84`):
  - `eventsCursor`: posição no log;
  - `eventCount`;
  - `rngDraws`: quantos sorteios do PRNG já foram feitos, para avançar a seed;
  - `lastUlid`;
  - `clockMs`;
  - `engineVersion`: build exato do QuickJS;
  - `formatVersion` (hoje 3, `:102`).
- Metadados e bytes vão num único envelope (`"WSNP"` + versão + JSON + dados) gravado atomicamente, para evitar parear bytes de uma suspensão com o cursor de outra (`snapshots.ts:107-150`).
- **Um snapshot é estado executável:** "the workflow's code and in-memory state, including step results and hook payloads it holds, and the copy of `process.env` [...] write access to snapshot storage is equivalent to running code in the workflow VM". Por isso, por padrão só se faz snapshot com criptografia, que só a Vercel fornece (`runtime-tuning.mdx:255-258`).
- Só o mesmo build do motor restaura o snapshot. Numa atualização *rolling* self-hosted que troque o motor, todas as runs em voo voltam ao replay completo de uma vez (`runtime-tuning.mdx:251`).

## 4. Determinismo e versionamento

**Regras de determinismo.**
- A função de workflow "must be **deterministic**" e "does not have access to the Node.js runtime" (`workflows-and-steps.mdx:26`, `:48`).
- `Math.random` e `Date` já são fixados pelo sandbox (`:52`).
- Efeitos colaterais devem ir para steps (`code-transform.mdx:309-321`).

**Na Vercel: run presa ao deployment.** "Workflow runs are pinned to the deployment that starts them" (`docs/content/docs/v5/foundations/versioning.mdx:14`).
- Um deploy durante um `sleep("2d")` não altera a run em curso (`:37`, `:67`).
- Uma entrega que chega ao deployment errado é reenviada para o deployment certo, até 3 vezes. Persistindo o erro, a run falha com `DEPLOYMENT_MISMATCH` (`docs/content/docs/v5/errors/deployment-mismatch.mdx:14`, `:36`, `:47`). A justificativa citada é *code skew*: o código novo pode não bater com o histórico gravado.

**Atualizar runs.** A doc recomenda dois caminhos:
- cancelar e rodar de novo no deployment novo (`versioning.mdx:69-117`);
- o padrão "self-upgrading": a run termina chamando `start(..., { deploymentId: "latest" })`. É o equivalente ao `continueAsNew` do Temporal, e "the serialized `state` is the migration boundary between versions" (`:127`, `:166`).

`deploymentId: "latest"` só existe na Vercel (`:120`; `interfaces.ts:801-811`).

**Fora da Vercel o pinning não existe de fato.**
- O World Postgres devolve `getDeploymentId() = 'postgres'`, uma constante (`packages/world-postgres/src/queue.ts:404-406`).
- O Local devolve `dpl_local@<versão do pacote>` (`packages/world-local/src/queue.ts:533-536`).
- Nenhum dos dois declara `deploymentAffinity`. O comentário da capability explica que IDs sintéticos "must leave this unset" (`interfaces.ts:707-720`).
- A doc do Postgres admite: "routing requests across incompatible worker versions remain unimplemented" (`docs/content/worlds/v5/postgres.mdx:122-123`).

**Inferência:** num deploy self-hosted, uma run em voo é reexecutada pelo código novo. Se o código mudou de forma que altere a sequência de steps, o resultado é `REPLAY_DIVERGENCE` e, depois do orçamento de recuperação, `CORRUPTED_EVENT_LOG`. Não há API de *patching* como o `patched()` do Temporal, nem um registro de versão do workflow por run além do `deploymentId`.

**Renomear quebra IDs.** Renomear um arquivo ou uma função muda o ID e "will prevent runs from being upgraded" (`code-transform.mdx:344`).

## 5. A transformação depende de SWC/Next?

**Depende de SWC, mas não de Next.** Todas as integrações chamam o mesmo plugin WASM pelo `@swc/core`:
- `packages/builders` usa esbuild para empacotar e aplica o plugin SWC arquivo a arquivo (`packages/builders/src/apply-swc-transform.ts:79-106`; dependências em `packages/builders/package.json`);
- `@workflow/next` registra um *loader* de webpack/Turbopack que aplica o plugin (`packages/next/src/index.ts:503`, `:678`, `:696`);
- `@workflow/rollup`, `@workflow/vite`, `@workflow/nitro`, `@workflow/astro`, `@workflow/sveltekit` e `@workflow/nest` dependem de `@swc/core` + `@workflow/swc-plugin` diretamente ou via builders (cada `package.json`).

Não há plugin Babel: nenhum `package.json` em `packages/` depende de `@babel/core`.

**Frameworks suportados:**
- Next.js, Vite, React Router, Astro, Express, Fastify, Hono, Nitro, Nuxt, SvelteKit e TanStack Start;
- NestJS e Python, em beta ([workflow-sdk.dev/docs](https://workflow-sdk.dev/docs); arquivos em `docs/content/docs/v5/getting-started/`).

Express, Fastify e Hono passam pelo `workflow/nitro`; TanStack Start pelo `workflow/vite` (`getting-started/express.mdx:13`, `hono.mdx:13`, `tanstack-start.mdx:13`). O SDK Python usa decoradores `@wf.step` e `@wf.workflow`, sem diretivas (`getting-started/python.mdx:14`). Há também um plugin de language service do TypeScript (`packages/typescript-plugin`).

**Bun.** A doc de integração usa o Bun como exemplo. Roda `bun x workflow build` para gerar os bundles, aplica o step mode ao código da app com um `plugin()` do Bun que chama `transform` do `@swc/core` com o plugin, e expõe as rotas com `Bun.serve` (`framework-integrations.mdx:16`, `:49-142`). Integrações de produção devem estender `BaseBuilder` (`:68`, `:163-191`).

## 6. Implicações para o Dormant (inferência)

### (a) O Dormant poderia implementar ou usar a interface World?

**Resposta curta:** dá para reaproveitar o World como camada de storage e fila, mas não o runtime da Vercel. O contrato do World foi desenhado em torno de replay, embora não o imponha.

- **O que assume replay.** Os IDs de evento precisam ser posições densas porque o replay lê o log como prefixo (seção 2.2). O `skip-step-inputs` só funciona porque o replay recalcula argumentos. O `eventCount`/`sinceCursor` serve para incorporar eventos a um replay em curso. As classes de duplicata e o `correlationId` derivado do PRNG pressupõem reexecução. E quem fala com o World é o `@workflow/core` (`workflowEntrypoint`), não o código gerado. Logo, gerar código do Dormant que rode **dentro** do runtime da Vercel significaria emitir chamadas `WORKFLOW_USE_STEP` e voltar ao replay, o que abandona a tese.
- **O que um runtime de snapshot reaproveita.** O World é uma interface TypeScript simples (`packages/world/src/interfaces.ts`).
  - O contrato prevê explicitamente outro runtime no mesmo World, com namespace de fila próprio (`building-a-world.mdx:324`).
  - Um runtime Dormant poderia gravar os mesmos eventos de ciclo de vida (`run_created` … `step_completed`, `wait_*`, `hook_*`). Esses eventos já carregam input e resultado, então são válidos sem replay. A fila daria `sleep` via `delaySeconds`.
  - O **checkpoint de variáveis vivas** iria em `experimental_snapshots`. Essa API é exatamente "bytes opacos + `eventsCursor`, gravados atomicamente, apagados no fim da run" (seção 3.6). A retomada seria carregar o checkpoint e aplicar só os eventos depois do cursor, por exemplo o resultado do Step que estava pendente.
  - Com isso, o Dormant herdaria Postgres, Turso/SQLite, Redis etc., e talvez as ferramentas `workflow inspect`/`web`, se os eventos forem compatíveis **(não verificado)**.
- **Ressalvas.**
  - `experimental_snapshots` é experimental e os metadados atuais são específicos do QuickJS (`serdeRootPtr`, `engineVersion`, `rngDraws`).
  - A spec muda rápido: 8 versões, e `specVersion` precisa acompanhar `@workflow/world`.
  - Para a disciplina, o ADR 0003 (runtime próprio em Bun/SQLite) continua mais simples. O World é uma **saída pós-semestre** comparável à do pi-durable: um "back-end de runtime" trocável pelo gerador de código.

### (b) "Dormant como plugin" comparado ao `@workflow/swc-plugin`

- **Inversão de papéis.** O plugin da Vercel **não transforma o corpo do workflow**, que fica intacto para o replay (`code-transform.mdx:169`). O trabalho dele é particionar (workflow vs. step), dar IDs, capturar closures, fazer DCE e validar. Um plugin Dormant faria o oposto: a reescrita pesada fica justamente no corpo do workflow. Ele vira uma máquina de estados (no estilo do *regenerator*, que transforma geradores e `async` em `switch` sobre um contador de estado), passando por *TAC* (código de três endereços, uma representação intermediária em que cada instrução tem no máximo um operador) e análise de *liveness* (quais variáveis ainda serão lidas depois de cada Ponto de suspensão).
- **O que copiar quase igual.**
  - Mesma superfície de diretivas e mesmas regras de posição (`spec.md:7-16`).
  - Mesma saída de step mode: IIFE de registro e `workflowId` para `start()`.
  - Mesmo pipeline de build: pré-filtro por regex → modo detect → transformação por arquivo dentro de esbuild, Vite, Rollup ou plugin do Bun (`framework-integrations.mdx:77-104`).
  - Mesmo formato de ID `workflow//arquivo//nome`.
- **Parente próximo da liveness.** A captura de closure do plugin já é uma análise de variáveis livres. Ela serializa **todas** as variáveis capturadas pelo step aninhado no momento da chamada (`spec.md:358-395`, `:536-569`). A liveness do Dormant é a versão precisa e por ponto: só o que é lido depois da suspensão.
- **Linguagem do plugin.** Um protótipo em Babel ou num plugin do Bun mantém tudo em TypeScript, como no ADR 0003. Um plugin SWC em Rust fica para depois, também como previsto no ADR 0003.
- **Determinismo.** Com snapshot, o código entre dois Pontos de suspensão roda uma vez por tentativa. A VM com seed passa a ser desnecessária para a correção, mas continua útil para o interpretador com replay da Entrega 3.

### (c) Ideias para o runtime do Dormant

1. **Esquema de eventos.** Usar os tipos `run_* / step_* / hook_* / wait_*` com `correlationId`, guarda de estado terminal e os diagramas de ciclo de vida (seção 2.2). No SQLite:
   - chave primária `(run_id, seq)`, com o `seq` alocado no próprio INSERT (`MAX+1`), como no Postgres;
   - índice único parcial `(run_id, correlation_id, type)` para os eventos de criação;
   - índice único `(run_id, resume_id)` para `hook_received`.
2. **IDs de Step em dois níveis.** O ID estático da função (`step//arquivo//nome`) mais o ID da invocação. No replay da Vercel, o ID da invocação vem da ordem de chamada via PRNG. No Dormant compilado, o natural é derivá-lo do Ponto de suspensão gerado pelo compilador mais um contador de iteração guardado no checkpoint. Assim ele fica determinístico sem PRNG.
3. **Uma fila e uma rota só** para workflow e Step, com `sleep` = mensagem atrasada + entidade `wait`. Somar:
   - entrega at-least-once com `idempotencyKey`;
   - no máximo um executor ativo por run, como a fila nomeada por run do graphile;
   - reenfileiramento das runs ativas no boot, como `recoverActiveRuns`.
4. **Execução inline primeiro.** Rodar Steps no mesmo processo e só usar a fila para `sleep`, hooks e timeout (`whats-new.mdx:41`). No Bun com um processo, isso sai de graça.
5. **Retries declarativos.** `fn.maxRetries` (padrão 3), `FatalError` e `RetryableError({ retryAfter })`, evento `step_retrying` com o erro. Documentar que o corpo do Step é at-least-once (seção 3.2).
6. **Hooks.** Token único com evento de conflito. `resumeHook` grava `hook_received` fora de banda e enfileira, deduplicado por `resumeId`. O registro só é efetivado na suspensão.
7. **Checkpoint com envelope e versão.** Gravar bytes e metadados (`eventsCursor`, `formatVersion`, versão do programa) numa escrita atômica e tratar incompatibilidade como erro explícito (seção 3.6). Diferente da Vercel, o Dormant não tem replay para onde cair, a menos que mantenha o interpretador da Entrega 3 como fallback, o que exigiria determinismo **(especulativo)**.
8. **Versão do programa presa à run**, como o `deploymentId`. O Dormant pode ir além: o compilador sabe quando o conjunto de variáveis vivas de um Ponto de suspensão mudou e pode exigir `migrate` (ver `pi-durable.md`, seção 5). Também vale oferecer o padrão "continue-as-new" via `start()`.
9. **Para o interpretador da Entrega 3** (replay):
   - seed = `runId:nome:versão`;
   - relógio que começa na criação da run e só avança em entregas;
   - remoção de APIs dependentes de tempo do host (seção 3.2);
   - divergência detectada por nome de Step na mesma correlação;
   - nova tentativa antes de declarar o log corrompido.
10. **Testes.** Um "world-sim" do Dormant, isto é, um storage e uma fila determinísticos em memória capazes de encenar corridas como "webhook chega entre `step_started` e a retomada" (`packages/world-sim/README.md`).

### Posicionamento: nota para o ADR 0001 e o `design.md`

O ADR 0001 diz que o Vercel Workflow "retoma por replay". Continua verdade como modelo, mas desde a 5.0 há um caminho **híbrido experimental**: replay + snapshot do heap QuickJS. A diferença para o Dormant fica mais nítida, não menor:

| Aspecto | Snapshot da Vercel | Snapshot do Dormant |
|---|---|---|
| Origem | opaco, imagem inteira da memória da VM | gerado pelo compilador |
| Conteúdo | código, `process.env` e resultados retidos | só as variáveis vivas |
| Inspeção e versionamento | preso ao build exato do motor | inspecionável e versionável |
| Segurança | precisa de criptografia por ser estado executável | dados, não código **(inferência)** |
| Papel | só otimização sobre o log | o próprio mecanismo de retomada |

O quadrante "código linear + snapshot gerado por compilador" continua vazio. Vale atualizar o texto do ADR 0001 e a tabela de quadrantes do `design.md` com essa nuance (não editados nesta pesquisa).

## 7. Itens não verificados

- **Mapeamento das URLs da doc.** A correspondência `docs/content/.../v5/x.mdx` → `workflow-sdk.dev/.../x` foi conferida só para `/docs`.
- **Worlds da comunidade.** Nenhum foi inspecionado (Turso/libSQL, Redis, Cloudflare etc.). Só as descrições do `worlds-manifest.json`.
- **Compatibilidade com observabilidade.** Não verifiquei se `workflow inspect`/`web` exibiriam runs gravadas por um runtime de terceiros num namespace próprio.
- **Versionamento self-hosted.** O comportamento de runs em voo após deploy com código alterado é inferência a partir de `getDeploymentId` constante e da nota do Postgres. Não encontrei teste nem doc que o descreva diretamente.
- **QuickJS no Cloudflare Workers.** A afirmação de que o QuickJS viabiliza Cloudflare Workers vem de comentário de código (`vm-mode.ts:10-13`). Não verifiquei uma integração Cloudflare oficial.
- **Linhas de `runtime.ts`.** O arquivo tem 5.860 linhas. Li o docstring de `workflowEntrypoint` e trechos de `step.ts`, `workflow.ts`, `vm/index.ts` e `suspension-handler.ts`, não o fluxo completo de despacho.
- **Runtime.** Nada foi compilado nem executado. As afirmações sobre comportamento vêm de docs, specs, comentários e leitura de código.
