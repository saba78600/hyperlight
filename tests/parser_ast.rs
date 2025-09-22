#[test]
fn parse_expression_ast_structure() {
    use hyperlight::{lexer::tokenize, parser::Parser, ast::{Expr, BinOp, Stmt}};

    // 1 + x * 2  => 1 + (x * 2)
    let toks = tokenize("1 + x * 2").expect("tokenize");
    let stmts = Parser::new(toks).parse().expect("parse");
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Expr(Expr::Binary { op: BinOp::Add, left, right }) => {
            // left should be the integer 1
            match &**left {
                Expr::Number(hyperlight::token::NumberLit::Int(1)) => {}
                other => panic!("left is not Int(1): {:?}", other),
            }

            // right should be a multiplication expression x * 2
            match &**right {
                Expr::Binary { op: BinOp::Mul, left: l, right: r } => {
                    match &**l {
                        Expr::Ident(s) if s == "x" => {},
                        other => panic!("expected ident 'x' on left of mul, got: {:?}", other),
                    }
                    match &**r {
                        Expr::Number(hyperlight::token::NumberLit::Int(2)) => {},
                        other => panic!("expected int 2 on right of mul, got: {:?}", other),
                    }
                }
                other => panic!("right is not multiplication: {:?}", other),
            }
        }
        other => panic!("expected top-level add expression, got: {:?}", other),
    }
}


#[test]
fn parse_parenthesis_precedence() {
    use hyperlight::{lexer::tokenize, parser::Parser, ast::{Expr, BinOp, Stmt}};

    // (1 + 2) * 3 => top-level multiplication with left being addition
    let toks = tokenize("(1 + 2) * 3").expect("tokenize");
    let stmts = Parser::new(toks).parse().expect("parse");
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Expr(Expr::Binary { op: BinOp::Mul, left, right }) => {
            // left is addition 1+2
            match &**left {
                Expr::Binary { op: BinOp::Add, left: l, right: r } => {
                    match &**l { Expr::Number(hyperlight::token::NumberLit::Int(1)) => {}, other => panic!("left add lhs wrong: {:?}", other) }
                    match &**r { Expr::Number(hyperlight::token::NumberLit::Int(2)) => {}, other => panic!("left add rhs wrong: {:?}", other) }
                }
                other => panic!("expected add on left of mul, got: {:?}", other),
            }
            match &**right { Expr::Number(hyperlight::token::NumberLit::Int(3)) => {}, other => panic!("expected int 3 on right of mul, got: {:?}", other) }
        }
        other => panic!("expected top-level mul expression, got: {:?}", other),
    }
}


#[test]
fn parse_let_with_and_without_type() {
    use hyperlight::{lexer::tokenize, parser::Parser, ast::{Type, Stmt}};

    // let with explicit type and without
    let src = "let a: int32 = 2 + 3; let b = a * 4;";
    let toks = tokenize(src).expect("tokenize");
    let stmts = Parser::new(toks).parse().expect("parse");
    assert_eq!(stmts.len(), 2);

    match &stmts[0] {
        Stmt::Let { name, ty, value: _ } => {
            assert_eq!(name, "a");
            assert!(matches!(ty.as_ref().unwrap(), Type::Int));
        }
        other => panic!("expected let for first stmt, got: {:?}", other),
    }

    match &stmts[1] {
        Stmt::Let { name, ty, value: _ } => {
            assert_eq!(name, "b");
            assert!(ty.is_none());
        }
        other => panic!("expected let for second stmt, got: {:?}", other),
    }
}
