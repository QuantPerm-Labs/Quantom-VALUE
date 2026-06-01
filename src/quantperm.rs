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
pub struct Heritage {
    pub state: QuantPerm,
    pub transition: TransitionHeritage,
}

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
     Heritage {
     state: self,
     transition: TransitionHeritage {
        tau,
        delta,
        gross_work,
        net_work,
        origin,
    },
  }
}
    
    /// physics: total-manifold work for a transition.
    ///
    /// Δ = CW + CCW arcs mapped to [0, 180] per arc.
    /// Work = τ × Δ, where τ = sqrt(E^2 + C^2).
    /// Returns (τ, Δ, gross_work).

    pub fn calculate_work(
    seed: &Euclid,
    retained_mass: u128,
    from: Dimension,
    to: Dimension,
) -> (u128, u128, u128){
        
    let c = BiasMirror::collapse(&seed, u128::from(from));
    let scalar = c.as_u128();
    // Resistance magnitude: τ = sqrt(E^2 + C^2)
    
    let diff = if to >= from { to - from } else { from - to };

    let map_to_180 = |d: u64| -> u128 {
        (d as u128)
            .saturating_mul(180)
            .saturating_div(u64::MAX as u128)
    };

    let delta_cw = map_to_180(diff);
    let delta_ccw = map_to_180(u64::MAX - diff);
    let delta = delta_cw.min(delta_ccw);

    let work_delta =
        delta_cw.saturating_add(delta_ccw);

    

        let gravity = Gravity::derive(
        retained_mass,
        scalar,
        delta,
    );

let tau = gravity.tau;
       
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
pub fn mirror_u128(mirror: &[u8; 32]) -> u128 {
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
