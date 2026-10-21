//! A AST da v1: o que o parser devolve.
//!
//! São `enum`s e `struct`s sem posições, para que o verificador e o
//! interpretador cubram cada construção por `match` exaustivo.

/// Um programa: a lista das funções do arquivo, na ordem em que aparecem.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}

/// `export? async? function nome(params): Tipo { diretiva? corpo }`
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub export: bool,
    pub is_async: bool,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
    /// `"use workflow"` ou `"use step"`, se for o primeiro comando do corpo.
    pub directive: Option<Directive>,
    pub body: Block,
}

/// `nome: Tipo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

/// A diretiva da Camada durável que marca uma função como Workflow ou Step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Directive {
    /// `"use workflow";`
    Workflow,
    /// `"use step";`
    Step,
}

/// Um tipo anotado. Os nomes (`int`, `string`, …) ficam como identificadores:
/// quem decide se um nome é um tipo válido é o verificador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// `Promise<T>`
    Promise(Box<Type>),
    /// Um nome seguido de `dims` pares de `[]`: `string[][]` tem `dims: 2`.
    Named { name: String, dims: usize },
}

/// Uma sequência de comandos entre chaves.
pub type Block = Vec<Stmt>;

/// Um comando.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let x: T = v;` ou `const x = v;`
    VarDecl {
        kind: DeclKind,
        name: String,
        ty: Option<Type>,
        value: Value,
    },
    /// `x = v;`
    Assign {
        name: String,
        value: Value,
    },
    If(If),
    While {
        cond: Expr,
        body: Block,
    },
    /// `return e;` ou `return;`
    Return(Option<Expr>),
    /// `v;`, como `await sleep(DIA);` ou `console.log(x);`
    Expr(Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclKind {
    Let,
    Const,
}

/// `Value ::= "await"? Expr`: o que pode vir depois do `=` de uma
/// declaração ou atribuição, ou sozinho num comando. É o único lugar da AST
/// em que um `await` cabe.
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub awaited: bool,
    pub expr: Expr,
}

/// `if (cond) { … }`, com `else` opcional.
#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub cond: Expr,
    pub then_block: Block,
    pub else_branch: Option<Else>,
}

/// O que vem depois de `else`: um bloco, ou outro `if` (o `else if`).
#[derive(Debug, Clone, PartialEq)]
pub enum Else {
    Block(Block),
    If(Box<If>),
}

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
