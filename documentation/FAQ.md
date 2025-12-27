# Frequently Asked Questions

## General Questions

### What is Gelap Cryptography?

Gelap Cryptography is a Rust library providing cryptographic primitives for privacy-preserving transactions on Ethereum. It enables:

- **Hidden amounts** via Pedersen commitments
- **Hidden senders** via ring signatures
- **Hidden receivers** via stealth addresses
- **Verifiable privacy** via ZK proofs (SP1)

### What problem does it solve?

Public blockchains like Ethereum expose all transaction details. Gelap Cryptography provides privacy while maintaining verifiability:

| Without Gelap | With Gelap |
|--------------|------------|
| Amounts visible | Amounts hidden |
| Sender visible | Sender anonymous |
| Receiver visible | Receiver unlinkable |
| Graph traceable | Graph private |

### Is this like Tornado Cash or Zcash?

It shares privacy goals but differs in implementation:

| Feature | Tornado Cash | Zcash | Gelap |
|---------|--------------|-------|-------|
| Fixed denominations | Yes | No | No |
| Sender privacy | Deposit mixing | Shielded | Ring signatures |
| Receiver privacy | Withdraw mixing | Shielded | Stealth addresses |
| Proof system | Groth16 | Groth16/Halo2 | SP1 zkVM |
| EVM compatible | Yes | No | Yes |

---

## Technical Questions

### What curves are used?

**Ristretto255** (based on Curve25519):
- Used for commitments and ring signatures
- Prime-order group, no cofactor issues
- ZK-friendly

**secp256k1**:
- Used for Ethereum compatibility
- Stealth address generation
- Bridged to Ristretto for ZK operations

### What hash functions are used?

- **SHA-512**: Used for Ristretto operations
- **Keccak-256**: Ethereum-compatible hashing
- Domain separation prefixes prevent collisions

### How large are ring signatures?

Ring signature size: `32 + 64n` bytes

| Ring Size | Signature Size |
|-----------|---------------|
| 4 | 288 bytes |
| 8 | 544 bytes |
| 16 | 1,056 bytes |
| 32 | 2,080 bytes |

### How does key image prevent double-spending?

Key images are deterministic for each private key:

```
key_image = secret_key * Hash(public_key)
```

Properties:
- Same key → same image (always)
- Different keys → different images
- Cannot derive public key from image

By tracking used key images on-chain, double-spending is prevented.

---

## Commitment Questions

### What is a Pedersen commitment?

A cryptographic commitment that hides a value while allowing later verification:

```
C = amount * G + blinding * H
```

Where:
- `amount`: The hidden value
- `blinding`: Random scalar (must be kept secret)
- `G, H`: Independent generator points

### Can I change the amount after committing?

No. Pedersen commitments are **binding** - you cannot find a different amount that produces the same commitment.

### Why does homomorphic addition work?

Because elliptic curve arithmetic is linear:

```
C(a,r1) + C(b,r2) = (a*G + r1*H) + (b*G + r2*H)
                  = (a+b)*G + (r1+r2)*H
                  = C(a+b, r1+r2)
```

### What if I lose my blinding factor?

You cannot prove ownership of the commitment. The blinding factor is essential for:
- Verification
- Spending the note
- Generating valid proofs

**Always backup blinding factors securely.**

---

## Ring Signature Questions

### What is the minimum ring size?

Technically 1 (no anonymity), but recommended minimum is **8** for production:

| Ring Size | Anonymity | Use Case |
|-----------|-----------|----------|
| 1 | None | Testing only |
| 2-4 | Low | Not recommended |
| 8 | Good | Minimum for production |
| 16+ | High | Recommended |

### How do I choose decoy public keys?

Best practices:
1. Select randomly from a pool of real keys
2. Include keys from recent transactions
3. Avoid patterns (same height, similar amounts)
4. Never reuse the same decoy set

### Can someone tell I'm part of a ring?

No. Anyone can include any public key in a ring without permission. You cannot prove someone **didn't** participate.

### What if a decoy key is compromised?

The ring signature remains valid. Even if all other keys are compromised, your signature cannot be identified (unless your key is also compromised).

---

## Stealth Address Questions

### How do stealth addresses work?

1. **Receiver** publishes view and spend public keys
2. **Sender** generates ephemeral keypair
3. **Sender** computes shared secret via ECDH
4. **Sender** derives stealth address
5. **Receiver** scans using view key to detect payment

### Do I need to scan all transactions?

Yes, the receiver must scan transactions to detect payments. This can be optimized:
- Light client scanning services
- Bloom filter announcements
- View key delegation

### Can I delegate scanning to a third party?

Yes, by sharing your **view key** (not spend key). The delegate can:
- Detect incoming payments
- Compute payment amounts

But cannot:
- Spend the funds
- Sign transactions

### Are stealth addresses reusable?

You publish the same view/spend public keys. Each payment generates a **unique** stealth address that cannot be linked to your public keys.

---

## Proof Questions

### How long does proof generation take?

| Mode | Hardware | Time |
|------|----------|------|
| Local (Groth16) | 16GB RAM | ~60 seconds |
| Local (Groth16) | 32GB RAM | ~45 seconds |
| Network | Prover Network | ~15 seconds |

### Why use SP1 instead of other proof systems?

SP1 advantages:
- **Rust zkVM**: Write circuits in regular Rust
- **EVM Compatible**: Groth16/PLONK for Ethereum
- **Network Prover**: Outsource computation
- **Updatable**: Program updates without trusted setup

### What does the zkVM circuit verify?

1. Ring signature is valid
2. Sum of inputs equals sum of outputs
3. Each commitment is correctly formed
4. Key image is correctly computed

### Can proofs be batched?

Currently, each transaction requires a separate proof. Batch verification is a potential future optimization.

---

## Integration Questions

### How do I integrate with Solidity?

1. Deploy SP1 verifier contract
2. Get `programVKey` from prover
3. Submit proofs to your contract
4. Contract calls `verifier.verifyProof()`

See [Proof Generation Guide](./PROOF_GENERATION.md) for details.

### What EVM chains are supported?

Any EVM chain with SP1 verifier deployment:
- Ethereum Mainnet
- Polygon
- Arbitrum
- Optimism
- Base
- And more...

### Can I use this with smart contract wallets?

Yes. The cryptography is account-agnostic. Smart contract wallets can:
- Hold stealth address private keys
- Submit proofs
- Manage key images

---

## Security Questions

### Has this been audited?

⚠️ **Not yet audited**. Do not use in production without a professional security audit.

### What are the security assumptions?

1. **ECDLP Hardness**: Cannot solve discrete log on curves
2. **Hash Collision Resistance**: Cannot find collisions
3. **SP1 Soundness**: zkVM proofs are sound
4. **Random Oracle Model**: Hash functions behave randomly

### What about quantum computers?

Current algorithms are not quantum-resistant. Post-quantum alternatives may be added in future versions.

### Can law enforcement trace transactions?

Without access to private keys:
- Cannot determine sender
- Cannot determine receiver
- Cannot determine amounts
- Can only see encrypted data

With access to view keys: Can see incoming payments but not outgoing.

---

## Development Questions

### What Rust version is required?

Stable Rust, version 1.70 or later. Check `rust-toolchain` file for exact version.

### How do I run tests?

```bash
# All tests
cargo test --all

# Specific crate
cargo test --package cryptography-crypto

# With output
cargo test -- --nocapture
```

### Where can I report bugs?

GitHub Issues: [gelap-cryptography/issues](https://github.com/GelapLabs/gelap-cryptography/issues)

For security vulnerabilities, email security@gelap.xyz first.

### How can I contribute?

1. Fork the repository
2. Create feature branch
3. Write tests
4. Submit PR to `dev` branch

See CONTRIBUTING.md for guidelines.

---

## Troubleshooting

### "Invalid signature" error

Common causes:
- Wrong message bytes
- Incorrect key index
- Mismatched ring
- Corrupted signature data

### "Commitment verification failed"

Common causes:
- Wrong blinding factor
- Wrong amount
- Corrupted commitment bytes

### Proof generation out of memory

Solutions:
- Increase RAM (minimum 16GB)
- Use prover network
- Close other applications

### Slow proof generation

Solutions:
- Build with `--release`
- Use CPU with AVX2 support
- Use prover network

---

## Future Plans

### What's on the roadmap?

- [ ] Solidity verifier contracts
- [ ] SDK for wallet integration
- [ ] Batch proof verification
- [ ] Mobile-friendly proofs
- [ ] Post-quantum alternatives

### How can I stay updated?

- Watch GitHub repository
- Join Discord community
- Follow on Twitter
- Subscribe to newsletter

---

## Still Have Questions?

- **Discord**: [Join our community](#)
- **GitHub Discussions**: [Ask questions](#)
- **Email**: support@gelap.xyz
