# pi-durable: durabilidade por checkpoint (e agora também por replay) e lições para o Dormant

Pesquisa sobre o repositório [earendil-works/pi](https://github.com/earendil-works/pi), pacote `packages/durable`, no commit `7d75a0e3eb273dff753d49c066ea29fbe7b3b6f6` (versão 1.2.0 do `@earendil-works/pi-durable`).

Caminhos como `packages/durable/docs/spec.md:1816` apontam para esse commit. Para abrir no GitHub, use `https://github.com/earendil-works/pi/blob/7d75a0e3eb273dff753d49c066ea29fbe7b3b6f6/<caminho>#L<linha>`. Itens tirados de memória ou de fonte secundária estão marcados como **(não verificado)**. A seção 5 é inferência, e o texto deixa isso explícito.

**Mudanças recentes (1.1.0 → 1.2.0).** A versão 1.2.0 saiu em 2026-10-12 (`packages/durable/CHANGELOG.md:5`). A mudança relevante para o Dormant é uma ferramenta `codemode` cujos scripts sobrevivem a um restart **por replay determinístico a partir de um journal**. Ou seja, o pi-durable deixou de ser só snapshot: agora combina as duas estratégias (seções 2.7 e 4). Os documentos de design auxiliares (`pico-v5-handoff.md`, `pico-v5-chord-usage.md`) saíram do repositório. Só resta `docs/spec.md`, que se declara normativo (`packages/durable/docs/spec.md:3`).

## 1. O que é

O pi-durable é um *harness* (a infraestrutura que roda o loop do agente) de agentes de IA durável, escrito em TypeScript. A Earendil, empresa por trás do agente de programação Pi, o anunciou junto com o Pi 1.0 em 2026-10-01 ([post do Pi 1.0](https://earendil.com/posts/pi-1-0/), [post do Pi Durable](https://earendil.com/posts/pi-durable/), ambos datados de 01/10/2026). O post do Pi 1.0 o define como "a new substrate for building long-running agentic applications". Segundo o mesmo post, Mario Zechner escreveu o texto sobre o Pi Durable, mas a assinatura da página é "Earendil Engineering".

O README resume a proposta: "Conversations, model turns, tool calls, and your own state are committed to storage before anything is shown. If the process dies mid-turn, reopening the storage picks the work up where it stopped." (`packages/durable/README.md:5`). O pacote é experimental: "The API changes without notice between releases." (`README.md:3`).

**Fatos básicos:**
- Licença MIT (`README.md:749`). O pacote npm é [`@earendil-works/pi-durable`](https://www.npmjs.com/package/@earendil-works/pi-durable). A versão 0.0.1 saiu em 2026-09-19 e a 1.2.0 em 2026-10-12, segundo `npm view` (`time`).
- O código-fonte em `packages/durable/src` tem cerca de 25 mil linhas (`wc -l`). Na 1.1.0 eram cerca de 20 mil.
- Mantenedores no npm: `mitsuhiko` (Armin Ronacher), `badlogic` (Mario Zechner) e `rwachtler` (`npm view`).

**Escopo.** O foco é durabilidade de agentes: conversas, chamadas ao modelo e tool calls. Mas o mecanismo de base é genérico: *tasks*, máquinas de estados duráveis definidas pelo usuário (seção 2.1). O exemplo de checkout com pagamentos no README não tem nada de LLM (`README.md:581-606`).

**Projeto anterior relacionado.** O [Absurd](https://github.com/earendil-works/absurd), de Armin Ronacher, é execução durável só com Postgres, por replay de Steps gravados. Ele se descreve como "the simplest durable execution workflow system you can think of" (README do repositório).

## 2. Como a durabilidade funciona

### 2.1 Regra central: só o que foi commitado existe

"A Session atomically commits immutable entries, full task records, and Chord-tracked documents. Only committed state is observable." (`packages/durable/docs/spec.md:43`). O README diz o mesmo: "nothing is shown before its commit is stored" (`README.md:82`). O Chord é a biblioteca de estado da Earendil, que representa um documento JSON como operações (deltas) mais bases completas ocasionais (`spec.md:1440-1476`).

### 2.2 Tasks são máquinas de estados escritas à mão (snapshot)

Uma *task* é "a durable state machine that saves a checkpoint at every step, so a restarted process continues from the last one" (`README.md:87`). Na definição (`defineTask`), o programador declara (`spec.md:1772-1785`):
- `initial(input)`, que gera o checkpoint inicial;
- um mapa `phases` com um handler por fase;
- um handler `abort`;
- um `migrate` opcional.

O checkpoint é um objeto JSON com o campo `phase`. Cada handler faz commit de um novo estado: outro checkpoint, uma espera (`waiting`) por tasks filhas ou um resultado final. "each durable checkpoint is a full replacement" (`spec.md:1816`). O checkpoint é gravado na mesma transação que as entradas do transcript e as edições de documentos.

**Retomada.** Ao reabrir o storage, tasks que estavam `running` voltam para `pending` com o checkpoint preservado (`spec.md:1904-1906`). O agendador então chama o handler de `checkpoint.phase`. **Nenhum código é reexecutado para reconstruir estado**, então os handlers não precisam ser determinísticos. No vocabulário do Dormant, isso é **Snapshot**: o programador escolhe o rótulo da fase e as variáveis salvas.

**Regra de progresso.** Se uma fase termina sem mudar o checkpoint, a task é marcada `faulted`, "because no durable progress was made" (`spec.md:1892-1896`).

### 2.3 Effect sandwich: intenção antes do efeito

O spec chama de *effect sandwich* a sequência abaixo (`spec.md:1922-1928`):

```text
commit intent phase
perform external effect
commit outcome or next phase
```

"Reopening in an intent phase means the effect may have happened." (`spec.md:1930`). O handler então refaz o efeito com segurança, consulta um handle externo ou registra a interrupção. O efeito, portanto, precisa ser idempotente: no exemplo de pagamento, a chave de idempotência fica no checkpoint (`spec.md`, seção 5.1, exemplo `charge`).

Tool calls seguem o mesmo padrão. A intenção é gravada antes de `execute()`. Depois de um crash, a ferramenta só roda de novo se declarar `replay: "safe"`. Caso contrário, o modelo recebe um resultado de erro `interrupted` (`README.md:181`, `spec.md:4103`, `packages/durable/src/harness/tool.ts:173`).

### 2.4 Memos e `requestId`

**Memos** são "small first-writer-wins values stored in the live task envelope" (`spec.md:1935`). Em caso de disputa, vale o primeiro valor gravado. Servem, por exemplo, para guardar a decisão de um hook de aprovação, de modo que uma reexecução reuse a decisão em vez de perguntar de novo ([post](https://earendil.com/posts/pi-durable/)).

**`requestId`.** Uma submissão repetida com o mesmo `requestId` devolve a submissão existente em vez de criar outra (`README.md:117-122`). É deduplicação de entrada.

### 2.5 Versões e mudança de código

- Tasks e documentos têm `version`. Um único `migrate(input, checkpoint, fromVersion)` trata todas as versões antigas suportadas (`spec.md:1494`, `:1781`).
- A migração roda na reserva da task, atomicamente com a transição para `running`. Uma task cuja definição está ausente, é mais antiga que a gravada ou falha na migração fica bloqueada até que uma definição adequada seja instalada (`spec.md:1906-1910`).
- Na 1.2.0, `pi.tool` passou para a versão 2 e "migrates live version 1 tasks" (`CHANGELOG.md:9`). É um caso real do mecanismo em uso.
- As conversas guardam **nomes** de extensões e ferramentas, não código. Na recarga a quente, o trabalho já iniciado termina com o código antigo, e a próxima fase usa o novo (`README.md:376`).

### 2.6 Storage

Há quatro backends: memória, SQLite (WAL com `synchronous = NORMAL`), JSONL e o SQLite de um Durable Object da Cloudflare (`README.md:656-663`). "One process owns a storage at a time; there is no cross-process locking." (`README.md:665`). Na 1.2.0, qualquer erro de storage passou a encerrar a Session (`SessionFailed`), sem retentativa (`CHANGELOG.md:11`).

O spec lista como não objetivo um "Session-kernel semantic event journal" (`spec.md:5332`). O núcleo não é event sourcing: a palavra "replay" no núcleo aparece só no nível dos documentos, em que deltas do Chord são reaplicados sobre a última base (`spec.md:1440-1476`).

### 2.7 Novo na 1.2.0: Code Mode durável por replay

O Code Mode deixa o modelo escrever JavaScript que chama as ferramentas da conversa como `await tools.<name>(args)`, numa VM [QuickJS](https://bellard.org/quickjs/) compilada para WebAssembly (`README.md:259-261`). O problema está descrito no cabeçalho do módulo: "a script is in-memory VM state", e um restart perderia o script e todas as chamadas já feitas (`packages/durable/src/codemode/codemode.ts:2-8`).

**Solução: replay determinístico.** O cabeçalho do módulo descreve: "given the same start data, clock, seed, and results in the same order, it makes the same calls with the same arguments" (`codemode.ts:10-15`). Funciona assim:
- **Antes de entregar cada resultado ao script**, o host grava esse resultado num journal, o documento `pi.codemode.journal` (`spec.md:3657-3661`). O ponto inicial (seed, relógio, ferramentas, fingerprint da VM) fica em `pi.codemode.run` (`spec.md:3653-3656`).
- **Após um restart**, o script roda de novo desde o início e recebe os resultados do journal na mesma ordem. As chamadas aninhadas se reconectam às suas tasks pela chave `c<id>`, o número que a VM dá à chamada, e por isso não rodam duas vezes. Depois disso, o script continua ao vivo (`codemode.ts:12-15`, `spec.md:3683-3686`).
- **Determinismo imposto pela VM** (`packages/codemode/src/runtime/vm.ts:23-36`):
  - relógio virtual;
  - `Math.random` sobre um PRNG (gerador pseudoaleatório) com seed;
  - fuso UTC;
  - sem `WeakRef` nem `FinalizationRegistry`;
  - entrega de resultados estritamente na ordem de `seq`.
- **Detecção de divergência.** O replay é recusado, com resultado de erro, quando muda a versão do módulo (`CODEMODE_MODULE_VERSION`, `codemode.ts:80-84`), quando muda o fingerprint da VM, quando o prazo passou, ou "when the script diverges: it makes another call, or calls in another order, than the journal says" (`codemode.ts:31-34`, `spec.md:3687-3693`).

O Code Mode é, ele mesmo, uma ferramenta `replay: "safe"` dentro do modelo de tasks (`spec.md:3590-3591`). Ou seja, **o replay é usado só onde o estado está preso numa VM opaca**, e o resto do sistema continua em checkpoint.

## 3. API (exemplo curto)

Uma task com tasks filhas, adaptada do README (`README.md:585-599`):

```ts
pay: async (task, runtime, context) => {
  await runtime.commit(async (tx) => {
    const payments = [];
    for (const card of task.input.cards) {
      payments.push(await tx.createTask(Payment, { card }, { ownership: { kind: "task", taskId: task.id } }));
    }
    // Retoma em `decide` quando todos os pagamentos terminarem; a primeira falha aborta o resto.
    return { status: "waiting", checkpoint: { phase: "decide", payments }, on: payments, policy: "failFast" };
  }, context);
},
decide: async (task, runtime, context) => {
  const outcomes = await runtime.outcomes(task.state.checkpoint.payments, context);
  // ...commit do resultado do checkout
},
```

No nível do agente (`README.md:110-122`):

```ts
const harness = await Harness.open(await openNodeSqliteStorage("./session.sqlite"), { models, registry }, context);
const root = await harness.root(context); // a mesma conversa raiz da última vez
harness.resume();                         // continua o que o processo anterior deixou pela metade
await root.submit({ type: "input", content: "Hello", requestId: "greeting-1" }, context);
```

O `"decide"` acima é exatamente o que o compilador do Dormant geraria a partir de um `await` em código linear: o rótulo do Ponto de suspensão mais as variáveis vivas (`payments`) no checkpoint.

## 4. Comparação com sistemas de replay

| | Replay (Temporal, Vercel Workflow, Cloudflare Workflows, Restate, Inngest, Effect Workflow, Absurd) | pi-durable: tasks | pi-durable: Code Mode (1.2.0) |
|---|---|---|---|
| Como o código é escrito | Função linear | Fases e checkpoint à mão | Script linear |
| Retomada | Reexecuta do início, devolvendo resultados do log | Chama o handler da fase gravada | Reexecuta do início, devolvendo resultados do journal |
| Determinismo | Exigido do programador | Não exigido | Imposto pela VM (relógio, seed, UTC) |
| Mudança de código | Versionamento ou *patching* | `version` + `migrate(checkpoint)` | Recusa o replay se versão ou fingerprint mudou |
| Divergência | Detectada pelo motor | Não se aplica | Detectada e convertida em erro |

As linhas sobre o pi-durable vêm das seções 2.2 a 2.7. As da coluna de replay vêm da documentação de cada sistema, registrada em `docs/references.md`. O pi-durable não se compara a nenhum desses sistemas nos posts ou no spec. O contraste é análise própria.

Outras diferenças: o pi-durable é embutido e local (uma biblioteca sobre um arquivo SQLite ou JSONL, um processo por storage), não um serviço em cluster. E ele traz conceitos de agente prontos: transcript, inbox com steering e follow-up, compactação, subagentes e vários clientes observando a mesma conversa (`README.md:378-606`).

## 5. O que aproveitar no Dormant (inferência)

**Runtime**
1. **Effect sandwich como contrato de Step.** Gravar a intenção (`step.started` com os argumentos) antes de rodar o Step e o resultado depois. Ao reabrir no meio, aplicar uma política declarada (seção 2.3).
2. **Política por Step `replay: "safe" | "unsafe"`.** Um Step seguro roda de novo; um inseguro vira erro `interrupted` visível ao Workflow. Pode ser uma anotação ou diretiva no Dormant (seção 2.3).
3. **Regra "fase sem progresso = falha"**, que pega loops em que o código gerado não avança o checkpoint (seção 2.2).
4. **`requestId` para iniciar Workflows** e memos first-writer-wins para decisões externas (seção 2.4).
5. **Um processo por storage, sem lock entre processos**, é uma simplificação aceitável para a v1; é a mesma restrição do pi-durable (seção 2.6).
6. **`sleep` durável.** O `sleep(until)` do runtime de tasks espera em memória (`spec.md:1856`). Para sobreviver a restart, o horário de acordar precisa estar no checkpoint, como o spec faz com provedores adiados: "a durable phase containing their handle and next poll time" (`spec.md:1930-1933`).

**Interpretador com replay (Entrega 3)**
1. **O Code Mode é um modelo quase direto do que a Entrega 3 faz:** reexecutar o programa desde o início, devolvendo os resultados de Steps já gravados, e continuar ao vivo (seção 2.7).
2. **Determinismo imposto pelo interpretador, e não exigido do programador.** Relógio virtual, seed fixa e UTC. Como o interpretador do Dormant controla todos os built-ins, pode fazer o mesmo.
3. **Detecção de divergência por posição.** Comparar nome e argumentos do Step pedido com o gravado naquela posição do journal e, se diferirem, falhar com mensagem clara (`spec.md:3687-3693`).
4. **Versão do interpretador gravada no início da execução**, recusando o replay se mudar (`CODEMODE_MODULE_VERSION`).

**Compilador com snapshot (Entrega 4)**
1. **O formato de saída já tem referência concreta.** `phases` + `checkpoint` com campo `phase`, `waiting` com `on`/`policy` e `migrate(checkpoint, fromVersion)` (seção 2.2). O ADR 0003 já adota isso como referência inicial.
2. **O argumento da tese ficou mais forte.** No pi-durable, o replay aparece justamente no componente cujo estado fica dentro de uma VM QuickJS opaca (seção 2.7), enquanto as tasks, de estado explícito, usam checkpoint. Um compilador que conhece o programa, faz análise de *liveness* (descobrir quais variáveis ainda serão lidas depois de cada ponto) e gera a máquina de estados dispensa o replay até para código linear. Esse é exatamente o quadrante do Dormant.
3. **Versionamento assistido pelo compilador.** Se uma mudança no Workflow altera o conjunto de variáveis vivas num Ponto de suspensão, o compilador pode avisar que é preciso um `migrate`. Nem o pi-durable nem os sistemas de replay fazem isso.

**Testes**
1. **Reproduzir os cenários dos exemplos do pi-durable:** crash durante pagamentos paralelos (`test/examples/24-child-tasks.ts`, citado em `README.md:606`), crash no meio de um tool call e retomada após restart (`test/examples/13-recovery.ts`, citado em `README.md:739`).
2. **Teste de equivalência entre as duas estratégias.** O mesmo Workflow, com crash injetado em cada Ponto de suspensão, deve produzir a mesma saída e a mesma sequência de Steps no interpretador (replay) e no código compilado (snapshot).
3. **Teste de divergência.** Alterar o Workflow entre o crash e a retomada e verificar que o replay falha com diagnóstico em vez de seguir com estado errado.

**Posicionamento**
- **A tabela de quadrantes do documento de design (`design.md`) precisa de nota.** Desde a 1.2.0, o pi-durable ocupa dois lugares: "snapshot escrito à mão" (tasks) e "código linear + replay" (Code Mode). O quadrante "código linear + snapshot gerado por compilador" continua vazio.
- **O pi-durable não torna o Dormant redundante.** Ele é uma biblioteca de runtime sem transformação em tempo de compilação. O Dormant é a camada que gera, a partir de código linear, a máquina de estados que o pi-durable pede para escrever à mão.
- **Saída futura.** O ADR 0003 já prevê trocar o runtime próprio pelo do pi-durable mudando só o gerador de código. Com o formato atual de `defineTask`, isso continua plausível.

## 6. Itens não verificados

- **Autoria do post.** Quem escreveu o post do Pi Durable: a página é assinada por "Earendil Engineering", e a atribuição a Mario Zechner vem do post do Pi 1.0.
- **Cobertura de imprensa.** Afirmações como "about 15,000 lines" ou "multiple humans steering the same agent" vêm de matérias sobre o lançamento ([alternativeto](https://alternativeto.net/news/2026/10/pi-1-0-brings-native-mcp-support-and-pi-durable-for-long-running-crash-resistant-ai-agents/), [letsdatascience](https://letsdatascience.com/news/pi-10-adds-mcp-and-durable-harness-6241c6f7)). A contagem de linhas medida aqui é diferente (seção 1).
- **Cloudflare.** A página da Cloudflare sobre rodar o Pi em Durable Objects ([developers.cloudflare.com/agents/harnesses/pi](https://developers.cloudflare.com/agents/harnesses/pi/)) apareceu em buscas, mas não foi aberta.
- **Comportamento dos sistemas de replay.** As características citadas na seção 4 vêm de memória e de `docs/references.md`, não foram reconferidas nesta pesquisa.
- **Arquivos de exemplo.** Os exemplos `13-recovery.ts` e `24-child-tasks.ts` foram citados a partir do README, não lidos.
- **Execução.** Nada foi compilado nem executado. Todas as afirmações sobre comportamento vêm do README, do spec, do CHANGELOG e da leitura de código.
