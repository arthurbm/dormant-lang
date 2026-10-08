---
status: accepted
---

# Implementação em TypeScript + Bun, alvo JavaScript, runtime próprio

O compilador é escrito em TypeScript e roda no Bun; gera JavaScript que roda no Bun junto com um runtime mínimo próprio (agendamento de `sleep`, persistência do estado em SQLite do Bun). O orçamento de aprendizado do semestre vai para compiladores e durabilidade, não para uma terceira curva (uma nova linguagem de implementação); compilador, runtime e testes compartilham uma só linguagem. O formato do código gerado se inspira no pi-durable (fases nomeadas + checkpoint com as variáveis vivas), como **referência inicial, aberta a pesquisa na Entrega 4** (regenerator, Roslyn, Kotlin coroutines, Rust async, XState, Vercel Workflow, Temporal).

## Considered Options

- **Rust**: defensável — o professor explora o MiniC em Rust (`nom`) nas aulas da turma de EC, e o MiniC tem parser, type checker, interpretador e TAC como referência; o plugin SWC do Vercel Workflow também é Rust. Rejeitado para o semestre porque o autor não tem experiência com Rust: a curva de aprendizado se somaria à de compiladores, com a Entrega 3 a ~2 semanas; reservado para um transform de TS real depois do semestre.
- **Go**: sem tipos-soma nem cultura de parser combinators, sem referência na disciplina.
- **Haskell** (SmartTS, turma de CC): fora da zona de conforto, sem TAC pronto.
- **Bend** como implementação ou alvo: ver ADR 0001.
- **C, WebAssembly ou RISC-V como alvo**: perdem sentido com a tese de durabilidade; o runtime durável é naturalmente JS/Bun.

## Consequences

- A proposta ao professor declara abertura a Rust caso ele recomende; a decisão pode ser revista após a resposta dele, antes de começar o parser.
- A Entrega 4 precisa confirmar com o professor que JS é um alvo aceitável; a descida "de verdade" acontece em TAC → máquina de estados.
- Trocar o runtime próprio pelo do pi-durable depois do semestre deve exigir só mudar o gerador de código.
