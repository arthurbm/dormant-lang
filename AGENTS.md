# Dormant

Um subconjunto sintático de TypeScript com semântica estendida para workflows duráveis. O compilador transforma código linear numa máquina de estados retomável por snapshot, salvando só as variáveis vivas. Escrito em Rust; gera JavaScript, executado por um Runtime próprio em JavaScript. É um projeto da disciplina de Construção de Compiladores (CIn-UFPE, 2026.2).

**Contexto privado:** se `workspace/AGENTS.md` existir, leia-o antes de começar. Ali estão o perfil do autor, os prazos e o material da disciplina, que ficam fora deste repositório.

## Convenções

- Este repositório é público e funciona como portfólio. Correspondência, material da disciplina, notas e rascunhos ficam em `workspace/`.
- Escreva em português do Brasil, com acentuação completa. Identificadores e termos técnicos ficam no original. Nomes de arquivos e pastas ficam em inglês.
- Use os termos do `GLOSSARY.md` e troque qualquer sinônimo listado em _Avoid_ pelo termo canônico.
- Mudança que contradiz um ADR atualiza ou substitui o ADR na mesma alteração.
- O compilador nasce de `cargo new`/`cargo init`, e dependências entram por `cargo add`: o `Cargo.toml` só muda por comandos. Se um comando oficial exigir respostas interativas, peça ao autor que o rode no terminal dele.

## Mapa

- `docs/adr/`: decisões aceitas (tese, sintaxe, implementação/alvo, cortes da v1). Leia antes de propor mudanças de arquitetura ou de escopo.
- `docs/design.md`: linguagem, tese, arquitetura e escopo da v1. Leia antes de implementar qualquer parte da linguagem. É um documento **vivo**: atualize-o na mesma alteração que mudar a linguagem ou a arquitetura, explicando cada conceito no próprio texto (diagramas em Mermaid).
- `docs/roadmap.md`: as quatro Entregas (compilador base + camada durável), status e riscos. Leia ao decidir o que fazer a seguir; atualize o status ao concluir uma etapa.
- `docs/research/`: pesquisas com fontes primárias sobre sistemas relacionados e o registro da escolha da tese. Consulte antes de pesquisar um sistema, pois a pesquisa pode já existir; novas pesquisas entram aqui, no formato dos arquivos existentes.
- `docs/references.md`: fontes e por que importam. Leia ao pesquisar e registre ali o que encontrar.
