use crate::parser::{BoxedParser, lex::Token, expr::Expr};
use chumsky::prelude::*;

fn int<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Num(val) => val }.try_map(|val, span| {
        val.parse::<i64>()
            .map(Expr::Int)
            .map_err(|_| Rich::custom(span, "Not a valid 64-bit integer"))
    }).boxed()
}

fn var<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Ident(name) => Expr::Var(name) }.boxed()
}

fn list<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    expr
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map(Expr::List)
        .boxed()
}

fn group<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    expr
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .boxed()
}

pub fn atom<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    choice((int(), var(), list(expr.clone()), group(expr))).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lex::lexer;
    use chumsky::Parser;

    #[test]
    fn test_int() {
        let tokens = lexer().parse("123").into_result().unwrap();
        let ast = int().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Int(123));
    }

    #[test]
    fn test_var() {
        let tokens = lexer().parse("foo").into_result().unwrap();
        let ast = var().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(ast, Expr::Var("foo".to_string()));
    }
}
