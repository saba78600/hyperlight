use crate::token::Token;
use crate::span::Span;

#[derive(Debug)]
pub enum LexError {
    InvalidChar(char, usize),
}

pub type SpannedToken = (Token, Span);

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().enumerate().peekable();
    let mut line = 1usize;
    let mut col = 1usize;

    while let Some((i, ch)) = chars.peek().cloned() {
        match ch {
            '\n' => {
                // advance
                chars.next();
                line += 1;
                col = 1;
            }
            c if c.is_whitespace() => {
                chars.next();
                col += 1;
            }
            '0'..='9' => {
                let start = i;
                let start_col = col;
                let mut num = String::new();
                while let Some((_, c)) = chars.peek().cloned() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(start + num.len());
                if num.contains('.') {
                    let n: f64 = num.parse().unwrap_or(0.0);
                    tokens.push((Token::Number(crate::token::NumberLit::Float(n)), Span::new(start, end, line, start_col)));
                } else {
                    let n: i128 = num.parse().unwrap_or(0);
                    tokens.push((Token::Number(crate::token::NumberLit::Int(n)), Span::new(start, end, line, start_col)));
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                let start_col = col;
                let mut ident = String::new();
                while let Some((_, c)) = chars.peek().cloned() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                        col += 1;
                    } else {
                        break;
                    }
                }
                let end = chars.peek().map(|(i, _)| *i).unwrap_or(start + ident.len());
                match ident.as_str() {
                    "let" => tokens.push((Token::Let, Span::new(start, end, line, start_col))),
                    "if" => tokens.push((Token::If, Span::new(start, end, line, start_col))),
                    "else" => tokens.push((Token::Else, Span::new(start, end, line, start_col))),
                    "while" => tokens.push((Token::While, Span::new(start, end, line, start_col))),
                    "true" => tokens.push((Token::True, Span::new(start, end, line, start_col))),
                    "false" => tokens.push((Token::False, Span::new(start, end, line, start_col))),
                    _ => tokens.push((Token::Ident(ident), Span::new(start, end, line, start_col))),
                }
            }
            '+' => { tokens.push((Token::Plus, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '-' => { tokens.push((Token::Minus, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '*' => { tokens.push((Token::Star, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '/' => { tokens.push((Token::Slash, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '%' => { tokens.push((Token::Mod, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '=' => {
                // could be '=='
                chars.next(); col += 1;
                if let Some((_, '=')) = chars.peek().cloned() {
                    chars.next(); col += 1;
                    tokens.push((Token::EqEq, Span::new(i, i+2, line, col-2)));
                } else {
                    tokens.push((Token::Equal, Span::new(i, i+1, line, col-1)));
                }
            }
            '!' => {
                // expect '!='
                chars.next(); col += 1;
                if let Some((_, '=')) = chars.peek().cloned() {
                    chars.next(); col += 1;
                    tokens.push((Token::Neq, Span::new(i, i+2, line, col-2)));
                } else {
                    return Err(LexError::InvalidChar('!', i));
                }
            }
            ':' => { tokens.push((Token::Colon, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '<' => {
                chars.next(); col += 1;
                if let Some((_, '=')) = chars.peek().cloned() {
                    chars.next(); col += 1;
                    tokens.push((Token::Leq, Span::new(i, i+2, line, col-2)));
                } else {
                    tokens.push((Token::LessThan, Span::new(i, i+1, line, col-1)));
                }
            }
            '>' => {
                chars.next(); col += 1;
                if let Some((_, '=')) = chars.peek().cloned() {
                    chars.next(); col += 1;
                    tokens.push((Token::Geq, Span::new(i, i+2, line, col-2)));
                } else {
                    tokens.push((Token::GreaterThan, Span::new(i, i+1, line, col-1)));
                }
            }
            ';' => { tokens.push((Token::Semicolon, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '{' => { tokens.push((Token::LBrace, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '}' => { tokens.push((Token::RBrace, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            ',' => { tokens.push((Token::Comma, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            '(' => { tokens.push((Token::LParen, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            ')' => { tokens.push((Token::RParen, Span::new(i, i+1, line, col))); chars.next(); col += 1; }
            other => return Err(LexError::InvalidChar(other, i)),
        }
    }

    tokens.push((Token::EOF, Span::new(input.len(), input.len(), line, col)));
    Ok(tokens)
}
