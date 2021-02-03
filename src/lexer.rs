use core::f64;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Def,
    Extern,
    Delimiter,
    OpenParen,
    CloseParen,
    Comma,
    Ident(String),
    Operator(String),
    Number(f64),
}

lazy_static! {
    static ref IGNORE_RE: Regex = Regex::new(r"(?m)#.*$").unwrap();
    static ref TOKEN_RE: Regex = Regex::new(concat!(
        r"(?P<ident>\p{Alphabetic}\w*)|",
        r"(?P<number>\d+\.?\d*)|",
        r"(?P<delimiter>;)|",
        r"(?P<oppar>\()|",
        r"(?P<clpar>\))|",
        r"(?P<comma>,)|",
        r"(?P<operator>\S)"
    ))
    .unwrap();
}

fn preprocess(input: &str) -> String {
    IGNORE_RE.replace_all(input, "").to_string()
}

/// lex the given input string - returns a stack, so first-on last-off
pub fn lex(input: &str) -> Vec<Token> {
    let preprocessed = preprocess(input);

    let mut res = Vec::new();
    for cap in TOKEN_RE.captures_iter(&preprocessed) {
        let token = if let Some(ident) = cap.name("ident") {
            match ident.as_str() {
                "def" => Token::Def,
                "extern" => Token::Extern,
                inner => Token::Ident(inner.to_string()),
            }
        } else if let Some(inner) = cap.name("number") {
            Token::Number(inner.as_str().parse().expect("failed to parse number!"))
        } else if let Some(op) = cap.name("operator") {
            Token::Operator(op.as_str().to_string())
        } else if let Some(_) = cap.name("comma") {
            Token::Comma
        } else if let Some(_) = cap.name("oppar") {
            Token::OpenParen
        } else if let Some(_) = cap.name("clpar") {
            Token::CloseParen
        } else if let Some(_) = cap.name("delimiter") {
            Token::Delimiter
        } else {
            panic!("unknown token!");
        };

        res.push(token);
    }
    res.reverse();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore_works() {
        assert_eq!(preprocess("# somebody \na"), "\na");
    }

    #[test]
    fn lex_works() {
        let input = "def func(x) x + 1;";
        let tokenized = [
            Token::Delimiter,
            Token::Number(1.0),
            Token::Operator("+".to_string()),
            Token::Ident("x".to_string()),
            Token::CloseParen,
            Token::Ident("x".to_string()),
            Token::OpenParen,
            Token::Ident("func".to_string()),
            Token::Def,
        ];
        assert_eq!(lex(input), tokenized);
    }
}
