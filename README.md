# Dormant

**Workflows duráveis compilados.** Um subconjunto de TypeScript em que funções podem pausar por horas ou dias e retomar exatamente de onde pararam, mesmo depois que o processo reinicia, e em que o compilador decide sozinho o que precisa ser salvo.

```ts
export async function cobrar(pedido: int): Promise<string> {
  "use workflow";
  const valor = await buscarValor(pedido);   // Step: efeito colateral registrado
  const total = valor + 500;
  await sleep(3 * DIA);                      // o processo pode morrer aqui
  const ok = await enviarCobranca(pedido, total);
  if (ok) { return "cobrado"; }
  return "falhou";
}
```

Todo programa Dormant é TypeScript válido. O que muda é a semântica: compilado pelo Dormant, `cobrar` vira uma máquina de estados que, a cada pausa, persiste **só as variáveis ainda necessárias**. No `sleep` acima, apenas `pedido` e `total`: `valor` já não é mais lida.

## Replay × snapshot

Sistemas como Temporal, Vercel Workflow e Cloudflare Workflows retomam por **replay**: reexecutam a função desde o início, consultando um log de resultados, o que exige código determinístico e torna delicado mudar o código enquanto há execuções em andamento: a versão nova precisa reproduzir o histórico gravado pela antiga. A alternativa é o **snapshot**: salvar o ponto de parada e o estado necessário, e continuar dali. Bibliotecas que fazem snapshot, como o pi-durable, pedem que o programador escreva a máquina de estados à mão.

O Dormant gera essa máquina de estados a partir de código linear, usando **análise de variáveis vivas** (*liveness*) sobre uma representação intermediária de três endereços (TAC). Como o estado salvo é dado, o compilador também compara versões de um Workflow e avisa, antes do deploy, quando uma mudança impede retomar as execuções em andamento. E, como só Steps causam efeitos, ele lista os Steps que cada Workflow pode chamar. O interpretador implementa a outra estratégia, replay, para comparar as duas com implementação própria.

|  | Código linear | Máquina de estados à mão |
|---|---|---|
| **Replay** | Temporal, Vercel Workflow, Cloudflare Workflows, Restate; Code Mode do pi-durable | — |
| **Snapshot** | **Dormant** | tasks do pi-durable |

## Como funciona

| Papel | Linguagem |
|---|---|
| Fonte | Dormant (subconjunto de TypeScript + diretivas `"use workflow"` / `"use step"`) |
| Implementação | Rust |
| Alvo | JavaScript em forma de máquina de estados, gerado a partir do TAC e executado por um Runtime mínimo em JavaScript, em qualquer motor JavaScript (Node, Deno, Bun) |

Pipeline: parser ([`nom`](https://github.com/rust-bakery/nom), parser combinators em Rust) → verificador de tipos → interpretador com replay · e, no compilador, TAC → otimizações (propagação de constantes, eliminação de código morto, subexpressões comuns) → liveness nos pontos de suspensão → JavaScript gerado a partir do TAC.

Interface: uma CLI. `dormant parse arquivo.ts` (imprime a AST) já existe; `dormant build arquivo.ts` (compila) vem na Entrega 4.

O diagrama do pipeline, com a Entrega de cada etapa, está em [`docs/diagrams/pipeline.excalidraw`](docs/diagrams/pipeline.excalidraw) (abra no [Excalidraw](https://excalidraw.com) ou na extensão do VS Code).

## Como rodar

Requer Rust (`cargo`).

```sh
cargo run -- parse tests/fixtures/valid/invoice.ts   # imprime a AST do Workflow da fatura
cargo test                                            # testes do parser e da CLI
cargo test --test tsc -- --ignored                    # opcional: confere os exemplos no tsc (requer Node)
```

Num erro de sintaxe, `dormant parse` imprime `arquivo:linha:coluna: mensagem` e sai com código 1:

```text
$ cargo run -q -- parse tests/fixtures/invalid/missing-semicolon.ts
tests/fixtures/invalid/missing-semicolon.ts:2:12: esperava `;`, encontrou `console`
```

## Status

Em desenvolvimento como projeto da disciplina de Construção de Compiladores (CIn-UFPE, 2026.2). A Entrega 2 está implementada: o parser da v1 em `nom` (`src/parser/`), a AST (`src/ast.rs`), a CLI `dormant parse` e a suíte de testes (`tests/`). A próxima etapa é o verificador de tipos e o interpretador com Replay. Veja o [roadmap](docs/roadmap.md).

## Documentação

- [Design](docs/design.md): a linguagem, a tese, a arquitetura e o escopo da v1.
- [Roadmap](docs/roadmap.md): as quatro Entregas e o status de cada uma.
- [Decisões (ADRs)](docs/adr/): por que snapshot compilado, por que TypeScript válido, por que Rust com alvo JavaScript, o que fica fora da v1.
- [Pesquisas](docs/research/) e [referências](docs/references.md): sistemas relacionados, com fontes primárias.
- [Glossário](GLOSSARY.md).
- [Identidade visual](docs/visual-identity.md). Logo do dragão e conceito das variáveis como chaminhas.
