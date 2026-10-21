// O Workflow da fatura com lembrete de pagamento (docs/design.md, seção 2).
// Os corpos dos Steps são de mentira: na aplicação real eles consultariam um
// banco ou chamariam o gateway de pagamento.

async function buscarValor(pedido: int): Promise<int> {
  "use step"; // Step: efeito colateral registrado
  return pedido * 100;
}

async function emitirFatura(pedido: int, total: int): Promise<string> {
  "use step";
  return "fatura-" + pedido + "-" + total;
}

async function consultarPagamento(fatura: string): Promise<boolean> {
  "use step";
  return fatura === "fatura-paga";
}

async function enviarLembrete(fatura: string): Promise<void> {
  "use step";
  console.log("Lembrete: a fatura " + fatura + " ainda não foi paga.");
}

export async function cobrar(pedido: int): Promise<string> {
  "use workflow"; // Workflow: execução que sobrevive a reinícios
  const valor = await buscarValor(pedido);
  const total = valor + 500;
  const fatura = await emitirFatura(pedido, total);
  await sleep(3 * DIA); // prazo para o cliente pagar; o processo pode morrer aqui
  const pago = await consultarPagamento(fatura);
  if (pago) {
    return "pago";
  }
  await enviarLembrete(fatura);
  await sleep(2 * DIA);
  const pagoDepois = await consultarPagamento(fatura);
  if (pagoDepois) {
    return "pago depois do lembrete";
  }
  return "em atraso";
}
