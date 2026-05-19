use super::{BoxedParser, lex::Token};
use chumsky::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Int(i64),
    Var(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Assign(String, Box<Expr>),
}

fn int<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Num(val) => val }
        .try_map(|val, span| {
            val.parse::<i64>()
                .map(Expr::Int)
                .map_err(|_| Rich::custom(span, "Not a valid 64-bit integer"))
        })
        .boxed()
}

fn var<'a>() -> BoxedParser<'a, Expr> {
    select! { Token::Ident(name) => Expr::Var(name) }.boxed()
}

pub fn expr<'a>() -> BoxedParser<'a, Expr> {
    recursive(|expr| {
        let assignment = select! { Token::Ident(name) => name }
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .map(|(name, e)| Expr::Assign(name, Box::new(e)));

        assignment.or(int()).or(var())
    })
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        let input = [Token::Num("123".to_string())];

        assert_eq!(expr().parse(&input[..]).into_result(), Ok(Expr::Int(123)));
    }

    #[test]
    fn test_var() {
        let input = [Token::Ident("foo".to_string())];

        assert_eq!(
            expr().parse(&input[..]).into_result(),
            Ok(Expr::Var("foo".to_string()))
        );
    }

    #[test]
    fn test_assign() {
        let input = [
            Token::Ident("x".to_string()),
            Token::Assign,
            Token::Num("5".to_string()),
        ];
        assert_eq!(
            expr().parse(&input[..]).into_result(),
            Ok(Expr::Assign("x".to_string(), Box::new(Expr::Int(5))))
        );
    }

    #[test]
    fn test_expr() {
        let input_int = [Token::Num("42".to_string())];
        let input_var = [Token::Ident("my_var".to_string())];

        assert_eq!(
            expr().parse(&input_int[..]).into_result(),
            Ok(Expr::Int(42))
        );
        assert_eq!(
            expr().parse(&input_var[..]).into_result(),
            Ok(Expr::Var("my_var".to_string()))
        );
    }
}
