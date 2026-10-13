# Roadmap

O Dormant é construído em quatro Entregas, uma por etapa do compilador. Cada Entrega tem um **compilador base**, que é uma etapa completa de compilador mesmo sem durabilidade, e uma **camada durável** por cima. Se a camada atrasar, o compilador base garante a Entrega. O design da linguagem está em [`design.md`](design.md).

| Entrega | Compilador base | Camada durável | Status |
|---|---|---|---|
| **1** | Definição da linguagem | Tese, posicionamento e regras específicas ([`design.md`](design.md)) | ✅ definida |
| **2** | Gramática; parser com combinators próprios; suíte de testes | Reconhecimento das diretivas `"use workflow"` / `"use step"` | ⏳ próxima |
| **3** | Verificador de tipos; interpretador | Regras de durabilidade no verificador; Manifesto de Steps conservador; interpretador com **replay** (log de Steps), demonstrado matando o processo no meio e retomando | — |
| **4** | Geração de TAC; CFG; propagação de constantes, eliminação de código morto, eliminação de subexpressões comuns | Liveness nos pontos de suspensão; geração da máquina de estados com **snapshot**; runtime mínimo no Bun com SQLite; Layout de Checkpoints com diagnóstico de compatibilidade entre versões; Manifesto de Steps refinado por propagação de constantes e DCE | — |

Ao final, o projeto terá implementado **as duas estratégias de retomada** (replay no interpretador, snapshot no compilador), o que permite compará-las com base em implementação própria.

## Riscos

| Risco | Mitigação |
|---|---|
| Transformação em máquina de estados mais difícil que o previsto | `await` só no nível de comando na v1; o compilador base da Entrega 4 (TAC + otimizações) independe dela |
| Semântica numérica divergindo do JavaScript | Decisão explícita na Entrega 2, coberta por testes diferenciais |
| Alvo JavaScript considerado alto nível demais | A descida relevante acontece em TAC → máquina de estados |
| Camada durável crescer além do prazo | O Manifesto de Steps e o diagnóstico de compatibilidade reaproveitam análises já feitas (Steps no verificador, liveness no TAC); se a Entrega atrasar, são os primeiros a sair, sem afetar a máquina de estados |

## Depois da v1

- Closures: primeiro proibindo apenas closures vivas num ponto de suspensão, depois defuncionalização ([ADR 0004](adr/0004-v1-scope-cuts.md)).
- Migração assistida pelo compilador: a v1 só detecta quando uma mudança no Workflow quebra o Layout de Checkpoints; depois, gerar o esqueleto da Migração e aplicá-la às execuções em andamento ([`design.md`](design.md), Seção 4.2).
- Um texto explicando replay × snapshot a partir da implementação própria das duas estratégias, com foco no caso em que o snapshot compensa: execuções que atravessam deploys ([`design.md`](design.md), Seção 3.4).
- Manifesto de Steps com argumentos: além de *quais* Steps um Workflow chama, restringir *com que* valores (por exemplo, por propagação de constantes).
- Aplicar a parte de análise e transformação (liveness + geração da máquina de estados) a TypeScript real, como plugin de um transpilador (como o plugin SWC do Vercel Workflow), gerando código para um runtime de snapshot. Para isso, essa parte fica separada da sintaxe desde o início.
