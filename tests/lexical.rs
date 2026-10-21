//! Testes de unidade da camada léxica (tokens de `docs/grammar.md`).
//!
//! Cada parser de token consome o espaço e os comentários que vêm depois dele,
//! então o resto esperado já começa no próximo token.

use dormant::parser::lexical::{float, ident, int, keyword, string, symbol, ws};
use nom::Parser;

#[test]
fn ident_reconhece_nome_e_consome_espaco_a_direita() {
    assert_eq!(ident("valor_1  + 2"), Ok(("+ 2", "valor_1")));
    assert_eq!(ident("_x;"), Ok((";", "_x")));
}

#[test]
fn ident_nao_comeca_com_digito() {
    assert!(ident("1abc").is_err());
}

#[test]
fn ident_recusa_palavras_reservadas() {
    for k in [
        "async", "await", "const", "else", "export", "false", "function", "if", "let", "return",
        "true", "while",
    ] {
        assert!(ident(k).is_err(), "{k} deveria ser recusada como ident");
    }
}

#[test]
fn ident_aceita_palavra_reservada_como_prefixo() {
    assert_eq!(ident("whilex = 1;"), Ok(("= 1;", "whilex")));
    assert_eq!(ident("constante"), Ok(("", "constante")));
    assert_eq!(ident("returnValue"), Ok(("", "returnValue")));
}

#[test]
fn nomes_de_tipo_e_promise_nao_sao_reservados() {
    for t in ["int", "float", "boolean", "string", "void", "Promise"] {
        assert_eq!(ident(t), Ok(("", t)));
    }
}

#[test]
fn keyword_casa_so_palavra_inteira() {
    assert_eq!(keyword("while").parse("while (x)"), Ok(("(x)", "while")));
    assert!(keyword("while").parse("whilex").is_err());
    assert_eq!(keyword("if").parse("if(x)"), Ok(("(x)", "if")));
}

#[test]
fn int_segue_a_gramatica() {
    assert_eq!(int("0"), Ok(("", 0)));
    assert_eq!(int("500 * 2"), Ok(("* 2", 500)));
    // "0" | [1-9][0-9]*: o zero à esquerda não faz parte do mesmo número.
    assert_eq!(int("07"), Ok(("7", 0)));
}

#[test]
fn int_e_float_no_fim_da_entrada_consomem_todos_os_digitos() {
    // Regressão: `recognize` com `digit0` do nom 8.0.0 devolvia só "1".
    assert_eq!(int("123"), Ok(("", 123)));
    assert_eq!(float("12.375"), Ok(("", 12.375)));
}

#[test]
fn int_fora_do_intervalo_de_i64_e_erro() {
    assert!(int("99999999999999999999").is_err());
}

#[test]
fn float_exige_digitos_dos_dois_lados_do_ponto() {
    assert_eq!(float("2.75 "), Ok(("", 2.75)));
    assert_eq!(float("0.5"), Ok(("", 0.5)));
    assert!(float("1.").is_err());
    assert!(float(".5").is_err());
    assert!(float("12").is_err());
}

#[test]
fn string_guarda_o_texto_sem_interpretar_escapes() {
    assert_eq!(string(r#""olá" ;"#), Ok((";", "olá".to_string())));
    assert_eq!(string(r#""a\"b\\n" "#), Ok(("", r#"a\"b\\n"#.to_string())));
    assert_eq!(string(r#""""#), Ok(("", String::new())));
}

#[test]
fn string_sem_fechar_e_erro() {
    assert!(string("\"abc").is_err());
    assert!(string("\"abc\ndef\"").is_err());
}

#[test]
fn ws_descarta_espacos_e_comentarios_de_linha() {
    assert_eq!(ws("  // um comentário\n\t x"), Ok(("x", ())));
    assert_eq!(ws("// até o fim"), Ok(("", ())));
    assert_eq!(ws("x"), Ok(("x", ())));
}

#[test]
fn token_consome_comentario_depois_dele() {
    assert_eq!(symbol("(").parse("(  // oi\n x"), Ok(("x", "(")));
}

#[test]
fn operadores_casam_pelo_mais_longo() {
    assert_eq!(symbol("<=").parse("<= b"), Ok(("b", "<=")));
    assert!(symbol("<").parse("<= b").is_err());
    assert!(symbol(">").parse(">= b").is_err());
    assert!(symbol("=").parse("=== b").is_err());
    assert!(symbol("!").parse("!== b").is_err());
    assert_eq!(symbol("=").parse("= b"), Ok(("b", "=")));
    assert_eq!(symbol("!").parse("!x"), Ok(("x", "!")));
}

#[test]
fn string_recusa_quebra_de_linha_crua_inclusive_cr() {
    // O TypeScript não aceita `\r` cru dentro de uma string.
    assert!(string("\"a\rb\"").is_err());
    assert_eq!(string(r#""a\rb""#), Ok(("", r"a\rb".to_string())));
}
