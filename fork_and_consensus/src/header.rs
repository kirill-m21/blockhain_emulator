use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub timestamp: String,
    pub nonce: BigInt,
}
