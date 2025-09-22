use super::{ParseError, Parser};
use crate::token::Token;

impl Parser {
    pub(crate) fn parse_type(&mut self) -> Result<crate::ast::Type, ParseError> {
        match self.bump() {
            Token::Ident(s) => {
                // look for optional <number>
                if matches!(self.peek(), Token::LessThan) {
                    self.bump(); // consume '<'
                    match self.bump() {
                        Token::Number(_n) => {
                            match self.bump() {
                                Token::GreaterThan => {}
                                t => return Err(ParseError::UnexpectedToken(t, self.pos)),
                            }
                            let ident = s.to_lowercase();
                            if ident.starts_with("int") || ident.starts_with("i") {
                                return Ok(crate::ast::Type::Int);
                            }
                            if ident.starts_with("uint") || ident.starts_with("u") {
                                return Ok(crate::ast::Type::UInt);
                            }
                            if ident.starts_with("float") || ident.starts_with("f") {
                                return Ok(crate::ast::Type::Float);
                            }
                            Ok(crate::ast::Type::Custom(s))
                        }
                        t => return Err(ParseError::UnexpectedToken(t, self.pos)),
                    }
                } else {
                    // bare identifier type e.g. int32
                    let low = s.to_lowercase();
                    if low.starts_with("int") {
                        return Ok(crate::ast::Type::Int);
                    }
                    if low.starts_with("uint") {
                        return Ok(crate::ast::Type::UInt);
                    }
                    if low.starts_with("float") {
                        return Ok(crate::ast::Type::Float);
                    }
                    Ok(crate::ast::Type::Custom(s))
                }
            }
            t => Err(ParseError::UnexpectedToken(t, self.pos)),
        }
    }
}
