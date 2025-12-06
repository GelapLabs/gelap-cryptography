// This program runs inside the zkVM and verifies:
// 1. Ring signature is valid (sender anonymity)
// 2. Commitments balance: sum(inputs) = sum(output)
// 3. Key image is correctly computes (prevents double-spend)

#![no_main]
sp1_zkvm::entrypoint!(main);

use cryptography_types::{proof::PublicInputs, transaction::PrivateTransaction};

use curve25519_dalek::{
    constants::RISTRETTO_BASEPOINT_POINT,
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};

use sha2::{Digest, Sha512};

pub fn main() {
    let tx: PrivateTransaction = sp1_zkvm::io::read();

    // Step 1: Verify Ring Signature
    let key_image = parse_ristretto_point(&tx.key_image).expect("Invalid key image");

    let mut ring: Vec<RistrettoPoint> = Vec::new();
    for member_bytes in &tx.ring {
        let point = parse_ristretto_point(member_bytes).expect("Invalid ring member");
        ring.push(point);
    }

    let mut c_values: Vec<Scalar> = Vec::new();
    let mut r_values: Vec<Scalar> = Vec::new();

    for c_bytes in &tx.ring_signature.c {
        let scalar = parse_scalar(c_bytes).expect("Invalid challenge");
        c_values.push(scalar)
    }

    for r_bytes in &tx.ring_signature.r {
        let scalar = parse_scalar(r_bytes).expect("Invalid response");
        r_values.push(scalar);
    }

    let message = b"PRIVATE_PAYMENT_TX";
    let ring_valid = verify_ring_signature(message, &key_image, &ring, &c_values, &r_values);

    assert!(ring_valid, "Ring signature verification failed");

    // Step 2 Verify Commitment Balance
    let input_sum: u64 = tx.input_amounts.iter().sum();
    let output_sum: u64 = tx.output_amounts.iter().sum();

    assert_eq!(
        input_sum, output_sum,
        "Transaction not balanced: inputs={}, outputs={}",
        input_sum, output_sum
    );

    // Verify Input
    for (i, amount) in tx.input_amounts.iter().enumerate() {
        let blinding = parse_scalar(&tx.input_blindings[i]).expect("Invalid input blinding");
        let computed_commitment = pedersen_commitment(*amount, &blinding);
        let claimed_commitment = parse_ristretto_point(&tx.input_commitments[i].commitment)
            .expect("Invalid input commitment");

        assert_eq!(
            computed_commitment, claimed_commitment,
            "Input commitment {} does not match",
            i
        );
    }

    // Verify Output
    for (i, amount) in tx.output_amounts.iter().enumerate() {
        let blinding = parse_scalar(&tx.output_blindings[i]).expect("Invalid output blinding");

        let computed_commitment = pedersen_commitment(*amount, &blinding);
        let claimed_commitment = parse_ristretto_point(&tx.output_commitments[i].commitment)
            .expect("Invalid output commitment");
        assert_eq!(
            computed_commitment, claimed_commitment,
            "Output commitment {} does not match",
            i
        );
    }

    // Step 3 Verify Key Image
    let secret_index = tx.secret_index;
    assert!(
        secret_index < ring.len(),
        "Invalid secret index: {} >= {}",
        secret_index,
        ring.len()
    );

    // Step 4 Commit Public Inputs

    let public_inputs = PublicInputs {
        input_commitments: tx.input_commitments.iter().map(|c| c.commitment).collect(),
        output_commitments: tx.output_commitments.iter().map(|c| c.commitment).collect(),
        key_image: tx.key_image,
        ring: tx.ring.clone(),
    };

    sp1_zkvm::io::commit(&public_inputs);
}

fn verify_ring_signature(
    message: &[u8],
    key_image: &RistrettoPoint,
    ring: &[RistrettoPoint],
    c_values: &[Scalar],
    r_values: &[Scalar],
) -> bool {
    let n = ring.len();

    if c_values.len() != n || r_values.len() != n {
        return false;
    }

    if n == 0 {
        return false;
    }

    for i in 0..n {
        let next_i = (i + 1) % n;

        let l = r_values[i] * RISTRETTO_BASEPOINT_POINT + c_values[i] * ring[i];

        let hash_point = hash_to_point(&ring[i]);
        let r_part = r_values[i] * hash_point + c_values[i] * key_image;

        let computed_c = hash_challenge(message, &l, &r_part);

        if computed_c != c_values[next_i] {
            return false;
        }
    }
    true
}

fn pedersen_commitment(amount: u64, blinding: &Scalar) -> RistrettoPoint {
    let g = RISTRETTO_BASEPOINT_POINT;
    let h = get_h_generator();

    Scalar::from(amount) * g + blinding * h
}

fn get_h_generator() -> RistrettoPoint {
    let g = RISTRETTO_BASEPOINT_POINT;
    let g_bytes = g.compress().to_bytes();

    let mut hasher = Sha512::new();
    hasher.update(b"Pedersen_H_GENERATOR_V2");
    hasher.update(&g_bytes);
    let hash = hasher.finalize();

    RistrettoPoint::from_uniform_bytes(&hash.into())
}

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

fn parse_ristretto_point(bytes: &[u8; 32]) -> Option<RistrettoPoint> {
    CompressedRistretto(*bytes).decompress()
}

fn parse_scalar(bytes: &[u8; 32]) -> Option<Scalar> {
    Some(Scalar::from_bytes_mod_order(*bytes))
}
