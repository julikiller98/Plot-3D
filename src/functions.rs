use num_complex::Complex32;

pub enum Function {
    StaticComplex(Box<dyn Fn(Complex32) -> Complex32 + Send + Sync>),
    TimedComplex(Box<dyn Fn(Complex32, Complex32) -> Complex32 + Send + Sync>),
    StaticPlane(Box<dyn Fn(Complex32, Complex32) -> Complex32 + Send + Sync>),
    TimedPlane(Box<dyn Fn(Complex32, Complex32, Complex32) -> Complex32 + Send + Sync>),
}

impl Function {
    pub fn rebuild_on_update(&self) -> bool {
        match self {
            Function::StaticComplex(_) => false,
            Function::StaticPlane(_) => false,
            Function::TimedComplex(_) => true,
            Function::TimedPlane(_) => false,
        }
    }
}

pub fn parse_anita_fn(fn_string: String) -> Result<Function, String> {
    Ok(Function::TimedPlane(Box::new(|a, b, c| 0f32.into())))
}