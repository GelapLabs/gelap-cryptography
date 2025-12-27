# Testing Guide

This guide covers testing the Gelap Cryptography library and ensuring correctness of cryptographic operations.

## Quick Start

```bash
# Run all tests
cargo test --all

# Run with output
cargo test -- --nocapture

# Run specific crate tests
cargo test --package cryptography-crypto
cargo test --package cryptography-types
cargo test --package cryptography-prover
```

## Test Structure

```
gelap-cryptography/
├── crypto/src/
│   ├── pedersen.rs      # Commitment tests
│   ├── ring_signature.rs # Ring signature tests
│   ├── ethereum.rs      # Stealth address tests
│   └── bridge.rs        # Curve bridge tests
├── types/src/
│   └── transaction.rs   # Serialization tests
└── prover/src/
    └── lib.rs           # Proof generation tests
```

## Crypto Crate Tests

### Pedersen Commitment Tests

```bash
cargo test --package cryptography-crypto pedersen
```

**Test Cases:**

```rust
#[cfg(test)]
mod pedersen_tests {
    use super::*;
    
    #[test]
    fn test_commitment_creation() {
        let amount = 100u64;
        let blinding = generate_blinding();
        let commitment = commit(amount, &blinding);
        
        assert!(verify_commitment(&commitment, amount, &blinding));
    }
    
    #[test]
    fn test_commitment_fails_wrong_amount() {
        let amount = 100u64;
        let blinding = generate_blinding();
        let commitment = commit(amount, &blinding);
        
        assert!(!verify_commitment(&commitment, 99, &blinding));
    }
    
    #[test]
    fn test_commitment_homomorphic_addition() {
        let b1 = generate_blinding();
        let b2 = generate_blinding();
        
        let c1 = commit(50, &b1);
        let c2 = commit(30, &b2);
        let c_sum = c1.add(&c2);
        
        let combined_blinding = b1 + b2;
        assert!(verify_commitment(&c_sum, 80, &combined_blinding));
    }
    
    #[test]
    fn test_commitment_serialization() {
        let amount = 100u64;
        let blinding = generate_blinding();
        let commitment = commit(amount, &blinding);
        
        let bytes = commitment.to_bytes();
        let restored = PedersenCommitment::from_bytes(&bytes).unwrap();
        
        assert_eq!(commitment.to_bytes(), restored.to_bytes());
    }
}
```

### Ring Signature Tests

```bash
cargo test --package cryptography-crypto ring_signature
```

**Test Cases:**

```rust
#[cfg(test)]
mod ring_signature_tests {
    use super::*;
    use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, scalar::Scalar};
    use rand::rngs::OsRng;
    
    fn generate_keypair() -> (Scalar, RistrettoPoint) {
        let secret = Scalar::random(&mut OsRng);
        let public = secret * RISTRETTO_BASEPOINT_POINT;
        (secret, public)
    }
    
    #[test]
    fn test_ring_signature_verification() {
        let (my_secret, my_public) = generate_keypair();
        
        // Create ring
        let mut ring = vec![];
        for _ in 0..3 {
            let (_, pk) = generate_keypair();
            ring.push(pk);
        }
        ring.insert(1, my_public);  // Insert at position 1
        
        let message = b"test message";
        let signature = sign_ring(message, &my_secret, 1, &ring);
        
        assert!(verify_ring(&signature, message, &ring));
    }
    
    #[test]
    fn test_ring_signature_fails_wrong_message() {
        let (my_secret, my_public) = generate_keypair();
        let ring = vec![my_public];
        
        let signature = sign_ring(b"message 1", &my_secret, 0, &ring);
        
        assert!(!verify_ring(&signature, b"message 2", &ring));
    }
    
    #[test]
    fn test_key_image_consistency() {
        let (my_secret, my_public) = generate_keypair();
        let ring = vec![my_public];
        
        let sig1 = sign_ring(b"msg1", &my_secret, 0, &ring);
        let sig2 = sign_ring(b"msg2", &my_secret, 0, &ring);
        
        // Same key produces same key image
        assert_eq!(
            sig1.key_image.compress().to_bytes(),
            sig2.key_image.compress().to_bytes()
        );
    }
    
    #[test]
    fn test_different_keys_different_images() {
        let (sk1, pk1) = generate_keypair();
        let (sk2, pk2) = generate_keypair();
        
        let sig1 = sign_ring(b"msg", &sk1, 0, &vec![pk1]);
        let sig2 = sign_ring(b"msg", &sk2, 0, &vec![pk2]);
        
        assert_ne!(
            sig1.key_image.compress().to_bytes(),
            sig2.key_image.compress().to_bytes()
        );
    }
}
```

### Stealth Address Tests

```bash
cargo test --package cryptography-crypto ethereum
```

**Test Cases:**

```rust
#[cfg(test)]
mod stealth_tests {
    use super::*;
    
    #[test]
    fn test_stealth_address_generation() {
        let view_kp = EthKeyPair::random().unwrap();
        let spend_kp = EthKeyPair::random().unwrap();
        
        let (stealth, _ephemeral) = generate_stealth_eth(
            &view_kp.public,
            &spend_kp.public
        ).unwrap();
        
        // Address should be 20 bytes
        assert_eq!(stealth.stealth_address.len(), 20);
    }
    
    #[test]
    fn test_stealth_address_scanning() {
        let view_kp = EthKeyPair::random().unwrap();
        let spend_kp = EthKeyPair::random().unwrap();
        
        let (stealth, _) = generate_stealth_eth(
            &view_kp.public,
            &spend_kp.public
        ).unwrap();
        
        // Recipient can scan and find the payment
        let found = scan_stealth_eth(
            &stealth,
            &view_kp.secret,
            &spend_kp.public
        ).unwrap();
        
        assert!(found.is_some());
    }
    
    #[test]
    fn test_stealth_not_found_wrong_keys() {
        let view_kp = EthKeyPair::random().unwrap();
        let spend_kp = EthKeyPair::random().unwrap();
        let wrong_kp = EthKeyPair::random().unwrap();
        
        let (stealth, _) = generate_stealth_eth(
            &view_kp.public,
            &spend_kp.public
        ).unwrap();
        
        // Wrong view key cannot find
        let found = scan_stealth_eth(
            &stealth,
            &wrong_kp.secret,
            &spend_kp.public
        ).unwrap();
        
        assert!(found.is_none());
    }
    
    #[test]
    fn test_address_formatting() {
        let addr: [u8; 20] = [0x42; 20];
        let formatted = format_address(&addr);
        
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42);
    }
}
```

### Curve Bridge Tests

```bash
cargo test --package cryptography-crypto bridge
```

**Test Cases:**

```rust
#[cfg(test)]
mod bridge_tests {
    use super::*;
    
    #[test]
    fn test_secp256k1_to_ristretto_deterministic() {
        let kp = EthKeyPair::random().unwrap();
        
        let point1 = secp256k1_to_ristretto(&kp.public);
        let point2 = secp256k1_to_ristretto(&kp.public);
        
        assert_eq!(
            point1.compress().to_bytes(),
            point2.compress().to_bytes()
        );
    }
    
    #[test]
    fn test_address_to_ristretto_deterministic() {
        let addr: [u8; 20] = [0x42; 20];
        
        let point1 = address_to_ristretto(&addr);
        let point2 = address_to_ristretto(&addr);
        
        assert_eq!(
            point1.compress().to_bytes(),
            point2.compress().to_bytes()
        );
    }
    
    #[test]
    fn test_hash_to_ristretto() {
        let data = b"test data";
        let point = hash_to_ristretto(data);
        
        // Point should be valid
        assert!(point.compress().decompress().is_some());
    }
}
```

## Types Crate Tests

### Transaction Serialization Tests

```bash
cargo test --package cryptography-types
```

**Test Cases:**

```rust
#[cfg(test)]
mod transaction_tests {
    use super::*;
    
    fn create_test_transaction() -> PrivateTransaction {
        PrivateTransaction {
            input_commitments: vec![CommitmentData::new([1u8; 32])],
            output_commitments: vec![
                CommitmentData::new([2u8; 32]),
                CommitmentData::new([3u8; 32]),
            ],
            key_image: [4u8; 32],
            ring: vec![[5u8; 32], [6u8; 32], [7u8; 32]],
            stealth_addresses: vec![],
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
    fn test_transaction_balance() {
        let tx = create_test_transaction();
        
        let input_sum: u64 = tx.input_amounts.iter().sum();
        let output_sum: u64 = tx.output_amounts.iter().sum();
        
        assert_eq!(input_sum, output_sum);
    }
    
    #[test]
    fn test_transaction_serialization() {
        let tx = create_test_transaction();
        
        let serialized = bincode::serialize(&tx).unwrap();
        let deserialized: PrivateTransaction = bincode::deserialize(&serialized).unwrap();
        
        assert_eq!(tx.key_image, deserialized.key_image);
        assert_eq!(tx.input_amounts, deserialized.input_amounts);
    }
}
```

## Prover Tests

### Integration Tests

```bash
cargo test --package cryptography-prover
```

> **Note**: Prover tests require significant resources and may take several minutes.

**Test Cases:**

```rust
#[cfg(test)]
mod prover_tests {
    use super::*;
    
    #[test]
    fn test_transaction_creation() {
        let tx = create_test_transaction();
        
        assert_eq!(tx.input_amounts.len(), 1);
        assert_eq!(tx.output_amounts.len(), 2);
    }
    
    #[test]
    #[ignore]  // Run with: cargo test -- --ignored
    fn test_generate_and_verify_proof() {
        let tx = create_test_transaction();
        
        let proof_data = generate_proof(&tx).expect("Failed to generate proof");
        verify_proof(&proof_data).expect("Failed to verify proof");
    }
}
```

Run ignored tests:

```bash
cargo test --package cryptography-prover -- --ignored
```

## Property-Based Testing

Use `proptest` for property-based testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn commitment_hiding_property(amount in 0u64..1_000_000) {
        let blinding = generate_blinding();
        let commitment = commit(amount, &blinding);
        
        assert!(verify_commitment(&commitment, amount, &blinding));
    }
    
    #[test]
    fn commitment_binding_property(
        amount1 in 0u64..1_000_000,
        amount2 in 0u64..1_000_000,
    ) {
        prop_assume!(amount1 != amount2);
        
        let blinding = generate_blinding();
        let commitment = commit(amount1, &blinding);
        
        // Cannot verify with different amount
        assert!(!verify_commitment(&commitment, amount2, &blinding));
    }
}
```

## Fuzzing

### Setup Fuzzing

```bash
cargo install cargo-fuzz
```

### Fuzz Targets

Create `fuzz/fuzz_targets/commitment.rs`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use cryptography_crypto::*;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 40 {
        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let blinding_bytes: [u8; 32] = data[8..40].try_into().unwrap();
        
        if let Some(blinding) = Scalar::from_bytes_mod_order(blinding_bytes) {
            let commitment = commit(amount, &blinding);
            let _ = commitment.to_bytes();
        }
    }
});
```

### Run Fuzzing

```bash
cargo +nightly fuzz run commitment
```

## Benchmarking

### Setup

```bash
cargo install criterion
```

### Benchmarks

Create `benches/crypto_bench.rs`:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use cryptography_crypto::*;

fn benchmark_commitment(c: &mut Criterion) {
    c.bench_function("pedersen_commit", |b| {
        let blinding = generate_blinding();
        b.iter(|| commit(100, &blinding))
    });
}

fn benchmark_ring_signature(c: &mut Criterion) {
    let (secret, public) = generate_keypair();
    let ring = vec![public; 8];
    let message = b"benchmark message";
    
    c.bench_function("ring_sign_8", |b| {
        b.iter(|| sign_ring(message, &secret, 0, &ring))
    });
    
    let signature = sign_ring(message, &secret, 0, &ring);
    c.bench_function("ring_verify_8", |b| {
        b.iter(|| verify_ring(&signature, message, &ring))
    });
}

criterion_group!(benches, benchmark_commitment, benchmark_ring_signature);
criterion_main!(benches);
```

### Run Benchmarks

```bash
cargo bench
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      
      - name: Run tests
        run: cargo test --all
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
```

## Test Coverage

### Generate Coverage Report

```bash
cargo install cargo-tarpaulin

cargo tarpaulin --out Html
```

### View Report

Open `tarpaulin-report.html` in browser.

## Troubleshooting

### Common Issues

**Issue**: Tests timeout
```
test ... has been running for over 60 seconds
```
**Solution**: Increase timeout or use `#[ignore]`

---

**Issue**: Random test failures
**Solution**: Check for race conditions or use deterministic seeds

```rust
#[test]
fn test_with_seed() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    // Use rng for reproducibility
}
```

---

**Issue**: Proof tests are slow
**Solution**: Use mock prover for unit tests

```rust
#[cfg(test)]
mod tests {
    use sp1_sdk::ProverClient;
    
    fn mock_client() -> ProverClient {
        std::env::set_var("SP1_PROVER", "mock");
        ProverClient::from_env()
    }
}
```
