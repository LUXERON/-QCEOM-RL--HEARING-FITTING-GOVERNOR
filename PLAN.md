# [QCEOM RL] Hearing-Fitting Governor — Spec & Phased Plan

## QCEOM-RL DOMAIN HARNESS SPEC — HEARING-AID FITTING GOVERNOR

1. **DOMAIN & BUYER**: audiology / hearing-device makers (OTC self-fitting
   category, Class II special controls per DEN180026 precedent) and
   fitting-software vendors; component-IP sale into an existing device
   maker is the primary commercial thesis (capital-fit reality). US-first
   (21st Century Cures §3060 favors reviewable deterministic CDS).
2. **DECISION PROBLEM**: given a measured audiogram + uncomfortable-
   loudness levels, choose per-band, per-input-level insertion gains that
   maximize speech intelligibility (SII) subject to hard comfort/feedback/
   compression constraints; solved offline per patient, deployed as a
   static table to an MCU/DSP.
3. **STATE ENCODER**: lattice = (frequency band × input level × broadband
   loudness-budget tier); 18 SII third-octave bands × 3 speech levels
   (50/65/80 dB SPL) × 16 budget tiers ≈ 864 states; actions = gain steps
   (e.g., 0–60 dB in 4 dB steps → 16 actions); band-sequential semi-MDP —
   each decision sets one band's gain at one level, loudness budget is the
   carried state (the coupling that makes DP non-trivial).
4. **CHARGE ENCODER**: attractors = high-band-importance × audibility-gap
   states (SII weight × sensation-level shortfall); repulsors = states
   near the loudness ceiling and feedback-margin floor; PME grid 32,
   resolution study vs band count.
5. **CONSTRAINT DECLARATIONS (the rulebook)**: hard gates (never violated):
   per-band output ≤ UCL − declared guard band; broadband loudness ≤
   ceiling; feedback margin ≥ 0 (gain ≤ open-loop-gain limit per band);
   compression ratio between level-specific gains within declared bounds
   (sound-quality rule). Soft pressures: inter-band gain smoothness.
   Authority: ANSI S3.5 (SII), published loudness-model literature,
   published UCL/guard-band clinical practice; all constants declared in
   code, none hidden.
6. **SCENARIO CORPUS**: seeded deterministic patient generator (audiogram
   shapes: flat/sloping/ski-slope/notched, severity grades, UCL spreads)
   via bit-mixing; real-audiogram CSV loader interface defined.
7. **INCUMBENT BASELINE**: published-literature prescriptive approximation
   (research label, no NAL/DSL software, no target-match claim) + a
   per-band greedy optimizer (the "obvious algorithm") — the falsifier
   demands DP beat greedy where the loudness budget binds.
8. **ACCEPTANCE CRITERIA (frozen before runs)**:
   C1 zero hard-gate violations across the full patient corpus (governed);
   C2 mean SII ≥ prescriptive incumbent at equal-or-lower broadband
   loudness, per patient class;
   C3 DP > per-band greedy SII on budget-bound patients (existence +
   corpus rate reported);
   C4 bit-determinism across repeated solves;
   C5 proof pair — ungoverned twin exceeds UCL/feedback limits at
   audibility-tempting states, governed never does;
   C6 solve < 5 s per patient (hosted).
9. **ENGINE CONFIG**: γ = 0.999 declared (band-sequential horizon ≈ 54
   decisions; L3 discount-distortion rule), default tolerance/cap;
   documented as domain configuration.
10. **DEPLOYMENT TIER**: hybrid — hosted solve; table packed into the
    QCMF-style provenance image (serial = patient/device ID, params-hash =
    audiogram+rulebook-version hash, fingerprint, CRC32); NOSTD/N657 rung
    reused from the fast-charge program (M55+Helium is literally
    hearing-aid-class silicon). Climb to QEMU/N657 only after hosted
    criteria pass.
11. **EVIDENCE PLAN**: DOMAIN-BENCHMARK.md (criteria table + measured);
    proof pair; determinism; MISS policy L2; falsifier result reported
    even if it kills the harness.
12. **REPO**: "[QCEOM-RL]-HEARING-FITTING-GOVERNOR" (GitHub sanitizes to
    -QCEOM-RL--HEARING-FITTING-GOVERNOR); register via
    github-repo-directory skill.

## Phases (execution loop grinds in order; DONE only with evidence)

- **A0 GATE-ZERO — DONE**: PATENT-LICENSING.md (licensed-formula fact,
  preference-ML design-around, standards posture).
- **A1 Falsifier — DONE, HARNESS LIVES** (falsifier/falsifier.py, measured
  2026-08-08): 200/200 budget-bound synthetic patients where exact DP over
  (band × loudness-budget) beats BOTH per-band-order greedy AND marginal
  importance-per-loudness ratio greedy; largest gap SII 0.3416 (DP) vs
  0.1815 (order) / 0.0328 (ratio) at seed 62. Caveat recorded: the ratio
  greedy is a heuristic and its weakness on some seeds may partly reflect
  discrete-step traps rather than fundamental inferiority; the DP-vs-
  band-order margin alone is decisive, and DP is exactly optimal on the
  discretized declared model by construction.
- **A2 Repo + sign-off**: spec (above) approved; repo created + pushed.
- **A3 Reference sim**: SII (published band-importance table) + loudness
  excitation + feedback margin; validated against published monotonicity/
  worked values; analytic sanity tests.
- **A4 Fitting MDP**: lattice, gates (L8 worst-case sense per constraint),
  reward = SII gain only (reward-neutral safety), proof pair,
  determinism.
- **A5 Benchmark**: incumbents + closed-loop scoring over seeded patient
  corpus; C1–C6 verdicts → DOMAIN-BENCHMARK.md.
- **A6 Identification + image**: audiogram/UCL → patient model (H1
  machinery); per-patient table → provenance image (H2 machinery).
- **A7 Publish**: whitepaper (incl. §3060 reviewability framing), README,
  push, register, memory.
