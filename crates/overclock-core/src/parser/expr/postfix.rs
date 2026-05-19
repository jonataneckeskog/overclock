use crate::parser::{BoxedParser, lex::Token, expr::{Expr, atom::atom}};
use chumsky::prelude::*;

pub enum PostfixOp {
    Call(Vec<Expr>),
    Member(String),
}

fn call_op<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, PostfixOp> {
    expr
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .map(PostfixOp::Call)
        .boxed()
}

fn dot_member_op<'a>() -> BoxedParser<'a, PostfixOp> {
    just(Token::Dot)
        .ignore_then(
            select! { Token::Ident(name) => name }.or(just(Token::Minus)
                .then(select! { Token::Num(n) => n })
                .map(|(_, n)| format!("-{}", n))),
        )
        .map(PostfixOp::Member)
        .boxed()
}

fn colon_member_op<'a>() -> BoxedParser<'a, PostfixOp> {
    just(Token::DoubleColon)
        .ignore_then(select! { Token::Ident(name) => name })
        .map(PostfixOp::Member)
        .boxed()
}

pub fn postfix<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    let op = choice((call_op(expr.clone()), dot_member_op(), colon_member_op()));

    atom(expr).foldl(op.repeated(), |lhs, op| match op {
        PostfixOp::Call(args) => Expr::Call(Box::new(lhs), args),
        PostfixOp::Member(name) => Expr::Member(Box::new(lhs), name),
    }).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lex::lexer;
    use chumsky::Parser;

    #[test]
    fn test_dot_member_op() {
        let tokens = lexer().parse(".foo").into_result().unwrap();
        let op = dot_member_op().parse(&tokens[..]).into_result().unwrap();
        match op {
            PostfixOp::Member(name) => assert_eq!(name, "foo"),
            _ => panic!("Expected Member"),
        }
    }

    #[test]
    fn test_colon_member_op() {
        let tokens = lexer().parse("::bar").into_result().unwrap();
        let op = colon_member_op().parse(&tokens[..]).into_result().unwrap();
        match op {
            PostfixOp::Member(name) => assert_eq!(name, "bar"),
            _ => panic!("Expected Member"),
        }
    }
}
