use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};

use crate::errors::{CryptoError, Result};
use rand::RngCore;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PedersenCommitment {
    pub point: RistrettoPoint,
}

impl PedersenCommitment {
    pub fn new(amount: u64, blinding: &Scalar) -> Self {
        let g = RISTRETTO_BASEPOINT_POINT;
        let h = get_h_generator();

        let point = Scalar::from(amount) * g + blinding * h;

        Self { point }
    }

    pub fn verify(&self, amount: u64, blinding: &Scalar) -> bool {
        let expected = Self::new(amount, blinding);
        self.point == expected.point
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let compressed = CompressedRistretto(*bytes);
        let point = compressed
            .decompress()
            .ok_or(CryptoError::InvalidRisettoPoints)?;
        Ok(Self { point })
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.point.compress().to_bytes()
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            point: self.point + other.point,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            point: self.point - other.point,
        }
    }
}

pub fn get_h_generator() -> RistrettoPoint {
    let g = RISTRETTO_BASEPOINT_POINT;
    let g_bytes = g.compress().to_bytes();

    let mut hasher = Sha512::new();
    hasher.update(b"PEDERSEN_H_GENERATOR_V1");
    hasher.update(g_bytes);
    let hash = hasher.finalize();

    RistrettoPoint::from_uniform_bytes(&hash.into())
}

pub fn commit(amount: u64, blinding: &Scalar) -> PedersenCommitment {
    PedersenCommitment::new(amount, blinding)
}

pub fn verify_commitment(commitment: &PedersenCommitment, amount: u64, blinding: &Scalar) -> bool {
    commitment.verify(amount, blinding)
}

pub fn generate_blinding() -> Scalar {
    let mut bytes = [0u8; 64];
    OsRng.fill_bytes(&mut bytes);
    Scalar::from_bytes_mod_order_wide(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_commitment_basic() {
        let amount = 100u64;
        let blinding = generate_blinding();

        let commitment = PedersenCommitment::new(amount, &blinding);

        assert!(commitment.verify(amount, &blinding));
        assert!(!commitment.verify(99, &blinding));

        let wrong_blinding = generate_blinding();
        assert!(!commitment.verify(amount, &wrong_blinding));
    }

    #[test]
    fn test_commitment_serialization() {
        let amount = 42u64;
        let blinding = generate_blinding();

        let commitment = PedersenCommitment::new(amount, &blinding);

        let bytes = commitment.to_bytes();
        assert_eq!(bytes.len(), 32);

        let recovered = PedersenCommitment::from_bytes(&bytes).unwrap();
        assert_eq!(commitment, recovered)
    }

    #[test]
    fn test_commitment_homomorphic_addition() {
        let amount1 = 50u64;
        let amount2 = 30u64;

        let blinding1 = generate_blinding();
        let blinding2 = generate_blinding();

        let c1 = PedersenCommitment::new(amount1, &blinding1);
        let c2 = PedersenCommitment::new(amount2, &blinding2);

        let c_sum = c1.add(&c2);

        let expected_sum = PedersenCommitment::new(amount1 + amount2, &(blinding1 + blinding2));

        assert_eq!(c_sum, expected_sum);
    }

    #[test]
    fn test_h_generator_independence() {
        let g = RISTRETTO_BASEPOINT_POINT;
        let h = get_h_generator();

        assert_ne!(g, h);

        let h2 = get_h_generator();
        assert_eq!(h, h2);
    }

    #[test]
    fn test_convenience_functions() {
        let amount = 123u64;
        let blinding = generate_blinding();

        let commitment = commit(amount, &blinding);
        assert!(verify_commitment(&commitment, amount, &blinding))
    }

    #[test]
    fn test_multiple_random_blindings() {
        let b1 = generate_blinding();
        let b2 = generate_blinding();

        assert_ne!(b1, b2);
    }
}
