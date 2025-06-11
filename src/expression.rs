use peg::error::ParseError;
use peg::str::LineCol;
use crate::expression::parser::expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    VariableRead { identifier: String },
    Const { value: f32 },
    Call { identifier: String, args: Vec<Expr> },
    Add { lhs: Box<Expr>, rhs: Box<Expr> },
    Sub { lhs: Box<Expr>, rhs: Box<Expr> },
    Mul { lhs: Box<Expr>, rhs: Box<Expr> },
    Div { lhs: Box<Expr>, rhs: Box<Expr> },
    Mod { lhs: Box<Expr>, rhs: Box<Expr> },
    Exp { lhs: Box<Expr>, rhs: Box<Expr> },
    Neg { value: Box<Expr> }
}

impl Expr {

    pub fn parse_string(expr: String) -> Result<Expr, ParseError<LineCol>> {
        expression(expr.as_str())
    }

    #[allow(unused)]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            Expr::VariableRead { identifier } => identifier.to_string(),
            Expr::Const { value } => format!("{:.20}", value),
            Expr::Call { identifier, args } => {
                format!(
                    "{identifier}({})",
                    args.iter().fold(String::new(), |acc, expr| acc
                        + "("
                        + &expr.to_string()
                        + ")"
                        + ", ")
                )
            }
            Expr::Add { lhs, rhs } => format!("({}) + ({})", lhs.to_string(), rhs.to_string()),
            Expr::Sub { lhs, rhs } => format!("({}) - ({})", lhs.to_string(), rhs.to_string()),
            Expr::Mul { lhs, rhs } => format!("({}) * ({})", lhs.to_string(), rhs.to_string()),
            Expr::Div { lhs, rhs } => format!("({}) / ({})", lhs.to_string(), rhs.to_string()),
            Expr::Mod { lhs, rhs } => format!("({}) % ({})", lhs.to_string(), rhs.to_string()),
            Expr::Exp { lhs, rhs } => format!("pow({},{})", lhs.to_string(), rhs.to_string()),
            Expr::Neg { value } => format!("-{}", value.to_string())
        }
    }

    pub fn walk<F>(&self, consumer: &mut F)
    where
        F: FnMut(&Expr)
    {
        consumer(self);
        match self {
            Expr::Call {
                identifier: _identifier,
                args,
            } => {
                args.iter().for_each(|arg| arg.walk(consumer));
            }
            Expr::Add { lhs, rhs }
            | Expr::Sub { lhs, rhs }
            | Expr::Mul { lhs, rhs }
            | Expr::Div { lhs, rhs }
            | Expr::Mod { lhs, rhs }
            | Expr::Exp { lhs, rhs } => {
                lhs.walk(consumer);
                rhs.walk(consumer);
            }
            Expr::Neg { value } => value.walk(consumer),
            Expr::Const { .. } | Expr::VariableRead { .. } => {}
        }
    }
}

peg::parser!(pub grammar parser() for str {
    pub rule expression() -> Expr
    = _ e:operations() _ {e}

    rule operations() -> Expr
    = precedence!{
        l:literal() { Expr::Const { value: l } }
        --
        a:(@) _ "+" _ b:@ { Expr::Add{ lhs: Box::new(a), rhs: Box::new(b) } }
        a:(@) _ "-" _ b:@ { Expr::Sub{ lhs: Box::new(a), rhs: Box::new(b) } }
        --
        a:(@) _ "*" _ b:@ { Expr::Mul{ lhs: Box::new(a), rhs: Box::new(b) } }
        a:(@) _ "/" _ b:@ { Expr::Div{ lhs: Box::new(a), rhs: Box::new(b) } }
        a:(@) _ "%" _ b:@ { Expr::Mod{ lhs: Box::new(a), rhs: Box::new(b) } }
        --
        a:(@) _ "**" _ b:@ { Expr::Exp{ lhs: Box::new(a), rhs: Box::new(b) } }
        "-" a:@  { Expr::Neg { value: Box::new(a) } }
        "(" _ e:operations() _ ")" { e }
        --
        i:identifier() _ "(" args:((_ e:operations() _ {e}) ** ",") ")" { Expr::Call { identifier: i, args } }
        i:identifier() { Expr::VariableRead { identifier: i }}
    }

    rule identifier() -> String
    = quiet!{ n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { n.to_owned() } }
    / expected!("identifier")

    rule literal() -> f32
    = n:$("-"?['0'..='9']+("."['0'..='9']*)?) {? n.parse().or(Err("f32"))}

    rule _() =  quiet!{[' ' | '\t']*}
});
