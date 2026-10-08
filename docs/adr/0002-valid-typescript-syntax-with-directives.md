---
status: accepted
---

# Sintaxe: subconjunto de TypeScript válido, com diretivas

Todo programa Dormant é TypeScript válido: workflows e steps são marcados pelas diretivas `"use workflow"` e `"use step"` (como no Vercel Workflow) e `sleep()`, `int`, `float` etc. vêm de um prelúdio declarado em TS. O Dormant é, portanto, um **subconjunto sintático de TypeScript com semântica estendida** (o mesmo texto, compilado pelo Dormant, se torna durável), não um superset. A meta é compatibilidade em três níveis: sintático (o parser do TS aceita), de tipos (o `tsc` aceita, via prelúdio) e semântico (sem reinícios, rodar no Bun dá o mesmo resultado — base dos testes diferenciais).

## Considered Options

- **Palavras-chave novas (`workflow function`, `step`)**: mais explícito, mas perde editores, `tsc` e execução no Bun, e cada palavra nova vira decisão a justificar.
- **Sintaxe totalmente nova**: muito ambicioso sem ganho para a tese, que está na semântica (como pausar e retomar), não na sintaxe.
- **Superset de TypeScript**: exigiria implementar o TS inteiro.

## Consequences

- O nível 3 de compatibilidade restringe a semântica dos números: `int`/`float` não podem divergir do `number` do JS (ex.: `7 / 2`). A regra exata está pendente (proposta: `/` sempre devolve `float` e a divisão inteira é `idiv(a, b)` do prelúdio).
- A mesma sintaxe serve para o caminho pós-semestre (um transform de TypeScript real), sem retrabalho.
