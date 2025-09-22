#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,
    If,
    Else,
    While,
    Ident(String),
    Number(NumberLit),
    True,
    False,
    Plus,
    Minus,
    Star,
    Slash,
    Mod,
    EqEq,
    Neq,
    Equal,
    Semicolon,
    Colon,
    LessThan,
    GreaterThan,
    Leq,
    Geq,
    LBrace,
    RBrace,
    Comma,
    LParen,
    RParen,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberLit {
    Int(i128),
    Float(f64),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Token::*;
        match self {
            Let => write!(f, "Let"),
            If => write!(f, "If"),
            Else => write!(f, "Else"),
            While => write!(f, "While"),
            Ident(s) => write!(f, "Ident({})", s),
            Number(n) => match n {
                NumberLit::Int(i) => write!(f, "Number(Int:{})", i),
                NumberLit::Float(x) => write!(f, "Number(Float:{})", x),
            },
            True => write!(f, "True"),
            False => write!(f, "False"),
            Plus => write!(f, "+"),
            Minus => write!(f, "-"),
            Star => write!(f, "*"),
            Slash => write!(f, "/"),
            Mod => write!(f, "%"),
            EqEq => write!(f, "=="),
            Neq => write!(f, "!="),
            Equal => write!(f, "="),
            Colon => write!(f, ":"),
            LessThan => write!(f, "<"),
            Leq => write!(f, "<="),
            GreaterThan => write!(f, ">"),
            Geq => write!(f, ">="),
            Semicolon => write!(f, ";"),
            LBrace => write!(f, "{{"),
            RBrace => write!(f, "}}"),
            Comma => write!(f, ","),
            LParen => write!(f, "("),
            RParen => write!(f, ")"),
            EOF => write!(f, "<EOF>"),
        }
    }
}
