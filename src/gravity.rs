use crate::quantperm::mirror_u128;

#[derive(Debug)]
pub struct Gravity {
    pub tau: u128,
}



impl Gravity {

    /// Manifold gravitational interaction law.
    ///
    /// τ = (E × C) / (Δ² + 1)
    ///
    /// where:
    /// - E = retained_mass
    /// - C = mirror curvature projection
    /// - Δ = manifold displacement
    ///
    pub fn derive(
        retained_mass: u128,
        mirror_scalar: u128,
        delta: u128,
    ) -> Self {

        let e = retained_mass;
        let c = mirror_scalar;

        // denominator = Δ² + 1
        let distance =
            delta
                .saturating_mul(delta)
                .saturating_add(1);

        // gravitational manifold interaction
        let tau =
            e.saturating_mul(c)
             .saturating_div(distance);

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
