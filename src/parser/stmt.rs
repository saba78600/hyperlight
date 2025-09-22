use crate::ast::Stmt;
use crate::token::Token;
use super::{ParseError, Parser};

impl Parser {
    pub fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Token::Let => return self.parse_let(),
            Token::If => return self.parse_if(),
            Token::While => return self.parse_while(),
            Token::LBrace => return Ok(Stmt::Expr(crate::ast::Expr::Ident("<block>".into()))), // placeholder if needed
            Token::Ident(_) => {
                // could be assignment or expression
                // peek ahead to see if '=' follows
                let pos = self.pos;
                if let Token::Ident(name) = self.bump() {
                    if matches!(self.peek(), Token::Equal) {
                        // assignment
                        self.bump(); // consume '='
                        let val = self.parse_expr()?;
                        if matches!(self.peek(), Token::Semicolon) { self.bump(); }
                        return Ok(Stmt::Assign { name, value: val });
                    } else {
                        // rewind and parse as expression
                        self.pos = pos;
                    }
                }
            }
            _ => {}
        }
        self.parse_expr_stmt()
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.bump(); // consume 'let'
        match self.bump() {
            Token::Ident(name) => {
                let ty = if matches!(self.peek(), Token::Colon) {
                    self.bump(); // ':'
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.consume_equal()?;
                let expr = self.parse_expr()?;
                self.consume_semicolon()?;
                Ok(Stmt::Let { name, ty, value: expr })
            }
            t => Err(ParseError::UnexpectedToken(t, self.pos)),
        }
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr()?;
        if matches!(self.peek(), Token::Semicolon) {
            self.bump();
        }
        Ok(Stmt::Expr(expr))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.bump(); // consume 'if'
        // expect '(' expr ')'
        match self.bump() { Token::LParen => {}, t => return Err(ParseError::UnexpectedToken(t, self.pos)), }
        let cond = self.parse_expr()?;
        match self.bump() { Token::RParen => {}, t => return Err(ParseError::UnexpectedToken(t, self.pos)), }
        let then_block = self.parse_block()?;
        let else_block = if matches!(self.peek(), Token::Else) {
            self.bump();
            Some(self.parse_block()?)
        } else { None };
        Ok(Stmt::If { cond, then_block, else_block })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.bump(); // consume 'while'
        match self.bump() { Token::LParen => {}, t => return Err(ParseError::UnexpectedToken(t, self.pos)), }
        let cond = self.parse_expr()?;
        match self.bump() { Token::RParen => {}, t => return Err(ParseError::UnexpectedToken(t, self.pos)), }
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        match self.bump() { Token::LBrace => {}, t => return Err(ParseError::UnexpectedToken(t, self.pos)), }
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::RBrace | Token::EOF) {
            stmts.push(self.parse_stmt()?);
        }
        match self.bump() { Token::RBrace => Ok(stmts), t => Err(ParseError::UnexpectedToken(t, self.pos)), }
    }

    fn consume_equal(&mut self) -> Result<(), ParseError> {
        match self.bump() {
            Token::Equal => Ok(()),
            t => Err(ParseError::UnexpectedToken(t, self.pos)),
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), ParseError> {
        match self.bump() {
            Token::Semicolon => Ok(()),
            t => Err(ParseError::UnexpectedToken(t, self.pos)),
        }
    }
}
