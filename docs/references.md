# Referências

Material levantado durante a definição do projeto (out/2026), com o porquê de cada item. Itens marcados com **(conferir)** vieram de memória ou de fonte secundária e precisam ser verificados antes de citar.

## Workflows duráveis

| Referência | Por que importa |
|---|---|
| [Vercel Workflow SDK — code transform](https://workflow-sdk.dev/docs/how-it-works/code-transform) | Origem da sintaxe com diretivas. O plugin SWC (`@workflow/swc-plugin`) **não** gera máquina de estados: separa o código em bundles (modo workflow mantém o corpo do workflow e troca steps por `WORKFLOW_USE_STEP`; modo step faz o inverso). Retoma por **replay** numa VM do Node com `Math.random`/`Date.now` controlados. IDs estáveis `step//{arquivo}//{função}` |
| [pi-durable](https://github.com/earendil-works/pi) (`packages/durable`) e [post](https://earendil.com/posts/pi-durable/) | Earendil (Armin Ronacher, Mario Zechner), lançado com o Pi 1.0 em 01/10/2026, MIT, TS. **Snapshot**: tarefas como máquinas de estados escritas à mão (`defineTask` com `phases` + `checkpoint` JSON), sem replay e sem exigência de determinismo; `version` + `migrate`; "effect sandwich" (commit da intenção → efeito → commit do resultado); steps marcados `replay: "safe"`; memos first-writer-wins; dedup por `requestId`; regra "fase sem progresso no checkpoint = falha". Referência inicial do formato de saída do Dormant |
| [Absurd](https://github.com/earendil-works/absurd) | Projeto anterior do Armin Ronacher: execução durável só com Postgres, por replay. Tem [comparação](https://earendil-works.github.io/absurd/comparison/) com Temporal, Inngest e DBOS |
| [Temporal](https://docs.temporal.io) | Replay do histórico de eventos; restrições de determinismo; limites de histórico e Continue-As-New; versionamento com `patched()`/`getVersion` |
| [Cloudflare Workflows — regras](https://developers.cloudflare.com/workflows/build/rules-of-workflows) | Replay; nomes de step como chave de cache |
| [Restate](https://docs.restate.dev/concepts/durable_execution) | Diário (journal) por invocação; replay |
| [Effect v4 — ClusterWorkflowEngine](https://effect.website/docs/v4/api/effect/unstable/cluster/ClusterWorkflowEngine) | Workflows duráveis no Effect, ainda em `effect/unstable/*` |
| Burckhardt et al., *Durable Functions: Semantics for Stateful Serverless*, OOPSLA 2021 **(conferir)** | Formalização da semântica do modelo de replay (Azure Durable Functions) |

## Transformação de código em máquinas de estados (Entrega 4)

| Referência | Por que importa |
|---|---|
| [Roslyn — CodeGen differences](https://github.com/dotnet/roslyn/blob/features/localsinit/docs/compilers/CSharp/CodeGen%20Differences.md) | Em Release, só locais que atravessam `await` são movidos para a máquina de estados, via análise de fluxo de dados. É a liveness do Dormant, em memória |
| [regenerator](https://github.com/facebook/regenerator) e o `__generator` do `tsc` (alvo ES5) | Lowering de generators/`async` em JS para `switch` sobre contador de estado |
| Kotlin coroutines | Campo `label` + variáveis "derramadas" no objeto de continuação |
| Rust `async` | Cada `Future` é um enum de estados; o tamanho depende das variáveis vivas através de cada `.await`. Analogia para a regra "closure não pode estar viva numa pausa" (`Send` através de `.await`) |
| Racket web server — continuações serializáveis; J. McCarthy, ICFP 2009 **(conferir título exato)** | Precedente acadêmico de "compilador torna a execução salvável" |
| Stackless Python (tasklets serializáveis), XState v5 (`getPersistedSnapshot`), AWS Step Functions (máquina em JSON) | Outras formas de snapshot/máquina explícita |
| CRIU, imagens Smalltalk, snapshot de memória WASM | Snapshot "força bruta" (salva tudo, sem liveness): contraste para "por que salvar o mínimo" |
| Reynolds, *Definitional Interpreters for Higher-Order Programming Languages* (1972) | Defuncionalização: caminho para closures serializáveis (pós-v1) |

## Contexto que levou à tese (descartados, mas úteis)

| Referência | Por que importa |
|---|---|
| [scriptc](https://scriptc.dev) — [repo](https://github.com/vercel-labs/scriptc) | Vercel Labs, jul/2026: TS → executável nativo. Compilador em TS (~260k LOC), front-end é o TypeScript 7 (Go); IR é árvore tipada (não TAC); **desde 0.2.0 só LLVM, sem C**; otimizações clássicas ficam com o LLVM; testes diferenciais contra Node (~2.000 programas). Zig só como toolchain de cross-compile. Inspirou a ideia de testes diferenciais e a tese descartada "MiniTS → C" |
| [gdp-ts](https://github.com/rauchg/gdp-ts) | Rauch, out/2026: "provas" de autorização checadas em compilação (Ghosts of Departed Proofs). Ideia de tipos-prova, descartada como tese |
| [Bend 2](https://github.com/bendlang/bend) | Lançado em 17/09/2026; tipos dependentes + afins, compilador em TS/Bun, `LAWS.bend`/`PROOF.bend`. Descartado como alvo e como implementação (ver ADR 0001). A demo `demos/proof_typed_eval` (otimizador com prova de correção) é uma ideia de extensão futura |
| Cloudflare Code Mode / "code execution with MCP" **(conferir)** | Base da tese alternativa "scripts de agente com permissões checadas" |

## Disciplina

| Referência | Por que importa |
|---|---|
| [MiniC](https://github.com/rbonifacio/MiniC) | Linguagem explorada nas aulas da turma de EC. Rust + `nom`; parser, type checker, interpretador e gerador de TAC (`src/ir/tac.rs`); sem otimizações nem back-end. `docs/` é um bom modelo de documentação |
| [SmartTS](https://github.com/rbonifacio/SmartTS) | Linguagem das aulas da turma de CC. Haskell + Megaparsec; gera LLTZ (Tezos). `docs/09-projects.md` lista as features do projeto padrão |
| [miniLisp](https://github.com/rbonifacio/miniLisp) | Scala; mostra os dois estilos de parser (ANTLR e combinators) |
| [ScalaWhileLang](https://github.com/rbonifacio/ScalaWhileLang) e TIP | Análises de fluxo de dados (reaching definitions, very busy expressions): referência para a Entrega 4 |
| *Engineering a Compiler* (Cooper, Torczon) | Livro-texto: parser (caps. 2–3), IR e otimização |
| *Principles of Program Analysis* (Nielson, Nielson, Hankin) | Seções 2.3–2.4 = Tema 09 do seminário (Monotone Framework), base da liveness |
| *Types and Programming Languages* (Pierce) | Bloco 2 dos seminários; verificação de tipos |
