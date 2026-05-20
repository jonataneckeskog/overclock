use super::Expr;
use std::fmt;

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pretty_print(0))
    }
}

impl Expr {
    /// Recursively prints the AST with indentation based on depth.
    pub fn pretty_print(&self, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        match self {
            Expr::Lit(l) => format!("{}", l),
            Expr::Var(v) => v.clone(),
            Expr::Binary(lhs, op, rhs) => {
                format!("{} {} {}", lhs.pretty_print(0), op, rhs.pretty_print(0))
            }
            Expr::Assign(name, body) => {
                format!("{} = {}", name, body.pretty_print(depth))
            }
            Expr::Pipe(lhs, rhs) => {
                match &**lhs {
                    Expr::Pipe(_, _) => {
                        format!(
                            "{}\n  {}-> {}",
                            lhs.pretty_print(depth),
                            indent,
                            rhs.pretty_print(depth).trim_start()
                        )
                    }
                    _ => {
                        format!(
                            "{}\n  {}-> {}",
                            lhs.pretty_print(depth),
                            indent,
                            rhs.pretty_print(depth + 1).trim_start()
                        )
                    }
                }
            }
            Expr::Lambda(arg, body) => {
                format!("{} => {}", arg, body.pretty_print(depth))
            }
            Expr::List(items) => {
                if items.is_empty() {
                    "[]".to_string()
                } else {
                    let inner = items
                        .iter()
                        .map(|i| i.pretty_print(depth + 1))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("[{}]", inner)
                }
            }
            Expr::Call(func, args) => {
                let args_str = args
                    .iter()
                    .map(|a| a.pretty_print(0))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", func.pretty_print(0), args_str)
            }
            Expr::Member(obj, member) => {
                format!("{}.{}", obj.pretty_print(0), member)
            }
        }
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

