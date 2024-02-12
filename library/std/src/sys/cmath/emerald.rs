extern "C" {
    pub fn acos(n: f64) -> f64;
    pub fn acosf(n: f32) -> f32;
    pub fn asin(n: f64) -> f64;
    pub fn asinf(n: f32) -> f32;
    pub fn atan(n: f64) -> f64;
    pub fn atan2(a: f64, b: f64) -> f64;
    pub fn atan2f(a: f32, b: f32) -> f32;
    pub fn atanf(n: f32) -> f32;
    pub fn cbrt(n: f64) -> f64;
    pub fn cbrtf(n: f32) -> f32;
    pub fn cosh(n: f64) -> f64;
    pub fn coshf(n: f32) -> f32;
    pub fn expm1(n: f64) -> f64;
    pub fn expm1f(n: f32) -> f32;
    pub fn fdim(a: f64, b: f64) -> f64;
    pub fn fdimf(a: f32, b: f32) -> f32;
    pub fn hypot(x: f64, y: f64) -> f64;
    pub fn hypotf(x: f32, y: f32) -> f32;
    pub fn log1p(n: f64) -> f64;
    pub fn log1pf(n: f32) -> f32;
    pub fn sinh(n: f64) -> f64;
    pub fn sinhf(n: f32) -> f32;
    pub fn tan(n: f64) -> f64;
    pub fn tanf(n: f32) -> f32;
    pub fn tanh(n: f64) -> f64;
    pub fn tanhf(n: f32) -> f32;
    pub fn tgamma(n: f64) -> f64;
    pub fn tgammaf(n: f32) -> f32;
    pub fn lgamma_r(n: f64, s: &mut i32) -> f64;
    pub fn lgammaf_r(n: f32, s: &mut i32) -> f32;
}

// pub fn j() {
//     emerald_builtins::math::fmod(1.0);
// }

// macro_rules! no_mangle {
//     ($(fn $fun:ident($($iid:ident : $ity:ty),+) -> $oty:ty;)+) => {
//         $(
//             #[no_mangle]
//             pub unsafe extern "C" fn $fun($($iid: $ity),+) -> $oty {
//                 emerald_builtins::math::$fun($($iid),+)
//             }
//         )+
//     }
// }

// no_mangle! {
//     fn acos(n: f64) -> f64;
//     fn acosf(n: f32) -> f32;
//     fn asin(n: f64) -> f64;
//     fn asinf(n: f32) -> f32;
//     fn atan(n: f64) -> f64;
//     fn atan2(a: f64, b: f64) -> f64;
//     fn atan2f(a: f32, b: f32) -> f32;
//     fn atanf(n: f32) -> f32;
//     fn cbrt(n: f64) -> f64;
//     fn cbrtf(n: f32) -> f32;
//     fn cosh(n: f64) -> f64;
//     fn coshf(n: f32) -> f32;
//     fn expm1(n: f64) -> f64;
//     fn expm1f(n: f32) -> f32;
//     fn fdim(a: f64, b: f64) -> f64;
//     fn fdimf(a: f32, b: f32) -> f32;
//     fn hypot(x: f64, y: f64) -> f64;
//     fn hypotf(x: f32, y: f32) -> f32;
//     fn log1p(n: f64) -> f64;
//     fn log1pf(n: f32) -> f32;
//     fn sinh(n: f64) -> f64;
//     fn sinhf(n: f32) -> f32;
//     fn tan(n: f64) -> f64;
//     fn tanf(n: f32) -> f32;
//     fn tanh(n: f64) -> f64;
//     fn tanhf(n: f32) -> f32;
//     fn tgamma(n: f64) -> f64;
//     fn tgammaf(n: f32) -> f32;
//     fn lgamma_r(n: f64, s: &mut i32) -> f64;
//     fn lgammaf_r(n: f32, s: &mut i32) -> f32;
// }
