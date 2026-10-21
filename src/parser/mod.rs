//! O parser da v1, organizado pelas camadas de `docs/grammar.md`.
//!
//! | Módulo | Regras |
//! |---|---|
//! | [`lexical`] | `ident`, `int`, `float`, `string`, espaços, comentários, palavras reservadas, símbolos |
//! | [`expressions`] | de `Expr` até `Primary` |
//! | [`statements`] | `Stmt`, `VarDecl`, `Assign`, `ExprStmt`, `Value`, `If`, `While`, `Return`, `Block` |
//! | [`program`] | `Program`, `Function`, `Params`, `Param`, `Type`, `FunctionBody`, `Directive` |

mod error;
pub mod expressions;
pub mod lexical;
pub mod program;
pub mod statements;

pub use error::ParseError;

use crate::ast::Program;

/// Lê o texto de um programa inteiro e devolve a sua AST, ou o primeiro erro
/// de sintaxe com linha e coluna. É a porta de entrada da biblioteca.
pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    match program::program(source) {
        Ok((_, program)) => Ok(program),
        Err(err) => Err(ParseError::from_nom(source, err)),
    }
}
