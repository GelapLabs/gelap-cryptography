use clap::{Parser, ValueEnum};
use cryptography_types::{
    commitment::CommitmentData, proof::PublicInputs, signature::RingSignatureData,
    stealth::StealthAddressData, transaction::PrivateTransaction,
};
use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use sp1_sdk::{
    include_elf, network::NetworkMode, HashableKey, Prover, ProverClient, SP1ProofWithPublicValues,
    SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;

const ELF: &[u8] = include_elf!("cryptography-zkvm");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(long, value_enum, default_value = "groth16")]
    system: ProofSystem,

    #[arg(long, default_value = "100")]
    amount: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrivatePaymentProofFixture {
    input_amount: u64,
    output_amounts: Vec<u64>,
    ring_size: usize,
    vkey: String,
    public_values: String,
    proof: String,
    key_image: String,
    input_commitments: Vec<String>,
    output_commitments: Vec<String>,
}

fn main() {
    sp1_sdk::utils::setup_logger();

    let args = EVMArgs::parse();

    let client = ProverClient::builder()
        .network_for(NetworkMode::Mainnet)
        .build();

    let (pk, vk) = client.setup(ELF);

    let tx = create_test_transaction(args.amount);
    let mut stdin = SP1Stdin::new();
    stdin.write(&tx);

    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("Failed to generated proof");

    create_proof_fixture(&proof, &vk, &tx, args.system);
}

fn create_proof_fixture(
    proof: &SP1ProofWithPublicValues,
    vk: &SP1VerifyingKey,
    tx: &PrivateTransaction,
    system: ProofSystem,
) {
    let bytes = proof.public_values.as_slice();
    let public_inputs: PublicInputs =
        bincode::deserialize(bytes).expect("Failed to deserialize public values");

    let fixture = PrivatePaymentProofFixture {
        input_amount: tx.input_amounts[0],
        output_amounts: tx.output_amounts.clone(),
        ring_size: tx.ring.len(),
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
        key_image: format!("0x{}", hex::encode(public_inputs.key_image)),
        input_commitments: public_inputs
            .input_commitments
            .iter()
            .map(|c| format!("0x{}", hex::encode(c)))
            .collect(),
        output_commitments: public_inputs
            .output_commitments
            .iter()
            .map(|c| format!("0x{}", hex::encode(c)))
            .collect(),
    };

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");

    std::fs::create_dir_all(&fixture_path).expect("Failed to create fixture directory");

    let filename = format!("{:?}--fixture.json", system).to_lowercase();
    let file_path = fixture_path.join(&filename);

    std::fs::write(&file_path, serde_json::to_string_pretty(&fixture).unwrap())
        .expect("Failed to write fixture file")
}

fn create_test_transaction(amount: u64) -> PrivateTransaction {
    let output1 = (amount * 6) / 10;
    let output2 = (amount * 4) / 10;

    let g = RISTRETTO_BASEPOINT_POINT;

    // Generate H for Pedersen commitments
    let h = {
        let g_bytes = g.compress().to_bytes();
        let mut hasher = Sha512::new();
        hasher.update(b"Pedersen_H_GENERATOR_V2");
        hasher.update(&g_bytes);
        let hash = hasher.finalize();
        RistrettoPoint::from_uniform_bytes(&hash.into())
    };

    // Generate REAL commitments
    let input_blinding = Scalar::from(12345u64);
    let input_commitment = Scalar::from(amount) * g + input_blinding * h;

    let output1_blinding = Scalar::from(67890u64);
    let output2_blinding = Scalar::from(11111u64);
    let output1_commitment = Scalar::from(output1) * g + output1_blinding * h;
    let output2_commitment = Scalar::from(output2) * g + output2_blinding * h;

    // Generate a proper LSAG ring signature
    let secret_index = 2usize;
    let secret_key = Scalar::from(424242u64); // Secret key of the signer
    let public_key = secret_key * g; // Public key corresponding to secret key

    // Create ring with real public key at secret_index
    let ring: Vec<[u8; 32]> = (0..5)
        .map(|i| {
            if i == secret_index {
                public_key.compress().to_bytes()
            } else {
                let scalar = Scalar::from(((i + 1) * 1111) as u64);
                (scalar * g).compress().to_bytes()
            }
        })
        .collect();

    // Compute key image: I = x * H_p(P)
    let key_image = {
        let h_point = hash_to_point(&public_key);
        (secret_key * h_point).compress().to_bytes()
    };

    // Generate LSAG ring signature
    let message = b"PRIVATE_PAYMENT_TX";
    let (c_values, r_values) = generate_ring_signature(
        message,
        &secret_key,
        secret_index,
        &ring.iter().map(|r| decompress_point(r)).collect::<Vec<_>>(),
    );

    PrivateTransaction {
        input_commitments: vec![CommitmentData::new(input_commitment.compress().to_bytes())],
        output_commitments: vec![
            CommitmentData::new(output1_commitment.compress().to_bytes()),
            CommitmentData::new(output2_commitment.compress().to_bytes()),
        ],
        key_image,
        ring,
        stealth_addresses: vec![
            StealthAddressData::new(vec![10u8; 33], [0x42u8; 20]),
            StealthAddressData::new(vec![11u8; 33], [0x43u8; 20]),
        ],
        input_amounts: vec![amount],
        input_blindings: vec![input_blinding.to_bytes()],
        output_amounts: vec![output1, output2],
        output_blindings: vec![output1_blinding.to_bytes(), output2_blinding.to_bytes()],
        ring_signature: RingSignatureData::new(c_values, r_values),
        secret_index,
    }
}

// Helper functions for ring signature generation
fn hash_to_point(point: &RistrettoPoint) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(b"HASH_TO_POINTS_V1");
    hasher.update(point.compress().as_bytes());
    let hash = hasher.finalize();
    RistrettoPoint::from_uniform_bytes(&hash.into())
}

fn hash_challenge(message: &[u8], l: &RistrettoPoint, r: &RistrettoPoint) -> Scalar {
    let mut hasher = Sha512::new();
    hasher.update(b"RING_SIG_V1");
    hasher.update(message);
    hasher.update(l.compress().as_bytes());
    hasher.update(r.compress().as_bytes());
    let hash = hasher.finalize();
    Scalar::from_bytes_mod_order_wide(&hash.into())
}

fn decompress_point(bytes: &[u8; 32]) -> RistrettoPoint {
    CompressedRistretto(*bytes).decompress().expect("Invalid point")
}

fn generate_ring_signature(
    message: &[u8],
    secret_key: &Scalar,
    secret_index: usize,
    ring: &[RistrettoPoint],
) -> (Vec<[u8; 32]>, Vec<[u8; 32]>) {
    use rand::rngs::OsRng;
    use rand::RngCore;

    let g = RISTRETTO_BASEPOINT_POINT;
    let n = ring.len();

    // Compute key image
    let public_key = &ring[secret_index];
    let h_point = hash_to_point(public_key);
    let key_image = secret_key * h_point;

    // Initialize arrays
    let mut c_values = vec![Scalar::ZERO; n];
    let mut r_values = vec![Scalar::ZERO; n];

    // Pick random alpha
    let mut alpha_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut alpha_bytes);
    let alpha = Scalar::from_bytes_mod_order(alpha_bytes);

    // Compute initial L and R at secret index
    let l_s = alpha * g;
    let r_s = alpha * h_point;

    // Pick random c and r values for all indices except secret
    for i in 0..n {
        if i != secret_index {
            let mut c_bytes = [0u8; 32];
            let mut r_bytes = [0u8; 32];
            OsRng.fill_bytes(&mut c_bytes);
            OsRng.fill_bytes(&mut r_bytes);
            c_values[i] = Scalar::from_bytes_mod_order(c_bytes);
            r_values[i] = Scalar::from_bytes_mod_order(r_bytes);
        }
    }

    // Compute the ring starting from (secret_index + 1)
    let mut current_c = Scalar::ZERO;
    for offset in 1..=n {
        let i = (secret_index + offset) % n;
        let prev_i = if i == 0 { n - 1 } else { i - 1 };

        let (l, r) = if prev_i == secret_index {
            (l_s, r_s)
        } else {
            let l = r_values[prev_i] * g + c_values[prev_i] * ring[prev_i];
            let h_i = hash_to_point(&ring[prev_i]);
            let r = r_values[prev_i] * h_i + c_values[prev_i] * key_image;
            (l, r)
        };

        current_c = hash_challenge(message, &l, &r);

        if i != secret_index {
            c_values[i] = current_c;
        }
    }

    // Close the ring: solve for r at secret_index
    c_values[secret_index] = current_c;
    r_values[secret_index] = alpha - c_values[secret_index] * secret_key;

    // Convert to byte arrays
    let c_bytes: Vec<[u8; 32]> = c_values.iter().map(|c| c.to_bytes()).collect();
    let r_bytes: Vec<[u8; 32]> = r_values.iter().map(|r| r.to_bytes()).collect();

    (c_bytes, r_bytes)
}
