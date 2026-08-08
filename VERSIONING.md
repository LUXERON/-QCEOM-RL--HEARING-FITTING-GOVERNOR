# Rulebook Versioning — the Subscription Mechanism, Made Mechanical

Prescription science revises: NAL-NL2 → NL3, DSL updates, changed guard-
band policy, post-market safety notices. In every deployed fitting stack
today, propagating a revision is a *process* problem — someone must
remember which fitted devices are stale. Here it is a *provenance*
problem, solved by construction.

## How it works

- Every fitting-rule constant lives in a versioned, hashable `Rulebook`
  ([src/rulebook.rs](src/rulebook.rs)). The hash binds CONTENT, not just
  the version label — a silently edited constant under the same version
  number still changes the hash.
- The deployable image's patient-hash mixes the full rulebook hash with
  the audiogram, UCLs, and budget ([src/image.rs](src/image.rs)). A
  table is therefore bound to (this ear) × (these rules).
- A fitter/device expecting rulebook V2 recomputes the expected hash;
  a V1-fitted image fails the comparison and is REFUSED fail-closed
  before a single gain is applied. Re-solve (seconds), re-image,
  redeploy.

## Measured demo (`cargo run --release --bin version_demo`, seed 4242)

```
V1 fit   : SII 0.2965, deployed, validates
revision : guard 3→5 dB, mid/high caps 38/30 → 34/26 (RULEBOOK_V2)
STALE TABLE REFUSED: hash 0x0a282151bf000e9a != expected 0xa7ad2b97e53ee40b
V2 refit : SII 0.2377, validates under v2, 28/54 table bytes changed
```

The SII drop is the honest cost of the tightened rules — the governed
solve gives up exactly the audibility the new rulebook forbids, and
nothing more (it remains the exact optimum under V2).

## Why this is the business

Versioned-rulebook maintenance is a proven nine-figure category in
medicine (First Databank, Medi-Span ship nothing but maintained
rulebooks). The recurring unit here: rulebook revision → fleet-wide
mechanical staleness → per-patient re-solve at ~1 s each → redeploy.
The provenance hash is also the 21st Century Cures §3060 story: the
basis of every recommendation is independently recomputable from
(audiogram, rulebook version, solver fingerprint).
