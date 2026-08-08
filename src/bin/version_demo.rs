//! The rulebook-versioning demo — the subscription mechanism made
//! mechanical. One patient, two rulebook versions:
//!
//! 1. Fit solved and imaged under RULEBOOK_V1.
//! 2. The rulebook revises (V2: comfort guard 3→5 dB, tighter mid/high
//!    feedback caps). A V2-expecting fitter recomputes the expected
//!    provenance hash — the V1 image's hash no longer matches, and the
//!    stale table is REFUSED before a single gain is applied.
//! 3. Re-solve under V2 → new image validates → the new fit measurably
//!    respects the tightened rules.

use hearing_gov::auditory::Patient;
use hearing_gov::domain_engine;
use hearing_gov::fit_env::{extract_gains, FitEnv};
use hearing_gov::image::{build, patient_hash, table_from_gains, validate};
use hearing_gov::rulebook::{RULEBOOK_V1, RULEBOOK_V2};

fn main() {
    let seed = 4242u64;
    println!("== rulebook-versioning demo: one patient, one revision ==\n");

    // Fit under V1.
    let p1 = Patient::generate(seed, RULEBOOK_V1);
    let env1 = FitEnv::new(&p1);
    let (pol1, _) = domain_engine().train(&env1);
    let g1 = extract_gains(&env1, &pol1);
    let t1 = table_from_gains(&p1.rulebook, &g1);
    let h1 = patient_hash(&p1);
    let img1 = build(0xEA12_4242, h1, &t1);
    println!("V1 fit   : SII {:.4}, patient-hash {:#018x}", p1.mean_sii(&g1), h1);
    let v = validate(&img1).expect("V1 image validates structurally");
    assert_eq!(v.patient_hash, h1);
    println!("V1 image : {} B, validates, hash matches (deployed)\n", img1.len());

    // The rulebook revises. The device/fitter now expects V2 provenance.
    let p2 = Patient::generate(seed, RULEBOOK_V2); // same ear, new rules
    assert_eq!(p1.thr, p2.thr, "audiogram unchanged");
    let expected_v2 = patient_hash(&p2);
    println!("rulebook v1 hash {:#018x}", RULEBOOK_V1.hash());
    println!("rulebook v2 hash {:#018x}", RULEBOOK_V2.hash());
    let stale = validate(&img1).expect("structurally intact");
    if stale.patient_hash != expected_v2 {
        println!(
            "STALE TABLE REFUSED: image hash {:#018x} != expected {:#018x} under v2\n",
            stale.patient_hash, expected_v2
        );
    } else {
        panic!("stale table was accepted - versioning is broken");
    }

    // Re-solve under V2.
    let env2 = FitEnv::new(&p2);
    let (pol2, _) = domain_engine().train(&env2);
    let g2 = extract_gains(&env2, &pol2);
    let t2 = table_from_gains(&p2.rulebook, &g2);
    let img2 = build(0xEA12_4242, patient_hash(&p2), &t2);
    let v2 = validate(&img2).expect("V2 image validates");
    assert_eq!(v2.patient_hash, expected_v2);
    println!("V2 refit : SII {:.4}, image validates under v2", p2.mean_sii(&g2));

    // The revision must actually bind: identical tables would mean the
    // demo demonstrated nothing.
    let changed = (0..t1.len()).filter(|&i| t1[i] != t2[i]).count();
    println!("table bytes changed by the revision: {changed}/54");
    assert!(changed > 0, "revision did not bind on this patient");
    println!("\nCHAIN COMPLETE: deploy -> revise -> refuse-stale -> refit -> redeploy");
}
