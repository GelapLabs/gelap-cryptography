# Proof Generation Guide

This guide explains how to generate ZK proofs for Ethereum verification using the SP1 prover.

## Prerequisites

```bash
# Install SP1
curl -L https://sp1.succinct.xyz | bash
sp1up

# Verify installation
sp1 --version
```

## Quick Start

### Generate a Core SP1 Proof

```bash
cd prover
cargo run --release -- --prove
```

### Generate an EVM-Compatible Proof

> **Warning**: You will need at least 16GB RAM to generate a Groth16 or PLONK proof.

```bash
# Groth16 (smallest, cheapest to verify on-chain)
cd prover
cargo run --release --bin evm -- --system groth16

# PLONK (alternative)
cargo run --release --bin evm -- --system plonk
```

### Retrieve Verification Key

```bash
cd prover
cargo run --release --bin vkey
```

This outputs the `programVKey` needed for your Solidity contract.

## Proof Systems Comparison

| System | Proof Size | Verification Gas | Setup | Best For |
|--------|------------|------------------|-------|----------|
| **Groth16** | ~200 KB | ~200K gas | Trusted | Production |
| **PLONK** | ~500 KB | ~300K gas | Universal | Flexibility |
| **SP1 Core** | ~1 MB | N/A | None | Development |

## Programmatic Proof Generation

### Basic Usage

```rust
use cryptography_prover::{generate_proof, verify_proof, get_verifying_key};
use cryptography_types::transaction::PrivateTransaction;

fn generate_and_verify() -> Result<(), Box<dyn std::error::Error>> {
    // Create transaction
    let tx = create_transaction();
    
    // Generate proof
    println!("Generating proof...");
    let proof_data = generate_proof(&tx)?;
    println!("Proof size: {} bytes", proof_data.proof.len());
    
    // Verify locally
    println!("Verifying proof...");
    verify_proof(&proof_data)?;
    println!("Proof verified!");
    
    // Get verification key
    let vkey = get_verifying_key()?;
    println!("VKey hash: {}", hex::encode(&vkey[..32]));
    
    Ok(())
}
```

### EVM Proof Generation

```rust
use sp1_sdk::{ProverClient, SP1Stdin};
use cryptography_prover::ELF;

fn generate_evm_proof(tx: &PrivateTransaction) -> Result<EvmProof, Error> {
    let client = ProverClient::from_env();
    
    // Prepare inputs
    let mut stdin = SP1Stdin::new();
    stdin.write(tx);
    
    // Setup proving/verification keys
    let (pk, vk) = client.setup(ELF);
    
    // Generate Groth16 proof
    let proof = client.prove(&pk, &stdin).groth16().run()?;
    
    // Extract for Solidity
    Ok(EvmProof {
        vkey: vk.bytes32(),
        public_values: proof.public_values.to_vec(),
        proof_bytes: proof.bytes(),
    })
}

struct EvmProof {
    vkey: String,
    public_values: Vec<u8>,
    proof_bytes: Vec<u8>,
}
```

## Using the Prover Network

For production, use the [Succinct Prover Network](https://docs.succinct.xyz/docs/network/introduction).

### Setup

```bash
# Copy environment template
cp .env.example .env

# Edit .env and set your private key
NETWORK_PRIVATE_KEY=0x...
```

### Generate Proof via Network

```bash
# Set prover to network mode
SP1_PROVER=network cargo run --release --bin evm
```

### Programmatic Network Usage

```rust
use sp1_sdk::{ProverClient, SP1ProverKind};

fn generate_network_proof(tx: &PrivateTransaction) -> Result<ProofData> {
    // Use network prover
    std::env::set_var("SP1_PROVER", "network");
    
    let client = ProverClient::from_env();
    let (pk, _vk) = client.setup(ELF);
    
    let mut stdin = SP1Stdin::new();
    stdin.write(tx);
    
    // Network proof generation
    let proof = client.prove(&pk, &stdin).groth16().run()?;
    
    Ok(ProofData {
        proof: proof.bytes(),
        public_inputs: proof.public_values.read(),
    })
}
```

## Output Fixtures for Testing

Generate JSON fixtures for Solidity contract testing:

```rust
use serde::{Serialize, Deserialize};
use std::fs::File;

#[derive(Serialize)]
struct ProofFixture {
    vkey: String,
    public_inputs: String,
    proof: String,
}

fn save_fixture(proof_data: &EvmProof, path: &str) {
    let fixture = ProofFixture {
        vkey: proof_data.vkey.clone(),
        public_inputs: hex::encode(&proof_data.public_values),
        proof: hex::encode(&proof_data.proof_bytes),
    };
    
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &fixture)?;
}
```

### Fixture Format

```json
{
    "vkey": "0x1234...5678",
    "public_inputs": "0xabcd...ef01",
    "proof": "0x9876...5432"
}
```

## Integration with Solidity

### Contract Interface

```solidity
interface ISP1Verifier {
    function verifyProof(
        bytes32 programVKey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view returns (bool);
}
```

### Using Proof Data

```solidity
contract GelapVerifier {
    ISP1Verifier public verifier;
    bytes32 public immutable PROGRAM_VKEY;
    
    constructor(address _verifier, bytes32 _vkey) {
        verifier = ISP1Verifier(_verifier);
        PROGRAM_VKEY = _vkey;
    }
    
    function verifyTransaction(
        bytes calldata publicInputs,
        bytes calldata proof
    ) external view returns (bool) {
        return verifier.verifyProof(PROGRAM_VKEY, publicInputs, proof);
    }
}
```

### JavaScript Integration

```javascript
const { ethers } = require('ethers');

async function submitProof(contract, proofData) {
    // Parse fixture
    const fixture = JSON.parse(fs.readFileSync('proof_fixture.json'));
    
    // Submit to contract
    const tx = await contract.transact(
        '0x' + fixture.public_inputs,
        '0x' + fixture.proof
    );
    
    await tx.wait();
    console.log('Transaction confirmed:', tx.hash);
}
```

## Performance Optimization

### Local Proving

| RAM | Groth16 Time | PLONK Time |
|-----|--------------|------------|
| 16 GB | ~60 seconds | ~90 seconds |
| 32 GB | ~45 seconds | ~70 seconds |
| 64 GB | ~30 seconds | ~50 seconds |

### Tips

1. **Use Release Mode**: Always build with `--release`
2. **Increase Stack Size**: `RUST_MIN_STACK=16777216`
3. **Enable AVX**: Modern CPUs accelerate proof generation
4. **Parallel Compilation**: `CARGO_BUILD_JOBS=8`

```bash
# Optimized build
RUST_MIN_STACK=16777216 cargo run --release --bin evm
```

## zkVM Circuit Details

### What the Circuit Verifies

```rust
// Inside zkvm/src/main.rs
pub fn main() {
    let tx: PrivateTransaction = sp1_zkvm::io::read();
    
    // 1. Ring Signature Verification
    assert!(verify_ring_signature(&tx), "Invalid ring signature");
    
    // 2. Balance Conservation
    let input_sum: u64 = tx.input_amounts.iter().sum();
    let output_sum: u64 = tx.output_amounts.iter().sum();
    assert_eq!(input_sum, output_sum, "Unbalanced transaction");
    
    // 3. Commitment Verification
    for (i, amount) in tx.input_amounts.iter().enumerate() {
        let computed = pedersen_commitment(*amount, &tx.input_blindings[i]);
        assert_eq!(computed, tx.input_commitments[i]);
    }
    
    // 4. Key Image Validation
    assert!(tx.secret_index < tx.ring.len());
    
    // 5. Output Public Inputs
    sp1_zkvm::io::commit(&public_inputs);
}
```

### Public Inputs Structure

```rust
pub struct PublicInputs {
    pub input_commitments: Vec<[u8; 32]>,  // Spent notes
    pub output_commitments: Vec<[u8; 32]>, // New notes
    pub key_image: [u8; 32],               // Double-spend prevention
    pub ring: Vec<[u8; 32]>,               // Anonymity set
}
```

## Troubleshooting

### Common Issues

**Issue**: Out of memory during proof generation
```
Error: memory allocation of X bytes failed
```
**Solution**: 
- Increase system RAM to minimum 16GB
- Close other applications
- Use network prover instead

---

**Issue**: Proof verification fails on-chain
```
Error: "Invalid proof"
```
**Solution**:
- Verify `programVKey` matches deployed verifier
- Check ABI encoding of public inputs
- Ensure correct proof system (Groth16 vs PLONK)

---

**Issue**: SP1 SDK not found
```
Error: sp1-sdk not found
```
**Solution**:
```bash
sp1up
source ~/.bashrc  # or restart terminal
```

---

**Issue**: Slow proof generation
**Solution**:
- Use `--release` flag
- Check CPU supports AVX2
- Consider using prover network

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SP1_PROVER` | Prover mode (local/network) | local |
| `NETWORK_PRIVATE_KEY` | Network prover key | - |
| `RUST_MIN_STACK` | Minimum stack size | 8MB |
| `SP1_GPU` | Enable GPU acceleration | false |

## Next Steps

- [Testing Guide](./TESTING.md) - Test proof generation
- [Security Considerations](./SECURITY.md) - Proof security
- [API Reference](./API_REFERENCE.md) - Prover API details
