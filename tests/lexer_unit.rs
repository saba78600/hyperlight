use hyperlight::token::Token;

#[test]
fn basic_tokenize() {
    let src = "let x: int = 42; if (x > 0) { x = x - 1; }";
    let tokens = hyperlight::tokenize(src).expect("tokenize failed");
    // find a sequence of some token variants
    assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Let)));
    assert!(
        tokens
            .iter()
            .any(|(t, _)| matches!(t, Token::Ident(s) if s == "x"))
    );
    assert!(tokens.iter().any(|(t, _)| matches!(t, Token::Number(_))));
    assert!(tokens.iter().any(|(t, _)| matches!(t, Token::If)));
}
