//! O parser da v1, organizado pelas camadas de `docs/grammar.md`.
//!
//! | Módulo | Regras |
//! |---|---|
//! | [`lexical`] | `ident`, `int`, `float`, `string`, espaços, comentários, palavras reservadas, símbolos |
//! | [`expressions`] | de `Expr` até `Primary` |

pub mod expressions;
pub mod lexical;
