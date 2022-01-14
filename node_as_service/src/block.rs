use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub mod transaction;
use transaction::Transaction;
pub mod header;
use header::Header;

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Block {
    pub header: Header,
    pub tr_data: Transaction,
    pub hash: String,
    pub prev_hash: String,
}
