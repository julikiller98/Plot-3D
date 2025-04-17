use num_complex::Complex32;
use anita::jit::JITError;
use anita::compile_expression;
use crate::complex_anita::ComplexAnitaType;
use crate::complex_functions::ComplexFunctions;

pub enum Function {
    StaticComplex(Box<dyn Fn(Complex32) -> Complex32 + Send + Sync>),
    TimedComplex(Box<dyn Fn(Complex32, Complex32) -> Complex32 + Send + Sync>),
    StaticPlane(Box<dyn Fn(Complex32, Complex32) -> Complex32 + Send + Sync>),
    TimedPlane(Box<dyn Fn(Complex32, Complex32, Complex32) -> Complex32 + Send + Sync>),
}

impl Function {
    pub fn is_timed(&self) -> bool {
        match self {
            Function::StaticComplex(_) => false,
            Function::StaticPlane(_) => false,
            Function::TimedComplex(_) => true,
            Function::TimedPlane(_) => true,
        }
    }
}

pub fn parse_anita_fn(fn_string: String) -> Result<Function, JITError> {
    if fn_string.is_empty() {
        return Ok(Function::StaticComplex(Box::new(|x| 0f32.into())))
    }
    let expression = fn_string.clone().replace("**", "^");
    let parsing_result = compile_expression!(expression.clone(), (x) -> ComplexAnitaType, ComplexFunctions);
    if let Ok(static_function) = parsing_result {
        return Ok(Function::StaticComplex(Box::new(move |x| static_function(ComplexAnitaType::from(x)).into())))
    }
    let err = parsing_result.err().unwrap();
    match err {
        JITError::UseOfUninitializedVariables(vars) => {
            let planed = vars.contains(&String::from("y"));
            let timed = vars.contains(&String::from("t"));
            if timed && planed {
                let parsing_result = compile_expression!(expression.clone(), (x,y,t) -> ComplexAnitaType, ComplexFunctions)?;
                return Ok(Function::TimedPlane(Box::new(move |x,y,t| parsing_result(ComplexAnitaType::from(x), ComplexAnitaType::from(y), ComplexAnitaType::from(t)).into())));
            }
            if timed {
                let parsing_result = compile_expression!(expression.clone(), (x,t) -> ComplexAnitaType, ComplexFunctions)?;
                return Ok(Function::TimedComplex(Box::new(move |x,t| parsing_result(ComplexAnitaType::from(x), ComplexAnitaType::from(t)).into())));
            }
            if planed {
                let parsing_result = compile_expression!(expression.clone(), (x,y) -> ComplexAnitaType, ComplexFunctions)?;
                return Ok(Function::StaticPlane(Box::new(move |x,y| parsing_result(ComplexAnitaType::from(x), ComplexAnitaType::from(y)).into())));
            }
            Err(JITError::UseOfUninitializedVariables(vars))
        }
        err => Err(err),
    }
}