# 01 — Why Audiology, Why Now

## The pattern

Hearing-aid fitting is a Tier-A instance of the QCEOM pattern: an
industry that keeps a *rulebook* (prescriptive gain targets, loudness
discomfort limits, feedback margins) directly beside an *optimization
problem* (make speech maximally intelligible for this ear), and today
solves it with either prescriptive formulas (no optimization at all) or
preference-elicitation ML (no reviewable basis at all).

The shape maps one-to-one onto the harnesses already proven in this
estate:

| Fast-charge program | This harness |
|---|---|
| Cell-cycler identification → PackParams | Audiogram + UCL → patient model |
| Charge-map solved under plating/thermal gates | Gain table solved under comfort/loudness/feedback gates |
| 128-byte map in a provenance flash image | 54-byte table in a provenance image |
| Re-characterize → re-solve → re-burn between sessions | Re-measure → re-solve → re-push between visits |

## The capital fit

Radiation oncology, insulin pumps, ICDs: pattern A+, entry impossible
(Class III, decades, nine figures). Audiology is the one Tier-A slot the
unbacked builder can enter: the Bose De Novo (DEN180026) created a
Class II special-controls category for self-fitting air-conduction
hearing aids; the 2022 OTC rule opened a consumer category the six
incumbent manufacturers do not monopolize; and component IP sold into an
existing device maker requires no device clearance at all.

## The regulatory asymmetry

21st Century Cures §3060 exempts clinical decision support from device
regulation when the clinician can *independently review the basis* for
the recommendation. A QCEOM fitting table is exact DP over a declared
model: every gain value reproduces bit-for-bit from (audiogram, rulebook
version, solver fingerprint) — the provenance hash is IN the deployable
image. No preference-trained neural fitter can make that claim. EU MDR
Rule 11 has no equivalent carve-out, which makes US-first the play — an
asymmetry that favors exactly this architecture and nothing the
incumbents currently ship.

## The subscription thesis

Prescription science versions (NAL-NL2 → NL3; DSL revisions), and the
fitting stack re-solves against the new rulebook. Versioned-rulebook
maintenance is a proven nine-figure business model in medicine (First
Databank, Medi-Span). The provenance hash makes the version linkage
mechanical: a table burned against rulebook v1 is *detectably* stale
under v2 before it is trusted.
