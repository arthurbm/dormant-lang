//! O programa: funções, parâmetros, tipos e diretivas.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    combinator::{all_consuming, cut, map, opt, value},
    multi::{many0, separated_list1},
    sequence::{pair, preceded, terminated},
};

use crate::ast::{Block, Directive, Function, Param, Program, Type};
use crate::parser::lexical::{ident, keyword, lexeme, symbol, ws};
use crate::parser::statements::stmt;

/// `Program ::= Function* EOF`
///
/// O espaço e os comentários do começo do arquivo são consumidos aqui; os
/// demais, por cada token. O `all_consuming` faz o parser ser estrito: texto
/// que sobra depois da última função é erro de sintaxe, e não é ignorado.
pub fn program(input: &str) -> IResult<&str, Program> {
    map(all_consuming(preceded(ws, many0(function))), |functions| {
        Program { functions }
    })
    .parse(input)
}

/// `Function ::= "export"? "async"? "function" ident "(" Params? ")" ":" Type FunctionBody`
///
/// O `cut` vem depois de `function`: dali em diante o texto só pode ser
/// uma função.
pub fn function(input: &str) -> IResult<&str, Function> {
    map(
        (
            opt(keyword("export")),
            opt(keyword("async")),
            keyword("function"),
            cut((
                ident,
                symbol("("),
                opt(params),
                symbol(")"),
                symbol(":"),
                type_,
                function_body,
            )),
        ),
        |(export, is_async, _, (name, _, params, _, _, return_type, (directive, body)))| Function {
            export: export.is_some(),
            is_async: is_async.is_some(),
            name: name.to_string(),
            params: params.unwrap_or_default(),
            return_type,
            directive,
            body,
        },
    )
    .parse(input)
}

/// `Params ::= Param ( "," Param )*`
pub fn params(input: &str) -> IResult<&str, Vec<Param>> {
    separated_list1(symbol(","), param).parse(input)
}

/// `Param ::= ident ":" Type`
pub fn param(input: &str) -> IResult<&str, Param> {
    map((ident, symbol(":"), type_), |(name, _, ty)| Param {
        name: name.to_string(),
        ty,
    })
    .parse(input)
}

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

/// `FunctionBody ::= "{" Directive? Stmt* "}"`
///
/// A diretiva só é reconhecida aqui, no começo do corpo, como em
/// TypeScript. Em qualquer outro lugar (depois de outro comando, dentro de um
/// `Block`), `"use step";` é um `ExprStmt` com uma string.
pub fn function_body(input: &str) -> IResult<&str, (Option<Directive>, Block)> {
    preceded(
        symbol("{"),
        cut(terminated(pair(opt(directive), many0(stmt)), symbol("}"))),
    )
    .parse(input)
}

/// `Directive ::= ( '"use workflow"' | '"use step"' ) ";"`
///
/// O texto da string precisa ser exatamente esse e vir seguido de `;`.
/// Se não for (`"use workflow" + x;`), o parser volta atrás e o texto é lido
/// como um comando comum.
pub fn directive(input: &str) -> IResult<&str, Directive> {
    terminated(
        alt((
            value(Directive::Workflow, lexeme(tag("\"use workflow\""))),
            value(Directive::Step, lexeme(tag("\"use step\""))),
        )),
        symbol(";"),
    )
    .parse(input)
}
