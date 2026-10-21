// Um programa que usa cada construção da v1 (docs/grammar.md).

// Função sem export nem async, com parâmetros de tipos simples.
function media(a: float, b: float): float {
  return (a + b) / 2.0;
}

// Listas: tipo int[], literal, indexação, while, atribuição e todos os
// operadores aritméticos.
function soma(xs: int[], n: int): int {
  let total: int = 0;
  let i = 0;
  while (i < n) {
    total = total + xs[i] * 1 - 0 % 7;
    i = i + 1;
  }
  return total;
}

function primeiros(): int[] {
  return [1, 2, 3];
}

// Relacionais, igualdade estrita, lógicos, unários e else if.
function classificar(x: int, ativo: boolean): string {
  if (!ativo || x <= 0 && x !== -1) {
    return "inativo";
  } else if (x >= 100) {
    return "grande";
  } else if (x > 10 && x === 50) {
    return "cinquenta";
  } else {
    "use step"; // dentro de um bloco é só uma string, não diretiva
  }
  if (x < 5) {
    return "pequeno";
  }
  return "médio";
}

// Nomes que começam com palavras reservadas, listas de listas, strings com
// escapes, true/false e um return sem valor.
function nomes(): void {
  const whilex = 1;
  const constante: string[][] = [["a", "b"], []];
  let returnValue: boolean = true;
  returnValue = false;
  const texto = "linha\n\"aspas\"\\"; // escapes ficam como foram escritos
  const vazio: string[] = [];
  console.log(whilex, constante[0][1], returnValue, texto, vazio);
  return;
}

// Step com tipo de retorno Promise de lista.
export async function listar(n: int): Promise<int[]> {
  "use step";
  console.log("listando", n);
  return [n, n + 1];
}

// Workflow: await em declaração, em atribuição e como comando; chamadas
// encadeadas; comentário no meio de uma expressão.
export async function principal(): Promise<void> {
  "use workflow";
  const xs = await listar(3);
  let ys: int[] = xs;
  ys = await listar(soma(primeiros(), 3));
  await sleep(10 * SEGUNDO);
  const m = media(
    // um comentário entre os argumentos
    1.5,
    soma(ys, 2)
  );
  const primeiro = primeiros()[0];
  const rotulo = classificar(primeiro + -1, m > 0.5);
  console.log(rotulo);
}
