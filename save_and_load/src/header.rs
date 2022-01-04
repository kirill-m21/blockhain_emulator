use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Header {
    pub timestamp: String,
    pub nonce: u128,
}
