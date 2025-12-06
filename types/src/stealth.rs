use serde::{Deserialize, Serialize};

pub type EthAddress = [u8; 20];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StealthAddressData {
    pub ephemeral_pubkey: Vec<u8>,
    pub stealth_address: EthAddress,
}

impl StealthAddressData {
    pub fn new(ephemeral_pubkey: Vec<u8>, stealth_address: EthAddress) -> Self {
        Self {
            ephemeral_pubkey,
            stealth_address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_address_data() {
        let stealth = StealthAddressData::new(vec![1u8; 33], [0x42u8; 20]);

        assert_eq!(stealth.ephemeral_pubkey.len(), 33);
        assert_eq!(stealth.stealth_address, [0x42u8; 20]);
    }
}
