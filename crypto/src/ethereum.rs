use crate::errors::{CryptoError, Result};
use crate::utils::hash_keccak256;
use rand::thread_rng;
use secp256k1::{All, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

pub type EthAddress = [u8; 20];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StealthAddressEth {
    pub ephemeral_pubkey: Vec<u8>,
    pub stealth_address: EthAddress,
}

#[derive(Debug, Clone)]
pub struct EthKeyPair {
    pub secret: SecretKey,
    pub public: PublicKey,
    pub address: EthAddress,
}

impl EthKeyPair {
    pub fn random() -> Result<Self> {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let secret = SecretKey::new(&mut rng);
        let public = PublicKey::from_secret_key(&secp, &secret);
        let address = pubkey_to_address(&public);

        Ok(Self {
            secret,
            public,
            address,
        })
    }

    pub fn from_secret(secret: SecretKey) -> Result<Self> {
        let secp = Secp256k1::new();
        let public = PublicKey::from_secret_key(&secp, &secret);
        let address = pubkey_to_address(&public);

        Ok(Self {
            secret,
            public,
            address,
        })
    }

    pub fn address_hex(&self) -> String {
        format_address(&self.address)
    }
}

pub fn generate_stealth_eth(
    recipient_view_pubkey: &PublicKey,
    recipient_spend_pubkey: &PublicKey,
) -> Result<(StealthAddressEth, SecretKey)> {
    let secp = Secp256k1::new();
    let mut rng = thread_rng();

    let ephemeral_secret = SecretKey::new(&mut rng);
    let ephemeral_pubkey = PublicKey::from_secret_key(&secp, &ephemeral_secret);

    let shared_secret_point = compute_ecdh(&secp, recipient_view_pubkey, &ephemeral_secret)?;

    let shared_hash = hash_shared_secret(&shared_secret_point);
    let hs_scalar = SecretKey::from_slice(&shared_hash).map_err(|_| CryptoError::InvalidScalar)?;

    let hs_point = PublicKey::from_secret_key(&secp, &hs_scalar);
    let stealth_pubkey = hs_point
        .combine(&recipient_spend_pubkey)
        .map_err(|_| CryptoError::PointAdditionFailed)?;

    let stealth_address = pubkey_to_address(&stealth_pubkey);

    Ok((
        StealthAddressEth {
            ephemeral_pubkey: ephemeral_pubkey.serialize().to_vec(),
            stealth_address,
        },
        ephemeral_secret,
    ))
}

pub fn scan_stealth_eth(
    stealth_addr: &StealthAddressEth,
    view_secret: &SecretKey,
    spend_pubkey: &PublicKey,
) -> Result<Option<SecretKey>> {
    let secp = Secp256k1::new();

    let ephmeral_pubkey = PublicKey::from_slice(&stealth_addr.ephemeral_pubkey)
        .map_err(|_| CryptoError::InvalidPublicKey)?;

    let shared_secret_point = compute_ecdh(&secp, &ephmeral_pubkey, view_secret)?;

    let shared_hash = hash_shared_secret(&shared_secret_point);
    let hs_scalar = SecretKey::from_slice(&shared_hash).map_err(|_| CryptoError::InvalidScalar)?;

    let hs_point = PublicKey::from_secret_key(&secp, &hs_scalar);
    let expected_stealth = hs_point
        .combine(spend_pubkey)
        .map_err(|_| CryptoError::PointAdditionFailed)?;

    let expected_address = pubkey_to_address(&expected_stealth);

    if expected_address == stealth_addr.stealth_address {
        Ok(Some(hs_scalar))
    } else {
        Ok(None)
    }
}

pub fn compute_ecdh(
    secp: &Secp256k1<All>,
    pubkey: &PublicKey,
    secret: &SecretKey,
) -> Result<PublicKey> {
    let mut shared_secret = *pubkey;

    shared_secret = shared_secret
        .mul_tweak(secp, &(*secret).into())
        .map_err(|_| CryptoError::EcdhFailed)?;

    Ok(shared_secret)
}

pub fn hash_shared_secret(point: &PublicKey) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(b"STEALTH_PAYMENT_V1");
    hasher.update(point.serialize());

    let hash = hasher.finalize();
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

pub fn pubkey_to_address(pubkey: &PublicKey) -> EthAddress {
    let uncompressed = pubkey.serialize_uncompressed();

    let pubkey_bytes = &uncompressed[1..];

    let hash = hash_keccak256(pubkey_bytes);

    let mut address = [0u8; 20];

    address.copy_from_slice(&hash[12..]);
    address
}

pub fn format_address(address: &EthAddress) -> String {
    format!("0x{}", hex::encode(address))
}

pub fn parse_address(s: &str) -> Result<EthAddress> {
    let s = s.strip_prefix("0x").unwrap_or(s);

    if s.len() != 40 {
        return Err(CryptoError::InvalidInput(format!(
            "Expected 40 hex chars, got {}",
            s.len()
        )));
    }

    let bytes = hex::decode(s).map_err(|e| CryptoError::InvalidInput(e.to_string()))?;

    let mut address = [0u8; 20];
    address.copy_from_slice(&bytes);
    Ok(address)
}

pub fn checksum_address(address: &EthAddress) -> String {
    let addr_hex = hex::encode(address);
    let hash = hash_keccak256(addr_hex.as_bytes());

    let mut result = String::from("0x");
    for (i, ch) in addr_hex.chars().enumerate() {
        if ch.is_ascii_digit() {
            result.push(ch);
        } else {
            let hash_byte = hash[i / 2];
            let hash_nibble = if i % 2 == 0 {
                hash_byte >> 4
            } else {
                hash_byte & 0x0f
            };

            if hash_nibble >= 8 {
                result.push(ch.to_ascii_uppercase());
            } else {
                result.push(ch);
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = EthKeyPair::random().unwrap();

        assert_eq!(keypair.address.len(), 20);

        let formatted = format_address(&keypair.address);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);

        assert_eq!(formatted, keypair.address_hex());
    }

    #[test]
    fn test_keypair_from_secret() {
        let keypair1 = EthKeyPair::random().unwrap();
        let keypair2 = EthKeyPair::from_secret(keypair1.secret).unwrap();

        assert_eq!(keypair1.public, keypair2.public);
        assert_eq!(keypair1.address, keypair2.address);
    }

    #[test]
    fn test_stealth_address_generation() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let view_secret = SecretKey::new(&mut rng);
        let view_pubkey = PublicKey::from_secret_key(&secp, &view_secret);

        let spend_secret = SecretKey::new(&mut rng);
        let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_secret);

        let (stealth_addr, _eph) = generate_stealth_eth(&view_pubkey, &spend_pubkey).unwrap();

        let formatted = format_address(&stealth_addr.stealth_address);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);

        assert_eq!(stealth_addr.ephemeral_pubkey.len(), 33);

        println!("Generated stealth address: {}", formatted);
    }

    #[test]
    fn test_stealth_scanning() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let view_secret = SecretKey::new(&mut rng);
        let view_pubkey = PublicKey::from_secret_key(&secp, &view_secret);
        let spend_secret = SecretKey::new(&mut rng);
        let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_secret);

        let (stealth_addr, _) = generate_stealth_eth(&view_pubkey, &spend_pubkey).unwrap();

        let found = scan_stealth_eth(&stealth_addr, &view_secret, &spend_pubkey).unwrap();
        assert!(
            found.is_some(),
            "Recipient should find their own stealth address"
        );

        let wrong_secret = SecretKey::new(&mut rng);
        let not_found = scan_stealth_eth(&stealth_addr, &wrong_secret, &spend_pubkey).unwrap();
        assert!(
            not_found.is_none(),
            "Wrong view key shouldn't find stealth address"
        );

        println!("Stealth address scanning works correctly");
    }

    #[test]
    fn test_stealth_address_uniqueness() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let view_secret = SecretKey::new(&mut rng);
        let view_pubkey = PublicKey::from_secret_key(&secp, &view_secret);
        let spend_secret = SecretKey::new(&mut rng);
        let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_secret);

        let (stealth1, _) = generate_stealth_eth(&view_pubkey, &spend_pubkey).unwrap();
        let (stealth2, _) = generate_stealth_eth(&view_pubkey, &spend_pubkey).unwrap();

        assert_ne!(stealth1.stealth_address, stealth2.stealth_address);
        assert_ne!(stealth1.ephemeral_pubkey, stealth2.ephemeral_pubkey);

        println!("Each stealth address is unique");
    }

    #[test]
    fn test_address_formatting() {
        let address: EthAddress = [0x1a; 20];
        let formatted = format_address(&address);

        assert_eq!(formatted, "0x1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a");

        let parsed = parse_address(&formatted).unwrap();
        assert_eq!(parsed, address);

        let parsed2 = parse_address("1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a").unwrap();
        assert_eq!(parsed2, address);

        println!("Address formatting works correctly");
    }

    #[test]
    fn test_address_parsing_invalid() {
        assert!(parse_address("0x123").is_err());

        assert!(parse_address("0x1234567890123456789012345678901234567890aa").is_err());

        assert!(parse_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG").is_err());

        println!("Invalid address parsing correctly rejected");
    }

    #[test]
    fn test_checksum_address() {
        let addr = parse_address("0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed").unwrap();
        let checksummed = checksum_address(&addr);

        assert_eq!(checksummed, "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed");

        println!("EIP-55 checksum works correctly");
    }

    #[test]
    fn test_pubkey_to_address_consistency() {
        let keypair = EthKeyPair::random().unwrap();

        let addr1 = pubkey_to_address(&keypair.public);

        assert_eq!(addr1, keypair.address);

        println!("Public key to address conversion consistent");
    }

    #[test]
    fn test_multiple_stealth_scans() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let recipients: Vec<_> = (0..3)
            .map(|_| {
                let view_secret = SecretKey::new(&mut rng);
                let view_pubkey = PublicKey::from_secret_key(&secp, &view_secret);
                let spend_secret = SecretKey::new(&mut rng);
                let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_secret);
                (view_secret, view_pubkey, spend_secret, spend_pubkey)
            })
            .collect();

        let (stealth_addr, _) = generate_stealth_eth(&recipients[1].1, &recipients[1].3).unwrap();

        for (i, (view_secret, _, _, spend_pubkey)) in recipients.iter().enumerate() {
            let found = scan_stealth_eth(&stealth_addr, view_secret, spend_pubkey).unwrap();
            if i == 1 {
                assert!(found.is_some(), "Recipient 1 should find their address");
            } else {
                assert!(
                    found.is_none(),
                    "Recipient {} should not find recipient 1's address",
                    i
                );
            }

            println!("Multiple recipient scanning works correctly");
        }
    }

    #[test]
    fn test_ecdh_symmetry() {
        let secp = Secp256k1::new();
        let mut rng = thread_rng();

        let alice_secret = SecretKey::new(&mut rng);
        let alice_pubkey = PublicKey::from_secret_key(&secp, &alice_secret);

        let bob_secret = SecretKey::new(&mut rng);
        let bob_pubkey = PublicKey::from_secret_key(&secp, &bob_secret);

        let shared1 = compute_ecdh(&secp, &bob_pubkey, &alice_secret).unwrap();
        let shared2 = compute_ecdh(&secp, &alice_pubkey, &bob_secret).unwrap();

        assert_eq!(shared1, shared2);

        println!("ECDH is works");
    }
}
