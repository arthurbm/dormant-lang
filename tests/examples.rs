//! Os programas de exemplo, pela seam principal (`parse_program`).
//!
//! Cada `.ts` de `tests/fixtures/valid/` tem ao lado um golden file `.ast`
//! com o `{:#?}` da AST esperada. Para regenerar os golden files depois de
//! uma mudança na AST, rode `BLESS=1 cargo test --test examples` e revise o
//! diff antes do commit. Cada `.ts` de `tests/fixtures/invalid/` precisa ser
//! recusado.

use std::fs;
use std::path::{Path, PathBuf};

use dormant::parse_program;

fn fixtures(dir: &str) -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(dir);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ts"))
        .collect();
    files.sort();
    files
}

/// Compara o `{:#?}` da AST de `source` com o golden file. Com `BLESS=1`,
/// reescreve o golden file em vez de comparar.
fn check_golden(source_path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(source_path).unwrap();
    let program =
        parse_program(&source).map_err(|err| format!("{}:{err}", source_path.display()))?;
    let actual = format!("{program:#?}\n");
    let golden_path = source_path.with_extension("ast");
    if std::env::var_os("BLESS").is_some() {
        fs::write(&golden_path, &actual).unwrap();
        return Ok(());
    }
    let expected = fs::read_to_string(&golden_path).map_err(|_| {
        format!(
            "falta o golden file {}; rode BLESS=1 cargo test --test examples",
            golden_path.display()
        )
    })?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "a AST de {} mudou; se a mudança é esperada, rode BLESS=1 cargo test --test examples\n--- esperado\n{expected}\n--- obtido\n{actual}",
            source_path.display()
        ))
    }
}

#[test]
fn programas_validos_batem_com_os_golden_files() {
    let files = fixtures("valid");
    assert!(files.len() >= 4, "faltam exemplos válidos: {files:?}");
    let failures: Vec<String> = files.iter().filter_map(|f| check_golden(f).err()).collect();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn programas_invalidos_sao_recusados() {
    let files = fixtures("invalid");
    assert!(files.len() >= 10, "faltam exemplos inválidos: {files:?}");
    let accepted: Vec<String> = files
        .iter()
        .filter(|f| parse_program(&fs::read_to_string(f).unwrap()).is_ok())
        .map(|f| f.display().to_string())
        .collect();
    assert!(accepted.is_empty(), "aceitos por engano: {accepted:?}");
}

#[test]
fn a_fatura_tem_um_workflow_e_quatro_steps() {
    use dormant::ast::Directive;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/invoice.ts");
    let program = parse_program(&fs::read_to_string(path).unwrap()).unwrap();
    let directives: Vec<_> = program.functions.iter().map(|f| f.directive).collect();
    assert_eq!(
        directives,
        [
            Some(Directive::Step),
            Some(Directive::Step),
            Some(Directive::Step),
            Some(Directive::Step),
            Some(Directive::Workflow),
        ]
    );
}
