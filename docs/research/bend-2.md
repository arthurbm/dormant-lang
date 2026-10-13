# Bend 2: o que é e por que foi descartado para o Dormant

Pesquisa sobre o repositório [bendlang/bend](https://github.com/bendlang/bend) no commit `0e14aa8b3082e68f419320b2c785762de1d0577b`. A versão mais recente nesse commit é a 2.0.36.

**Como ler as referências**
- Caminhos como `guide/GUIDE.md:113` apontam para esse commit. Para abrir no GitHub, use `https://github.com/bendlang/bend/blob/0e14aa8b3082e68f419320b2c785762de1d0577b/<caminho>#L<linha>`.
- Os posts do Victor Taelin no X foram lidos pelo espelho público `api.fxtwitter.com`, porque o X exige login. O link citado é sempre o original no X.
- Itens tirados de memória ou de fonte secundária estão marcados como **(não verificado)**.
- As seções 6 e 7 misturam fatos citados com inferência, e o texto deixa explícito onde é inferência.

## 1. O que é o Bend 2

### 1.1 Proposta

O Bend 2 se apresenta como "a fast language that blocks AI mistakes via proof", com o lema "C speed · CUDA parallelism · Lean proofs · Python syntax" ([bend-lang.com](https://bend-lang.com)). O argumento central do README:

> "In the post-AGI economy, humans will eventually stop writing and reading code, but we still need an ambiguity-free language to communicate our intents to the AIs building the world around us." (`README.md:3-5`)

O guia resume a linguagem como sintaxe "Python-shaped", com semântica "closer to Haskell / Lean" e controle de recursos "like Rust" (`guide/GUIDE.md:11-13`). É uma linguagem *pura*, isto é, sem efeitos colaterais fora de um tipo especial. Os efeitos ficam numa mônada `IO`, como em Haskell (`guide/GUIDE.md:26-29`).

### 1.2 Histórico e organização

**Lançamento.** O Bend 2.0.0 saiu em 2026-09-17, com "a new type checker (an affine dependent type theory), a new compiler to C, Metal, CUDA and JavaScript, and a new runtime" (`CHANGELOG.md:658-661`). O anúncio no X é do mesmo dia ([post de lançamento](https://x.com/VictorTaelin/status/2100681226143092875)).

**Não é o Bend de 2024.** O Bend 1 (maio de 2024, em Rust, sobre o runtime HVM2) foi substituído. Pelo README, "Bend 2 is a new language. Bend 1 programs and HVM do not carry over" (`README.md:223`).

Na véspera do lançamento, Taelin explicou a ruptura ([post](https://x.com/VictorTaelin/status/2100374221671051472)): "Bend2 doesn't have interaction nets, because I could not make them as fast as lower-order variants on everyday hardware […] It is closer to C than it is to Haskell". *Interaction nets* são o modelo de grafos em que o HVM se baseava.

**Repositórios.** Os antigos ficaram na organização HigherOrderCO com descrições de legado (consulta via `gh api orgs/HigherOrderCO/repos`):
- `Bend1` ("superseded by github.com/bendlang/bend");
- `HVM1`, `HVM2` e `HVM3` (este último em Haskell);
- `Bend2`, que só aponta para o repositório novo.

**Licença.** Apache-2.0 (arquivo `LICENSE`; metadado do GitHub).

**Implementação.** É TypeScript rodando no Bun, a mesma pilha do Dormant:
- `bend2/bend.ts` é "the language (parser, theory, checker)" e é "human-written: do not edit it" (`AGENTS.md:5-6`);
- `bend2/comp.ts` é "the one compiler", com o runtime C, o emissor C e o emissor JS (`AGENTS.md:6-8`);
- o README diz que `bun bend2/main.ts` roda o compilador a partir de um checkout (`README.md:116`).

**Equipe.** Criado por Victor Taelin e mantido por uma equipe pequena (`README.md:265-274`). No post da véspera do lançamento, Taelin fala em "4 humans (and 25 non-humans!)".

### 1.3 Mudanças recentes (2.0.35 → 2.0.36)

- **Nova versão.** Saiu a 2.0.36 em 2026-10-07 (`CHANGELOG.md:6`). Na conta do CHANGELOG, são 37 versões (2.0.0 a 2.0.36) em 20 dias.
- **Bender virou BendAI.** O agente de provas pago passou a se chamar BendAI: "HOC's proving agent is now named BendAI" (`CHANGELOG.md:8`, [bend-lang.com/bendai](https://bend-lang.com/bendai)).
- **Novo limite de runtime.** O WONTFIX ganhou um limite: "A value with more than 2^24-1 live copies fail-stops on the C lane" (`WONTFIX.txt:129`).
- **Issues abertas.** 127 no commit inspecionado (metadado do GitHub).
- **Bend escrito em Bend.** Um PR externo propôs um compilador escrito no próprio Bend, isto é, *self-hosted* ([PR #1207](https://github.com/bendlang/bend/pull/1207), de 2026-09-29). Foi fechado em 2026-10-01 por um mantenedor: "this is something we'll do ourselves for this repo". Ver seção 5.
- **Pedido de alvo WebAssembly.** Foi aberta uma issue pedindo "A library target for the browser: pure defs as WebAssembly" ([#1352](https://github.com/bendlang/bend/issues/1352)).

## 2. Sistema de tipos e restrições da linguagem

### 2.1 Tipos dependentes e afins

**Tipos dependentes.** Num sistema de tipos dependentes, tipos podem depender de valores. Com isso, uma proposição matemática vira um tipo, e uma prova vira um programa desse tipo, como em Lean e Rocq. No Bend: "Since types are terms, a def may return a `Type` […] which is all dependent types are" (`guide/GUIDE.md:343-344`).

A teoria se chama BendTT e está descrita em `paper/BendTT.pdf`. Ela tem um só universo, `Type : Type`, e separa dois modos de checagem: código que roda é checado *live*, e tipos e provas são checados *dead* (`guide/GUIDE.md:655-659`). Um núcleo pequeno, formalizado em Lean (`bend2/bendtt.lean`), recheca as provas com `--verdict` (`guide/GUIDE.md:328-332`).

**Tipos afins.** Num sistema afim, cada variável pode ser usada no máximo uma vez. O guia: "Bend, by default, is *affine*, meaning variables must be used, at most, once" (`guide/GUIDE.md:55`). As *quantidades* controlam isso por variável (`guide/GUIDE.md:205-207`):
- `-x`: apagada em tempo de execução;
- `x`: afim, o padrão;
- `+x`: reutilizável, e só vale para tipos `Data`.

Funções, arrays e handles de IO são `Type` e por isso "can never be copied" (`guide/GUIDE.md:221-222`).

### 2.2 Restrições que pesam para escrever um compilador em Bend

Todas são declaradas pelo próprio projeto:

| Restrição | Fonte |
|---|---|
| Closures são afins: "A closure is affine: it can be called at most once, even when everything it captures is `Data`. Only top-level definitions can be called freely." | `guide/GUIDE.md:83-85`; `README.md:227` |
| "Termination is mandatory and mutual recursion is not allowed." A recomendação é usar um argumento de combustível (*fuel*) `Nat` para laços externos e fundir funções mutuamente recursivas numa só def com um argumento seletor | `guide/GUIDE.md:113-117` |
| O checker rejeita referência adiante a uma def ainda sem corpo, "so mutual recursion cannot bypass the wall" | `bend2/bend.ts:3788-3790` |
| A checagem de terminação lê os argumentos da esquerda para a direita, e o argumento que diminui deve vir primeiro | `guide/GUIDE.md:105-110`; `WONTFIX.txt:30` |
| `@unsafe` desliga a checagem, mas o def "falls outside Bend's proof guarantees" | `guide/GUIDE.md:117-120`; `README.md:228` |
| *Templates* (`~f`, substituição em tempo de compilação) podem ser chamados várias vezes, mas o argumento "must be closed" e "A template may call only templates declared above it" | `guide/GUIDE.md:272-276` |
| Quase nenhuma inferência de tipos: "Everything is annotated and nothing is inferred, so code is verbose" | `guide/GUIDE.md:61-65`; `README.md:224` |
| Não há `if`: um desvio é um `match` em `True{}`/`False{}` | `guide/GUIDE.md:110-111`; `README.md:230` |
| `match` só inspeciona parâmetros ou variáveis de padrão, nunca um valor computado | `guide/GUIDE.md:123-126`; `README.md:229` |
| Números só `Nat`, `U32` e `F32`. Strings são listas ligadas de caracteres, e "text processing is slow" | `README.md:231`, `README.md:233` |
| Sem type classes ou traits, sem tactics, sem framework de testes | `README.md:225-226`, `README.md:255` |

## 3. Pipeline do compilador e runtime

### 3.1 Front-end (`bend2/bend.ts`, 3.882 linhas)

O arquivo se declara "99% human-designed and audited" e "Bend's trusted kernel, including interpreter and checker" (`bend2/bend.ts:1-2`). A gramática está documentada no cabeçalho a partir de `bend2/bend.ts:8`. As seções do arquivo, na ordem:
- `Parse` (`:1500`), com `Binders`, `Patt` e `Term`;
- `Book` (`:2419`), a tabela de definições do programa;
- `Flatten` (`:2589`), que desaçucara o pattern matching aninhado;
- `WNF` (`:2774`) e `SNF` (`:3000`), avaliação até forma normal fraca e forte;
- `Compare` (`:3046`), a conversão, que decide se dois tipos são iguais;
- `Check` (`:3208`);
- `Valid` (`:3777`).

A checagem é bidirecional e feita em uma única passada linear (`AGENTS.md:3-4`).

### 3.2 Back-end (`bend2/comp.ts`, 6.340 linhas)

O cabeçalho avisa: "this file was mostly written by AI's […] bugs ARE expected" (`bend2/comp.ts:1-3`). O README diz que "The compiler (not kernel) is 99% AI-written and not yet fully audited" (`README.md:246`). Seções relevantes:
- `Ty` (`:822`) e `Lay` (`:919`): escolha de representação (`w32`, `w64` ou `box`);
- `Mat` (`:1070`);
- `Seg` (`:1510`);
- `ANF` (`:1939`);
- `Emit` (`:2039`);
- `Compile` (`:2728`);
- `JS` (`:2894`);
- `RuntimeC` (`:3178`) e `RuntimeJs` (`:5952`).

A passada `ANF` (*A-normal form*, forma em que todo resultado intermediário ganha um nome) é a parte mais próxima de um TAC (*three-address code*, código de três endereços). Nela, "A statement in normal form is a fork, a cut, a let of a value, or a tail" (`bend2/comp.ts:1942`).

Não há eliminação de subexpressões comuns, por decisão de projeto: "No common-subexpression elimination (#812)" (`WONTFIX.txt:34`).

### 3.3 Alvos e runtime

**Alvos.** "Targets are C, Metal, CUDA and JavaScript; Lua, Luau and Python are planned" (`README.md:237`). Detalhes:
- O C é só para clang (`WONTFIX.txt:27`).
- Cada programa vira um único arquivo C, sem compilação separada (`README.md:242`).
- O alvo JS "ignores all that and just runs sequentially" (`guide/GUIDE.md:170`, `README.md:238`).

**Runtime (BendRT, `paper/BendRT.pdf`)** (`guide/GUIDE.md:641-652`):
- não há coletor de lixo: como os valores são afins, um `match` libera o nó que abre;
- só valores `+` têm contagem de referências;
- não há pilha C: "each def compiles to a segment of a flat state machine, a call is a jump";
- uma chamada paralela cria tarefas de *fork-join* (dividir o trabalho em duas chamadas e juntar os resultados);
- `f!(x)` manda a chamada para a GPU (`guide/GUIDE.md:159-162`).

**Testes.** São 1.617 arquivos `.bend` em `tests/`, contados no commit. Cada teste termina com as linhas `#|` que sua execução deve imprimir (`AGENTS.md:8-9`).

## 4. LAWS/PROOF e BendHub

### 4.1 LAWS e PROOF

Uma `law` declara um fato, e uma `def` com o mesmo nome é a prova (`guide/GUIDE.md:279-303`). Por convenção, cada projeto tem dois arquivos (`guide/GUIDE.md:319-326`):
- `LAWS.bend` é escrito pelo humano e "the AI does not touch it";
- `PROOF.bend` é escrito pela IA, junto com o código.

`bend PROOF.bend` funciona como portão de commit: imprime "ALL PROOFS CHECK" só quando toda lei vale. O README resume: "`LAWS.bend` is `AGENTS.md` backed by **proof**" (`README.md:102`).

Não há *tactics*, isto é, comandos que constroem provas automaticamente: "a proposition is a type, and a proof is a def of that type" (`guide/GUIDE.md:334`).

### 4.2 BendHub

O [BendHub](https://hub.bend-lang.com) é o registro de pacotes, "where Bend packages live".

**Publicação** (`guide/GUIDE.md:529-542`):
- `bend main.bend --publish` sobe o arquivo e tudo o que ele importa, endereçado por conteúdo (`import 0x<hash>/main.bend`);
- nomes e versões exigem `bend login` e `--publish <name>@<version>`;
- a publicação é "public and permanent";
- um pacote sem `LICENSE` fica sob MIT-0.

**Leilões de nomes.** Nomes com menos de 12 caracteres são leiloados em "Bend Credits" ([hub.bend-lang.com/auctions](https://hub.bend-lang.com/auctions)).

**BendAI.** Ao lado do hub está o BendAI, um agente pago de provas. Hoje é descrito como "a thin harness around models from Anthropic, OpenAI and others", e o SupGen (prover simbólico) virá "in future updates" ([bend-lang.com/bendai](https://bend-lang.com/bendai)).

## 5. Roadmap

### 5.1 Oficial, no repositório

O `WONTFIX.txt` tem uma seção "SOON" (`WONTFIX.txt:135-155`) com:
- U64 e F64, que a equipe fará e para os quais não aceita PRs;
- F32 computável no checker;
- continuações para recursão profunda no alvo JS;
- um alvo de biblioteca nativa para defs puras.

O README lista Lua, Luau e Python como alvos planejados (`README.md:237`).

### 5.2 O post de Taelin de 2026-09-29: reescrever tudo em Bend

O post ([X](https://x.com/VictorTaelin/status/2104949395770712276)) anuncia uma captação de US$ 20 milhões e dois times internos:
- BendCore: linguagem, compilador e ecossistema;
- SupGen: "internal symbolic AI R&D (goal: GD-free GPT-3)".

Planos para o **proof kernel**:
- "fully rewritten in Bend (no more TypeScript)";
- "live λ contraction";
- "funext + univalence (most likely HOTT-like, NOT cubical)";
- talvez trazer de volta os *self types* do Kind.

Planos para o **compilador**:
- "fully rewritten in Bend (no more TypeScript)";
- alvos "HTML apps (WASM + WebGPU) and AMD GPUs";
- clusters de GPUs;
- arrays atômicos compartilhados melhores;
- "more parallelism options (not just fork/join)".

Planos para o **ecossistema**: motor 3D, framework de IA ("llama.bend"), bibliotecas de criptografia e rede, e "all good parts of mathlib, npm, hackage". A tese do post: "It is a language evolved by AI, secured by math". O mesmo post diz que o BendHub foi publicado e que cerca de 400 IPs distintos usam o comando `bend` por dia (números do autor, ver seção 8).

**Relação com o PR #1207.** O mantenedor que fechou o PR #1207 confirma a intenção de reescrever o compilador em Bend: "something we'll do ourselves" ([PR #1207](https://github.com/bendlang/bend/pull/1207)). Segundo o autor do PR, a versão self-hosted feita por um agente de IA chegou a "3x slower than typescript and 100% conformance. About 15k lines".

### 5.3 Contexto anterior

Em 2025-11-03, Taelin explicou o abandono do HVM2 ([post](https://x.com/VictorTaelin/status/1985423273304473788)). Entre os motivos: "full RTX 4090 to beat 1-core OCaml / JavaScript" e "turns out most devs can't think recursively". No mesmo post, colocou o SupGen (síntese de programas por busca simbólica) como "the main novelty behind Bend2".

Em 2026-09-04, listou o que foi cortado do lançamento: SupGen, U64/F64, saída HTML5, outros alvos e fibras concorrentes ([post](https://x.com/VictorTaelin/status/2095876946080968838)).

## 6. Por que foi descartado para o Dormant

A decisão está registrada no ADR 0001 (`docs/adr/0001-compiled-snapshot-thesis.md`) e é referenciada no ADR 0003. Esta seção detalha os motivos com as fontes.

### 6.1 Como alvo de compilação (TS-subset → Bend)

- **Usaria só a velocidade.** O que distingue o Bend são as leis e provas (`README.md:3-12`; seção 4). O Dormant não as aproveitaria: a tese dele é snapshot de variáveis vivas, não verificação formal.
- **O paralelismo não ajudaria.** O ganho de desempenho do Bend vem de chamadas paralelas fork-join (`guide/GUIDE.md:128-168`), e a v1 do Dormant não tem Steps em paralelo (ADR 0004). O alvo JS do Bend, o único que conversaria com o runtime durável em Bun do Dormant (ADR 0003), roda sequencialmente (`guide/GUIDE.md:170`).
- **Instabilidade.** São 37 versões em 20 dias (seção 1.3), o compilador é "99% AI-written and not yet fully audited" (`README.md:246`), e o README avisa "Bend is young" (`README.md:141`). O roadmap ainda prevê reescrever compilador e kernel do zero (seção 5.2). Um projeto de 7 semanas teria de congelar uma versão e conviver com bugs do alvo.

### 6.2 Como linguagem de implementação do compilador do Dormant

- **Recursão mútua proibida** (`guide/GUIDE.md:113-117`; `bend2/bend.ts:3788-3790`). Um parser descendente recursivo é naturalmente mutuamente recursivo (expressão → termo → fator → `(` expressão `)`). O mesmo vale para o par `checkStmt`/`checkExpr` do type checker. Contornar isso exige fundir tudo numa def com seletor ou marcar `@unsafe`, o que sai das garantias de prova (`guide/GUIDE.md:117-120`).
- **Closures afins quebram parser combinators.** *Parser combinators* são funções que recebem parsers (closures) e os combinam. `many(p)`, por exemplo, chama `p` repetidamente. No Bend, uma closure só pode ser chamada uma vez (`guide/GUIDE.md:83-85`). Os templates `~` podem ser chamados várias vezes, mas não capturam variáveis locais e só chamam templates declarados acima (`guide/GUIDE.md:272-276`). **Inferência:** isso impede gramáticas recursivas escritas como combinators.
- **Terminação obrigatória** (`guide/GUIDE.md:113`). **Inferência:** o interpretador da Entrega 3 executa `while` de um programa arbitrário, que pode não terminar. Em Bend, ele precisaria de um argumento de combustível ou de `@unsafe`.
- **Atrito geral.**
  - Sem inferência de tipos (`README.md:224`), sem `if` (`README.md:230`), sem `match` sobre valor computado (`README.md:229`).
  - Strings como listas ligadas (`README.md:233`), ruins para um lexer.
  - Sem framework de testes (`README.md:255`).
  - Mensagens de erro concisas, sem debugger nem REPL (`README.md:253`).
- **Curva de aprendizado.** O ADR 0003 reserva o orçamento do semestre para compiladores e durabilidade, não para uma nova linguagem de implementação. O próprio Bend é implementado em TypeScript sobre Bun (seção 1.2).

## 7. Ideias que ficam para o futuro

1. **Provar que uma otimização preserva a semântica.** A demo `demos/proof_typed_eval` define uma linguagem de expressões *intrinsecamente tipada*: `Expr<t>` só representa programas bem tipados, graças a uma equação de tipo em cada construtor, o "Ford trick" (`demos/proof_typed_eval/main.bend:1-7`). A demo também tem um avaliador sem caso de erro (`:65`) e um otimizador de constant folding (`:112`).

   `LAWS.bend` declara duas leis, e `PROOF.bend` (59 linhas) prova as duas:
   - `opt_sound`: "`TE.eval(t, TE.opt(t, e)) == TE.eval(t, e)`" (`demos/proof_typed_eval/LAWS.bend:9-12`);
   - `opt_complete` (`:16-19`).

   **Ideia:** modelar em Bend um fragmento do TAC do Dormant e provar algo semelhante para as otimizações da Entrega 4 (propagação de constantes, eliminação de código morto). Uma versão mais ambiciosa provaria que "salvar só as variáveis vivas e retomar" equivale a executar sem interrupção, ou seja, a correção da liveness usada no snapshot. É extensão pós-semestre e não faz parte da v1.
2. **BendRT como referência para a Entrega 4.** O runtime compila cada def em "a segment of a flat state machine, a call is a jump" (`guide/GUIDE.md:649-651`). É uma descida para máquina de estados parecida com a que o Dormant faz para Workflows, embora com outro objetivo (eliminar a pilha C, não persistir estado). As seções `Seg` e `ANF` de `bend2/comp.ts` (`:1510`, `:1939`) podem ser lidas como material de pesquisa.
3. **Bend como alvo secundário.** Só faria sentido depois de existir paralelismo de Steps no Dormant e de o Bend estabilizar. Fica registrado, sem prioridade.

## 8. Itens não verificados

- **Benchmarks.** Tanto os do próprio projeto ("nearly as fast as C", "up to a hundred times faster" na GPU, checker "a second at most"; [bend-lang.com](https://bend-lang.com), `bench/`) quanto os de terceiros. Uma análise externa diz que "no number here shows Bend faster than Rust" ([akitaonrails.com](https://akitaonrails.com/en/2026/09/19/new-ai-language-just-released-bend-2/)), mas é fonte secundária e não foi conferida. Nada foi compilado nem executado nesta pesquisa.
- **Posts do X.** Foram lidos pelo espelho `api.fxtwitter.com` e não diretamente no x.com. O texto pode diferir de uma eventual edição posterior do post.
- **Números de adoção do post de 2026-09-29:** cerca de 400 IPs por dia, "more installs than Zigs on Homebrew", cerca de US$ 3,5 mil em Bend Credits vendidos. São números do autor, sem verificação independente.
- **Repercussão do lançamento:** pontuação no Hacker News, vídeo da Fireship etc. Conhecida só por fonte secundária.
- **Conteúdo do BendHub:** quais pacotes existem. A lista é renderizada por JavaScript e não foi inspecionada.
- **HVM4, HVM5 e SupGen:** o estado atual. O HVM4 tem repositório público sem licença (`HigherOrderCO/HVM4`). O "hvm5 done" vem de um post de 2026-06-06 ([X](https://x.com/VictorTaelin/status/2063392469195338129)), e não encontrei repositório do HVM5. O SupGen não é público.
- **Restrições inferidas.** As restrições ao uso de Bend como linguagem de implementação marcadas como inferência na seção 6.2 decorrem das regras citadas, mas não foram testadas com código.
