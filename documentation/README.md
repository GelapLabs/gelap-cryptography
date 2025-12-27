# Gelap Cryptography Documentation

Welcome to the Gelap Cryptography documentation! This library provides cryptographic primitives for building privacy-preserving transactions using SP1 zkVM.

## 📚 Documentation Index

### Getting Started
- [Architecture Overview](./ARCHITECTURE.md) - System design and core components
- [System Design](./SYSTEM_DESIGN.md) - Detailed technical design document
- [API Reference](./API_REFERENCE.md) - Complete Rust API documentation

### Development
- [Usage Guide](./USAGE_GUIDE.md) - Build privacy-preserving transactions
- [Testing Guide](./TESTING.md) - Test the cryptographic primitives
- [Proof Generation](./PROOF_GENERATION.md) - Generate ZK proofs for Ethereum

### Additional Resources
- [Security Considerations](./SECURITY.md) - Security best practices
- [FAQ](./FAQ.md) - Frequently asked questions

## 🚀 Quick Start

### 1. Clone and Install

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SP1
curl -L https://sp1.succinct.xyz | bash
sp1up

# Clone repository
git clone https://github.com/GelapLabs/gelap-cryptography.git
cd gelap-cryptography
```

### 2. Build Project

```bash
# Build all workspace members
cargo build --release
```

### 3. Run Tests

```bash
cargo test --all
```

## 🏗️ Project Structure

```
gelap-cryptography/
├── crypto/               # Core cryptographic primitives
│   ├── pedersen.rs       # Commitment scheme (hide amounts)
│   ├── ring_signature.rs # Ring sigs (hide senders)
│   ├── ethereum.rs       # Stealth addresses (hide receivers)
│   ├── bridge.rs         # Curve conversions (secp256k1↔Ristretto)
│   ├── zkproof.rs        # Unified ZK primitives exports
│   └── utils.rs          # Hash functions & utilities
├── types/                # Shared data structures
│   ├── transaction.rs    # Transaction format
│   ├── commitment.rs     # Commitment types
│   └── signature.rs      # Signature types
├── zkvm/                 # SP1 guest program (verification circuit)
│   └── main.rs           # Runs inside zkVM to verify tx
├── prover/               # SP1 host (proof generation service)
│   └── main.rs           # Generates proofs for Solidity
└── documentation/        # Documentation
```

## 🔑 Key Features

- **Hidden Amounts**: Pedersen commitments on Ristretto curve
- **Hidden Senders**: Ring signatures (LSAG variant)
- **Hidden Receivers**: Stealth addresses for Ethereum (secp256k1)
- **Curve Bridge**: Convert between secp256k1 and Ristretto curves
- **ZK Proofs**: SP1-based SNARK proofs for Ethereum verification

## 📖 Core Concepts

### Pedersen Commitments
Hide transaction amounts while maintaining verifiability. Commitments are:
- **Hiding**: Amount is hidden from observers
- **Binding**: Cannot change committed amount
- **Homomorphic**: Supports add/subtract operations

### Ring Signatures
Hide transaction sender within an anonymity set. Provides:
- **Anonymity**: Signer hidden among ring members
- **Linkability**: Key images prevent double-spending
- **Unforgeability**: Cannot forge without private key

### Stealth Addresses
Hide transaction receiver using one-time addresses:
- **Unlinkable**: Each payment gets unique address
- **Scannable**: Recipient can detect payments
- **EVM Compatible**: Works with Ethereum addresses

### zkVM Verification
SP1 program that verifies:
- Ring signature validity
- Commitment balance (inputs = outputs)
- Key image correctness

## 🔗 Links

- **GitHub**: [GelapLabs/gelap-cryptography](https://github.com/GelapLabs/gelap-cryptography)
- **SP1 Docs**: [docs.succinct.xyz](https://docs.succinct.xyz/)
- **Curve25519-Dalek**: [dalek-cryptography](https://github.com/dalek-cryptography/curve25519-dalek)

## 🤝 Contributing

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

## 📄 License

MIT License - see LICENSE file for details

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/GelapLabs/gelap-cryptography/issues)
- **Discord**: [Join our Discord](#)
- **Email**: support@gelap.xyz

## ⚠️ Disclaimer

This software is in active development and has not been audited. Use at your own risk. Do not use in production without a professional security audit.
