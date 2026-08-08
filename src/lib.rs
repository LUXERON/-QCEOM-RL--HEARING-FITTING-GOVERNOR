//! [QCEOM RL] Hearing-aid fitting governor.
//!
//! Exact-DP per-patient gain tables maximizing the ANSI S3.5 Speech
//! Intelligibility Index under hard comfort/loudness/feedback gates,
//! benchmarked against prescriptive and greedy incumbents on a seeded
//! patient corpus. Licensing posture (PATENT-LICENSING.md): no licensed
//! prescription software (NAL/DSL) is shipped or claimed; the incumbent
//! is a published-literature approximation, research-labeled. Deployable
//! artifact: a provenance-hashed static table (image.rs) — the 21st
//! Century Cures §3060 "independently reviewable basis" posture.

pub mod auditory;
pub mod fit_env;
pub mod image;
pub mod prescriptive;
pub mod rulebook;

use qceom_core::{EngineConfig, MathematicalRLEngine};

/// Kernel defaults with two DECLARED domain deviations, both set by the
/// plain-DP probe (the domain's ground-truth optimum on the quantized
/// model):
///
/// - γ = 0.9999 (L3): the fit is a band-sequential DAG of 18 decisions;
///   γ=0.999 measurably shaded late (high-frequency) bands (~3% SII),
///   and γ=1.0 collapses the kernel (its machinery assumes contraction —
///   an honest MISS kept in the log). 0.9999¹⁸ = 0.9982 is measured
///   indistinguishable from the undiscounted reference.
/// - shaping_weight = 0.0: this domain's per-step rewards are tiny
///   (band importance × audibility ≈ 3e-3), and the default PME shaping
///   weight of 2.0 — policy-invariant in exact arithmetic — cost a
///   measured 3.2% SII through the kernel's finite-tolerance machinery.
///   With shaping off the kernel reproduces the plain-DP optimum
///   EXACTLY. Shaping weight is a domain unit, not a universal constant
///   (new factory lesson).
pub fn domain_engine() -> MathematicalRLEngine {
    MathematicalRLEngine::new(EngineConfig {
        gamma: 0.9999,
        shaping_weight: 0.0,
        ..EngineConfig::default()
    })
}
