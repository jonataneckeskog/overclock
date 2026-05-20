use crate::ast::{Expr, Literal};
use crate::parser::{BoxedParser, lex::Token};
use chumsky::prelude::*;

/// Parses a boolean literal.
fn bool<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Bool(val) => Expr::Lit(Literal::Bool(val)) }.boxed()
}

/// Parses an integer literal.
fn int<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Int(val) => val }.try_map(|val, span| {
        val.parse::<i64>()
            .map(|n| Expr::Lit(Literal::Int(n)))
            .map_err(|_| Rich::custom(span, "Not a valid 64-bit integer"))
    }).boxed()
}

/// Parses a float literal.
fn float<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Float(val) => val }.try_map(|val, span| {
        val.parse::<f64>()
            .map(|n| Expr::Lit(Literal::Float(n)))
            .map_err(|_| Rich::custom(span, "Not a valid 64-bit float"))
    }).boxed()
}

/// Parses a character literal.
fn char<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Char(val) => Expr::Lit(Literal::Char(val)) }.boxed()
}

/// Parses a variable identifier.
fn var<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Ident(name) => Expr::Var(name) }.boxed()
}

/// Parses a list literal, e.g., [1, 2, 3].
fn list<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    expr
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map(Expr::List)
        .boxed()
}

/// Parses a parenthesized expression.
fn group<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    expr
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .boxed()
}

/// Parses an atomic expression (literals, variables, lists, or groups).
pub fn atom<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    choice((
        bool(),
        float(),
        int(),
        char(),
        var(),
        list(expr.clone()),
        group(expr),
    ))
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer;
    use chumsky::Parser;

    #[test]
    fn test_bool() {
        let tokens = lexer().parse("true").into_result().unwrap();
        let ast = bool().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Lit(Literal::Bool(true)));
    }

    #[test]
    fn test_int() {
        let tokens = lexer().parse("123").into_result().unwrap();
        let ast = int().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Lit(Literal::Int(123)));
    }

    #[test]
    fn test_float() {
        let tokens = lexer().parse("3.14").into_result().unwrap();
        let ast = float().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Lit(Literal::Float(3.14)));
    }

    #[test]
    fn test_char() {
        let tokens = lexer().parse("'a'").into_result().unwrap();
        let ast = char().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Lit(Literal::Char('a')));
    }

    #[test]
    fn test_var() {
        let tokens = lexer().parse("foo").into_result().unwrap();
        let ast = var().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Var("foo".to_string()));
    }
}
