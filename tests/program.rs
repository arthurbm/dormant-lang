//! Testes de unidade do programa: funções, parâmetros, tipos e diretivas.

mod common;

use common::*;
use dormant::ast::Type;
use dormant::parser::program::type_;

// ---------- Type ----------

#[test]
fn type_nome_simples() {
    assert_eq!(type_("int"), Ok(("", named("int", 0))));
    assert_eq!(type_("boolean"), Ok(("", named("boolean", 0))));
}

#[test]
fn type_listas() {
    assert_eq!(type_("int[]"), Ok(("", named("int", 1))));
    assert_eq!(type_("string[][]"), Ok(("", named("string", 2))));
}

#[test]
fn type_promise_de_lista() {
    assert_eq!(
        type_("Promise<int[]>"),
        Ok(("", Type::Promise(Box::new(named("int", 1)))))
    );
}

#[test]
fn type_promise_aninhada() {
    assert_eq!(
        type_("Promise<Promise<void>>"),
        Ok((
            "",
            Type::Promise(Box::new(Type::Promise(Box::new(named("void", 0)))))
        ))
    );
}

#[test]
fn type_promise_e_palavra_inteira() {
    assert_eq!(type_("Promises"), Ok(("", named("Promises", 0))));
}
