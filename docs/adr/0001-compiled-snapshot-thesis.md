---
status: accepted
---

# Snapshot compilado como tese do projeto

O Dormant é uma linguagem para workflows duráveis cuja tese é: **o compilador gera, a partir de código linear, a máquina de estados retomável por snapshot**, salvando em cada ponto de suspensão apenas as variáveis vivas (decididas por análise de liveness sobre o TAC). Os sistemas populares (Temporal, Vercel Workflow, Cloudflare Workflows, Restate, Effect Workflow) retomam por replay e exigem determinismo; o pi-durable usa snapshot, mas obriga o programador a escrever as fases e o checkpoint à mão. O Dormant ocupa o quadrante "código linear + snapshot", que nenhum deles ocupa, e faz da análise de fluxo de dados da Entrega 4 o coração do produto em vez de um apêndice.

## Considered Options

- **MiniTS → C nativo (à la scriptc)**: seguro, mas sem ângulo próprio; não responde "por que não usar o scriptc?".
- **MiniTS → Bend 2 como alvo**: usa o Bend só como back-end rápido e ignora o que o diferencia (leis + provas); alvo instável (dezenas de versões em semanas).
- **Compilador escrito em Bend**: inviável no prazo (recursão mútua proibida, closures afins quebram parser combinators, terminação obrigatória, linguagem lançada em 17/09/2026).
- **Scripts de agente com permissões checadas; erros tipados; tipos-prova (gdp-ts); linguagem reativa; Gleam → BEAM**: avaliados; ou a Entrega 4 (TAC + data-flow) encaixa mal, ou a teoria de tipos fica profunda demais para o prazo.

## Consequences

- A Entrega 3 implementa **replay** no interpretador e a Entrega 4 implementa **snapshot** no compilador, permitindo comparar as duas estratégias com implementação própria.
- Snapshot exige que todo valor vivo num ponto de suspensão seja serializável; isso motiva os cortes da v1 (ver ADR 0004).
- Para agentes de IA, o escopo é a **orquestração**, não o loop do agente: o estado de um loop de agente já é o transcript serializado (o OpenCode 2 o torna durável por event sourcing, sem liveness), enquanto o código que orquestra agentes guarda estado em variáveis locais. Ver `docs/research/opencode-2.md`.
- Sistemas de replay também passaram a usar snapshots de VM como otimização (Vercel Workflow 5.x com QuickJS, experimental; Code Mode do pi-durable usa replay numa VM determinística). São snapshots opacos do heap, presos ao build do motor e subordinados ao log; o quadrante "código linear + snapshot gerado por compilador" continua vazio. Ver `docs/research/vercel-workflow-worlds.md` e `docs/research/pi-durable.md`.
- A frase de posicionamento correta é "uma biblioteca só consegue snapshot se o programador escrever a máquina de estados à mão; uma linguagem consegue a partir de código linear" — bibliotecas não estão impedidas de fazer snapshot.
