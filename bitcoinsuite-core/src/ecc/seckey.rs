use std::fmt::Debug;

use secrecy::{ExposeSecret, Secret};

pub const SECKEY_LENGTH: usize = 32;

#[derive(Clone)]
pub struct SecKey(Secret<[u8; SECKEY_LENGTH]>);

impl SecKey {
    pub fn new_unchecked(seckey: [u8; SECKEY_LENGTH]) -> SecKey {
        SecKey(Secret::new(seckey))
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.expose_secret()
    }
}

impl Debug for SecKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecKey([SECRET])")
    }
}

#[cfg(test)]
mod tests {
    use super::SecKey;

    #[test]
    fn test_as_slice() {
        let seckey = SecKey::new_unchecked([1; 32]);
        assert_eq!(seckey.as_slice(), &[1; 32]);
    }

    #[test]
    fn test_format_debug_doesnt_leak() {
        let seckey = SecKey::new_unchecked([1; 32]);
        assert_eq!(format!("{:?}", seckey), "SecKey([SECRET])");
    }
}
