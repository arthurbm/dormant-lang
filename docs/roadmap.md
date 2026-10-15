# Roadmap

O Dormant é construído em quatro Entregas, uma por etapa do compilador. Cada Entrega tem um **compilador base**, que é uma etapa completa de compilador mesmo sem durabilidade, e uma **camada durável** por cima. Se a camada atrasar, o compilador base garante a Entrega. O design da linguagem está em [`design.md`](design.md).

| Entrega | Compilador base | Camada durável | Status |
|---|---|---|---|
| **1** | Definição da linguagem | Tese, posicionamento e regras específicas ([`design.md`](design.md)) | ✅ definida |
| **2** | Gramática ([`grammar.md`](grammar.md)); parser em `nom` para todo o subconjunto da v1, que devolve a AST e rejeita entrada que sobra; `dormant parse arquivo.ts` imprime a AST; suíte `cargo test` com programas válidos (AST esperada) e inválidos | Reconhecimento das diretivas `"use workflow"` / `"use step"` | ⏳ próxima |
| **3** | Verificador de tipos; interpretador | Regras de durabilidade no verificador; Manifesto de Steps conservador; interpretador com **replay** (log de Steps), demonstrado matando o processo no meio e retomando | — |
| **4** | Geração de TAC; CFG; propagação de constantes, eliminação de código morto, eliminação de subexpressões comuns | Liveness nos pontos de suspensão; geração da máquina de estados com **snapshot**, em JavaScript gerado a partir do TAC otimizado (`dormant build arquivo.ts`); Runtime mínimo em JavaScript, independente do motor JavaScript; Layout de Checkpoints com diagnóstico de compatibilidade entre versões; Manifesto de Steps refinado por propagação de constantes e DCE | — |

Ao final, o projeto terá implementado **as duas estratégias de retomada** (replay no interpretador, snapshot no compilador), o que permite compará-las com base em implementação própria.

## Riscos

| Risco | Mitigação |
|---|---|
| Transformação em máquina de estados mais difícil que o previsto | `await` só no nível de comando na v1; o compilador base da Entrega 4 (TAC + otimizações) independe dela |
| Semântica numérica divergindo do JavaScript | Decisão explícita na Entrega 2, coberta por testes diferenciais |
| Curva do Rust somada à de compiladores | Parser em `nom`, como o MiniC, e Entrega 2 com escopo mínimo (parser, AST, testes); se faltar tempo, as mensagens de erro com linha:coluna são o primeiro corte, nunca os testes |
| Alvo JavaScript considerado alto nível demais | A descida relevante acontece em TAC → máquina de estados, e o JavaScript é gerado a partir do TAC otimizado, não da AST. Se não bastar, a Opção B (bytecode próprio executado por uma VM em Rust) entra como segundo back-end sobre o mesmo TAC e substitui o back-end JavaScript na Entrega 4; o back-end JavaScript vai para depois do semestre ([ADR 0005](adr/0005-rust-compiler-js-target.md)) |
| Camada durável crescer além do prazo | O Manifesto de Steps e o diagnóstico de compatibilidade reaproveitam análises já feitas (Steps no verificador, liveness no TAC); se a Entrega atrasar, são os primeiros a sair, sem afetar a máquina de estados |

## Depois da v1

- Closures: primeiro proibindo apenas closures vivas num ponto de suspensão, depois defuncionalização ([ADR 0004](adr/0004-v1-scope-cuts.md)).
- Migração assistida pelo compilador: a v1 só detecta quando uma mudança no Workflow quebra o Layout de Checkpoints; depois, gerar o esqueleto da Migração e aplicá-la às execuções em andamento ([`design.md`](design.md), Seção 4.2).
- Um texto explicando replay × snapshot a partir da implementação própria das duas estratégias, com foco no caso em que o snapshot compensa: execuções que atravessam deploys ([`design.md`](design.md), Seção 3.4).
- Manifesto de Steps com argumentos: além de *quais* Steps um Workflow chama, restringir *com que* valores (por exemplo, por propagação de constantes).
- Adaptadores de plugin para ferramentas de build (Bun com `onLoad`; Vite e Rolldown com `transform`, na convenção do Rollup), que compilam os Workflows na importação, sem etapa de build separada. Com o compilador em Rust, chamam-no por Wasm, FFI ou subprocesso; no semestre, só a CLI.
- Aplicar a parte de análise e transformação (liveness + geração da máquina de estados) a TypeScript real, como plugin de um transpilador (como o plugin SWC do Vercel Workflow), gerando código para um Runtime de snapshot. Para isso, essa parte fica separada da sintaxe desde o início ([`design.md`](design.md), Seção 7). Como o compilador é Rust, o caminho direto é SWC ou oxc, que recebem o crate de análise e geração sem port; o React Compiler é o modelo (baixa a AST do Babel para uma IR própria com CFG, e o port para Rust vai para o oxc). O obstáculo central é a falta de tipos nesses hospedeiros (ver [ADR 0005](adr/0005-rust-compiler-js-target.md)).
