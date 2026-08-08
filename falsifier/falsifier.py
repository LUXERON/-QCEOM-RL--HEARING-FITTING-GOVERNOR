"""GATE-A1 falsifier: does exact DP beat per-band greedy fitting when the
broadband loudness budget binds?

Minimal declared model (all constants in the open literature):
- 18 third-octave SII bands, ANSI S3.5 Table 3 band-importance weights.
- Audibility per band: A_k = clip((speech_k + gain_k - threshold_k)/30, 0, 1)
  (the SII 30-dB speech-dynamic-range rule); SII = sum I_k * A_k.
- Loudness cost per band (compressive Stevens-style proxy):
  N_k = max(0, speech_k + gain_k - threshold_k)^0.6; broadband sum N <= B.
- Gains: 0..48 dB in 4 dB steps, per-band feedback cap 40 dB.

Contenders on the same discrete lattice:
- greedy-band-order: maximize each band's audibility in order while budget
  remains (what naive per-band fitting does under a loudness clip).
- greedy-by-ratio: global marginal importance-per-loudness ratio ordering
  (the smart heuristic).
- exact DP over (band, budget-tier) - the kernel's shape.

Kill criterion: if DP never beats BOTH greedies across the seeded corpus,
the kernel adds nothing and the harness dies. Deterministic, seeded.
"""

import itertools

I18 = [0.0083, 0.0095, 0.0150, 0.0289, 0.0440, 0.0578, 0.0653, 0.0711,
       0.0818, 0.0844, 0.0882, 0.0898, 0.0868, 0.0844, 0.0771, 0.0527,
       0.0364, 0.0185]
SPEECH65 = [32.5, 34.8, 34.4, 34.7, 33.1, 31.5, 30.6, 30.0, 28.6, 27.6,
            26.4, 25.0, 23.4, 22.2, 20.8, 18.9, 17.6, 16.5]  # LTASS-ish
GAINS = list(range(0, 49, 4))
FEEDBACK_CAP = 40.0
BUDGET_STEP = 4.0  # loudness-budget discretization


def splitmix(x):
    x = (x + 0x9E3779B97F4A7C15) & (2**64 - 1)
    z = x
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & (2**64 - 1)
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & (2**64 - 1)
    return x, (z ^ (z >> 31))


def audibility(sp, g, thr):
    return max(0.0, min(1.0, (sp + g - thr) / 30.0))


def loudness(sp, g, thr):
    sl = max(0.0, sp + g - thr)
    return sl ** 0.6


def score(thresholds, gains):
    sii = sum(I18[k] * audibility(SPEECH65[k], gains[k], thresholds[k])
              for k in range(18))
    loud = sum(loudness(SPEECH65[k], gains[k], thresholds[k])
               for k in range(18))
    return sii, loud


def greedy_band_order(thr, budget):
    # Per-band audibility maximization in band order, reserving the
    # zero-gain base loudness of the bands still to come.
    gains, spent = [], 0.0
    for k in range(18):
        base_rest = sum(loudness(SPEECH65[j], 0, thr[j])
                        for j in range(k + 1, 18))
        best_g, best_a = 0, -1.0
        for g in GAINS:
            if g > FEEDBACK_CAP:
                break
            cost = loudness(SPEECH65[k], g, thr[k])
            if spent + cost + base_rest > budget + 1e-9:
                continue
            a = audibility(SPEECH65[k], g, thr[k])
            if a > best_a:
                best_a, best_g = a, g
        spent += loudness(SPEECH65[k], best_g, thr[k])
        gains.append(best_g)
    return gains


def greedy_by_ratio(thr, budget):
    gains = [0] * 18
    spent = sum(loudness(SPEECH65[k], 0, thr[k]) for k in range(18))
    while True:
        best = None
        for k in range(18):
            gi = GAINS.index(gains[k])
            if gi + 1 >= len(GAINS) or GAINS[gi + 1] > FEEDBACK_CAP:
                continue
            g2 = GAINS[gi + 1]
            dsii = I18[k] * (audibility(SPEECH65[k], g2, thr[k]) -
                             audibility(SPEECH65[k], gains[k], thr[k]))
            dloud = (loudness(SPEECH65[k], g2, thr[k]) -
                     loudness(SPEECH65[k], gains[k], thr[k]))
            if dsii <= 0:
                continue
            if spent + dloud > budget:
                continue
            ratio = dsii / max(dloud, 1e-9)
            if best is None or ratio > best[0]:
                best = (ratio, k, g2, dloud)
        if best is None:
            break
        _, k, g2, dloud = best
        gains[k] = g2
        spent += dloud
    return gains


def exact_dp(thr, budget):
    # states: (band k, budget tier t); value = max SII from band k on.
    tiers = int(budget / BUDGET_STEP) + 1
    NEG = -1e9
    val = [[NEG] * (tiers + 1) for _ in range(19)]
    act = [[0] * (tiers + 1) for _ in range(19)]
    for t in range(tiers + 1):
        val[18][t] = 0.0
    for k in range(17, -1, -1):
        for t in range(tiers + 1):
            rem = t * BUDGET_STEP
            best, bg = NEG, 0
            for g in GAINS:
                if g > FEEDBACK_CAP:
                    break
                cost = loudness(SPEECH65[k], g, thr[k])
                if cost > rem + 1e-9:
                    continue
                t2 = int((rem - cost) / BUDGET_STEP)
                v = I18[k] * audibility(SPEECH65[k], g, thr[k]) + val[k + 1][t2]
                if v > best + 1e-12:
                    best, bg = v, g
            val[k][t] = best
            act[k][t] = bg
    # rollout from the tier the actual budget maps to; re-floor the tier
    # after each spend exactly as the DP transition did.
    gains, t = [], int(budget / BUDGET_STEP)
    rem = t * BUDGET_STEP
    for k in range(18):
        g = act[k][t]
        gains.append(g)
        rem = rem - loudness(SPEECH65[k], g, thr[k])
        t = max(0, int(rem / BUDGET_STEP))
        rem = t * BUDGET_STEP
    return gains


def gen_patient(seed):
    # sloping loss: start 20-40 dB, slope 2-6 dB/band, plus notch chance
    s = seed
    s, r = splitmix(s)
    start = 20 + (r % 21)
    s, r = splitmix(s)
    slope = 2 + (r % 5)
    thr = [min(90.0, start + slope * k) for k in range(18)]
    s, r = splitmix(s)
    if r % 3 == 0:
        s, r = splitmix(s)
        notch = 6 + (r % 8)
        thr[notch] = min(95.0, thr[notch] + 25)
    return thr


def main():
    wins_vs_order, wins_vs_ratio, wins_vs_both, n = 0, 0, 0, 0
    worst_example = None
    for seed in range(1, 201):
        thr = gen_patient(seed * 7919)
        # budget: 60% of the unconstrained max loudness -> binding
        full = sum(loudness(SPEECH65[k], min(48, FEEDBACK_CAP), thr[k])
                   for k in range(18))
        budget = 0.6 * full
        budget = round(budget / BUDGET_STEP) * BUDGET_STEP
        g_dp = exact_dp(thr, budget)
        g_go = greedy_band_order(thr, budget)
        g_gr = greedy_by_ratio(thr, budget)
        s_dp, l_dp = score(thr, g_dp)
        s_go, l_go = score(thr, g_go)
        s_gr, l_gr = score(thr, g_gr)
        assert l_dp <= budget + 1e-6
        n += 1
        if s_dp > s_go + 1e-9:
            wins_vs_order += 1
        if s_dp > s_gr + 1e-9:
            wins_vs_ratio += 1
        if s_dp > s_go + 1e-9 and s_dp > s_gr + 1e-9:
            wins_vs_both += 1
            gap = s_dp - max(s_go, s_gr)
            if worst_example is None or gap > worst_example[0]:
                worst_example = (gap, seed, s_dp, s_go, s_gr, budget)
    print(f"corpus: {n} budget-bound synthetic patients")
    print(f"DP > greedy-band-order : {wins_vs_order}/{n}")
    print(f"DP > greedy-by-ratio   : {wins_vs_ratio}/{n}")
    print(f"DP > BOTH              : {wins_vs_both}/{n}")
    if worst_example:
        gap, seed, s_dp, s_go, s_gr, b = worst_example
        print(f"largest joint win: seed {seed}, SII dp={s_dp:.4f} "
              f"order={s_go:.4f} ratio={s_gr:.4f} (gap {gap:.4f}, budget {b:.0f})")
    verdict = "HARNESS LIVES" if wins_vs_both > 0 else "HARNESS DIES"
    print(f"VERDICT: {verdict}")


if __name__ == "__main__":
    main()
