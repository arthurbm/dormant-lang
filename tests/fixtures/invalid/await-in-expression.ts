async function main(): Promise<void> {
  "use workflow";
  g(await f());
}
