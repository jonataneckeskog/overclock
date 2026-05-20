use crate::ast::Expr;
use chumsky::{Boxed, Parser, error::Rich, extra};

mod expr;
mod lex;

use expr::expr;
use lex::{Token, lexer};

pub type BoxedParser<'a, O, I = &'a [Token], E = extra::Err<Rich<'a, Token>>> =
    Boxed<'a, 'a, I, O, E>;

/// The main entry point for the front-end.
/// Takes raw text, lexes it, parses it, and returns the AST.
pub fn parse(source: &str) -> Result<Expr, String> {
    let (tokens, lex_errs) = lexer().parse(source).into_output_errors();

    // These could be mapped to custom diagnostic struct using something like `ariadne` or `miette`.
    if !lex_errs.is_empty() {
        return Err(format!("Lexer errors: {:?}", lex_errs));
    }

    // Chumsky can recover from errors, so output might exist even if there were errors.
    let tokens = tokens.unwrap_or_default();

    let (ast, parse_errs) = expr().parse(tokens.as_slice()).into_output_errors();

    if !parse_errs.is_empty() {
        return Err(format!("Parser errors: {:?}", parse_errs));
    }

    ast.ok_or_else(|| "Failed to parse expression".to_string())
}
