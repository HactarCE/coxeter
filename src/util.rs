pub const EPSILON: f32 = 0.001;

pub fn f32_approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

pub fn factorial(n: usize) -> usize {
    (2..=n).fold(1, |x, y| x * y)
}

pub fn permutation_parity(mut n: usize) -> bool {
    let mut res = false;
    let mut i = 2;
    while n > 0 {
        res ^= (n % i) % 2 != 0;
        n /= i;
        i += 1;
    }
    res
}
