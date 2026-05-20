use crate::parser::{BoxedParser, lex::Token, expr::{Expr, BinaryOp, postfix::postfix}};
use chumsky::prelude::*;

/// Parses addition and subtraction operators.
fn add_sub_op<'a>() -> BoxedParser<'a, BinaryOp> {
    just(Token::Plus)
        .to(BinaryOp::Add)
        .or(just(Token::Minus).to(BinaryOp::Sub))
        .boxed()
}

/// Parses binary expressions (left-associative addition and subtraction).
pub fn binary<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    let op = add_sub_op();

    postfix(expr.clone()).foldl(
        op.then(postfix(expr)).repeated(),
        |lhs, (op, rhs)| Expr::Binary(Box::new(lhs), op, Box::new(rhs)),
    ).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lex::lexer;
    use chumsky::Parser;

    #[test]
    fn test_add_op() {
        let tokens = lexer().parse("+").into_result().unwrap();
        let op = add_sub_op().parse(&tokens[..]).into_result().unwrap();
        assert_eq!(op, BinaryOp::Add);
    }
}
