// mirror.rs
use blake3;
use crate::euclid::{Euclid, SeedType};

/// A Mirror is the collapsed projection of (Perm.dimension × Euclid.constant).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mirror([u8; 32]);

impl Mirror {
    /// Collapse a PERM dimension into mirror space using Euclid's constant.
    pub fn collapse(euclid: &Euclid, perm_dimension: u128) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"MIRROR::COLLAPSE::V1");
        hasher.update(euclid.constant()); // exposed constant
        hasher.update(&perm_dimension.to_le_bytes());
        let mut out = [0u8; 32];
        out.copy_from_slice(hasher.finalize().as_bytes());
        Mirror(out)
    }

    /// Seed classification passthrough—observer-safe.
    pub fn seed_type(euclid: &Euclid) -> SeedType {
        euclid.seed_type()
    }

    /// Project mirror bytes into u128 space (for physics).
    pub fn as_u128(&self) -> u128 {
        let mut lo = [0u8; 16];
        let mut hi = [0u8; 16];
        lo.copy_from_slice(&self.0[..16]);
        hi.copy_from_slice(&self.0[16..]);
        u128::from_le_bytes(lo) ^ u128::from_le_bytes(hi)
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
