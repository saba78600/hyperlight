#[test]
fn unary_minus() {
    use hyperlight::lexer::tokenize;
    use hyperlight::parser::Parser;
    use hyperlight::ast;
    use hyperlight::ast::Expr;

    let toks = tokenize("-5 + 3").unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    match &stmts[0] {
        ast::Stmt::Expr(Expr::Binary { left, right, .. }) => {
            assert!(matches!(&**left, Expr::Binary { .. }));
            assert!(matches!(&**right, Expr::Number(hyperlight::token::NumberLit::Int(3))));
        }
        _ => panic!("expected expression with unary minus"),
    }
}

#[test]
fn missing_semicolon_error() {
    use hyperlight::lexer::tokenize;
    use hyperlight::parser::Parser;

    let toks = tokenize("let a = 1 a * 2");
    // tokenization may succeed; parsing should return an error
    if let Ok(toks) = toks {
        let res = Parser::new(toks).parse();
        assert!(res.is_err());
    }
}
