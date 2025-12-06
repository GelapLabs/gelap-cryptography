pub use crate::pedersen::{commit, generate_blinding, verify_commitment, PedersenCommitment};

pub use crate::ring_signature::{sign_ring, verify_ring, RingSignature};

pub use crate::bridge::{address_to_ristretto, hash_to_ristretto, secp256k1_to_ristretto};

pub use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zkproof_exports() {
        let blinding = generate_blinding();
        let commitment = commit(100, &blinding);
        assert!(verify_commitment(&commitment, 100, &blinding));

        println!("All zkproofs exports work")
    }
}
