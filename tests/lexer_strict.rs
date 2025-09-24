use hyperlight::token::Token;

#[test]
fn keywords_and_identifiers() {
    let src = "let letx iffy else_ while2 truefalse";
    let toks = hyperlight::tokenize(src).expect("tokenize");
    // let -> Let, letx -> Ident("letx")
    assert!(toks.iter().any(|(t, _)| matches!(t, Token::Let)));
    assert!(
        toks.iter()
            .any(|(t, _)| matches!(t, Token::Ident(s) if s == "letx"))
    );
    assert!(
        toks.iter()
            .any(|(t, _)| matches!(t, Token::Ident(s) if s == "iffy"))
    );
}

#[test]
fn numbers_and_floats() {
    let src = "0 123 3.1415 10.0";
    let toks = hyperlight::tokenize(src).expect("tokenize");
    let nums: Vec<_> = toks
        .into_iter()
        .filter_map(|(t, _)| match t {
            Token::Number(n) => Some(n),
            _ => None,
        })
        .collect();
    assert!(matches!(nums[0], hyperlight::token::NumberLit::Int(0)));
    assert!(matches!(nums[1], hyperlight::token::NumberLit::Int(123)));
    assert!(
        matches!(nums[2], hyperlight::token::NumberLit::Float(f) if (f - 3.1415).abs() < 1e-12)
    );
}

#[test]
fn invalid_char_error() {
    let src = "let x = 1 $ 2;";
    let res = hyperlight::tokenize(src);
    assert!(res.is_err());
}
