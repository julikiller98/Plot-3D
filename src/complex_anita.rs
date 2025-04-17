use crate::functions;
use crate::functions::Function;
use anita::cranelift::frontend::FunctionBuilder;
use anita::cranelift::prelude::{types, FloatCC, InstBuilder, Type, Value};
use anita::jit::types::AnitaType;
use num_complex::{Complex32, ComplexFloat};
use rand::random;
use std::ops::Deref;
use std::simd::f64x2;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ComplexAnitaType(pub f64x2);

impl AnitaType for ComplexAnitaType {
    fn cranelift_repr() -> Type {
        types::F64X2
    }

    fn constant(builder: &mut FunctionBuilder, value: f32) -> Value {
        let cnst_value = builder.ins().f64const(value as f64);
        let out = builder.ins().splat(types::F64X2, cnst_value);
        let cnst_null = builder.ins().f64const(0f64);
        builder.ins().insertlane(out, cnst_null, 1)
    }

    fn add(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fadd(lhs, rhs)
    }

    fn sub(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fsub(lhs, rhs)
    }

    fn mul(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        let (a, ai) = _extract_complex_lanes(builder, lhs);
        let (b, bi) = _extract_complex_lanes(builder, rhs);
        _mul(builder, lhs, a, ai, b, bi)
    }

    fn div(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        //upper part of fraction:
        let (a, ai) = _extract_complex_lanes(builder, lhs);
        let (b, bi) = _extract_complex_lanes(builder, rhs);

        let bi = builder.ins().fneg(bi); //negate di
        let upper_part = _mul(builder, lhs, a, ai, b, bi);

        let squared_ind = builder.ins().fmul(rhs, rhs);
        let (squared_c, squared_d) = _extract_complex_lanes(builder, squared_ind);
        // divide re and i of upper part by (c * c - d * d)
        let sc_minus_sd = builder.ins().fadd(squared_c, squared_d);
        let lower_part = builder.ins().splat(types::F64X2, sc_minus_sd);
        builder.ins().fdiv(upper_part, lower_part)
    }

    fn modulo(builder: &mut FunctionBuilder, value: Value, modulus: Value) -> Value {
        let div = builder.ins().fdiv(value, modulus);
        let trunc = builder.ins().trunc(div);
        let full_div = builder.ins().fmul(trunc, modulus);
        builder.ins().fsub(value, full_div)
    }

    fn neg(builder: &mut FunctionBuilder, value: Value) -> Value {
        builder.ins().fneg(value)
    }

    fn eq(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fcmp(FloatCC::Equal, lhs, rhs)
    }

    fn neq(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs)
    }

    fn gt(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs)
    }

    fn lt(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fcmp(FloatCC::LessThan, lhs, rhs)
    }

    fn geq(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs)
    }

    fn leq(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs)
    }

    fn and(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        let zero = builder.ins().f64const(0.0);
        let two = builder.ins().f64const(2.0);
        let lhs = builder.ins().fcmp(FloatCC::NotEqual, lhs, zero);
        let rhs = builder.ins().fcmp(FloatCC::NotEqual, rhs, zero);
        let intermediate = builder.ins().fadd(lhs, rhs);
        builder.ins().fcmp(FloatCC::Equal, intermediate, two)
    }

    fn or(builder: &mut FunctionBuilder, lhs: Value, rhs: Value) -> Value {
        let zero = builder.ins().f64const(0.0);
        let lhs = builder.ins().fcmp(FloatCC::NotEqual, lhs, zero);
        let rhs = builder.ins().fcmp(FloatCC::NotEqual, rhs, zero);
        let intermediate = builder.ins().fadd(lhs, rhs);
        builder.ins().fcmp(FloatCC::NotEqual, intermediate, zero)
    }

    fn not(builder: &mut FunctionBuilder, value: Value) -> Value {
        let zero = builder.ins().f64const(0.0);
        builder.ins().fcmp(FloatCC::Equal, value, zero)
    }

    extern "C" fn inbuilt_pow(self, value: Self) -> Self {
        let complex: Complex32 = self.into();
        ComplexAnitaType::from(complex.powc(value.into()))
    }
}

fn _mul(
    builder: &mut FunctionBuilder,
    result_override: Value,
    a: Value,
    ai: Value,
    b: Value,
    bi: Value,
) -> Value {
    //(a + bi) * (c + di) =  (r: a * c - b * d, i:  a * d + c * b)
    let a_c = builder.ins().fmul(a, b);
    let b_d = builder.ins().fmul(ai, bi);
    let r = builder.ins().fsub(a_c, b_d);
    let a_d = builder.ins().fmul(a, bi);
    let c_b = builder.ins().fmul(b, ai);
    let i = builder.ins().fadd(a_d, c_b);
    let l0 = builder.ins().insertlane(result_override, r, 0);
    builder.ins().insertlane(l0, i, 1)
}

fn _extract_complex_lanes(builder: &mut FunctionBuilder, complex_value: Value) -> (Value, Value) {
    let re = builder.ins().extractlane(complex_value, 0);
    let im = builder.ins().extractlane(complex_value, 1);
    (re, im)
}

impl From<Complex32> for ComplexAnitaType {
    fn from(value: Complex32) -> Self {
        ComplexAnitaType(f64x2::from_array([value.re as f64, value.im as f64]))
    }
}

impl Into<Complex32> for ComplexAnitaType {
    fn into(self) -> Complex32 {
        let value_array = self.0.as_array();
        Complex32::new(value_array[0] as f32, value_array[1] as f32)
    }
}

impl Deref for ComplexAnitaType {
    type Target = f64x2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test]
fn test_complex_mul() {
    for _ in 0..100 {
        let a = Complex32::new(random(), random());
        let b = Complex32::new(random(), random());
        let antia_fn = functions::parse_anita_fn(format!(
            "const({},{}) * const({}, {})",
            a.re, a.im, b.re, b.im
        ))
        .unwrap();
        match antia_fn {
            Function::StaticComplex(static_function) => {
                let expected = a * b;
                let delta = static_function(Complex32::ZERO) - expected;
                assert!(delta.abs() < 0.001f32);
            }
            _ => {
                panic!("Wrong function type");
            }
        }
    }
}

#[test]
fn test_complex_div() {
    for _ in 0..100 {
        let a = Complex32::new(random(), random());
        let b = Complex32::new(random(), random());
        let antia_fn = functions::parse_anita_fn(format!(
            "const({},{}) / const({}, {})",
            a.re, a.im, b.re, b.im
        ))
        .unwrap();
        match antia_fn {
            Function::StaticComplex(static_function) => {
                let expected = a / b;
                let delta = static_function(Complex32::ZERO) - expected;
                assert!(delta.abs() < 0.001f32);
            }
            _ => {
                panic!("Wrong function type");
            }
        }
    }
}
