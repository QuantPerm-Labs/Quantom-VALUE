#[derive(Debug)]
pub struct Gravity {
    pub tau: u128,
}


impl Gravity {
    pub fn derive(retained_mass: u128, mirror: &[u8; 32]) -> Self {
        let c = mirror_u128(mirror);

        // E
        let e = retained_mass;

        // τ = sqrt(E² + C²)
        let tau = integer_sqrt(
            e.saturating_mul(e)
             .saturating_add(c.saturating_mul(c))
        );

        Gravity { tau }
    }
}

/// Deterministic integer square root for u128.
/// Returns the largest x such that x*x ≤ n.
fn integer_sqrt(n: u128) -> u128 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x.saturating_add(1)) >> 1;
    while y < x {
        x = y;
        // Use saturating_add to avoid overflow in debug builds
        y = (x.saturating_add(n / x)) >> 1;
    }
    x
}
