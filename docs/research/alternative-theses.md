# Teses alternativas: como a tese do Dormant foi escolhida

Este documento registra a escolha da tese: quais candidatas foram avaliadas, com que critérios e por que a tese A virou o Dormant (ver [ADR 0001](../adr/0001-compiled-snapshot-thesis.md)).

Toda afirmação traz o link da fonte. Itens tirados de memória ou de fonte secundária estão marcados como **(não verificado)**. As notas e a classificação (seções 3 e 4) são julgamento, não fato.

## 1. Critérios

O projeto tem quatro Entregas:
1. proposta;
2. gramática + parser;
3. tipos + interpretador;
4. IR + otimização + geração de código.

A Entrega 4 pede uma *IR* (representação intermediária, a forma do programa entre o parser e o código final) do tipo *TAC*. TAC é o código de três endereços, em que cada instrução tem no máximo um operador, como `t1 = a + b`. Sobre ela rodam *análises de fluxo de dados*, que calculam fatos válidos em cada ponto do programa propagando informação pelo *CFG* (grafo de fluxo de controle, os blocos de código ligados pelos saltos possíveis). As otimizações esperadas são:
- **propagação de constantes:** substituir uma variável pelo valor constante que ela sempre tem naquele ponto;
- ***DCE*** (eliminação de código morto): remover cálculos cujo resultado nunca é usado;
- ***CSE*** (eliminação de subexpressões comuns): reaproveitar um cálculo repetido;
- ***liveness*:** descobrir, em cada ponto, quais variáveis ainda podem ser lidas mais adiante.

Cada candidata foi avaliada por quatro critérios:
- **Encaixe em cada Entrega,** sobretudo na 4. Uma tese em que o data-flow é um apêndice vale menos que uma em que ele é o produto.
- **Ancoragem real:** um sistema existente em 2025–2026 que dê credibilidade ao problema.
- **Trade-offs defensáveis:** decisões de design com alternativas reais para discutir.
- **Risco:** a chance de não caber no prazo da disciplina num projeto individual implementado em TypeScript sobre Bun ([ADR 0003](../adr/0003-typescript-bun-js-target-own-runtime.md)).

## 2. Ponto de partida: MiniTS → C

A tese de reserva era um subconjunto estaticamente tipado de TypeScript compilado para C via TAC. O problema é que a Vercel Labs já publicou o [scriptc](https://github.com/vercel-labs/scriptc), que "compiles TypeScript and JavaScript to native executables and WebAssembly" e "uses TypeScript's type information to compile supported code to native instructions". O README atual diz que "there is no runtime JavaScript engine fallback" e exibe o selo "Vercel Labs Experiment".

A cobertura da [InfoQ](https://infoq.com/news/2026/09/vercel-scriptc-node) (2026-09) descreveu uma flag `--dynamic` com QuickJS embutido e emissão de C e LLVM IR. Isso contradiz o README atual **(não verificado: pode ser uma versão anterior)**.

Seja como for, o MiniTS teria de responder à pergunta "por que não usar o scriptc?". **Nota: 6/10.** É seguro, mas não tem ângulo próprio.

## 3. Candidatas

### A. Compilador de workflows duráveis — 8,5/10 (escolhida)

**Ancoragem.** Os sistemas populares de workflow durável retomam por **Replay**: reexecutam o código do início e devolvem de um log os resultados dos Steps já concluídos.
- **Vercel Workflow SDK.** O plugin `@workflow/swc-plugin` lê as diretivas `"use workflow"`/`"use step"`. No modo workflow, "Step function bodies are replaced with calls to `globalThis[Symbol.for("WORKFLOW_USE_STEP")]`", e o runtime precisa "replay past steps from the event log rather than re-executing them". Os IDs estáveis seguem `{type}//{filepath}//{functionName}`. O código roda numa VM com "seeded random/time APIs" (`Math.random()`, `Date.now()`). Os modos documentados hoje são Step, Workflow e Detect ([code-transform](https://workflow-sdk.dev/docs/how-it-works/code-transform)). O antigo modo "client" foi incorporado ao modo step na versão 5.0 (mesma página).
- **Cloudflare Workflows.** "Step names act as the "cache key" in your Workflow." E também: "Workflows may hibernate and lose all in-memory state", por isso o estado deve vir "exclusively comprised of `step.do` returns" ([rules of workflows](https://developers.cloudflare.com/workflows/build/rules-of-workflows/)).
- **Restate.** "Restate replays the journal, skipping completed steps" ([durable execution](https://docs.restate.dev/concepts/durable_execution)).
- **Effect.** O engine de workflows do Effect 4 "runs durable workflows on top of cluster sharding and message storage", num módulo marcado como *unstable* ([ClusterWorkflowEngine](https://effect.website/docs/v4/api/effect/unstable/cluster/ClusterWorkflowEngine)).
- **Temporal** também retoma por replay determinístico **(não verificado nesta rodada)**.

**Precedente do caminho alternativo.** O caminho alternativo é a **máquina de estados compilada**: cada `await` vira um estado, e o compilador salva os valores necessários para continuar dali. É o que o C# faz. A nota de codegen do Roslyn diz: "it relies on precise data flow analysis. Basically only locals that cross awaits need to be lifted" ([CodeGen Differences](https://github.com/dotnet/roslyn/blob/main/docs/compilers/CSharp/CodeGen%20Differences.md)). O [regenerator](https://github.com/facebook/regenerator) faz uma transformação de fonte análoga para generators em ES5.

**Tese.** Compilar código linear para uma máquina de estados retomável por **Snapshot**, salvando em cada Ponto de suspensão só as variáveis vivas, calculadas por liveness sobre o TAC.

**Escopo mini.** Os tipos básicos, funções anotadas, `if`/`while`, Steps, `await` de Step e `sleep` no nível de comando do workflow. Os cortes estão no [ADR 0004](../adr/0004-v1-scope-cuts.md).

**Encaixe nas Entregas.**

| Entrega | Encaixe | Motivo |
|---|---|---|
| 1 | forte | Problema real, com um trade-off claro (replay vs. snapshot). |
| 2 | ok | Gramática comum; as diretivas são strings, sem sintaxe nova ([ADR 0002](../adr/0002-valid-typescript-syntax-with-directives.md)). |
| 3 | ok/forte | O interpretador implementa replay, o que permite comparar as duas estratégias com implementação própria. |
| 4 | **forte** | Liveness *é* o produto. As outras otimizações têm efeito visível: o DCE encolhe o conjunto vivo e, com ele, o snapshot. |

**Pontos para defender o design.**
- Replay exige determinismo e custa reexecutar o histórico.
- Snapshot exige valores serializáveis e esbarra no versionamento de código em voo.
- Liveness é uma análise *backward* ("para trás": a informação flui do uso para a definição) e *may* ("pode": uma variável é viva se *algum* caminho a lê).
- Por que o Roslyn só "levanta" (salva no estado) os locais que cruzam `await`.

**Risco principal.** Um `await` dentro de laços e blocos aninhados complica a divisão em estados. A v1 só permite `await` no nível de comando.

### B. Erros tipados / rastreamento de efeitos — 7/10

**Ancoragem.**
- **Swift.** O typed throws (`throws(CatError)`) está "Implemented (Swift 6.0)". A própria proposta avisa que "the existing (untyped) throws remains the better default" e que "errors are usually propagated or rendered, but not exhaustively handled". `do throws(E)` existe, mas a inferência do tipo lançado em closures "did not get implemented in Swift 6.0" ([SE-0413](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0413-typed-throws.md)).
- **Koka.** "polymorphic effects through row-polymorphism using duplicate labels", com tipo e efeito inferidos automaticamente ([Leijen, 2014](https://arxiv.org/abs/1406.2061)). Uma *row* (linha) é uma lista extensível de efeitos, como `<exn|e>`.
- **Effect.** O Effect 4.0 foi lançado ("Today, we're releasing Effect 4.0", página atualizada em 2026-09-30, [anúncio](https://effect.website/blog/releases/effect/40)). A dist-tag `latest` no npm é `4.0.3` ([registry](https://registry.npmjs.org/effect)). O canal `E` de `Effect<A, E, R>` rastreia erros no tipo.
- **TypeScript.** O pedido de cláusula `throws` foi recusado ("Declined", fechado como "not planned" em 2023-11-10, [issue #13219](https://github.com/microsoft/TypeScript/issues/13219)).

**Escopo mini.** `function f(): int throws ParseErr | IoErr`. Um `try`/`catch` remove do conjunto os erros que trata, e `main` precisa tratar todos.

**Encaixe.**

| Entrega | Encaixe | Motivo |
|---|---|---|
| 1 | forte | Lacuna real e documentada no TypeScript (issue #13219). |
| 2 | ok | Gramática comum. |
| 3 | **forte** | O checker de tipos é o centro do projeto. |
| 4 | fraco/ok | O rastreamento vive no tipo. No TAC sobram só as arestas de exceção do CFG, e as otimizações não têm relação com a tese. |

**Pontos para defender o design.** União vs. rows vs. subtipagem. A fadiga das checked exceptions do Java. A orientação da própria SE-0413 contra o uso indiscriminado.

**Risco principal.** A tese e a Entrega 4 ficam desconectadas.

### C. Tipos-prova / capacidades (gdp-ts) — 6,5/10

**Ancoragem.**
- **gdp-ts.** O [rauchg/gdp-ts](https://github.com/rauchg/gdp-ts) é "a verification system to make API contracts more secure at compile (type check) time". Os proofs são "ghosts": "they do nothing at runtime, and everything at compile time". Só um módulo confiável em `proofs/` cria cada proof. Uma função sensível exige o proof na assinatura, como em `disablePasswordProtection<U, P>(project: Named<P, ProjectId>, _proof: UserIsProjectAdmin<U, P>)`. Presets de ESLint e Oxlint pegam o que o checker não pega, como forjar um proof com `as`. Requer TypeScript 5.4 ou mais novo.
- **Origem.** A ideia vem de [gdp](https://hackage.haskell.org/package/gdp), de Matt Noonan (2018), creditado no README.

**Tese.** Numa linguagem, a impossibilidade de forjar um proof seria nativa, sem linter, e os proofs seriam apagados na geração de código.

**Encaixe.**

| Entrega | Encaixe | Motivo |
|---|---|---|
| 1 | forte | A pergunta "por que linguagem e não biblioteca?" tem resposta direta: o linter vira regra de tipos. |
| 2 | ok | Gramática comum. |
| 3 | ok/arriscado | Valores nomeados exigem tipos existenciais ou gerativos, teoria de tipos profunda para o prazo. |
| 4 | fraco | Apagar os proofs é trivial. Daria para acrescentar uma análise de dominância ("a checagem precede a chamada em todo caminho"), mas seria acessória. |

**Risco principal.** A teoria de tipos.

### D. Linguagem reativa à la React Compiler / Svelte — 5,5/10

**Ancoragem.**
- **React Compiler.** A 1.0 saiu em 2025-10-07. O compilador "lowers the Abstract Syntax Tree (AST) provided by Babel into its own novel HIR, and through multiple compiler passes, carefully understands data-flow and mutability". A HIR é "Control Flow Graph (CFG) based" e permite "memoize conditionally" ([React Compiler v1.0](https://react.dev/blog/2025/10/07/react-compiler-1)). O post não menciona *SSA* (forma de atribuição única estática, em que cada variável é atribuída uma só vez); o uso de SSA internamente vem de memória **(não verificado)**.
- **Svelte 5.** "signals are an under-the-hood implementation detail", e o compilador "can ditch the signals altogether" no SSR ([Introducing runes](https://svelte.dev/blog/runes), 2023-09-20).

**Escopo mini.** Células reativas no estilo planilha (`let a = 1; derived b = a * 2`), sem UI.

**Encaixe.**

| Entrega | Encaixe | Motivo |
|---|---|---|
| 1 | ok | A ancoragem é forte, mas sem UI a demonstração perde força. |
| 2 | ok | Gramática comum. |
| 3 | ok | O interpretador é um grafo de dependências. |
| 4 | ok em teoria, armadilha na prática | O data-flow que importa (escopos de memoização, faixas de mutabilidade) é nível de pesquisa. |

**Risco principal.** Demo fraca sem UI, e uma UI traz um runtime inteiro junto.

### E. Linguagem funcional à la Gleam → BEAM — 5,5/10

**Ancoragem.** O compilador do Gleam foi reescrito em Rust porque "the lack of static types was making refactoring a slow and error prone process". Seus alvos são Erlang abstract forms, código-fonte Erlang e JavaScript ([FAQ](https://gleam.run/frequently-asked-questions/)). A release mais recente é a v1.19.2, de 2026-10-12 ([releases](https://github.com/gleam-lang/gleam/releases)).

**Encaixe.**

| Entrega | Encaixe | Motivo |
|---|---|---|
| 1 | forte | Proposta clara, com tipos anotados e sem inferência Hindley–Milner. |
| 2 | forte | Gramática funcional enxuta. |
| 3 | forte | Checker e interpretador são diretos. |
| 4 | fraco | Compiladores funcionais costumam usar *ANF* (forma normal A, em que todo valor intermediário ganha nome) ou CPS, não TAC. Erlang já tem atribuição única, então passar por TAC e voltar é artificial, e liveness pouco importa com a BEAM como alvo. |

**Ponto para defender o design.** ANF vs. TAC.

**Risco principal.** A Entrega 4 encaixa mal.

### F. Scripts de agente com capacidades checadas — 7,5/10

**Ancoragem.**
- **Cloudflare.** O post de 2025-09-26 afirma que "LLMs are better at writing code to call MCP, than at calling MCP directly" ([Code Mode](https://blog.cloudflare.com/code-mode/)). O pacote atual gera declarações TypeScript das tools e roda o código num Worker isolado, com rede bloqueada por padrão (`globalOutbound: null`). Tools com `requiresApproval: true` pausam, e `approve()` "Resumes a paused execution through replay" ([Codemode API](https://developers.cloudflare.com/agents/api-reference/codemode/)).
- **Anthropic.** "Agents scale better by writing code to call tools instead" ([Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp), 2025-11-04).

**Tese.** O checker infere estaticamente quais tools e efeitos um script pode alcançar, para que o host aprove ou recuse o script *antes* de rodar.

**Encaixe.**

| Entrega | Encaixe | Motivo |
|---|---|---|
| 1 | forte | Tendência de 2025–2026. |
| 2 | ok | Gramática comum. |
| 3 | forte | O interpretador é o sandbox, e os conjuntos de capacidades são tipos. |
| 4 | ok/forte | Propagação de constantes e DCE podam chamadas inalcançáveis, e o conjunto de permissões encolhe. |

**Risco principal.** Crescer o escopo até integrar um LLM.

### Bend 2 como alvo ou como linguagem de implementação (descartado)

O [Bend 2](https://github.com/HigherOrderCO/Bend) é "a fast language that blocks AI mistakes via proof". Os alvos são "C, Metal, CUDA and JavaScript". O README admite "Error messages are terse; no debugger, profiler or REPL" e avisa que "Bend 1 programs and HVM do not carry over". O lançamento por volta de 2026-09-17 e as dezenas de versões em poucas semanas vêm de fontes secundárias **(não verificado)**. As razões do descarte estão no [ADR 0001](../adr/0001-compiled-snapshot-thesis.md).

## 4. Resumo

| Tese | E1 | E2 | E3 | E4 | Nota |
|---|---|---|---|---|---|
| **A. Workflows duráveis (snapshot via liveness)** | forte | ok | ok/forte | **forte** | **8,5** |
| F. Scripts de agente com capacidades | forte | ok | forte | ok/forte | 7,5 |
| B. Erros tipados | forte | ok | forte | fraco/ok | 7 |
| C. Tipos-prova (gdp-ts) | forte | ok | ok/arriscado | fraco | 6,5 |
| MiniTS → C | ok | ok | ok | forte | 6 |
| D. Reativa | ok | ok | ok | armadilha | 5,5 |
| E. Gleam → BEAM | forte | forte | forte | fraco | 5,5 |

## 5. Por que A venceu

1. **É a única candidata em que a análise de fluxo de dados da Entrega 4 é o produto.** Nas outras, liveness e as demais otimizações seriam um apêndice. No Dormant, liveness decide o que entra no snapshot, e cada otimização tem efeito mensurável no tamanho do estado salvo.
2. **O trade-off é real e tem ancoragem dupla.** Vercel, Cloudflare e Restate documentam o replay e suas regras de determinismo. O Roslyn documenta o caminho compilado, com locais salvos por análise de fluxo de dados. O Dormant implementa os dois: replay no interpretador (Entrega 3) e snapshot no compilador (Entrega 4).
3. **O ângulo é próprio.** Ao contrário do MiniTS, não existe um "scriptc" que já faça isso. A frase de posicionamento correta está no [ADR 0001](../adr/0001-compiled-snapshot-thesis.md): uma biblioteca só consegue snapshot se o programador escrever a máquina de estados à mão, e uma linguagem consegue a partir de código linear.
4. **O compilador base garante as Entregas.** Se a camada durável atrasar, o subconjunto de TS com parser, checker, interpretador e TAC cumpre o plano sozinho ([ADR 0004](../adr/0004-v1-scope-cuts.md)).

## 6. O que as outras teses podem dar ao Dormant (funcionalidades futuras)

- **B → erros tipados em Steps.** Um Step poderia declarar com que erros falha (`throws(PaymentDeclined)`), e o checker exigiria que o Workflow tratasse cada erro ou o marcasse como passível de retry. Isso conversa com a regra de Steps idempotentes da [Cloudflare](https://developers.cloudflare.com/workflows/build/rules-of-workflows/) ("a step might be retried multiple times"). Depende de `try`/`catch`, cortado da v1 ([ADR 0004](../adr/0004-v1-scope-cuts.md)).
- **C → proofs que atravessam Pontos de suspensão.** Um Step de aprovação humana devolveria um proof, e o Step de cobrança o exigiria na assinatura. Como proofs são "ghosts" sem conteúdo em runtime, eles nunca precisariam entrar no snapshot. É um caso didático de valor vivo para os tipos e morto para a serialização.
- **F → Code Mode durável por snapshot.** O runtime durável do Code Mode da Cloudflare retoma execuções pausadas "through replay" ([Codemode API](https://developers.cloudflare.com/agents/api-reference/codemode/)). Um script de agente escrito em Dormant mostraria a alternativa por snapshot. Isso fica dentro do escopo de orquestração definido no ADR 0001.
- **D e E** não deixam herança direta. A comparação entre ANF e TAC (E) é útil para justificar a escolha de TAC na Entrega 4.

## 7. Itens não verificados

- **scriptc e QuickJS.** A flag `--dynamic` com QuickJS e a emissão de C/LLVM IR vêm da InfoQ e contradizem o README atual do scriptc.
- **Temporal.** A retomada por replay determinístico vem de memória.
- **SSA no React Compiler.** O uso de SSA vem de memória e de um espelho não oficial da documentação; o post oficial não menciona SSA.
- **Lançamento do Bend 2.** A data e o ritmo de versões vêm de fontes secundárias.
- **Effect 4.0.** O post de lançamento não cita nominalmente os módulos `effect/unstable/workflow` e `cluster`. O status *unstable* vem da página de API do `ClusterWorkflowEngine`.
- **Notas.** As notas e o encaixe por Entrega são julgamento do autor, não medição.
