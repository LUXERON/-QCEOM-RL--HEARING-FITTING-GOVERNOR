//! The provenance-hashed deployable fitting table — the §3060
//! "independently reviewable basis" artifact, reusing the flash-image
//! discipline proven on silicon in the fast-charge program (H2).
//!
//! Layout (92 bytes, little-endian): magic "QCFT" u32 · version u32 ·
//! patient/device serial u64 · patient-model hash u64 (audiogram + UCL +
//! rulebook version) · table fingerprint u64 · 54-byte table (18 bands ×
//! 3 levels, gain in dB as u8) + 2 pad · CRC32 over bytes 0..88.
//! Validation is fail-closed: magic → version → CRC → fingerprint.

use crate::auditory::{Patient, BANDS};
use crate::rulebook::Rulebook;

pub const MAGIC: u32 = 0x5446_4351; // "QCFT" little-endian
pub const VERSION: u32 = 1;
pub const TABLE_LEN: usize = BANDS * 3; // 54
pub const IMAGE_LEN: usize = 92;

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Hash the patient model + the FULL rulebook content hash: the
/// provenance binding that makes a stale table (re-measured ear OR
/// revised rulebook) detectable before it is trusted.
pub fn patient_hash(p: &Patient) -> u64 {
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15 ^ p.rulebook.hash();
    for k in 0..BANDS {
        h = h.rotate_left(7) ^ p.thr[k].to_bits();
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = h.rotate_left(7) ^ p.ucl[k].to_bits();
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    }
    h.rotate_left(7) ^ p.budget.to_bits()
}

pub fn table_from_gains(rb: &Rulebook, g65: &[f64; BANDS]) -> [u8; TABLE_LEN] {
    let mut t = [0u8; TABLE_LEN];
    for k in 0..BANDS {
        let gains = rb.linked_gains(g65[k], k);
        for (li, &g) in gains.iter().enumerate() {
            t[k * 3 + li] = g.round() as u8;
        }
    }
    t
}

pub fn fingerprint(table: &[u8; TABLE_LEN]) -> u64 {
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
    for &b in table.iter() {
        h = h.rotate_left(7) ^ b as u64;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    }
    h
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageError {
    BadMagic,
    BadVersion,
    BadCrc,
    FingerprintMismatch,
}

pub fn build(serial: u64, phash: u64, table: &[u8; TABLE_LEN]) -> [u8; IMAGE_LEN] {
    let mut img = [0u8; IMAGE_LEN];
    img[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    img[4..8].copy_from_slice(&VERSION.to_le_bytes());
    img[8..16].copy_from_slice(&serial.to_le_bytes());
    img[16..24].copy_from_slice(&phash.to_le_bytes());
    img[24..32].copy_from_slice(&fingerprint(table).to_le_bytes());
    img[32..32 + TABLE_LEN].copy_from_slice(table);
    let crc = crc32(&img[..88]);
    img[88..92].copy_from_slice(&crc.to_le_bytes());
    img
}

#[derive(Debug)]
pub struct ValidImage {
    pub serial: u64,
    pub patient_hash: u64,
    pub table: [u8; TABLE_LEN],
}

pub fn validate(img: &[u8]) -> Result<ValidImage, ImageError> {
    let u32_at = |i: usize| u32::from_le_bytes(img[i..i + 4].try_into().unwrap());
    let u64_at = |i: usize| u64::from_le_bytes(img[i..i + 8].try_into().unwrap());
    if img.len() < IMAGE_LEN || u32_at(0) != MAGIC {
        return Err(ImageError::BadMagic);
    }
    if u32_at(4) != VERSION {
        return Err(ImageError::BadVersion);
    }
    if crc32(&img[..88]) != u32_at(88) {
        return Err(ImageError::BadCrc);
    }
    let mut table = [0u8; TABLE_LEN];
    table.copy_from_slice(&img[32..32 + TABLE_LEN]);
    if fingerprint(&table) != u64_at(24) {
        return Err(ImageError::FingerprintMismatch);
    }
    Ok(ValidImage { serial: u64_at(8), patient_hash: u64_at(16), table })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit_env::{extract_gains, FitEnv};
    use crate::rulebook::{RULEBOOK_V1, RULEBOOK_V2};
    use crate::domain_engine;

    #[test]
    fn crc32_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn image_roundtrip_and_fail_closed() {
        let p = Patient::generate(42, RULEBOOK_V1);
        let env = FitEnv::new(&p);
        let (policy, _) = domain_engine().train(&env);
        let table = table_from_gains(&p.rulebook, &extract_gains(&env, &policy));
        let phash = patient_hash(&p);
        let img = build(0xEA12_0042, phash, &table);
        let v = validate(&img).expect("valid");
        assert_eq!(v.serial, 0xEA12_0042);
        assert_eq!(v.patient_hash, phash);
        assert_eq!(v.table, table);
        // Table flip → CRC refuses; forged CRC → fingerprint refuses.
        let mut bad = img;
        bad[40] ^= 1;
        assert_eq!(validate(&bad).unwrap_err(), ImageError::BadCrc);
        let crc = crc32(&bad[..88]);
        bad[88..92].copy_from_slice(&crc.to_le_bytes());
        assert_eq!(validate(&bad).unwrap_err(), ImageError::FingerprintMismatch);
        // A different patient (or rulebook version) changes the hash: the
        // stale-table detection that anchors the reviewability posture.
        let p2 = Patient::generate(43, RULEBOOK_V1);
        assert_ne!(patient_hash(&p2), phash);
        let p_v2 = Patient::generate(42, RULEBOOK_V2);
        assert_ne!(patient_hash(&p_v2), phash);
    }
}
