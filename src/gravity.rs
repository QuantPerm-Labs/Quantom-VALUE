use crate::quantperm::mirror_u128;

#[derive(Debug)]
pub struct Gravity {
    pub tau: u128,
}


impl Gravity {
    pub fn derive(retained_mass: u128, mirror: &[u8; 32]) -> Self {
        // Convert raw mirror → physics scalar C
        let c = mirror_u128(mirror);

        // E = retained_mass
        let e = retained_mass;

        // τ = sqrt(E² + C²)

        // Resistance magnitude: τ = sqrt(E^2 + C^2)
let e2 = (e as u128) * (e as u128);
let c2 = (c as u128) * (c as u128);

let tau = integer_sqrt(e2 + c2);


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
