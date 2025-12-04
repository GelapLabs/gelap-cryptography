use crate::commitment::CommitmentData;
use crate::signature::RingSignatureData;
use crate::stealth::StealthAddressData;
use serde::{Deserialize, Serialize};

pub type EthAddress = [u8; 20];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateTransaction {
    pub input_commitments: Vec<CommitmentData>,
    pub output_commitments: Vec<CommitmentData>,
    pub key_image: [u8; 32],
    pub ring: Vec<[u8; 32]>,
    pub stealth_addresses: Vec<StealthAddressData>,

    pub input_amounts: Vec<u64>,
    pub input_blindings: Vec<[u8; 32]>,
    pub output_amounts: Vec<u64>,
    pub output_blindings: Vec<[u8; 32]>,
    pub ring_signature: RingSignatureData,
    pub secret_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub commitment: [u8; 32],
    pub stealth_address: EthAddress,
    pub ephemeral_pubkey: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub commitment: [u8; 32],
    pub key_image: [u8; 32],
}

#[derive(Debug, Default)]
pub struct TransactionBuilder {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    input_amounts: Vec<u64>,
    input_blindings: Vec<[u8; 32]>,
    output_amounts: Vec<u64>,
    output_blindings: Vec<[u8; 32]>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_input(
        mut self,
        commitment: [u8; 32],
        key_image: [u8; 32],
        amount: u64,
        blinding: [u8; 32],
    ) -> Self {
        self.inputs.push(TransactionInput {
            commitment,
            key_image,
        });
        self.input_amounts.push(amount);
        self.input_blindings.push(blinding);
        self
    }

    pub fn add_output(
        mut self,
        commitment: [u8; 32],
        stealth_address: EthAddress,
        ephemeral_pubkey: Vec<u8>,
        amount: u64,
        blinding: [u8; 32],
    ) -> Self {
        self.outputs.push(TransactionOutput {
            commitment,
            stealth_address,
            ephemeral_pubkey,
        });

        self.output_amounts.push(amount);
        self.output_blindings.push(blinding);
        self
    }

    pub fn inputs(&self) -> &[TransactionInput] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[TransactionOutput] {
        &self.outputs
    }

    pub fn input_amounts(&self) -> &[u64] {
        &self.input_amounts
    }

    pub fn output_amounts(&self) -> &[u64] {
        &self.output_amounts
    }

    pub fn verify_balance(&self) -> bool {
        let input_sum: u64 = self.input_amounts.iter().sum();
        let output_sum: u64 = self.output_amounts.iter().sum();

        input_sum == output_sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builder() {
        let builder = TransactionBuilder::new()
            .add_input([1u8; 32], [2u8; 32], 100, [3u8; 32])
            .add_output([4u8; 32], [0x42u8; 20], vec![5u8; 33], 60, [6u8; 32])
            .add_output([7u8; 32], [0x43u8; 20], vec![8u8; 33], 40, [9u8; 32]);

        assert!(builder.verify_balance());
        assert_eq!(builder.inputs().len(), 1);
        assert_eq!(builder.outputs().len(), 2);
    }
}
