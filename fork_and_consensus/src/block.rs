
pub mod transaction;
use transaction::Transaction;
pub mod header;
use header::Header;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub header: Header,
    pub tr_data: Transaction,
    pub hash: String,
    pub prev_hash: String,
}
