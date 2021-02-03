use std::{char::ToLowercase, collections::HashMap, path::StripPrefixError};

use super::lexer::{lex, Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(f64),
    Variable(String),
    Binary(String, Box<Expression>, Box<Expression>),
    Call(String, Vec<Expression>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub prototype: Prototype,
    pub body: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Extern(Prototype),
    Function(Function),
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum ParserError {
    // TODO: add more context information
    #[error("invalid token {0}")]
    InvalidToken(String),
}

pub type PartialParseResult = Result<Expression, ParserError>;

#[derive(Debug, Clone)]
pub struct Parser {
    pub operator_precedence: HashMap<String, u32>,
}

impl std::default::Default for Parser {
    fn default() -> Self {
        let mut operator_precedence = HashMap::new();
        operator_precedence.insert("*".to_string(), 40);
        operator_precedence.insert("/".to_string(), 40);
        operator_precedence.insert("+".to_string(), 20);
        operator_precedence.insert("-".to_string(), 20);
        Self {
            operator_precedence,
        }
    }
}

impl Parser {
    fn parse_number(&self, input: &mut Vec<Token>) -> PartialParseResult {
        if let Some(Token::Number(num)) = input.pop() {
            Ok(Expression::Literal(num))
        } else {
            // TODO: clean up this logic a bit (macro?)
            unreachable!()
        }
    }

    fn parse_identifier(&self, input: &mut Vec<Token>) -> PartialParseResult {
        if let Some(Token::Ident(ident)) = input.pop() {
            if let Some(Token::OpenParen) = input.last() {
                unimplemented!()
            } else {
                Ok(Expression::Variable(ident.to_string()))
            }
        } else {
            unreachable!()
        }
    }

    fn parse_nested(&self, input: &mut Vec<Token>) -> PartialParseResult {
        if input.last() != Some(&Token::OpenParen) {
            return Err(ParserError::InvalidToken("(".to_string()));
        }
        input.pop();
        let res = self.parse_expr(input)?;
        if input.last() != Some(&Token::CloseParen) {
            return Err(ParserError::InvalidToken(")".to_string()));
        }
        input.pop();
        Ok(res)
    }

    fn parse_primary(&self, input: &mut Vec<Token>) -> PartialParseResult {
        match input.last().unwrap() {
            Token::Number(_) => self.parse_number(input),
            Token::Ident(_) => self.parse_identifier(input),
            Token::OpenParen => self.parse_nested(input),
            _ => unreachable!(),
        }
    }

    fn parse_rhs(
        &self,
        input: &mut Vec<Token>,
        expr_precedence: u32,
        lhs: &Expression,
    ) -> PartialParseResult {
        let mut result = lhs.clone();

        loop {
            let (operator, precedence) = match input.last() {
                Some(&Token::Operator(ref op)) => match self.operator_precedence.get(op) {
                    Some(pr) if *pr >= expr_precedence => (op.clone(), *pr),
                    None => panic!(),
                    _ => break,
                },
                _ => break,
            };
            input.pop();

            let mut rhs = self.parse_expr(input)?;

            match input.last() {
                Some(&Token::Operator(ref op)) => match self.operator_precedence.get(op) {
                    Some(next_precedence) if precedence < *next_precedence => {
                        rhs = self.parse_rhs(input, precedence + 1, &rhs)?
                    }
                    None => panic!(),
                    _ => (),
                },
                _ => (),
            };

            result = Expression::Binary(operator, Box::new(result), Box::new(rhs));
        }

        Ok(result)
    }

    fn parse_expr(&self, input: &mut Vec<Token>) -> PartialParseResult {
        let lhs = self.parse_primary(input)?;

        let expr = self.parse_rhs(input, 0, &lhs)?;
        Ok(expr)
    }

    pub fn parse(input: &mut Vec<Token>, settings: &Parser) -> Result<Vec<ASTNode>, ParserError> {
        let ast = Vec::new();

        while !input.is_empty() {
            let cur_tok = input.last().unwrap();

            let node = match cur_tok {
                Token::Def => unimplemented!(),
                Token::Extern => unimplemented!(),
                _ => unimplemented!(),
            };
        }

        Ok(ast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn parse_expr_works() {
        let input = "x + 1 * (2 - 3)";
        let parser = Parser::default();
        let mut tokens = lex(input);
        let res = parser.parse_expr(&mut tokens).unwrap();
        let target = Expression::Binary(
            "+".to_string(),
            Box::new(Expression::Variable("x".to_string())),
            Box::new(Expression::Binary(
                "*".to_string(),
                Box::new(Expression::Literal(1.0)),
                Box::new(Expression::Binary(
                    "-".to_string(),
                    Box::new(Expression::Literal(2.0)),
                    Box::new(Expression::Literal(3.0)),
                )),
            )),
        );
        assert_eq!(res, target);
    }
}
