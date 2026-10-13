---
status: accepted
---

# Cortes da v1

A v1 aceita: `let`/`const`, `int`, `float`, `boolean`, `string`, listas, funções com tipos anotados, `if`/`else`, `while`, `return`, operadores aritméticos/relacionais/lógicos, `console.log`, as diretivas `"use workflow"`/`"use step"`, `await` de steps e `sleep`. Ficam fora: `await` dentro de laços ou blocos aninhados (só no nível de comando do workflow), `try`/`catch`, steps em paralelo (o `await` é pausa sequencial, não concorrência), objetos e classes, closures e arrow functions, generics, union types, `any`, módulos, Migração de execuções em andamento (`migrate`). Os cortes têm dois motivos: o plano da disciplina pede só tipos básicos na primeira versão, e snapshot exige que tudo o que está vivo num ponto de suspensão seja serializável.

## Consequences

- **Closures** são o corte menos óbvio: salvar uma closure viva exige salvar código (que muda entre versões) e variáveis capturadas (possivelmente compartilhadas entre closures — salvar separado cria cópias e muda o comportamento). O caminho de volta está mapeado: (1) permitir closures, mas proibir que uma closure esteja viva num ponto de suspensão — checado pela própria liveness, análogo a valores `Send` através de `.await` em Rust; (2) depois, defuncionalização (Reynolds) para torná-las serializáveis.
- A Migração fica fora, mas o **diagnóstico de compatibilidade** entra: o compilador emite o Layout de Checkpoints e aponta os Pontos de suspensão que uma nova versão deixou incompatíveis. O corte de `await` fora do nível de comando é o que o torna barato: os Pontos de suspensão formam uma sequência, e cada um é identificado pela posição e pelo que espera.
- Cada Entrega tem um **compilador base** (cumpre o plano sozinho) e uma **camada durável** por cima; se a camada atrasar, o compilador base garante a Entrega.
