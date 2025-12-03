#![cfg_attr(not(feature = "std"), no_std)]

// Declare modules
pub mod errors;
pub mod ethereum;
pub mod pedersen;
pub mod ring_signature;
pub mod utils;

// pub mod bridge;
// pub mod zkproof;

// Re-export commonly used items
pub use errors::{CryptoError, Result};

// Pedersen commitment exports
pub use pedersen::{commit, verify_commitment, PedersenCommitment};

// Ethereum module exports
pub use ethereum::{
    checksum_address, format_address, generate_stealth_eth, parse_address, pubkey_to_address,
    scan_stealth_eth, EthAddress, EthKeyPair, StealthAddressEth,
};

// Ring signature module exports
pub use ring_signature::{sign_ring, verify_ring, RingSignature};

// // Stealth address exports
// pub use stealth::{generate_stealth, scan_stealth, StealthAddress};

// pub use bridge::{address_to_ring_member, secp256k1_to_commitment};

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        assert!(true)
    }
}
