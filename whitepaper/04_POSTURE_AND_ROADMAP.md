# 04 — Licensing, Deployment & Roadmap

## Licensing posture (GATE-ZERO, in force)

[PATENT-LICENSING.md](../PATENT-LICENSING.md) governs. The load-bearing
facts: NAL-NL2/NL3 and DSL v5 are *licensed software libraries* — this
harness ships neither, claims target-match to neither, and its
prescriptive incumbent is a deliberately simple published-shape
approximation carrying a research label. The live patent clusters
(preference-elicitation ML fitting; self-fitting consumer UIs) are
avoided by architecture: no user preferences, no online learning, no
self-adjustment interface — measured inputs in, deterministic table out.
Standing gates: FTO by counsel before commercialization; no NAL/DSL
marks; never "clinically validated" or "prescription-equivalent."

## Deployment shape

Per-patient artifact: a 92-byte provenance image (`src/image.rs`) —
magic "QCFT", version, device serial, patient-model hash (audiogram +
UCL + rulebook version), table fingerprint, the 54-byte gain table
(18 bands × 3 levels), CRC32 — validated fail-closed (magic → version →
CRC → fingerprint) before a single gain is applied, exactly the
discipline verified on physical silicon in the fast-charge program. A
stale table (re-measured ear, revised rulebook) is detectable before it
is trusted. The M55/Helium NOSTD rung is proven and waiting if a
device-resident story is wanted; fitting solves are naturally off-device.

## The honest boundary

The policy is exactly optimal on the DECLARED model — SII with published
weights, a compressive loudness proxy, a fixed WDRC linkage, declared
caps. The model is not a validated clinical simulator: real-ear acoustics
(RECD), individual loudness growth, binaural summation, and actual
feedback paths are all richer than the declared forms. The claim ladder
is therefore: deterministic + rulebook-governed + exactly optimal on the
declared model (claimable today) → model fidelity upgrades (roadmap) →
clinical validation (owned by a future clinical partner, never
self-awarded).

## Roadmap

1. **Model fidelity**: Moore–Glasberg loudness in place of the Stevens
   proxy; RECD-corrected ear-canal levels; measured feedback-path caps.
2. **Identification deepening**: reuse the H1 Nelder–Mead machinery to
   fit individual loudness-growth exponents from category-scaling data.
3. **Rulebook versioning demo**: two rulebook versions, one patient —
   show the provenance hash catching the stale table.
4. **NOSTD twin** on the proven N657 rung if a device maker engagement
   materializes.
5. **Veterinary/OTC edges**: the no-FDA-pathway and Class II
   special-controls corridors from the strategy analysis.
