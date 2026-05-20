#[cfg(test)]
mod tests {
    use crate::ast::{BinaryOp, Expr};
    use crate::parser::expr;
    use crate::parser::lexer;
    use chumsky::Parser;

    #[test]
    fn test_fibonacci_pipeline() {
        let src = "fibbonacci = [1, 2] -> (_ => self.-1 + self.-2)";
        let tokens = lexer().parse(src).into_result().unwrap();
        let ast = expr().parse(&tokens[..]).into_result().unwrap();

        match ast {
            Expr::Assign(name, body) => {
                assert_eq!(name, "fibbonacci");
                match *body {
                    Expr::Pipe(lhs, rhs) => {
                        if let Expr::List(items) = *lhs {
                            assert_eq!(items.len(), 2);
                        } else {
                            panic!("Expected list, got {:?}", lhs);
                        }
                        if let Expr::Lambda(arg, body) = *rhs {
                            assert_eq!(arg, "_");
                            if let Expr::Binary(l, op, r) = *body {
                                assert_eq!(op, BinaryOp::Add);
                                match (*l, *r) {
                                    (Expr::Member(_, m1), Expr::Member(_, m2)) => {
                                        assert_eq!(m1, "-1");
                                        assert_eq!(m2, "-2");
                                    }
                                    _ => panic!("Expected members"),
                                }
                            }
                        }
                    }
                    _ => panic!("Expected pipe"),
                }
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_console_pipeline() {
        let src = "fibbonacci -> take(10) -> Console::out";
        let tokens = lexer().parse(src).into_result().unwrap();
        let ast = expr().parse(&tokens[..]).into_result().unwrap();

        if let Expr::Pipe(lhs, rhs) = ast {
            if let Expr::Pipe(l, m) = *lhs {
                assert_eq!(*l, Expr::Var("fibbonacci".to_string()));
                match *m {
                    Expr::Call(f, args) => {
                        assert_eq!(*f, Expr::Var("take".to_string()));
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected call"),
                }
            }
            match *rhs {
                Expr::Member(e, name) => {
                    assert_eq!(*e, Expr::Var("Console".to_string()));
                    assert_eq!(name, "out");
                }
                _ => panic!("Expected member access"),
            }
        } else {
            panic!("Expected pipe chain");
        }
    }
}
