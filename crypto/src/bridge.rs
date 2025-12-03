use crate::ethereum::EthAddress;
use curve25519_dalek::ristretto::RistrettoPoint;
use secp256k1::PublicKey;
use sha2::{Digest, Sha512};

pub fn secp256k1_to_ristretto(pubkey: &PublicKey) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"SECP256K1_TO_RISTRETTO_V1");
    hasher.update(pubkey.serialize());

    let hash = hasher.finalize();
    RistrettoPoint::from_uniform_bytes(&hash.into())
}

pub fn address_to_ristretto(address: &EthAddress) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"ETH_ADDRESS_TO_RISTRETTO_V1");
    hasher.update(address);

    let hash = hasher.finalize();
    RistrettoPoint::from_uniform_bytes(&hash.into())
}

pub fn hash_to_ristretto(data: &[u8]) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"HASH_TO_RISTRETTO_V1");
    hasher.update(data);

    let hash = hasher.finalize();
    RistrettoPoint::from_uniform_bytes(&hash.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use secp256k1::{Secp256k1, SecretKey};

    #[test]
    fn test_secp256k1_to_ristretto() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let secret = SecretKey::new(&mut rng);
        let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret);

        let point = secp256k1_to_ristretto(&pubkey);

        assert_ne!(point, RistrettoPoint::default());

        println!("secp256k1 to Ristretto conversion works");
    }

    #[test]
    fn test_secp256k1_to_ristretto_deterministic() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let secret = SecretKey::new(&mut rng);
        let pubkey = secp256k1::PublicKey::from_secret_key(&secp, &secret);

        let point1 = secp256k1_to_ristretto(&pubkey);
        let point2 = secp256k1_to_ristretto(&pubkey);

        assert_eq!(point1, point2);

        println!("Conversion is deterministic");
    }

    #[test]
    fn test_address_to_ristretto() {
        let address: EthAddress = [0x42; 20];
        let point = address_to_ristretto(&address);

        assert_ne!(point, RistrettoPoint::default());

        println!("Address to Ristretto conversion works");
    }

    #[test]
    fn test_address_to_ristretto_deterministic() {
        let address: EthAddress = [0x42; 20];

        let point1 = address_to_ristretto(&address);
        let point2 = address_to_ristretto(&address);

        assert_eq!(point1, point2);

        println!("Address conversion is deterministic");
    }

    #[test]
    fn test_different_addresses_different_points() {
        let address1: EthAddress = [0x11; 20];
        let address2: EthAddress = [0x22; 20];

        let point1 = address_to_ristretto(&address1);
        let point2 = address_to_ristretto(&address2);

        assert_ne!(point1, point2);

        println!("Different addresses produce different points");
    }

    #[test]
    fn test_hash_to_ristretto() {
        let data1 = b"Hello world";
        let data2 = b"Goodbye world";

        let point1 = hash_to_ristretto(data1);
        let point2 = hash_to_ristretto(data2);

        assert_ne!(point1, point2);

        let point1_again = hash_to_ristretto(data1);
        assert_eq!(point1, point1_again);

        println!("Generic hash-to-point works correctly");
    }
}
