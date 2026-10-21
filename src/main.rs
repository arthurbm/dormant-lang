//! A CLI do Dormant: `dormant parse <arquivo>`.
//!
//! Lê os argumentos com a biblioteca padrão. Um parser de argumentos (como o
//! `clap`) só passa a valer a pena com um segundo subcomando.

use std::process::ExitCode;

const USAGE: &str = "uso: dormant parse <arquivo>";

/// Código de saída para erro de sintaxe no arquivo.
const SYNTAX_ERROR: u8 = 1;
/// Código de saída para uso errado da CLI ou arquivo ilegível.
const USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [command, path] = args.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(USAGE_ERROR);
    };
    if command != "parse" {
        eprintln!("{USAGE}");
        return ExitCode::from(USAGE_ERROR);
    }
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("{path}: {err}");
            eprintln!("{USAGE}");
            return ExitCode::from(USAGE_ERROR);
        }
    };
    match dormant::parse_program(&source) {
        Ok(program) => {
            println!("{program:#?}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{path}:{err}");
            ExitCode::from(SYNTAX_ERROR)
        }
    }
}
