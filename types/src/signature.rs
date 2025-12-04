use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSignatureData {
    pub c: Vec<[u8; 32]>,
    pub r: Vec<[u8; 32]>,
}

impl RingSignatureData {
    pub fn new(c: Vec<[u8; 32]>, r: Vec<[u8; 32]>) -> Self {
        Self { c, r }
    }

    pub fn ring_size(&self) -> usize {
        self.c.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_signature_data() {
        let sig = RingSignatureData::new(
            vec![[1u8; 32], [2u8; 32], [3u8; 32]],
            vec![[4u8; 32], [5u8; 32], [6u8; 32]],
        );

        assert_eq!(sig.ring_size(), 3);
        assert_eq!(sig.c.len(), 3);
        assert_eq!(sig.r.len(), 3);
    }
}
