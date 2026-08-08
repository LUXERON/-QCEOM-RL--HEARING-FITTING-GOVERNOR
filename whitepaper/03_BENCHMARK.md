# 03 — Benchmark & the MISS Trail

## Measured result (40 seeded patients, criteria frozen first)

All six declared criteria PASS — see
[DOMAIN-BENCHMARK.md](../DOMAIN-BENCHMARK.md) for the table. Headlines:

- **Kernel = plain-DP reference exactly** (mean SII 0.2718 = 0.2718).
  The shipped table is the declared model's true optimum.
- **+3.1% SII over per-band greedy**, strictly better on 38/40 — the
  loudness-budget coupling is real and the exact solve monetizes it.
- **Zero hard-gate violations**, proof pair holds on all 40 patients,
  bit-deterministic, < 1.5 s per patient.
- +481% over the prescriptive half-gain approximation — a headroom
  demonstration against a deliberately simple published-shape formula,
  NOT a clinical-superiority claim against licensed prescriptions.

## The MISS trail — five failures worth more than the passes

1. **Unaided loudness charged against the budget.** Gain 0 wasn't free,
   severe patients had no feasible actions, the solve starved (first run:
   17 violations, SII 0.081). The budget now governs *added* loudness.
2. **Quantization ate the optimum three times.** 24 tiers wasted up to
   75% of the budget to ceil-rounding; 192 and 768 still lost to the
   continuous-accounting greedy; 2048 tiers (<0.9% waste) closed it.
3. **γ=1.0 collapses the kernel** — its machinery assumes contraction.
   γ=0.9999 is measured indistinguishable from undiscounted.
4. **The default shaping weight cost 3.2% SII.** Potential-based shaping
   is policy-invariant on paper; against rewards of scale 3×10⁻³ the
   kernel's finite-tolerance machinery turned shaping_weight 2.0 into a
   measured optimality gap. The plain-DP probe (same table, pure Bellman)
   isolated it in one run — the same probe discipline that caught the
   H1 optimizer failure in the fast-charge program. **New factory
   lesson: shaping weight is a domain unit, not a constant; probe any
   new domain against plain DP before trusting defaults.**
5. **Off-lattice incumbent caps.** The feedback caps (45/38/30) are not
   multiples of the 4 dB action step; un-snapped clipping quietly handed
   the incumbents 1–2 dB per band the DP could not express. Fair
   benchmarks share one lattice.

Every one of these was surfaced by a declared, mechanical check — the
falsifier, the plain-DP probe, the criteria themselves. That is the
factory's actual product: not the numbers, the discipline that makes the
numbers trustworthy.
