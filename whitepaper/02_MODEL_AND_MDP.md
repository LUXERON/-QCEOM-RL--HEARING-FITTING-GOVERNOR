# 02 — The Declared Model & the MDP

## The objective: ANSI S3.5 SII

Speech intelligibility is computable: the Speech Intelligibility Index
sums band audibility weighted by the published third-octave
band-importance function (18 bands, 160 Hz–8 kHz). Audibility per band is
sensation level over the 30 dB speech dynamic range, clipped to [0,1].
The harness evaluates three input levels (50/65/80 dB SPL) under a
declared WDRC linkage (+6 dB soft, −6 dB loud around the optimized
average-level gain) and maximizes the mean SII.

## The rulebook (hard gates, reward-neutral)

1. **Comfort**: amplified output ≤ UCL − 3 dB guard, every band, every
   level. Gain 0 is exempt — a device adding nothing cannot violate
   comfort (a modeling decision the first benchmark run forced).
2. **Broadband loudness budget**: amplification may ADD at most half the
   loudness headroom between unaided loud speech and every band at UCL
   (compressive loudness proxy, exponent 0.6). Loudness summation is the
   coupling that makes fitting a sequential problem: the budget spent on
   one band is unavailable to every other.
3. **Feedback margin**: per-band gain caps (45/38/30 dB, declining with
   frequency) modeling open-loop-gain headroom.

None of these appear in the reward. The proof pair certifies the
separation: on all 40 corpus patients the ungoverned twin's
audibility-optimal fit breaches comfort or feedback somewhere; the
governed fit never does.

## The MDP

Band-sequential semi-MDP: states = (band × budget tier), 18 × 2048 =
38,912; actions = 13 average-level gain steps (0–48 dB, 4 dB); each
decision sets one band's gain, paying its added loudness (worst level —
the loud input dominates band-for-band under the linkage, factory lesson
L8 applied to loudness) into the carried budget, ceil-rounded so real
spend can only land under the declared ceiling. Rewards are the band's
importance-weighted mean audibility. γ=0.9999 and shaping_weight=0 are
declared domain configuration measured against a plain-DP reference —
chapter 03 tells that story honestly.

## The falsifier that had to pass first

Before any Rust: a 200-patient Python experiment (falsifier/falsifier.py)
asking whether exact DP actually beats per-band greedy when the budget
binds — if not, the kernel adds nothing and the harness dies. Result:
DP won 200/200 against both a band-order greedy and a marginal
importance-per-loudness heuristic (largest gap: SII 0.342 vs 0.182 —
nearly double the intelligibility from the same loudness budget). The
Rust corpus reproduces the effect at scale: strictly better than greedy
on 38/40 patients.
