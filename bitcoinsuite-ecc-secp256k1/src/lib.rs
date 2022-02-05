use bitcoinsuite_core::{
    ecc::{Ecc, EccError, PubKey, SecKey, VerifySignatureError, PUBKEY_LENGTH},
    ByteArray, Bytes,
};
use secp256k1_abc::{All, Message, PublicKey, Secp256k1, SecretKey, Signature};

#[derive(Clone)]
pub struct EccSecp256k1 {
    curve: Secp256k1<All>,
}

impl Default for EccSecp256k1 {
    fn default() -> Self {
        EccSecp256k1 {
            curve: Secp256k1::new(),
        }
    }
}

impl Ecc for EccSecp256k1 {
    fn pubkey_from_array(&self, pubkey: [u8; PUBKEY_LENGTH]) -> Result<PubKey, EccError> {
        PublicKey::from_slice(&pubkey).map_err(|_| EccError::InvalidPublicKey)?;
        Ok(PubKey::new_unchecked(pubkey))
    }

    fn seckey_from_array(&self, seckey: [u8; 32]) -> Result<SecKey, EccError> {
        SecretKey::from_slice(&seckey).map_err(|_| EccError::InvalidSecretKey)?;
        Ok(SecKey::new_unchecked(seckey))
    }

    fn sign(&self, seckey: &SecKey, msg: ByteArray<32>) -> Bytes {
        let msg = Message::from_slice(&msg).expect("Impossible");
        let seckey = SecretKey::from_slice(seckey.as_slice()).expect("Invalid secret key");
        let sig = self.curve.sign(&msg, &seckey);
        sig.serialize_der().to_vec().into()
    }

    fn schnorr_sign(&self, seckey: &SecKey, msg: ByteArray<32>) -> Bytes {
        let msg = Message::from_slice(&msg).expect("Impossible");
        let seckey = SecretKey::from_slice(seckey.as_slice()).expect("Invalid secret key");
        let sig = self.curve.schnorrabc_sign(&msg, &seckey);
        sig.as_ref().to_vec().into()
    }

    fn verify(
        &self,
        pubkey: &PubKey,
        msg: ByteArray<32>,
        sig: &Bytes,
    ) -> Result<(), VerifySignatureError> {
        let pubkey = PublicKey::from_slice(pubkey.as_slice()).expect("Invalid pubkey");
        let msg = Message::from_slice(&msg).expect("Impossible");
        let sig = Signature::from_der(sig).map_err(|_| VerifySignatureError::InvalidFormat)?;
        self.curve
            .verify(&msg, &sig, &pubkey)
            .map_err(|_| VerifySignatureError::IncorrectSignature)
    }

    fn schnorr_verify(
        &self,
        pubkey: &PubKey,
        msg: ByteArray<32>,
        sig: &Bytes,
    ) -> Result<(), VerifySignatureError> {
        let pubkey = PublicKey::from_slice(pubkey.as_slice()).expect("Invalid pubkey");
        let msg = Message::from_slice(&msg).expect("Impossible");
        let sig = secp256k1_abc::schnorrsig::Signature::from_slice(sig)
            .map_err(|_| VerifySignatureError::InvalidFormat)?;
        self.curve
            .schnorrabc_verify(&sig, &msg, &pubkey)
            .map_err(|_| VerifySignatureError::IncorrectSignature)
    }

    fn derive_pubkey(&self, seckey: &SecKey) -> PubKey {
        let seckey = SecretKey::from_slice(seckey.as_slice()).expect("Invalid secret key");
        let pubkey = PublicKey::from_secret_key(&self.curve, &seckey);
        PubKey::new_unchecked(pubkey.serialize())
    }

    fn serialize_pubkey_uncompressed(&self, pubkey: &PubKey) -> [u8; 65] {
        PublicKey::from_slice(pubkey.as_slice())
            .expect("Invalid pubkey")
            .serialize_uncompressed()
    }

    fn normalize_sig(&self, sig: &Bytes) -> Result<Bytes, EccError> {
        let mut sig = Signature::from_der_lax(sig).map_err(|_| EccError::InvalidSignatureFormat)?;
        sig.normalize_s();
        Ok(sig.serialize_der().to_vec().into())
    }
}

#[cfg(test)]
mod tests {
    use super::EccSecp256k1;
    use bitcoinsuite_core::ecc::{Ecc, EccError, PubKey, VerifySignatureError};
    use hex_literal::hex;

    #[test]
    fn test_pubkey_from_array() {
        let ecc = EccSecp256k1::default();
        ecc.pubkey_from_array([2; 33]).unwrap();
        ecc.pubkey_from_array(hex!(
            "030303030303030303030303030303030303030303030303030303030303030302"
        ))
        .unwrap();
        ecc.pubkey_from_array(PubKey::default().array()).unwrap();
        assert_eq!(
            ecc.pubkey_from_array([0; 33]).unwrap_err(),
            EccError::InvalidPublicKey
        );
        assert_eq!(
            ecc.pubkey_from_array([4; 33]).unwrap_err(),
            EccError::InvalidPublicKey
        );
    }

    #[test]
    fn test_seckey_from_array() {
        let ecc = EccSecp256k1::default();
        ecc.seckey_from_array([2; 32]).unwrap();
        assert_eq!(
            ecc.seckey_from_array([0; 32]).unwrap_err(),
            EccError::InvalidSecretKey
        );
    }

    #[test]
    fn test_sign() {
        let ecc = EccSecp256k1::default();
        let seckey = ecc.seckey_from_array([2; 32]).unwrap();
        let msg = [3; 32];
        let sig = ecc.sign(&seckey, msg.into());
        assert_eq!(sig.hex(), "304402207228f8a93734f17480911e04ee5d83d8ccb1e880c8b46f71ce1c2f99c87627bd022069a70f991882d15929b565507cb380108719c69f8105e2c16c6e1d4b4efb747f");
        let pubkey = ecc.derive_pubkey(&seckey);
        assert_eq!(
            pubkey.hex(),
            "024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766"
        );
        ecc.verify(&pubkey, msg.into(), &sig).unwrap();
    }

    #[test]
    fn test_schnorr_sign_02() {
        let ecc = EccSecp256k1::default();
        let seckey = ecc.seckey_from_array([2; 32]).unwrap();
        let msg = [3; 32];
        let sig = ecc.schnorr_sign(&seckey, msg.into());
        assert_eq!(sig.len(), 64);
        let pubkey = ecc.derive_pubkey(&seckey);
        assert_eq!(
            pubkey.hex(),
            "024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766"
        );
        ecc.schnorr_verify(&pubkey, msg.into(), &sig).unwrap();
    }

    #[test]
    fn test_schnorr_sign_03() {
        let ecc = EccSecp256k1::default();
        let seckey = ecc.seckey_from_array([1; 32]).unwrap();
        let msg = [3; 32];
        let sig = ecc.schnorr_sign(&seckey, msg.into());
        assert_eq!(sig.len(), 64);
        let pubkey = ecc.derive_pubkey(&seckey);
        assert_eq!(
            pubkey.hex(),
            "031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f"
        );
        ecc.schnorr_verify(&pubkey, msg.into(), &sig).unwrap();
    }

    #[test]
    fn test_verify() {
        let ecc = EccSecp256k1::default();
        let pubkey = PubKey::from_hex_unchecked(
            "024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766",
        )
        .unwrap();
        let msg = [3; 32];
        let sig = hex!("304402201c0ae2b7a4767475abb53f6cdfc1f2bd46666d0bb4ea75d3c47dd439ad7a541302207a04da160132a0a73a891ac6ab3263665edb523c6dcccc11bbe4661117ce3eef");
        ecc.verify(&pubkey, msg.into(), &sig.into())
            .expect("Verify failed!");

        let wrong_pubkey = PubKey::from_hex_unchecked(
            "020000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();
        assert_eq!(
            ecc.verify(&wrong_pubkey, msg.into(), &sig.into())
                .unwrap_err(),
            VerifySignatureError::IncorrectSignature
        );

        let bad_sig = hex!("204402201c0ae2b7a4767475abb53f6cdfc1f2bd46666d0bb4ea75d3c47dd439ad7a541302207a04da160132a0a73a891ac6ab3263665edb523c6dcccc11bbe4661117ce3eef");
        assert_eq!(
            ecc.verify(&pubkey, msg.into(), &bad_sig.into())
                .unwrap_err(),
            VerifySignatureError::InvalidFormat
        );
    }

    #[test]
    fn test_derive_pubkey() {
        let ecc = EccSecp256k1::default();
        let seckey = ecc.seckey_from_array([2; 32]).unwrap();
        let pubkey = ecc.derive_pubkey(&seckey);
        assert_eq!(
            pubkey.hex(),
            "024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766"
        );
    }

    #[test]
    fn test_normalize_sig() {
        let ecc = EccSecp256k1::default();
        // normalized sig unchanged
        let sig = hex!("304402201c0ae2b7a4767475abb53f6cdfc1f2bd46666d0bb4ea75d3c47dd439ad7a541302207a04da160132a0a73a891ac6ab3263665edb523c6dcccc11bbe4661117ce3eef").into();
        let normalized_sig = ecc.normalize_sig(&sig).unwrap();
        assert_eq!(sig.hex(), normalized_sig.hex());

        // valid DER but superfluous 0
        let unnormal_sig = hex!("30450221001c0ae2b7a4767475abb53f6cdfc1f2bd46666d0bb4ea75d3c47dd439ad7a541302207a04da160132a0a73a891ac6ab3263665edb523c6dcccc11bbe4661117ce3eef").into();
        let normalized_sig = ecc.normalize_sig(&unnormal_sig).unwrap();
        assert_eq!(sig.hex(), normalized_sig.hex());

        // valid DER but high S
        let high_s_sig = hex!("304502202289e8e0dfd833a207da5bf6e2f8edc8fb2beb78ab4982c48fc557c27b7c9e68022100c41560847ba73867553a9cc14f02b56228eda5b9d263aa9aacab68680cc1a99301").into();
        let normalized_sig = ecc.normalize_sig(&high_s_sig).unwrap();
        assert_eq!(normalized_sig.hex(), "304402202289e8e0dfd833a207da5bf6e2f8edc8fb2beb78ab4982c48fc557c27b7c9e6802203bea9f7b8458c798aac5633eb0fd4a9c91c1372cdce4f5a11326f624c37497ae");
    }
}
