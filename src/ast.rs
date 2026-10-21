//! A AST da v1: o que o parser devolve.
//!
//! São `enum`s e `struct`s sem posições, para que o verificador e o
//! interpretador cubram cada construção por `match` exaustivo.

/// Uma expressão. Não existe caso para `await`: na v1 ele só aparece no começo
/// de um comando (ver `Value`), e a AST não consegue representar
/// `g(await f())`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    /// O texto entre as aspas, como foi escrito, com os escapes intactos.
    Str(String),
    Bool(bool),
    Ident(String),
    /// Literal de lista: `[1, 2, 3]`.
    List(Vec<Expr>),
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `callee(args)`.
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// `target[index]`.
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    /// `object.property`.
    Member {
        object: Box<Expr>,
        property: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// `!`
    Not,
    /// `-` unário
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// `||`
    Or,
    /// `&&`
    And,
    /// `===`
    StrictEq,
    /// `!==`
    StrictNe,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `%`
    Rem,
}
