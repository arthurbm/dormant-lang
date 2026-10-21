//! Comandos e blocos: de `Stmt` até `Block`.
//!
//! Depois de reconhecer a palavra reservada que abre um comando (`let`,
//! `const`, `if`, `while`, `return`), o parser usa `cut`. A partir dali o
//! texto só pode ser aquele comando, então um erro dentro dele vira
//! `Err::Failure`, que atravessa o `alt` de [`stmt`] sem voltar atrás e
//! aponta o lugar real da falha.

use nom::{
    IResult, Parser,
    branch::alt,
    combinator::{cut, map, opt, value},
    multi::many_till,
    sequence::{preceded, terminated},
};

use crate::ast::{Block, DeclKind, Else, If, Stmt, Value};
use crate::parser::expressions::expr;
use crate::parser::lexical::{ident, keyword, semicolon, symbol};
use crate::parser::program::type_;

/// `Stmt ::= VarDecl | Assign | If | While | Return | ExprStmt`
///
/// `Assign` vem antes de `ExprStmt` porque os dois começam com `ident`.
/// Sem o `=`, o `Assign` falha com `Err::Error` e o `alt` tenta `ExprStmt`
/// na mesma entrada.
pub fn stmt(input: &str) -> IResult<&str, Stmt> {
    alt((
        var_decl,
        map(if_stmt, Stmt::If),
        while_stmt,
        return_stmt,
        assign,
        expr_stmt,
    ))
    .parse(input)
}

/// `VarDecl ::= ( "let" | "const" ) ident ( ":" Type )? "=" Value ";"`
pub fn var_decl(input: &str) -> IResult<&str, Stmt> {
    let kind = alt((
        value(DeclKind::Let, keyword("let")),
        value(DeclKind::Const, keyword("const")),
    ));
    map(
        (
            kind,
            cut((
                ident,
                opt(preceded(symbol(":"), type_)),
                symbol("="),
                value_expr,
                semicolon,
            )),
        ),
        |(kind, (name, ty, _, value, _))| Stmt::VarDecl {
            kind,
            name: name.to_string(),
            ty,
            value,
        },
    )
    .parse(input)
}

/// `Assign ::= ident "=" Value ";"`
///
/// O `cut` vem depois do `=`, e não antes: até ali o texto ainda pode ser um
/// `ExprStmt` (`x === 1;`, `f(x);`). Depois de `ident "="` nenhuma outra
/// regra serve, então `x = 1` sem `;` acusa a falta do `;`, e não o `=`.
/// O `=` de [`symbol`] não casa com o começo de `===`.
pub fn assign(input: &str) -> IResult<&str, Stmt> {
    map(
        (ident, symbol("="), cut(terminated(value_expr, semicolon))),
        |(name, _, value)| Stmt::Assign {
            name: name.to_string(),
            value,
        },
    )
    .parse(input)
}

/// `ExprStmt ::= Value ";"`
pub fn expr_stmt(input: &str) -> IResult<&str, Stmt> {
    map(terminated(value_expr, semicolon), Stmt::Expr).parse(input)
}

/// `Value ::= "await"? Expr`
///
/// O nome não pode ser `value`, que já é um combinator do `nom`.
pub fn value_expr(input: &str) -> IResult<&str, Value> {
    alt((
        map(preceded(keyword("await"), cut(expr)), |expr| Value {
            awaited: true,
            expr,
        }),
        map(expr, |expr| Value {
            awaited: false,
            expr,
        }),
    ))
    .parse(input)
}

/// `If ::= "if" "(" Expr ")" Block ( "else" ( Block | If ) )?`
///
/// As chaves são obrigatórias, então o `else` só pode vir depois de `}` e
/// sempre pertence ao `if` daquele bloco: não existe "else pendurado".
pub fn if_stmt(input: &str) -> IResult<&str, If> {
    let else_branch = preceded(
        keyword("else"),
        cut(alt((
            map(block, Else::Block),
            map(if_stmt, |nested| Else::If(Box::new(nested))),
        ))),
    );
    map(
        preceded(
            keyword("if"),
            cut((symbol("("), expr, symbol(")"), block, opt(else_branch))),
        ),
        |(_, cond, _, then_block, else_branch)| If {
            cond,
            then_block,
            else_branch,
        },
    )
    .parse(input)
}

/// `While ::= "while" "(" Expr ")" Block`
pub fn while_stmt(input: &str) -> IResult<&str, Stmt> {
    map(
        preceded(
            keyword("while"),
            cut((symbol("("), expr, symbol(")"), block)),
        ),
        |(_, cond, _, body)| Stmt::While { cond, body },
    )
    .parse(input)
}

/// `Return ::= "return" Expr? ";"`
pub fn return_stmt(input: &str) -> IResult<&str, Stmt> {
    map(
        preceded(keyword("return"), cut(terminated(opt(expr), semicolon))),
        Stmt::Return,
    )
    .parse(input)
}

/// `Block ::= "{" Stmt* "}"`
pub fn block(input: &str) -> IResult<&str, Block> {
    preceded(symbol("{"), cut(stmts_until_brace)).parse(input)
}

/// `Stmt* "}"`, para [`block`] e para o corpo de uma função.
///
/// Usa `many_till` em vez de `terminated(many0(stmt), "}")`. A cada volta ele
/// tenta o `}` e, se não for, um `Stmt`. Se o `Stmt` falhar, o erro devolvido
/// é o dele, no ponto exato da falha (`x++;` acusa o `++`). Com `many0`, o
/// erro do `Stmt` seria descartado e sobraria só "esperava `}`" no começo do
/// comando.
pub(crate) fn stmts_until_brace(input: &str) -> IResult<&str, Block> {
    map(many_till(stmt, symbol("}")), |(stmts, _)| stmts).parse(input)
}
