use chumsky::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Void,          // "Void"
    Ident(String), // Identifyer
    Num(String),   // Numerical value
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

pub fn lexer<'a>() -> impl Parser<'a, &'a str, Vec<Token>, extra::Err<Rich<'a, char>>> {
    let num = text::digits(10)
        .to_slice()
        .map(|s: &str| Token::Num(s.to_string()));

    let ident = text::ident().map(|s: &str| match s {
        "Void" => Token::Void,
        _ => Token::Ident(s.to_string()),
    });

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

    let token = num.or(ident).or(symbol);

    token.padded().repeated().collect()
}
