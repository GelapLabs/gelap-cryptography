#![cfg_attr(not(feature = "std"), no_std)]

// Declare modules
pub mod bridge;
pub mod errors;
pub mod ethereum;
pub mod pedersen;
pub mod ring_signature;
pub mod utils;
pub mod zkproof;

// Re-export commonly used items
pub use errors::{CryptoError, Result};

// Pedersen commitment exports
pub use pedersen::{commit, generate_blinding, verify_commitment, PedersenCommitment};

// Ethereum module exports
pub use ethereum::{
    checksum_address, format_address, generate_stealth_eth, parse_address, pubkey_to_address,
    scan_stealth_eth, EthAddress, EthKeyPair, StealthAddressEth,
};

// Ring signature module exports
pub use ring_signature::{sign_ring, verify_ring, RingSignature};

// Bridge module exports
pub use bridge::{address_to_ristretto, hash_to_ristretto, secp256k1_to_ristretto};

pub use zkproof::*;

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_module_structure() {
        let _blinding = generate_blinding();
        println!("All modules accessible");
    }
}
