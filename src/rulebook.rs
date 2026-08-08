//! The versioned rulebook — the constants a fitting is solved AGAINST,
//! gathered in one hashable struct. This is the subscription mechanism:
//! prescription science revises (NAL-NL2 → NL3, DSL updates, a changed
//! guard-band policy), the rulebook version bumps, every deployed table's
//! provenance hash goes stale, and the fail-closed loader refuses it
//! until the fit is re-solved. Nothing relies on anyone remembering to
//! re-fit — staleness is mechanical.

/// All fitting-rule constants, versioned. Copy so patients can carry it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rulebook {
    pub version: u32,
    /// Comfort guard below UCL (dB).
    pub ucl_guard_db: f64,
    /// Feedback-margin gain caps by band tercile (low/mid/high), dB.
    pub caps: [f64; 3],
    /// WDRC linkage: soft-input boost / loud-input cut around g65 (dB).
    pub soft_boost: f64,
    pub loud_cut: f64,
    /// Fraction of UCL loudness headroom the amplification may add.
    pub budget_fraction: f64,
}

/// Today's rulebook (the constants the A-phase benchmark ran under).
pub const RULEBOOK_V1: Rulebook = Rulebook {
    version: 1,
    ucl_guard_db: 3.0,
    caps: [45.0, 38.0, 30.0],
    soft_boost: 6.0,
    loud_cut: 6.0,
    budget_fraction: 0.5,
};

/// A plausible revision: widened comfort guard (3→5 dB) and tightened
/// mid/high feedback caps — the kind of change a prescription-science
/// update or a post-market safety notice would ship.
pub const RULEBOOK_V2: Rulebook = Rulebook {
    version: 2,
    ucl_guard_db: 5.0,
    caps: [45.0, 34.0, 26.0],
    soft_boost: 6.0,
    loud_cut: 6.0,
    budget_fraction: 0.5,
};

impl Rulebook {
    pub fn feedback_cap(&self, band: usize) -> f64 {
        if band < 6 {
            self.caps[0]
        } else if band < 12 {
            self.caps[1]
        } else {
            self.caps[2]
        }
    }

    /// Linked (soft, avg, loud) gains from the average-level gain.
    pub fn linked_gains(&self, g65: f64, band: usize) -> [f64; 3] {
        let cap = self.feedback_cap(band);
        [
            (g65 + self.soft_boost).min(cap),
            g65,
            (g65 - self.loud_cut).max(0.0),
        ]
    }

    /// Order-sensitive hash of every constant — feeds the provenance
    /// image so a table solved under one rulebook cannot validate under
    /// another.
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0x9E37_79B9_7F4A_7C15 ^ self.version as u64;
        for f in [
            self.ucl_guard_db,
            self.caps[0],
            self.caps[1],
            self.caps[2],
            self.soft_boost,
            self.loud_cut,
            self.budget_fraction,
        ] {
            h = h.rotate_left(7) ^ f.to_bits();
            h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rulebook_hashes_are_version_and_content_sensitive() {
        assert_ne!(RULEBOOK_V1.hash(), RULEBOOK_V2.hash());
        // Same version number with a silently edited constant still
        // changes the hash — content is bound, not just the label.
        let mut tampered = RULEBOOK_V1;
        tampered.ucl_guard_db = 4.0;
        assert_ne!(tampered.hash(), RULEBOOK_V1.hash());
    }
}
