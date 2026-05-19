#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Void,          // "Void"
    Ident(String), // Identifyer
    Num(String),   // Numerical value
    Indent,        // Virtual token: Layout shifted right
    Dedent,        // Virtual token: Layout shifted left
    Assign,        // "="
    Arrow,         // "->"
    MapArrow,      // "=>"
}
