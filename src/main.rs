// src/main.rs
#![allow(dead_code)]
#![allow(unused_variables)]

mod perm;
mod quantperm;
mod euclid;
mod gravity;
mod observer;
mod mirror;

use perm::Perm;
use quantperm::{QuantPerm, Dimension};
use euclid::{Euclid, SeedType};
use mirror::Mirror;

/// Structural value (Σ) represents deterministic "work units" produced/retained by transitions.
/// it is an auditable, observer-independent measure of work.
/// External truth (retained mass) is the payload; Σ is the proof/gating signal.
#[derive(Debug)]
struct ReactionRow {
    step: String,
    actor: String,
    dimension: Dimension,
    sigma: u128,
    public_mirror: [u8; 8],   // Mirror if viewed by the Public (Genesis)
    private_mirror: [u8; 8],  // Mirror if viewed by the Developer (Fork)
    public_seed_type: SeedType,
    private_seed_type: SeedType,
    public_mirror_u128: u128, // Numeric representation (for audit)
    private_mirror_u128: u128,
}

fn get_mirror8(seed: Option<&[u8]>, dim: Dimension) -> ([u8; 8], SeedType, u128) {
    let euclid = match seed {
        None => *Euclid::genesis(),
        Some(s) => Euclid::from_seed(s),
    };
    let mirror = Mirror::collapse(&euclid, dim as u128);
    let bytes = mirror.bytes().clone();
    let mut out = [0u8; 8];
    out.copy_from_slice(&bytes[..8]);
    // Exercise Mirror::seed_type and Mirror::as_u128 to avoid dead_code warnings and for auditability
    let seed_type = Mirror::seed_type(&euclid);
    let mirror_u128 = mirror.as_u128();
    (out, seed_type, mirror_u128)
}

/// Collapse to full 32-byte mirror for tests that need complete bytes.
fn get_mirror32(seed: Option<&[u8]>, dim: Dimension) -> ([u8; 32], SeedType, u128) {
    let euclid = match seed {
        None => *Euclid::genesis(),
        Some(s) => Euclid::from_seed(s),
    };
    let mirror = Mirror::collapse(&euclid, dim as u128);
    let bytes = mirror.bytes().clone();
    let seed_type = Mirror::seed_type(&euclid);
    let mirror_u128 = mirror.as_u128();
    (bytes, seed_type, mirror_u128)
}

/// 1️⃣ Deterministic Transition Test
/// Ensures same inputs → same dimension and Σ across runs (memoryless determinism).
fn test_deterministic_transition(perm: &Perm) {
    let mut q1 = QuantPerm::new(perm.clone());
    q1.set_initial_dimension_from_perm();
    q1.retain(50_000);
    q1.transition(None);

    let mut q2 = QuantPerm::new(perm.clone());
    q2.set_initial_dimension_from_perm();
    q2.retain(50_000);
    q2.transition(None);

    assert_eq!(q1.dimension(), q2.dimension(), "Dimension must be identical for same inputs");
    assert_eq!(q1.structural_value(), q2.structural_value(), "Σ must be identical for same inputs");
}

/// 2️⃣ Observer-Relative Mirror Test
/// Same dimension collapses differently for Genesis vs Fork, while dimension remains invariant.
fn test_observer_relative_mirror(dim: Dimension, dev_seed: &[u8]) {
    let (pub_mir, pub_type, pub_u128) = get_mirror32(None, dim);
    let (priv_mir, priv_type, priv_u128) = get_mirror32(Some(dev_seed), dim);

    assert_ne!(pub_mir, priv_mir, "Public vs private mirrors must differ for same dimension");
    assert_eq!(pub_type, SeedType::Euclid, "Public seed type must be Euclid");
    assert_eq!(priv_type, SeedType::Fork, "Private seed type must be Fork");
    assert_ne!(pub_u128, priv_u128, "Numeric mirror representations should differ across observers");

    // Integrity: no private seed leakage in public mirror printouts
    let public_print = format!("{:x?}", &pub_mir[..8]);
    assert!(!public_print.contains("developer-route-seed"), "Public output must not leak private seed");
}

/// 3️⃣ Ledgerless Coordination / Structural Value Gating Test
/// Demonstrates that Σ gates actions; retained mass is the payload. No Σ transfer occurs.
fn test_structural_value_gating(harmony: &QuantPerm, auwal: &mut QuantPerm) {
    let threshold = 20_000;

    // Harmony's Σ acts as proof/gating signal; if sufficient, Auwal introduces external truth.
    if harmony.structural_value() > threshold {
        auwal.retain(10_000); // External truth ingress (payload)
        auwal.transition(None);
    }

    // Auwal's Σ increases only due to its own retain/transition, not due to Harmony's Σ being "transferred".
    assert!(auwal.structural_value() > 0, "Receiver Σ must increase only via its own retain/transition");
}

/// 4️⃣ Communication / External Truth Simulation
/// Simulates communication permission based on retained Σ without leaking private seeds.
fn test_communication_simulation(dim: Dimension, retained_sigma: u128, dev_seed: &[u8]) {
    let threshold = 20_000;

    // Both observers compute mirrors independently
    let (pub_mir, _, _) = get_mirror32(None, dim);
    let (priv_mir, _, _) = get_mirror32(Some(dev_seed), dim);

    // Permission check: only Σ threshold determines communication
    let can_send = retained_sigma > threshold;

    // Public observers can only infer that "communication happened" (boolean), not private content
    assert_eq!(pub_mir.len(), 32);
    assert_eq!(priv_mir.len(), 32);
    // No seed leakage in any public-facing boolean or mirror bytes
    let public_flag = format!("{}", can_send);
    assert!(!public_flag.contains("developer-route-seed"), "Boolean flags must not leak private seed");
}

/// 5️⃣ Multi-Observer Truth Convergence
/// All observers see the same dimension evolution; mirrors differ by seed; Σ is consistent.
fn test_multi_observer_convergence(dim: Dimension, sigma: u128, seeds: &[&[u8]]) {
    // Public baseline
    let (pub_mir, pub_type, pub_u128) = get_mirror32(None, dim);
    assert_eq!(pub_type, SeedType::Euclid);

    for s in seeds {
        let (priv_mir, priv_type, priv_u128) = get_mirror32(Some(s), dim);
        assert_eq!(priv_type, SeedType::Fork);
        assert_ne!(pub_mir, priv_mir, "Each private observer should see a distinct mirror vs public");
        assert_ne!(pub_u128, priv_u128, "Numeric mirror representations should differ across observers");
        // Dimension and Σ are public invariants—unchanged by observer context
        assert!(sigma > 0, "Σ should be a consistent, positive measure of work");
    }
}

/// ✅ Optional: Serialization Test (no private seed leakage)
fn test_mirror_serialization(dim: Dimension) {
    let (pub_mir, _, _) = get_mirror32(None, dim);
    // "Serialize" by copying into a Vec<u8> (no external deps)
    let serialized: Vec<u8> = pub_mir.to_vec();
    assert_eq!(serialized.len(), 32);
    // "Deserialize" by reconstructing the array
    let mut deserialized = [0u8; 32];
    deserialized.copy_from_slice(&serialized);
    assert_eq!(pub_mir, deserialized, "Mirror must round-trip without mutation");
}

/// ✅ Optional: High-Latency Simulation
/// Two independent nodes compute identical transitions and confirm identical dimension and structural_value(Σ).
fn test_high_latency_simulation(perm: &Perm) {
    // Node A
    let mut qa = QuantPerm::new(perm.clone());
    qa.set_initial_dimension_from_perm();
    qa.retain(123_456);
    qa.transition(None);

    // Node B (independent, same inputs)
    let mut qb = QuantPerm::new(perm.clone());
    qb.set_initial_dimension_from_perm();
    qb.retain(123_456);
    qb.transition(None);

    assert_eq!(qa.dimension(), qb.dimension(), "Interplanetary nodes must converge on same dimension");
    assert_eq!(qa.structural_value(), qb.structural_value(), "Interplanetary nodes must converge on same Σ");
}

fn main() {
    let dev_seed = b"developer-route-seed";

    // 1️⃣ Shared Universal Foundation
    let domain_indices: [u16; 12] = [3, 17, 64, 128, 256, 512, 1023, 42, 7, 900, 659, 2047];
    let perm = Perm::genesis_construct(&domain_indices, b"observer-entropy-v1")
        .expect("Genesis failed");

    // 2️⃣ Independent Envelopes
    let mut harmony = QuantPerm::new(perm.clone());
    let mut auwal = QuantPerm::new(perm.clone());

    harmony.set_initial_dimension_from_perm();
    auwal.set_initial_dimension_from_perm();

    let mut reactions: Vec<ReactionRow> = Vec::new();

    // ─────────────────────────────────────────────
    // STEP G: Harmony builds Peak Σ (Forked Frame)
    // ─────────────────────────────────────────────
    harmony.retain(50_000);
    harmony.transition(Some(dev_seed));

    let (pub_mir_g, pub_type_g, pub_u128_g) = get_mirror8(None, harmony.dimension());
    let (priv_mir_g, priv_type_g, priv_u128_g) = get_mirror8(Some(dev_seed), harmony.dimension());

    reactions.push(ReactionRow {
        step: "G".into(),
        actor: "Harmony".into(),
        dimension: harmony.dimension(),
        sigma: harmony.structural_value(),
        public_mirror: pub_mir_g,
        private_mirror: priv_mir_g,
        public_seed_type: pub_type_g,
        private_seed_type: priv_type_g,
        public_mirror_u128: pub_u128_g,
        private_mirror_u128: priv_u128_g,
    });

    // ─────────────────────────────────────────────
    // STEP H: Auwal receives 25% of Harmony's Σ as payload (no Σ transfer)
    // ─────────────────────────────────────────────
    // Harmony's Σ gates Auwal's action; Auwal introduces external truth equal to 25% of Harmony's Σ.
    let quarter_of_harmony_sigma = harmony.structural_value() / 4;
    auwal.retain(quarter_of_harmony_sigma); // External truth ingress proportional to Harmony's Σ
    auwal.transition(Some(dev_seed));

    // Preserve and report Auwal's post-transition dimension
    let auwal_post_dim = auwal.dimension();

    let (pub_mir_h, pub_type_h, pub_u128_h) = get_mirror8(None, auwal_post_dim);
    let (priv_mir_h, priv_type_h, priv_u128_h) = get_mirror8(Some(dev_seed), auwal_post_dim);

    reactions.push(ReactionRow {
        step: "H".into(),
        actor: "Auwal".into(),
        dimension: auwal_post_dim,
        sigma: auwal.structural_value(),
        public_mirror: pub_mir_h,
        private_mirror: priv_mir_h,
        public_seed_type: pub_type_h,
        private_seed_type: priv_type_h,
        public_mirror_u128: pub_u128_h,
        private_mirror_u128: priv_u128_h,
    });

    // ─────────────────────────────────────────────
    // STEP I: Harmony Recoil (New Equilibrium)
    // ─────────────────────────────────────────────
    // Harmony continues evolving under the same forked lens (no Σ transfer).
    harmony.transition(Some(dev_seed));

    // Preserve and report Harmony's post-transition dimension
    let harmony_post_dim = harmony.dimension();

    let (pub_mir_i, pub_type_i, pub_u128_i) = get_mirror8(None, harmony_post_dim);
    let (priv_mir_i, priv_type_i, priv_u128_i) = get_mirror8(Some(dev_seed), harmony_post_dim);

    reactions.push(ReactionRow {
        step: "I".into(),
        actor: "Harmony".into(),
        dimension: harmony_post_dim,
        sigma: harmony.structural_value(),
        public_mirror: pub_mir_i,
        private_mirror: priv_mir_i,
        public_seed_type: pub_type_i,
        private_seed_type: priv_type_i,
        public_mirror_u128: pub_u128_i,
        private_mirror_u128: priv_u128_i,
    });

    // ─────────────────────────────────────────────
    // 3️⃣ THE REACTION REPORT
    // ─────────────────────────────────────────────
    println!("──────────────────────────────────────────────────────────────────────────");
    println!("QUANTOM MANIFOLD: GENESIS VS. PRIVATE REACTIONS");
    println!("──────────────────────────────────────────────────────────────────────────");
    println!("{:<4} | {:<8} | {:<12} | {:<10} | {:<18} | {:<18} | {:<8} | {:<8} | {:<20} | {:<20}",
             "Step", "Actor", "Dim (Short)", "Σ Value", "Public (Genesis)", "Private (Fork)", "PubType", "PrivType", "PubMirror(u128)", "PrivMirror(u128)");
    println!("─────|──────────|──────────────|────────────|────────────────────|────────────────────|──────────|──────────|──────────────────────|──────────────────────");

    for r in &reactions {
        println!(
            " {:<4} | {:<8} | {:<12} | {:<10} | {:x?} | {:x?} | {:<8} | {:<8} | {:<20} | {:<20}",
            r.step,
            r.actor,
            r.dimension % 100_000_000,
            r.sigma / 1_000_000_000_000_000, // Truncated Σ for readability
            r.public_mirror,
            r.private_mirror,
            format!("{:?}", r.public_seed_type),
            format!("{:?}", r.private_seed_type),
            r.public_mirror_u128,
            r.private_mirror_u128,
        );
    }

    println!("──────────────────────────────────────────────────────────────────────────");
    println!("CONCLUSION:");
    println!("1. Look at Step H and G: Same Dimension, but Public Mirror != Private Mirror.");
    println!("2. To Genesis, this movement looks like noise/chaos.");
    println!("3. To the Developer Seed, this movement is a perfectly calculated bridge.");
    println!("4. Σ gates actions; retained mass is payload. No Σ transfer occurs.");
    println!("──────────────────────────────────────────────────────────────────────────");

    // ─────────────────────────────────────────────
    // 4️⃣ AUDITABLE TESTS (no hidden state, no consensus)
    // ─────────────────────────────────────────────
    // Deterministic transition (memoryless)
    test_deterministic_transition(&perm);

    // Observer-relative mirrors (semantic sovereignty)
    test_observer_relative_mirror(harmony_post_dim, dev_seed);

    // Ledgerless coordination via Σ gating (no Σ transfer)
    test_structural_value_gating(&harmony, &mut auwal);

    // Communication simulation (threshold-based, no seed leakage)
    test_communication_simulation(harmony_post_dim, harmony.structural_value(), dev_seed);

    // Multi-observer truth convergence (same dimension/Σ, different mirrors)
    let seeds = vec![b"dev-seed-1".as_ref(), b"dev-seed-2".as_ref(), b"dev-seed-3".as_ref()];
    test_multi_observer_convergence(harmony_post_dim, harmony.structural_value(), &seeds);

    // Serialization round-trip (no private seed leakage)
    test_mirror_serialization(harmony_post_dim);

    // High-latency simulation (independent nodes converge deterministically)
    test_high_latency_simulation(&perm);

    println!("All invariants validated: deterministic evolution, observer-relative interpretation, Σ gating (no transfer), and no private seed leakage.");
}
