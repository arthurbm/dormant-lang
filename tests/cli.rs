//! Testes da CLI: rodam o binário compilado e conferem a saída e o código de saída.

use std::process::{Command, Output};

fn dormant(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dormant"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("o binário dormant deveria rodar")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn parse_de_arquivo_valido_imprime_a_ast_e_sai_com_0() {
    let out = dormant(&["parse", "tests/fixtures/valid/minimal.ts"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("Program {"), "stdout: {text}");
    assert!(text.contains("name: \"main\""), "stdout: {text}");
    assert!(stderr(&out).is_empty());
}

#[test]
fn parse_de_arquivo_invalido_imprime_arquivo_linha_coluna_e_sai_com_1() {
    let path = "tests/fixtures/invalid/missing-semicolon.ts";
    let out = dormant(&["parse", path]);
    assert_eq!(out.status.code(), Some(1));
    let text = stderr(&out);
    assert!(
        text.starts_with(&format!("{path}:2:12: ")),
        "stderr: {text}"
    );
    assert!(stdout(&out).is_empty());
}

#[test]
fn sem_argumentos_imprime_o_uso_e_sai_com_2() {
    let out = dormant(&[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("uso: dormant parse <arquivo>"));
}

#[test]
fn subcomando_desconhecido_sai_com_2() {
    let out = dormant(&["build", "tests/fixtures/valid/minimal.ts"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("uso:"));
}

#[test]
fn arquivo_ilegivel_sai_com_2() {
    let out = dormant(&["parse", "tests/fixtures/nao-existe.ts"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("tests/fixtures/nao-existe.ts"));
}
