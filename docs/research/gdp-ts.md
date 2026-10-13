# gdp-ts: provas de autorização checadas em compilação, e uma ideia futura para o Dormant

Pesquisa sobre o repositório [rauchg/gdp-ts](https://github.com/rauchg/gdp-ts), no commit `0b6ebeeec827b05210f3acfa4149a70dda9a0de5` (pacote `@gdp-ts/core` 0.1.0, `package.json:2-3`). Também usei o post de lançamento no X e o artigo original de Matt Noonan.

Os caminhos citados, como `src/index.ts:42`, apontam para esse commit. Para abrir um deles no GitHub, use `https://github.com/rauchg/gdp-ts/blob/0b6ebeeec827b05210f3acfa4149a70dda9a0de5/<caminho>#L<linha>`. Itens tirados de memória ou de fonte secundária estão marcados com **(não verificado)**. A seção 4 é inferência minha. O motivo de a ideia ter sido descartada como tese está em [`alternative-theses.md`](alternative-theses.md), seção C, e no [ADR 0001](../adr/0001-compiled-snapshot-thesis.md).

## 1. O anúncio

Guillermo Rauch (CEO da Vercel) anunciou o projeto em 2026-10-05 ([post no X](https://x.com/rauchg/status/2107119811444748555)). O link do post, `https://t.co/eRWdiMvQ2V`, redireciona para `https://github.com/rauchg/gdp-ts`. Trechos:

> "Introducing gdp-ts: Ghosts of Departed Proofs for TypeScript."
>
> "Under this contract, sensitive functions require 'proofs' that the caller performed an authorization check. The typechecker verifies these proofs at compile time […]"
>
> "Agents are writing more code than we can review, and they *thrive* in tight loops with hard constraints that would frustrate us."

O argumento é que a técnica já existia em Haskell, mas ficou nichada por causa do custo de revisão humana e da sobrecarga sintática. Com agentes escrevendo código, restrições duras checadas pelo compilador passam a compensar.

O primeiro commit é de 2026-10-05. Os dois PRs seguintes (#18 e #19, 06 e 07/10) fecharam brechas de falsificação:
- o `Kind` da prova passou a ficar fixo no slot fantasma (`src/index.ts:114-120`);
- uma regra nova de lint proíbe `null!` e `undefined!` (`src/lint/plugin.ts:255-258`).

## 2. Como funciona

O README resume o projeto como "a verification system to make API contracts more secure at compile (type check) time" (`README.md:5`). O núcleo tem 173 linhas (`src/index.ts`) e três ideias (`src/index.ts:4-22`):

1. **Nomear valores.** `name(value, k)` dá a um valor de runtime um *nome* `N` que só existe em tempo de compilação, dentro do callback `k`. Cada chamada gera um `N` diferente e incompatível (`src/index.ts:75-90`). `Named<in out N, out A>` é invariante em `N`: as anotações de variância `in out` impedem que `never` ou `unknown` se passem por um nome específico (`src/index.ts:42-49`). Um *tipo fantasma* (*phantom type*) é um parâmetro de tipo que não corresponde a nenhum dado em runtime e serve só para o verificador.
2. **Provar fatos sobre nomes.** `defineProof("Kind")` cria um *prover*. O módulo que o define e não o exporta é o *módulo confiável*, o único capaz de produzir `Proof<"Kind", [nomes…]>` (`src/index.ts:10-13`, `:165-173`). Em runtime, uma prova é só um objeto congelado `{ kind }`, e o único `as` da biblioteca está dentro de `prove` (`src/index.ts:169-171`).
3. **Exigir provas na assinatura.** Uma função sensível recebe a prova sobre os seus argumentos nomeados e não pode ser chamada com prova sobre outro valor, com o ID cru ou sem prova (`src/index.ts:19-22`). O exemplo do README é `setPasswordProtection(project, password, { admin, plan })`, que exige uma prova de papel e outra de plano (`README.md:54`).

Nomes e provas não podem sair do callback em que foram criados. Tentar retorná-los é erro de compilação, porque `N` é invariante (`src/index.ts:15-17`). É a versão em TypeScript de um tipo de rank 2 do Haskell (`skills/gdp-ts/references/limits.md:12`).

**Lint.** O TypeScript não impede `{} as UserIsProjectAdmin<U, P>`, e o lint fecha essa e outras brechas (`src/lint/plugin.ts:1-4`). As regras são puramente sintáticas, sem informação de tipos, e escritas na API de regras do ESLint, que o Oxlint também implementa (`src/lint/plugin.ts:6-12`). São seis (`src/lint/plugin.ts:311-318`):
- `no-define-proof`: só `proofs/` pode chamar `defineProof`;
- `no-exported-prover`;
- `no-proof-assertion`;
- `no-null-assertion`;
- e, no modo estrito, `no-type-assertion` e `no-any`.

Os presets aplicam cada regra aos arquivos certos (`src/lint/shared.ts:34-67`).

**Limites declarados** (`skills/gdp-ts/references/limits.md:6-16`):
- Falsificar uma prova é impossível em Haskell, mas em TS só o lint impede.
- **Provas envelhecem**: "A proof says the fact held when it was checked. Keep proofs request-scoped" (`:15`).

**Pacote e skill.** Requer TypeScript 5.4 ou mais novo (`README.md:23`). Segundo o próprio benchmark do projeto, custa cerca de 0,3 ms por chamada autorizada numa checagem a frio com TypeScript 7 (`README.md:15`). Também vem como skill para agentes (`README.md:19`, `:140`).

## 3. A origem: Ghosts of Departed Proofs

O artigo é Matt Noonan, *Ghosts of Departed Proofs (Functional Pearl)*, Haskell Symposium 2018 ([PDF](https://kataskeue.com/gdp.pdf), [DOI 10.1145/3242744.3242755](https://doi.org/10.1145/3242744.3242755)). A tese do resumo é que "sophisticated preconditions can be encoded in Haskell's type system with no run-time overhead, by using proofs that inhabit phantom type parameters attached to newtype wrappers". O artigo afirma que a técnica obtém "many of the benefits of dependent types and refinement types" usando apenas extensões conhecidas do Haskell 2010.

A biblioteca correspondente é [`gdp`](https://hackage.haskell.org/package/gdp) no Hackage, cuja versão mais recente é a 0.0.3.0, de 2019. O README do gdp-ts também credita o pacote `justified-containers` (2017), também de Noonan, e o post de Ollie Charles [Who Authorized These Ghosts!?](https://blog.ocharles.org.uk/posts/2019-08-09-who-authorized-these-ghosts.html) (2019), que aplicou a ideia a autorização (`README.md:177-179`). O README cita ainda versões em Rust (`departed`, `mononym`, `gdp_rs`) (`README.md:172`).

## 4. Tipos-prova como feature futura do Dormant (inferência)

No Dormant, o efeito colateral só acontece em Steps, então o lugar natural para exigir uma prova é a **chamada de Step**. Esboço de como a ideia caberia, sem compromisso com a v1:

1. **Prova como tipo nativo, não como codificação em biblioteca.** A v1 corta generics e closures (proposta, Seção 8), que são justamente o que o gdp-ts usa para escopar nomes. Além disso, as expressões da v1 não incluem asserções de tipo (proposta, Seção 6). Um tipo `Proof<"Aprovado", pedido>` reconhecido pelo próprio verificador dispensaria o lint: forjar uma prova seria um erro de tipo, não uma regra opcional. O "módulo confiável" viraria uma marcação no Step que pode produzir a prova.
2. **Quem produz a prova é um Step, e o replay já sabe lidar com isso.** Verificar uma permissão é I/O: consulta ao banco ou aprovação humana. Por isso, no Dormant, essa verificação teria de ser um Step, e o resultado (a prova ou `null`) fica registrado como o de qualquer Step. Na retomada por Replay, a prova volta do log, sem reconsultar.
3. **O ponto novo: provas envelhecem através de pontos de suspensão.** O gdp-ts resolve o envelhecimento limitando a prova a uma requisição (`limits.md:15`). Um Workflow não tem esse limite natural: uma aprovação obtida antes de `await sleep(3 * DIA)` pode ter sido revogada ao acordar. A análise de liveness, que o Dormant já faz em cada ponto de suspensão para decidir o que salvar no Snapshot, responderia de graça se *existe uma prova viva atravessando uma pausa*. Uma regra possível é proibir isso, forçando a reverificação depois da pausa, na mesma forma da regra planejada para closures (proposta, Seção 8). Outra é exigir que o tipo da prova declare uma validade explícita.
4. **Serializar é fácil; o problema é semântico.** Em runtime, uma prova do gdp-ts é um `{ kind }` congelado e um valor nomeado é um `{ value }` (`limits.md:14`). As duas coisas cabem num Snapshot sem esforço. A questão não é *como* salvar a prova, e sim se ela *ainda vale* ao ser restaurada. Esse é um problema próprio de execução durável, que o gdp-ts, pensado para requisições curtas, não precisa enfrentar.

Resumo: tipos-prova não serviriam como tese do Dormant, mas encaixam como extensão. Eles reaproveitam a liveness nos pontos de suspensão para uma regra de segurança, além do uso original de economizar estado.

## 5. Itens não verificados

- **Texto completo do post.** Eu o li por um espelho em texto ([defuddle.md](https://defuddle.md/x.com/rauchg/status/2107119811444748555)), não no X diretamente.
- **Benchmark de 0,3 ms.** É medição do próprio projeto (`bench/README.md`), não reproduzida.
- **Rust:** que as bibliotecas Rust citadas implementam a mesma ideia. Não verifiquei.
- **Seção 4.** É especulação de design; nada disso está na proposta da v1.
