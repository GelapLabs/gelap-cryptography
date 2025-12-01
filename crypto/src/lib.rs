#![cfg_attr(not(feature = "std"), no_std)]

// Declare modules
pub mod bridge;
pub mod errors;
pub mod ethereum;
pub mod pedersen;
pub mod ring_signature;
pub mod stealth;
pub mod utils;
pub mod utils;
pub mod zkproof;

// Re-export commonly used items
pub use errors::{CryptoError, Result};

// Ethereum module exports
pub use ethereum::{
    format_address, generate_stealth_eth, parse_address, pubkey_to_address, scan_stealth_eth,
    EthAddress, EthKeyPair, StealthAddressEth,
};

// Stealth address exports
pub use stealth::{generate_stealth, scan_stealth, StealthAddress};

pub use pedersen::{commit, verify_commitment, PedersenCommitment};

pub use bridge::{address_to_ring_member, secp256k1_to_commitment};

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        assert!(true)
    }
}
