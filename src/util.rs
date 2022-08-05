pub const EPSILON: f32 = 0.001;

pub fn f32_approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}
