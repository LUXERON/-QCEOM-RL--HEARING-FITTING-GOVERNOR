# Domain Benchmark — Hearing-Fitting Governor (measured)

Criteria frozen in PLAN.md before the measurement run (L2).

| # | Criterion | Verdict |
|---|---|---|
| C1 | Zero hard-gate violations (governed, full corpus) | **PASS** (0 total) |
| C2 | SII ≥ prescriptive incumbent on every patient | **PASS** (40/40) |
| C3 | DP > per-band greedy on budget-bound patients | **PASS** (38/40) |
| C4 | Bit-determinism across repeated solves | **PASS** |
| C5 | Proof pair on every binding patient | **PASS** (40/40) |
| C6 | Solve < 5 s per patient (hosted) | **PASS** (max 1.478 s) |

## Setup

Declared model: ANSI S3.5 third-octave SII (published band-importance
table), three speech levels (50/65/80 dB SPL) under a declared WDRC
linkage (+6/−6 dB), compressive loudness proxy with a broadband
added-loudness budget (0.5 × UCL headroom), per-band feedback caps
(45/38/30 dB), UCL − 3 dB comfort guard. Corpus: 40 seeded deterministic
patients (sloping/flat/ski-slope/notched, UCL spreads 25–40 dB). MDP:
(18 bands × 2048 budget tiers) = 38,912 states × 13 gain actions,
γ=0.9999, shaping off (both declared, see `src/lib.rs`). All contenders
share the same 4 dB gain lattice and the same rulebook.

## Measured (corpus means)

| Fitter | Mean SII | Violations |
|---|---|---|
| **QCEOM-RL governed** | **0.2718** | **0** |
| Plain-DP reference (same MDP) | 0.2718 | 0 |
| Per-band greedy | 0.2635 | 0 |
| Prescriptive (half-gain approx., research label) | 0.0468 | 0 |

- The kernel reproduces the plain-DP optimum **exactly** — the governed
  policy is the declared model's true optimum, not an approximation of it.
- +3.1% mean SII over greedy; strictly better on 38/40 patients (the
  falsifier's knapsack coupling, now corpus-wide). The two non-wins are
  patients where the budget barely binds and both land on the same table.
- +481% over the prescriptive approximation — read honestly: that
  incumbent is a deliberately simple published-shape formula, not licensed
  NAL/DSL software; the number demonstrates rulebook-governed optimization
  headroom, NOT clinical superiority over commercial prescriptions.
- Proof pair: on all 40 patients the comfort/feedback rules bind
  somewhere, and the ungoverned twin's optimal fit breaches them; the
  governed fit never does. Safety is architecture, not preference.

## The MISS trail (kept, per L2)

1. **Budget currency**: counting unaided loudness against the budget
   starved the solve (gain 0 wasn't free) → budget now governs ADDED
   loudness.
2. **Tier quantization**: 24 → 192 → 768 tiers each left enough
   ceil-rounding waste for the continuous-accounting greedy to win;
   2048 tiers (<0.9% waste) closed it. Rounding stays conservative.
3. **γ=1.0 collapses the kernel** (contraction assumption) — γ=0.9999
   declared instead.
4. **Default shaping_weight 2.0 cost 3.2% SII** against rewards of scale
   3e-3 — the plain-DP probe isolated it; shaping off restores exactness.
   New factory lesson: shaping weight is a domain unit.
5. **Off-lattice incumbent caps** (45/38/30 not multiples of 4) quietly
   handed the incumbents 1–2 dB/band the DP could not express — snap
   after cap.

Reproduce: `cargo test --release` (12/12) then
`cargo run --release --bin fitting-bench`.

## B1 addendum — Moore–Glasberg-form loudness (plan 3, measured)

The declared loudness model is now MG-form specific loudness
N' = C·[(E+A)^α − A^α] on sensation energy (α=0.3 so the single-band
shape reproduces the sone doubling anchor at moderate SL; property tests
in `auditory.rs`). Full benchmark re-run under MG, same frozen criteria:

| | Stevens proxy (pre-B1) | MG form (B1) |
|---|---|---|
| Governed mean SII | 0.2718 | **0.2780** |
| Plain-DP reference | 0.2718 | 0.2780 |
| Greedy | 0.2635 | 0.2707 |
| Prescriptive | 0.0468 | 0.0468 |
| Verdicts | 6/6 PASS | **6/6 PASS** |
| DP > greedy | 38/40 | 36/40 |

Kernel = plain-DP optimum EXACTLY under both models (L9 probe re-run).
The MG knee near threshold makes low-SL gain cheaper in loudness, so
every fitter serves slightly more audibility from the same budget; the
governed-vs-greedy gap narrows to +2.7% but remains strict on 36/40.
One anchor-test MISS during bring-up is kept honest: the doubling ratio
was first probed at SL 30→40 where the +A knee legitimately steepens
growth (measured 2.38×); the anchor holds at SL 40→50 (2.16×).
