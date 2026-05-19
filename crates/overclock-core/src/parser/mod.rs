use chumsky::{Boxed, error::Rich, extra};

pub mod expr;
pub mod lex;

pub use lex::Token;
pub type BoxedParser<'a, O> = Boxed<'a, 'a, &'a [Token], O, extra::Err<Rich<'a, Token>>>;
