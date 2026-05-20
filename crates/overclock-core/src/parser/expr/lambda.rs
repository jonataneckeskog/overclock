use crate::ast::Expr;
use crate::parser::{BoxedParser, lex::Token};
use super::pipe::pipe;
use chumsky::prelude::*;

/// Parses a lambda expression, e.g., x => x + 1.
fn lambda_only<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    select! { Token::Ident(name) => name }
        .then_ignore(just(Token::MapArrow))
        .then(expr)
        .map(|(name, body)| Expr::Lambda(name, Box::new(body)))
        .boxed()
}

/// Parses either a lambda or a pipe expression.
pub fn lambda<'a>(expr: BoxedParser<'a, Expr>) -> BoxedParser<'a, Expr> {
    lambda_only(expr.clone()).or(pipe(expr)).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::parser::lexer;
    use chumsky::Parser;

    #[test]
    fn test_lambda_only() {
        let tokens = lexer().parse("x => 42").into_result().unwrap();
        let dummy = any().map(|_| Expr::Lit(Literal::Int(42))).ignored().map(|_| Expr::Lit(Literal::Int(42))).boxed();
        let ast = lambda_only(dummy).parse(&tokens[..]).into_result().unwrap();
        if let Expr::Lambda(name, _) = ast {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Lambda");
        }
    }
}
