use super::BoxedParser;
use chumsky::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Int(i32),
    Var(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

fn int<'a>() -> BoxedParser<'a, Expr> {
    text::int(10)
        .from_str::<i32>()
        .unwrapped()
        .map(Expr::Int)
        .boxed()
}

fn var<'a>() -> BoxedParser<'a, Expr> {
    text::ident()
        .map(|s: &str| Expr::Var(s.to_string()))
        .boxed()
}

pub fn expr<'a>() -> BoxedParser<'a, Expr> {
    int().or(var()).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int() {
        assert_eq!(int().parse("123").into_result(), Ok(Expr::Int(123)));
        assert_eq!(int().parse("0").into_result(), Ok(Expr::Int(0)));
    }

    #[test]
    fn test_var() {
        assert_eq!(
            var().parse("foo").into_result(),
            Ok(Expr::Var("foo".to_string()))
        );
        assert_eq!(
            var().parse("x_y1").into_result(),
            Ok(Expr::Var("x_y1".to_string()))
        );
    }

    #[test]
    fn test_expr() {
        assert_eq!(expr().parse("42").into_result(), Ok(Expr::Int(42)));
        assert_eq!(
            expr().parse("my_var").into_result(),
            Ok(Expr::Var("my_var".to_string()))
        );
    }
}
