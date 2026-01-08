// perm.rs
#[allow(dead_code)]
use blake3;

/// A PERM is a pure dimensional coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Perm {
    dimension: u128,
}

impl Perm {
    /// Protocol constants for deterministic implementation
    pub const PROTOCOL: &'static str = "PERM::GENESIS";
    pub const DOMAIN_SIZE: u16 = 2048;
    pub const NUM_INDICES: usize = 12;
    pub const ENTROPY_SIZE: usize = 32; // 256 bits

    /// - Same inputs → same dimension across all implementations
    pub fn genesis(
        domain_indices: &[u16; Self::NUM_INDICES],
        entropy: &[u8; Self::ENTROPY_SIZE],
    ) -> Result<Self, &'static str> {
        // Validate domain indices
        if !domain_indices.iter().all(|&i| i < Self::DOMAIN_SIZE) {
            return Err("Domain indices must be < 2048");
        }

        // Initialize BLAKE3 hasher with domain separator
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"PERM::GENESIS");

        // Serialize indices deterministically
        for &i in domain_indices {
            hasher.update(&i.to_le_bytes());
        }

        // Fixed-size entropy ensures reproducibility
        hasher.update(entropy);

        // Finalize hash
        let hash = hasher.finalize();
        let hash_bytes = hash.as_bytes();

        // Project hash into 128-bit dimension by XORing both halves
        let lo = u128::from_le_bytes(hash_bytes[0..16].try_into().map_err(|_| "Hash slice conversion failed")?);
        let hi = u128::from_le_bytes(hash_bytes[16..32].try_into().map_err(|_| "Hash slice conversion failed")?);

        Ok(Self {
            dimension: lo ^ hi,
        })
    }

    ///
    /// Hashes variable-length entropy to 32 bytes, then calls fixed-size genesis.
    pub fn genesis_construct(
        domain_indices: &[u16; Self::NUM_INDICES],
        entropy: &[u8],
    ) -> Result<Self, &'static str> {
        if entropy.is_empty() {
            return Err("Entropy must not be empty");
        }

        // Hash variable entropy to fixed size
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"PERM::VAR_ENTROPY");
        hasher.update(&(entropy.len() as u64).to_le_bytes());
        hasher.update(entropy);
        let fixed_entropy = hasher.finalize();

        // Convert to [u8; 32] safely
      let mut entropy_bytes = [0u8; 32];
    entropy_bytes.copy_from_slice(fixed_entropy.as_bytes());

        Self::genesis(domain_indices, &entropy_bytes)
    }

    /// Wrap an existing coordinate directly.
    pub fn from_u128(dimension: u128) -> Self {
        Self { dimension }
    }

    /// Return the internal dimension
    pub fn dimension(&self) -> u128 {
        self.dimension
    }
}

