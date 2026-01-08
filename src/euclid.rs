// src/euclid.rs
use blake3;
use std::sync::OnceLock;

/// Classification of seed type: public Euclid or fork.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedType {
    Euclid,
    Fork,
}

/// Euclid enforces the constant and seed classification.
#[derive(Clone, Copy)]
pub(super) struct Euclid {
    constant: [u8; 32],
    seed: SeedType,
}

/// Global Genesis instance, initialized once at startup.
static GENESIS: OnceLock<Euclid> = OnceLock::new();

impl Euclid {
    pub fn genesis() -> &'static Self {
        GENESIS.get_or_init(|| {
            // Hard‑coded phrase: this is the universal anchor.
            let hash = blake3::hash(b"EUCLID::BLACK_HOLE::GENESIS::HERITAGE");
            Euclid {
                constant: *hash.as_bytes(),
                seed: SeedType::Euclid,
            }
        })
    }
    /// Create a forked Euclid from a developer seed.
    pub fn from_seed(seed: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"EUCLID::FORK");
        hasher.update(seed);
        Euclid {
            constant: *hasher.finalize().as_bytes(),
            seed: SeedType::Fork,
        }
    }

    /// Getter for the constant—Mirror uses this to collapse.
    pub fn constant(&self) -> &[u8; 32] {
        &self.constant
    }

    /// Observer‑safe classification.
    pub fn seed_type(&self) -> SeedType {
        self.seed
    }
}
