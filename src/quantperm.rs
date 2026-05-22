// quantperm.rs
use blake3;

use crate::gravity::{Gravity};
use crate::perm::Perm;
use crate::euclid::{Euclid, SeedType};
use crate::mirror::Mirror;
use crate::mirrorb::BiasMirror;
pub type Dimension = u64;

/// QuantPerm is a closed thermodynamic state envelope.
///
/// Invariants:
/// - PERM is immutable
/// - All state mutation occurs atomically in `transition`
/// - Structural value (Σ) increases only from real work
/// - Helpers are pure and non-mutating
///
/// Thermodynamic definitions:
/// - τ (tau): resistance magnitude, τ = sqrt(E^2 + C^2),
///   where E = retained_mass (conserved inertia) and
///   C = mirror projection
/// - Δ (delta): total manifold resistance, Δ = CW + CCW arcs
///   mapped deterministically to [0, 180] per arc.
/// - gross_work: τ × Δ
/// - net_work: max(gross_work - Σ, 0)
/// - Σ (structural_value): accumulates net_work only; monotonic.
/// Irreversibility: any movement pays full manifold cost (CW+CCW),
/// amortized by Σ; post-quantum anchoring preserved.

#[repr(C)]
#[derive(Debug)]
pub struct TransitionHeritage {
    pub tau: u128,
    pub delta: u128,
    pub gross_work: u128,
    pub net_work: u128,
    pub origin: SeedType
}

pub struct QuantPerm {
    perm: Perm,
    retained_mass: u128,    // E: conserved inertia
    activation_count: u64,  // transition count
    dimension: Dimension,   // angular coordinate
    structural_value: u128, // Σ: accumulated work residue
}

pub struct Retain {
    pub mass: u128,
    pub from: Dimension,
}

impl QuantPerm {
    pub fn new(perm: Perm) -> Self {
        Self {
            perm,
            retained_mass: 0,
            activation_count: 0,
            dimension: 0,
            structural_value: 0,
        }
    }

    /// Retain external mass (E), measured in bytes of irreversibly
/// accepted external truth (communication payloads, files,
/// proofs, sensor data).
///
/// This does not mutate geometry or trigger transitions.
/// Retained mass influences future work calculations only.
   pub fn retain(&self, mass: u128, from: Dimension) -> Retain {
        Retain { mass, from}
    }
    /// Initialize dimension from PERM geometry.
    /// Wrap-around semantics explicitly documented.
    pub fn set_initial_dimension_from_perm(&mut self) {
        self.dimension = self.perm.dimension() as u64;
    }

    /// The single, atomic entry point for state evolution.
    ///
    /// Internalizes:
    /// - destination derivation
    /// - work calculation (total-manifold Δ)
    /// - Σ credit application
    /// - dimensional mutation
    ///
    /// Returns a receipt representing the post-transition field.
  pub fn transition(&mut self, retain: &Retain, provided_seed: Option<&[u8]>) -> TransitionHeritage {
        // ── 1. Field constants ──
        let euclid = match provided_seed {
            Some(seed) => Euclid::from_seed(seed),
            None => *Euclid::genesis(),
        };
        let origin = euclid.seed_type();

        let from = retain.from;
        

        // ── 2. Destination ──

      let forward =
        Mirror::collapse(
            &euclid,
            from as u128,
        );
        
        let to =
        forward.as_u128() as Dimension;
        let retained_mass = retain.mass;

        // ── 3. Physics (FULL) ──
        let (tau, delta, gross_work) = Self::calculate_work(
            &euclid,
            retained_mass,
            from,
            to,
        );

       // ── 4. Σ credit ──
        let net_work = gross_work.saturating_sub(self.structural_value);
        self.structural_value = self.structural_value.saturating_add(net_work);
      
        // ── 5. Commit state ──
        self.retained_mass = retained_mass;
        self.dimension = to;
        self.activation_count += 1;

        // ── 6. Return FULL RECEIPT ──
        TransitionHeritage {
            tau,
            delta,
            gross_work,
            net_work,
            origin,
        }
    }

    
    /// physics: total-manifold work for a transition.
    ///
    /// Δ = CW + CCW arcs mapped to [0, 180] per arc.
    /// Work = τ × Δ, where τ = sqrt(E^2 + C^2).
    /// Returns (τ, Δ, gross_work).
    pub fn calculate_work(
    euclid: &Euclid,
    retained_mass: u128,
    from: Dimension,
    to: Dimension,
) -> (u128, u128, u128) {

    // ─────────────────────────────────────────────
    // 1. Build dual field geometry (IMPORTANT FIX)
    // ─────────────────────────────────────────────

    let mirror_from = Mirror::collapse(euclid, from as u128);
    let mirror_to   = Mirror::collapse(euclid, to as u128);

    let bias_from = BiasMirror::collapse(euclid, from as u128);
    let bias_to   = BiasMirror::collapse(euclid, to as u128);

    // ─────────────────────────────────────────────
    // 2. Construct full curvature tensor input
    // ─────────────────────────────────────────────

    let mut field_mix = [0u8; 32];

    for i in 0..32 {
        field_mix[i] =
            mirror_from.bytes()[i]
            ^ mirror_to.bytes()[i]
            ^ bias_from.bytes()[i]
            ^ bias_to.bytes()[i];
    }

    // ─────────────────────────────────────────────
    // 3. Resistance magnitude τ (full-field aware)
    // ─────────────────────────────────────────────

    let gravity = Gravity::derive(retained_mass, &field_mix);
    let tau = gravity.tau;

    // ─────────────────────────────────────────────
    // 4. Geodesic Δ (curved manifold distance)
    // ─────────────────────────────────────────────

    let forward_cost: u128 = {
        let diff = to.wrapping_sub(from);
        map_to_180(diff)
    };

    let backward_cost: u128 = {
        let diff = from.wrapping_sub(to);
        map_to_180(diff)
    };

    // Bias correction: curvature asymmetry (key fix)
    let curvature_bias = {
        let b = BiasMirror::collapse(euclid, from as u128);
        let bias_scalar = mirror_u128(b.bytes()) % 180;
        bias_scalar
    };

    let delta = forward_cost
        .saturating_add(backward_cost)
        .saturating_add(curvature_bias);

    // ─────────────────────────────────────────────
    // 5. Gross work (manifold integral)
    // ─────────────────────────────────────────────

    let gross_work = tau.saturating_mul(delta);

    (tau, delta, gross_work)
}
    // ── Read-only observers ──

    pub fn retained_mass(&self) -> u128 {
        self.retained_mass
    }

    pub fn structural_value(&self) -> u128 {
        self.structural_value
    }

    pub fn activations(&self) -> u64 {
        self.activation_count
    }

    pub fn dimension(&self) -> Dimension {
        self.dimension
    }
}

/// Project mirror bytes into u128 space.
/// XOR both halves to avoid bias and use full entropy.
fn mirror_u128(mirror: &[u8; 32]) -> u128 {
    let mut lo = [0u8; 16];
    let mut hi = [0u8; 16];
    lo.copy_from_slice(&mirror[..16]);
    hi.copy_from_slice(&mirror[16..]);
    u128::from_le_bytes(lo) ^ u128::from_le_bytes(hi)
}

/// Deterministic integer square root.
/// Exact integer square root for u128.
/// Returns the largest x such that x*x ≤ n.
fn integer_sqrt(n: u128) -> u128 {
    if n == 0 { return 0; }
    let mut x = n;
    let mut y = (x.saturating_add(1)) >> 1;
    while y < x {
        x = y;
        y = (x.saturating_add(n / x)) >> 1;
    }
    x
}
