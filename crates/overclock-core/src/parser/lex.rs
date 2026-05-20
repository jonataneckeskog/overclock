use crate::parser::BoxedParser;
use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Void,          // "Void"
    Bool(bool),    // "true" or "false"
    Ident(String), // Identifyer
    Int(String),   // "42"
    Float(String), // "3.14"
    Char(char),    // "'a'"
    Indent,        // Virtual token: Layout shifted right
    Dedent,        // Virtual token: Layout shifted left
    Assign,        // "="
    Arrow,         // "->"
    MapArrow,      // "=>"
    Plus,          // "+"
    Minus,         // "-"
    Colon,         // ":"
    DoubleColon,   // "::"
    Comma,         // ","
    Dot,           // "."
    LParen,        // "("
    RParen,        // ")"
    LBracket,      // "["
    RBracket,      // "]"
    Newline,       // Explicit newline token
}

pub fn lexer<'a>() -> BoxedParser<'a, Vec<Token>, &'a str, extra::Err<Rich<'a, char>>> {
    let num = text::digits(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .map(|s: &str| {
            if s.contains('.') {
                Token::Float(s.to_string())
            } else {
                Token::Int(s.to_string())
            }
        });

    let ident = text::ident().map(|s: &str| match s {
        "Void" => Token::Void,
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        _ => Token::Ident(s.to_string()),
    });

    let char_ = just('\'')
        .ignore_then(none_of('\''))
        .then_ignore(just('\''))
        .map(Token::Char);

    let symbol = choice((
        just("::").to(Token::DoubleColon),
        just(":").to(Token::Colon),
        just("=>").to(Token::MapArrow),
        just("->").to(Token::Arrow),
        just("=").to(Token::Assign),
        just("+").to(Token::Plus),
        just("-").to(Token::Minus),
        just(",").to(Token::Comma),
        just(".").to(Token::Dot),
        just("(").to(Token::LParen),
        just(")").to(Token::RParen),
        just("[").to(Token::LBracket),
        just("]").to(Token::RBracket),
    ));

    let token = char_.or(num).or(ident).or(symbol);

    token.padded().repeated().collect().boxed()
}
