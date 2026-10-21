//! Testes de unidade do programa: funções, parâmetros, tipos e diretivas.

mod common;

use common::*;
use dormant::ast::Type;
use dormant::parser::program::type_;

// ---------- Type ----------

#[test]
fn type_nome_simples() {
    assert_eq!(type_("int"), Ok(("", named("int", 0))));
    assert_eq!(type_("boolean"), Ok(("", named("boolean", 0))));
}

#[test]
fn type_listas() {
    assert_eq!(type_("int[]"), Ok(("", named("int", 1))));
    assert_eq!(type_("string[][]"), Ok(("", named("string", 2))));
}

#[test]
fn type_promise_de_lista() {
    assert_eq!(
        type_("Promise<int[]>"),
        Ok(("", Type::Promise(Box::new(named("int", 1)))))
    );
}

#[test]
fn type_promise_aninhada() {
    assert_eq!(
        type_("Promise<Promise<void>>"),
        Ok((
            "",
            Type::Promise(Box::new(Type::Promise(Box::new(named("void", 0)))))
        ))
    );
}

#[test]
fn type_promise_e_palavra_inteira() {
    assert_eq!(type_("Promises"), Ok(("", named("Promises", 0))));
}

// ---------- Param / Params ----------

use dormant::ast::{Directive, Function, Param, Program, Stmt};
use dormant::parse_program;
use dormant::parser::program::{directive, function, function_body, param, params, program};
use nom::Err;

fn p(name: &str, ty: Type) -> Param {
    Param {
        name: name.to_string(),
        ty,
    }
}

#[test]
fn param_nome_e_tipo() {
    assert_eq!(param("pedido: int"), Ok(("", p("pedido", named("int", 0)))));
}

#[test]
fn params_separados_por_virgula() {
    assert_eq!(
        params("a: int, xs: string[]"),
        Ok((
            "",
            vec![p("a", named("int", 0)), p("xs", named("string", 1))]
        ))
    );
}

#[test]
fn param_sem_tipo_e_erro() {
    assert!(param("a").is_err());
}

// ---------- Directive / FunctionBody ----------

#[test]
fn directive_workflow_e_step() {
    assert_eq!(
        directive(r#""use workflow";"#),
        Ok(("", Directive::Workflow))
    );
    assert_eq!(directive(r#""use step";"#), Ok(("", Directive::Step)));
}

#[test]
fn directive_exige_ponto_e_virgula_e_texto_exato() {
    assert!(directive(r#""use workflow""#).is_err());
    assert!(directive(r#""use strict";"#).is_err());
}

#[test]
fn function_body_com_diretiva_no_comeco() {
    assert_eq!(
        function_body(r#"{ "use step"; return 1; }"#),
        Ok((
            "",
            (Some(Directive::Step), vec![Stmt::Return(Some(int(1)))])
        ))
    );
}

#[test]
fn function_body_diretiva_fora_do_comeco_e_comando() {
    assert_eq!(
        function_body(r#"{ f(); "use step"; }"#),
        Ok((
            "",
            (
                None,
                vec![
                    Stmt::Expr(plain(call(id("f"), vec![]))),
                    Stmt::Expr(plain(str_lit("use step"))),
                ]
            )
        ))
    );
}

#[test]
fn function_body_string_que_nao_e_diretiva_e_comando() {
    assert_eq!(
        function_body(r#"{ "use workflow" + x; }"#),
        Ok((
            "",
            (
                None,
                vec![Stmt::Expr(plain(bin(
                    dormant::ast::BinaryOp::Add,
                    str_lit("use workflow"),
                    id("x")
                )))]
            )
        ))
    );
}

// ---------- Function ----------

#[test]
fn function_minima() {
    assert_eq!(
        function("function main(): void {}"),
        Ok((
            "",
            Function {
                export: false,
                is_async: false,
                name: "main".into(),
                params: vec![],
                return_type: named("void", 0),
                directive: None,
                body: vec![],
            }
        ))
    );
}

#[test]
fn function_export_async_com_diretiva() {
    let src = r#"export async function cobrar(pedido: int): Promise<string> {
        "use workflow";
        return "cobrado";
    }"#;
    assert_eq!(
        function(src),
        Ok((
            "",
            Function {
                export: true,
                is_async: true,
                name: "cobrar".into(),
                params: vec![p("pedido", named("int", 0))],
                return_type: Type::Promise(Box::new(named("string", 0))),
                directive: Some(Directive::Workflow),
                body: vec![Stmt::Return(Some(str_lit("cobrado")))],
            }
        ))
    );
}

#[test]
fn function_sem_tipo_de_retorno_e_erro() {
    assert!(matches!(function("function f() {}"), Err(Err::Failure(_))));
}

#[test]
fn functionx_nao_e_function() {
    assert!(matches!(
        function("functionx(): void {}"),
        Err(Err::Error(_))
    ));
}

// ---------- Program ----------

#[test]
fn program_vazio_e_so_comentarios() {
    assert_eq!(program(""), Ok(("", Program { functions: vec![] })));
    assert_eq!(
        program("  // só um comentário\n\n// e outro\n"),
        Ok(("", Program { functions: vec![] }))
    );
}

#[test]
fn program_varias_funcoes() {
    let (_, prog) = program("function a(): void {}\n// meio\nfunction b(): void {}").unwrap();
    let names: Vec<_> = prog.functions.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn program_recusa_texto_sobrando() {
    assert!(program("function a(): void {} @@@").is_err());
    assert!(program("function a(): void {} let x = 1;").is_err());
}

// ---------- parse_program (seam principal) ----------

#[test]
fn parse_program_devolve_a_ast() {
    assert_eq!(parse_program(""), Ok(Program { functions: vec![] }));
}

#[test]
fn parse_program_devolve_linha_e_coluna() {
    let err = parse_program("function a(): void {}\n  @@@").unwrap_err();
    assert_eq!((err.line, err.column), (2, 3));
}
