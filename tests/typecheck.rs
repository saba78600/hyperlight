#[test]
fn typecheck_success_and_infer() {
    use hyperlight::{lexer::tokenize, parser::Parser, typecheck::check};

    let src = "let a: float64 = 1.0; let b = a + 2.0;";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert!(check(&stmts).is_ok());
}

#[test]
fn typecheck_mismatch() {
    use hyperlight::{lexer::tokenize, parser::Parser, typecheck::check};

    let src = "let a: int32 = 1.0;";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert!(check(&stmts).is_err());
}
