# scriptc: TypeScript compilado para nativo, e o que serve ao Dormant

Pesquisa sobre o repositório [vercel-labs/scriptc](https://github.com/vercel-labs/scriptc), no commit `89cadd38b06f2074a87ae8fc4806763fe235b8bd` (versão 0.3.0, `packages/compiler/package.json:3`). Também usei o site [scriptc.dev](https://scriptc.dev/), cujas páginas de documentação são os arquivos `docs/content/docs/*.mdx` do próprio repositório.

Os caminhos citados, como `packages/compiler/src/ir/ir.ts:4`, apontam para esse commit. Para abrir um deles no GitHub, use `https://github.com/vercel-labs/scriptc/blob/89cadd38b06f2074a87ae8fc4806763fe235b8bd/<caminho>#L<linha>`. Itens tirados de memória ou de fonte secundária estão marcados com **(não verificado)**. As seções 5 e 6 são inferência minha, e o texto deixa isso explícito.

**Mudanças recentes (v0.2.6 → v0.3.0):**
- **A v0.3.0 (2026-10-12) removeu o modo `--dynamic` e o QuickJS embutido.** Hoje "The `--dynamic` option and embedded JavaScript execution runtime have been removed" (`CHANGELOG.md:13`). O QuickJS é um interpretador de JavaScript pequeno, escrito em C. Até a v0.2.x, o código que o scriptc não conseguia compilar (dependências npm, valores `any`) podia rodar dentro desse interpretador, levado junto no executável. Agora esse código é compilado ou rejeitado.
- **A v0.2.10 trouxe os "contract types" (`@scriptc/types`)** (`CHANGELOG.md:35`); ver a seção 3.
- **Volume de lançamentos:** foram cinco versões em seis dias, da v0.2.6 à v0.3.0 (`CHANGELOG.md:9-116`).
- **Contribuidores:** a API do GitHub mostra `ctate` com 1183 commits e `cramforce` com 360.
- **O que não mudou:** o back-end continua sendo só LLVM, e a IR continua sendo uma árvore tipada (seção 2).

## 1. O que é

O README resume assim: "scriptc compiles TypeScript and JavaScript to native executables and WebAssembly. It uses TypeScript's type information to compile supported code to native instructions. Executables run without Node.js or a JavaScript engine." (`README.md:10`). E logo depois: "Unsupported operations produce diagnostics; there is no runtime JavaScript engine fallback." (`README.md:12`).

- **Status.** O próprio projeto se descreve como "experimental", com suporte a "a subset of JavaScript, TypeScript, and Node.js APIs" (`README.md:14`). Ele aparece como experimento da Vercel Labs.
- **Licença.** Apache-2.0 (`LICENSE`).
- **Público-alvo e casos de uso.** CLIs, servidores HTTP pequenos (as APIs `node:http`, `net`, `fs` etc. foram reimplementadas em C), módulos WASI e bibliotecas nativas linkáveis por FFI. WASI é a interface de sistema padronizada para WebAssembly. FFI (*foreign function interface*) é o mecanismo para chamar funções C a partir do programa. Fontes: `docs/content/docs/index.mdx:47-50` e as páginas `wasm.mdx` e `ffi.mdx`.
- **Benchmarks.** A documentação atual não publica nenhum. A imprensa de julho citou ganhos de 47 ms → 2,4 ms na inicialização, de ~100 MB → 4 MB de memória e de ~80 MB → 3 MB de binário, números do próprio projeto ([runtimewire](https://runtimewire.com/article/vercel-scriptc-typescript-native-compiler-no-javascript-engine)) **(não verificado)**.
- **Comando `scriptc coverage`.** Ele analisa o programa sem compilar e informa quantas instruções compilam para código nativo. As que não compilam são agrupadas por código de diagnóstico, com motivo, local e dica (`docs/content/docs/coverage.mdx:5`, `:28-38`).

## 2. Arquitetura

### 2.1 Pipeline e front-end

A página "How It Works" desenha o pipeline (`docs/content/docs/how-it-works.mdx:9-26`):

```text
TypeScript / JavaScript
  → parser e type checker do TypeScript
  → lowering e IR tipada      (--emit=ir, JSON)
  → LLVM IR                   (--emit=llvm)
  → helper LLVM               (--emit=asm|obj)
  → runtime pack + linker     → executável
```

- **Lowering** é a tradução de uma representação de alto nível para outra mais baixa.
- **IR** (*intermediate representation*) é a representação interna entre o front-end e o back-end.
- **Cada estágio pode ser despejado em disco com `--emit`.** O comando `scriptc build fib.ts --emit=ir` grava `.scriptc/fib.ir.json` (`how-it-works.mdx:44-60`).

**O front-end não é do scriptc: é o compilador TypeScript oficial.** "TypeScript parses and checks the program against `es2025`… The compiler uses the checked types and narrowing information to lower supported operations" (`how-it-works.mdx:28`).
- **Qual versão.** A dependência é o `typescript` 7.0.2, a versão nativa do TS escrita em Go, acessada pelos módulos `typescript/unstable/ast`, `/sync` e `/proto`. Há também um `typescript5` (5.9.3) para alguns caminhos (`packages/compiler/package.json:42-43`).
- **Parser e checker.** O scriptc não escreve parser nem type checker. Ele lê os tipos e o *narrowing* calculados pelo `tsc` e escolhe, a partir deles, a representação nativa de cada valor. *Narrowing* é o estreitamento de tipo depois de testes como `typeof x === "string"`.
- **Rejeição tem uma etapa própria.** Um programa que passa no `tsc` ainda pode usar operações fora do subconjunto. Uma segunda checagem prévia (*preflight*) separa esses casos dos erros de tipo comuns e emite diagnósticos próprios, com dica de reescrita (`docs/content/docs/limitations.mdx:127-129`).

### 2.2 A IR: árvore tipada, não TAC

O arquivo `packages/compiler/src/ir/ir.ts` tem cerca de 4.000 linhas, e o validador `validate.ts` tem outras 6.000. O comentário de abertura de `ir.ts` (`:4-18`) fixa as regras de desenho:

> "scriptc IR — the only interface between frontend and backends. […] Plain JSON-safe discriminated unions: no classes, no parent pointers, no cycles. Cross-references are by string id. Every node carries `loc` (source span) and every expression carries its computed `type` — backends never re-derive types. Structured control flow (statement tree), not basic blocks […] All switches over IrType/IrExpr/IrStmt must have exhaustive `never` default arms so adding a member turns into compile errors, not silent misbehavior."

Ou seja, **não é TAC nem SSA**.
- **TAC** (*three-address code*) é a forma em que cada instrução tem no máximo um operador e três endereços, como `t1 = a + b`, organizada em blocos básicos.
- **SSA** (*static single assignment*) é uma variante em que cada variável recebe valor uma única vez.
- **A IR do scriptc mantém `if`/`while` como árvore.** Quem monta o grafo de fluxo de controle é o LLVM.

Trecho representativo:

```ts
// packages/compiler/src/ir/ir.ts
export type IrType =                                        // :62
  | { kind: "f64" }                                         // :63
  | { kind: "string" } // heap, refcounted, UTF-8           // :76
  | { kind: "array"; elem: IrType } // heap, refcounted, monomorphic elements  // :78
  | { kind: "record"; shapeId: string }                     // :301
  | { kind: "union"; unionId: string }                      // :307
  // ...
export type IrStmt =                                        // :1603
  | { kind: "if"; cond: IrExpr; then: IrStmt[]; else_: IrStmt[] | null; loc: SrcLoc }  // :1616
```

Outros detalhes:
- **Locals.** Cada função lista todas as suas variáveis locais "pre-collected and scope-flat", com IDs únicos como `x.0` e `x.1` para nomes sombreados. Assim, o front-end resolve o escopo léxico uma única vez (`ir.ts:1500`).
- **Closures.** Uma closure é um nó com a lista explícita de capturas (`{ kind: "closure"; fnName; captures: string[] … }`, `ir.ts:2614-2621`).
- **Tipos concretos.** "generic functions are specialized, unions use tags, and closures have explicit captures" (`how-it-works.mdx:29`). Especializar funções genéricas, gerando uma cópia por tipo concreto, é o que se chama de *monomorfização*.

### 2.3 Passes próprios e o que fica com o LLVM

A pasta `packages/compiler/src/ir/` tem análises específicas de JavaScript:
- `integer-ranges.ts` prova que um `number` (f64) cabe em inteiro de 32 bits ("Exactly representable integers, excluding negative zero", `:6`), o que permite usar aritmética inteira;
- `integer-loops.ts` detecta laços com contador;
- `scalar-records.ts` quebra records pequenos em escalares;
- `constant-tables.ts`, `final-fields.ts` e `int32-slots.ts` cobrem tabelas constantes, campos finais e slots inteiros.

As otimizações clássicas ficam com o LLVM:
- **Quais são elas.** Propagação de constantes; eliminação de código morto, ou DCE (*dead code elimination*); eliminação de subexpressões comuns, ou CSE (*common subexpression elimination*); alocação de registradores.
- **Onde isso está escrito.** O helper nativo usa "LLVM 22's default per-module O2 pipeline" (`native/llvm-codegen/README.md:31`). Não achei nenhuma implementação própria desses passes. Que eles sejam deixados inteiramente ao LLVM é leitura minha do código, não uma afirmação da documentação.

### 2.4 Back-ends

- **Só existe o back-end LLVM, desde a v0.2.0.** "LLVM is the only code generation backend. C emission, the debugging C backend, and automatic C fallback have been removed" (`CHANGELOG.md:226`). Versões anteriores geravam C.
- **Saídas.** O LLVM 22 gera assembly ou objeto para macOS, Linux, Windows e WASI (`how-it-works.mdx:30`).
- **O próprio compilador é distribuído como binário nativo.** Ele embute o parser e o checker do TS 7 (`CHANGELOG.md:231`).

### 2.5 Runtime e representação de valores

O runtime é escrito em C (`packages/runtime/src/scr_*.c`, cerca de 116 mil linhas). Segundo `how-it-works.mdx:75-81`:
- **Memória:** contagem de referências (*refcounting*), em que cada valor sabe quantos ponteiros apontam para ele e é liberado quando o contador zera. Um coletor de ciclos roda em pontos definidos.
- **Assincronismo:** `async`/`await` sobre *fibers* com pilha própria e um event loop em kqueue (macOS) ou epoll (Linux).
- **Números:** sempre double, o f64 do JS.

Representação de valores, conforme `ir.ts`:
- **string:** UTF-8 com refcount (`:76`).
- **array:** monomórfico, isto é, todos os elementos têm o mesmo tipo (`:78`).
- **Object literals e interfaces:** viram *records* com layout nativo exato. As formas são internadas estruturalmente, de modo que "equal shapeId ⇔ equal shape" (`:297-301`). Por isso, passar `{ a, b }` onde se espera `{ a }` copia o valor (`limitations.mdx:38`).
- **Classes:** objetos nominais (`ir.ts:275`).
- **Unions:** "IMMUTABLE tagged boxes: a runtime tag (the arm's index in the canonical order) plus one payload slot" (`ir.ts:303-307`).
- **`unknown`:** vira uma árvore JSON dinâmica que só pode ser usada depois de um *checked cast* (`ir.ts:308-321`). Por exemplo, `JSON.parse(s) as Config` valida o dado em tempo de execução e lança `expected number at $.port, got string` (`docs/content/docs/index.mdx:73`).

## 3. Subconjunto de TypeScript suportado

A página `docs/content/docs/index.mdx:47-50` lista o que compila estaticamente:
- classes, herança, closures, genéricos, *discriminated unions*, desestruturação e spread;
- `async`/`await`, promises, exceções, iteradores e `using`;
- strings, BigInt, arrays, Map, Set, JSON, Math, typed arrays e Buffer;
- APIs do Node para fs, path, process, child_process, crypto, timers e rede.

Restrições notáveis (`docs/content/docs/limitations.mdx`):
- `==` entre objeto e primitivo é proibido, porque `valueOf` e `toString` podem rodar código do usuário (`:26`);
- genéricos declarados dentro de outra função não são suportados (`:29`);
- algumas unions exigem narrowing antes do uso, como em `u.length` sobre `string | string[]` (`:39`);
- várias operações sobre `any` resultam em `SC2011` (`:117`);
- `JSON.parse` devolve `unknown` (`:127`);
- `Object.keys` segue a ordem de declaração do record, não a ordem de inserção (`:173`).

Os **contract types** da v0.2.10 (`docs/content/docs/contracts.mdx`) são aliases como `NoThrow<T>`, `Int32`, `Immutable<T>` e `Acyclic`.
- **Para o TypeScript e o Node, não significam nada:** são importados com `import type` e apagados na compilação TS.
- **O scriptc prova cada um deles ou rejeita o programa,** e usa a garantia para otimizar.
- **Exemplo:** `Acyclic` dispensa o coletor de ciclos para aquele tipo.

Também existe `comptime(() => …)`, que avalia uma função durante a compilação e embute o resultado como literal (`index.mdx:80-83`).

## 4. Testes

- **Oráculo diferencial.** O cabeçalho de `tests/harness/differential.test.ts:1-6` diz: "every corpus program runs under Node AND as a scriptc-compiled native binary; stdout AND stderr must match byte-for-byte and exit codes must agree […] No golden files — Node IS the expected output, so tests can't drift." *Teste diferencial* é rodar o mesmo programa em duas implementações e comparar as saídas. O Node funciona como especificação executável.
- **Corpus.** A pasta `tests/corpus/` tem 2.243 entradas, com nomes numerados como `001-hello.ts` e `1002-json-parse-cast.ts`.
- **Testes de diagnóstico.** São 171 arquivos em `tests/diagnostics/`, cada um com construções que precisam ser rejeitadas e um snapshot da mensagem esperada em `tests/harness/__snapshots__/`. Exemplo (`any.ts.txt`):

  ```text
  any.ts:8:5 - error SC2011: destructuring on 'any'-typed values cannot compile statically
    8 | var { x } = <any>0;
      |     ^~~~~~~~~~~~~~
    hint: use a supported typed operation, or 'unknown' with a checked cast ('x as T')
  ```

- **Registro único de códigos.** Os códigos ficam todos em `packages/compiler/src/diagnostics/diagnostic.ts:1-12`, onde "Message strings are never inlined at rejection sites" e cada código pertence a uma faixa:
  - `SC0xxx`: erros de TS repassados;
  - `SC1xxx`: TS válido fora do subconjunto;
  - `SC2xxx`: regras de tipo do scriptc;
  - `SC3xxx`: limitações de back-end ou de alvo.

  Um teste garante que "every diagnostic carries a category and a hint" (`packages/compiler/src/diagnostics/envelope.test.ts:50`).
- **Outras frentes:** a suíte oficial do ECMAScript, `test262` (`tests/test262/`), e uma bateria com AddressSanitizer (ferramenta que detecta acessos inválidos à memória) mais uma auditoria de refcount ao final da execução (`how-it-works.mdx:89`).

## 5. O que aproveitar no Dormant (inferência, em ordem de utilidade)

1. **Teste diferencial, sem arquivos dourados.** O ADR 0002 já prevê que um Workflow rodado no Bun, sem reinícios, dê o mesmo resultado que no Dormant. O scriptc mostra como transformar isso na bateria principal. No Dormant, o oráculo tem três lados:
   - o Bun rodando o `.ts` cru;
   - o interpretador com replay (Entrega 3);
   - o JS gerado com snapshot (Entrega 4).

   A extensão natural é a **injeção de crash**: matar o processo em cada ponto de suspensão, retomar e exigir que stdout e código de saída sejam idênticos aos da execução sem interrupção.
2. **Registro de diagnósticos com código, faixa e dica.** O Dormant também rejeita parte do TypeScript. Vale copiar o formato `arquivo:linha:col - error DMxxxx: … / hint: …`, o registro central e o teste de que "todo diagnóstico tem dica". O caso mais característico do Dormant seria uma variável viva num ponto de suspensão com tipo não serializável, por exemplo uma closure, com a dica "mova para dentro de um Step".
3. **`--emit` por estágio.** Despejar `ast`, `ir`, `tac`, `liveness` e `js` em JSON ou texto, cada um com sua flag. Isso facilita a depuração e serve de material para mostrar cada Entrega, como "aqui está o TAC, aqui as variáveis vivas em cada `sleep`".
4. **Regras de desenho da IR.** As quatro valem para o Dormant:
   - uniões discriminadas serializáveis em JSON, sem ponteiros para o pai;
   - todo nó com `loc` e o tipo já calculado;
   - `switch` exaustivo com `never`;
   - um validador de IR rodando entre os passes.

   **Contraste importante:** o scriptc pode ficar numa árvore porque o LLVM monta o CFG (*control-flow graph*, o grafo de blocos básicos) e faz as otimizações. O Dormant precisa descer até TAC com CFG por conta própria, porque a análise de *liveness* nos pontos de suspensão é a tese (ADR 0001). *Liveness* é a análise que descobre quais variáveis ainda serão lidas adiante.
5. **Relatório de "cobertura" adaptado.** Um `dormant check` poderia listar, para cada ponto de suspensão, quais variáveis estão vivas e se são serializáveis, no espírito do `scriptc coverage`. Isso torna a análise da Entrega 4 visível ao usuário.
6. **Checked casts na fronteira.** O `JSON.parse(s) as T` validado do scriptc é o mesmo problema da desserialização de snapshot no Dormant. Ao retomar, validar o formato do snapshot contra os tipos das variáveis vivas e falhar com o caminho do erro, no estilo `expected number at $.x`.
7. **Unions como tag + payload, records por forma.** Escolher uma representação explícita também facilita definir o formato serializado do snapshot.
8. **Contrato de tipo provado pelo compilador.** A ideia dos contract types, apagados pelo TS e provados pelo compilador, combina com o prelúdio do ADR 0002. Um `Serializable<T>`, ou um `Durable<T>`, que o Dormant prova para tudo o que cruza um ponto de suspensão, manteria o programa como TS válido.

**Fora de escopo para o Dormant:** back-end LLVM, refcounting e coletor de ciclos, reimplementação das APIs do Node, monomorfização de genéricos e FFI.

## 6. Por que a tese "TS → nativo" foi descartada (inferência)

A alternativa "MiniTS → C nativo" aparece como opção rejeitada no ADR 0001: "seguro, mas sem ângulo próprio; não responde 'por que não usar o scriptc?'" (`docs/adr/0001-compiled-snapshot-thesis.md`). A leitura do repositório reforça a decisão:

- **Escala.** O scriptc já cobre esse espaço com muito mais recursos: cerca de 186 mil linhas de front-end em TS, 55 mil de back-end, 20 mil de IR e 116 mil de runtime C, com lançamentos quase diários (seções 2 e 4). Um projeto de semestre só repetiria uma fração disso.
- **A Entrega 4 seria redundante.** Num compilador TS → nativo, propagação de constantes, DCE, CSE e liveness são exatamente o que o LLVM O2 já faz melhor. O scriptc deixa isso ao LLVM (seção 2.3). No Dormant, a liveness deixa de ser um exercício e vira o produto: ela decide o que entra no snapshot.
- **O alvo C perdeu apoio no próprio exemplo.** O scriptc removeu o back-end C na v0.2.0 (`CHANGELOG.md:226`), e a citação "inspirado no scriptc, gerando C" deixou de descrever o projeto real.
- **A semântica de `number` é cara de reproduzir.** O scriptc usa f64 em todo lugar e mantém um analisador só para provar quando dá para usar inteiro (`ir/integer-ranges.ts`). O Dormant herda a mesma restrição de compatibilidade com o JS (ADR 0002) sem precisar resolver o problema de desempenho.
- **O que sobra do scriptc é método, não tese:** teste diferencial, diagnósticos com dica, `--emit` por estágio e uma IR bem tipada (seção 5).

## 7. Itens não verificados

- **Benchmarks de julho** (47 ms → 2,4 ms, ~100 MB → 4 MB, ~80 MB → 3 MB). Vêm de imprensa secundária ([runtimewire](https://runtimewire.com/article/vercel-scriptc-typescript-native-compiler-no-javascript-engine), [ITWriting, 2026-07-27](https://www.itwriting.com/blog/?p=12467)) e não aparecem na documentação atual.
- **"Largamente feito com IA"** e a recepção cética no Hacker News. Também vêm do [ITWriting](https://www.itwriting.com/blog/?p=12467).
- **Motivação da Vercel ou de Guillermo Rauch.** Não achei post nem declaração primária. O enquadramento "aposta do Rauch" é da imprensa.
- **Identidade de `cramforce`.** Que seja Malte Ubl, da Vercel, é memória minha.
- **Otimizações clássicas.** A afirmação de que não há passes próprios de propagação de constantes, CSE ou DCE vem de leitura parcial do código (nomes de arquivos e cabeçalhos), não de busca exaustiva.
- **Comparações com projetos parecidos** (memória, não pesquisadas aqui):
  - Static Hermes, da Meta: JS tipado compilado antecipadamente para nativo;
  - Porffor: JS compilado antecipadamente para Wasm/C;
  - AssemblyScript: linguagem parecida com TS, com tipos `i32`/`f64`, compilada para Wasm;
  - `bun build --compile` e `deno compile`: embutem o motor JS inteiro no binário.
- **Comportamento em execução.** Nada foi compilado nem executado. As afirmações sobre comportamento vêm da documentação, do CHANGELOG e da leitura do código.
