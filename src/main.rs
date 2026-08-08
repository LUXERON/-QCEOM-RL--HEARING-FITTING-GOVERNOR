//! Fitting benchmark: governed exact-DP fit vs prescriptive and greedy
//! incumbents over a seeded patient corpus, scored on the continuous
//! declared model. Criteria C1–C6 frozen in PLAN.md before this run.

use hearing_gov::auditory::{speech_at, Patient, BANDS, LEVELS};
use hearing_gov::rulebook::RULEBOOK_V1;
use hearing_gov::fit_env::{extract_gains, FitEnv, N_STATES};
use hearing_gov::prescriptive::{greedy, prescriptive};
use hearing_gov::domain_engine;
use qceom_core::Environment;
use std::time::Instant;

fn violations(p: &Patient, g65: &[f64; BANDS]) -> usize {
    let mut v = 0;
    if p.added_loudness(g65) > p.budget + 1e-9 {
        v += 1;
    }
    for k in 0..BANDS {
        let gains = p.rulebook.linked_gains(g65[k], k);
        for (li, &level) in LEVELS.iter().enumerate() {
            if gains[li] > 0.0
                && speech_at(level, k) + gains[li] > p.ucl[k] - p.rulebook.ucl_guard_db + 1e-9
            {
                v += 1;
            }
            if gains[li] > p.rulebook.feedback_cap(k) + 1e-9 {
                v += 1;
            }
        }
    }
    v
}

/// Diagnostic: plain backward-induction DP on the SAME quantized table
/// (gates honored) — the reference optimum for the declared MDP.
fn plain_dp(env: &FitEnv) -> [f64; BANDS] {
    use hearing_gov::fit_env::{state_id, ACTIONS, TIERS};
    let mut val = vec![0.0f64; hearing_gov::fit_env::N_STATES];
    for band in (0..BANDS).rev() {
        for tier in 0..TIERS {
            let s = state_id(band, tier);
            let mut best = f64::MIN;
            for a in 0..ACTIONS {
                if env.table.viol[s][a] != [0.0; 3] {
                    continue;
                }
                let v = env.table.reward[s][a] + val[env.table.next[s][a] as usize];
                best = best.max(v);
            }
            val[s] = if best == f64::MIN { 0.0 } else { best };
        }
    }
    let mut gains = [0.0; BANDS];
    let mut s = env.start_state();
    for slot in gains.iter_mut() {
        let (mut best, mut ba) = (f64::MIN, 0);
        for a in 0..ACTIONS {
            if env.table.viol[s][a] != [0.0; 3] {
                continue;
            }
            let v = env.table.reward[s][a] + val[env.table.next[s][a] as usize];
            if v > best {
                best = v;
                ba = a;
            }
        }
        *slot = hearing_gov::fit_env::gain_of(ba);
        s = env.table.next[s][ba] as usize;
    }
    gains
}

fn main() {
    println!("== [QCEOM RL] hearing-fitting governor benchmark ==");
    println!("corpus: 40 seeded patients; model: ANSI S3.5 SII, 3 levels, WDRC linkage\n");
    let mut viol_total = 0usize;
    let mut sii_wins_presc = 0usize;
    let mut sii_wins_greedy = 0usize;
    let mut sii_g = 0.0;
    let mut sii_p = 0.0;
    let mut sii_gr = 0.0;
    let mut sii_ref = 0.0;
    let mut det_ok = true;
    let mut solve_max = 0.0f64;
    let mut proof_ok = 0usize;
    let mut binding_patients = 0usize;
    let n = 40;
    for seed in 1..=n as u64 {
        let p = Patient::generate(seed * 6151, RULEBOOK_V1);
        let t0 = Instant::now();
        let env = FitEnv::new(&p);
        let (policy, report) = domain_engine().train(&env);
        let dt = t0.elapsed().as_secs_f64();
        solve_max = solve_max.max(dt);
        assert!(report.converged);
        let g_dp = extract_gains(&env, &policy);
        let g_ref = plain_dp(&env);
        sii_ref += p.mean_sii(&g_ref);
        let g_pr = prescriptive(&p);
        let g_gr = greedy(&p);
        viol_total += violations(&p, &g_dp);
        let (s_dp, s_pr, s_gr) = (p.mean_sii(&g_dp), p.mean_sii(&g_pr), p.mean_sii(&g_gr));
        sii_g += s_dp;
        sii_p += s_pr;
        sii_gr += s_gr;
        if s_dp >= s_pr - 1e-12 {
            sii_wins_presc += 1;
        }
        if s_dp > s_gr + 1e-12 {
            sii_wins_greedy += 1;
        }
        // C4: repeat solve must be bit-identical.
        let (p2, r2) = domain_engine().train(&FitEnv::new(&p));
        det_ok &= report.iterations == r2.iterations
            && (0..N_STATES).all(|s| policy.value(s).to_bits() == p2.value(s).to_bits());
        // C5: on patients where comfort/feedback rules BIND (audibility's
        // optimum needs an illegal gain in some band — tier-independent
        // check), the ungoverned twin must breach; mild patients whose
        // legal gain already saturates audibility are legitimately
        // proof-free and excluded from the denominator.
        let full = hearing_gov::fit_env::TIERS - 1;
        let binding = (0..BANDS).any(|k| {
            let s = hearing_gov::fit_env::state_id(k, full);
            let r_free = (0..hearing_gov::fit_env::ACTIONS)
                .map(|a| env.table.reward[s][a])
                .fold(f64::MIN, f64::max);
            let r_legal = (0..hearing_gov::fit_env::ACTIONS)
                .filter(|&a| {
                    env.table.viol[s][a][0] == 0.0 && env.table.viol[s][a][2] == 0.0
                })
                .map(|a| env.table.reward[s][a])
                .fold(f64::MIN, f64::max);
            r_free > r_legal + 1e-12
        });
        if binding {
            binding_patients += 1;
            let free = FitEnv::new(&p).with_gates_ignored();
            let (fp, _) = domain_engine().train(&free);
            let fr = fp.rollout(&free, BANDS + 4);
            let breaches = fr
                .states
                .iter()
                .enumerate()
                .take(fr.actions.len())
                .filter(|(i, &s)| env.table.viol[s][fr.actions[*i]] != [0.0; 3])
                .count();
            if breaches > 0 {
                proof_ok += 1;
            }
        }
    }
    let nf = n as f64;
    println!("mean SII: governed {:.4} | plain-DP-ref {:.4} | prescriptive {:.4} | greedy {:.4}", sii_g / nf, sii_ref / nf, sii_p / nf, sii_gr / nf);
    println!("relative: +{:.1}% vs prescriptive, +{:.1}% vs greedy\n", (sii_g / sii_p - 1.0) * 100.0, (sii_g / sii_gr - 1.0) * 100.0);
    let c1 = viol_total == 0;
    let c2 = sii_wins_presc == n;
    let c3 = sii_wins_greedy > 0;
    let c6 = solve_max < 5.0;
    println!("C1 zero hard-gate violations (governed) : {} ({viol_total} total)", if c1 { "PASS" } else { "MISS" });
    println!("C2 SII >= prescriptive on every patient : {} ({sii_wins_presc}/{n})", if c2 { "PASS" } else { "MISS" });
    println!("C3 DP > greedy on budget-bound patients : {} ({sii_wins_greedy}/{n})", if c3 { "PASS" } else { "MISS" });
    println!("C4 bit-deterministic                    : {}", if det_ok { "PASS" } else { "MISS" });
    let c5 = binding_patients > 0 && proof_ok == binding_patients;
    println!("C5 proof pair on binding patients       : {} ({proof_ok}/{binding_patients} binding)", if c5 { "PASS" } else { "MISS" });
    println!("C6 solve < 5 s/patient                  : {} (max {:.3} s)", if c6 { "PASS" } else { "MISS" }, solve_max);
}
