use once_cell::sync::Lazy;
use regex::Regex;

use crate::{opcode::*, ser_script_num, BitcoinSuiteError, Bytes, BytesError, BytesMut, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op {
    Code(u8),
    Push(u8, Bytes),
}

impl Op {
    pub fn push_script_num(num: i32) -> Self {
        if num == 0 {
            return Op::Code(OP_0);
        }
        if num == -1 {
            return Op::Code(OP_1NEGATE);
        }
        if (1..=16).contains(&num) {
            return Op::Code(num as u8 + OP_1 - 1);
        }
        let bytes = ser_script_num(num);
        Op::Push(bytes.len() as u8, bytes)
    }

    pub fn push_bytes(bytes: Bytes) -> Self {
        match bytes.len() {
            0 => Op::Code(OP_0),
            0x01..=0x4b => Op::Push(bytes.len() as u8, bytes),
            0x4c..=0xff => Op::Push(OP_PUSHDATA1, bytes),
            0x100..=0xffff => Op::Push(OP_PUSHDATA2, bytes),
            0x10000..=0xffffffff => Op::Push(OP_PUSHDATA4, bytes),
            _ => panic!("Bytes way too large"),
        }
    }

    pub fn deser_op(data: &mut Bytes) -> std::result::Result<Op, BytesError> {
        let opcode = data.split_to(1)?[0];
        Ok(match opcode {
            0x01..=0x4b => Op::Push(opcode, data.split_to(opcode as usize)?),
            OP_PUSHDATA1 => {
                let size = data.split_to(1)?[0] as usize;
                Op::Push(opcode, data.split_to(size)?)
            }
            OP_PUSHDATA2 => {
                let size = u16::from_le_bytes(data.split_to_array::<2>()?.array());
                Op::Push(opcode, data.split_to(size as usize)?)
            }
            OP_PUSHDATA4 => {
                let size = u32::from_le_bytes(data.split_to_array::<4>()?.array());
                Op::Push(opcode, data.split_to(size as usize)?)
            }
            _ => Op::Code(opcode),
        })
    }

    pub fn ser_op(&self, data: &mut BytesMut) -> Result<()> {
        match *self {
            Op::Code(opcode) => data.put_slice(&[opcode]),
            Op::Push(opcode, ref bytes) => {
                data.put_slice(&[opcode]);
                match opcode {
                    0x01..=0x4b => {}
                    OP_PUSHDATA1 => data.put_slice(&[bytes.len() as u8]),
                    OP_PUSHDATA2 => data.put_slice(&(bytes.len() as u16).to_le_bytes()),
                    OP_PUSHDATA4 => data.put_slice(&(bytes.len() as u32).to_le_bytes()),
                    _ => return Err(BitcoinSuiteError::InconsistentOpPush(opcode)),
                };
                data.put_slice(bytes);
            }
        }
        Ok(())
    }
}

static TEXT_HEURISTIC: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[\w\s\p{Punctuation}\p{Symbol}\p{Emoji}]+$").unwrap());

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Code(code) => f.write_str(match *code {
                // push value
                OP_0 => "OP_FALSE",
                OP_1 => "OP_TRUE",
                OP_2 => "OP_2",
                OP_3 => "OP_3",
                OP_4 => "OP_4",
                OP_5 => "OP_5",
                OP_6 => "OP_6",
                OP_7 => "OP_7",
                OP_8 => "OP_8",
                OP_9 => "OP_9",
                OP_10 => "OP_10",
                OP_11 => "OP_11",
                OP_12 => "OP_12",
                OP_13 => "OP_13",
                OP_14 => "OP_14",
                OP_15 => "OP_15",
                OP_16 => "OP_16",

                // control
                OP_NOP => "OP_NOP",
                OP_SCRIPTTYPE => "OP_SCRIPTTYPE", // Lotus only
                OP_IF => "OP_IF",
                OP_NOTIF => "OP_NOTIF",
                OP_VERIF => "OP_VERIF",
                OP_VERNOTIF => "OP_VERNOTIF",
                OP_ELSE => "OP_ELSE",
                OP_ENDIF => "OP_ENDIF",
                OP_VERIFY => "OP_VERIFY",
                OP_RETURN => "OP_RETURN",

                // stack ops
                OP_TOALTSTACK => "OP_TOALTSTACK",
                OP_FROMALTSTACK => "OP_FROMALTSTACK",
                OP_2DROP => "OP_2DROP",
                OP_2DUP => "OP_2DUP",
                OP_3DUP => "OP_3DUP",
                OP_2OVER => "OP_2OVER",
                OP_2ROT => "OP_2ROT",
                OP_2SWAP => "OP_2SWAP",
                OP_IFDUP => "OP_IFDUP",
                OP_DEPTH => "OP_DEPTH",
                OP_DROP => "OP_DROP",
                OP_DUP => "OP_DUP",
                OP_NIP => "OP_NIP",
                OP_OVER => "OP_OVER",
                OP_PICK => "OP_PICK",
                OP_ROLL => "OP_ROLL",
                OP_ROT => "OP_ROT",
                OP_SWAP => "OP_SWAP",
                OP_TUCK => "OP_TUCK",

                // splice ops
                OP_CAT => "OP_CAT",
                OP_SPLIT => "OP_SPLIT", // after monolith upgrade (May 2018)
                OP_NUM2BIN => "OP_NUM2BIN", // after monolith upgrade (May 2018)
                OP_BIN2NUM => "OP_BIN2NUM", // after monolith upgrade (May 2018)
                OP_SIZE => "OP_SIZE",

                // bit logic
                OP_INVERT => "OP_INVERT",
                OP_AND => "OP_AND",
                OP_OR => "OP_OR",
                OP_XOR => "OP_XOR",
                OP_EQUAL => "OP_EQUAL",
                OP_EQUALVERIFY => "OP_EQUALVERIFY",
                OP_RESERVED1 => "OP_RESERVED1",
                OP_RESERVED2 => "OP_RESERVED2",

                // numeric
                OP_1ADD => "OP_1ADD",
                OP_1SUB => "OP_1SUB",
                OP_2MUL => "OP_2MUL",
                OP_2DIV => "OP_2DIV",
                OP_NEGATE => "OP_NEGATE",
                OP_ABS => "OP_ABS",
                OP_NOT => "OP_NOT",
                OP_0NOTEQUAL => "OP_0NOTEQUAL",

                OP_ADD => "OP_ADD",
                OP_SUB => "OP_SUB",
                OP_MUL => "OP_MUL",
                OP_DIV => "OP_DIV",
                OP_MOD => "OP_MOD",
                OP_RAWLEFTBITSHIFT => "OP_RAWLEFTBITSHIFT", // Lotus only
                OP_MULPOW2 => "OP_MULPOW2",                 // Lotus only

                OP_BOOLAND => "OP_BOOLAND",
                OP_BOOLOR => "OP_BOOLOR",
                OP_NUMEQUAL => "OP_NUMEQUAL",
                OP_NUMEQUALVERIFY => "OP_NUMEQUALVERIFY",
                OP_NUMNOTEQUAL => "OP_NUMNOTEQUAL",
                OP_LESSTHAN => "OP_LESSTHAN",
                OP_GREATERTHAN => "OP_GREATERTHAN",
                OP_LESSTHANOREQUAL => "OP_LESSTHANOREQUAL",
                OP_GREATERTHANOREQUAL => "OP_GREATERTHANOREQUAL",
                OP_MIN => "OP_MIN",
                OP_MAX => "OP_MAX",

                OP_WITHIN => "OP_WITHIN",

                // crypto
                OP_RIPEMD160 => "OP_RIPEMD160",
                OP_SHA1 => "OP_SHA1",
                OP_SHA256 => "OP_SHA256",
                OP_HASH160 => "OP_HASH160",
                OP_HASH256 => "OP_HASH256",
                OP_CODESEPARATOR => "OP_CODESEPARATOR",
                OP_CHECKSIG => "OP_CHECKSIG",
                OP_CHECKSIGVERIFY => "OP_CHECKSIGVERIFY",
                OP_CHECKMULTISIG => "OP_CHECKMULTISIG",
                OP_CHECKMULTISIGVERIFY => "OP_CHECKMULTISIGVERIFY",

                // expansion
                OP_NOP1 => "OP_NOP1",
                OP_CHECKLOCKTIMEVERIFY => "OP_CHECKLOCKTIMEVERIFY",
                OP_CHECKSEQUENCEVERIFY => "OP_CHECKSEQUENCEVERIFY",
                OP_NOP4 => "OP_NOP4",
                OP_NOP5 => "OP_NOP5",
                OP_NOP6 => "OP_NOP6",
                OP_NOP7 => "OP_NOP7",
                OP_NOP8 => "OP_NOP8",
                OP_NOP9 => "OP_NOP9",
                OP_NOP10 => "OP_NOP10",

                // More crypto
                OP_CHECKDATASIG => "OP_CHECKDATASIG",
                OP_CHECKDATASIGVERIFY => "OP_CHECKDATASIGVERIFY",

                // additional byte string operations
                OP_REVERSEBYTES => "OP_REVERSEBYTES",

                // multi-byte opcodes
                OP_PREFIX_BEGIN => "OP_PREFIX_BEGIN",
                OP_PREFIX_END => "OP_PREFIX_END",

                OP_INVALIDOPCODE => "OP_INVALIDOPCODE",

                _ => "[unrecognized opcode]",
            }),
            Op::Push(_, data) => {
                if data.is_empty() {
                    return write!(f, "\"\"");
                }
                if let Ok(text) = std::str::from_utf8(data) {
                    if TEXT_HEURISTIC.is_match(text.trim_end_matches('\0')) {
                        return write!(f, "\"{}\"", text.replace('\0', "\\0"));
                    }
                }
                write!(f, "0x{}", data.hex())
            }
        }
    }
}
