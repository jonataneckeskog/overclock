use super::Expr;
use std::fmt;

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_depth(f, 0)
    }
}

impl Expr {
    /// Recursively writes the AST directly into the formatter buffer.
    fn fmt_with_depth(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = "  ".repeat(depth);

        match self {
            Expr::Lit(l) => write!(f, "{}", l),
            Expr::Var(v) => write!(f, "{}", v),
            Expr::Binary(lhs, op, rhs) => {
                lhs.fmt_with_depth(f, depth)?;
                write!(f, " {} ", op)?;
                rhs.fmt_with_depth(f, depth)
            }
            Expr::Assign(name, body) => {
                write!(f, "{} = ", name)?;
                body.fmt_with_depth(f, depth)
            }
            Expr::Pipe(lhs, rhs) => {
                lhs.fmt_with_depth(f, depth)?;
                write!(f, "\n{}  -> ", indent)?;

                let next_depth = if matches!(**lhs, Expr::Pipe(_, _)) {
                    depth
                } else {
                    depth + 1
                };

                rhs.fmt_with_depth(f, next_depth)
            }
            Expr::Lambda(arg, body) => {
                write!(f, "{} => ", arg)?;
                body.fmt_with_depth(f, depth)
            }
            Expr::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt_with_depth(f, depth + 1)?;
                }
                write!(f, "]")
            }
            Expr::Call(func, args) => {
                func.fmt_with_depth(f, depth)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    arg.fmt_with_depth(f, depth)?;
                }
                write!(f, ")")
            }
            Expr::Member(obj, member) => {
                obj.fmt_with_depth(f, depth)?;
                write!(f, ".{}", member)
            }
        }
    }

    /// Recursively prints the AST with indentation based on depth.
    pub fn pretty_print(&self, depth: usize) -> String {
        struct Wrapper<'a>(&'a Expr, usize);
        impl fmt::Display for Wrapper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt_with_depth(f, self.1)
            }
        }
        format!("{}", Wrapper(self, depth))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Literal};

    #[test]
    fn test_pretty_print_simple() {
        let expr = Expr::Binary(
            Box::new(Expr::Lit(Literal::Int(1))),
            BinaryOp::Add,
            Box::new(Expr::Lit(Literal::Int(2))),
        );
        assert_eq!(expr.pretty_print(0), "1 + 2");
    }

    #[test]
    fn test_pretty_print_pipe() {
        let expr = Expr::Pipe(
            Box::new(Expr::Var("data".to_string())),
            Box::new(Expr::Var("process".to_string())),
        );
        // data
        //   -> process
        assert_eq!(expr.pretty_print(0), "data\n  -> process");
    }

    #[test]
    fn test_pretty_print_complex() {
        // pipeline = [1, 2] -> x => x + 10 -> Console.out()
        // This is left-associative: ([1, 2] -> (x => x + 10)) -> Console.out
        let expr = Expr::Assign(
            "pipeline".to_string(),
            Box::new(Expr::Pipe(
                Box::new(Expr::Pipe(
                    Box::new(Expr::List(vec![
                        Expr::Lit(Literal::Int(1)),
                        Expr::Lit(Literal::Int(2)),
                    ])),
                    Box::new(Expr::Lambda(
                        "x".to_string(),
                        Box::new(Expr::Binary(
                            Box::new(Expr::Var("x".to_string())),
                            BinaryOp::Add,
                            Box::new(Expr::Lit(Literal::Int(10))),
                        )),
                    )),
                )),
                Box::new(Expr::Call(
                    Box::new(Expr::Member(
                        Box::new(Expr::Var("Console".to_string())),
                        "out".to_string(),
                    )),
                    vec![],
                )),
            )),
        );

        let printed = expr.pretty_print(0);
        // pipeline = [1, 2]
        //   -> x => x + 10
        //   -> Console.out()
        assert!(printed.contains("pipeline = [1, 2]"));
        assert!(printed.contains("\n  -> x => x + 10"));
        assert!(printed.contains("\n  -> Console.out()"));
    }
}

