//! The incumbents.
//!
//! 1. A prescriptive half-gain-style formula reimplemented from published
//!    descriptions — **research benchmark only**: this is NOT the licensed
//!    NAL or DSL software and makes no target-match claim to either. The
//!    shape (gain ≈ fraction of hearing loss with high-frequency emphasis,
//!    clipped to comfort/feedback, then uniformly scaled to the loudness
//!    ceiling) is what generic fitting software does.
//! 2. The per-band-order greedy optimizer — the "obvious algorithm" the
//!    A1 falsifier already showed loses to exact DP when the budget binds.

use crate::auditory::{loudness, speech_at, Patient, BANDS};
use crate::fit_env::{gain_of, ACTIONS};

/// Snap a gain down to the harness's 4 dB action lattice (both incumbents
/// live on the same discrete lattice as the policy — no resolution edge).
fn snap(g: f64) -> f64 {
    ((g / 4.0).floor() * 4.0).clamp(0.0, 48.0)
}

/// Clip a candidate average-level gain to the per-band hard rules
/// (comfort at every level, feedback cap).
fn clip_band(p: &Patient, k: usize, g65: f64) -> f64 {
    // Snap AFTER the cap: the caps (45/38/30) are off-lattice, and an
    // un-snapped cap would hand the incumbents 1-2 dB per band the DP's
    // action set cannot express (a measured unfairness caught by the
    // plain-DP probe).
    let mut g = snap(g65.min(p.rulebook.feedback_cap(k)));
    loop {
        let gains = p.rulebook.linked_gains(g, k);
        let mut ok = true;
        for (li, &level) in crate::auditory::LEVELS.iter().enumerate() {
            if speech_at(level, k) + gains[li] > p.ucl[k] - p.rulebook.ucl_guard_db {
                ok = false;
            }
        }
        if ok || g <= 0.0 {
            break;
        }
        g -= 4.0;
    }
    g.max(0.0)
}

/// Prescriptive incumbent: half-gain + high-frequency emphasis, clipped,
/// then uniformly reduced until the loud-level broadband loudness fits
/// the budget (the "volume-limiter" every fitting stack applies).
pub fn prescriptive(p: &Patient) -> [f64; BANDS] {
    let mut g = [0.0f64; BANDS];
    for k in 0..BANDS {
        let hl = (p.thr[k] - 20.0).max(0.0);
        let emphasis = 1.0 + 0.3 * (k as f64 / (BANDS - 1) as f64);
        g[k] = clip_band(p, k, snap(0.5 * hl * emphasis));
    }
    loop {
        if p.added_loudness(&g) <= p.budget || g.iter().all(|&x| x <= 0.0) {
            break;
        }
        for x in g.iter_mut() {
            *x = (*x - 4.0).max(0.0);
        }
    }
    g
}

/// Greedy band-order: maximize each band's audibility in turn, spending
/// the loud-level loudness budget as it goes (reserving nothing).
pub fn greedy(p: &Patient) -> [f64; BANDS] {
    let mut g = [0.0f64; BANDS];
    let mut spent = 0.0;
    for k in 0..BANDS {
        let mut best = 0.0f64;
        let mut best_a = -1.0f64;
        for ai in 0..ACTIONS {
            let cand = clip_band(p, k, gain_of(ai));
            let gains = p.rulebook.linked_gains(cand, k);
            let sp80 = speech_at(80.0, k);
            let cost =
                loudness(sp80, gains[2], p.thr[k]) - loudness(sp80, 0.0, p.thr[k]);
            if spent + cost > p.budget {
                continue;
            }
            let a65 = crate::auditory::audibility(
                speech_at(65.0, k),
                gains[1],
                p.thr[k],
            );
            if a65 > best_a {
                best_a = a65;
                best = cand;
            }
        }
        let gains = p.rulebook.linked_gains(best, k);
        let sp80 = speech_at(80.0, k);
        spent += loudness(sp80, gains[2], p.thr[k]) - loudness(sp80, 0.0, p.thr[k]);
        g[k] = best;
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incumbents_respect_the_rulebook() {
        for seed in [7u64, 42, 99] {
            let p = Patient::generate(seed, crate::rulebook::RULEBOOK_V1);
            for g in [prescriptive(&p), greedy(&p)] {
                assert!(p.added_loudness(&g) <= p.budget + 1e-9);
                for k in 0..BANDS {
                    assert!(g[k] <= p.rulebook.feedback_cap(k) + 1e-9);
                }
            }
        }
    }
}
