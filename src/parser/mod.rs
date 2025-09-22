pub mod expr;
pub mod stmt;
pub mod r#type;
use crate::ast::Stmt;
use crate::token::Token;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token, usize),
    UnexpectedEOF,
}

pub type SpannedToken = (crate::token::Token, crate::span::Span);

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].0
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens[self.pos].0.clone();
        self.pos += 1;
        t
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.parse_statements()
    }

    fn parse_statements(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }
}
