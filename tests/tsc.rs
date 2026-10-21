//! Compatibilidade com TypeScript: todo programa válido do Dormant também é
//! TypeScript válido (issue #3, história 36).
//!
//! Ignorado por padrão, porque depende de Node e de rede (`npx` baixa o
//! pacote `typescript`). Para rodar: `cargo test --test tsc -- --ignored`.

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
#[ignore = "depende de Node e de rede; rode com --ignored"]
fn programas_validos_passam_no_tsc() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<String> = fs::read_dir(root.join("tests/fixtures/valid"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "ts"))
        .map(|path| path.display().to_string())
        .collect();
    files.sort();
    assert!(!files.is_empty());

    let output = Command::new("npx")
        .args(["-y", "-p", "typescript", "tsc", "--noEmit", "--strict"])
        // Sem a lib do DOM: `console` vem do prelúdio, como no Dormant.
        .args(["--target", "es2017", "--lib", "es2017"])
        // Cada arquivo é um módulo, para que funções de mesmo nome em
        // exemplos diferentes não colidam no escopo global.
        .args(["--moduleDetection", "force"])
        .arg(root.join("tests/fixtures/prelude.d.ts"))
        .args(&files)
        .current_dir(root)
        .output()
        .expect("não consegui rodar o npx; o Node está instalado?");

    assert!(
        output.status.success(),
        "o tsc recusou os exemplos:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
