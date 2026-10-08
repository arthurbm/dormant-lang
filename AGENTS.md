# Dormant

Um subconjunto sintático de TypeScript com semântica estendida para workflows duráveis. O compilador transforma código linear numa máquina de estados retomável por snapshot, salvando só as variáveis vivas. Escrito em TypeScript sobre o Bun e gera JavaScript. É um projeto da disciplina de Construção de Compiladores (CIn-UFPE, 2026.2).

**Fase atual:** design da v1 definido. Ainda não há código.

**Contexto privado:** se `workspace/AGENTS.md` existir, leia-o antes de começar. Ali estão o perfil do autor, os prazos e o material da disciplina, que ficam fora deste repositório.

## Convenções

- Este repositório é público e funciona como portfólio. Correspondência, material da disciplina, notas e rascunhos ficam em `workspace/`.
- Escreva em português do Brasil, com acentuação completa. Identificadores e termos técnicos ficam no original. Nomes de arquivos e pastas ficam em inglês.
- Use o vocabulário do `GLOSSARY.md`: Workflow, Step, Ponto de suspensão, Replay, Snapshot, Entrega…
- `docs/proposal.md` é **autocontido**: só links externos, e todo conceito é explicado no próprio texto. Diagramas em Mermaid.
- Mudança que contradiz um ADR atualiza ou substitui o ADR na mesma alteração.

## Mapa

- `docs/adr/`: decisões aceitas (tese, sintaxe, implementação/alvo, cortes da v1). Leia antes de propor mudanças de arquitetura ou de escopo.
- `docs/proposal.md`: o design da v1 (linguagem, tese, arquitetura, plano por Entrega).
- `docs/references.md`: fontes e por que importam. Leia ao pesquisar e registre ali o que encontrar.
