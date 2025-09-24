use super::{ParseError, Parser};
use crate::ast::{BinOp, Expr};
use crate::token::Token;

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_cmp()
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Mod => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Mod,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::EqEq => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Eq,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::Neq => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Ne,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::LessThan => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Lt,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::GreaterThan => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Gt,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::Leq => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Le,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::Geq => {
                    self.bump();
                    let rhs = self.parse_term()?;
                    node = Expr::Binary {
                        op: BinOp::Ge,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.parse_factor()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.bump();
                    let rhs = self.parse_factor()?;
                    node = Expr::Binary {
                        op: BinOp::Add,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::Minus => {
                    self.bump();
                    let rhs = self.parse_factor()?;
                    node = Expr::Binary {
                        op: BinOp::Sub,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut node = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.bump();
                    let rhs = self.parse_unary()?;
                    node = Expr::Binary {
                        op: BinOp::Mul,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                Token::Slash => {
                    self.bump();
                    let rhs = self.parse_unary()?;
                    node = Expr::Binary {
                        op: BinOp::Div,
                        left: Box::new(node),
                        right: Box::new(rhs),
                    };
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Token::Minus) {
            self.bump();
            let expr = self.parse_primary()?;
            return Ok(Expr::Binary {
                op: BinOp::Sub,
                left: Box::new(Expr::Number(crate::token::NumberLit::Int(0))),
                right: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.bump() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Ident(s) => {
                // function call syntax: ident '(' args ')'
                if matches!(self.peek(), Token::LParen) {
                    self.bump(); // consume '('
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        loop {
                            let e = self.parse_expr()?;
                            args.push(e);
                            if matches!(self.peek(), Token::Comma) {
                                self.bump();
                                continue;
                            }
                            break;
                        }
                    }
                    match self.bump() {
                        Token::RParen => Ok(Expr::Call { callee: s, args }),
                        t => Err(ParseError::UnexpectedToken(t, self.pos)),
                    }
                } else {
                    Ok(Expr::Ident(s))
                }
            }
            Token::LParen => {
                let e = self.parse_expr()?;
                match self.bump() {
                    Token::RParen => Ok(e),
                    t => Err(ParseError::UnexpectedToken(t, self.pos)),
                }
            }
            t => Err(ParseError::UnexpectedToken(t, self.pos)),
        }
    }
}
