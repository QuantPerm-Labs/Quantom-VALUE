#[derive(Debug)]
pub struct Gravity {
    pub tau: u128,
}

impl Gravity {
    pub fn derive(retained_mass: u128, mirror: &[u8; 32]) -> Self {
        let mut lo = [0u8; 16];
        let mut hi = [0u8; 16];
        lo.copy_from_slice(&mirror[..16]);
        hi.copy_from_slice(&mirror[16..]);
        let c = u128::from_le_bytes(lo) ^ u128::from_le_bytes(hi);

        // τ = sqrt(E² + C²), all operations saturating
        let e2 = retained_mass.saturating_mul(retained_mass);
        let c2 = c.saturating_mul(c);
        let tau = integer_sqrt(e2.saturating_add(c2));

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
