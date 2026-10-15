---
status: accepted
---

# Compilador em Rust, alvo JavaScript gerado a partir do TAC

Substitui o ADR 0003. O compilador é escrito em Rust, com o parser na biblioteca de parser combinators `nom`, e gera JavaScript executado por um Runtime mínimo próprio, também em JavaScript. O Runtime acessa o armazenamento por uma interface pequena, então o código gerado roda em qualquer motor JavaScript (Node, Deno, Bun); o `bun:sqlite` deixa de ser acoplamento. Rust por três motivos:

- **O MiniC como referência de técnica.** A linguagem de referência da disciplina é Rust + `nom` e tem parser, verificador de tipos, interpretador e TAC: cada Entrega tem um análogo para consultar. É referência de técnica, não base de código. A linguagem, a gramática e a camada durável do Dormant são outras, e nada do parser de uma sintaxe de C com ponteiros seria reaproveitado.
- **O caminho pós-semestre.** SWC e oxc são Rust. O crate de análise e geração (TAC, CFG, liveness, geração de JavaScript) é o que um plugin de transpilador sobre TypeScript real precisa, sem port.
- **`enum` + `match` exaustivo.** Esquecer um tipo de nó da AST ou de instrução do TAC vira erro de compilação do próprio compilador.

O parser usa `nom`, como o MiniC, em vez de combinators próprios: em Rust, um combinator genérico próprio é uma função que recebe e devolve closures, com lifetimes nos tipos, a parte da linguagem mais dura para quem está começando.

O alvo continua JavaScript. As otimizações e a liveness rodam sobre o TAC e independem do alvo. A descida relevante é TAC → máquina de estados. Os Steps são funções da aplicação TypeScript, e o JavaScript gerado as chama diretamente, enquanto uma VM própria exigiria reimplementar cada efeito ou uma ponte Rust↔JS. O caminho pós-semestre também emite JavaScript. Executar o programa como TypeScript comum pode servir de oráculo nos testes diferenciais. E o back-end é trocável, porque o TAC não depende dele.

A direção para a Entrega 4, cujo formato final continua aberto, é gerar o JavaScript **a partir do TAC otimizado, não da AST**, como um compilador de C gera assembly (e como o MiniC prevê TAC → RISC-V ou LLVM IR), e ao contrário do `tsc` e do Babel, que geram JavaScript legível direto da AST. Como o TAC tem saltos e o JavaScript não tem `goto`, a saída é um laço com `switch` sobre uma variável de estado, o padrão do regenerator e do transform de geradores do `tsc`: cada rótulo vira um `case`, cada salto vira `estado = N`. Essa mesma variável é o que vai para o Checkpoint num Ponto de suspensão, então um único mecanismo resolve os saltos e a retomada. Ver `docs/design.md`, Seção 5.1.

## Considered Options

- **Manter TypeScript no semestre e portar para Rust depois** (o ADR 0003): descartada. Perde o MiniC como referência em cada Entrega, e o plugin de transpilador pós-semestre viraria port em vez de reaproveitamento.
- **Combinators próprios em Rust**: ver acima; o `nom` resolve o mesmo problema com a técnica do MiniC.
- **Opção B, bytecode próprio executado por uma VM em Rust** (Checkpoint serializado com `serde` e gravado com `rusqlite`): fica de reserva como **segundo back-end sobre o mesmo TAC**. O projeto voltaria a ter uma linguagem só e a descida ficaria mais baixa, ao custo de projetar o bytecode, a representação dos valores e a serialização, de transformar os efeitos dos Steps em built-ins ou numa ponte Rust↔JS, e de perder o oráculo e o caminho pós-semestre. Se for adotada no semestre, substitui o back-end JavaScript na Entrega 4 (não os dois), e o back-end JavaScript vai para depois do semestre.
- **Reconstruir `if`/`while` a partir dos saltos do TAC**, para gerar JavaScript legível: exige um algoritmo próprio, como o Relooper do Emscripten, e ainda precisaria de uma variável de estado para retomar.
- **Gerar o JavaScript a partir da AST**, como o `tsc` e o Babel: o código executado deixaria de ser o que as otimizações e a liveness analisaram.
- **Dormant como plugin de transpilador sobre TypeScript real (Babel, SWC, oxc)**, já no semestre: rejeitado. O parser passa a ser do hospedeiro, o que elimina a Entrega 2. Os hospedeiros trabalham arquivo a arquivo e sem tipos, e o Snapshot precisa de tipos para verificar que o que está vivo num Ponto de suspensão é serializável. O plugin SWC do Vercel Workflow dispensa tipos porque não transforma o corpo do workflow, que fica intacto para o Replay. O único hospedeiro com tipos, o `tsc` com `customTransformers`, é a linha em JS; o TypeScript 7 saiu sem API. TypeScript real ainda traria de volta os cortes do ADR 0004 (closures, objetos, `await` em laços), e o plugin acabaria definindo um subconjunto mesmo assim. Ver `docs/research/compiler-landscape.md`, seção 10.4.
- **Plugin do Bun (`onLoad`) como distribuição**: fora da v1. Com o compilador em Rust, exigiria Wasm, FFI ou subprocesso.
- **Go, Haskell, Bend; C, WebAssembly ou RISC-V como alvo**: os motivos do ADR 0003 continuam valendo. Com C ou LLVM, o `-O2` ainda refaria as otimizações e apagaria o efeito delas.

## Consequences

- A curva do Rust vem junto com a de compiladores. A mitigação é o `nom` e o escopo mínimo da Entrega 2: parser, AST e testes.
- O projeto passa a ter duas linguagens: o compilador em Rust; o Runtime e o harness dos testes diferenciais em JavaScript. O argumento "uma linguagem só" do ADR 0003 deixa de valer, e os testes diferenciais chamam o binário do compilador.
- No semestre, o compilador é só uma CLI: `dormant build arquivo.ts` e, na Entrega 2, `dormant parse arquivo.ts`, que imprime a AST. Adaptadores de plugin para ferramentas de build ficam para depois da v1.
- O formato de fases nomeadas do pi-durable, referência inicial no ADR 0003, dá lugar ao laço com `switch`; o esboço com fases em `docs/design.md` fica como ilustração da máquina de estados.
- A geração de TAC recebe a AST por uma fronteira estreita desde o início, para que um plugin de transpilador possa alimentar a mesma análise depois do semestre.
