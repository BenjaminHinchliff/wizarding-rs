use std::collections::HashMap;

use super::ast::*;
use super::lexer::{self, Token};

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum ParserError {
    // TODO: add more context information
    #[error("invalid token {0}")]
    InvalidToken(Token),
    #[error("invalid operator {0}")]
    InvalidOperator(String),
    #[error("unexpected end of file")]
    UnexpectedEOF,
}

pub type PartialParseResult = Result<Expression, ParserError>;

macro_rules! ensure_next {
    ($input:ident, $($next:expr),+) => {
        match $input.last() {
            Some(tok) if $(*tok != $next)||+ => return Err(ParserError::InvalidToken(tok.clone())),
            None => return Err(ParserError::UnexpectedEOF),
            _ => (),
        }
        $input.pop();
    };
}

macro_rules! extract_token {
    ($input:expr) => {
        match $input {
            Some(tok) => tok,
            None => return Err(ParserError::UnexpectedEOF),
        }
    };
    ($input:expr, $next:pat, $inner:expr) => {
        match $input {
            Some(tok) => match tok {
                $next => $inner,
                tok => return Err(ParserError::InvalidToken(tok.clone())),
            },
            None => return Err(ParserError::UnexpectedEOF),
        }
    };
}

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
        let num = extract_token!(input.pop(), Token::Number(extract), extract);
        Ok(Expression::Literal(num))
    }

    fn parse_identifier(&self, input: &mut Vec<Token>) -> PartialParseResult {
        let ident = extract_token!(input.pop(), Token::Ident(extract), extract);
        if let Some(Token::OpenParen) = input.last() {
            let mut args = Vec::new();
            ensure_next!(input, Token::OpenParen);
            // TODO: try to prevent code duplication with argument parsing
            if input.last() != Some(&Token::CloseParen) {
                loop {
                    args.push(self.parse_expr(input)?);
                    if input.last() != Some(&Token::Comma) {
                        if input.last() == Some(&Token::CloseParen) {
                            break;
                        } else if let Some(tok) = input.last() {
                            return Err(ParserError::InvalidToken(tok.clone()));
                        } else {
                            return Err(ParserError::UnexpectedEOF);
                        }
                    }
                    input.pop();
                }
            }
            ensure_next!(input, Token::CloseParen);
            Ok(Expression::Call(ident.to_string(), args))
        } else {
            Ok(Expression::Variable(ident.to_string()))
        }
    }

    fn parse_nested(&self, input: &mut Vec<Token>) -> PartialParseResult {
        ensure_next!(input, Token::OpenParen);
        let res = self.parse_expr(input)?;
        ensure_next!(input, Token::CloseParen);
        Ok(res)
    }

    fn parse_primary(&self, input: &mut Vec<Token>) -> PartialParseResult {
        match extract_token!(input.last()) {
            Token::Number(_) => self.parse_number(input),
            Token::Ident(_) => self.parse_identifier(input),
            Token::OpenParen => self.parse_nested(input),
            tok => return Err(ParserError::InvalidToken(tok.clone())),
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
                    None => return Err(ParserError::InvalidOperator(op.to_string())),
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
                    None => return Err(ParserError::InvalidOperator(op.to_string())),
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

    fn parse_prototype(&self, input: &mut Vec<Token>) -> Result<Prototype, ParserError> {
        let name = extract_token!(input.pop(), Token::Ident(ident), ident);
        ensure_next!(input, Token::OpenParen);
        let mut args = Vec::new();
        if input.last() != Some(&Token::CloseParen) {
            while let Some(Token::Ident(ident)) = input.pop() {
                args.push(ident);
                if input.last() != Some(&Token::Comma) {
                    if input.last() == Some(&Token::CloseParen) {
                        break;
                    } else if let Some(tok) = input.last() {
                        return Err(ParserError::InvalidToken(tok.clone()));
                    } else {
                        return Err(ParserError::UnexpectedEOF);
                    }
                }
                input.pop();
            }
        }
        ensure_next!(input, Token::CloseParen);
        Ok(Prototype { name, args })
    }

    fn parse_function(&self, input: &mut Vec<Token>) -> Result<ASTNode, ParserError> {
        input.pop();
        let proto = self.parse_prototype(input)?;
        let body = self.parse_expr(input)?;
        Ok(ASTNode::Function(Function {
            prototype: proto,
            body,
        }))
    }

    fn parse_extern(&self, input: &mut Vec<Token>) -> Result<ASTNode, ParserError> {
        input.pop();
        Ok(ASTNode::Extern(self.parse_prototype(input)?))
    }

    fn parse_lambda(&self, input: &mut Vec<Token>) -> Result<ASTNode, ParserError> {
        Ok(ASTNode::Function(Function {
            prototype: Prototype {
                name: "".to_string(),
                args: vec![],
            },
            body: self.parse_expr(input)?,
        }))
    }

    pub fn parse(&self, input: &mut Vec<Token>) -> Result<Vec<ASTNode>, ParserError> {
        let mut ast = Vec::new();

        while !input.is_empty() {
            let cur_tok = input.last().unwrap();

            match cur_tok {
                Token::Def => ast.push(self.parse_function(input)?),
                Token::Extern => ast.push(self.parse_extern(input)?),
                Token::Delimiter => {
                    input.pop();
                }
                _ => ast.push(self.parse_lambda(input)?),
            };
        }

        Ok(ast)
    }

    pub fn parse_str(&self, input: &str) -> Result<Vec<ASTNode>, ParserError> {
        let mut tokens = lexer::lex(input);
        self.parse(&mut tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn lamda_parse_works() {
        let parser = Parser::default();
        let mut tokens = lexer::lex("1;");
        let res = parser.parse(&mut tokens).unwrap();
        let target = vec![ASTNode::Function(Function {
            prototype: Prototype {
                name: "".to_string(),
                args: vec![],
            },
            body: Expression::Literal(1.0),
        })];
        assert_eq!(res, target);
    }

    #[test]
    fn extern_parse_works() {
        let parser = Parser::default();
        let mut tokens = lexer::lex("extern sin(x);");
        let res = parser.parse(&mut tokens).unwrap();
        let target = vec![ASTNode::Extern(Prototype {
            name: "sin".to_string(),
            args: vec!["x".to_string()],
        })];
        assert_eq!(res, target);
    }

    #[test]
    fn def_parse_works() {
        let parser = Parser::default();
        let mut tokens = lexer::lex("def add(x, y) x + y;");
        let res = parser.parse(&mut tokens).unwrap();
        let target = vec![ASTNode::Function(Function {
            prototype: Prototype {
                name: "add".to_string(),
                args: vec!["x".to_string(), "y".to_string()],
            },
            body: Expression::Binary(
                "+".to_string(),
                Box::new(Expression::Variable("x".to_string())),
                Box::new(Expression::Variable("y".to_string())),
            ),
        })];
        assert_eq!(res, target);
        let mut tokens = lexer::lex("def one() 1.0;");
        let res = parser.parse(&mut tokens).unwrap();
        let target = vec![ASTNode::Function(Function {
            prototype: Prototype {
                name: "one".to_string(),
                args: vec![],
            },
            body: Expression::Literal(1.0),
        })];
        assert_eq!(res, target);
    }

    #[test]
    fn parse_call_works() {
        let parser = Parser::default();
        let input = "add(1, 2)";
        let mut tokens = lexer::lex(input);
        let res = parser.parse_expr(&mut tokens).unwrap();
        let target = Expression::Call(
            "add".to_string(),
            vec![Expression::Literal(1.0), Expression::Literal(2.0)],
        );
        assert_eq!(res, target);
        let mut tokens = lexer::lex("one()");
        let res = parser.parse_expr(&mut tokens).unwrap();
        let target = Expression::Call("one".to_string(), vec![]);
        assert_eq!(res, target);
    }

    #[test]
    fn parse_expr_works() {
        let input = "x + 1 * (2 - 3)";
        let parser = Parser::default();
        let mut tokens = lexer::lex(input);
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

    #[test]
    fn invalid_operator_works() {
        let input = "x : 1";
        let parser = Parser::default();
        let mut tokens = lexer::lex(input);
        let res = parser.parse_expr(&mut tokens);
        assert_eq!(res, Err(ParserError::InvalidOperator(":".to_string())));
    }

    #[test]
    fn invalid_token_works() {
        let input = "(1 + )";
        let parser = Parser::default();
        let mut tokens = lexer::lex(input);
        let res = parser.parse_expr(&mut tokens);
        assert_eq!(res, Err(ParserError::InvalidToken(Token::CloseParen)));
    }

    #[test]
    fn unexpected_eof_works() {
        let parser = Parser::default();
        let mut tokens = lexer::lex("1 + ");
        let res = parser.parse_expr(&mut tokens);
        assert_eq!(res, Err(ParserError::UnexpectedEOF));
    }
}
