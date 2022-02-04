mod pubkey;
mod seckey;

use bitcoinsuite_core::{ByteArray, Bytes};

pub use crate::pubkey::*;
pub use crate::seckey::*;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum VerifySignatureError {
    #[error("Invalid signature format")]
    InvalidFormat,
    #[error("Signature check failed")]
    IncorrectSignature,
}

#[derive(Debug, Error, PartialEq)]
pub enum EccError {
    #[error("Invalid signature format")]
    InvalidSignatureFormat,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid secret key")]
    InvalidSecretKey,
    #[error("Pubkey has invalid length, expected {:.0} but got {0}", 33)]
    InvalidPubKeyLen(usize),
    #[error("Invalid hex: {0}")]
    Hex(#[from] hex::FromHexError),
}

pub trait Ecc: Default {
    fn pubkey_from_array(&self, pubkey: [u8; PUBKEY_LENGTH]) -> Result<PubKey, EccError>;

    fn seckey_from_array(&self, seckey: [u8; 32]) -> Result<SecKey, EccError>;

    fn sign(&self, seckey: &SecKey, msg: ByteArray<32>) -> Bytes;

    fn verify(
        &self,
        pubkey: &PubKey,
        msg: ByteArray<32>,
        sig: &Bytes,
    ) -> Result<(), VerifySignatureError>;

    fn derive_pubkey(&self, seckey: &SecKey) -> PubKey;

    fn normalize_sig(&self, sig: &Bytes) -> Result<Bytes, EccError>;
}
