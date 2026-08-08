//! The declared auditory model. Every constant is from open literature:
//! ANSI S3.5 third-octave band-importance weights, the SII 30-dB speech
//! dynamic range, an LTASS-shaped speech spectrum, and a compressive
//! (Stevens-style, exponent 0.6) loudness proxy for the broadband
//! discomfort budget. This is the model the policy is EXACTLY optimal on;
//! clinical validation of the model itself is a roadmap item, not a claim.

pub const BANDS: usize = 18;

/// ANSI S3.5 Table 3 one-third-octave band importance (160 Hz .. 8 kHz).
pub const IMPORTANCE: [f64; BANDS] = [
    0.0083, 0.0095, 0.0150, 0.0289, 0.0440, 0.0578, 0.0653, 0.0711, 0.0818,
    0.0844, 0.0882, 0.0898, 0.0868, 0.0844, 0.0771, 0.0527, 0.0364, 0.0185,
];

/// LTASS-shaped band levels (dB SPL) for 65 dB SPL overall speech.
pub const SPEECH65: [f64; BANDS] = [
    32.5, 34.8, 34.4, 34.7, 33.1, 31.5, 30.6, 30.0, 28.6, 27.6, 26.4, 25.0,
    23.4, 22.2, 20.8, 18.9, 17.6, 16.5,
];

/// Speech input levels evaluated (dB SPL overall): soft / average / loud.
pub const LEVELS: [f64; 3] = [50.0, 65.0, 80.0];

/// Per-band feedback-margin gain caps (dB): open-loop-gain headroom
/// shrinks toward high frequencies (declared piecewise curve).
pub fn feedback_cap(band: usize) -> f64 {
    if band < 6 {
        45.0
    } else if band < 12 {
        38.0
    } else {
        30.0
    }
}

/// Comfort guard band below UCL (dB), declared (same discipline as the
/// fast-charge 15 mV plating guard).
pub const UCL_GUARD_DB: f64 = 3.0;

/// Declared WDRC level linkage: soft inputs get +6 dB over the average-
/// level gain, loud inputs −6 dB (a fixed, declared compression rule —
/// the harness optimizes the average-level gain and the linkage follows).
pub const WDRC_SOFT_BOOST: f64 = 6.0;
pub const WDRC_LOUD_CUT: f64 = 6.0;

/// Band speech level at a given overall input level.
pub fn speech_at(level: f64, band: usize) -> f64 {
    SPEECH65[band] + (level - 65.0)
}

/// SII band audibility: (sensation level)/30 clipped to [0,1].
pub fn audibility(speech_spl: f64, gain_db: f64, thr_spl: f64) -> f64 {
    ((speech_spl + gain_db - thr_spl) / 30.0).clamp(0.0, 1.0)
}

/// Compressive loudness proxy for the broadband budget.
pub fn loudness(speech_spl: f64, gain_db: f64, thr_spl: f64) -> f64 {
    let sl = (speech_spl + gain_db - thr_spl).max(0.0);
    sl.powf(0.6)
}

/// Linked gains (soft, avg, loud) from the average-level gain.
pub fn linked_gains(g65: f64, band: usize) -> [f64; 3] {
    let cap = feedback_cap(band);
    [
        (g65 + WDRC_SOFT_BOOST).min(cap),
        g65,
        (g65 - WDRC_LOUD_CUT).max(0.0),
    ]
}

/// A patient in the declared model: per-band thresholds and UCLs (dB SPL)
/// plus the derived broadband loudness budget.
#[derive(Debug, Clone)]
pub struct Patient {
    pub thr: [f64; BANDS],
    pub ucl: [f64; BANDS],
    /// Broadband loudness ceiling (declared: 0.6 × Σ loudness-at-UCL).
    pub budget: f64,
    pub seed: u64,
}

fn splitmix(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Patient {
    /// Seeded deterministic generator: flat / sloping / ski-slope /
    /// notched audiograms with plausible UCL spreads.
    pub fn generate(seed: u64) -> Self {
        let mut s = seed;
        let start = 20.0 + (splitmix(&mut s) % 21) as f64;
        let slope = (splitmix(&mut s) % 5) as f64 + 1.0;
        let shape = splitmix(&mut s) % 3;
        let mut thr = [0.0; BANDS];
        for (k, t) in thr.iter_mut().enumerate() {
            *t = match shape {
                0 => start + slope * k as f64,                    // sloping
                1 => start + 10.0 + 1.5 * k as f64,               // flat-ish
                _ => start + (k as f64 / 17.0).powi(2) * 55.0,    // ski-slope
            };
            *t = t.min(90.0);
        }
        if splitmix(&mut s) % 3 == 0 {
            let notch = 6 + (splitmix(&mut s) % 8) as usize;
            thr[notch] = (thr[notch] + 25.0).min(95.0);
        }
        let mut ucl = [0.0; BANDS];
        for k in 0..BANDS {
            let spread = 25.0 + (splitmix(&mut s) % 16) as f64; // 25..40 dB
            ucl[k] = (thr[k] + spread).min(105.0);
        }
        // Declared broadband discomfort ceiling on ADDED loudness: the
        // amplification may add at most half of the headroom between the
        // unaided loud-speech loudness and every band sitting at UCL.
        // Added (not total) loudness is the right currency — gain 0 must
        // always be feasible, and loudness summation makes broadband
        // discomfort arrive well before any single band reaches its own
        // limit (the coupling this harness exists to manage).
        let headroom: f64 = (0..BANDS)
            .map(|k| loudness(ucl[k], 0.0, thr[k]) - loudness(speech_at(80.0, k), 0.0, thr[k]))
            .sum();
        let budget = 0.5 * headroom;
        Self { thr, ucl, budget, seed }
    }

    /// SII at one input level for a given per-band average-level gain set
    /// (linkage applied), plus that level's broadband loudness.
    pub fn score_level(&self, level_idx: usize, g65: &[f64; BANDS]) -> (f64, f64) {
        let level = LEVELS[level_idx];
        let mut sii = 0.0;
        let mut loud = 0.0;
        for k in 0..BANDS {
            let g = linked_gains(g65[k], k)[level_idx];
            let sp = speech_at(level, k);
            sii += IMPORTANCE[k] * audibility(sp, g, self.thr[k]);
            loud += loudness(sp, g, self.thr[k]);
        }
        (sii, loud)
    }

    /// Mean SII across the three levels.
    pub fn mean_sii(&self, g65: &[f64; BANDS]) -> f64 {
        (0..3).map(|l| self.score_level(l, g65).0).sum::<f64>() / 3.0
    }

    /// Loudness ADDED by amplification at the loud (worst) level — the
    /// quantity the broadband budget governs.
    pub fn added_loudness(&self, g65: &[f64; BANDS]) -> f64 {
        (0..BANDS)
            .map(|k| {
                let sp = speech_at(80.0, k);
                let g = linked_gains(g65[k], k)[2];
                loudness(sp, g, self.thr[k]) - loudness(sp, 0.0, self.thr[k])
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn importance_weights_sum_to_one() {
        let sum: f64 = IMPORTANCE.iter().sum();
        assert!((sum - 1.0).abs() < 0.005, "ANSI S3.5 weights sum {sum}");
    }

    #[test]
    fn audibility_is_monotone_and_saturates() {
        assert_eq!(audibility(30.0, 0.0, 60.0), 0.0);
        let a1 = audibility(30.0, 20.0, 40.0);
        let a2 = audibility(30.0, 30.0, 40.0);
        assert!(a2 > a1 && a1 > 0.0);
        assert_eq!(audibility(30.0, 60.0, 40.0), 1.0);
    }

    #[test]
    fn patients_are_deterministic_and_plausible() {
        let a = Patient::generate(42);
        let b = Patient::generate(42);
        assert_eq!(a.thr, b.thr);
        assert_eq!(a.ucl, b.ucl);
        for k in 0..BANDS {
            assert!(a.ucl[k] > a.thr[k] + 20.0);
            assert!(a.thr[k] >= 20.0 && a.thr[k] <= 95.0);
        }
        assert!(a.budget > 0.0);
        let c = Patient::generate(43);
        assert_ne!(a.thr, c.thr);
    }

    #[test]
    fn loudness_budget_binds_across_the_corpus() {
        // At maximum feedback-capped gain the loud-level broadband
        // loudness must exceed the budget for most patients — the
        // coupling the A1 falsifier proved must exist in the Rust model
        // too, corpus-wide rather than for one lucky seed.
        let mut binds = 0;
        let n = 40;
        for seed in 1..=n as u64 {
            let p = Patient::generate(seed * 6151);
            let g65: [f64; BANDS] = core::array::from_fn(feedback_cap);
            if p.added_loudness(&g65) > p.budget {
                binds += 1;
            }
        }
        assert!(binds * 10 >= n * 6, "budget binds on only {binds}/{n}");
    }
}
