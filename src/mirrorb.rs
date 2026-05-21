// biasmirror.rs

use crate::euclid::{Euclid, SeedType};

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

        let constant = euclid.constant();

        let mut out = [0u8; 32];

        // Expand dimension into 32-byte field
        let mut dim_bytes = [0u8; 32];
        dim_bytes[..16].copy_from_slice(&perm_dimension.to_le_bytes());
        dim_bytes[16..].copy_from_slice(
            &(perm_dimension.rotate_left(37)).to_le_bytes()
        );

        // Deterministic field-biased projection
        for i in 0..32 {

            let d = dim_bytes[i];
            let c = constant[i];

            // Geometric boundary bias
            let boundary =
                ((i as u8).wrapping_mul(17))
                ^ ((perm_dimension as u8).rotate_left(i as u32 % 8));

            // Directional field interaction
            let projected =
                d.rotate_left((c % 8) as u32)
                 ^ c
                 ^ boundary;

            // Local curvature deformation
            out[i] =
                projected
                .wrapping_add(c.rotate_right(3))
                .wrapping_sub(d.rotate_left(1));
        }

        Self(out)
    }

    /// Inverted manifold projection.
    ///
    /// Produces directional opposite curvature.
    pub fn invert(&self) -> Self {

        let mut out = [0u8; 32];

        for i in 0..32 {
            out[i] = !self.0[i];
        }

        Self(out)
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
