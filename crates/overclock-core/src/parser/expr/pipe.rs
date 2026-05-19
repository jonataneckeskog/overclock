use crate::parser::{BoxedParser, lex::Token, expr::{Expr, binary::binary}};
use chumsky::prelude::*;

fn pipe_op<'a>() -> BoxedParser<'a, Token> {
    just(Token::Arrow).boxed()
}

pub fn pipe<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    binary(expr.clone()).foldl(
        pipe_op().then(binary(expr)).repeated(),
        |lhs, (_, rhs)| Expr::Pipe(Box::new(lhs), Box::new(rhs)),
    ).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lex::lexer;
    use chumsky::Parser;

    #[test]
    fn test_pipe_op() {
        let tokens = lexer().parse("->").into_result().unwrap();
        let _ = pipe_op().parse(&tokens[..]).into_result().unwrap();
    }
}
