use crate::ast::Expr;
use super::{BoxedParser, lex::Token};
use chumsky::prelude::*;

mod atom;
mod binary;
mod lambda;
mod pipe;
mod postfix;

#[cfg(test)]
mod tests;

/// Parses an expression, supporting assignments, lambdas, pipes, and other constructs.
pub fn expr<'a>() -> BoxedParser<'a, Expr> {
    recursive(|expr| {
        let boxed_expr = expr.boxed();
        let assignment = select! { Token::Ident(name) => name }
            .then_ignore(just(Token::Assign))
            .then(boxed_expr.clone())
            .map(|(name, e)| Expr::Assign(name, Box::new(e)));

        assignment.or(lambda::lambda(boxed_expr)).boxed()
    })
    .boxed()
}
