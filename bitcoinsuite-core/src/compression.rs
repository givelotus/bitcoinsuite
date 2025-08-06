use crate::{
    ecc::Ecc, BitcoinSuiteError, Bytes, BytesMut, Coin, Hashed, Op, Script, ScriptVariant,
    ShaRmd160, TxOutput,
};

/// Amount compression:
/// * If the amount is 0, output 0
/// * first, divide the amount (in base units) by the largest power of 10
///   possible; call the exponent e (e is max 9)
/// * if e<9, the last digit of the resulting number cannot be 0; store it as d,
///   and drop it (divide by 10)
///   * call the result n
///   * output 1 + 10*(9*n + d - 1) + e
/// * if e==9, we only know the resulting number is not zero, so output
///   1 + 10*(n - 1) + 9.
///   (this is decodable, as d is in [1-9] and e is in [0-9])
pub fn compress_amount(mut amount: u64) -> u64 {
    if amount == 0 {
        return 0;
    }
    let mut e = 0;
    while ((amount % 10) == 0) && e < 9 {
        amount /= 10;
        e += 1;
    }
    if e < 9 {
        let d = amount % 10;
        assert!((1..=9).contains(&d));
        amount /= 10;
        1 + (amount * 9 + d - 1) * 10 + e
    } else {
        1 + (amount - 1) * 10 + 9
    }
}

pub fn decompress_amount(mut x: u64) -> u64 {
    // x = 0  OR  x = 1+10*(9*n + d - 1) + e  OR  x = 1+10*(n - 1) + 9
    if x == 0 {
        return 0;
    }
    x -= 1;
    // x = 10*(9*n + d - 1) + e
    let mut e = x % 10;
    x /= 10;
    let mut n = if e < 9 {
        // x = 9*n + d - 1
        let d = (x % 9) + 1;
        x /= 9;
        // x = n
        x * 10 + d
    } else {
        x + 1
    };
    while e != 0 {
        n *= 10;
        e -= 1;
    }
    n
}

pub fn write_var_int(bytes: &mut BytesMut, mut n: u64) {
    let mut tmp = [0u8; 10];
    let mut len = 0;
    loop {
        tmp[len] = (n & 0x7F) as u8 | (if len != 0 { 0x80 } else { 0x00 });
        if n <= 0x7F {
            break;
        }
        n = (n >> 7) - 1;
        len += 1;
    }
    loop {
        bytes.put_slice(&[tmp[len]]);
        if len == 0 {
            break;
        }
        len -= 1;
    }
}

pub fn read_var_int(bytes: &mut Bytes) -> Result<u64, BitcoinSuiteError> {
    let mut n = 0u64;
    loop {
        let ch_data = bytes.split_to_array::<1>()?[0];
        if n > (u64::MAX >> 7) {
            return Err(BitcoinSuiteError::InvalidVarInt);
        }
        n = (n << 7) | (ch_data & 0x7F) as u64;
        if (ch_data & 0x80) == 0 {
            return Ok(n);
        }
        if n == u64::MAX {
            return Err(BitcoinSuiteError::InvalidVarInt);
        }
        n += 1;
    }
}

pub fn compress_script(script: &Script) -> Option<Vec<u8>> {
    Some(match script.parse_variant() {
        ScriptVariant::P2PKH(hash) => [[0x00].as_ref(), hash.as_slice()].concat(),
        ScriptVariant::P2SH(hash) => [[0x01].as_ref(), hash.as_slice()].concat(),
        ScriptVariant::P2PK(pk) => pk.as_slice().to_vec(),
        ScriptVariant::P2PKLegacy(pk) => {
            let mut result = vec![0; 33];
            result[0] = 0x04 | (pk[64] & 0x01);
            result[1..].copy_from_slice(&pk[1..33]);
            result
        }
        _ => return None,
    })
}

pub fn decompress_script(
    ecc: &dyn Ecc,
    compressed: &mut Bytes,
) -> Result<Script, BitcoinSuiteError> {
    let head = read_var_int(compressed)?;
    Ok(match head {
        0x00 => Script::p2pkh(&ShaRmd160::new(compressed.split_to_array::<20>()?.array())),
        0x01 => Script::p2sh(&ShaRmd160::new(compressed.split_to_array::<20>()?.array())),
        0x02 | 0x03 => {
            let x_coords = compressed.split_to_array::<32>()?.array();
            let mut pk = [0u8; 33];
            pk[0] = head as u8;
            pk[1..].clone_from_slice(&x_coords);
            Script::p2pk(&ecc.pubkey_from_array(pk)?)
        }
        0x04 | 0x05 => {
            let x_coords = compressed.split_to_array::<32>()?.array();
            let mut pk = [0u8; 33];
            pk[0] = if head == 0x04 { 0x02 } else { 0x03 };
            pk[1..].clone_from_slice(&x_coords);
            let pubkey = ecc.pubkey_from_array(pk)?;
            let pubkey = ecc.serialize_pubkey_uncompressed(&pubkey);
            Script::from_ops([Op::Push(65, pubkey.into())].into_iter())?
        }
        _ => {
            let script = compressed.split_to(head as usize - 6)?;
            Script::new(script)
        }
    })
}

pub fn read_undo_coin(ecc: &dyn Ecc, undo_data: &mut Bytes) -> Result<Coin, BitcoinSuiteError> {
    read_coin_as::<true>(ecc, undo_data)
}

pub fn read_coin(ecc: &dyn Ecc, coin_data: &mut Bytes) -> Result<Coin, BitcoinSuiteError> {
    read_coin_as::<false>(ecc, coin_data)
}

fn read_coin_as<const IS_UNDO: bool>(
    ecc: &dyn Ecc,
    undo_data: &mut Bytes,
) -> Result<Coin, BitcoinSuiteError> {
    let height_and_is_coinbase = read_var_int(undo_data)?;
    let height = height_and_is_coinbase >> 1;
    let is_coinbase = (height_and_is_coinbase & 1) != 0;
    if IS_UNDO && height > 0 {
        read_var_int(undo_data)?;
    }
    let amount_compressed = read_var_int(undo_data)?;
    let amount = decompress_amount(amount_compressed);
    let script = decompress_script(ecc, undo_data)?;
    Ok(Coin {
        tx_output: TxOutput {
            value: amount as i64,
            script,
        },
        height: Some(height as i32),
        is_coinbase,
    })
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{
        compression::{compress_amount, decompress_amount, read_var_int},
        ecc::DummyEcc,
        Bytes, BytesMut, Coin, Hashed, Script, ShaRmd160, TxOutput,
    };

    use super::{read_coin, write_var_int};

    #[test]
    fn test_compress_amount() {
        fn check_amount_pair(amount: u64, compressed: u64) {
            assert_eq!(compress_amount(amount), compressed);
            assert_eq!(decompress_amount(compressed), amount);
        }

        check_amount_pair(0, 0x0);
        check_amount_pair(1, 0x1);
        check_amount_pair(1_000_000, 0x7);
        check_amount_pair(100_000_000, 0x9);
        check_amount_pair(5_000_000_000, 0x32);
        check_amount_pair(2_100_000_000_000_000, 0x1406f40);

        for i in 1..=100_000 {
            assert_eq!(i, decompress_amount(compress_amount(i)))
        }

        for i in 1..=10_000 {
            let i = i * 10_000;
            assert_eq!(i, decompress_amount(compress_amount(i)))
        }

        for i in 1..=10_000 {
            let i = i * 1_000_000;
            assert_eq!(i, decompress_amount(compress_amount(i)))
        }

        for i in 1..=420000 {
            let i = i * 50_000_000;
            assert_eq!(i, decompress_amount(compress_amount(i)))
        }

        for i in 1..=100_000 {
            assert_eq!(i, compress_amount(decompress_amount(i)))
        }
    }

    #[test]
    fn test_var_int() {
        fn check_var_int_pair(num: u64, data: &str) {
            let mut bytes = BytesMut::new();
            write_var_int(&mut bytes, num);
            assert_eq!(bytes.freeze().hex(), data);

            let mut bytes = Bytes::from_bytes(hex::decode(data).unwrap());
            let actual_num = read_var_int(&mut bytes).unwrap();
            assert_eq!(actual_num, num);
        }

        check_var_int_pair(0, "00");
        check_var_int_pair(0x7f, "7f");
        check_var_int_pair(0x80, "8000");
        check_var_int_pair(0x1234, "a334");
        check_var_int_pair(0xffff, "82fe7f");
        check_var_int_pair(0x123456, "c7e756");
        check_var_int_pair(0x80123456, "86ffc7e756");
        check_var_int_pair(0xffffffff, "8efefefe7f");
        check_var_int_pair(0x7fffffffffffffff, "fefefefefefefefe7f");
        check_var_int_pair(0xffffffffffffffff, "80fefefefefefefefe7f");
    }

    #[test]
    fn test_coin() -> Result<(), Box<dyn std::error::Error>> {
        fn check_coin(data: &str, coin: Coin) {
            let mut bytes = Bytes::from_bytes(hex::decode(data).unwrap());
            let actual_coin = read_coin(&DummyEcc, &mut bytes).unwrap();
            assert_eq!(actual_coin, coin);
        }
        fn check_coin_err(data: &str, msg: &str) {
            let mut bytes = Bytes::from_bytes(hex::decode(data).unwrap());
            let actual_err = read_coin(&DummyEcc, &mut bytes).unwrap_err();
            assert_eq!(actual_err.to_string(), msg);
        }
        check_coin(
            "97f23c835800816115944e077fe7c803cfa57f29b36bf87c1d35",
            Coin {
                tx_output: TxOutput {
                    value: 60_000_000_000,
                    script: Script::p2pkh(&ShaRmd160::from_hex(
                        "816115944e077fe7c803cfa57f29b36bf87c1d35",
                    )?),
                },
                height: Some(203998),
                is_coinbase: false,
            },
        );
        check_coin(
            "8ddf77bbd123008c988f1a4a4de2161e0f50aac7f17e7f9555caa4",
            Coin {
                tx_output: TxOutput {
                    value: 110_397,
                    script: Script::p2pkh(&ShaRmd160::from_hex(
                        "8c988f1a4a4de2161e0f50aac7f17e7f9555caa4",
                    )?),
                },
                height: Some(120891),
                is_coinbase: true,
            },
        );
        check_coin(
            "000006",
            Coin {
                tx_output: TxOutput::default(),
                height: Some(0),
                is_coinbase: false,
            },
        );
        check_coin_err(
            "000007",
            "Bytes error: Index 1 is out of bounds for array with length 0",
        );
        check_coin_err(
            "00008a95c0bb00",
            "Bytes error: Index 2999999994 is out of bounds for array with length 0",
        );
        Ok(())
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    #[test]
    fn test_height_amount() -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes: Bytes = hex!("32").into();
        let compressed = read_var_int(&mut bytes)?;
        let amount = decompress_amount(compressed);
        assert_eq!(amount, 50_000_000_00);

        let mut bytes: Bytes = hex!("03").into();
        let num = read_var_int(&mut bytes)?;
        assert_eq!(num, 3);
        assert_eq!(bytes.len(), 0);

        let mut bytes: Bytes = hex!("97f23c8358").into();
        let num = read_var_int(&mut bytes)?;
        assert_eq!(num & 1, 0); // IsCoinBase() == false
        assert_eq!(num >> 1, 203_998); // GetHeight() == 203998U
        let num = decompress_amount(read_var_int(&mut bytes)?);
        assert_eq!(num, 600_000_000_00);
        assert_eq!(bytes.len(), 0);

        let mut bytes: Bytes = hex!("8ddf77bbd123").into();
        let num = read_var_int(&mut bytes)?;
        assert_eq!(num & 1, 1); // IsCoinBase() == true
        assert_eq!(num >> 1, 120_891); // GetHeight() == 120891U
        let num = decompress_amount(read_var_int(&mut bytes)?);
        assert_eq!(num, 1103_97);
        assert_eq!(bytes.len(), 0);

        Ok(())
    }
}
