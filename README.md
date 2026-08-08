# [QCEOM RL] HEARING-FITTING-GOVERNOR

**Hearing-aid fitting as exact constrained optimization on the QCEOM-RL
kernel**: per-patient gain tables that maximize the ANSI S3.5 Speech
Intelligibility Index under hard comfort / broadband-loudness / feedback
gates — deterministic, bit-replayable, and provenance-hashed so every
gain value is independently reviewable from (audiogram, rulebook version,
solver fingerprint). The first medical-domain harness from the QCEOM
factory, entering the one Tier-A slot open to an unbacked builder
(Class II special-controls category, 21st Century Cures §3060
reviewability posture — see whitepaper/01).

## Measured result (40 seeded patients, criteria frozen before the run)

| Fitter | Mean SII | Hard-gate violations |
|---|---|---|
| **QCEOM-RL governed** | **0.2718** | **0** |
| Plain-DP reference (same MDP) | 0.2718 | 0 |
| Per-band greedy | 0.2635 | 0 |
| Prescriptive approximation (research label) | 0.0468 | 0 |

Six declared criteria, six PASS ([DOMAIN-BENCHMARK.md](DOMAIN-BENCHMARK.md)):
zero violations, SII ≥ prescriptive on 40/40, **strictly better than
greedy on 38/40** (+3.1% mean — the loudness-budget coupling the A1
falsifier proved 200/200 in Python), bit-deterministic, proof pair on all
40 binding patients, < 1.5 s per patient. The kernel reproduces the
plain-DP optimum **exactly** — the shipped table is the declared model's
true optimum.

The +481% over the prescriptive incumbent demonstrates rulebook-governed
optimization headroom against a deliberately simple published-shape
formula. It is NOT a clinical claim against licensed prescriptions
(NAL/DSL are licensed software this repo neither ships nor approximates
beyond a research-labeled baseline — [PATENT-LICENSING.md](PATENT-LICENSING.md)).

## How it works

- **Model** ([src/auditory.rs](src/auditory.rs)): ANSI S3.5 SII (published
  band-importance), three input levels under a declared WDRC linkage,
  compressive added-loudness budget, per-band feedback caps, UCL−3 dB
  comfort guard. Seeded deterministic patient corpus.
- **MDP** ([src/fit_env.rs](src/fit_env.rs)): (18 bands × 2048 budget
  tiers) × 13 gain steps; worst-level budget accounting (L8), ceil-rounded
  conservative; reward = importance-weighted audibility ONLY — safety is
  hard gates excluded from the Bellman max, certified by the proof pair.
- **Incumbents** ([src/prescriptive.rs](src/prescriptive.rs)): published-
  shape prescriptive approximation + per-band greedy, same lattice, same
  rulebook.
- **Deployable** ([src/image.rs](src/image.rs)): 92-byte provenance image
  (magic/version/serial/patient-hash/fingerprint/54-B table/CRC32),
  fail-closed loader — the flash-image discipline verified on physical
  STM32N657 silicon in the fast-charge program.
- **Kernel config** ([src/lib.rs](src/lib.rs)): γ=0.9999 and shaping off,
  both DECLARED and set by a plain-DP probe. The probe surfaced a new
  factory lesson — the default shaping weight cost a measured 3.2% SII
  against this domain's 3e-3-scale rewards; shaping weight is a domain
  unit, not a constant.

## The MISS trail

Five instructive failures are kept in
[whitepaper/03](whitepaper/03_BENCHMARK.md): unaided loudness wrongly
charged to the budget; three rounds of quantization losses (24→192→768→
2048 tiers); the γ=1.0 kernel collapse; the shaping-weight optimality
gap; off-lattice incumbent caps. Each was caught by a declared mechanical
check, not by eyeballing.

## Reproduce

```bash
cargo test --release          # 11/11
cargo run --release --bin fitting-bench
python falsifier/falsifier.py # the A1 kill-test: 200/200
```

Requires SSH access to LUXERON/QCEOM-RL-CORE-KERNEL.

## White paper

[whitepaper/](whitepaper/README.md): why audiology, the declared model,
the benchmark with its MISS trail, and the licensing/deployment posture.
