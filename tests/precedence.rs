#[test]
fn operator_precedence() {
    use hyperlight::ast;
    use hyperlight::ast::{BinOp, Expr};
    use hyperlight::lexer::tokenize;
    use hyperlight::parser::Parser;

    // 2 + 3 * 4 => 2 + (3 * 4)
    let toks = tokenize("2 + 3 * 4").unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    match &stmts[0] {
        ast::Stmt::Expr(Expr::Binary {
            op: BinOp::Add,
            left,
            right,
        }) => {
            assert!(matches!(
                &**left,
                Expr::Number(hyperlight::token::NumberLit::Int(2))
            ));
            match &**right {
                Expr::Binary {
                    op: BinOp::Mul,
                    left: l,
                    right: r,
                } => {
                    assert!(matches!(
                        &**l,
                        Expr::Number(hyperlight::token::NumberLit::Int(3))
                    ));
                    assert!(matches!(
                        &**r,
                        Expr::Number(hyperlight::token::NumberLit::Int(4))
                    ));
                }
                _ => panic!("expected multiplication on right-hand side"),
            }
        }
        _ => panic!("expected binary add at top-level"),
    }
}
