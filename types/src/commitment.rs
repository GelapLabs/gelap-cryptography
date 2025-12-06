#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitmentData {
    pub commitment: [u8; 32],
}

impl CommitmentData {
    pub fn new(commitment: [u8; 32]) -> Self {
        Self { commitment }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.commitment
    }
}

impl From<[u8; 32]> for CommitmentData {
    fn from(commitment: [u8; 32]) -> Self {
        Self::new(commitment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_data() {
        let data = CommitmentData::new([42u8; 32]);
        assert_eq!(data.as_bytes(), &[42u8; 32]);

        let from_array: CommitmentData = [1u8; 32].into();
        assert_eq!(from_array.commitment, [1u8; 32]);
    }
}
