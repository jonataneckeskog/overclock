use super::{BoxedParser, lex::Token};
use chumsky::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add, // "+"
    Sub, // "-"
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Int(i64),                               // "42"
    Var(String),                            // "x"
    Binary(Box<Expr>, BinaryOp, Box<Expr>), // "a + b"
    Assign(String, Box<Expr>),              // "x = 5"
    Pipe(Box<Expr>, Box<Expr>),             // "a -> b"
    Lambda(String, Box<Expr>),              // "x => x"
    List(Vec<Expr>),                        // "[1, 2]"
    Call(Box<Expr>, Vec<Expr>),             // "f(x)"
    Member(Box<Expr>, String),              // "obj.prop"
}

pub mod atom;
pub mod binary;
pub mod lambda;
pub mod pipe;
pub mod postfix;

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
