//! Testes de unidade dos comandos e blocos (`docs/grammar.md`, de `Stmt` a `Block`).

mod common;

use common::*;
use dormant::ast::{BinaryOp::*, DeclKind, Else, If, Stmt};
use dormant::parser::statements::{
    assign, block, expr_stmt, if_stmt, return_stmt, stmt, value_expr, var_decl, while_stmt,
};
use nom::Err;

// ---------- Value ----------

#[test]
fn value_com_e_sem_await() {
    assert_eq!(value_expr("f()"), Ok(("", plain(call(id("f"), vec![])))));
    assert_eq!(
        value_expr("await f()"),
        Ok(("", awaited(call(id("f"), vec![]))))
    );
}

#[test]
fn value_await_e_palavra_inteira() {
    assert_eq!(value_expr("awaitx"), Ok(("", plain(id("awaitx")))));
}

// ---------- VarDecl ----------

#[test]
fn var_decl_let_e_const_com_e_sem_tipo() {
    assert_eq!(
        var_decl("let x: int = 1;"),
        Ok((
            "",
            Stmt::VarDecl {
                kind: DeclKind::Let,
                name: "x".into(),
                ty: Some(named("int", 0)),
                value: plain(int(1)),
            }
        ))
    );
    assert_eq!(
        var_decl("const valor = await buscarValor(pedido);"),
        Ok((
            "",
            Stmt::VarDecl {
                kind: DeclKind::Const,
                name: "valor".into(),
                ty: None,
                value: awaited(call(id("buscarValor"), vec![id("pedido")])),
            }
        ))
    );
}

#[test]
fn var_decl_recusa_palavra_reservada_como_nome() {
    assert!(matches!(var_decl("let if = 1;"), Err(Err::Failure(_))));
}

#[test]
fn var_decl_sem_ponto_e_virgula_e_erro() {
    assert!(matches!(var_decl("let x = 1"), Err(Err::Failure(_))));
}

#[test]
fn var_decl_constante_e_nome_nao_declaracao() {
    // `constante` não é `const` + `ante`.
    assert!(matches!(var_decl("constante = 1;"), Err(Err::Error(_))));
    assert_eq!(
        stmt("constante = 1;"),
        Ok((
            "",
            Stmt::Assign {
                name: "constante".into(),
                value: plain(int(1)),
            }
        ))
    );
}

// ---------- Assign x ExprStmt ----------

#[test]
fn assign_com_await() {
    assert_eq!(
        assign("x = await f();"),
        Ok((
            "",
            Stmt::Assign {
                name: "x".into(),
                value: awaited(call(id("f"), vec![])),
            }
        ))
    );
}

#[test]
fn assign_nao_casa_com_igualdade_estrita() {
    // Sem `=` sozinho, o `alt` volta atrás com Err::Error e tenta ExprStmt.
    assert!(matches!(assign("x === 1;"), Err(Err::Error(_))));
    assert_eq!(
        stmt("x === 1;"),
        Ok(("", Stmt::Expr(plain(bin(StrictEq, id("x"), int(1))))))
    );
    assert_eq!(
        stmt("x = 1;"),
        Ok((
            "",
            Stmt::Assign {
                name: "x".into(),
                value: plain(int(1)),
            }
        ))
    );
}

#[test]
fn expr_stmt_com_e_sem_await() {
    assert_eq!(
        expr_stmt("await sleep(3 * DIA);"),
        Ok((
            "",
            Stmt::Expr(awaited(call(
                id("sleep"),
                vec![bin(Mul, int(3), id("DIA"))]
            )))
        ))
    );
    assert_eq!(
        expr_stmt("console.log(x);"),
        Ok((
            "",
            Stmt::Expr(plain(call(member(id("console"), "log"), vec![id("x")])))
        ))
    );
}

#[test]
fn expr_stmt_await_so_no_comeco() {
    assert!(stmt("g(await f());").is_err());
}

#[test]
fn expr_stmt_diretiva_dentro_de_bloco_e_string() {
    assert_eq!(
        stmt(r#""use step";"#),
        Ok(("", Stmt::Expr(plain(str_lit("use step")))))
    );
}

// ---------- If ----------

fn call_stmt(f: &str) -> Stmt {
    Stmt::Expr(plain(call(id(f), vec![])))
}

#[test]
fn if_sem_else() {
    assert_eq!(
        if_stmt("if (ok) { f(); }"),
        Ok((
            "",
            If {
                cond: id("ok"),
                then_block: vec![call_stmt("f")],
                else_branch: None,
            }
        ))
    );
}

#[test]
fn if_com_else_e_else_if() {
    assert_eq!(
        if_stmt("if (a) { f(); } else if (b) { g(); } else { h(); }"),
        Ok((
            "",
            If {
                cond: id("a"),
                then_block: vec![call_stmt("f")],
                else_branch: Some(Else::If(Box::new(If {
                    cond: id("b"),
                    then_block: vec![call_stmt("g")],
                    else_branch: Some(Else::Block(vec![call_stmt("h")])),
                }))),
            }
        ))
    );
}

#[test]
fn if_sem_chaves_e_erro() {
    assert!(matches!(if_stmt("if (x) y();"), Err(Err::Failure(_))));
}

#[test]
fn else_sem_chaves_e_erro() {
    assert!(matches!(
        if_stmt("if (x) { y(); } else z();"),
        Err(Err::Failure(_))
    ));
}

// ---------- While ----------

#[test]
fn while_com_bloco() {
    assert_eq!(
        while_stmt("while (i < n) { i = i + 1; }"),
        Ok((
            "",
            Stmt::While {
                cond: bin(Lt, id("i"), id("n")),
                body: vec![Stmt::Assign {
                    name: "i".into(),
                    value: plain(bin(Add, id("i"), int(1))),
                }],
            }
        ))
    );
}

#[test]
fn whilex_e_nome() {
    assert!(matches!(while_stmt("whilex = 1;"), Err(Err::Error(_))));
    assert!(matches!(stmt("whilex = 1;"), Ok(("", Stmt::Assign { .. }))));
}

// ---------- Return ----------

#[test]
fn return_com_e_sem_valor() {
    assert_eq!(
        return_stmt("return \"cobrado\";"),
        Ok(("", Stmt::Return(Some(str_lit("cobrado")))))
    );
    assert_eq!(return_stmt("return;"), Ok(("", Stmt::Return(None))));
}

#[test]
fn return_value_e_nome() {
    assert!(matches!(
        stmt("returnValue = 1;"),
        Ok(("", Stmt::Assign { .. }))
    ));
}

#[test]
fn return_sem_ponto_e_virgula_e_erro() {
    assert!(matches!(return_stmt("return x"), Err(Err::Failure(_))));
}

// ---------- Block ----------

#[test]
fn block_vazio_e_com_comandos() {
    assert_eq!(block("{}"), Ok(("", vec![])));
    assert_eq!(
        block("{ f(); // comentário\n g(); }"),
        Ok(("", vec![call_stmt("f"), call_stmt("g")]))
    );
}

#[test]
fn block_aninhado_em_while_e_if() {
    let (rest, stmts) = block("{ while (a) { if (b) { return; } } }").unwrap();
    assert_eq!(rest, "");
    assert_eq!(stmts.len(), 1);
}

#[test]
fn assign_sem_ponto_e_virgula_acusa_o_fim_e_nao_o_igual() {
    // `cut` depois de `ident "="`: o `alt` não volta atrás para ExprStmt,
    // e o erro aponta onde o `;` devia estar.
    match stmt("x = 1 }") {
        Err(Err::Failure(e)) => assert_eq!(e.input, "}"),
        outro => panic!("esperava Failure no `}}`, veio {outro:?}"),
    }
}
