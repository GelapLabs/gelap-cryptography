use sha2::{Digest, Sha256};
use sha3::Keccak256;

pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

pub fn keccak_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

pub fn random_bytes<const N: usize>() -> [u8; N] {
    use rand::RngCore;
    let mut bytes = [0u8; N];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

pub fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(s).map_err(|e| format!("Invalid hex: {}", e));
}

pub fn to_hex(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let data = b"Gelap accounts";
        let hash = hash_sha256(data);
        assert_eq!(hash.len(), 32);

        let hash2 = hash_sha256(data);
        assert_eq!(hash, hash2)
    }

    #[test]
    fn test_hex_conversion() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let hex = to_hex(&data);
        assert_eq!(hex, "0x12345678");

        let decoded = from_hex(&hex).unwrap();
        assert_eq!(decoded, data);

        let decoded2 = from_hex("12345678").unwrap();
        assert_eq!(decoded2, data);
    }

    #[test]
    fn test_random_bytes() {
        let bytes1: [u8; 32] = random_bytes();
        let bytes2: [u8; 32] = random_bytes();

        assert_ne!(bytes1, bytes2)
    }
}
