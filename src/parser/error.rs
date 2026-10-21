//! O erro de sintaxe que o parser devolve: linha, coluna e mensagem.

use std::fmt;

/// Um erro de sintaxe. A linha e a coluna contam a partir de 1, e a coluna
/// conta caracteres, não bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Converte o erro do `nom` num `ParseError`. O `nom` guarda a entrada que
    /// sobrou no ponto da falha; o offset da falha é o tamanho total menos o
    /// tamanho desse resto.
    pub(crate) fn from_nom(source: &str, err: nom::Err<nom::error::Error<&str>>) -> Self {
        let rest = match &err {
            nom::Err::Error(e) | nom::Err::Failure(e) => e.input,
            // Os parsers de token vêm de `complete`, que nunca pedem mais
            // entrada; se acontecer, a falha é no fim do arquivo.
            nom::Err::Incomplete(_) => "",
        };
        let offset = source.len() - rest.len();
        let (line, column) = line_and_column(source, offset);
        ParseError {
            line,
            column,
            message: "erro de sintaxe".to_string(),
        }
    }
}

/// A linha e a coluna (a partir de 1) do byte `offset` de `source`.
fn line_and_column(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.matches('\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    let column = before[line_start..].chars().count() + 1;
    (line, column)
}
