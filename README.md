# Gelap Cryptography - Privacy-Preserving Transaction System

A zero-knowledge proof system for private transactions using SP1 zkVM, combining Ethereum compatibility with ZK-efficient cryptography.

## Overview

This system provides **cryptographic primitives** for building privacy-preserving transactions with:

- **Hidden Amounts**: Pedersen commitments on Ristretto curve
- **Hidden Senders**: Ring signatures (LSAG variant)
- **Hidden Receivers**: Stealth addresses (coming soon)
- **ZK Proofs**: SP1-based SNARK proofs for Ethereum verification

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Gelap Cryptography                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  crypto/          → Cryptographic primitives library         │
│  ├── pedersen.rs  → Commitment scheme (hide amounts)         │
│  ├── ring_signature.rs → Ring sigs (hide senders)           │
│  ├── stealth.rs   → Stealth addresses (hide receivers)       │
│  └── utils.rs     → Hash functions & utilities               │
│                                                               │
│  types/           → Shared data structures                   │
│  ├── transaction.rs → Transaction format                     │
│  ├── commitment.rs  → Commitment types                       │
│  └── signature.rs   → Signature types                        │
│                                                               │
│  zkvm/            → SP1 guest program (verification circuit) │
│  └── main.rs      → Runs inside zkVM to verify tx           │
│                                                               │
│  prover/          → SP1 host (proof generation service)      │
│  └── main.rs      → Generates proofs for Solidity           │
│                                                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Solidity Smart  │
                    │    Contracts     │
                    │  (Your Team)     │
                    └──────────────────┘
```

## Features Implemented

### ✅ Pedersen Commitments
Hide transaction amounts while maintaining verifiability.

```rust
use cryptography_crypto::{commit, verify_commitment, generate_blinding};

// Commit to an amount
let amount = 100u64;
let blinding = generate_blinding();
let commitment = commit(amount, &blinding);

// Verify commitment
assert!(verify_commitment(&commitment, amount, &blinding));

// Homomorphic addition (C1 + C2 = C(amount1 + amount2))
let c1 = commit(50, &blinding1);
let c2 = commit(30, &blinding2);
let c_sum = c1.add(&c2); // Commitment to 80
```

### ✅ Ring Signatures
Hide transaction sender within an anonymity set.

```rust
use cryptography_crypto::{sign_ring, verify_ring};
use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, scalar::Scalar};

// Create a ring of public keys
let public_keys = vec![pk1, pk2, pk3, pk4, pk5]; // 5 possible senders

// Sign as one member (index 2) without revealing which one
let secret_key = /* your private key */;
let secret_index = 2;
let message = b"transfer 100 tokens";

let signature = sign_ring(message, &secret_key, secret_index, &public_keys);

// Anyone can verify, but cannot determine which member signed
assert!(verify_ring(&signature, message, &public_keys));

// Key image prevents double-spending (same for all sigs from same key)
let key_image = signature.key_image;
```

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SP1
curl -L https://sp1.succinct.xyz | bash
sp1up
```

### Build Project

```bash
# Clone repository
git clone https://github.com/GelapLabs/gelap-cryptography.git
cd gelap-cryptography

# Build all workspace members
cargo build --release

# Run tests
cargo test --all
```

## Usage for Solidity Integration

### Step 1: Create Private Transaction

```rust
use cryptography_crypto::*;

// 1. Create Pedersen commitment for amount
let amount = 1000u64;
let blinding = generate_blinding();
let commitment = commit(amount, &blinding);

// 2. Create ring signature for sender anonymity
let public_keys = vec![pk1, pk2, pk3]; // Ring members
let signature = sign_ring(
    b"tx_data",
    &my_secret_key,
    1, // My index in ring
    &public_keys
);

// 3. Serialize for proof generation
let commitment_bytes = commitment.to_bytes(); // 32 bytes
let signature_bytes = signature.to_bytes();   // Variable size
let key_image = signature.key_image.compress().to_bytes(); // 32 bytes
```

### Step 2: Generate ZK Proof (Future)

```rust
// This will be implemented in prover/src/main.rs
use cryptography_prover::*;

let proof_input = ProofInput {
    commitment: commitment_bytes,
    signature: signature_bytes,
    key_image,
    public_keys: ring_public_keys,
    // ... other fields
};

// Generate SP1 proof
let proof = generate_proof(proof_input)?;

// Get proof data for Solidity
let proof_bytes = proof.bytes();        // Proof
let public_values = proof.public_values(); // Public inputs
let vkey = proof.verification_key();    // Verification key
```

### Step 3: Verify in Solidity

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface ISP1Verifier {
    function verifyProof(
        bytes32 programVKey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view returns (bool);
}

contract GelapPrivateTransfer {
    ISP1Verifier public verifier;
    bytes32 public immutable PROGRAM_VKEY;

    // Track used key images to prevent double-spending
    mapping(bytes32 => bool) public usedKeyImages;

    // Track commitments (UTXOs)
    mapping(bytes32 => bool) public validCommitments;

    constructor(address _verifier, bytes32 _programVKey) {
        verifier = ISP1Verifier(_verifier);
        PROGRAM_VKEY = _programVKey;
    }

    function processPrivateTransaction(
        bytes32 keyImage,
        bytes32[] calldata newCommitments,
        bytes calldata publicValues,
        bytes calldata proof
    ) external {
        // 1. Check key image not used (prevent double-spend)
        require(!usedKeyImages[keyImage], "Key image already used");

        // 2. Verify ZK proof
        require(
            verifier.verifyProof(PROGRAM_VKEY, publicValues, proof),
            "Invalid proof"
        );

        // 3. Mark key image as used
        usedKeyImages[keyImage] = true;

        // 4. Add new commitments as valid UTXOs
        for (uint i = 0; i < newCommitments.length; i++) {
            validCommitments[newCommitments[i]] = true;
        }

        emit PrivateTransfer(keyImage, newCommitments);
    }

    event PrivateTransfer(bytes32 indexed keyImage, bytes32[] commitments);
}
```

## Proof Generation Workflow

```
┌──────────────────┐
│  User Wallet     │
│  (Off-chain)     │
└────────┬─────────┘
         │
         │ 1. Create tx with commitments & ring sig
         ▼
┌──────────────────┐
│  Prover Service  │
│  (prover/)       │
└────────┬─────────┘
         │
         │ 2. Generate SP1 proof
         │    - Input: tx data
         │    - Circuit: zkvm/
         │    - Output: SNARK proof
         ▼
┌──────────────────┐
│  Smart Contract  │
│  (Solidity)      │
└────────┬─────────┘
         │
         │ 3. Verify proof on-chain
         │    - Check key image unused
         │    - Verify proof
         │    - Update state
         ▼
┌──────────────────┐
│  Transaction     │
│  Confirmed       │
└──────────────────┘
```

## API Reference

### Pedersen Commitments

```rust
// Create commitment
pub fn commit(amount: u64, blinding: &Scalar) -> PedersenCommitment

// Verify commitment
pub fn verify_commitment(
    commitment: &PedersenCommitment,
    amount: u64,
    blinding: &Scalar
) -> bool

// Generate random blinding factor
pub fn generate_blinding() -> Scalar

// Homomorphic operations
impl PedersenCommitment {
    pub fn add(&self, other: &Self) -> Self
    pub fn sub(&self, other: &Self) -> Self
    pub fn to_bytes(&self) -> [u8; 32]
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self>
}
```

### Ring Signatures

```rust
// Sign message with ring
pub fn sign_ring(
    message: &[u8],
    secret_key: &Scalar,
    secret_index: usize,
    public_keys: &[RistrettoPoint]
) -> RingSignature

// Verify ring signature
pub fn verify_ring(
    signature: &RingSignature,
    message: &[u8],
    public_keys: &[RistrettoPoint]
) -> bool

impl RingSignature {
    pub fn to_bytes(&self) -> Vec<u8>
    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
    pub key_image: RistrettoPoint // For double-spend prevention
}
```

### Utilities

```rust
// Hash functions
pub fn hash_sha256(data: &[u8]) -> [u8; 32]
pub fn keccak_256(data: &[u8]) -> [u8; 32]

// Hex conversion
pub fn to_hex(data: &[u8]) -> String
pub fn from_hex(s: &str) -> Result<Vec<u8>, String>

// Random generation
pub fn random_bytes<const N: usize>() -> [u8; N]
```

## Data Types for Solidity

### Commitment (32 bytes)
```
Compressed Ristretto point: bytes32
```

### Key Image (32 bytes)
```
Compressed Ristretto point: bytes32
Used for double-spend prevention
```

### Ring Signature (variable size)
```
struct RingSignature {
    bytes32 keyImage;
    bytes32[] c;  // Challenge scalars (n elements)
    bytes32[] r;  // Response scalars (n elements)
}
// Total size: 32 + 32*n + 32*n bytes for ring of size n
```

## Security Considerations

### For Solidity Developers

1. **Always check key images**: Prevent double-spending by tracking used key images
2. **Validate commitment balance**: Ensure input commitments = output commitments
3. **Store verification key**: Use immutable PROGRAM_VKEY from prover
4. **Gas optimization**: Batch verify multiple proofs when possible
5. **Upgrade path**: Use proxy pattern for verifier contract

### Cryptographic Guarantees

- **Hiding**: Commitments reveal nothing about amounts
- **Binding**: Cannot change committed amount after creation
- **Anonymity**: Ring signatures hide signer in anonymity set
- **Linkability**: Same key creates same key image (prevent double-spend)
- **Unforgeability**: Cannot forge signature without private key

## Testing

```bash
# Run all tests
cargo test --all

# Run specific module tests
cargo test --package cryptography-crypto

# Run with output
cargo test -- --nocapture

# Test ring signatures
cargo test --package cryptography-crypto ring_signature

# Test Pedersen commitments
cargo test --package cryptography-crypto pedersen
```

## Performance

### Pedersen Commitments
- Create: ~50 μs
- Verify: ~50 μs
- Serialize: 32 bytes

### Ring Signatures
- Sign (ring size 8): ~2 ms
- Verify (ring size 8): ~2 ms
- Serialize: 32 + 64*n bytes (n = ring size)

### ZK Proof Generation (Estimated)
- Local proving: ~30-60 seconds
- Network proving: ~10-20 seconds
- Proof size: ~200 KB (Groth16)

## SP1 Proof Generation

### Generate an SP1 Core Proof

```bash
cd prover
cargo run --release -- --prove
```

### Generate an EVM-Compatible Proof

> **Warning**: You will need at least 16GB RAM to generate a Groth16 or PLONK proof.

```bash
# Generate Groth16 proof (smallest, cheapest to verify)
cd prover
cargo run --release --bin evm -- --system groth16

# Generate PLONK proof
cargo run --release --bin evm -- --system plonk
```

### Retrieve the Verification Key

```bash
cd prover
cargo run --release --bin vkey
```

This outputs the `programVKey` needed for your Solidity contract.

### Using the Prover Network

For production use, leverage the [Succinct Prover Network](https://docs.succinct.xyz/docs/network/introduction):

```bash
# Setup environment
cp .env.example .env
# Edit .env and set NETWORK_PRIVATE_KEY

# Generate proof using network
SP1_PROVER=network cargo run --release --bin evm
```

## Roadmap

- [x] Pedersen commitments
- [x] Ring signatures
- [ ] Stealth addresses
- [ ] Ethereum address integration (secp256k1 bridge)
- [ ] SP1 zkVM circuit implementation
- [ ] Prover service with network support
- [ ] Solidity verifier contracts
- [ ] Transaction format & serialization
- [ ] SDK for wallet integration

## Examples

See `examples/` directory (coming soon):
- `private_transfer.rs` - Complete private transaction flow
- `commitment_basics.rs` - Working with commitments
- `ring_signature_demo.rs` - Ring signature usage
- `solidity_integration.rs` - Generate data for contracts

## Contributing

```bash
# Create feature branch
git checkout -b feature/your-feature

# Make changes and test
cargo test --all

# Commit with conventional commits
git commit -m "feat: add stealth address generation"

# Push to dev branch
git push origin dev
```

## License

MIT

## Contact

- GitHub: [GelapLabs/gelap-cryptography](https://github.com/GelapLabs/gelap-cryptography)
- Issues: Report bugs and feature requests

## References

- [SP1 Documentation](https://docs.succinct.xyz/)
- [Curve25519-Dalek](https://github.com/dalek-cryptography/curve25519-dalek)
- [Ring Signatures](https://en.wikipedia.org/wiki/Ring_signature)
- [Pedersen Commitments](https://crypto.stanford.edu/~dabo/pubs/papers/commitments.pdf)
