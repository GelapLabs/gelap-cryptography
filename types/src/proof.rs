use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    pub proof: Vec<u8>,
    pub public_inputs: PublicInputs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    pub input_commitments: Vec<[u8; 32]>,
    pub output_commitments: Vec<[u8; 32]>,
    pub key_image: [u8; 32],
    pub ring: Vec<[u8; 32]>,
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    #[test]
    fn test_public_inputs() {
        let inputs = PublicInputs {
            input_commitments: vec![[1u8; 32]],
            output_commitments: vec![[2u8; 32]],
            key_image: [3u8; 32],
            ring: vec![[4u8; 32], [5u8; 32]],
        };

        assert_eq!(inputs.ring.len(), 2)
    }
}
