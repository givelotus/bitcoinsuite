use crate::{
    ecc::{DummyEcc, Ecc},
    BitcoinCode, BitcoinSuiteError, Script, SignError, Signatory, TxInput, TxOutput, UnhashedTx,
    UnsignedTx,
};

use crate::sign::error::Result;

#[derive(Default)]
pub struct TxBuilder {
    pub version: i32,
    pub inputs: Vec<TxBuilderInput>,
    pub outputs: Vec<TxBuilderOutput>,
    pub lock_time: u32,
}

pub struct TxBuilderInput {
    input: TxInput,
    signatory: Option<Box<dyn Signatory>>,
}

pub enum TxBuilderOutput {
    Fixed(TxOutput),
    Leftover(Script),
}

impl TxBuilder {
    pub fn from_tx(tx: UnhashedTx) -> Self {
        TxBuilder {
            version: tx.version,
            inputs: tx
                .inputs
                .into_iter()
                .map(TxBuilderInput::from_input)
                .collect(),
            outputs: tx.outputs.into_iter().map(TxBuilderOutput::Fixed).collect(),
            lock_time: tx.lock_time,
        }
    }

    fn input_sum(&self) -> Option<i64> {
        let mut input_sum = 0;
        for builder_input in &self.inputs {
            match &builder_input.input.sign_data {
                Some(sign_data) => match sign_data.find_value() {
                    Ok(value) => input_sum += value,
                    Err(SignError::MissingValue) => return None,
                    _ => unreachable!(),
                },
                None => return None,
            }
        }
        Some(input_sum)
    }

    fn prepare_outputs(
        builder_outputs: Vec<TxBuilderOutput>,
    ) -> Result<(i64, Option<usize>, Vec<TxOutput>)> {
        let mut fixed_output_sum = 0;
        let mut leftover_idx = None;
        let mut outputs = Vec::with_capacity(builder_outputs.len());
        for (output_idx, builder_output) in builder_outputs.into_iter().enumerate() {
            match builder_output {
                TxBuilderOutput::Fixed(output) => {
                    fixed_output_sum += output.value;
                    outputs.push(output);
                }
                TxBuilderOutput::Leftover(script) => {
                    if leftover_idx.is_some() {
                        return Err(SignError::MultipleLeftover);
                    }
                    leftover_idx = Some(output_idx);
                    outputs.push(TxOutput { value: 0, script })
                }
            }
        }
        Ok((fixed_output_sum, leftover_idx, outputs))
    }

    pub fn sign(
        self,
        ecc: &dyn Ecc,
        fee_per_kb: i64,
        dust_limit: i64,
    ) -> std::result::Result<UnhashedTx, BitcoinSuiteError> {
        let input_sum = self.input_sum();
        let (inputs, signatories): (Vec<_>, Vec<_>) = self
            .inputs
            .into_iter()
            .map(|input| (input.input, input.signatory))
            .unzip();
        let (fixed_output_sum, leftover_idx, mut outputs) = Self::prepare_outputs(self.outputs)?;
        // If we have a leftover output, we need to measure the tx size and adjust the outputs
        if let Some(leftover_idx) = leftover_idx {
            let input_sum = match input_sum {
                Some(input_sum) => input_sum,
                None => return Err(SignError::MissingValue.into()),
            };
            let mut dummy_unsigned_tx = UnsignedTx::new_dummy(UnhashedTx {
                version: self.version,
                inputs: inputs.clone(),
                outputs,
                lock_time: self.lock_time,
            });
            for (input_idx, signatory) in signatories.iter().enumerate() {
                if let Some(signatory) = signatory {
                    signatory.sign_input(&DummyEcc, dummy_unsigned_tx.input_at(input_idx))?;
                }
            }
            let mut tx_size = dummy_unsigned_tx.tx().ser().len();
            let mut tx_fee = tx_size as i64 * fee_per_kb / 1000;
            let mut new_outputs = dummy_unsigned_tx.into_tx().outputs;
            // inputs cannot pay for a dust leftover -> remove
            let leftover_value = input_sum - (fixed_output_sum + tx_fee);
            if leftover_value < dust_limit {
                let output = new_outputs.remove(leftover_idx);
                tx_size -= output.ser().len();
                tx_fee = tx_size as i64 * fee_per_kb / 1000;
            } else {
                new_outputs[leftover_idx].value = leftover_value;
            }
            if input_sum < fixed_output_sum + tx_fee {
                return Err(SignError::InsufficientInputsForFee {
                    input_sum,
                    max_fee: input_sum - fixed_output_sum,
                    required_fee: tx_fee,
                }
                .into());
            }
            outputs = new_outputs;
        }
        let mut unsigned_tx = UnsignedTx::new(UnhashedTx {
            version: self.version,
            inputs,
            outputs,
            lock_time: self.lock_time,
        });
        for (input_idx, signatory) in signatories.iter().enumerate() {
            if let Some(signatory) = signatory {
                signatory.sign_input(ecc, unsigned_tx.input_at(input_idx))?;
            }
        }
        Ok(unsigned_tx.into_tx())
    }
}

impl TxBuilderInput {
    pub fn new(input: TxInput, signatory: Box<dyn Signatory>) -> Self {
        TxBuilderInput {
            input,
            signatory: Some(signatory),
        }
    }

    pub fn from_input(input: TxInput) -> Self {
        TxBuilderInput {
            input,
            signatory: None,
        }
    }

    pub fn signatory(&self) -> &Option<Box<dyn Signatory>> {
        &self.signatory
    }

    pub fn signatory_mut(&mut self) -> &mut Option<Box<dyn Signatory>> {
        &mut self.signatory
    }

    pub fn input(&self) -> &TxInput {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut TxInput {
        &mut self.input
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ecc::{DummyEcc, Ecc},
        BitcoinCode, BitcoinSuiteError, OutPoint, Result, Script, SequenceNo, SignData, SignError,
        SignField, Signatory, TxBuilder, TxBuilderOutput, TxInput, TxOutput, UnhashedTx,
        UnsignedTxInput,
    };

    pub struct ConstSignatory(Script);
    impl Signatory for ConstSignatory {
        fn sign_input<'tx>(&self, _: &dyn Ecc, mut input: UnsignedTxInput<'tx>) -> Result<()> {
            *input.input_script_mut() = self.0.clone();
            Ok(())
        }
    }

    #[test]
    fn test_sign() -> Result<()> {
        let tx = UnhashedTx {
            version: 1,
            inputs: vec![TxInput {
                prev_out: OutPoint::default(),
                script: Script::default(),
                sequence: SequenceNo::finalized(),
                sign_data: None,
            }],
            outputs: vec![TxOutput::default()],
            lock_time: 0,
        };
        {
            // No leftover
            let mut tx_builder = TxBuilder::from_tx(tx.clone());
            let script = Script::from_slice(&[0x01, 0x51]);
            *tx_builder.inputs[0].signatory_mut() = Some(Box::new(ConstSignatory(script.clone())));
            let signed_tx = tx_builder.sign(&DummyEcc, 1000, 546)?;
            assert_eq!(signed_tx.inputs[0].script, script);
            assert_eq!(signed_tx.outputs, vec![TxOutput::default()]);
        }
        {
            // Leftover
            let mut tx = tx.clone();
            tx.inputs[0].sign_data = Some(SignData::new(vec![SignField::Value(10000)]));
            tx.outputs[0].value = 2000;
            let mut tx_builder = TxBuilder::from_tx(tx);
            let leftover_script = Script::from_slice(&[52]).to_p2sh();
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script.clone()));
            let script = Script::from_slice(&[0; 8]);
            *tx_builder.inputs[0].signatory_mut() = Some(Box::new(ConstSignatory(script.clone())));
            let signed_tx = tx_builder.sign(&DummyEcc, 1000, 546)?;
            let tx_size = signed_tx.ser().len();
            assert_eq!(tx_size, 100);
            assert_eq!(signed_tx.inputs[0].script, script);
            assert_eq!(
                signed_tx.outputs,
                vec![
                    TxOutput {
                        value: 2000,
                        script: Script::from_slice(&[]),
                    },
                    TxOutput {
                        value: 8000 - tx_size as i64,
                        script: leftover_script,
                    },
                ]
            );
        }
        {
            // Leftover = dust
            let mut tx = tx.clone();
            tx.inputs[0].sign_data = Some(SignData::new(vec![SignField::Value(10000)]));
            tx.outputs[0].value = 9400;
            let mut tx_builder = TxBuilder::from_tx(tx);
            let leftover_script = Script::from_slice(&[52]).to_p2sh();
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script.clone()));
            let script = Script::from_slice(&[0; 8]);
            *tx_builder.inputs[0].signatory_mut() = Some(Box::new(ConstSignatory(script.clone())));
            let signed_tx = tx_builder.sign(&DummyEcc, 1000, 500)?;
            let tx_size = signed_tx.ser().len();
            assert_eq!(tx_size, 100);
            assert_eq!(signed_tx.inputs[0].script, script);
            assert_eq!(
                signed_tx.outputs,
                vec![
                    TxOutput {
                        value: 9400,
                        script: Script::from_slice(&[]),
                    },
                    TxOutput {
                        value: 500,
                        script: leftover_script,
                    },
                ]
            );
        }
        {
            // Leftover < dust
            let mut tx = tx.clone();
            tx.inputs[0].sign_data = Some(SignData::new(vec![SignField::Value(10000)]));
            tx.outputs[0].value = 9401;
            let mut tx_builder = TxBuilder::from_tx(tx);
            let leftover_script = Script::from_slice(&[52]).to_p2sh();
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script));
            let script = Script::from_slice(&[0; 8]);
            *tx_builder.inputs[0].signatory_mut() = Some(Box::new(ConstSignatory(script.clone())));
            let signed_tx = tx_builder.sign(&DummyEcc, 1000, 500)?;
            let tx_size = signed_tx.ser().len();
            assert_eq!(tx_size, 68);
            assert_eq!(signed_tx.inputs[0].script, script);
            assert_eq!(
                signed_tx.outputs,
                vec![TxOutput {
                    value: 9401,
                    script: Script::from_slice(&[]),
                },]
            );
        }
        {
            // Error: missing value
            let mut tx_builder = TxBuilder::from_tx(tx.clone());
            let leftover_script = Script::from_slice(&[52]).to_p2sh();
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script));
            match tx_builder.sign(&DummyEcc, 1000, 546) {
                Err(BitcoinSuiteError::Sign(SignError::MissingValue)) => {}
                result => panic!("Unexpected: {:?}", result),
            }
        }
        {
            // Error: insufficient inputs for fee
            let mut tx = tx.clone();
            tx.inputs[0].sign_data = Some(SignData::new(vec![SignField::Value(1000)]));
            tx.outputs[0].value = 999;
            let mut tx_builder = TxBuilder::from_tx(tx);
            let leftover_script = Script::from_slice(&[52]).to_p2sh();
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script));
            let script = Script::from_slice(&[0; 8]);
            *tx_builder.inputs[0].signatory_mut() = Some(Box::new(ConstSignatory(script)));
            match tx_builder.sign(&DummyEcc, 1000, 500) {
                Err(BitcoinSuiteError::Sign(SignError::InsufficientInputsForFee {
                    input_sum: 1000,
                    required_fee: 68,
                    max_fee: 1,
                })) => {}
                result => panic!("Unexpected: {:?}", result),
            }
        }
        {
            // Error: multiple leftover
            let mut tx = tx;
            tx.inputs[0].sign_data = Some(SignData::new(vec![SignField::Value(10000)]));
            tx.outputs[0].value = 2000;
            let mut tx_builder = TxBuilder::from_tx(tx);
            let leftover_script = Script::from_slice(&[52]).to_p2sh();
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script.clone()));
            tx_builder
                .outputs
                .push(TxBuilderOutput::Leftover(leftover_script));
            match tx_builder.sign(&DummyEcc, 1000, 546) {
                Err(BitcoinSuiteError::Sign(SignError::MultipleLeftover)) => {}
                result => panic!("Unexpected: {:?}", result),
            }
        }
        Ok(())
    }
}
