//! O parser da v1, organizado pelas camadas de `docs/grammar.md`.
//!
//! | Módulo | Regras |
//! |---|---|
//! | [`lexical`] | `ident`, `int`, `float`, `string`, espaços, comentários, palavras reservadas, símbolos |
//! | [`expressions`] | de `Expr` até `Primary` |
//! | [`statements`] | `Stmt`, `VarDecl`, `Assign`, `ExprStmt`, `Value`, `If`, `While`, `Return`, `Block` |
//! | [`program`] | `Program`, `Function`, `Params`, `Param`, `Type`, `FunctionBody`, `Directive` |

pub mod expressions;
pub mod lexical;
pub mod program;
pub mod statements;
