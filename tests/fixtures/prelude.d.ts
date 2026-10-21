// As declarações do prelúdio do Dormant, para o `tsc` aceitar os exemplos
// (docs/design.md, seção 6.1, nível "de tipos").

type int = number;
type float = number;

declare function sleep(ms: int): Promise<void>;

declare const SEGUNDO: int;
declare const MINUTO: int;
declare const HORA: int;
declare const DIA: int;

declare const console: { log(...values: unknown[]): void };
