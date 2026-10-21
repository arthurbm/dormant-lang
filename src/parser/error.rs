//! O erro de sintaxe que o parser devolve: linha, coluna e mensagem.

use std::fmt;

use nom::error::ErrorKind;

use crate::parser::lexical::{EXPECTED_SEMICOLON, RESERVED, is_ident_char};

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
    /// Converte o erro do `nom` num `ParseError`.
    ///
    /// O `nom` guarda a entrada que sobrou no ponto da falha. Como ela é
    /// sempre um sufixo do texto, o offset da falha é o tamanho total menos o
    /// tamanho desse resto.
    pub(crate) fn from_nom(source: &str, err: nom::Err<nom::error::Error<&str>>) -> Self {
        let (rest, kind) = match &err {
            nom::Err::Error(e) | nom::Err::Failure(e) => (e.input, e.code),
            // Os parsers de token vêm de `complete`, que nunca pedem mais
            // entrada; se acontecer, a falha é no fim do arquivo.
            nom::Err::Incomplete(_) => ("", ErrorKind::Complete),
        };
        let mut offset = source.len() - rest.len();
        if kind == EXPECTED_SEMICOLON {
            offset = end_of_previous_token(source, offset);
        }
        let (line, column) = line_and_column(source, offset);
        ParseError {
            line,
            column,
            message: message(rest, kind),
        }
    }
}

/// Cada token consome o espaço que vem depois dele, então um `;` faltando
/// só é percebido no começo do token seguinte, muitas vezes na linha de
/// baixo. Para esse erro o offset recua sobre o espaço até o fim do último
/// token, que é onde o `;` devia estar. (Se houver um comentário entre os
/// dois, o recuo para no fim do comentário.)
fn end_of_previous_token(source: &str, offset: usize) -> usize {
    source[..offset]
        .trim_end_matches([' ', '\t', '\r', '\n'])
        .len()
}

/// A mensagem: o que foi encontrado no ponto da falha, com o que faltou
/// quando o parser sabe (`;`) e uma dica quando é uma palavra reservada.
fn message(rest: &str, kind: ErrorKind) -> String {
    let found = match found_token(rest) {
        None => "o fim do arquivo".to_string(),
        Some("\n") | Some("\r") => "o fim da linha".to_string(),
        Some(token) => format!("`{token}`"),
    };
    if kind == ErrorKind::Eof {
        return format!("texto sobrando depois da última função: {found}");
    }
    let hint = match found.trim_matches('`') {
        "await" => " (`await` só pode vir no começo de um comando ou logo depois de `=`)",
        word if RESERVED.contains(&word) => " (palavra reservada)",
        _ => "",
    };
    if kind == EXPECTED_SEMICOLON {
        return format!("esperava `;`, encontrou {found}{hint}");
    }
    format!("erro de sintaxe: encontrou {found}{hint}")
}

/// O token que começa em `rest`: uma palavra inteira ou um único caractere.
fn found_token(rest: &str) -> Option<&str> {
    let first = rest.chars().next()?;
    if is_ident_char(first) {
        let end = rest.find(|c| !is_ident_char(c)).unwrap_or(rest.len());
        Some(&rest[..end])
    } else {
        Some(&rest[..first.len_utf8()])
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
