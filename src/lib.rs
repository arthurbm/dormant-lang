//! Dormant: um subconjunto de TypeScript para workflows duráveis.
//!
//! A biblioteca tem a AST ([`ast`]) e o parser ([`parser`]). O binário
//! `dormant` (em `src/main.rs`) é uma camada fina sobre a biblioteca.

pub mod ast;
pub mod parser;

pub use parser::{ParseError, parse_program};
