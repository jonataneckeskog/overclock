/// Supported literals in the language.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Bool(bool),   // "true"
    Int(i64),     // "42"
    Float(f64),   // "3.14"
    Char(char),   // "'a'"
}

/// Supported binary operators.
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add, // "+"
    Sub, // "-"
}

/// Abstract Syntax Tree (AST) for expressions.
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Lit(Literal),                           // "42", "true", etc.
    Var(String),                            // "x"
    Binary(Box<Expr>, BinaryOp, Box<Expr>), // "a + b"
    Assign(String, Box<Expr>),              // "x = 5"
    Pipe(Box<Expr>, Box<Expr>),             // "a -> b"
    Lambda(String, Box<Expr>),              // "x => x"
    List(Vec<Expr>),                        // "[1, 2]"
    Call(Box<Expr>, Vec<Expr>),             // "f(x)"
    Member(Box<Expr>, String),              // "obj.prop"
}
