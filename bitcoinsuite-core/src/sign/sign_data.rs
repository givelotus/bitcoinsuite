use crate::{Script, SignError};

use crate::sign::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SignData {
    fields: Vec<SignField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignField {
    OutputScript(Script),
    RedeemScript(Script),
    Value(i64),
}

impl SignData {
    pub fn new(fields: Vec<SignField>) -> Self {
        SignData { fields }
    }
}

impl SignData {
    pub fn find_script_code(&self) -> Result<Script> {
        let mut is_p2sh = false;
        for field in &self.fields {
            match field {
                SignField::OutputScript(script) => {
                    if !script.is_p2sh() {
                        return Ok(script.clone());
                    }
                    is_p2sh = true;
                }
                SignField::RedeemScript(script) => {
                    return Ok(script.clone());
                }
                _ => {}
            }
        }
        match is_p2sh {
            true => Err(SignError::MissingScriptCodeP2SH),
            false => Err(SignError::MissingScriptCode),
        }
    }

    pub fn find_value(&self) -> Result<i64> {
        for field in &self.fields {
            if let &SignField::Value(value) = field {
                return Ok(value);
            }
        }
        Err(SignError::MissingValue)
    }
}

#[cfg(test)]
mod tests {
    use crate::{sign::error::Result, Script, SignData, SignError, SignField};

    #[test]
    fn test_find_script_code_success() -> Result<()> {
        let sign_data = SignData::new(vec![SignField::OutputScript(Script::from_slice(&[0x51]))]);
        assert_eq!(sign_data.find_script_code()?, Script::from_slice(&[0x51]));

        let sign_data = SignData::new(vec![SignField::RedeemScript(Script::from_slice(&[0x51]))]);
        assert_eq!(sign_data.find_script_code()?, Script::from_slice(&[0x51]));

        let sign_data = SignData::new(vec![
            SignField::OutputScript(Script::from_slice(&[0x51]).to_p2sh()),
            SignField::RedeemScript(Script::from_slice(&[0x51])),
            SignField::Value(1234),
        ]);
        assert_eq!(sign_data.find_script_code()?, Script::from_slice(&[0x51]));

        let sign_data = SignData::new(vec![
            SignField::OutputScript(Script::from_slice(&[0x99]).to_p2sh()),
            SignField::RedeemScript(Script::from_slice(&[0x55])),
        ]);
        assert_eq!(sign_data.find_script_code()?, Script::from_slice(&[0x55]));
        Ok(())
    }

    #[test]
    fn test_find_script_code_failure() {
        let sign_data = SignData::new(vec![]);
        assert_eq!(
            sign_data.find_script_code().unwrap_err(),
            SignError::MissingScriptCode
        );
        let sign_data = SignData::new(vec![SignField::Value(1234)]);
        assert_eq!(
            sign_data.find_script_code().unwrap_err(),
            SignError::MissingScriptCode
        );
        let sign_data = SignData::new(vec![SignField::OutputScript(
            Script::from_slice(&[0x51]).to_p2sh(),
        )]);
        assert_eq!(
            sign_data.find_script_code().unwrap_err(),
            SignError::MissingScriptCodeP2SH
        );
        let sign_data = SignData::new(vec![
            SignField::OutputScript(Script::from_slice(&[0x51]).to_p2sh()),
            SignField::Value(1234),
        ]);
        assert_eq!(
            sign_data.find_script_code().unwrap_err(),
            SignError::MissingScriptCodeP2SH
        );
    }

    #[test]
    fn test_find_value() -> Result<()> {
        let sign_data = SignData::new(vec![SignField::Value(1234)]);
        assert_eq!(sign_data.find_value()?, 1234);

        let sign_data = SignData::new(vec![
            SignField::OutputScript(Script::from_slice(&[0x51])),
            SignField::Value(1234),
        ]);
        assert_eq!(sign_data.find_value()?, 1234);

        let sign_data = SignData::new(vec![]);
        assert_eq!(sign_data.find_value().unwrap_err(), SignError::MissingValue);

        let sign_data = SignData::new(vec![SignField::OutputScript(Script::from_slice(&[0x51]))]);
        assert_eq!(sign_data.find_value().unwrap_err(), SignError::MissingValue);

        Ok(())
    }
}
