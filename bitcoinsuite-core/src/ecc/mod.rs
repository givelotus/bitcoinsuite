mod pubkey;
mod seckey;

use crate::{ByteArray, Bytes};

pub use crate::ecc::pubkey::*;
pub use crate::ecc::seckey::*;

use thiserror::Error;

pub const MAX_ECDSA_SIGNATURE_SIZE: usize = 71;
pub const SCHNORR_SIGNATURE_SIZE: usize = 64;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
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

pub trait Ecc {
    fn pubkey_from_array(&self, pubkey: [u8; PUBKEY_LENGTH]) -> Result<PubKey, EccError>;

    fn seckey_from_array(&self, seckey: [u8; 32]) -> Result<SecKey, EccError>;

    fn sign(&self, seckey: &SecKey, msg: ByteArray<32>) -> Bytes;

    fn schnorr_sign(&self, seckey: &SecKey, msg: ByteArray<32>) -> Bytes;

    fn verify(
        &self,
        pubkey: &PubKey,
        msg: ByteArray<32>,
        sig: &Bytes,
    ) -> Result<(), VerifySignatureError>;

    fn schnorr_verify(
        &self,
        pubkey: &PubKey,
        msg: ByteArray<32>,
        sig: &Bytes,
    ) -> Result<(), VerifySignatureError>;

    fn derive_pubkey(&self, seckey: &SecKey) -> PubKey;

    fn serialize_pubkey_uncompressed(&self, pubkey: &PubKey) -> [u8; 65];

    fn normalize_sig(&self, sig: &Bytes) -> Result<Bytes, EccError>;
}

#[derive(Debug, Clone, Copy)]
pub struct DummyEcc;

impl Ecc for DummyEcc {
    fn pubkey_from_array(&self, _pubkey: [u8; PUBKEY_LENGTH]) -> Result<PubKey, EccError> {
        Ok(PubKey::new_unchecked([0; PUBKEY_LENGTH]))
    }

    fn seckey_from_array(&self, _seckey: [u8; 32]) -> Result<SecKey, EccError> {
        Ok(SecKey::new_unchecked([0; 32]))
    }

    fn sign(&self, _seckey: &SecKey, _msg: ByteArray<32>) -> Bytes {
        vec![0; MAX_ECDSA_SIGNATURE_SIZE].into()
    }

    fn schnorr_sign(&self, _seckey: &SecKey, _msg: ByteArray<32>) -> Bytes {
        vec![0; SCHNORR_SIGNATURE_SIZE].into()
    }

    fn verify(
        &self,
        _pubkey: &PubKey,
        _msg: ByteArray<32>,
        _sig: &Bytes,
    ) -> Result<(), VerifySignatureError> {
        unimplemented!()
    }

    fn schnorr_verify(
        &self,
        _pubkey: &PubKey,
        _msg: ByteArray<32>,
        _sig: &Bytes,
    ) -> Result<(), VerifySignatureError> {
        unimplemented!()
    }

    fn derive_pubkey(&self, _seckey: &SecKey) -> PubKey {
        PubKey::new_unchecked([0; PUBKEY_LENGTH])
    }

    fn serialize_pubkey_uncompressed(&self, _pubkey: &PubKey) -> [u8; 65] {
        [0; 65]
    }

    fn normalize_sig(&self, sig: &Bytes) -> Result<Bytes, EccError> {
        Ok(vec![0; sig.len()].into())
    }
}
