use std::fmt;

/// Supported literals in the language.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Bool(bool),   // "true"
    Int(i64),     // "42"
    Float(f64),   // "3.14"
    Char(char),   // "'a'"
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Int(i) => write!(f, "{}", i),
            Literal::Float(fl) => write!(f, "{}", fl),
            Literal::Char(c) => write!(f, "'{}'", c),
        }
    }
}

/// Supported binary operators.
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add, // "+"
    Sub, // "-"
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
        }
    }
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

pub mod printer;
