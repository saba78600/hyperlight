use hyperlight::{lexer::tokenize, parser::Parser, typecheck::check};

#[test]
fn unknown_identifier_error() {
    let src = "x = 1;";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert!(check(&stmts).is_err());
}

#[test]
fn int_to_float_promote_ok() {
    let src = "let a: float64 = 1;";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert!(check(&stmts).is_ok());
}

#[test]
fn modulo_type_error() {
    let src = "let a = 1.0; let b = 2; let c = a % b;"; // modulo with float should error
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert!(check(&stmts).is_err());
}
