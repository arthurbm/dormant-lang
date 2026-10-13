# Panorama de compiladores: quais estágios cada ferramenta implementa, e onde o Dormant se encaixa

Pesquisa comparativa sobre vários projetos. Do ecossistema JavaScript/TypeScript: `tsc` 5.x, TypeScript 7, Babel, SWC, esbuild, oxc, Rolldown, Bun, webpack, Rollup, Vite, Terser, React Compiler, scriptc e V8. De outras linguagens: CPython, javac + JVM, Clang/GCC + LLVM, Go e rustc. A referência é o pipeline clássico da disciplina, e cada ferramenta é avaliada contra ele.

**Fontes inspecionadas.** Os repositórios abaixo foram lidos num clone raso, sem compilar nem executar nada. O rótulo da primeira coluna é usado nas citações.

| Rótulo | Repositório | Commit | Versão |
|---|---|---|---|
| `ts5` | [microsoft/TypeScript](https://github.com/microsoft/TypeScript), branch `release-5.9` | `7e133bea121ac13e6f11ddc69a4fd25c6797ffbf` | 5.9.3 |
| `ts7` | [microsoft/TypeScript](https://github.com/microsoft/TypeScript), branch `main` (o port em Go) | `b5c34ba0c61a756c8abaf133236dddf982c27f45` | linha 7.x |
| `babel` | [babel/babel](https://github.com/babel/babel) | `0915316cf4a66e85af77ff9043d2f9c8a439418b` | `@babel/parser` 8.0.7 |
| `swc` | [swc-project/swc](https://github.com/swc-project/swc) | `c07290bb931df2a415649bd1fbda86b312426034` | — |
| `esbuild` | [evanw/esbuild](https://github.com/evanw/esbuild) | `f6058f8364fe7ab91ca57a83e02577ed74c9cae4` | 0.28.2 (`version.txt`) |
| `oxc` | [oxc-project/oxc](https://github.com/oxc-project/oxc) | `1be95031ec2ab4be00db1a0479134894033a1b04` | crates 0.153.0 |
| `rolldown` | [rolldown/rolldown](https://github.com/rolldown/rolldown) | `2d44fc3f5ae63b422e2f235215eafe499afefea0` | 1.2.13 |
| `bun` | [oven-sh/bun](https://github.com/oven-sh/bun) | `11b450ca9b575b03f95bde209ff4f037fa14b374` | 1.4.4 (`package.json`) |
| `webpack` | [webpack/webpack](https://github.com/webpack/webpack) | `ff064c2fd3c131fec3b76325e6c90f5649dbbcf0` | 5.111.1 (`main`) |
| `rollup` | [rollup/rollup](https://github.com/rollup/rollup) | `74517e69d219685f3a31f267a08f5ad9927a2011` | 4.64.5 |
| `vite` | [vitejs/vite](https://github.com/vitejs/vite) | `dce9fe69d37e957b62f3b5df61a761f7f2f949f5` | 8.3.4 |
| `terser` | [terser/terser](https://github.com/terser/terser) | `707c978227009aa6948f2a1048aa59f5fdd5bc40` | 5.51.2 |
| `react` | [facebook/react](https://github.com/facebook/react), só a pasta `compiler/` | `9f2e1f888e5675e0feb12b4e58ecfae36b14cd4d` | — |
| `scriptc` | [vercel-labs/scriptc](https://github.com/vercel-labs/scriptc) | `89cadd38b06f2074a87ae8fc4806763fe235b8bd` | 0.3.0 |
| `workflow` | [vercel/workflow](https://github.com/vercel/workflow) | `02993a26cbabec8f00df1f8a3a758158d095008a` | `@workflow/swc-plugin` 5.0.1 |
| `cpython` | [python/cpython](https://github.com/python/cpython), branch `main` | `f9132f1c555f0e47d7e38251f96adaf0355f9c08` | — |
| `go` | [golang/go](https://github.com/golang/go), branch `master` | `0de83291e3bab5ed6eaa0adc3df752e44c9563f2` | árvore de desenvolvimento do Go 1.28 |
| `rust` | [rust-lang/rust](https://github.com/rust-lang/rust) (só a listagem de `compiler/`) | `88d982422ac482951195284e27f5bcb0d28f298b` | — |

V8, javac/JVM, Clang/LLVM, GCC e parte do rustc vieram só de documentação oficial e de blogs dos mantenedores, citados por URL. Os fatos sobre `scriptc` e `workflow` foram levantados nas pesquisas [`scriptc.md`](scriptc.md) e [`vercel-workflow-worlds.md`](vercel-workflow-worlds.md), nos mesmos commits.

**Como ler as citações.** `esbuild: docs/architecture.md:42-48` aponta para o arquivo e as linhas no commit da tabela. Para abrir no GitHub, use `https://github.com/<repositório>/blob/<commit>/<caminho>#L<linha>`. Caminhos sem rótulo, como `docs/design.md`, são deste repositório. Itens tirados de memória ou de fonte secundária estão marcados como **(não verificado)**. A seção 10 é inferência, e o texto deixa isso explícito.

## 1. O pipeline de referência e o vocabulário

O pipeline da disciplina tem estes estágios:

1. **Lexer** (*scanner*, analisador léxico): quebra o texto em *tokens*, como `if`, `x`, `+`, `42`.
2. **Parser** (analisador sintático): organiza os tokens segundo a gramática.
3. **AST** (*abstract syntax tree*, árvore sintática abstrata): a árvore que o parser produz, sem pontuação nem parênteses supérfluos.
4. **Análise semântica e checagem de tipos:** resolve nomes (a qual declaração cada `x` se refere) e verifica tipos.
5. **Interpretador** (opcional): executa a AST ou outra representação diretamente.
6. **IR** (*intermediate representation*, representação intermediária): uma forma interna mais baixa que a AST. Exemplos: TAC, bytecode, SSA.
7. **Análise e otimização**, em geral por **análise de fluxo de dados** (*data-flow analysis*): propaga fatos ("esta variável é constante", "esta variável ainda será lida") pelo grafo de fluxo de controle até um ponto fixo.
8. **Geração de código** (*codegen*): produz o alvo, que pode ser código de máquina, bytecode ou outra linguagem.

Termos que aparecem ao longo do texto:
- **CFG** (*control-flow graph*): grafo cujos nós são *blocos básicos*, sequências de instruções sem desvio no meio, ligados pelos saltos possíveis. É onde rodam as análises de fluxo de dados.
- **TAC** (*three-address code*): IR em que cada instrução tem no máximo um operador, como `t1 = a + b`.
- **SSA** (*static single assignment*): forma em que cada variável recebe valor uma única vez. Onde dois caminhos se juntam, um nó *phi* escolhe o valor conforme o caminho por onde o controle chegou.
- **Liveness** (análise de variáveis vivas): descobre, para cada ponto do programa, quais variáveis ainda serão lidas mais adiante. É a análise central do Dormant.
- **DCE** (*dead code elimination*): remove código inalcançável ou cujo resultado nunca é usado.
- **Constant folding** (dobramento de constantes): calcula em tempo de compilação expressões como `2 * 60`. **Propagação de constantes** leva esse valor aos usos da variável.
- **Type erasure** (apagamento de tipos): remover anotações de tipo sem mudar o comportamento.
- **Downleveling** (rebaixamento de sintaxe): reescrever sintaxe nova em sintaxe antiga equivalente, como `a ?? b` para uma expressão com `!== null`.
- **Lowering:** tradução de uma representação mais alta para uma mais baixa. O downleveling é um caso particular.

## 2. Quadro-resumo

Legenda: ✅ tem, de forma própria; 🔸 tem em forma limitada (detalhe na célula); ➡️ delega a outra ferramenta; — não tem.

| Ferramenta (linguagem) | Lexer + parser | AST | Semântica / tipos | Interpretador | IR além da AST | Fluxo de dados / otimização | Codegen (alvo) |
|---|---|---|---|---|---|---|---|
| `tsc` 5.x (TS) | ✅ | ✅ | ✅ checagem completa; CFG para *narrowing* | — | 🔸 mini-IR só no transform de geradores | 🔸 análise de fluxo para tipos, sem otimização | JS (erasure + downlevel) e `.d.ts` |
| TypeScript 7 (Go) | ✅ | ✅ | ✅ igual ao 5.x, em paralelo | — | — | 🔸 idem | JS e `.d.ts`; sem alvo ES5 |
| Babel (TS) | ✅ (derivado do acorn) | ✅ | 🔸 escopos; sem tipos | — | — | 🔸 avaliação estática pontual | JS |
| SWC (Rust) | ✅ | ✅ | 🔸 escopos; sem tipos | — | — | 🔸 minifier: DCE, folding, inlining na AST | JS |
| esbuild (Go) | ✅ | ✅ | 🔸 escopos e símbolos; sem tipos | — | — | 🔸 folding mínimo, tree-shaking | JS, CSS |
| oxc (Rust) | ✅ | ✅ (arena) | 🔸 escopos, símbolos; sem tipos | — | 🔸 CFG opcional (linter) | 🔸 minifier na AST, até ponto fixo | JS, `.d.ts` isolado |
| Bun (Rust; antes Zig) | ✅ (porta do esbuild) | ✅ | — (não verificado em doc) | ➡️ JavaScriptCore | — | 🔸 folding | JS |
| webpack (JS) | ✅ (porta do acorn) | ✅ | ➡️ loaders | — | 🔸 grafo de módulos | 🔸 marca exports não usados; ➡️ Terser remove | bundle JS + runtime |
| Rollup (TS + Rust) | ➡️ parser do SWC | ✅ (classes de nó) | 🔸 escopos, efeitos colaterais | — | 🔸 grafo de módulos | ✅ tree-shaking na AST | bundle JS |
| Rolldown (Rust) | ➡️ oxc | ✅ | 🔸 via oxc | — | 🔸 grafo de módulos | ✅ tree-shaking / DCE | bundle JS |
| Vite (TS) | ➡️ Oxc / Rolldown | ➡️ | — por decisão | — | — | ➡️ | ➡️ Rolldown |
| Terser (JS) | ✅ | ✅ (formato próprio) | 🔸 escopos | — | — | ✅ DCE, folding, inlining, só na AST | JS minificado |
| React Compiler (TS) | ➡️ Babel | ➡️ Babel | 🔸 inferência de tipos conservadora + validações | — | ✅ HIR (CFG + SSA) | ✅ const. prop., DCE, aliasing | AST do Babel → JS |
| scriptc (TS) | ➡️ `tsc` 7 | ➡️ `tsc` | ➡️ `tsc` + *preflight* próprio | — | ✅ IR tipada em árvore → LLVM IR | ✅ passes próprios + ➡️ LLVM O2 | nativo, WASM |
| V8 (C++) | ✅ (+ *preparser*) | ✅ | — | ✅ Ignition (bytecode) | ✅ bytecode; CFG/SSA nos JITs | ✅ especulativa, com deoptimização | código de máquina |
| **Dormant v1 (TS, planejado)** | ✅ parser combinators | ✅ | ✅ próprio + regras de durabilidade | ✅ *tree-walking* com Replay | ✅ TAC + CFG | ✅ liveness, const. prop., DCE, CSE | JS (máquina de estados) |

As linhas de outras linguagens estão na seção 9. As fontes de cada célula estão nas seções 3 a 8.

## 3. Compiladores de tipos: `tsc` 5.x e TypeScript 7

### 3.1 `tsc` 5.x, escrito em TypeScript

- **Um arquivo por estágio** em `src/compiler/` (tamanho em linhas):
  - `scanner.ts` (4.101), o lexer;
  - `parser.ts` (10.819), que monta a AST;
  - `binder.ts` (4.008), que cria os símbolos e o grafo de fluxo;
  - `checker.ts` (53.776), o verificador de tipos;
  - `transformer.ts` (696) e `transformers/*.ts`, os transforms;
  - `emitter.ts` (6.365), que imprime o JavaScript.
- **O binder monta um grafo de fluxo, mas só para tipos.**
  - `createFlowNode` está em `ts5: src/compiler/binder.ts:506`.
  - As flags `FlowFlags` incluem `Unreachable`, `Start`, `BranchLabel` e `LoopLabel` (`ts5: src/compiler/types.ts:4149-4153`).
  - O checker percorre esse grafo de trás para frente em `getFlowTypeOfReference` (`ts5: src/compiler/checker.ts:28896`). É isso que implementa o *narrowing*: o estreitamento do tipo de `x` depois de um `if (typeof x === "string")`.
  - É análise de fluxo usada para checar tipos, não para otimizar.
- **A emissão é uma cadeia fixa de transforms AST → AST** (`ts5: src/compiler/transformer.ts:127-190`):
  1. transforms do usuário (`before`);
  2. `transformTypeScript`, o apagamento de tipos;
  3. decorators, JSX, ESNext, class fields;
  4. um transform por versão do ECMAScript, de ES2021 a ES2016, cada um ativado só `if (languageVersion < ScriptTarget.ESxxxx)`;
  5. para alvos abaixo de ES2015, `transformES2015` e `transformGenerators`;
  6. o transform de módulos;
  7. transforms do usuário (`after`).

  A emissão de `.d.ts` é outra cadeia (`:192-197`). Ou seja, o *codegen* do `tsc` é apagamento de tipos mais downleveling, sem otimização.
- **Apagamento de tipos.** `transformTypeScript` (`ts5: src/compiler/transformers/ts.ts:235`). Os comentários dizem que anotações de tipo, assinaturas de índice, declarações só de tipo, modificadores de acesso e declarações `declare` "are elided" (`:580`, `:644`, `:665`, `:695-709`).
- **Conexão direta com o Dormant: o transform de geradores é um lowering para máquina de estados com IR própria.** Ele está em `ts5: src/compiler/transformers/generators.ts:100-103`:

  > "Transforms generator functions into a compatible ES5 representation with similar runtime semantics. This is accomplished by first transforming the body of each generator function into an intermediate representation that is the compiled into a JavaScript switch statement."

  - Os opcodes dessa IR são `.mark LABEL`, `.br`, `.brtrue`, `.brfalse`, `.yield`, `.try`, `.endfinally`, `.return` e outros (`:109-144`). Há também as variáveis especiais `%sent%` e `%error%` (`:149-153`).
  - A saída é `return __generator(function (state) { switch (state.label) { ... } })`, com um `case` por rótulo (`:158-165`).
  - As variáveis locais são içadas para `var` no topo da função. Pelo template, todas sobrevivem entre retomadas, sem análise de liveness (inferência).
- **Modelo de plugins.** Só a API `customTransformers`, com `before`, `after` e `afterDeclarations` (`ts5: src/compiler/transformer.ts:135`, `:188`, `:194`). Que seja o único ponto de extensão da emissão é leitura do código, não afirmação de documentação.

### 3.2 TypeScript 7 ("Corsa"), o port nativo em Go

- **O código está no próprio microsoft/TypeScript.** A branch `main` é o port em Go, em `tsc/`. O antigo repositório de preparação, microsoft/typescript-go, diz no README que está fechado e será arquivado.
- **Os estágios são os mesmos, em Go**, em `ts7: tsc/internal/`: `scanner`, `parser`, `ast`, `binder`, `checker` (com `flow.go`, `inference.go`, `relater.go`), `transformers` e `printer`.
  - Os transforms estão em `tstransforms/` (`typeeraser.go`, `importelision.go`, `legacydecorators.go`, `runtimesyntax.go`) e `estransforms/` (`async.go`, `classfields.go`, `optionalchain.go`, `nullishcoalescing.go`, `using.go`, `forawait.go` e outros).
  - Não há transform de ES2015 nem de geradores, o que bate com o fim do alvo ES5 (inferência).
  - As diferenças intencionais entre o compilador em TS ("Strada") e o em Go ("Corsa") estão listadas em `ts7: tsc/CHANGES.md:1`.
- **Anúncio do port.** O post [A 10x Faster TypeScript](https://devblogs.microsoft.com/typescript/typescript-native-port/), de Anders Hejlsberg (2025-03-11), promete "reduce most build times by 10x" e cerca de metade da memória. A linha em JS continua como 6.x, e a nativa vira 7.0.
- **Lançamento.** O post [Announcing TypeScript 7.0](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/) é de 2026-07-08:
  - desempenho: "typically yield speedups between 8x and 12x on full builds";
  - paralelismo: "TypeScript 7.0 now performs many steps in parallel, including parsing, type-checking, and emitting";
  - cortes: "`target: es5` is no longer supported";
  - API: "While TypeScript 7.0 is here, it does not ship with an API".
- **Por que Go.** A explicação oficial está na [discussão #411](https://github.com/microsoft/typescript-go/discussions/411), de Ryan Cavanaugh (2025-03-05):
  - O critério principal foi compatibilidade: "By far the most important aspect is that we need to keep the new codebase as compatible as possible". O projeto é um **port**, e não uma reescrita, e o estilo de Go se parece com o do código existente.
  - Go dá controle de layout de memória e ainda tem coleta de lixo: "Batch compilations can effectively forego garbage collection entirely, since the process terminates at the end."
  - A equipe avaliou "many language options", fez protótipos e estudou swc, oxc e esbuild. Concluiu que "Many languages would be suitable in a ground-up rewrite situation", mas que Go servia melhor a um port.
  - O ponto fraco admitido é a interoperabilidade com JavaScript.
  - O post **não** nomeia Rust nem C# como alternativas rejeitadas. Essas comparações vêm de comentaristas **(não verificado)**.

## 4. Transpiladores

Um *transpilador* traduz código-fonte para código-fonte no mesmo nível de abstração. Aqui, de TS/JSX/JS moderno para JS. Nenhum dos transpiladores abaixo checa tipos. Eles também não constroem IR além da AST: as transformações são reescritas da árvore.

### 4.1 Babel (TypeScript)

- **O que é.** "Babel is a JavaScript compiler", "a toolchain that is mainly used to convert ECMAScript 2015+ code into a backwards compatible version". Além disso, "Babel is built out of plugins" e "A plugin is just a function" ([docs](https://babeljs.io/docs/)).
- **Parser.** É "Heavily based on acorn and acorn-jsx", e a AST segue o ESTree com desvios documentados ([babel-parser](https://babeljs.io/docs/babel-parser)). É descendente recursivo, com um parser de precedência de operadores para expressões binárias (`babel: packages/babel-parser/src/parser/expression.ts:1-17`).
- **Pipeline: parse → transform → generate** (`babel: packages/babel-core/src/transformation/index.ts:41`, `:50`, `:62`).
  - Em cada passada, os visitors de todos os plugins são fundidos num visitor só, e a árvore é percorrida uma vez (`:110-120`).
  - Ordem: "Plugins run before Presets", "Plugin ordering is first to last", "Preset ordering is reversed" ([plugins](https://babeljs.io/docs/plugins)).
- **Análise na AST.** O `@babel/traverse` rastreia escopos e bindings (`babel: packages/babel-traverse/src/scope/binding.ts`) e avalia expressões estáticas (`babel: packages/babel-traverse/src/path/evaluation.ts`).
- **Sem tipos.** O plugin de TS "does not add the ability to type-check". Ele "will always behave as though isolatedModules is turned on", porque "Babel doesn't support cross-file analysis" ([plugin-transform-typescript](https://babeljs.io/docs/babel-plugin-transform-typescript)).

### 4.2 SWC (Rust)

- **O que é.** "a super-fast TypeScript / JavaScript compiler written in Rust. It's a library for Rust and JavaScript at the same time" (`swc: README.md:27`).
- **Crates por estágio:** `swc_ecma_parser`, `swc_ecma_ast`, `swc_ecma_transforms_typescript`, `swc_ecma_minifier` e `swc_ecma_codegen`, além de `swc_plugin_runner`.
- **Plugins em WebAssembly.** São escritos em Rust, compilados para `.wasm` e implementam `VisitMut`, um visitor que "supports mutating AST nodes" ([plugin getting started](https://swc.rs/docs/plugin/ecmascript/getting-started)). O `@workflow/swc-plugin` do Vercel Workflow é um plugin desse tipo (ver seção 10).
- **Sem tipos.** "SWC works on file-by-file, so any code transforms that depend on understanding the full type system will not work" ([migrating from tsc](https://swc.rs/docs/migrating-from-tsc)).
- **Minifier: o mais próximo de fluxo de dados entre os transpiladores, mas na AST.**
  - As premissas declaradas incluem a ausência de violações de TDZ (*temporal dead zone*) e de efeitos colaterais no acesso a globais (`swc: crates/swc_ecma_minifier/src/lib.rs:1-15`).
  - Os passes estão em `src/compress/optimize/`: `dead_code.rs`, `evaluate.rs`, `inline.rs`, `unused.rs`, `conditionals.rs`, `loops.rs`, `iife.rs` e `sequences.rs`.
  - Uma análise de uso conta referências e atribuições por variável (`ProgramData`, `src/program_data.rs:40`; `VarUsageInfo`, `:106`, com `ref_count` e `assign_count` em `:109` e `:113`).
  - Os passes se repetem até um ponto fixo (`src/compress/mod.rs:105-110`), com limite de 200 rodadas e o erro "Infinite loop detected" (`:136-141`).

### 4.3 esbuild (Go)

- **Decisão de design com a alternativa rejeitada explícita** (`esbuild: docs/architecture.md:42-48`):

  > "Try to do as few full-AST passes as possible for better cache locality. Compilers usually have many more passes because separate passes makes code easier to understand and maintain. There are currently only three full-AST passes in esbuild because individual passes have been merged together as much as possible"

  As três passadas são:
  1. lexing + parsing + escopos + declaração de símbolos;
  2. resolução de símbolos + constant folding + downleveling + *mangling* (encurtamento de nomes);
  3. impressão + source maps.

  Ou seja, esbuild funde numa passada o que o pipeline da disciplina separa em vários estágios.
- **Paralelismo.** "Maximize parallelism" (`:30`). A varredura de arquivos usa uma goroutine por arquivo (`:62`).
- **Otimização mínima.** O constant folding "is pretty minimal", e o ramo morto de um `if` é removido quando a condição vem de `--define` (`:94-104`). O tree-shaking está em `:227-241`.
- **Tipos tratados como espaço em branco.** Parsear TS "just involves skipping over type declarations as if they are whitespace" (`:108`). Em outro ponto: "esbuild does not do any type checking", porque compila "each file in isolation" ([content types](https://esbuild.github.io/content-types/)).
- **Plugins não tocam a AST.** Há `onResolve`, `onLoad`, `onStart` e `onEnd`, e "it's not currently possible to modify the AST directly", para preservar o desempenho ([plugins](https://esbuild.github.io/plugins/)). O FAQ diz que esbuild é "written in Go", que "Everything in esbuild is written from scratch" e que ele "only touches the whole JavaScript AST three times" ([FAQ](https://esbuild.github.io/faq/)).

### 4.4 oxc (Rust)

- **O que é.** "a collection of high-performance tools for JavaScript and TypeScript written in Rust" (`oxc: README.md:33`). As bibliotecas centrais são Parser, Semantic, Linter, Transformer, Minifier e Codegen (`oxc: ARCHITECTURE.md:24`).
- **Pipeline.** `oxc_parser` produz a AST, `oxc_semantic` monta a tabela de símbolos e os escopos, e cada ferramenta segue daí (`oxc: ARCHITECTURE.md:200-206`).
  - A resolução de nomes e alguns erros de sintaxe ficam de propósito com o analisador semântico, e não com o parser (`:233`).
  - `oxc_semantic` monta a cadeia de escopos e a tabela de símbolos e detecta código morto (`:118-125`).
- **Há um CFG, mas para o linter.** O `oxc_cfg` monta um grafo de fluxo de controle com a biblioteca petgraph (`oxc: crates/oxc_cfg/src/lib.rs:9-15`). É uma feature opcional `cfg` do `oxc_semantic` (`oxc: crates/oxc_semantic/Cargo.toml:30`, `:54`). O resto do pipeline trabalha na AST.
- **Sem checagem de tipos.** Um "Type Checker: Full TypeScript type checking" aparece só em "Future Considerations" (`oxc: ARCHITECTURE.md:353`). O `oxc_isolated_declarations` gera `.d.ts` "without requiring full type checking" (`oxc: crates/oxc_isolated_declarations/README.md:1-8`).
- **Minifier na AST.**
  - Faz DCE, constant folding e propagação, mangling e simplificação de comandos e expressões (`oxc: ARCHITECTURE.md:147-155`).
  - Declara inspiração no Closure Compiler, Terser/UglifyJS, esbuild e SWC (`oxc: crates/oxc_minifier/README.md:6-11`).
  - Itera até um ponto fixo: "Fixed-point iteration for optimal size" (`:17`).
- **Duas decisões com alternativa rejeitada** (`oxc: ARCHITECTURE.md`):
  - **Arena em vez de `Rc`/`Arc`.** Todos os nós da AST vivem numa *arena*, um bloco de memória liberado de uma vez, sem contagem de referências (`:36`). O ganho declarado é "10-50% performance improvement", e o custo é "Requires lifetime management" (`:297-311`).
  - **Parser descendente recursivo escrito à mão em vez de gerador de parser** (`:113`, `:314-327`).
- **Plugins.** Visitors em Rust gerados por macros (`:38-40`).

### 4.5 Bun (Rust, antes Zig)

- **Linhagem.** "Bun started as a line-for-line port of esbuild's JavaScript & TypeScript transpiler from Go to Zig." O post [Rewriting Bun in Rust](https://bun.com/blog/bun-in-rust) (2026-07-08) anuncia: "Bun v1.3.14 was the last version of Bun written in Zig. Bun v1.4.0 will be the first version of Bun written in Rust."
- **Implementação atual.** "Bun's transpiler and runtime are written in Rust" (`bun: docs/runtime/index.mdx:10`). O código ainda mostra a origem: `bun: src/ast/base.rs:109` traz "-- original comment from esbuild --", e `bun: src/ast/symbol.rs:672` diz "Equivalent to followSymbols in esbuild".
- **Estágios.**
  - Lexer em `src/js_parser/lexer.rs` e parser em `src/js_parser/parse/`.
  - Os tipos de TS são pulados por `skip_type_script_type` (`bun: src/js_parser/parse/parse_skip_typescript.rs:40`).
  - Folding em `fold.rs`, downleveling em `lower/` e impressão em `src/js_printer`.
  - Não achei verificador de tipos no código nem frase na documentação dizendo que não há (não verificado).
- **API.** `Bun.Transpiler.transformSync()` "does not resolve modules or execute the code. The result is a string of vanilla JavaScript code" (`bun: docs/runtime/transpiler.mdx:18`). Quem executa é o JavaScriptCore, o motor do Safari.
- **Plugins** no estilo do esbuild: `onStart`, `onResolve` e `onLoad` (`bun: docs/bundler/plugins.mdx:6-16`).

## 5. Bundlers

Um *bundler* junta muitos módulos num ou em poucos arquivos. O estágio extra é o **grafo de módulos**, quem importa quem. A otimização típica é o **tree-shaking**: remover exports que ninguém importa. É uma forma de DCE que olha para o programa inteiro.

### 5.1 webpack (JavaScript)

- **O que é.** Um bundler de módulos ESM, CommonJS e AMD (`webpack: README.md:29`, `:67-77`). "Loaders can preprocess files while compiling, e.g. TypeScript to JavaScript", e "Most of the features within webpack itself use this plugin interface" (`:97-100`).
- **Plugins via tapable.** O compilador expõe *hooks* (`SyncHook`, `AsyncSeriesHook`) da biblioteca tapable (`webpack: lib/Compiler.js:13`, `:198-207`). Um hook é um ponto onde plugins se registram para rodar.
- **Parser próprio no `main`.** Um comentário diz: "The ECMAScript parser webpack owns, ported from acorn 8.18.0 so the bundler ships no parser dependency" (`webpack: lib/javascript/syntax-parser.js:811-813`). O acorn ficou só como dependência de desenvolvimento (`webpack: package.json:165`). O `JavascriptParser` percorre a AST disparando hooks (`webpack: lib/javascript/JavascriptParser.js:1173`).
- **Delega a tradução de linguagens aos loaders.** Loaders são "transformations that are applied to the source code of a module" e "can transform files from a different language (like TypeScript) to JavaScript". A cadeia roda "right to left (or from bottom to top)" ([loaders](https://webpack.js.org/concepts/loaders/)). Checar tipos fica com loaders como ts-loader **(não verificado na documentação deles)**.
- **Tree-shaking dividido.** Com `usedExports`, o webpack só marca exports não usados e "relies on terser to detect side effects". `sideEffects` "is much more effective since it allows to skip whole modules/files" ([tree shaking](https://webpack.js.org/guides/tree-shaking/)). O minificador padrão roda Terser com `compress: { passes: 2 }` (`webpack: lib/config/defaults.js:2527-2563`).

### 5.2 Rollup (TypeScript, com parser em Rust)

- **Trocou o parser acorn pelo do SWC na versão 4.** O guia de migração diz que ele "no longer ha[s] the acorn and acornInjectPlugin options", e portanto "you can no longer add plugins for unsupported syntax" (`rollup: docs/migration/index.md:35`). A alternativa rejeitada foi manter o acorn e seus plugins de sintaxe.
  - O lado Rust usa `swc_ecma_parser` (`rollup: rust/parse_ast/Cargo.toml:10-14`).
  - O lado JS converte um buffer binário em AST ESTree (`rollup: src/utils/parseAst.ts:9`).
- **A AST é feita de classes.** São 93 classes em `src/ast/nodes/`, cada uma com `hasEffects(context)` e `include(...)` (`rollup: src/ast/nodes/shared/Node.ts:70`, `:86`, `:216`, `:237`). O tree-shaking é implementado como métodos dos próprios nós.
- **Tree-shaking é análise na AST, não no grafo de módulos.** Rollup "first marks all relevant statements and then 'shakes the syntax tree'", algo "similar in idea to the mark-and-sweep garbage collection algorithm". Como usa ES modules, ele trata todos os módulos como "a big abstract syntax tree with shared bindings" (`rollup: docs/faqs/index.md:13-15`).
- **Plugins** são objetos com hooks de build e de geração de saída (`rollup: docs/plugin-development/index.md:9-13`).

### 5.3 Rolldown (Rust)

- **O que é.** "a JavaScript/TypeScript bundler written in Rust intended to serve as the future bundler used in Vite. It provides Rollup-compatible APIs and plugin interface, but will be more similar to esbuild in scope." (`rolldown: README.md:51`).
- **Construído sobre o oxc** "for the underlying parser, resolver, and sourcemap support" (`rolldown: README.md:77`). O objetivo é "replace esbuild and Rollup … with one unified build tool" (`rolldown: docs/guide/introduction.md:13`).
- **DCE.** Remove código não usado e sem efeitos colaterais (`rolldown: docs/in-depth/dead-code-elimination.md:1-8`).
- **Decisão de design: filtros de hook.** Plugins JS chamados para cada módulo deixam o build "3-4x" mais lento. Com filtros, o Rolldown pula essas chamadas ainda no lado Rust (`rolldown: docs/in-depth/why-plugin-hook-filter.md:5-9`).

### 5.4 Vite (TypeScript): um orquestrador que delega

- **Até o Vite 7:** "esbuild handled fast compilation during development (dependency pre-bundling and TypeScript/JSX transforms)… Rollup handled production bundling, chunking, and optimization" (`vite: docs/blog/announcing-vite8.md:55`).
- **No Vite 8** (post de 2026-03-12): "Vite 8 ships with Rolldown as its single, unified, Rust-based bundler" (`:34`). A pilha passa a ser Vite (ferramenta de build) + Rolldown (bundler) + Oxc (compilador) (`:86`).
  - Transpilar TS: "Vite uses Oxc Transformer to transpile TypeScript into JavaScript" (`vite: docs/guide/features.md:45`).
  - O pré-empacotamento de dependências "is performed with Rolldown" (`vite: docs/guide/dep-pre-bundling.md:30`).
  - A divisão anterior, esbuild no dev e Rollup no build, "introduced inconsistencies" (`vite: docs/guide/why.md:44`).
- **Decisão de design: não checar tipos.** "Vite only performs transpilation on `.ts` files and does **NOT** perform type checking" (`vite: docs/guide/features.md:35`). O motivo (`:37`):

  > "Transpilation can work on a per-file basis and aligns perfectly with Vite's on-demand compile model. In comparison, type checking requires knowledge of the entire module graph. Shoe-horning type checking into Vite's transform pipeline will inevitably compromise Vite's speed benefits."

  O custo é exigir `isolatedModules: true` e abrir mão de `const enum` (`:68-70`). É o mesmo argumento que separa transpiladores (arquivo a arquivo) de verificadores de tipos (programa inteiro).
- **Plugins** "based on Rollup's conventions" (`vite: docs/guide/why.md:38`).

## 6. Minificadores: Terser

- **O que é.** "A JavaScript mangler/compressor toolkit for ES6+" (`terser: README.md:8`), derivado do uglify-es (`:34`).
- **Pipeline, todo sobre a AST própria** (`terser: lib/minify.js`):
  1. `parse` → AST (`:225`);
  2. `figure_out_scope`, a análise de escopos (`:256`);
  3. `Compressor(...).compress(toplevel)` (`:265`);
  4. de novo `figure_out_scope`, e então `mangle_names`, que encurta nomes (`:270-274`);
  5. `OutputStream` / `print`, o codegen (`:327-328`).
- **Otimizações clássicas, sem CFG e sem IR** (opções de `compress`, `terser: README.md`):
  - `dead_code`, "remove unreachable code" (`:730`), é DCE;
  - `evaluate`, "attempt to evaluate constant expressions" (`:747`), é constant folding;
  - `reduce_vars`, "Improve optimization on variables assigned with and used as constant values" (`:832`), funciona como propagação de constantes;
  - `unused`, "drop unreferenced functions and variables" (`:908`), e `inline` (`:767`);
  - `passes`, "The maximum number of times to run compress", com padrão 1 (`:805`).
- **Como `reduce_vars` funciona.** Ele percorre a AST e marca, para cada definição, se o valor é fixo (`def.fixed`, `terser: lib/compress/reduce-vars.js:110-174`). É uma análise de fluxo feita direto na árvore, e não num CFG (inferência a partir do código).
- **Decisão de design.** "Terser has its own abstract syntax tree format; for practical reasons we can't easily change to using the SpiderMonkey AST internally". Em vez disso, há um importador de ESTree (`terser: README.md:1234-1250`).

**Resumo:** minificadores (Terser, e os minifiers do SWC e do oxc) são o lugar do ecossistema JS onde DCE, constant folding e inlining aparecem, mas sempre na AST.

## 7. Compiladores com IR própria

### 7.1 React Compiler (TypeScript)

- **O que é.** "React Compiler is a build-time tool that optimizes your React app through automatic memoization." A v1.0 saiu em 2025-10-07 ([React Compiler v1.0](https://react.dev/blog/2025/10/07/react-compiler-1)). *Memoização* é guardar o resultado de uma computação e reaproveitá-lo enquanto as entradas não mudam.
- **Roda como plugin do Babel, mas tem IR própria.**
  - "The core of the compiler is largely decoupled from Babel, using its own intermediate representations" (`react: compiler/docs/DESIGN_GOALS.md:42`).
  - Há dois pontos de entrada, um plugin do Babel e um do ESLint, que "use the same core compiler logic" (`:40`).
  - Pelo post da v1.0, ele "lowers the Abstract Syntax Tree (AST) provided by Babel into its own novel HIR, and through multiple compiler passes, carefully understands data-flow and mutability".
- **Decisão de design com as alternativas rejeitadas** (`react: compiler/docs/DESIGN_GOALS.md:33-36`):
  - A saída precisa ser código de alto nível, parecido com a entrada: "rather than convert logical expressions (`a ?? b`) into an `if` statement, we retain the high-level form… Rather than convert all looping constructs to a single form, we retain the original form of the loop." Os motivos são tamanho de código e facilidade de depuração.
  - Por isso a IR é uma **HIR** (*High-level Intermediate Representation*), "a name borrowed from Rust Compiler". Ela "retains high-level information (distinguishing if vs logical vs ternary, or for vs while vs for..of) but also represents code as a control-flow graph with no nesting".
- **A HIR é um CFG com SSA.**
  - Os blocos básicos ficam em "reverse postorder" (`DESIGN_GOALS.md:45`).
  - `type BasicBlock = { kind, id, instructions, terminal, preds, phis }` está em `react: compiler/packages/babel-plugin-react-compiler/src/HIR/HIR.ts:347`, e `type Phi` em `:789`. Há análise de dominadores em `src/HIR/Dominator.ts`.
- **Os passes, na ordem**, em `react: compiler/packages/babel-plugin-react-compiler/src/Entrypoint/Pipeline.ts`. As marcas ⟶ indicam troca de representação:
  1. `lower` (`:157`) ⟶ AST do Babel vira HIR;
  2. `enterSSA` (`:188`) e `eliminateRedundantPhi` (`:191`);
  3. `constantPropagation` (`:197`) e `inferTypes` (`:200`);
  4. validações das regras do React, como `validateHooksUsage` (`:205`);
  5. `inferMutationAliasingEffects` (`:220`), `deadCodeElimination` (`:229`) e `inferMutationAliasingRanges` (`:234`);
  6. inferência de *reactive scopes*, grupos de valores criados e mutados juntos (`inferReactiveScopeVariables`, `:334`), com alinhamento, fusão e propagação de dependências (`:384-428`);
  7. `buildReactiveFunction` (`:435`) ⟶ HIR vira ReactiveFunction, "a hybrid of the HIR and an AST" (`DESIGN_GOALS.md:51`);
  8. podas e fusões de escopos (`:458-493`) e `renameVariables` (`:528`);
  9. `codegenFunction` (`:554`) ⟶ volta a ser AST do Babel.

  É o exemplo mais claro, no ecossistema JS, de compilador com o pipeline completo da disciplina: AST → IR em CFG/SSA → fluxo de dados → codegen. Só que o alvo é a própria linguagem de entrada.
- **História.** O primeiro protótipo, de 2021, "was eventually rewritten… moving the compiler's architecture into a Control Flow Graph (CFG) based High-Level Intermediate Representation (HIR). This paved the way for much more precise analysis and even type inference" (post da v1.0).
- **Port para Rust em andamento.** `react: compiler/docs/rust-port/rust-port-architecture.md:5-16` descreve arenas indexadas por ID no lugar das referências compartilhadas do JS. O oxc já tem `crates/oxc_react_compiler`, uma "oxc integration for the Rust port of the React Compiler" (`oxc: crates/oxc_react_compiler/README.md:3`). A maturidade do port não foi verificada.

### 7.2 scriptc (TypeScript): TS para código nativo

Os detalhes estão em [`scriptc.md`](scriptc.md). Em resumo, no commit da tabela:
- **O front-end é o do `tsc` 7.** "TypeScript parses and checks the program against `es2025`… The compiler uses the checked types and narrowing information to lower supported operations" (`scriptc: docs/content/docs/how-it-works.mdx:28`). Um *preflight* próprio rejeita operações fora do subconjunto (`scriptc: docs/content/docs/limitations.mdx:127-129`).
- **IR tipada em árvore, não TAC.** "Every node carries `loc` (source span) and every expression carries its computed `type`… Structured control flow (statement tree), not basic blocks" (`scriptc: packages/compiler/src/ir/ir.ts:4-18`).
- **Passes próprios específicos de JS**, como `integer-ranges.ts`, que prova que um `number` cabe em inteiro de 32 bits. As otimizações clássicas ficam com o LLVM: "LLVM 22's default per-module O2 pipeline" (`scriptc: native/llvm-codegen/README.md:31`).
- **Alvo:** LLVM IR → executável nativo ou WASM, sem motor JavaScript (`scriptc: README.md:10-12`).

## 8. O lado da execução: V8

O V8 é o motor JavaScript do Chrome e do Node. Ele recebe JS, não TS: tudo o que as seções anteriores fazem acontece antes dele.

- **Parser e *lazy parsing*.** O caminho normal gera "an abstract syntax tree (AST) for each function and then compiling it to bytecode". O *preparser* só verifica a sintaxe das funções e adia o parsing completo até a primeira chamada ([Blazingly fast parsing, part 2](https://v8.dev/blog/preparser), 2019).
- **Ignition, o interpretador.** "All JavaScript code is first compiled to ignition bytecode, and executed by interpreting it" ([Maglev](https://v8.dev/blog/maglev), 2023). "Ignition is a register-based interpreter" ([Sparkplug](https://v8.dev/blog/sparkplug), 2021). *Bytecode* é uma IR linear de instruções compactas que um interpretador executa.
- **Sparkplug, o compilador *baseline*.** "compiles directly to machine code in a single linear pass over the bytecode" e "doesn't generate any intermediate representation (IR)" ([Sparkplug](https://v8.dev/blog/sparkplug)).
- **Maglev, o otimizador intermediário.** Usa SSA sobre "a CFG (control flow graph) rather than TurboFan's more flexible but cache unfriendly sea-of-nodes representation". Ele especializa o código para os tipos observados em execução, então "needs to be able to deoptimize": voltar ao bytecode quando a suposição falha ([Maglev](https://v8.dev/blog/maglev)). Um compilador assim é um **JIT** (*just-in-time*): compila durante a execução.
- **TurboFan e a saída do *Sea of Nodes*.** O post [Land ahoy: leaving the Sea of Nodes](https://v8.dev/blog/leaving-the-sea-of-nodes) (2025) explica a troca de IR:
  - O *Sea of Nodes* "does not represent basic blocks, but rather only true dependencies between the instructions".
  - Ele foi substituído por "a more traditional Control-Flow Graph (CFG) Intermediate Representation (IR), which we named Turboshaft".
  - Motivos: "Effect and control chains are hard to understand, leading to many subtle bugs". Depois da troca, o tempo de compilação caiu pela metade.

## 9. Outras linguagens

| Compilador | Lexer + parser | Semântica / tipos | IR | Fluxo de dados / otimização | Alvo |
|---|---|---|---|---|---|
| CPython | tokenizer + parser PEG gerado | tabela de símbolos; sem tipos estáticos | sequência de instruções → CFG → bytecode | folding e DCE no CFG; análise de variáveis não inicializadas | bytecode (+ JIT experimental) |
| javac + HotSpot | scanner + parser | Attr (tipos) e Flow (atribuição definida) | AST → bytecode | quase nada no javac; JIT com perfil na JVM | bytecode JVM → máquina |
| Clang + LLVM | lexer/pré-processador + descendente recursivo | Sema, junto com o parser | LLVM IR em SSA | dezenas de passes (mem2reg, GVN, DCE, SCCP…) | máquina |
| GCC | front-ends por linguagem | por front-end | GENERIC → GIMPLE (TAC) → RTL | Tree SSA e passes RTL | máquina |
| Go (`gc`) | escrito à mão | types2 | IR unificada → SSA | inlining, escape analysis, DCE, regras de reescrita | máquina |
| rustc | lexer + parser | resolução de nomes; tipos na HIR | AST → HIR → THIR → MIR → LLVM IR | borrow checking e otimizações na MIR; LLVM | máquina |

### 9.1 CPython

O pipeline tem cinco etapas (`cpython: InternalDocs/compiler.md:7-17`): tokenização, parsing para AST (`Parser/parser.c`), geração de uma sequência de instruções (`Python/compile.c`), construção e otimização de um CFG (`Python/flowgraph.c`) e montagem do bytecode (`Python/assemble.c`). Antes da geração, uma tabela de símbolos resolve escopos (`_PySymtable_Build()`, `:349-350`).

**Decisão marcante: parser PEG desde o 3.9.** "As of Python 3.9, Python's parser is a PEG parser of a somewhat unusual design" (`cpython: InternalDocs/compiler.md:31-34`). *PEG* (*parsing expression grammar*) é um formalismo em que a escolha entre alternativas é ordenada, então "PEG grammar cannot be ambiguous" (`cpython: InternalDocs/parser.md:41-43`). Usa memoização (*packrat*), que garante tempo linear ao custo de memória (`:60-69`). O parser é gerado a partir de `Grammar/python.gram` (`compiler.md:36-40`). O [PEP 617](https://peps.python.org/pep-0617/) substituiu o parser LL(1) anterior porque ele forçava gambiarras na gramática, não aceitava regras recursivas à esquerda como `expr: expr '+' term` e mantinha uma árvore concreta intermediária em memória (paráfrase; ver seção 11).

Um detalhe útil para a disciplina: no commit inspecionado não existe mais `Python/ast_opt.c`. O arquivo atual é `Python/ast_preprocess.c` (`compiler.md:512-513`), e o constant folding geral acontece **no CFG**: `fold_const_binop` (`cpython: Python/flowgraph.c:1947`), `optimize_load_const` (`:2413`) e `remove_unreachable` (`:1010`), que é DCE. Há também uma análise de fluxo de dados de verdade: `add_checks_for_loads_of_uninitialized_variables` (`flowgraph.c:3397-3435`) propaga uma máscara de bits de "possivelmente não inicializada" pelos blocos básicos, com uma *worklist* (fila de blocos a reprocessar), e insere `LOAD_FAST_CHECK` onde uma variável local pode estar indefinida. É o mesmo esquema da liveness, na direção oposta.

Na execução, o [PEP 659](https://peps.python.org/pep-0659/) trouxe o interpretador adaptativo especializante, que substitui instruções genéricas por variantes especializadas para os tipos observados. O [PEP 744](https://peps.python.org/pep-0744/) descreve um JIT experimental *copy-and-patch*, gerado a partir da mesma DSL do interpretador, que "is currently not part of the default build configuration".

### 9.2 javac + JVM

A [visão geral oficial da compilação](https://openjdk.org/groups/compiler/doc/compilation-overview/index.html) lista as fases: Parse e Enter (tokens, árvores e tabelas de símbolos), processamento de anotações, **Attr** ("names, expressions and other elements within the syntax tree are resolved", a checagem de tipos), **Flow** ("Flow analysis is used to check for definite assignment to variables, and unreachable statements"), TransTypes (apaga genéricos) e Lower (troca construções por árvores mais simples), e por fim Gen, que gera os bytecodes. A JVM é uma máquina de pilha: as instruções "take operands from the operand stack, operate on them, and push the result back onto the operand stack" ([JVMS 21 §2.6.2](https://docs.oracle.com/javase/specs/jvms/se21/html/jvms-2.html)). A HotSpot usa compilação em camadas: o compilador *client* gera versões que coletam perfil, e o *server* otimiza com base nele ([HotSpot performance enhancements](https://docs.oracle.com/en/java/javase/21/vm/java-hotspot-virtual-machine-performance-enhancements.html)).

**Decisão marcante (inferência):** o javac praticamente não otimiza. A fase Flow é fluxo de dados usado para **checar** (atribuição definida), não para otimizar, e a otimização pesada fica com o JIT, que tem o perfil de execução em mãos.

### 9.3 Clang/GCC + LLVM

No Clang ([Internals Manual](https://clang.llvm.org/docs/InternalsManual.html)), o parser "contains a recursive-descent parser that polls tokens from the preprocessor". O Sema "is called by the Parser library during parsing to do semantic analysis… For valid programs, Sema builds an AST". Ou seja, parsing e análise semântica são intercalados, não passadas separadas. Depois, "CodeGen takes an AST as input and produces LLVM IR code from it". A LLVM IR "is a Static Single Assignment (SSA) based representation" com três formas equivalentes: em memória, bitcode em disco e texto legível ([LangRef](https://llvm.org/docs/LangRef.html)).

**Decisão marcante: o front-end não constrói SSA sozinho.** O passe `mem2reg` "promotes alloca instructions which only have loads and stores as uses… This is just the standard SSA construction algorithm to construct 'pruned' SSA form" ([Passes](https://llvm.org/docs/Passes.html)). A mesma página lista os passes clássicos: `gvn` (numeração global de valores, que elimina cálculos redundantes), `dce`, `adce`, `sccp` (propagação de constantes condicional e esparsa), `inline` e `licm` (tira do laço o que não muda a cada iteração). Que o Clang emita `alloca` para as variáveis locais e deixe a promoção para o `mem2reg` vem de memória **(não verificado)**.

No GCC, a [GIMPLE](https://gcc.gnu.org/onlinedocs/gccint/GIMPLE.html) "is a three-address representation derived from GENERIC by breaking down GENERIC expressions into tuples of no more than 3 operands". É um TAC de produção, irmão do TAC da disciplina. A cadeia é GENERIC → GIMPLE → RTL, com passes Tree SSA e RTL ([Passes](https://gcc.gnu.org/onlinedocs/gccint/Passes.html)).

### 9.4 Go (`gc`)

O `go: src/cmd/compile/README.md` descreve sete fases:
1. parsing no pacote `syntax`, onde "Each syntax tree is an exact representation of the respective source file" (`:28-39`);
2. checagem de tipos pelo `types2`, "a port of go/types to use the syntax package's AST" (`:41-46`);
3. *noding*, que converte as árvores tipadas na IR do compilador, a "Unified IR" (`:48-62`);
4. middle end com "dead code elimination, (early) devirtualization, function call inlining, and escape analysis" (`:64-74`);
5. *walk*, que fixa a ordem de avaliação e remove açúcar sintático (`:76-88`);
6. SSA genérico, com DCE, remoção de checagens de `nil` desnecessárias e de ramos inúteis (`:90-115`);
7. lowering por arquitetura e emissão de código de máquina (`:117-141`).

O parser "is now hand-written instead of generated from yacc" desde o [Go 1.6](https://go.dev/doc/go1.6). Parte dos passes SSA é gerada a partir de regras de reescrita em `_gen/*.rules` (`go: src/cmd/compile/internal/ssa/README.md:229-236`). Para ver a SSA de uma função, `GOSSAFUNC=Foo go build` gera um `ssa.html` (`README.md:205-208`).

**Decisão marcante: o back-end SSA do [Go 1.7](https://go.dev/doc/go1.7).** "a new code generation back end for 64-bit x86 systems… based on SSA, generates more compact, more efficient code and provides a better platform for optimizations such as bounds check elimination", com redução de 5% a 35% no tempo de CPU dos benchmarks.

### 9.5 rustc

Pelo [overview do rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/overview.html):
- lexing e parsing produzem a AST, e na mesma etapa acontecem expansão de macros, validação e resolução de nomes;
- a AST vira HIR, onde roda a checagem de tipos;
- a HIR vira THIR, "an even more desugared HIR used for pattern and exhaustiveness checking";
- a THIR vira MIR, que "is basically a Control-Flow Graph (CFG)" e "is used for borrow checking and other important dataflow-based checks";
- por fim, "the first step is to convert the MIR to LLVM-IR".

O compilador inteiro é organizado como consultas (*queries*) com cache, para compilação incremental. Os back-ends no repositório são `rustc_codegen_llvm`, `rustc_codegen_cranelift` e `rustc_codegen_gcc` (listagem de `rust: compiler/`).

**Decisão marcante: o borrow checker na MIR.** O [borrow checker](https://rustc-dev-guide.rust-lang.org/borrow-check.html) "operates on the MIR". "An older implementation operated on the HIR". O motivo: "the radical desugaring helps prevent bugs in the borrow checker", e a MIR permite *non-lexical lifetimes* (NLL), "regions derived from the control-flow graph". Ele roda "dataflow analyses that compute what data is moved and when". É uma análise de fluxo de dados sobre CFG usada para **checar** o programa, de família próxima à liveness que o Dormant vai usar nos Pontos de suspensão.

## 10. Onde o Dormant se encaixa (inferência)

Esta seção é inferência a partir das seções anteriores e do design em `docs/design.md`.

### 10.1 O que o Dormant tem

Pelo design (`docs/design.md`, seção 7), o Dormant tem:
- parser com uma biblioteca própria de parser combinators;
- verificador de tipos próprio, com regras de durabilidade (seção 6.2): efeitos só em Steps, `await` só de Steps e `sleep`;
- interpretador *tree-walking* (que percorre a AST diretamente) com Replay;
- geração de TAC, CFG e otimizações (propagação de constantes, DCE, CSE);
- liveness nos Pontos de suspensão;
- geração de uma máquina de estados em JavaScript, executada pelo Runtime no Bun.

A implementação é em TypeScript sobre o Bun (ADR 0003).

### 10.2 Posição no mapa

| Comparado com | Semelhança | Diferença |
|---|---|---|
| `tsc` | Parser e verificador de tipos próprios; emite JS | O `tsc` não tem IR nem otimização, e a emissão é apagamento de tipos + downleveling. O Dormant aceita só um subconjunto, mas vai até TAC, CFG e fluxo de dados |
| Transform de geradores do `tsc` 5.x | Lowering para `switch` sobre um rótulo de estado, via mini-IR com rótulos e saltos (seção 3.1) | O `tsc` iça **todas** as variáveis locais. O Dormant salva só as **vivas** no Checkpoint, e o estado precisa sobreviver a reinícios, não só a um `yield` em memória |
| Transpiladores (Babel, SWC, esbuild, oxc, Bun) | Mesma entrada (sintaxe TS), mesma saída (JS) | Trabalham arquivo a arquivo, sem tipos e sem IR além da AST. O que fazem de otimização é na AST |
| Minificadores (Terser, minifiers do SWC/oxc) | DCE e constant folding | O Dormant faz isso sobre TAC + CFG, como no livro-texto. Os minificadores fazem na AST, com contagem de uso e pontos fixos |
| React Compiler | Transformação de JS para JS com IR própria (CFG) e fluxo de dados; distribuído como plugin de outra ferramenta | O React Compiler usa o parser do Babel, tem SSA e quer saída parecida com a entrada. O Dormant tem parser próprio, usa TAC sem SSA, e a saída (máquina de estados) é deliberadamente diferente da entrada |
| scriptc | Subconjunto de TS com compilador próprio de verdade | O scriptc reaproveita o front-end do `tsc`, usa IR em árvore e deixa as otimizações clássicas para o LLVM. O Dormant escreve front-end e otimizações, e o alvo é JS |
| V8 | Interpretador antes do compilador | O interpretador do V8 executa bytecode e alimenta JITs especulativos. O do Dormant é a semântica de referência (Replay) para os testes diferenciais |
| CPython, rustc, javac | Fluxo de dados sobre CFG usado para checar ou preparar o programa (variáveis não inicializadas, borrow checking, atribuição definida) | A liveness do Dormant decide o **conteúdo do Checkpoint**, e não um erro de compilação. A regra futura de proibir closures vivas num Ponto de suspensão (`docs/design.md`, seção 8) seria do mesmo tipo das checagens do rustc |

**Resumo.** No ecossistema JS, o Dormant fica mais perto do React Compiler e do transform de geradores do `tsc` do que dos transpiladores. É um compilador JS → JS que precisa de CFG e fluxo de dados para fazer uma transformação que não dá para fazer só reescrevendo a árvore. Entre os projetos pesquisados, é o único que cobre todos os estágios da disciplina, de parser a codegen, inclusive um interpretador.

### 10.3 Estágios da disciplina que cada família tem e não tem

- **Transpiladores:** têm lexer, parser, AST, escopos e codegen. **Não têm** tipos, IR além da AST nem fluxo de dados em CFG. O oxc tem um CFG, mas para o linter.
- **Bundlers:** acrescentam o grafo de módulos e o tree-shaking. Este último é um DCE de programa inteiro, feito na AST (Rollup, Rolldown) ou dividido com o Terser (webpack).
- **Minificadores:** fazem DCE, constant folding, inlining e propagação de constantes, todos na AST.
- **`tsc`:** tem tudo no front-end, inclusive um grafo de fluxo para narrowing. **Não tem** IR geral nem otimização. A exceção é a mini-IR do transform de geradores, que o TS 7 não tem mais.
- **React Compiler:** tem IR em CFG + SSA, fluxo de dados e codegen, mas delega o parser ao Babel.
- **scriptc:** tem IR e passes próprios, mas delega o front-end ao `tsc` e a otimização clássica ao LLVM.

### 10.4 A ideia pós-semestre: liveness + máquina de estados como plugin de transpilador

O roadmap prevê aplicar a análise e a geração da máquina de estados a TypeScript real, como plugin de um transpilador (`docs/roadmap.md:27`). O mapa acima sugere o que isso exige:

- **Um plugin de transpilador só recebe AST.** Babel e SWC entregam visitors sobre a AST (seções 4.1 e 4.2), e esbuild nem isso (4.3). Para fazer liveness, o plugin teria de construir **dentro de si** o que o transpilador não tem: CFG e análise de fluxo. É exatamente o que o React Compiler faz dentro do Babel (seção 7.1). Ele é o precedente mais próximo de "compilador completo empacotado como plugin".
- **O plugin do Vercel Workflow faz o oposto.** Ele particiona o código em workflow e step, gera IDs e captura closures, mas não transforma o corpo do workflow, que fica intacto para o Replay (`workflow: docs/content/docs/v5/how-it-works/code-transform.mdx:169`, conforme [`vercel-workflow-worlds.md`](vercel-workflow-worlds.md), seção 3.1). A captura de closures já é uma análise de variáveis livres, mas serializa **todas** as variáveis capturadas (`workflow: packages/swc-plugin-workflow/spec.md:358-395`). Um plugin Dormant reescreveria o corpo e salvaria só as variáveis vivas.
- **Sem tipos no transpilador.** As regras de durabilidade do verificador do Dormant (seção 6.2 do design) não estariam disponíveis num plugin de Babel ou SWC, que trabalham sem tipos e arquivo a arquivo (seções 4.1, 4.2 e 5.4). Seria preciso aceitar checagens só sintáticas, ou rodar o `tsc` à parte, como o Vite recomenda para tipos.
- **Separação já prevista.** Manter TAC, CFG e liveness independentes da sintaxe (`docs/roadmap.md:27`) é o que permite trocar o front-end de parser combinators por uma AST de Babel, SWC ou oxc, como o React Compiler faz com o Babel e, no port para Rust, com o oxc.

## 11. Itens não verificados

- **Por que não Rust ou C# no TypeScript 7.** A discussão oficial não nomeia linguagens rejeitadas. Essas comparações vêm de comentaristas.
- **Bun não checa tipos.** É leitura do código (o parser pula os tipos), e não encontrei frase na documentação que diga isso.
- **Citações de páginas web.** Os trechos dos posts do React Compiler, do V8 e do PEP 617 foram obtidos por uma ferramenta de busca que resume o conteúdo. O sentido foi conferido, mas a redação exata deve ser revista antes de uma citação literal. O PEP 617 está parafraseado de propósito.
- **CPython.** A versão em que o constant folding saiu da AST para o CFG, e em que `ast_opt.c` virou `ast_preprocess.c`, não foi conferida. O nome da opção de build do JIT do PEP 744 também não.
- **JVM.** A documentação da Oracle fala em compiladores *client* e *server*. Os nomes C1 e C2 vêm de memória. O Graal não foi pesquisado.
- **Clang.** Que ele emita `alloca` para variáveis locais e deixe a SSA para o `mem2reg` vem de memória.
- **GCC.** O que cada passe RTL faz não foi pesquisado.
- **Go e rustc.** A expressão "descendente recursivo" não aparece literalmente nas fontes lidas sobre esses dois parsers. Para o Go, a fonte diz só "hand-written".
- **webpack.** O parser próprio e o minificador `jsMinify` foram vistos no `main`, e não conferi se já estão numa versão publicada. Também não conferi que o webpack dependia antes do acorn em tempo de execução.
- **Loaders do webpack.** A documentação do ts-loader e do babel-loader não foi lida.
- **Vite 6 e 7.** Só o post do Vite 8 foi lido. A situação intermediária do pacote `rolldown-vite` não foi detalhada.
- **Minificação no Rolldown.** A introdução do Rolldown diz "Minification (WIP)" (`rolldown: docs/guide/introduction.md:27-35`). Já o `vite: docs/guide/why.md:46` diz que ele "uses Oxc for parsing, transforming, and minifying".
- **Port do React Compiler para Rust.** A maturidade e o status de lançamento não foram verificados, nem como o Bun integra um crate `bun_react_compiler` (`bun: src/js_parser/react_compiler_host.rs:1-5`).
- **Execução.** Nada foi compilado nem executado. Todas as afirmações vêm de documentação e de leitura de código.
