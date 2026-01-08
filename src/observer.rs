use crate::quantperm::{QuantPerm, Dimension};
use crate::euclid::{Euclid, SeedType};

/// The projected, shareable truth of a coordinate.
/// Immutable, deterministic, and side-effect free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionObservation {
    pub dimension: Dimension,
    pub structural_value: u128, // Σ — stored work / credit
    pub activations: u64,       // Sequential transition count
    pub seed: SeedType,         // Governing physics
}

/// Read-only optical lens over QuantPerm.
/// Has zero authority to mutate state.
pub struct Observer;

impl Observer {
    /// Observe a QuantPerm under a given seed context.
    /// Deterministic, memoryless, and side-effect free.
    pub fn observe(qp: &QuantPerm, seed: Option<&[u8]>) -> DimensionObservation {
        let euclid = match seed {
            Some(s) => Euclid::from_seed(s),
            None => *Euclid::genesis(),
        };

        DimensionObservation {
            dimension: qp.dimension(),
            structural_value: qp.structural_value(),
            activations: qp.activations(),
            seed: euclid.seed_type(), // ✅ correct accessor
        }
    }

    /// Structural density of a coordinate.
    /// Interpreted as "laminar smoothness" per transition.
    /// Returns None if activations == 0.
    pub fn calculate_density(obs: &DimensionObservation) -> Option<u128> {
        if obs.activations == 0 {
            None
        } else {
            Some(obs.structural_value / obs.activations as u128)
        }
    }
}
