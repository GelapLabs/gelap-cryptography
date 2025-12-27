# Gelap Cryptography System Design

## Table of Contents

1. [High-Level Overview](#high-level-overview)
2. [System Architecture](#system-architecture)
3. [Component Design](#component-design)
4. [Data Flow](#data-flow)
5. [Cryptographic Design](#cryptographic-design)
6. [zkVM Circuit Design](#zkvm-circuit-design)
7. [Proof System Design](#proof-system-design)
8. [Privacy Design](#privacy-design)
9. [Performance Design](#performance-design)
10. [Security Architecture](#security-architecture)

---

## High-Level Overview

### System Purpose

Gelap Cryptography is a **Rust cryptographic library** providing privacy primitives for anonymous transactions on Ethereum. It enables confidential token transfers by hiding amounts, senders, and receivers using zero-knowledge proofs.

### Key Design Goals

1. **Privacy**: Hide all transaction details from observers
2. **Security**: Cryptographically sound implementations
3. **Verifiability**: Proofs verifiable on-chain
4. **Modularity**: Composable cryptographic components
5. **Performance**: Optimized for zkVM execution
6. **Compatibility**: Bridge between Ethereum and ZK-friendly curves

### System Boundaries

```mermaid
graph TB
    subgraph "External Systems"
        A[User Wallets]
        B[Solidity Contracts]
        C[Ethereum Network]
    end
    
    subgraph "Gelap Cryptography"
        D[crypto/ - Primitives]
        E[types/ - Data Structures]
        F[zkvm/ - Verification Circuit]
        G[prover/ - Proof Generation]
    end
    
    subgraph "Infrastructure"
        H[SP1 zkVM]
        I[SP1 Verifier Contract]
        J[Succinct Prover Network]
    end
    
    A --> D
    D --> E
    E --> F
    F --> G
    G --> H
    G --> J
    B --> I
    I --> C
```

---

## System Architecture

### Workspace Structure

```mermaid
graph TB
    subgraph "Cargo Workspace"
        subgraph "crypto/"
            C1[pedersen.rs]
            C2[ring_signature.rs]
            C3[ethereum.rs]
            C4[bridge.rs]
            C5[utils.rs]
        end
        
        subgraph "types/"
            T1[transaction.rs]
            T2[commitment.rs]
            T3[signature.rs]
            T4[proof.rs]
        end
        
        subgraph "zkvm/"
            Z1[main.rs - Guest Program]
        end
        
        subgraph "prover/"
            P1[lib.rs - Host SDK]
            P2[bin/evm.rs]
            P3[bin/vkey.rs]
        end
    end
    
    C1 --> T2
    C2 --> T3
    C3 --> T1
    C4 --> C1
    C4 --> C3
    
    T1 --> Z1
    T2 --> Z1
    T3 --> Z1
    
    Z1 --> P1
    P1 --> P2
```

### Dependency Graph

```mermaid
graph LR
    subgraph "External Dependencies"
        ED1[curve25519-dalek]
        ED2[secp256k1]
        ED3[k256]
        ED4[sha2/sha3]
        ED5[sp1-zkvm]
        ED6[sp1-sdk]
    end
    
    subgraph "Internal Crates"
        IC1[crypto]
        IC2[types]
        IC3[zkvm]
        IC4[prover]
    end
    
    ED1 --> IC1
    ED2 --> IC1
    ED3 --> IC1
    ED4 --> IC1
    
    IC1 --> IC2
    IC2 --> IC3
    ED5 --> IC3
    
    IC2 --> IC4
    IC3 --> IC4
    ED6 --> IC4
```

### Layered Architecture

```mermaid
graph TB
    subgraph "Layer 1: Application Interface"
        L1A[Wallet Integration]
        L1B[SDK/Library]
        L1C[CLI Tools]
    end
    
    subgraph "Layer 2: Transaction Building"
        L2A[Transaction Builder]
        L2B[Note Manager]
        L2C[Key Manager]
    end
    
    subgraph "Layer 3: Cryptographic Primitives"
        L3A[Pedersen Commitments]
        L3B[Ring Signatures]
        L3C[Stealth Addresses]
        L3D[Curve Bridge]
    end
    
    subgraph "Layer 4: Zero-Knowledge Layer"
        L4A[zkVM Circuit]
        L4B[Proof Generation]
        L4C[Proof Verification]
    end
    
    subgraph "Layer 5: Blockchain Layer"
        L5A[SP1 Verifier]
        L5B[Smart Contracts]
        L5C[EVM Chains]
    end
    
    L1A --> L2A
    L1B --> L2A
    L1C --> L2A
    
    L2A --> L3A
    L2B --> L3A
    L2B --> L3B
    L2C --> L3C
    
    L3A --> L4A
    L3B --> L4A
    L3D --> L4A
    
    L4A --> L4B
    L4B --> L5A
    L5A --> L5B
    L5B --> L5C
```

---

## Component Design

### 1. Crypto Crate

#### Pedersen Commitment Module

**Purpose**: Hide transaction amounts using homomorphic commitments.

```mermaid
graph LR
    A[amount: u64] --> COMMIT
    B[blinding: Scalar] --> COMMIT
    C[G: Generator] --> COMMIT
    D[H: Generator] --> COMMIT
    COMMIT[C = amount*G + blinding*H] --> E[Commitment]
```

**Design Decisions:**
- **Curve**: Ristretto255 for prime-order group
- **Generators**: Deterministic H derived from G
- **Blinding**: 256-bit random scalar
- **Serialization**: Compressed point (32 bytes)

#### Ring Signature Module

**Purpose**: Hide sender identity within an anonymity set.

```mermaid
graph TB
    subgraph "LSAG Signature Generation"
        A[Message] --> S[Sign]
        B[Secret Key] --> S
        C[Ring of Public Keys] --> S
        D[Secret Index] --> S
        S --> E[Key Image]
        S --> F["Challenges c[]"]
        S --> G["Responses r[]"]
    end
    
    subgraph "Verification"
        E --> V[Verify]
        F --> V
        G --> V
        A --> V
        C --> V
        V --> H{Valid?}
    end
```

**Design Decisions:**
- **Scheme**: LSAG (Linkable Spontaneous Anonymous Group)
- **Key Image**: `I = x * Hp(P)` for linkability
- **Challenge**: Hash-based Fiat-Shamir transform
- **Ring Size**: Variable (recommended ≥8)

#### Ethereum Stealth Module

**Purpose**: Hide receiver identity using one-time addresses.

```mermaid
sequenceDiagram
    participant Sender
    participant Shared
    participant Receiver
    
    Note over Receiver: Publishes view_pk, spend_pk
    
    Sender->>Sender: Generate ephemeral keypair (r, R)
    Sender->>Shared: Compute S = r * view_pk
    Sender->>Sender: ss = Hash(S)
    Sender->>Sender: P = spend_pk + ss * G
    Sender->>Sender: stealth_addr = keccak(P)[-20:]
    
    Note over Sender: Sends to stealth_addr, publishes R
    
    Receiver->>Shared: Compute S = view_sk * R
    Receiver->>Receiver: ss = Hash(S)
    Receiver->>Receiver: P = spend_pk + ss * G
    Receiver->>Receiver: Verify: keccak(P)[-20:] == stealth_addr
    Receiver->>Receiver: spending_key = spend_sk + ss
```

**Design Decisions:**
- **Curve**: secp256k1 for Ethereum compatibility
- **Key Derivation**: ECDH with view key
- **Address**: Standard 20-byte Ethereum format
- **Scanning**: O(n) in transactions

#### Bridge Module

**Purpose**: Convert between curves for ZK operations.

```mermaid
graph LR
    subgraph "Ethereum Domain"
        A[secp256k1 PublicKey]
        B[Ethereum Address]
    end
    
    subgraph "Hash to Curve"
        H[SHA-512 + RistrettoPoint::from_uniform_bytes]
    end
    
    subgraph "ZK Domain"
        C[RistrettoPoint]
    end
    
    A --> H
    B --> H
    H --> C
```

**Design Decisions:**
- **Method**: Hash-to-curve (not algebraic)
- **Hash**: SHA-512 for 512-bit output
- **Domain Separation**: Unique prefixes per operation
- **Deterministic**: Same input → same output

### 2. Types Crate

#### Transaction Structure

```mermaid
classDiagram
    class PrivateTransaction {
        +Vec~CommitmentData~ input_commitments
        +Vec~CommitmentData~ output_commitments
        +[u8; 32] key_image
        +Vec~[u8; 32]~ ring
        +Vec~StealthAddressData~ stealth_addresses
        +Vec~u64~ input_amounts
        +Vec~[u8; 32]~ input_blindings
        +Vec~u64~ output_amounts
        +Vec~[u8; 32]~ output_blindings
        +RingSignatureData ring_signature
        +usize secret_index
    }
    
    class CommitmentData {
        +[u8; 32] commitment
        +new(bytes) CommitmentData
    }
    
    class RingSignatureData {
        +Vec~[u8; 32]~ c
        +Vec~[u8; 32]~ r
        +new(c, r) RingSignatureData
    }
    
    class PublicInputs {
        +Vec~[u8; 32]~ input_commitments
        +Vec~[u8; 32]~ output_commitments
        +[u8; 32] key_image
        +Vec~[u8; 32]~ ring
    }
    
    PrivateTransaction --> CommitmentData
    PrivateTransaction --> RingSignatureData
```

**Design Decisions:**
- **Serialization**: Serde + Bincode for zkVM
- **Fixed Sizes**: Arrays where possible for efficiency
- **Separation**: Public vs private data clearly separated

### 3. zkVM Crate

#### Circuit Design

```mermaid
graph TB
    subgraph "Input Parsing"
        A[Read PrivateTransaction from stdin]
    end
    
    subgraph "Ring Signature Verification"
        B[Parse ring members]
        C[Parse signature values]
        D[Verify LSAG signature]
    end
    
    subgraph "Balance Verification"
        E[Sum input amounts]
        F[Sum output amounts]
        G[Assert inputs == outputs]
    end
    
    subgraph "Commitment Verification"
        H[For each input: verify commitment]
        I[For each output: verify commitment]
    end
    
    subgraph "Key Image Validation"
        J[Validate secret_index in range]
    end
    
    subgraph "Output"
        K[Commit PublicInputs]
    end
    
    A --> B
    A --> E
    B --> C
    C --> D
    E --> F
    F --> G
    D --> H
    G --> H
    H --> I
    I --> J
    J --> K
```

### 4. Prover Crate

#### Proof Generation Pipeline

```mermaid
sequenceDiagram
    participant App as Application
    participant Lib as prover/lib.rs
    participant SDK as SP1 SDK
    participant zkVM as SP1 zkVM
    participant Verifier as Verification
    
    App->>Lib: generate_proof(tx)
    Lib->>SDK: ProverClient::from_env()
    Lib->>SDK: setup(ELF)
    SDK-->>Lib: (pk, vk)
    Lib->>SDK: SP1Stdin::write(tx)
    Lib->>SDK: prove(&pk, &stdin)
    SDK->>zkVM: Execute circuit
    zkVM->>zkVM: Verify all constraints
    zkVM-->>SDK: PublicInputs
    SDK->>SDK: Generate SNARK proof
    SDK-->>Lib: SP1ProofWithPublicValues
    Lib-->>App: ProofData { proof, public_inputs }
    
    App->>Lib: verify_proof(proof_data)
    Lib->>SDK: client.verify(&proof, &vk)
    SDK->>Verifier: Check proof validity
    Verifier-->>SDK: Valid/Invalid
    SDK-->>Lib: Result
    Lib-->>App: Ok(()) or Err
```

---

## Data Flow

### Private Transaction Creation

```mermaid
flowchart TD
    A[User initiates transfer] --> B[Select input notes]
    B --> C[Calculate required amount]
    C --> D[Generate output commitments]
    D --> E[Generate blinding factors]
    E --> F[Compute ring signature]
    F --> G[Build PrivateTransaction]
    G --> H[Send to prover]
    H --> I[zkVM verifies]
    I --> J{Valid?}
    J -->|Yes| K[Generate ZK proof]
    J -->|No| L[Return error]
    K --> M[Submit to contract]
    M --> N[On-chain verification]
    N --> O[State updated]
```

### Data Transformation Pipeline

```mermaid
graph LR
    subgraph "User Input"
        A1[Amounts]
        A2[Recipients]
        A3[Keys]
    end
    
    subgraph "Crypto Processing"
        B1[Commitments]
        B2[Ring Signature]
        B3[Stealth Addresses]
    end
    
    subgraph "Transaction"
        C1[PrivateTransaction]
    end
    
    subgraph "Proof"
        D1[PublicInputs]
        D2[ProofBytes]
    end
    
    subgraph "On-Chain"
        E1[Verification]
        E2[State Update]
    end
    
    A1 --> B1
    A3 --> B2
    A2 --> B3
    
    B1 --> C1
    B2 --> C1
    B3 --> C1
    
    C1 --> D1
    C1 --> D2
    
    D1 --> E1
    D2 --> E1
    E1 --> E2
```

---

## Cryptographic Design

### Curve Selection

| Curve | Use Case | Properties |
|-------|----------|------------|
| Ristretto255 | Commitments, Ring Sigs | Prime order, twist-secure |
| secp256k1 | Stealth Addresses | Ethereum native |

### Hash Function Usage

```mermaid
graph TB
    subgraph "Operations"
        A[Pedersen H Generator]
        B[Key Image]
        C[Ring Challenge]
        D[Curve Bridge]
        E[Ethereum Address]
    end
    
    subgraph "Hash Functions"
        H1[SHA-512]
        H2[Keccak-256]
    end
    
    A --> H1
    B --> H1
    C --> H1
    D --> H1
    E --> H2
```

### Domain Separation

| Operation | Domain Prefix |
|-----------|---------------|
| H Generator | `Pedersen_H_GENERATOR_V2` |
| Key Image Hash | `HASH_TO_POINTS_V1` |
| Ring Challenge | `RING_SIG_V1` |
| Stealth ECDH | Native secp256k1 ECDH |

### Generator Points

```rust
// G: Standard Ristretto basepoint
let G = RISTRETTO_BASEPOINT_POINT;

// H: Derived from G (nothing-up-my-sleeve)
let H = SHA512("Pedersen_H_GENERATOR_V2" || G.compress())
    .map_to_ristretto();

// No known discrete log relationship: H = ?*G
```

---

## zkVM Circuit Design

### Constraint System

```mermaid
graph TB
    subgraph "Constraints"
        C1[Ring signature valid]
        C2[sum inputs = sum outputs]
        C3[Each input commitment valid]
        C4[Each output commitment valid]
        C5[secret_index < ring.len]
    end
    
    subgraph "Input Witness"
        W1[PrivateTransaction]
    end
    
    subgraph "Public Output"
        P1[PublicInputs]
    end
    
    W1 --> C1
    W1 --> C2
    W1 --> C3
    W1 --> C4
    W1 --> C5
    
    C1 --> P1
    C2 --> P1
    C3 --> P1
    C4 --> P1
    C5 --> P1
```

### Verification Steps

1. **Ring Signature Verification**
   - Parse compressed points
   - Compute challenge chain
   - Verify response values

2. **Balance Conservation**
   - Sum all input amounts
   - Sum all output amounts
   - Assert equality

3. **Commitment Verification**
   - Recompute each Pedersen commitment
   - Compare with claimed commitment

4. **Key Image Validation**
   - Verify index within bounds
   - (Future: verify key image computation)

### Circuit Complexity

| Operation | Approximate Constraints |
|-----------|------------------------|
| Point decompression | ~1000 |
| Scalar multiplication | ~2000 |
| SHA-512 hash | ~50000 |
| Per ring member | ~5000 |
| Per commitment | ~3000 |

---

## Proof System Design

### Proof Types

```mermaid
graph LR
    subgraph "SP1 Core"
        A[Execution Trace] --> B[Core Proof]
    end
    
    subgraph "EVM Proofs"
        B --> C[Groth16]
        B --> D[PLONK]
    end
    
    subgraph "On-Chain"
        C --> E[~200K gas]
        D --> F[~300K gas]
    end
```

### Proof Generation Modes

```mermaid
graph TB
    subgraph "Local Mode"
        L1[Load ELF] --> L2[Execute in zkVM]
        L2 --> L3[Generate Core Proof]
        L3 --> L4[Wrap to Groth16/PLONK]
    end
    
    subgraph "Network Mode"
        N1[Submit to Network] --> N2[Distributed Execution]
        N2 --> N3[Aggregated Proving]
        N3 --> N4[Return EVM Proof]
    end
```

---

## Privacy Design

### Privacy Properties

```mermaid
graph TB
    subgraph "Hidden Data"
        H1[Transaction Amount]
        H2[Sender Identity]
        H3[Receiver Identity]
        H4[Transaction Graph]
    end
    
    subgraph "Defense Mechanism"
        D1[Pedersen Commitment]
        D2[Ring Signature]
        D3[Stealth Address]
        D4[All Combined]
    end
    
    subgraph "Adversary Knowledge"
        A1[Only sees commitment]
        A2[Only sees ring set]
        A3[Only sees one-time address]
        A4[Cannot link transactions]
    end
    
    H1 --> D1 --> A1
    H2 --> D2 --> A2
    H3 --> D3 --> A3
    H4 --> D4 --> A4
```

### Privacy Leakage Points

| Leak Point | Mitigation |
|------------|------------|
| Timing correlation | Add random delays |
| Amount correlation | Use common denominations |
| Ring reuse | Rotate decoy sets |
| Network metadata | Use Tor/VPN |
| Key image linkability | Inherent (for security) |

### Anonymity Set Size

```
Effective Anonymity = Ring Size × Stealth Factor

Where:
- Ring Size: Number of possible senders
- Stealth Factor: 1 (each address is unique)

Example: Ring of 8 = 1/8 probability of identification
```

---

## Performance Design

### Benchmarks

| Operation | Time | Memory |
|-----------|------|--------|
| Commitment creation | ~50 μs | <1 KB |
| Ring sign (n=8) | ~2 ms | ~10 KB |
| Ring verify (n=8) | ~2 ms | ~10 KB |
| Stealth generate | ~100 μs | ~1 KB |
| Full proof (local) | ~60 s | ~16 GB |
| Full proof (network) | ~15 s | Remote |

### Optimization Strategies

```mermaid
graph TB
    subgraph "Compile Time"
        A[LTO enabled]
        B[Single codegen unit]
        C[Optimized dependencies]
    end
    
    subgraph "Runtime"
        D[Batch operations]
        E[Parallel processing]
        F[Lazy evaluation]
    end
    
    subgraph "Memory"
        G[Stack allocation]
        H[Zero-copy serialization]
        I[Streaming processing]
    end
```

### Resource Requirements

| Component | Min RAM | Recommended RAM |
|-----------|---------|-----------------|
| Library usage | 256 MB | 1 GB |
| Local proving (Core) | 8 GB | 16 GB |
| Local proving (Groth16) | 16 GB | 32 GB |
| Network proving | 1 GB | 4 GB (client) |

---

## Security Architecture

### Threat Model

```mermaid
graph TB
    subgraph "Threats"
        T1[Key Extraction]
        T2[Double Spending]
        T3[Proof Forgery]
        T4[Deanonymization]
        T5[Front-running]
    end
    
    subgraph "Defenses"
        D1[ECDLP Hardness]
        D2[Key Images]
        D3[ZK Soundness]
        D4[Ring Signatures + Stealth]
        D5[Receiver in Proof]
    end
    
    T1 --> D1
    T2 --> D2
    T3 --> D3
    T4 --> D4
    T5 --> D5
```

### Security Assumptions

1. **Discrete Log Hardness**: Cannot solve DL on Ristretto or secp256k1
2. **Hash Collision Resistance**: SHA-512 and Keccak-256 are secure
3. **Random Oracle Model**: Hash functions behave randomly
4. **SP1 Soundness**: zkVM proofs are computationally sound

### Attack Surface

| Surface | Risk | Mitigation |
|---------|------|------------|
| RNG | High | Use OsRng exclusively |
| Serialization | Medium | Validate all inputs |
| Side channels | Medium | Constant-time ops |
| Dependencies | Low | Audit, pin versions |

### Cryptographic Audit Scope

- [ ] Pedersen commitment implementation
- [ ] Ring signature implementation
- [ ] Key image derivation
- [ ] Stealth address ECDH
- [ ] Hash-to-curve implementation
- [ ] Scalar/point serialization
- [ ] zkVM circuit constraints

---

## Appendix

### A. Data Sizes

| Type | Size (bytes) |
|------|--------------|
| Scalar | 32 |
| RistrettoPoint (compressed) | 32 |
| Commitment | 32 |
| Key Image | 32 |
| Ring Signature (n members) | 32 + 64n |
| Ethereum Address | 20 |
| secp256k1 Public Key | 33 |
| PrivateTransaction (typical) | ~500-2000 |
| PublicInputs | ~200-500 |
| Groth16 Proof | ~200,000 |

### B. Configuration Constants

```rust
// Tree depth (from contracts)
const MERKLE_TREE_DEPTH: usize = 32;

// Minimum ring size
const MIN_RING_SIZE: usize = 2;

// Recommended ring size
const RECOMMENDED_RING_SIZE: usize = 8;

// Hash domain separators
const PEDERSEN_H_DOMAIN: &[u8] = b"Pedersen_H_GENERATOR_V2";
const RING_SIG_DOMAIN: &[u8] = b"RING_SIG_V1";
const HASH_TO_POINT_DOMAIN: &[u8] = b"HASH_TO_POINTS_V1";
```

### C. Error Codes

| Code | Description |
|------|-------------|
| `InvalidPoint` | Curve point decompression failed |
| `InvalidScalar` | Scalar out of range |
| `InvalidSignature` | Ring signature verification failed |
| `BalanceMismatch` | Input/output amounts don't match |
| `InvalidIndex` | Secret index out of bounds |
| `ProofGenerationFailed` | SP1 proof generation error |
| `ProofVerificationFailed` | SP1 proof verification error |
