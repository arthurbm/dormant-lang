//! Camada léxica: os tokens da gramática, sem um lexer separado.
//!
//! Não existe uma lista de tokens. Cada parser de token reconhece o próprio
//! texto e depois consome o espaço e os comentários que vêm em seguida
//! ([`lexeme`]). Assim, quem chama um token já recebe o resto da entrada
//! posicionado no próximo token. Só o começo do arquivo precisa de um [`ws`]
//! explícito, e quem faz isso é o parser do programa.

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_while, take_while1},
    character::complete::{anychar, char, multispace1, none_of, one_of, satisfy},
    combinator::{cut, map_res, not, recognize, value, verify},
    error::Error,
    multi::many0,
    sequence::{pair, preceded, terminated},
};

/// As palavras reservadas de `docs/grammar.md`. `Promise` e os nomes de tipo
/// (`int`, `string`…) ficam de fora: para o parser eles são `ident`.
pub const RESERVED: &[&str] = &[
    "async", "await", "const", "else", "export", "false", "function", "if", "let", "return",
    "true", "while",
];

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// `whitespace` e `comment`: zero ou mais espaços (`[ \t\r\n]`) e comentários
/// de linha (`// …` até o fim da linha), descartados.
pub fn ws(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(alt((
            multispace1,
            recognize(pair(tag("//"), take_till(|c| c == '\n'))),
        ))),
    )
    .parse(input)
}

/// Aplica `p` e depois consome o espaço e os comentários seguintes.
pub fn lexeme<'a, O, P>(p: P) -> impl Parser<&'a str, Output = O, Error = Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = Error<&'a str>>,
{
    terminated(p, ws)
}

/// Uma palavra reservada, só como palavra inteira: `while` não casa com o
/// começo de `whilex`, porque o caractere seguinte não pode continuar um
/// identificador.
pub fn keyword<'a>(
    k: &'static str,
) -> impl Parser<&'a str, Output = &'a str, Error = Error<&'a str>> {
    lexeme(terminated(tag(k), not(satisfy(is_ident_char))))
}

/// Os caracteres que, logo depois de `s`, formariam um operador mais longo.
/// É o que faz os operadores casarem pelo mais longo: `<` não casa com o
/// começo de `<=`, o `=` da atribuição não casa com o começo de `===`, e o
/// `!` unário não casa com o começo de `!==`.
fn longer_operator_continuations(s: &str) -> &'static str {
    match s {
        "<" | ">" | "=" | "!" => "=",
        "-" => "-",
        "+" => "+",
        _ => "",
    }
}

/// Um símbolo (pontuação ou operador), pelo casamento mais longo.
pub fn symbol<'a>(
    s: &'static str,
) -> impl Parser<&'a str, Output = &'a str, Error = Error<&'a str>> {
    lexeme(terminated(
        tag(s),
        not(one_of(longer_operator_continuations(s))),
    ))
}

/// `ident ::= [A-Za-z_] [A-Za-z0-9_]*`, exceto as palavras reservadas.
pub fn ident(input: &str) -> IResult<&str, &str> {
    lexeme(verify(
        recognize(pair(satisfy(is_ident_start), take_while(is_ident_char))),
        |s: &str| !RESERVED.contains(&s),
    ))
    .parse(input)
}

/// Os dígitos de um inteiro: `"0" | [1-9] [0-9]*`.
///
/// Usa `take_while` em vez de `digit0`/`digit1`: no `nom` 8.0.0,
/// `recognize(pair(satisfy(..), digit0))` no fim da entrada devolve só o
/// primeiro dígito (`"123"` vira `"1"`), e o resto some.
fn int_digits(input: &str) -> IResult<&str, &str> {
    alt((
        tag("0"),
        recognize(pair(
            satisfy(|c| ('1'..='9').contains(&c)),
            take_while(is_digit),
        )),
    ))
    .parse(input)
}

/// `int ::= "0" | [1-9] [0-9]*`, como `i64`. Um literal que não cabe em `i64`
/// é erro de sintaxe.
pub fn int(input: &str) -> IResult<&str, i64> {
    lexeme(map_res(int_digits, str::parse::<i64>)).parse(input)
}

/// `float ::= ( "0" | [1-9] [0-9]* ) "." [0-9]+`, como `f64`.
pub fn float(input: &str) -> IResult<&str, f64> {
    lexeme(map_res(
        recognize((int_digits, char('.'), take_while1(is_digit))),
        str::parse::<f64>,
    ))
    .parse(input)
}

/// `string ::= '"' ( [^"\\\n] | "\\" . )* '"'`. Guarda o texto entre as
/// aspas como foi escrito, com os escapes intactos, para que o gerador de
/// JavaScript possa copiá-lo. Depois da aspa de abertura o parser usa `cut`:
/// uma string sem fechar é erro no ponto em que ela acaba, não no começo.
pub fn string(input: &str) -> IResult<&str, String> {
    let content = recognize(many0(alt((
        value((), none_of("\"\\\n")),
        value((), pair(char('\\'), anychar)),
    ))));
    lexeme(preceded(char('"'), cut(terminated(content, char('"')))))
        .map(str::to_string)
        .parse(input)
}
