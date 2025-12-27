# Usage Guide

This guide explains how to use the Gelap Cryptography library to build privacy-preserving transactions.

## Prerequisites

```bash
# Add to Cargo.toml
[dependencies]
cryptography-crypto = { path = "../gelap-cryptography/crypto" }
cryptography-types = { path = "../gelap-cryptography/types" }
```

## Quick Start

### 1. Create a Pedersen Commitment

Hide an amount using a commitment:

```rust
use cryptography_crypto::{commit, generate_blinding, verify_commitment};

fn main() {
    // Amount to hide
    let amount = 100u64;
    
    // Generate random blinding factor
    let blinding = generate_blinding();
    
    // Create commitment
    let commitment = commit(amount, &blinding);
    
    // Verify commitment
    assert!(verify_commitment(&commitment, amount, &blinding));
    
    // Get bytes for storage/transmission
    let commitment_bytes = commitment.to_bytes();
    println!("Commitment: {}", hex::encode(commitment_bytes));
}
```

### 2. Create a Ring Signature

Hide sender identity within an anonymity set:

```rust
use cryptography_crypto::{sign_ring, verify_ring};
use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, scalar::Scalar};
use rand::rngs::OsRng;

fn main() {
    // Generate your keypair
    let secret_key = Scalar::random(&mut OsRng);
    let public_key = secret_key * RISTRETTO_BASEPOINT_POINT;
    
    // Create ring with decoy public keys
    let decoy1 = Scalar::random(&mut OsRng) * RISTRETTO_BASEPOINT_POINT;
    let decoy2 = Scalar::random(&mut OsRng) * RISTRETTO_BASEPOINT_POINT;
    let decoy3 = Scalar::random(&mut OsRng) * RISTRETTO_BASEPOINT_POINT;
    
    // Ring: [decoy1, YOU, decoy2, decoy3]
    let ring = vec![decoy1, public_key, decoy2, decoy3];
    let secret_index = 1; // Your position in ring
    
    // Sign message
    let message = b"Private transfer: 100 tokens";
    let signature = sign_ring(message, &secret_key, secret_index, &ring);
    
    // Anyone can verify, but cannot determine signer
    assert!(verify_ring(&signature, message, &ring));
    
    // Key image for double-spend prevention
    let key_image = signature.key_image;
    println!("Key Image: {:?}", key_image.compress().to_bytes());
}
```

### 3. Generate a Stealth Address

Hide receiver identity:

```rust
use cryptography_crypto::{
    generate_stealth_eth, scan_stealth_eth, 
    format_address, EthKeyPair
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // === RECEIVER SETUP ===
    // Generate view and spend keypairs
    let view_keypair = EthKeyPair::random()?;
    let spend_keypair = EthKeyPair::random()?;
    
    // Publish: view_keypair.public, spend_keypair.public
    
    // === SENDER ACTION ===
    // Create stealth address for receiver
    let (stealth_addr, _ephemeral_secret) = generate_stealth_eth(
        &view_keypair.public,
        &spend_keypair.public
    )?;
    
    // Send funds to this address
    let eth_address = format_address(&stealth_addr.stealth_address);
    println!("Send to: {}", eth_address);
    
    // === RECEIVER SCANNING ===
    // Check if stealth address belongs to you
    let found_key = scan_stealth_eth(
        &stealth_addr,
        &view_keypair.secret,
        &spend_keypair.public
    )?;
    
    if let Some(spending_key) = found_key {
        println!("Found payment! Can spend with key.");
    }
    
    Ok(())
}
```

## Complete Transaction Flow

### Step 1: Build Transaction Data

```rust
use cryptography_crypto::*;
use cryptography_types::*;

fn build_private_transaction() -> PrivateTransaction {
    // === INPUT NOTES (what you're spending) ===
    let input_amount = 100u64;
    let input_blinding = generate_blinding();
    let input_commitment = commit(input_amount, &input_blinding);
    
    // === OUTPUT NOTES (what you're creating) ===
    let output1_amount = 60u64;
    let output1_blinding = generate_blinding();
    let output1_commitment = commit(output1_amount, &output1_blinding);
    
    let output2_amount = 40u64; // Change back to self
    let output2_blinding = generate_blinding();
    let output2_commitment = commit(output2_amount, &output2_blinding);
    
    // === RING SIGNATURE ===
    let secret_key = /* your secret key */;
    let ring = /* fetch ring members from pool */;
    let secret_index = /* your index in ring */;
    
    let message = b"PRIVATE_PAYMENT_TX";
    let signature = sign_ring(message, &secret_key, secret_index, &ring);
    
    // === BUILD TRANSACTION ===
    PrivateTransaction {
        input_commitments: vec![CommitmentData::new(input_commitment.to_bytes())],
        output_commitments: vec![
            CommitmentData::new(output1_commitment.to_bytes()),
            CommitmentData::new(output2_commitment.to_bytes()),
        ],
        key_image: signature.key_image.compress().to_bytes(),
        ring: ring.iter().map(|p| p.compress().to_bytes()).collect(),
        stealth_addresses: vec![/* stealth addresses for outputs */],
        input_amounts: vec![input_amount],
        input_blindings: vec![input_blinding.to_bytes()],
        output_amounts: vec![output1_amount, output2_amount],
        output_blindings: vec![
            output1_blinding.to_bytes(), 
            output2_blinding.to_bytes()
        ],
        ring_signature: RingSignatureData::new(
            signature.c.iter().map(|s| s.to_bytes()).collect(),
            signature.r.iter().map(|s| s.to_bytes()).collect(),
        ),
        secret_index,
    }
}
```

### Step 2: Generate Proof

```rust
use cryptography_prover::generate_proof;

fn generate_transaction_proof(tx: &PrivateTransaction) {
    // Generate SP1 proof
    let proof_data = generate_proof(tx)
        .expect("Failed to generate proof");
    
    // Proof ready for Solidity submission
    let proof_bytes = proof_data.proof;
    let public_inputs = proof_data.public_inputs;
    
    println!("Proof size: {} bytes", proof_bytes.len());
}
```

### Step 3: Submit to Contract

```javascript
// JavaScript/ethers.js example
const proofData = /* from Rust prover */;

// Encode public inputs for Solidity
const publicInputs = ethers.utils.defaultAbiCoder.encode(
    ['bytes32[]', 'bytes32[]', 'bytes32', 'bytes32[]'],
    [
        proofData.public_inputs.input_commitments,
        proofData.public_inputs.output_commitments,
        proofData.public_inputs.key_image,
        proofData.public_inputs.ring
    ]
);

// Submit transaction
const tx = await gelapContract.transact(publicInputs, proofData.proof);
await tx.wait();
```

## Homomorphic Operations

Pedersen commitments support arithmetic operations:

```rust
use cryptography_crypto::{commit, generate_blinding};

// Create commitments to different amounts
let blinding1 = generate_blinding();
let blinding2 = generate_blinding();

let c1 = commit(50, &blinding1);  // Commitment to 50
let c2 = commit(30, &blinding2);  // Commitment to 30

// Add commitments (sum = commitment to 80)
let c_sum = c1.add(&c2);

// Verify: combined blinding is blinding1 + blinding2
let combined_blinding = blinding1 + blinding2;
assert!(verify_commitment(&c_sum, 80, &combined_blinding));

// Subtraction works too
let c_diff = c1.sub(&c2);  // Commitment to 20
```

## Balance Verification

Verify inputs equal outputs (conservation):

```rust
fn verify_balance(
    input_commitments: &[PedersenCommitment],
    output_commitments: &[PedersenCommitment],
) -> bool {
    // Sum all inputs
    let mut input_sum = input_commitments[0].clone();
    for c in &input_commitments[1..] {
        input_sum = input_sum.add(c);
    }
    
    // Sum all outputs
    let mut output_sum = output_commitments[0].clone();
    for c in &output_commitments[1..] {
        output_sum = output_sum.add(c);
    }
    
    // Check equality (requires matching blinding sums)
    input_sum.to_bytes() == output_sum.to_bytes()
}
```

## Key Image Management

Prevent double-spending:

```rust
use std::collections::HashSet;

struct KeyImageRegistry {
    used: HashSet<[u8; 32]>,
}

impl KeyImageRegistry {
    fn new() -> Self {
        Self { used: HashSet::new() }
    }
    
    fn check_and_mark(&mut self, key_image: [u8; 32]) -> Result<(), &str> {
        if self.used.contains(&key_image) {
            return Err("Double spend detected!");
        }
        self.used.insert(key_image);
        Ok(())
    }
}

// Usage
let mut registry = KeyImageRegistry::new();
let key_image = signature.key_image.compress().to_bytes();

registry.check_and_mark(key_image)?;  // First spend: OK
registry.check_and_mark(key_image)?;  // Second spend: ERROR
```

## Curve Bridge Usage

Convert Ethereum data for ZK operations:

```rust
use cryptography_crypto::{secp256k1_to_ristretto, address_to_ristretto};
use secp256k1::PublicKey;

// Convert Ethereum public key
let eth_pubkey: PublicKey = /* from wallet */;
let ristretto_point = secp256k1_to_ristretto(&eth_pubkey);

// Use in ring signature
let ring = vec![
    ristretto_point,  // Converted Ethereum key
    native_ristretto_key1,
    native_ristretto_key2,
];

// Or convert Ethereum address
let eth_address: [u8; 20] = /* 0x... */;
let point = address_to_ristretto(&eth_address);
```

## Error Handling

```rust
use cryptography_crypto::{CryptoError, Result};

fn safe_operation() -> Result<()> {
    // Operations that can fail
    let commitment = PedersenCommitment::from_bytes(&bytes)?;
    
    // Handle specific errors
    match result {
        Ok(value) => println!("Success: {:?}", value),
        Err(CryptoError::InvalidPoint) => println!("Bad curve point"),
        Err(CryptoError::InvalidSignature) => println!("Signature verification failed"),
        Err(e) => println!("Other error: {:?}", e),
    }
    
    Ok(())
}
```

## Performance Tips

### 1. Batch Operations

```rust
// Instead of individual commits
let commitments: Vec<_> = amounts.iter()
    .map(|&amount| {
        let blinding = generate_blinding();
        commit(amount, &blinding)
    })
    .collect();
```

### 2. Reuse Ring Members

```rust
// Cache ring members for multiple transactions
struct RingCache {
    members: Vec<RistrettoPoint>,
}

impl RingCache {
    fn get_ring(&self, my_pubkey: RistrettoPoint, size: usize) -> Vec<RistrettoPoint> {
        // Select random decoys + your key
        let mut ring: Vec<_> = self.members
            .choose_multiple(&mut rand::thread_rng(), size - 1)
            .cloned()
            .collect();
        ring.insert(rand::thread_rng().gen_range(0..size), my_pubkey);
        ring
    }
}
```

### 3. Parallel Proof Generation

```rust
use rayon::prelude::*;

// Generate multiple proofs in parallel
let proofs: Vec<_> = transactions
    .par_iter()
    .map(|tx| generate_proof(tx))
    .collect();
```

## Next Steps

- [Proof Generation Guide](./PROOF_GENERATION.md) - Generate EVM-compatible proofs
- [Testing Guide](./TESTING.md) - Test your implementation
- [Security Considerations](./SECURITY.md) - Security best practices
