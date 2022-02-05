use bitcoinsuite_core::{Network, OutPoint, Sha256d, Tx, TxInput, TxOutput};

use crate::{SlpBurn, SlpToken, SlpTxData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichTx {
    pub tx: Tx,
    pub txid: Sha256d,
    pub block: Option<RichTxBlock>,
    pub slp_tx_data: Option<Box<SlpTxData>>,
    pub spent_outputs: Option<Vec<TxOutput>>,
    pub spends: Vec<Option<OutPoint>>,
    pub slp_burns: Vec<Option<Box<SlpBurn>>>,
    pub slp_error_msg: Option<String>,
    pub network: Network,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichTxBlock {
    pub height: i32,
    pub hash: Sha256d,
}

pub struct RichTxInput<'tx> {
    pub tx_input: &'tx TxInput,
    pub slp_burn: Option<&'tx SlpBurn>,
    pub slp_token: SlpToken,
    pub spent_output: Option<&'tx TxOutput>,
}

pub struct RichTxOutput<'tx> {
    pub tx_output: &'tx TxOutput,
    pub slp_token: SlpToken,
    pub spent_by: Option<&'tx OutPoint>,
}

impl RichTx {
    pub fn inputs(&self) -> impl ExactSizeIterator<Item = RichTxInput> {
        (0..self.tx.inputs().len()).map(|idx| RichTxInput {
            tx_input: &self.tx.inputs()[idx],
            slp_burn: self.slp_burns[idx].as_deref(),
            slp_token: self
                .slp_tx_data
                .as_ref()
                .and_then(|slp| slp.input_tokens.get(idx).cloned())
                .unwrap_or_default(),
            spent_output: self
                .spent_outputs
                .as_ref()
                .map(|spent_outputs| &spent_outputs[idx]),
        })
    }

    pub fn outputs(&self) -> impl ExactSizeIterator<Item = RichTxOutput> {
        (0..self.tx.outputs().len()).map(|idx| RichTxOutput {
            tx_output: &self.tx.outputs()[idx],
            slp_token: self
                .slp_tx_data
                .as_ref()
                .and_then(|slp| slp.output_tokens.get(idx).cloned())
                .unwrap_or_default(),
            spent_by: self.spends[idx].as_ref(),
        })
    }
}
