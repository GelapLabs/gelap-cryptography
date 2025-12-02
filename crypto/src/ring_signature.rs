use crate::errors::{CryptoError, Result};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT, ristretto::RistrettoPoint, scalar::Scalar,
};
use rand::RngCore;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSignature {
    pub key_image: RistrettoPoint,
    pub c: Vec<Scalar>,
    pub r: Vec<Scalar>,
}

impl RingSignature {
    pub fn sign(
        message: &[u8],
        secret_key: &Scalar,
        secret_index: usize,
        public_keys: &[RistrettoPoint],
    ) -> Self {
        let n = public_keys.len();
        assert!(n > 0, "Ring must have at least one member");
        assert!(secret_index < n, "Secret index out of bonds");

        let key_image = compute_key_image(secret_key, &public_keys[secret_index]);

        let mut c = vec![Scalar::ZERO; n];
        let mut r = vec![Scalar::ZERO; n];

        let alpha = generate_random_scalar();

        let start_idx = (secret_index + 1) % n;

        let mut hasher = Sha512::new();
        hasher.update(b"RING_SIG_V1");
        hasher.update(message);
        hasher.update((alpha * RISTRETTO_BASEPOINT_POINT).compress().as_bytes());
        hasher.update(
            (alpha * hash_to_point(&public_keys[secret_index]))
                .compress()
                .as_bytes(),
        );

        let hash = hasher.finalize();
        c[start_idx] = Scalar::from_bytes_mod_order_wide(&hash.into());

        for i in 0..(n - 1) {
            let idx = (start_idx + i) % n;
            let next_idx = (idx + 1) % n;

            r[idx] = generate_random_scalar();

            let l = r[idx] * RISTRETTO_BASEPOINT_POINT + c[idx] * public_keys[idx];

            let r_part = r[idx] * hash_to_point(&public_keys[idx]) + c[idx] * key_image;

            let mut hasher = Sha512::new();
            hasher.update(b"RING_SIG_V1");
            hasher.update(message);
            hasher.update(l.compress().as_bytes());
            hasher.update(r_part.compress().as_bytes());

            let hash = hasher.finalize();
            c[next_idx] = Scalar::from_bytes_mod_order_wide(&hash.into());
        }

        r[secret_index] = alpha - c[secret_index] * secret_key;

        Self { key_image, c, r }
    }

    pub fn verify(&self, message: &[u8], public_keys: &[RistrettoPoint]) -> bool {
        let n = public_keys.len();

        if self.c.len() != n || self.r.len() != n {
            return false;
        }

        if n == 0 {
            return false;
        }

        for i in 0..n {
            let next_i = (i + 1) % n;

            let l = self.r[i] * RISTRETTO_BASEPOINT_POINT + self.c[i] * public_keys[i];

            let r_part = self.r[i] * hash_to_point(&public_keys[i]) + self.c[i] * self.key_image;

            let mut hasher = Sha512::new();
            hasher.update(b"RING_SIG_V1");
            hasher.update(message);
            hasher.update(l.compress().as_bytes());
            hasher.update(r_part.compress().as_bytes());

            let hash = hasher.finalize();
            let computed_c = Scalar::from_bytes_mod_order_wide(&hash.into());

            if computed_c != self.c[next_i] {
                return false;
            }
        }
        true
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Serialization should not fail")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| CryptoError::Deserialization(e.to_string()))
    }
}

fn compute_key_image(secret_key: &Scalar, public_key: &RistrettoPoint) -> RistrettoPoint {
    let hash_point = hash_to_point(public_key);
    secret_key * hash_point
}

fn hash_to_point(point: &RistrettoPoint) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"HASH_TO_POINT_V1");
    hasher.update(point.compress().as_bytes());
    let hash = hasher.finalize();

    RistrettoPoint::from_uniform_bytes(&hash.into())
}

fn generate_random_scalar() -> Scalar {
    let mut bytes = [0u8; 64];
    OsRng.fill_bytes(&mut bytes);
    Scalar::from_bytes_mod_order_wide(&bytes)
}

pub fn sign_ring(
    message: &[u8],
    secret_key: &Scalar,
    secret_index: usize,
    public_keys: &[RistrettoPoint],
) -> RingSignature {
    RingSignature::sign(message, secret_key, secret_index, public_keys)
}

pub fn verify_ring(
    signature: &RingSignature,
    message: &[u8],
    public_keys: &[RistrettoPoint],
) -> bool {
    signature.verify(message, public_keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_ring(size: usize) -> (Vec<Scalar>, Vec<RistrettoPoint>) {
        let mut secret_keys = Vec::new();
        let mut public_keys = Vec::new();

        for _ in 0..size {
            let sk = generate_random_scalar();
            let pk = sk * RISTRETTO_BASEPOINT_POINT;
            secret_keys.push(sk);
            public_keys.push(pk);
        }

        (secret_keys, public_keys)
    }

    #[test]
    fn test_ring_signature_basic() {
        let ring_size = 8;
        let (secret_keys, public_keys) = create_ring(ring_size);

        let secret_index = 2;
        let message = b"test transaction";

        let signature = RingSignature::sign(
            message,
            &secret_keys[secret_index],
            secret_index,
            &public_keys,
        );

        assert!(signature.verify(message, &public_keys));
    }

    #[test]
    fn test_ring_signature_different_sizes() {
        for size in [3, 8, 16, 32] {
            let (secret_keys, public_keys) = create_ring(size);
            let secret_index = size / 2;
            let message = b"test";

            let signature = RingSignature::sign(
                message,
                &secret_keys[secret_index],
                secret_index,
                &public_keys,
            );

            assert!(
                signature.verify(message, &public_keys),
                "Failed for ring size {}",
                size
            )
        }
    }

    #[test]
    fn test_key_image_uniqueness() {
        let (secret_keys, public_keys) = create_ring(5);

        let message = b"test";

        let sig1 = RingSignature::sign(message, &secret_keys[0], 0, &public_keys);
        let sig2 = RingSignature::sign(message, &secret_keys[1], 1, &public_keys);

        assert_ne!(sig1.key_image, sig2.key_image);
    }

    #[test]
    fn test_key_image_consistency() {
        let (secret_keys, public_keys) = create_ring(5);
        let secret_index = 2;

        let sig1 = RingSignature::sign(
            b"msg1",
            &secret_keys[secret_index],
            secret_index,
            &public_keys,
        );
        let sig2 = RingSignature::sign(
            b"msg2",
            &secret_keys[secret_index],
            secret_index,
            &public_keys,
        );

        assert_eq!(sig1.key_image, sig2.key_image);
    }

    #[test]
    fn test_serialization() {
        let ring_size = 8;
        let (secret_keys, public_keys) = create_ring(ring_size);

        let secret_index = 3;
        let message = b"test transaction";

        let signature = RingSignature::sign(
            message,
            &secret_keys[secret_index],
            secret_index,
            &public_keys,
        );

        let bytes = signature.to_bytes();
        assert!(!bytes.is_empty());

        let recovered = RingSignature::from_bytes(&bytes).unwrap();
        assert!(recovered.verify(message, &public_keys));

        assert_eq!(signature.key_image, recovered.key_image);
    }

    #[test]
    fn test_convenience_functions() {
        let ring_size = 5;
        let (secret_keys, public_keys) = create_ring(ring_size);

        let secret_index = 2;
        let message = b"test";

        let signature = sign_ring(
            message,
            &secret_keys[secret_index],
            secret_index,
            &public_keys,
        );

        assert!(verify_ring(&signature, message, &public_keys));
    }

    #[test]
    fn test_anonymity_set_size_one() {
        let (secret_keys, public_keys) = create_ring(1);
        let signature = RingSignature::sign(b"msg", &secret_keys[0], 0, &public_keys);

        assert!(signature.verify(b"msg", &public_keys));
    }

    #[test]
    fn test_signature_components_length() {
        let ring_size = 10;
        let (secret_keys, public_keys) = create_ring(ring_size);

        let signature = RingSignature::sign(b"test", &secret_keys[5], 5, &public_keys);

        assert_eq!(signature.c.len(), ring_size);
        assert_eq!(signature.r.len(), ring_size);
    }

    #[test]
    fn test_wrong_ring_size() {
        let (secret_keys, public_keys) = create_ring(5);
        let signature = RingSignature::sign(b"msg", &secret_keys[2], 2, &public_keys);

        let (_, wrong_public_keys) = create_ring(3);
        assert!(!signature.verify(b"msg", &wrong_public_keys));
    }

    #[test]
    fn test_tampered_signatured() {
        let (secret_keys, public_keys) = create_ring(5);
        let mut signature = RingSignature::sign(b"msg", &secret_keys[2], 2, &public_keys);

        signature.r[0] = generate_random_scalar();

        assert!(!signature.verify(b"msg", &public_keys));
    }

    #[test]
    fn test_multiple_signatures_same_ring() {
        let (secret_keys, public_keys) = create_ring(8);

        let sig1 = RingSignature::sign(b"tx1", &secret_keys[0], 0, &public_keys);
        let sig2 = RingSignature::sign(b"tx2", &secret_keys[3], 3, &public_keys);
        let sig3 = RingSignature::sign(b"tx3", &secret_keys[7], 7, &public_keys);

        assert_ne!(sig1.key_image, sig2.key_image);
        assert_ne!(sig2.key_image, sig3.key_image);
        assert_ne!(sig1.key_image, sig3.key_image);
    }

    #[test]
    fn test_key_image_computation() {
        let sk = generate_random_scalar();
        let pk = sk * RISTRETTO_BASEPOINT_POINT;

        let ki1 = compute_key_image(&sk, &pk);
        let ki2 = compute_key_image(&sk, &pk);

        assert_eq!(ki1, ki2);
    }
}
