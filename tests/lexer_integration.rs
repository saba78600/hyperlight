#[test]
fn tokenize_basic_tokens() {
    use hyperlight::lexer::tokenize;
    use hyperlight::token::{Token, NumberLit};

    let toks = tokenize("let x = 3 + y;").expect("tokenize");
    assert_eq!(toks[0].0, Token::Let);
    assert_eq!(toks[1].0, Token::Ident("x".into()));
    assert_eq!(toks[2].0, Token::Equal);
    assert_eq!(toks[3].0, Token::Number(NumberLit::Int(3)));
}
