// biasmirror.rs

use crate::euclid::{Euclid, SeedType};
use crate::mirror::Mirror;

/// A BiasMirror is a deterministic field-biased projection
/// of a PERM dimension through Euclidean curvature.
/// The projection behaves as:
///
///     M = rotate(D ⊕ C) + bias(D,C)
///
/// where:
/// - D = PERM dimension
/// - C = Euclid constant field
/// - bias = deterministic geometric deformation
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiasMirror([u8; 32]);

impl BiasMirror {

    /// deterministic geometric distortion.
    pub fn collapse(
        euclid: &Euclid,
        perm_dimension: u128,
    ) -> Self {

        let mirror = Mirror::collapse(euclid, perm_dimension,
            );
        
         Self(
            *mirror.invert().bytes()

        )
    }


    /// Seed classification passthrough.
    pub fn seed_type(euclid: &Euclid) -> SeedType {
        euclid.seed_type()
    }

    /// Project biased field into u128 space.
    pub fn as_u128(&self) -> u128 {

        let mut lo = [0u8; 16];
        let mut hi = [0u8; 16];

        lo.copy_from_slice(&self.0[..16]);
        hi.copy_from_slice(&self.0[16..]);

        u128::from_le_bytes(lo)
            ^ u128::from_le_bytes(hi)
    }

    /// Raw bytes.
    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
