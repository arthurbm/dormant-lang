# Dormant

Vocabulário do Dormant: um subconjunto de TypeScript para workflows duráveis, desenvolvido como projeto da disciplina de Construção de Compiladores (CIn-UFPE, 2026.2).

## Durabilidade

**Workflow**:
Função marcada com a diretiva `"use workflow"` cuja execução sobrevive a reinícios do processo; só causa efeitos colaterais chamando Steps.
_Avoid_: job, fluxo, task

**Step**:
Função marcada com a diretiva `"use step"`; a única unidade de efeito colateral de um Workflow, cujo resultado fica registrado.
_Avoid_: passo, activity, tarefa

**Ponto de suspensão**:
Um lugar do Workflow onde a execução pode pausar e ser retomada depois, possivelmente em outro processo: um `await` de Step ou um `sleep`.
_Avoid_: pausa, yield

**Replay**:
Estratégia de retomada que reexecuta o Workflow desde o início, devolvendo de um log os resultados dos Steps já concluídos.
_Avoid_: reexecução, journaling

**Snapshot**:
Estratégia de retomada que, em cada Ponto de suspensão, salva um Checkpoint e, ao retomar, continua dali sem reexecutar nada.
_Avoid_: fotografia

**Checkpoint**:
O registro salvo por um Snapshot: o estado atual da máquina de estados e os valores das variáveis vivas naquele Ponto de suspensão.
_Avoid_: dump, imagem da memória

## Linguagem

**Prelúdio**:
O módulo TypeScript que declara o que o Dormant oferece além do subconjunto de TS (`int`, `float`, `sleep`, unidades de tempo), mantendo os programas válidos para o `tsc`.
_Avoid_: stdlib

**Runtime**:
O programa que executa o JavaScript gerado pelo compilador: agenda os `sleep`s, chama os Steps e persiste os Checkpoints.
_Avoid_: engine, motor

## Roadmap

**Entrega**:
Uma das quatro etapas do projeto, uma por etapa do compilador: definição da linguagem; gramática + parser; tipos + interpretador; IR + otimização + geração de código.
_Avoid_: marco, milestone

**Núcleo**:
A parte de cada Entrega que é uma etapa completa de compilador mesmo sem durabilidade.
_Avoid_: base, MVP

**Camada durável**:
A parte de cada Entrega construída sobre o Núcleo para dar suporte a Workflows (diretivas, regras de tipos, Replay, Snapshot).
_Avoid_: extensão, diferencial
