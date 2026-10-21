//! Testes de unidade das expressões, de `Primary` até `Expr` (`docs/grammar.md`).

mod common;

use common::*;
use dormant::ast::{BinaryOp::*, Expr, UnaryOp::*};
use dormant::parser::expressions::{
    add, and, args, eq, expr, mul, or, postfix, primary, rel, unary,
};

// ---------- Primary ----------

#[test]
fn primary_literais() {
    assert_eq!(primary("42"), Ok(("", int(42))));
    assert_eq!(primary("2.5"), Ok(("", Expr::Float(2.5))));
    assert_eq!(primary(r#""oi\n""#), Ok(("", str_lit(r"oi\n"))));
    assert_eq!(primary("true"), Ok(("", Expr::Bool(true))));
    assert_eq!(primary("false"), Ok(("", Expr::Bool(false))));
    assert_eq!(primary("valor"), Ok(("", id("valor"))));
}

#[test]
fn primary_true_so_como_palavra_inteira() {
    assert_eq!(primary("trueish"), Ok(("", id("trueish"))));
}

#[test]
fn primary_lista() {
    assert_eq!(primary("[]"), Ok(("", Expr::List(vec![]))));
    assert_eq!(
        primary("[1, 2, 3]"),
        Ok(("", Expr::List(vec![int(1), int(2), int(3)])))
    );
}

#[test]
fn primary_parenteses_agrupam_sem_no_proprio() {
    assert_eq!(primary("(a)"), Ok(("", id("a"))));
}

#[test]
fn primary_recusa_await() {
    assert!(primary("await f()").is_err());
}

// ---------- Args ----------

#[test]
fn args_separados_por_virgula() {
    assert_eq!(args("a, 1, b"), Ok(("", vec![id("a"), int(1), id("b")])));
    assert_eq!(args("x)"), Ok((")", vec![id("x")])));
}

// ---------- Postfix ----------

#[test]
fn postfix_chamada_indexacao_e_membro() {
    assert_eq!(postfix("f()"), Ok(("", call(id("f"), vec![]))));
    assert_eq!(postfix("xs[0]"), Ok(("", index(id("xs"), int(0)))));
    assert_eq!(
        postfix("console.log(x)"),
        Ok(("", call(member(id("console"), "log"), vec![id("x")])))
    );
}

#[test]
fn postfix_encadeado() {
    assert_eq!(
        postfix("f(x)[0]"),
        Ok(("", index(call(id("f"), vec![id("x")]), int(0))))
    );
    assert_eq!(
        postfix("m[1][2]"),
        Ok(("", index(index(id("m"), int(1)), int(2))))
    );
}

#[test]
fn postfix_recusa_await_dentro_dos_argumentos() {
    assert!(postfix("g(await f())").is_err());
}

// ---------- Unary ----------

#[test]
fn unary_negacao_e_menos() {
    assert_eq!(unary("!ok"), Ok(("", un(Not, id("ok")))));
    assert_eq!(unary("-x"), Ok(("", un(Neg, id("x")))));
    assert_eq!(unary("!-x"), Ok(("", un(Not, un(Neg, id("x"))))));
}

#[test]
fn unary_aplica_sobre_o_postfix() {
    assert_eq!(
        unary("-f(x)"),
        Ok(("", un(Neg, call(id("f"), vec![id("x")]))))
    );
}

#[test]
fn unary_menos_menos_nao_e_dupla_negacao() {
    // Em TypeScript `--x` é decremento, fora da v1.
    assert!(unary("--x").is_err());
}

// ---------- Mul ----------

#[test]
fn mul_os_tres_operadores() {
    assert_eq!(mul("a * b"), Ok(("", bin(Mul, id("a"), id("b")))));
    assert_eq!(mul("a / b"), Ok(("", bin(Div, id("a"), id("b")))));
    assert_eq!(mul("a % b"), Ok(("", bin(Rem, id("a"), id("b")))));
}

#[test]
fn mul_associa_a_esquerda() {
    assert_eq!(
        mul("a / b * c"),
        Ok(("", bin(Mul, bin(Div, id("a"), id("b")), id("c"))))
    );
}

#[test]
fn mul_agrupa_depois_do_unary() {
    assert_eq!(mul("-a * b"), Ok(("", bin(Mul, un(Neg, id("a")), id("b")))));
}

// ---------- Add ----------

#[test]
fn add_associa_a_esquerda() {
    assert_eq!(
        add("a - b - c"),
        Ok(("", bin(Sub, bin(Sub, id("a"), id("b")), id("c"))))
    );
}

#[test]
fn add_precedencia_sobre_mul() {
    assert_eq!(
        add("valor + 500 * 2"),
        Ok(("", bin(Add, id("valor"), bin(Mul, int(500), int(2)))))
    );
}

#[test]
fn add_parenteses_mudam_a_precedencia() {
    assert_eq!(
        add("(a + b) * c"),
        Ok(("", bin(Mul, bin(Add, id("a"), id("b")), id("c"))))
    );
}

#[test]
fn add_operador_sem_operando_e_erro() {
    assert!(add("a + ").is_err());
}

// ---------- Rel ----------

#[test]
fn rel_operadores_longos_sao_um_so() {
    assert_eq!(rel("a <= b"), Ok(("", bin(Le, id("a"), id("b")))));
    assert_eq!(rel("a >= b"), Ok(("", bin(Ge, id("a"), id("b")))));
    assert_eq!(rel("a < b"), Ok(("", bin(Lt, id("a"), id("b")))));
    assert_eq!(rel("a > b"), Ok(("", bin(Gt, id("a"), id("b")))));
}

#[test]
fn rel_agrupa_depois_do_add() {
    assert_eq!(
        rel("a + 1 < b"),
        Ok(("", bin(Lt, bin(Add, id("a"), int(1)), id("b"))))
    );
}

// ---------- Eq ----------

#[test]
fn eq_igualdade_estrita() {
    assert_eq!(eq("a === b"), Ok(("", bin(StrictEq, id("a"), id("b")))));
    assert_eq!(eq("a !== b"), Ok(("", bin(StrictNe, id("a"), id("b")))));
}

#[test]
fn eq_agrupa_depois_do_rel() {
    assert_eq!(
        eq("a < b === c"),
        Ok(("", bin(StrictEq, bin(Lt, id("a"), id("b")), id("c"))))
    );
}

#[test]
fn eq_nao_aceita_igualdade_frouxa() {
    assert_eq!(eq("a == b"), Ok(("== b", id("a"))));
}

// ---------- And / Or ----------

#[test]
fn and_agrupa_depois_do_eq() {
    assert_eq!(
        and("a === 1 && b"),
        Ok(("", bin(And, bin(StrictEq, id("a"), int(1)), id("b"))))
    );
}

#[test]
fn or_agrupa_por_ultimo() {
    assert_eq!(
        or("a || b && c"),
        Ok(("", bin(Or, id("a"), bin(And, id("b"), id("c")))))
    );
}

#[test]
fn expr_e_or() {
    assert_eq!(
        expr("!a || b"),
        Ok(("", bin(Or, un(Not, id("a")), id("b"))))
    );
}
