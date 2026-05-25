pub mod ops;
mod pipeline;

pub use pipeline::Pipeline;

/// A declarative macro for building pipelines.
/// 
/// # Example
/// ```
/// let p = pipeline![
///     pipe(|x| Some(x * 2)),
///     sink(|x| println!("{}", x))
/// ];
/// ```
#[macro_export]
macro_rules! pipeline {
    // Base case: start the pipeline
    ( $( $op:ident ( $($args:tt)* ) ),* $(,)? ) => {
        $crate::Pipeline::start()
            $( .$op ( $($args)* ) )*
    };
}
