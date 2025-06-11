use crate::expression::Expr;

pub struct Function {
    pub expression: Expr,
}

impl Function {
}

pub fn parse_anita_fn(fn_string: String) -> Result<Function, String> {
    let expression = Expr::parse_string(fn_string).map_err(|err| err.to_string())?;
    let mut error = None;
    expression.walk(
        &mut |expr: &Expr| {
            match expr {
                Expr::VariableRead { identifier } => {
                    match identifier.as_str() {
                        "x" | "y" | "t" | "e" | "pi" => {}
                        var => {
                            error = Some(format!("Use of undefinied var {}", var));
                        }
                    }
                }
                Expr::Call { identifier, args } => {
                    match identifier.as_str() {
                        "sin" | "cos" | "tan" | "cosh" | "sinh" | "asin" | "acos" | "atan" => {
                            if args.len() != 1 {
                                error = Some(format!("Wrong argument count on {}", identifier));
                            }
                        }
                        identifier => {
                            error = Some(format!("Unknown function {}", identifier));
                        }
                    }
                }
                _ => {}
            }
        }
    );
    if error.is_none() {
        Ok(Function {expression})
    } else {
        Err(error.unwrap())
    }
}