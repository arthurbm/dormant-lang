//! Construtores curtos de AST para os testes escreverem a árvore esperada à mão.
#![allow(dead_code)]

use dormant::ast::{BinaryOp, Expr, UnaryOp};

pub fn id(name: &str) -> Expr {
    Expr::Ident(name.to_string())
}

pub fn int(n: i64) -> Expr {
    Expr::Int(n)
}

pub fn str_lit(s: &str) -> Expr {
    Expr::Str(s.to_string())
}

pub fn bin(op: BinaryOp, lhs: Expr, rhs: Expr) -> Expr {
    Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

pub fn un(op: UnaryOp, operand: Expr) -> Expr {
    Expr::Unary {
        op,
        operand: Box::new(operand),
    }
}

pub fn call(callee: Expr, args: Vec<Expr>) -> Expr {
    Expr::Call {
        callee: Box::new(callee),
        args,
    }
}

pub fn index(target: Expr, i: Expr) -> Expr {
    Expr::Index {
        target: Box::new(target),
        index: Box::new(i),
    }
}

pub fn member(object: Expr, property: &str) -> Expr {
    Expr::Member {
        object: Box::new(object),
        property: property.to_string(),
    }
}
