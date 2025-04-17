use crate::complex_anita::ComplexAnitaType;
use anita::anita_macros::function_manager;
use anita::jit::types::AnitaType;
use num_complex::{Complex32, ComplexFloat};
use std::convert::Into;
use std::simd::f64x2;

pub struct ComplexFunctions {}

#[function_manager]
impl ComplexFunctions {

    #[name = "const"]
    fn constant(re: ComplexAnitaType, im: ComplexAnitaType) -> ComplexAnitaType{
        ComplexAnitaType(f64x2::from_array([re.as_array()[0],  im.as_array()[0]]))
    }

    fn sin(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.sin())
    }
    fn cos(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.cos())
    }

    fn tan(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.tan())
    }

    fn tanh(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.tanh())
    }

    fn sinh(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.sinh())
    }

    fn cosh(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.cosh())
    }

    fn exp(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        ComplexAnitaType::from(x.exp())
    }

    fn re(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        let complex_result: Complex32 = x.re().into();
        ComplexAnitaType::from(complex_result)
    }

    fn im(x: ComplexAnitaType) -> ComplexAnitaType{
        let x: Complex32 = x.into();
        let complex_result: Complex32 = x.im().into();
        ComplexAnitaType::from(complex_result)
    }

    fn pi() -> ComplexAnitaType{
        ComplexAnitaType::from(Complex32::from(std::f32::consts::PI))
    }

}

