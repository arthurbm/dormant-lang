# Roadmap

O Dormant é construído em quatro Entregas, uma por etapa do compilador. Cada Entrega tem um **núcleo**, que é uma etapa completa de compilador mesmo sem durabilidade, e uma **camada durável** por cima. Se a camada atrasar, o núcleo garante a Entrega. O design da linguagem está em [`design.md`](design.md).

| Entrega | Núcleo | Camada durável | Status |
|---|---|---|---|
| **1** | Definição da linguagem | Tese, posicionamento e regras específicas ([`design.md`](design.md)) | ✅ definida |
| **2** | Gramática; parser com combinators próprios; suíte de testes | Reconhecimento das diretivas `"use workflow"` / `"use step"` | ⏳ próxima |
| **3** | Verificador de tipos; interpretador | Regras de durabilidade no verificador; interpretador com **replay** (log de Steps), demonstrado matando o processo no meio e retomando | — |
| **4** | Geração de TAC; CFG; propagação de constantes, eliminação de código morto, eliminação de subexpressões comuns | Liveness nos pontos de suspensão; geração da máquina de estados com **snapshot**; runtime mínimo no Bun com SQLite | — |

Ao final, o projeto terá implementado **as duas estratégias de retomada** (replay no interpretador, snapshot no compilador), o que permite compará-las com base em implementação própria.

## Riscos

| Risco | Mitigação |
|---|---|
| Transformação em máquina de estados mais difícil que o previsto | `await` só no nível de comando na v1; o núcleo da Entrega 4 (TAC + otimizações) independe dela |
| Semântica numérica divergindo do JavaScript | Decisão explícita na Entrega 2, coberta por testes diferenciais |
| Alvo JavaScript considerado alto nível demais | A descida relevante acontece em TAC → máquina de estados |

## Depois da v1

- Closures: primeiro proibindo apenas closures vivas num ponto de suspensão, depois defuncionalização ([ADR 0004](adr/0004-v1-scope-cuts.md)).
- Versionamento assistido pelo compilador: avisar quando uma mudança no Workflow altera as variáveis vivas de um ponto de suspensão e exige uma migração.
- Aplicar o núcleo (liveness + geração da máquina de estados) a TypeScript real, como plugin de um transpilador.
