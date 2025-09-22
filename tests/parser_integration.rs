#[test]
fn parse_let_with_type() {
    use hyperlight::lexer::tokenize;
    use hyperlight::parser::Parser;
    use hyperlight::ast::{Stmt, Type};

    let src = "let a: int32 = 2 + 3; a * 4";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert_eq!(stmts.len(), 2);
    match &stmts[0] {
        Stmt::Let { name, ty, value: _ } => {
            assert_eq!(name, "a");
            assert!(matches!(ty.as_ref().unwrap(), Type::Int));
        }
        _ => panic!("expected let statement"),
    }
}
