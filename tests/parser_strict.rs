use hyperlight::{lexer::tokenize, parser::Parser, ast::{Stmt, Expr}};

#[test]
fn parse_if_else_and_while() {
    let src = "if (x < 10) { x = x + 1; } else { x = 0; } while (x < 5) { x = x + 1; }";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    // Expect three top-level statements: if, else handled in same stmt, while
    assert!(stmts.len() >= 2);
    // first should be If
    match &stmts[0] {
        Stmt::If { cond, then_block, else_block } => {
            // cond should be a comparison
            match cond {
                Expr::Binary { .. } => {}
                other => panic!("expected binary cond, got: {:?}", other),
            }
            assert!(then_block.len() == 1);
            assert!(else_block.as_ref().unwrap().len() == 1);
        }
        other => panic!("expected If stmt, got: {:?}", other),
    }
}

#[test]
fn parse_assignment_and_expression_stmt() {
    let src = "x = 5; x = x + 2; 3 + 4;";
    let toks = tokenize(src).unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert_eq!(stmts.len(), 3);
    match &stmts[0] { Stmt::Assign { name, .. } => assert_eq!(name, "x"), other => panic!("expected assign, got: {:?}", other) }
}
