use crate::{opcode::*, BitcoinSuiteError, Bytes, BytesError, BytesMut, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op {
    Code(u8),
    Push(u8, Bytes),
}

impl Op {
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
