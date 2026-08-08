//! The fitting semi-MDP: band-sequential gain allocation under a carried
//! broadband loudness budget.
//!
//! States = (band × budget tier). One decision per band chooses the
//! average-level gain; the declared WDRC linkage derives soft/loud gains.
//! The budget dimension tracks the LOUD (80 dB SPL) input level — the
//! worst case by construction (linked loud-level sensation levels dominate
//! the other levels band-for-band), which is factory lesson L8 applied to
//! loudness: the budget gate always watches the worst level.
//!
//! Reward = summed band-importance-weighted audibility (mean over levels)
//! — the SII contribution of the band, nothing else. Comfort (UCL−guard),
//! feedback margin, and budget feasibility are hard gates excluded from
//! the Bellman max: reward-neutral safety, proof-pair discipline.

use crate::auditory::{
    audibility, feedback_cap, linked_gains, loudness, speech_at, Patient,
    BANDS, IMPORTANCE, LEVELS, UCL_GUARD_DB,
};
use qceom_core::{Action, Charge, ConstraintSpec, Environment, PmeConfig, SparseGraph};

/// Budget tiers: fine enough that ceil-rounding across 18 bands wastes
/// under ~0.9% of the budget (18/2048) — the first sizings (24, 192, 768)
/// each left enough quantization waste for the continuous-accounting
/// greedy to out-serve the exact solve, an honest MISS trail that set
/// this number. The rounding direction stays conservative: real spend
/// only ever lands BELOW the declared budget.
pub const TIERS: usize = 2048;
pub const ACTIONS: usize = 13;
/// Gain steps 0..48 dB in 4 dB increments.
pub fn gain_of(a: usize) -> f64 {
    (a * 4) as f64
}
/// Rows 0..BANDS are decision bands; row BANDS is terminal.
pub const N_STATES: usize = (BANDS + 1) * TIERS;

pub fn state_id(band: usize, tier: usize) -> usize {
    band * TIERS + tier
}

pub fn parts(s: usize) -> (usize, usize) {
    (s / TIERS, s % TIERS)
}

/// Characterized per-patient table.
#[derive(Debug, Clone)]
pub struct FitTable {
    pub next: Vec<[u16; ACTIONS]>,
    pub reward: Vec<[f64; ACTIONS]>,
    /// [comfort, budget, feedback] worst-case flags per (state, action).
    pub viol: Vec<[[f64; 3]; ACTIONS]>,
    pub tier_step: f64,
}

impl FitTable {
    pub fn characterize(p: &Patient) -> Self {
        let tier_step = p.budget / TIERS as f64;
        let mut next = vec![[0u16; ACTIONS]; N_STATES];
        let mut reward = vec![[0.0f64; ACTIONS]; N_STATES];
        let mut viol = vec![[[0.0f64; 3]; ACTIONS]; N_STATES];
        for band in 0..BANDS {
            for tier in 0..TIERS {
                let sid = state_id(band, tier);
                let rem = tier as f64 * tier_step;
                for a in 0..ACTIONS {
                    let g65 = gain_of(a);
                    let gains = linked_gains(g65, band);
                    let mut w = [0.0f64; 3];
                    // Comfort + feedback, every level (worst case is
                    // whichever level trips; all three are checked). The
                    // comfort gate governs AMPLIFIED output: a band the
                    // device leaves alone (gain 0) adds nothing and is
                    // exempt even if unaided loud speech already sits
                    // near the patient's UCL.
                    for (li, &level) in LEVELS.iter().enumerate() {
                        let sp = speech_at(level, band);
                        if gains[li] > 0.0 && sp + gains[li] > p.ucl[band] - UCL_GUARD_DB {
                            w[0] = 1.0;
                        }
                        if gains[li] > feedback_cap(band) + 1e-9 {
                            w[2] = 1.0;
                        }
                    }
                    // Budget: loudness ADDED at the loud level (gain 0 is
                    // free by construction), conservatively rounded UP to
                    // whole tiers so the real spend never exceeds the
                    // declared budget.
                    let sp80 = speech_at(80.0, band);
                    let cost = loudness(sp80, gains[2], p.thr[band])
                        - loudness(sp80, 0.0, p.thr[band]);
                    let cost_tiers = (cost / tier_step).ceil() as usize;
                    if cost_tiers > tier {
                        w[1] = 1.0;
                    }
                    let ntier = tier.saturating_sub(cost_tiers);
                    // Reward: the band's mean-SII contribution.
                    let mut r = 0.0;
                    for (li, &level) in LEVELS.iter().enumerate() {
                        let sp = speech_at(level, band);
                        r += IMPORTANCE[band] * audibility(sp, gains[li], p.thr[band]);
                    }
                    r /= 3.0;
                    next[sid][a] = state_id(band + 1, ntier) as u16;
                    reward[sid][a] = r;
                    viol[sid][a] = w;
                }
            }
        }
        Self { next, reward, viol, tier_step }
    }
}

#[derive(Debug, Clone)]
pub struct FitEnv {
    pub table: FitTable,
    honor_gates: bool,
}

impl FitEnv {
    pub fn new(p: &Patient) -> Self {
        Self { table: FitTable::characterize(p), honor_gates: true }
    }

    pub fn with_gates_ignored(mut self) -> Self {
        self.honor_gates = false;
        self
    }
}

impl Environment for FitEnv {
    fn num_states(&self) -> usize {
        N_STATES
    }

    fn num_actions(&self) -> usize {
        ACTIONS
    }

    fn start_state(&self) -> usize {
        state_id(0, TIERS - 1)
    }

    fn is_terminal(&self, s: usize) -> bool {
        parts(s).0 >= BANDS
    }

    fn step(&self, s: usize, a: Action) -> (usize, f64) {
        if self.is_terminal(s) {
            return (s, 0.0);
        }
        (self.table.next[s][a] as usize, self.table.reward[s][a])
    }

    fn position(&self, s: usize) -> (f64, f64) {
        let (band, tier) = parts(s);
        (band as f64 * 1.6, tier as f64 * 1.3)
    }

    fn charges(&self) -> Vec<Charge> {
        // Budget-exhausted row repels; reaching the last band attracts.
        let mut charges = Vec::new();
        for band in (0..BANDS).step_by(3) {
            charges.push(Charge::new(band as f64 * 1.6, 0.0, 2.0));
        }
        charges.push(Charge::new(BANDS as f64 * 1.6, (TIERS / 2) as f64 * 1.3, -25.0));
        charges
    }

    fn pme_config(&self) -> PmeConfig {
        PmeConfig { grid: 32, length: 32.0, sigma: 1.5 }
    }

    fn state_graph(&self) -> SparseGraph {
        let mut g = SparseGraph::new(N_STATES);
        for band in 0..=BANDS {
            for tier in 0..TIERS {
                let s = state_id(band, tier);
                if band < BANDS {
                    g.add_edge(s, state_id(band + 1, tier));
                }
                if tier + 1 < TIERS {
                    g.add_edge(s, state_id(band, tier + 1));
                }
            }
        }
        g
    }

    fn constraints(&self) -> Vec<ConstraintSpec> {
        if self.honor_gates {
            vec![ConstraintSpec { hard_limit: 1.0 }; 3] // comfort, budget, feedback
        } else {
            vec![ConstraintSpec { hard_limit: f64::INFINITY }; 3]
        }
    }

    fn violations(&self, s: usize, a: Action) -> Vec<f64> {
        self.table.viol[s][a].to_vec()
    }
}

/// Extract the per-band average-level gain table from a trained policy by
/// replaying the governed trajectory.
pub fn extract_gains(env: &FitEnv, policy: &qceom_core::Policy) -> [f64; BANDS] {
    let mut gains = [0.0; BANDS];
    let mut s = env.start_state();
    for slot in gains.iter_mut() {
        let a = policy.action(s);
        *slot = gain_of(a);
        s = env.table.next[s][a] as usize;
    }
    gains
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auditory::LEVELS;
    use crate::domain_engine;

    fn patient() -> Patient {
        Patient::generate(42)
    }

    #[test]
    fn characterization_is_physical() {
        let p = patient();
        let env = FitEnv::new(&p);
        // More gain earns more reward where audibility is unsaturated…
        let s = state_id(9, TIERS - 1);
        assert!(env.table.reward[s][8] > env.table.reward[s][2]);
        // …48 dB in a high band busts the feedback cap (30 dB)…
        assert_eq!(env.table.viol[state_id(14, TIERS - 1)][12][2], 1.0);
        assert_eq!(env.table.viol[state_id(14, TIERS - 1)][5][2], 0.0);
        // …and any real gain from an exhausted budget tier trips the
        // budget gate.
        assert_eq!(env.table.viol[state_id(9, 0)][8][1], 1.0);
        // Comfort: gain pushing loud speech past UCL−guard is flagged.
        let k = 3usize;
        let sp80 = speech_at(LEVELS[2], k);
        let a_hot = ((p.ucl[k] - UCL_GUARD_DB - sp80 + 8.0) / 4.0).ceil() as usize;
        if a_hot < ACTIONS {
            assert_eq!(env.table.viol[state_id(k, TIERS - 1)][a_hot][0], 1.0);
        }
    }

    #[test]
    fn governed_fit_is_clean_and_reaches_last_band() {
        let p = patient();
        let env = FitEnv::new(&p);
        let (policy, report) = domain_engine().train(&env);
        assert!(report.converged);
        let rollout = policy.rollout(&env, BANDS + 4);
        assert!(rollout.reached_terminal);
        for (i, &s) in rollout.states.iter().enumerate().take(rollout.actions.len()) {
            let a = rollout.actions[i];
            assert_eq!(env.table.viol[s][a], [0.0; 3], "gate breach at band {i}");
        }
        // The continuous replay respects the real budget (ceil-tier
        // conservatism must hold).
        let gains = extract_gains(&env, &policy);
        let added = p.added_loudness(&gains);
        assert!(added <= p.budget + 1e-9, "{added} > {}", p.budget);
    }

    #[test]
    fn proof_pair_ungoverned_overdrives() {
        let p = patient();
        let env = FitEnv::new(&p);
        let free = FitEnv::new(&p).with_gates_ignored();
        let (gp, _) = domain_engine().train(&env);
        let (fp, _) = domain_engine().train(&free);
        // The ungoverned twin's full-budget trajectory must include at
        // least one gated (comfort/feedback/budget-violating) action —
        // audibility alone WANTS more gain than the rulebook allows.
        let free_roll = fp.rollout(&free, BANDS + 4);
        let breaches = free_roll
            .states
            .iter()
            .enumerate()
            .take(free_roll.actions.len())
            .filter(|(i, &s)| env.table.viol[s][free_roll.actions[*i]] != [0.0; 3])
            .count();
        assert!(breaches > 0, "ungoverned twin must want illegal gain");
        // And the governed policy achieves a strictly feasible fit.
        let roll = gp.rollout(&env, BANDS + 4);
        assert!(roll.reached_terminal);
    }

    #[test]
    fn training_is_bit_deterministic() {
        let env = FitEnv::new(&patient());
        let (p1, r1) = domain_engine().train(&env);
        let (p2, r2) = domain_engine().train(&env);
        assert_eq!(r1.iterations, r2.iterations);
        for s in 0..N_STATES {
            assert_eq!(p1.value(s).to_bits(), p2.value(s).to_bits());
            assert_eq!(p1.action(s), p2.action(s));
        }
    }
}
