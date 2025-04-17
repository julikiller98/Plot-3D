use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Neg, Sub};
use num_complex::Complex;
use std::f64::consts::PI;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Hash, Eq, PartialEq)]
enum Expression {
    Product(Vec<Expression>),
    Sum(Vec<Expression>),
    Var(VarName),
    Const(ConstValue),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Power(Box<Expression>, Box<Expression>),
    Ln(Box<Expression>),
}

#[repr(transparent)]
#[derive(Clone)]
struct ConstValue(f64);

const TOLERANCE: f64 = f64::EPSILON * 100f64;

impl PartialEq<Self> for ConstValue {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < f64::EPSILON * 100f64
    }
}

impl Hash for ConstValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64((self.0 - (self.0 % TOLERANCE)) as u64);
    }
}

impl Display for ConstValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Eq for ConstValue {}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Product(a) => write!(f, "{}", a.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" * ")),
            Expression::Sum(a) => write!(f, "({})", a.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" + ")),
            Expression::Var(a) => write!(f, "{}", a.0),
            Expression::Const(a) => write!(f, "{}", a),
            Expression::Sin(a) => write!(f, "sin({})", a),
            Expression::Cos(b) => write!(f, "cos({})", b),
            Expression::Power(a, b) => write!(f, "({} ^ {})", a, b),
            Expression::Ln(a) => write!(f, "ln({})", a),
        }
    }
}


#[repr(transparent)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct VarName(String);

impl Expression {
    fn eval(&self, vars: &HashMap<VarName, Expression>) -> f64 {
        match self {
            Expression::Product(a) => {
                a.iter().map(|x| x.eval(vars)).product()
            }
            Expression::Sum(a) => {
                a.iter().map(|x| x.eval(vars)).sum()
            }
            Expression::Const(a) => {a.0.clone()},
            Expression::Var(a) => {vars[a].eval(&vars)}
            Expression::Sin(a) => {a.eval(&vars).sin()}
            Expression::Cos(a) => {a.eval(&vars).cos()}
            Expression::Power(a, b) => {
                a.eval(vars).powf(b.eval(vars))
            }
            Expression::Ln(a) => {a.eval(vars).ln()}
        }
    }

    fn derive(&self) -> Box<Expression> {
        match self {
            Expression::Product(a) => {
                derive_product(a.clone())
            }
            Expression::Sum(a) => {
                sum(a.iter().map(|exp| *exp.derive()).collect())
            }
            Expression::Var(_) => {cnst(1f64)}
            Expression::Const(_) => {cnst(0f64)}
            Expression::Sin(a) => { a.derive() * cos(a.clone())}
            Expression::Cos(a) => { a.derive() * -sin(a.clone())}
            Expression::Power(a, b) => {
                pow(a.clone(), b.clone()) * (ln(a.clone()) * b.clone()).derive()
            },
            Expression::Ln(a) => {
                pow(a.clone(), cnst(-1f64)) * a.derive()
            }
        }
    }
}

fn derive_multiply(a: &Box<Expression>, b: &Box<Expression>) -> Box<Expression> {
    (a.clone() * b.derive()) + (a.derive() * b.clone())
}

fn derive_product(parts: Vec<Expression>) -> Box<Expression> {
    if parts.len() == 0 {
        return panic!("Cannot derive product from an empty list!")
    }
    let mut derive = parts[0].derive();
    let mut product = product(vec![parts[0].clone()]);
    for expr in parts.into_iter().map(Box::new).skip(1) {
        derive = (product.clone() * expr.derive()) + (derive * expr.clone());
        product = product * expr;
    }
    derive
}

fn sin(exps: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Sin(exps))
}

fn ln(exps: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Ln(exps))
}

fn pow(expression: Box<Expression>, power: Box<Expression>) -> Box<Expression> {
    if let Expression::Const(exponent) = *power.clone() {
        let exp_val = exponent.0;
        match exp_val {
            0f64 => {
                return cnst(1f64)
            }
            1f64 => {
                return expression
            }
            _ => {}
        }
    }
    Box::new(Expression::Power(expression, power))
}

fn cos(exps: Box<Expression>) -> Box<Expression> {
    Box::new(Expression::Cos(exps))
}

fn cnst(f: f64) -> Box<Expression> {
    Box::new(Expression::Const(ConstValue(f)))
}

fn var(var_name: VarName) -> Box<Expression> {
    Box::new(Expression::Var(var_name))
}

fn sum(vec: Vec<Expression>) -> Box<Expression> {
    let mut out = Vec::new();
    let mut cnst_sum = 0f64;
    for exp in vec {
        if let Expression::Const(a) = exp {
            cnst_sum += a.0;
            continue
        }
        if let Expression::Sum(a)  = exp {
            out.extend(a)
        } else {
            out.push(exp)
        }
    }
    if cnst_sum != 0f64 {
        out.push(*cnst(cnst_sum))
    }
    match out.len() {
        0 => cnst(0f64),
        1 => Box::new(out.pop().unwrap()),
        _ => Box::new(Expression::Sum(out))
    }
}

fn product(vec: Vec<Expression>) -> Box<Expression> {
    let mut out = Vec::new();
    let mut power_map: HashMap<Box<Expression>, Box<Expression>> = HashMap::new();
    let mut cnst_product = 1f64;
    let mut do_again = false;
    for exp in vec {
        match exp {
            Expression::Product(a) => {
                out.extend(a);
                do_again = true;
            }
            Expression::Const(a) => {
                if a.0 == 0f64 {
                    return cnst(0f64);
                }
                cnst_product *= a.0;
            }
            Expression::Power(a, b) => {
                power_map.entry(a).and_modify(|e| *e = e.clone() + b.clone()).or_insert(b);
            }
            _ => {
                power_map.entry(Box::new(exp)).and_modify(|e| *e = e.clone() + cnst(1f64)).or_insert(cnst(1f64));
            }
        }
    }
    for base in power_map.into_iter() {
        out.push(*pow(base.0, base.1))
    }
    if cnst_product != 1f64 {
        out.push(*cnst(cnst_product))
    }
    if out.len() == 1 {
        return Box::new(out.pop().unwrap());
    }
    if do_again {
        product(out)
    } else {
        Box::new(Expression::Product(out))
    }
}

impl Mul for Box<Expression> {
    type Output = Box<Expression> ;

    fn mul(self, rhs: Self) -> Self::Output {
        product(vec![*self, *rhs])
    }
}

impl Neg for Box<Expression> {
    type Output = Box<Expression>;

    fn neg(self) -> Self::Output {
        cnst(-1f64) * self
    }
}

impl Sub for Box<Expression> {
    type Output = Box<Expression>;
    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl Add for Box<Expression> {
    type Output = Box<Expression> ;

    fn add(self, rhs: Self) -> Self::Output {
        sum(vec![*self, *rhs])
    }
}
