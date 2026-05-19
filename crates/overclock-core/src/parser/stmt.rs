use super::{
    BoxedParser,
    expr::{Expr, expr},
    skip,
};
use chumsky::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assign(String, Expr),
}

pub fn assign<'a>() -> BoxedParser<'a, Statement> {
    text::ident()
        .map(String::from)
        .padded_by(skip())
        .then_ignore(just('='))
        .padded_by(skip())
        .then(expr())
        .then_ignore(skip())
        .map(|(name, expr)| Statement::Assign(name, expr))
        .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assign() {
        let input = "x = 5 // some comment";
        assert_eq!(
            assign().parse(input).into_result(),
            Ok(Statement::Assign("x".to_string(), Expr::Int(5)))
        );

        let input2 = "   count   =   counter  ";
        assert_eq!(
            assign().parse(input2).into_result(),
            Ok(Statement::Assign(
                "count".to_string(),
                Expr::Var("counter".to_string())
            ))
        );
    }
}
