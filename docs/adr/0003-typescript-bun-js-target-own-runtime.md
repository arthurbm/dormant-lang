---
status: superseded by ADR-0005
---

# Implementação em TypeScript + Bun, alvo JavaScript, runtime próprio

O compilador é escrito em TypeScript e roda no Bun; gera JavaScript que roda no Bun junto com um runtime mínimo próprio (agendamento de `sleep`, persistência do estado em SQLite do Bun). O orçamento de aprendizado do semestre vai para compiladores e durabilidade, não para uma terceira curva (uma nova linguagem de implementação); compilador, runtime e testes compartilham uma só linguagem. O formato do código gerado se inspira no pi-durable (fases nomeadas + checkpoint com as variáveis vivas), como **referência inicial, aberta a pesquisa na Entrega 4** (regenerator, Roslyn, Kotlin coroutines, Rust async, XState, Vercel Workflow, Temporal).

## Considered Options

- **Rust**: defensável — o professor explora o MiniC em Rust (`nom`) nas aulas da turma de EC, e o MiniC tem parser, type checker, interpretador e TAC como referência; o plugin SWC do Vercel Workflow também é Rust. Rejeitado para o semestre porque o autor não tem experiência com Rust: a curva de aprendizado se somaria à de compiladores, com a Entrega 3 a ~2 semanas; reservado para um transform de TS real depois do semestre.
- **Go**: sem tipos-soma nem cultura de parser combinators, sem referência na disciplina.
- **Haskell** (SmartTS, turma de CC): fora da zona de conforto, sem TAC pronto.
- **Bend** como implementação ou alvo: ver ADR 0001.
- **C, WebAssembly ou RISC-V como alvo**: perdem sentido com a tese de durabilidade; o runtime durável é naturalmente JS/Bun.
- **Dormant como plugin de transpilador sobre TypeScript real (Babel, SWC, oxc)**: rejeitado para o semestre. O parser passa a ser do hospedeiro, o que elimina a Entrega 2. Os hospedeiros trabalham arquivo a arquivo e sem tipos, e o snapshot precisa de tipos para verificar que o que está vivo num Ponto de suspensão é serializável. O plugin SWC do Vercel Workflow dispensa tipos porque não transforma o corpo do workflow, que fica intacto para o replay. O único hospedeiro com tipos, o `tsc` com `customTransformers`, é a linha em JS; o TypeScript 7 saiu sem API. TypeScript real ainda traria de volta os cortes do ADR 0004 (closures, objetos, `await` em laços), e o plugin acabaria definindo um subconjunto mesmo assim. Ver `docs/research/compiler-landscape.md`, seção 10.4.

## Consequences

- A proposta ao professor declara abertura a Rust caso ele recomende; a decisão pode ser revista após a resposta dele, antes de começar o parser.
- A Entrega 4 precisa confirmar com o professor que JS é um alvo aceitável; a descida "de verdade" acontece em TAC → máquina de estados.
- Trocar o runtime próprio pelo do pi-durable depois do semestre deve exigir só mudar o gerador de código.
- No semestre, "plugin" significa só distribuição: um plugin do Bun (`onLoad`) que roda o compilador inteiro sobre o texto do arquivo. O plugin de transpilador sobre TypeScript real fica para depois, começando pelo Babel, e exige que TAC, CFG e liveness recebam a AST por uma fronteira estreita desde o início.
