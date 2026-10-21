//! Expressões: de `Expr` até `Primary`, uma função por nível de precedência.
//!
//! A ordem segue `docs/grammar.md`: `Or` agrupa por último e `Postfix`
//! agrupa primeiro. Cada nível binário tem a forma `X ( op X )*`: lê o
//! primeiro operando e depois um laço de pares (operador, operando), dobrados
//! à esquerda. Assim `a - b - c` vira `(a - b) - c`. A gramática não tem
//! recursão à esquerda (`Add ::= Add "+" Mul`), porque um parser descendente
//! chamaria `add` de novo sem consumir nada e entraria em laço infinito.

use nom::{
    IResult, Parser,
    branch::alt,
    combinator::{cut, map, opt, value},
    error::Error,
    multi::{many0, separated_list1},
    sequence::{pair, preceded, terminated},
};

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::parser::lexical::{float, ident, int, keyword, string, symbol};

/// O tipo de um parser de um nível de precedência.
type Level = fn(&str) -> IResult<&str, Expr>;

/// `X ( op X )*`, com associatividade à esquerda.
///
/// Depois de um operador o parser usa `cut`: um operador sem operando à
/// direita (`a + ;`) é erro ali mesmo, porque nenhuma outra regra da
/// gramática aceita um operador binário solto.
fn left_assoc<'a, O>(input: &'a str, operand: Level, op: O) -> IResult<&'a str, Expr>
where
    O: Parser<&'a str, Output = BinaryOp, Error = Error<&'a str>>,
{
    let (input, first) = operand(input)?;
    let (input, rest) = many0(pair(op, cut(operand))).parse(input)?;
    let tree = rest.into_iter().fold(first, |lhs, (op, rhs)| Expr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    });
    Ok((input, tree))
}

/// `Expr ::= Or`
pub fn expr(input: &str) -> IResult<&str, Expr> {
    or(input)
}

/// `Or ::= And ( "||" And )*`
pub fn or(input: &str) -> IResult<&str, Expr> {
    left_assoc(input, and, value(BinaryOp::Or, symbol("||")))
}

/// `And ::= Eq ( "&&" Eq )*`
pub fn and(input: &str) -> IResult<&str, Expr> {
    left_assoc(input, eq, value(BinaryOp::And, symbol("&&")))
}

/// `Eq ::= Rel ( ( "===" | "!==" ) Rel )*`
pub fn eq(input: &str) -> IResult<&str, Expr> {
    left_assoc(
        input,
        rel,
        alt((
            value(BinaryOp::StrictEq, symbol("===")),
            value(BinaryOp::StrictNe, symbol("!==")),
        )),
    )
}

/// `Rel ::= Add ( ( "<" | "<=" | ">" | ">=" ) Add )*`
pub fn rel(input: &str) -> IResult<&str, Expr> {
    left_assoc(
        input,
        add,
        alt((
            value(BinaryOp::Le, symbol("<=")),
            value(BinaryOp::Lt, symbol("<")),
            value(BinaryOp::Ge, symbol(">=")),
            value(BinaryOp::Gt, symbol(">")),
        )),
    )
}

/// `Add ::= Mul ( ( "+" | "-" ) Mul )*`
pub fn add(input: &str) -> IResult<&str, Expr> {
    left_assoc(
        input,
        mul,
        alt((
            value(BinaryOp::Add, symbol("+")),
            value(BinaryOp::Sub, symbol("-")),
        )),
    )
}

/// `Mul ::= Unary ( ( "*" | "/" | "%" ) Unary )*`
pub fn mul(input: &str) -> IResult<&str, Expr> {
    left_assoc(
        input,
        unary,
        alt((
            value(BinaryOp::Mul, symbol("*")),
            value(BinaryOp::Div, symbol("/")),
            value(BinaryOp::Rem, symbol("%")),
        )),
    )
}

/// `Unary ::= ( "!" | "-" ) Unary | Postfix`
pub fn unary(input: &str) -> IResult<&str, Expr> {
    let op = alt((
        value(UnaryOp::Not, symbol("!")),
        value(UnaryOp::Neg, symbol("-")),
    ));
    alt((
        map(pair(op, cut(unary)), |(op, operand)| Expr::Unary {
            op,
            operand: Box::new(operand),
        }),
        postfix,
    ))
    .parse(input)
}

/// Um sufixo de `Postfix`, ainda sem o alvo em que ele se aplica.
enum Suffix {
    Call(Vec<Expr>),
    Index(Expr),
    Member(String),
}

/// `Postfix ::= Primary ( "(" Args? ")" | "[" Expr "]" | "." ident )*`
///
/// Os sufixos se aplicam em sequência, da esquerda para a direita:
/// `f(x)[0]` é a indexação do resultado da chamada. Depois de `(`, `[` ou `.`
/// o parser usa `cut`, porque nenhuma outra regra aceita esses símbolos
/// depois de uma expressão. É o que faz `g(await f())` acusar o `await`.
pub fn postfix(input: &str) -> IResult<&str, Expr> {
    let (input, first) = primary(input)?;
    let suffix = alt((
        map(
            preceded(symbol("("), cut(terminated(opt(args), symbol(")")))),
            |args| Suffix::Call(args.unwrap_or_default()),
        ),
        map(
            preceded(symbol("["), cut(terminated(expr, symbol("]")))),
            Suffix::Index,
        ),
        map(preceded(symbol("."), cut(ident)), |name| {
            Suffix::Member(name.to_string())
        }),
    ));
    let (input, suffixes) = many0(suffix).parse(input)?;
    let tree = suffixes.into_iter().fold(first, |target, s| match s {
        Suffix::Call(args) => Expr::Call {
            callee: Box::new(target),
            args,
        },
        Suffix::Index(index) => Expr::Index {
            target: Box::new(target),
            index: Box::new(index),
        },
        Suffix::Member(property) => Expr::Member {
            object: Box::new(target),
            property,
        },
    });
    Ok((input, tree))
}

/// `Args ::= Expr ( "," Expr )*`
pub fn args(input: &str) -> IResult<&str, Vec<Expr>> {
    separated_list1(symbol(","), expr).parse(input)
}

/// `Primary ::= int | float | string | "true" | "false" | ident
///            | "[" Args? "]" | "(" Expr ")"`
///
/// `float` vem antes de `int`, porque `int` casaria com o `1` de `1.5`.
/// Os parênteses só agrupam: não viram um nó da AST.
pub fn primary(input: &str) -> IResult<&str, Expr> {
    alt((
        map(float, Expr::Float),
        map(int, Expr::Int),
        map(string, Expr::Str),
        value(Expr::Bool(true), keyword("true")),
        value(Expr::Bool(false), keyword("false")),
        map(ident, |name| Expr::Ident(name.to_string())),
        map(
            preceded(symbol("["), cut(terminated(opt(args), symbol("]")))),
            |items| Expr::List(items.unwrap_or_default()),
        ),
        preceded(symbol("("), cut(terminated(expr, symbol(")")))),
    ))
    .parse(input)
}
