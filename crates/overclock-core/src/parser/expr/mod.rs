use super::{BoxedParser, lex::Token};
use chumsky::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Int(i64),
    Var(String),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Assign(String, Box<Expr>),
    Pipe(Box<Expr>, Box<Expr>),
    Lambda(String, Box<Expr>),
    List(Vec<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, String),
}

pub mod atom;
pub mod binary;
pub mod lambda;
pub mod pipe;
pub mod postfix;

#[cfg(test)]
mod tests;

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
