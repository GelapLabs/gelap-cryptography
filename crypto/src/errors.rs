use thiserror::Error;

#[derive(Debug, Error)]

pub enum CryptoError {
    #[error("ECDH computation failed")]
    EcdhFailed,

    #[error("Invalid secp256k1 public key")]
    InvalidPublicKey,

    #[error("Invalid secp256k1 secret key")]
    InvalidSecretKey,

    #[error("Point addition failed")]
    PointAdditionFailed,

    #[error("Invalid scalar value")]
    InvalidScalar,

    #[error("Invalid ristretto points")]
    InvalidRisettoPoints,

    #[error("Commitmet verification failed")]
    CommitmentVerificationFailed,

    #[error("Ring signature verification failed")]
    RingSignatureVerificationFailed,

    #[error("Key image already used (double spend detected)")]
    KeyImageUsed,

    #[error("Serialization error: {0}")]
    SerilizationError(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid output: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
