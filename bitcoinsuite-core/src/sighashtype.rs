use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SigHashType {
    pub variant: SigHashTypeVariant,
    pub input_type: SigHashTypeInputs,
    pub output_type: SigHashTypeOutputs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SigHashTypeInputs {
    Fixed,
    AnyoneCanPay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SigHashTypeOutputs {
    All,
    None,
    Single,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SigHashTypeVariant {
    Legacy,
    Bip143,
}

impl SigHashType {
    pub const ALL_BIP143: SigHashType = SigHashType {
        variant: SigHashTypeVariant::Bip143,
        input_type: SigHashTypeInputs::Fixed,
        output_type: SigHashTypeOutputs::All,
    };
    pub const NONE_BIP143: SigHashType = SigHashType {
        variant: SigHashTypeVariant::Bip143,
        input_type: SigHashTypeInputs::Fixed,
        output_type: SigHashTypeOutputs::None,
    };
    pub const SINGLE_BIP143: SigHashType = SigHashType {
        variant: SigHashTypeVariant::Bip143,
        input_type: SigHashTypeInputs::Fixed,
        output_type: SigHashTypeOutputs::Single,
    };
    pub const ALL_BIP143_ANYONECANPAY: SigHashType = SigHashType {
        variant: SigHashTypeVariant::Bip143,
        input_type: SigHashTypeInputs::AnyoneCanPay,
        output_type: SigHashTypeOutputs::All,
    };
    pub const NONE_BIP143_ANYONECANPAY: SigHashType = SigHashType {
        variant: SigHashTypeVariant::Bip143,
        input_type: SigHashTypeInputs::AnyoneCanPay,
        output_type: SigHashTypeOutputs::None,
    };
    pub const SINGLE_BIP143_ANYONECANPAY: SigHashType = SigHashType {
        variant: SigHashTypeVariant::Bip143,
        input_type: SigHashTypeInputs::AnyoneCanPay,
        output_type: SigHashTypeOutputs::Single,
    };

    pub fn to_u32(&self) -> u32 {
        self.input_type.to_u32() | self.output_type.to_u32() | self.variant.to_u32()
    }
}

impl SigHashTypeInputs {
    pub fn to_u32(&self) -> u32 {
        match self {
            SigHashTypeInputs::Fixed => 0x00,
            SigHashTypeInputs::AnyoneCanPay => 0x80,
        }
    }
}

impl SigHashTypeOutputs {
    pub fn to_u32(&self) -> u32 {
        match self {
            SigHashTypeOutputs::All => 1,
            SigHashTypeOutputs::None => 2,
            SigHashTypeOutputs::Single => 3,
        }
    }
}

impl SigHashTypeVariant {
    pub fn to_u32(&self) -> u32 {
        match self {
            SigHashTypeVariant::Legacy => 0x00,
            SigHashTypeVariant::Bip143 => 0x40,
        }
    }
}

impl Display for SigHashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.output_type {
            SigHashTypeOutputs::All => write!(f, "ALL")?,
            SigHashTypeOutputs::None => write!(f, "NONE")?,
            SigHashTypeOutputs::Single => write!(f, "SINGLE")?,
        }
        if let SigHashTypeVariant::Bip143 = self.variant {
            write!(f, "|FORKID")?;
        }
        if let SigHashTypeInputs::AnyoneCanPay = self.input_type {
            write!(f, "|ANYONECANPAY")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::SigHashType;

    #[test]
    fn test_sighash_display() {
        assert_eq!(SigHashType::ALL_BIP143.to_string(), "ALL|FORKID");
        assert_eq!(SigHashType::NONE_BIP143.to_string(), "NONE|FORKID");
        assert_eq!(SigHashType::SINGLE_BIP143.to_string(), "SINGLE|FORKID");
        assert_eq!(
            SigHashType::ALL_BIP143_ANYONECANPAY.to_string(),
            "ALL|FORKID|ANYONECANPAY"
        );
        assert_eq!(
            SigHashType::NONE_BIP143_ANYONECANPAY.to_string(),
            "NONE|FORKID|ANYONECANPAY"
        );
        assert_eq!(
            SigHashType::SINGLE_BIP143_ANYONECANPAY.to_string(),
            "SINGLE|FORKID|ANYONECANPAY"
        );
    }

    #[test]
    fn test_sighash_to_u32() {
        assert_eq!(SigHashType::ALL_BIP143.to_u32(), 0x41);
        assert_eq!(SigHashType::NONE_BIP143.to_u32(), 0x42);
        assert_eq!(SigHashType::SINGLE_BIP143.to_u32(), 0x43);
        assert_eq!(SigHashType::ALL_BIP143_ANYONECANPAY.to_u32(), 0xc1);
        assert_eq!(SigHashType::NONE_BIP143_ANYONECANPAY.to_u32(), 0xc2);
        assert_eq!(SigHashType::SINGLE_BIP143_ANYONECANPAY.to_u32(), 0xc3);
    }
}
