# Documentação de domínio

Como as skills de engenharia devem consumir a documentação de domínio deste repositório ao explorar o código.

## Antes de explorar, leia

- **`GLOSSARY.md`** na raiz.
- **`docs/adr/`**: leia os ADRs que tocam a área em que você vai trabalhar.

Se algum desses arquivos não existir, **siga em silêncio**. Não aponte a ausência nem sugira criá-los de antemão. A skill `/domain-modeling` (acionada por `/grill-with-docs` e `/improve-codebase-architecture`) os cria quando termos ou decisões forem de fato resolvidos.

## Estrutura

Repositório single-context:

```
/
├── GLOSSARY.md
├── docs/adr/
│   ├── 0001-compiled-snapshot-thesis.md
│   └── ...
└── src/
```

## Use o vocabulário do glossário

Quando sua saída nomear um conceito do domínio (num título de issue, numa proposta de refatoração, numa hipótese, num nome de teste), use o termo como definido no `GLOSSARY.md`. Troque qualquer sinônimo listado em _Avoid_ pelo termo canônico.

Se o conceito de que você precisa ainda não está no glossário, isso é um sinal: ou você está inventando linguagem que o projeto não usa (reconsidere), ou há uma lacuna real (anote para o `/domain-modeling`).

## Aponte conflitos com ADRs

Se sua saída contradiz um ADR existente, diga isso explicitamente em vez de sobrescrevê-lo em silêncio:

> _Contradiz o ADR-0004 (cortes da v1), mas vale reabrir porque…_

Pela regra do `AGENTS.md`, uma mudança que contradiz um ADR atualiza ou substitui o ADR na mesma alteração.
