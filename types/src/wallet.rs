use crate::stealth::EthAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletState {
    pub view_secret: [u8; 32],
    pub spend_secret: [u8; 32],
    pub outputs: Vec<OwnedOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedOutput {
    pub commitment: [u8; 32],
    pub amount: u64,
    pub blinding: [u8; 32],
    pub stealth_address: EthAddress,
    pub spent: bool,
}

impl OwnedOutput {
    pub fn mark_spent(&mut self) {
        self.spent = true;
    }

    pub fn is_unspent(&self) -> bool {
        !self.spent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owned_output() {
        let mut output = OwnedOutput {
            commitment: [1u8; 32],
            amount: 100,
            blinding: [2u8; 32],
            stealth_address: [0x42u8; 20],
            spent: false,
        };

        assert!(output.is_unspent());
        output.mark_spent();
        assert!(!output.is_unspent())
    }
}
