pub mod expr;
pub mod stmt;

use chumsky::prelude::*;

pub type BoxedParser<'a, O> = Boxed<'a, 'a, &'a str, O, extra::Err<Rich<'a, char>>>;

pub fn comment<'a>() -> BoxedParser<'a, ()> {
    just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .ignored()
        .boxed()
}

pub fn skip<'a>() -> BoxedParser<'a, ()> {
    text::whitespace()
        .at_least(1)
        .or(comment())
        .repeated()
        .ignored()
        .boxed()
}
