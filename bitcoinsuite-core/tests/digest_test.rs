//! This is a test

use bitcoinsuite_core::{
   Hashed,  Sha256d,
};

use hex_literal::hex;

#[test]
fn digest_function(){
    let x = Sha256d::new(hex!("19c6197e2140b9d034fb20b9ac7bb753a41233caf1e1dafda7316a99cef41416"));
    let hash = Sha256d::digest(&[1,2,3]);
    assert_eq!(x, hash);
}
