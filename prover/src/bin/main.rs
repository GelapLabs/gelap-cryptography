use anyhow::{Ok, Result};
use cryptography_prover::{generate_proof, verify_proof};
use cryptography_types::{
    commitment::CommitmentData, signature::RingSignatureData, stealth::StealthAddressData,
    transaction::PrivateTransaction,
};

fn main() -> Result<()> {
    let tx = create_example_transaction();
    let proof_data = generate_proof(&tx)?;
    verify_proof(&proof_data)?;
    Ok(())
}

fn create_example_transaction() -> PrivateTransaction {
    PrivateTransaction {
        input_commitments: vec![CommitmentData::new([1u8; 32])],
        output_commitments: vec![
            CommitmentData::new([2u8; 32]),
            CommitmentData::new([3u8; 32]),
        ],
        key_image: [4u8; 32],
        ring: vec![[5u8; 32], [6u8; 32], [7u8; 32], [8u8; 32], [9u8; 32]],
        stealth_addresses: vec![
            StealthAddressData::new(vec![10u8; 32], [0x42u8; 20]),
            StealthAddressData::new(vec![11u8; 32], [0x43u8; 20]),
        ],
        input_amounts: vec![100],
        input_blindings: vec![[12u8; 32]],
        output_amounts: vec![60, 40],
        output_blindings: vec![[13u8; 32], [14u8; 32]],
        ring_signature: RingSignatureData::new(
            vec![[15u8; 32], [16u8; 32], [17u8; 32], [18u8; 32], [19u8; 32]],
            vec![[20u8; 32], [21u8; 32], [22u8; 32], [23u8; 32], [24u8; 32]],
        ),
        secret_index: 2,
    }
}
