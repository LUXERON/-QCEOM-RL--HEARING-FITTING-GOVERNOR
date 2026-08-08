# GATE-ZERO — Patent & Licensing Scan: Hearing-Aid Fitting

Written BEFORE any code (factory discipline; same gate the fast-charge
governor passed). Scan date 2026-08-08. This is an engineering risk
posture, not legal advice; FTO review by counsel is the standing gate
before any commercial use.

## 1. The prescription formulas are LICENSED SOFTWARE — that is the central fact

- **NAL-NL2 / NAL-NL3** (National Acoustic Laboratories, Australia): the
  prescription is delivered to manufacturers as a licensed library; NAL-NL3
  is being licensed to "more than 90% of the global hearing aid
  manufacturer market". The *scientific descriptions* (Keidser et al.,
  2011, and the Siemens/AudiologyOnline expert series) are published in the
  open literature. Copyright protects NAL's code, and "NAL-NL2"/"NAL-NL3"
  function as marks; the published math itself is not copyrightable.
- **DSL v5** (National Centre for Audiology, Western Ontario): same
  structure — manufacturers purchase the DSL dynamic-linked library; the
  m[i/o] algorithm's stages (expansion, linear gain, compression, output
  limiting) are described in the open literature.

**Posture**: this harness never ships, embeds, or claims either licensed
library. The incumbent baseline is a *published-literature prescriptive
approximation*, clearly labeled "research benchmark reimplemented from
published descriptions; not NAL or DSL licensed software; no target-match
claim". The harness's own product is NOT a prescription formula at all —
it is a governed optimizer over a declared objective (speech
intelligibility index) under declared constraints, which is a different
artifact class from a prescription rulebook.

## 2. The live patent clusters — and the architecture that avoids them

- **Preference-driven ML fitting (Widex/WS Audiology "SoundSense Learn"
  family)**: Bayesian/Gaussian optimization over user A/B paired
  comparisons, cloud-aggregated learning, real-time preference-based
  adjustment (~a dozen comparisons over a >2000-setting space). Claims
  cluster on *eliciting user preferences and learning settings from them*.
  **Design-around by architecture**: this harness takes NO user
  preference input, does NO online learning, NO paired comparisons, NO
  cloud aggregation. Input is a measured audiogram + discomfort levels;
  output is a deterministic table solved offline. The entire claim
  surface (preference elicitation loop) is absent.
- **Self-fitting OTC UI (Bose → Lexie De Novo DEN180026; Starkey
  US 12,273,683)**: user-driven self-adjustment interfaces (two-wheel
  gain/compression control; self-reported hearing measures). Claims
  cluster on *the self-fitting interaction*. This harness has no consumer
  self-adjustment UI; fitting is computed from measured inputs. The De
  Novo did establish the **Class II special-controls category for
  self-fitting air-conduction hearing aids** — regulatory precedent that
  benefits any later entrant, patent-wise irrelevant to this design.
- **Core WDRC compression** (K-Amp era, 1990s): foundational wide-dynamic-
  range-compression patents are expired. Multi-band compression as such is
  free ground.
- **Apple** holds the largest hearing-related portfolio (transparency
  modes, AirPods hearing features) — relevant to consumer earbud
  products, not to a fitting-table optimizer sold as component IP.

## 3. Standards and models used by the harness

- **ANSI S3.5-1997 (R2017) Speech Intelligibility Index**: a public
  standard with a published calculation procedure and band-importance
  tables; implementing a standard's procedure is normal practice (the
  standard text is copyrighted; the method is not). The SII is the
  harness's declared objective.
- **Loudness models (Moore & Glasberg family)**: published psychoacoustic
  models; used for the loudness-ceiling constraint.
- Audiogram formats and UCL measurement are open clinical practice.

## 4. What is kept as potential own-novelty

Exact-DP fitting over a (band × level × loudness-budget) lattice with
hard-gated discomfort/feedback constraints, reward-neutral safety, proof-
pair governance evidence, and provenance-hashed deployable tables
(bit-reproducible from audiogram + rulebook version + solver fingerprint —
the 21st Century Cures §3060 "independently reviewable basis" posture).
Novelty search before any filing; nothing here is a filing decision.

## 5. Standing gates

1. FTO review by counsel before commercialization (unchanged, inherited
   from the fast-charge program).
2. No "NAL", "DSL", or manufacturer marks in product naming or claims.
3. Language discipline: "deterministic, bit-replayable, exactly optimal
   on the declared model" — never "clinically validated", "certified",
   or "prescription-equivalent". Clinical validation is a roadmap item
   owned by a future clinical partner, not a claim.
