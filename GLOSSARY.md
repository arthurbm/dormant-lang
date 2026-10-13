# Dormant

Vocabulário do Dormant: um subconjunto de TypeScript para workflows duráveis, desenvolvido como projeto da disciplina de Construção de Compiladores (CIn-UFPE, 2026.2).

## Roadmap

**Entrega**:
Um dos quatro marcos do projeto, um por etapa do compilador: definição da linguagem; gramática + parser; tipos + interpretador; IR + otimização + geração de código.
_Avoid_: marco, milestone, apresentação

## Durabilidade

**Workflow**:
Função marcada com a diretiva `"use workflow"` cuja execução sobrevive a reinícios do processo; só pode causar efeitos colaterais chamando Steps.
_Avoid_: job, fluxo, task

**Step**:
Função marcada com a diretiva `"use step"`; a única unidade de efeito colateral de um Workflow, cujo resultado fica registrado.
_Avoid_: passo, activity, tarefa

**Ponto de suspensão**:
Um lugar do workflow onde a execução pode pausar e ser retomada depois, possivelmente em outro processo (um `await` de passo ou um `sleep`).
_Avoid_: pausa, yield, checkpoint

**Replay**:
Estratégia de retomada que re-executa o workflow desde o início, devolvendo de um log os resultados dos passos já concluídos.
_Avoid_: reexecução, journaling

**Snapshot**:
Estratégia de retomada que salva o estado atual da máquina de estados e apenas as variáveis vivas naquele ponto de suspensão, continuando dali sem re-executar nada.
_Avoid_: checkpoint, fotografia

## Linguagem e escopo

**Prelúdio**:
O módulo TypeScript que declara o que o Dormant oferece além do subconjunto de TS (`int`, `float`, `sleep`, unidades de tempo), mantendo os programas válidos para o `tsc`.
_Avoid_: stdlib, runtime

**Núcleo**:
A parte de cada Entrega que cumpre sozinha o que o plano da disciplina exige, independentemente da durabilidade.
_Avoid_: base, MVP

**Camada durável**:
A parte de cada Entrega construída sobre o Núcleo para dar suporte a Workflows (diretivas, regras de tipos, replay, snapshot).
_Avoid_: extensão, diferencial
