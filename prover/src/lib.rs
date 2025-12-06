pub mod network;

use anyhow::{Ok, Result};
use cryptography_types::{proof::ProofData, proof::PublicInputs, transaction::PrivateTransaction};
use sp1_sdk::{
    include_elf, HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};

pub const ELF: &[u8] = include_elf!("cryptography-zkvm");

pub fn generate_proof(tx: &PrivateTransaction) -> Result<ProofData> {
    let client = ProverClient::from_env();

    let mut stdin = SP1Stdin::new();
    stdin.write(tx);

    let (pk, _vk) = client.setup(ELF);

    let mut proof = client.prove(&pk, &stdin).run()?;

    let public_inputs: PublicInputs = proof.public_values.read();

    let proof_bytes = bincode::serialize(&proof)?;

    Ok(ProofData {
        proof: proof_bytes,
        public_inputs,
    })
}

pub fn verify_proof(proof_data: &ProofData) -> Result<()> {
    let client = ProverClient::from_env();

    let proof: SP1ProofWithPublicValues = bincode::deserialize(&proof_data.proof)?;

    let (_, vk) = client.setup(ELF);

    client.verify(&proof, &vk)?;

    Ok(())
}

pub fn get_verifying_key() -> Result<Vec<u8>> {
    let client = ProverClient::from_env();
    let (_, vk) = client.setup(ELF);

    Ok(bincode::serialize(&vk)?)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use cryptography_types::{
        commitment::CommitmentData, signature::RingSignatureData, stealth::StealthAddressData,
    };

    fn create_test_transaction() -> PrivateTransaction {
        PrivateTransaction {
            input_commitments: vec![CommitmentData::new([1u8; 32])],
            output_commitments: vec![
                CommitmentData::new([2u8; 32]),
                CommitmentData::new([3u8; 32]),
            ],
            key_image: [4u8; 32],
            ring: vec![[5u8; 32], [6u8; 32], [7u8; 32]],
            stealth_addresses: vec![StealthAddressData::new(vec![8u8; 32], [0x42u8; 20])],
            input_amounts: vec![100],
            input_blindings: vec![[9u8; 32]],
            output_amounts: vec![60, 40],
            output_blindings: vec![[10u8; 32], [11u8; 32]],
            ring_signature: RingSignatureData::new(
                vec![[12u8; 32], [13u8; 32], [14u8; 32]],
                vec![[15u8; 32], [16u8; 32], [17u8; 32]],
            ),
            secret_index: 1,
        }
    }

    #[test]
    fn test_transaction_creation() {
        let tx = create_test_transaction();
        assert_eq!(tx.input_amounts.len(), 1);
        assert_eq!(tx.output_amounts.len(), 2);

        let input_sum: u64 = tx.input_amounts.iter().sum();
        let output_sum: u64 = tx.output_amounts.iter().sum();
        assert_eq!(input_sum, output_sum);
    }

    #[test]
    fn test_generate_and_verify_proof() {
        let tx = create_test_transaction();

        let proof_data = generate_proof(&tx).expect("Failed to generate proof");
        verify_proof(&proof_data).expect("Failed to verify proof")
    }
}
