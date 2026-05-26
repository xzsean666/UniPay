use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public};
use openssl::sign::{Signer, Verifier};
use sha2::Sha256;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("failed to parse private key")]
    InvalidPrivateKey,
    #[error("failed to parse public key")]
    InvalidPublicKey,
    #[error("failed to decode signature")]
    InvalidSignatureEncoding,
    #[error("signature verification failed")]
    VerificationFailed,
}

pub type SigningResult<T> = Result<T, SigningError>;

pub trait MessageSigner: Send + Sync {
    fn sign(&self, message: &[u8]) -> SigningResult<String>;
}

pub trait MessageVerifier: Send + Sync {
    fn verify(&self, message: &[u8], base64_signature: &str) -> SigningResult<()>;
}

pub struct RsaSha256Signer {
    private_key: PKey<Private>,
}

impl RsaSha256Signer {
    pub fn from_pkcs8_pem(pem: &str) -> SigningResult<Self> {
        let private_key = PKey::private_key_from_pem(pem.as_bytes())
            .map_err(|_| SigningError::InvalidPrivateKey)?;
        Ok(Self { private_key })
    }
}

impl MessageSigner for RsaSha256Signer {
    fn sign(&self, message: &[u8]) -> SigningResult<String> {
        let mut signer = Signer::new(MessageDigest::sha256(), &self.private_key)
            .map_err(|_| SigningError::InvalidPrivateKey)?;
        signer
            .update(message)
            .map_err(|_| SigningError::InvalidPrivateKey)?;
        let signature = signer
            .sign_to_vec()
            .map_err(|_| SigningError::InvalidPrivateKey)?;
        Ok(STANDARD.encode(signature))
    }
}

pub struct RsaSha256Verifier {
    public_key: PKey<Public>,
}

impl RsaSha256Verifier {
    pub fn from_public_key_pem(pem: &str) -> SigningResult<Self> {
        let public_key = PKey::public_key_from_pem(pem.as_bytes())
            .map_err(|_| SigningError::InvalidPublicKey)?;
        Ok(Self { public_key })
    }
}

impl MessageVerifier for RsaSha256Verifier {
    fn verify(&self, message: &[u8], base64_signature: &str) -> SigningResult<()> {
        let signature_bytes = STANDARD
            .decode(base64_signature)
            .map_err(|_| SigningError::InvalidSignatureEncoding)?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &self.public_key)
            .map_err(|_| SigningError::InvalidPublicKey)?;
        verifier
            .update(message)
            .map_err(|_| SigningError::InvalidPublicKey)?;
        let valid = verifier
            .verify(&signature_bytes)
            .map_err(|_| SigningError::VerificationFailed)?;
        if valid {
            Ok(())
        } else {
            Err(SigningError::VerificationFailed)
        }
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest;

    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
