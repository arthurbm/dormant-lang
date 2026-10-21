//! Linha, coluna e mensagem dos erros de sintaxe, pela seam principal.
//!
//! Quando o que falta é algo no fim de uma linha (um `;`, por exemplo), o
//! erro aponta o fim do último token, como o `tsc`, e não o começo da linha
//! seguinte.

use std::fs;
use std::path::Path;

use dormant::{ParseError, parse_program};

fn error_of(fixture: &str) -> ParseError {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/invalid")
        .join(fixture);
    let source = fs::read_to_string(&path).unwrap();
    parse_program(&source).expect_err(fixture)
}

fn assert_position(fixture: &str, line: usize, column: usize) {
    let err = error_of(fixture);
    assert_eq!(
        (err.line, err.column),
        (line, column),
        "{fixture}: veio {err}"
    );
}

#[test]
fn if_sem_chaves_acusa_o_comando_solto() {
    // `  if (x) y();`: o `{` devia estar onde está o `y`.
    assert_position("if-without-braces.ts", 2, 10);
}

#[test]
fn await_dentro_de_expressao_acusa_o_await() {
    // `  g(await f());`
    assert_position("await-in-expression.ts", 3, 5);
}

#[test]
fn ponto_e_virgula_faltando_acusa_o_fim_da_linha() {
    // `  let x = 1` (sem `;`): o erro fica logo depois do `1`.
    assert_position("missing-semicolon.ts", 2, 12);
    // `  x = 1` (sem `;`): o `cut` do Assign evita acusar o `=`.
    assert_position("assign-missing-semicolon.ts", 2, 8);
}

#[test]
fn comando_invalido_no_meio_do_bloco_acusa_o_ponto_exato() {
    // `  x++;`: o `++` não existe na v1; o erro aponta o `+`, não o `x`.
    assert_position("increment.ts", 3, 4);
    // `  /* comentário de bloco */`
    assert_position("block-comment.ts", 2, 3);
}

#[test]
fn mensagem_de_ponto_e_virgula_faltando() {
    let message = error_of("missing-semicolon.ts").message;
    assert!(message.contains("esperava `;`"), "{message}");
    let message = error_of("assign-missing-semicolon.ts").message;
    assert!(message.contains("esperava `;`"), "{message}");
}

#[test]
fn texto_sobrando_acusa_o_comeco_da_sobra() {
    assert_position("trailing-text.ts", 3, 1);
}

#[test]
fn palavra_reservada_como_nome_acusa_a_palavra() {
    // `  let if = 1;`
    assert_position("reserved-word-as-name.ts", 2, 7);
}

#[test]
fn else_sem_chaves_acusa_o_comando_solto() {
    // `  } else z();`
    assert_position("else-without-braces.ts", 4, 10);
}

#[test]
fn string_sem_fechar_acusa_o_fim_da_linha() {
    // `  console.log("sem fechar);`
    assert_position("unterminated-string.ts", 2, 28);
}

#[test]
fn mensagem_mostra_o_que_foi_encontrado() {
    assert!(error_of("if-without-braces.ts").message.contains("`y`"));
    assert!(
        error_of("missing-semicolon.ts")
            .message
            .contains("`console`")
    );
}

#[test]
fn mensagem_de_texto_sobrando() {
    let message = error_of("trailing-text.ts").message;
    assert!(message.contains("depois da última função"), "{message}");
}

#[test]
fn mensagem_de_palavra_reservada() {
    let message = error_of("reserved-word-as-name.ts").message;
    assert!(message.contains("palavra reservada"), "{message}");
}

#[test]
fn mensagem_de_await_fora_do_comeco_do_comando() {
    let message = error_of("await-in-expression.ts").message;
    assert!(message.contains("começo de um comando"), "{message}");
}

#[test]
fn mensagem_de_fim_de_linha_e_de_arquivo() {
    let message = error_of("unterminated-string.ts").message;
    assert!(message.contains("fim da linha"), "{message}");
    assert!(!message.contains("esperava `;`"), "{message}");
    let err = parse_program("function f(): void {").unwrap_err();
    assert!(err.message.contains("fim do arquivo"), "{}", err.message);
    assert_eq!((err.line, err.column), (1, 21));
}

#[test]
fn coluna_conta_caracteres_e_nao_bytes() {
    // "ã" e "é" ocupam 2 bytes cada em UTF-8.
    let err = parse_program("// ação é\nfunction f(): void { é }").unwrap_err();
    assert_eq!((err.line, err.column), (2, 22));
}

#[test]
fn await_depois_de_return_ganha_a_dica_do_await() {
    let err =
        parse_program("async function f(): Promise<int> {\n  return await g();\n}").unwrap_err();
    assert_eq!((err.line, err.column), (2, 9));
    assert!(
        err.message.contains("começo de um comando"),
        "{}",
        err.message
    );
}
