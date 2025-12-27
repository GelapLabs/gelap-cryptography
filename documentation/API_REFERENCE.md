# API Reference

## Crypto Crate (`cryptography-crypto`)

### Pedersen Commitments

#### `commit`

Creates a Pedersen commitment for a given amount.

```rust
pub fn commit(amount: u64, blinding: &Scalar) -> PedersenCommitment
```

**Parameters:**
- `amount`: The value to commit to (u64)
- `blinding`: Random blinding factor (Scalar)

**Returns:**
- `PedersenCommitment`: The resulting commitment

**Example:**
```rust
use cryptography_crypto::{commit, generate_blinding};

let amount = 100u64;
let blinding = generate_blinding();
let commitment = commit(amount, &blinding);
```

---

#### `verify_commitment`

Verifies that a commitment corresponds to a given amount and blinding factor.

```rust
pub fn verify_commitment(
    commitment: &PedersenCommitment,
    amount: u64,
    blinding: &Scalar
) -> bool
```

**Parameters:**
- `commitment`: The commitment to verify
- `amount`: Expected amount
- `blinding`: Expected blinding factor

**Returns:**
- `bool`: `true` if commitment matches, `false` otherwise

**Example:**
```rust
use cryptography_crypto::{commit, verify_commitment, generate_blinding};

let amount = 100u64;
let blinding = generate_blinding();
let commitment = commit(amount, &blinding);

assert!(verify_commitment(&commitment, amount, &blinding));
```

---

#### `generate_blinding`

Generates a cryptographically secure random blinding factor.

```rust
pub fn generate_blinding() -> Scalar
```

**Returns:**
- `Scalar`: Random scalar for use as blinding factor

**Example:**
```rust
use cryptography_crypto::generate_blinding;

let blinding = generate_blinding();
// Use blinding in commitment creation
```

---

#### `PedersenCommitment`

A Pedersen commitment structure.

```rust
pub struct PedersenCommitment {
    pub point: RistrettoPoint,
}

impl PedersenCommitment {
    /// Add two commitments (homomorphic addition)
    pub fn add(&self, other: &Self) -> Self
    
    /// Subtract two commitments
    pub fn sub(&self, other: &Self) -> Self
    
    /// Convert to 32-byte representation
    pub fn to_bytes(&self) -> [u8; 32]
    
    /// Create from bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self>
}
```

**Homomorphic Properties:**
```rust
let c1 = commit(50, &b1);
let c2 = commit(30, &b2);
let c_sum = c1.add(&c2); // Commitment to 80
```

---

### Ring Signatures

#### `sign_ring`

Creates an LSAG ring signature.

```rust
pub fn sign_ring(
    message: &[u8],
    secret_key: &Scalar,
    secret_index: usize,
    public_keys: &[RistrettoPoint]
) -> RingSignature
```

**Parameters:**
- `message`: Message to sign
- `secret_key`: Signer's private key
- `secret_index`: Position of signer's public key in ring
- `public_keys`: Ring of public keys (anonymity set)

**Returns:**
- `RingSignature`: The linkable ring signature with key image

**Example:**
```rust
use cryptography_crypto::sign_ring;
use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, scalar::Scalar};

let secret_key = Scalar::random(&mut rand::thread_rng());
let public_key = secret_key * RISTRETTO_BASEPOINT_POINT;

let ring = vec![pk1, pk2, public_key, pk3, pk4];
let message = b"transfer 100 tokens";

let signature = sign_ring(message, &secret_key, 2, &ring);
```

---

#### `verify_ring`

Verifies an LSAG ring signature.

```rust
pub fn verify_ring(
    signature: &RingSignature,
    message: &[u8],
    public_keys: &[RistrettoPoint]
) -> bool
```

**Parameters:**
- `signature`: The ring signature to verify
- `message`: Original signed message
- `public_keys`: Ring of public keys

**Returns:**
- `bool`: `true` if signature is valid

**Example:**
```rust
use cryptography_crypto::{sign_ring, verify_ring};

let signature = sign_ring(message, &secret_key, index, &ring);
assert!(verify_ring(&signature, message, &ring));
```

---

#### `RingSignature`

Ring signature data structure.

```rust
pub struct RingSignature {
    pub key_image: RistrettoPoint,  // For double-spend prevention
    pub c: Vec<Scalar>,              // Challenge values
    pub r: Vec<Scalar>,              // Response values
}

impl RingSignature {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8>
    
    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self>
}
```

**Key Image Usage:**
```rust
// Key image is deterministic for same key
// Use it to track spent notes
let key_image = signature.key_image;
if used_key_images.contains(&key_image) {
    return Err("Double spend detected");
}
used_key_images.insert(key_image);
```

---

### Ethereum Stealth Addresses

#### `EthKeyPair`

Ethereum key pair for stealth operations.

```rust
pub struct EthKeyPair {
    pub secret: SecretKey,
    pub public: PublicKey,
}

impl EthKeyPair {
    /// Generate random key pair
    pub fn random() -> Result<Self>
    
    /// Create from existing secret key
    pub fn from_secret(secret: SecretKey) -> Self
}
```

---

#### `generate_stealth_eth`

Generates a stealth address for a recipient.

```rust
pub fn generate_stealth_eth(
    recipient_view_pubkey: &PublicKey,
    recipient_spend_pubkey: &PublicKey,
) -> Result<(StealthAddressEth, SecretKey)>
```

**Parameters:**
- `recipient_view_pubkey`: Recipient's view public key
- `recipient_spend_pubkey`: Recipient's spend public key

**Returns:**
- `StealthAddressEth`: The generated stealth address data
- `SecretKey`: Ephemeral secret (sender keeps this)

**Example:**
```rust
use cryptography_crypto::{generate_stealth_eth, EthKeyPair};

let view_keypair = EthKeyPair::random()?;
let spend_keypair = EthKeyPair::random()?;

let (stealth_addr, ephemeral_secret) = generate_stealth_eth(
    &view_keypair.public,
    &spend_keypair.public
)?;
```

---

#### `scan_stealth_eth`

Scans for stealth payments belonging to a recipient.

```rust
pub fn scan_stealth_eth(
    stealth_addr: &StealthAddressEth,
    view_secret: &SecretKey,
    spend_pubkey: &PublicKey,
) -> Result<Option<SecretKey>>
```

**Parameters:**
- `stealth_addr`: The stealth address to check
- `view_secret`: Recipient's view secret key
- `spend_pubkey`: Recipient's spend public key

**Returns:**
- `Option<SecretKey>`: The spending key if address belongs to recipient

**Example:**
```rust
use cryptography_crypto::scan_stealth_eth;

let found_key = scan_stealth_eth(
    &stealth_addr,
    &view_keypair.secret,
    &spend_keypair.public
)?;

if let Some(spending_key) = found_key {
    // This stealth address belongs to us!
    // Use spending_key to claim funds
}
```

---

#### `StealthAddressEth`

Stealth address data structure.

```rust
pub struct StealthAddressEth {
    pub stealth_address: EthAddress,      // 20-byte Ethereum address
    pub ephemeral_pubkey: PublicKey,      // For recipient scanning
}
```

---

#### Utility Functions

```rust
/// Convert public key to Ethereum address
pub fn pubkey_to_address(pubkey: &PublicKey) -> EthAddress

/// Format address as hex string (0x...)
pub fn format_address(address: &EthAddress) -> String

/// Parse address from hex string
pub fn parse_address(s: &str) -> Result<EthAddress>

/// Format with EIP-55 checksum
pub fn checksum_address(address: &EthAddress) -> String
```

---

### Curve Bridge

#### `secp256k1_to_ristretto`

Converts secp256k1 public key to Ristretto point.

```rust
pub fn secp256k1_to_ristretto(pubkey: &PublicKey) -> RistrettoPoint
```

**Parameters:**
- `pubkey`: secp256k1 public key

**Returns:**
- `RistrettoPoint`: Equivalent point on Ristretto curve

**Example:**
```rust
use cryptography_crypto::secp256k1_to_ristretto;
use secp256k1::PublicKey;

let eth_pubkey: PublicKey = /* ... */;
let ristretto_point = secp256k1_to_ristretto(&eth_pubkey);
```

---

#### `address_to_ristretto`

Converts Ethereum address to Ristretto point.

```rust
pub fn address_to_ristretto(address: &EthAddress) -> RistrettoPoint
```

**Parameters:**
- `address`: 20-byte Ethereum address

**Returns:**
- `RistrettoPoint`: Derived Ristretto point

---

#### `hash_to_ristretto`

Hashes arbitrary data to a Ristretto point.

```rust
pub fn hash_to_ristretto(data: &[u8]) -> RistrettoPoint
```

**Parameters:**
- `data`: Arbitrary bytes to hash

**Returns:**
- `RistrettoPoint`: Deterministic point derived from data

---

### Utilities

#### Hash Functions

```rust
/// SHA-256 hash
pub fn hash_sha256(data: &[u8]) -> [u8; 32]

/// Keccak-256 hash (Ethereum compatible)
pub fn hash_keccak256(data: &[u8]) -> [u8; 32]
```

#### Hex Conversion

```rust
/// Convert bytes to hex string
pub fn to_hex(data: &[u8]) -> String

/// Parse hex string to bytes
pub fn from_hex(s: &str) -> Result<Vec<u8>, String>
```

#### Random Generation

```rust
/// Generate N random bytes
pub fn random_bytes<const N: usize>() -> [u8; N]
```

---

## Types Crate (`cryptography-types`)

### PrivateTransaction

Complete transaction data for zkVM verification.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrivateTransaction {
    /// Input note commitments being spent
    pub input_commitments: Vec<CommitmentData>,
    
    /// Output note commitments being created
    pub output_commitments: Vec<CommitmentData>,
    
    /// Key image for double-spend prevention
    pub key_image: [u8; 32],
    
    /// Ring of public keys for anonymity
    pub ring: Vec<[u8; 32]>,
    
    /// Stealth addresses for receivers
    pub stealth_addresses: Vec<StealthAddressData>,
    
    /// Input amounts (private)
    pub input_amounts: Vec<u64>,
    
    /// Input blinding factors (private)
    pub input_blindings: Vec<[u8; 32]>,
    
    /// Output amounts (private)
    pub output_amounts: Vec<u64>,
    
    /// Output blinding factors (private)
    pub output_blindings: Vec<[u8; 32]>,
    
    /// Ring signature proving ownership
    pub ring_signature: RingSignatureData,
    
    /// Index of real signer in ring (private)
    pub secret_index: usize,
}
```

---

### CommitmentData

Serializable commitment representation.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitmentData {
    pub commitment: [u8; 32],
}

impl CommitmentData {
    pub fn new(commitment: [u8; 32]) -> Self
}
```

---

### RingSignatureData

Serializable ring signature representation.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RingSignatureData {
    pub c: Vec<[u8; 32]>,  // Challenge values
    pub r: Vec<[u8; 32]>,  // Response values
}

impl RingSignatureData {
    pub fn new(c: Vec<[u8; 32]>, r: Vec<[u8; 32]>) -> Self
}
```

---

### PublicInputs

Public values output by zkVM.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicInputs {
    pub input_commitments: Vec<[u8; 32]>,
    pub output_commitments: Vec<[u8; 32]>,
    pub key_image: [u8; 32],
    pub ring: Vec<[u8; 32]>,
}
```

---

### ProofData

Complete proof data for verification.

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct ProofData {
    pub proof: Vec<u8>,
    pub public_inputs: PublicInputs,
}
```

---

### StealthAddressData

Serializable stealth address.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StealthAddressData {
    pub ephemeral_pubkey: Vec<u8>,
    pub stealth_address: [u8; 20],
}

impl StealthAddressData {
    pub fn new(ephemeral_pubkey: Vec<u8>, stealth_address: [u8; 20]) -> Self
}
```

---

## Prover Crate (`cryptography-prover`)

### generate_proof

Generates a ZK proof for a private transaction.

```rust
pub fn generate_proof(tx: &PrivateTransaction) -> Result<ProofData>
```

**Parameters:**
- `tx`: The private transaction to prove

**Returns:**
- `ProofData`: Generated proof and public inputs

**Example:**
```rust
use cryptography_prover::generate_proof;
use cryptography_types::transaction::PrivateTransaction;

let tx = PrivateTransaction { /* ... */ };
let proof_data = generate_proof(&tx)?;
```

---

### verify_proof

Verifies a generated proof locally.

```rust
pub fn verify_proof(proof_data: &ProofData) -> Result<()>
```

**Parameters:**
- `proof_data`: The proof to verify

**Returns:**
- `Result<()>`: Ok if valid, Err otherwise

---

### get_verifying_key

Retrieves the verification key for Solidity contracts.

```rust
pub fn get_verifying_key() -> Result<Vec<u8>>
```

**Returns:**
- `Vec<u8>`: Serialized verification key

---

## Error Types

### CryptoError

Errors from the crypto crate.

```rust
pub enum CryptoError {
    InvalidPoint,
    InvalidScalar,
    InvalidSignature,
    InvalidProof,
    DeserializationError(String),
    Other(String),
}
```

---

## Data Sizes

| Type | Size (bytes) |
|------|--------------|
| Commitment | 32 |
| Key Image | 32 |
| Scalar | 32 |
| Ristretto Point | 32 |
| Ethereum Address | 20 |
| secp256k1 Public Key | 33 (compressed) |
| Ring Signature (n members) | 32 + 64n |
| SP1 Proof (Groth16) | ~200,000 |
