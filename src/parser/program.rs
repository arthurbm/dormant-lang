//! O programa: funções, parâmetros, tipos e diretivas.

use nom::{
    IResult, Parser,
    branch::alt,
    combinator::{cut, map},
    multi::many0,
    sequence::{pair, preceded, terminated},
};

use crate::ast::Type;
use crate::parser::lexical::{ident, keyword, symbol};

/// `Type ::= "Promise" "<" Type ">" | ident ( "[" "]" )*`
///
/// `Promise` não é palavra reservada: sem o `<` logo depois, é um nome
/// comum, como `int`. O `cut` só vem depois de `Promise<`.
pub fn type_(input: &str) -> IResult<&str, Type> {
    alt((
        map(
            preceded(
                pair(keyword("Promise"), symbol("<")),
                cut(terminated(type_, symbol(">"))),
            ),
            |inner| Type::Promise(Box::new(inner)),
        ),
        map(
            pair(ident, many0(pair(symbol("["), symbol("]")))),
            |(name, dims)| Type::Named {
                name: name.to_string(),
                dims: dims.len(),
            },
        ),
    ))
    .parse(input)
}
