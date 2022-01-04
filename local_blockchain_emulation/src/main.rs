use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::{LinkedList, VecDeque};

#[derive(Clone, Debug, PartialEq)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Block {
    tr_data: Transaction,
    hash: String,
    prev_hash: String,
}

impl Block {
    fn generate_hash(&mut self) {
        let mut sum_string: String = "".to_owned();

        sum_string.push_str(&self.prev_hash);
        sum_string.push_str(&self.tr_data.from);
        sum_string.push_str(&self.tr_data.to);
        sum_string.push_str(&self.tr_data.amount.to_string());

        let mut hasher = Sha256::new();
        hasher.update(sum_string);
        self.hash = format!("{:x}", hasher.finalize());
    }
}
#[derive(Clone, Debug, PartialEq)]
struct Blockchain {
    chain: LinkedList<Block>,
    tr_queue: VecDeque<Transaction>,
}

impl Blockchain {
    fn new() -> Blockchain {
        let mut block_chain_tmp: LinkedList<Block> = LinkedList::new();
        let tr_queue_tmp: VecDeque<Transaction> = VecDeque::new();

        let genesis = Block {
            tr_data: Transaction {
                from: String::from("Satoshi"),
                to: String::from("GENESIS"),
                amount: 100_000_000,
            },
            hash: Utc::now().timestamp().to_string(),
            prev_hash: String::from("0"),
        };

        block_chain_tmp.push_back(genesis);

        Blockchain {
            chain: block_chain_tmp,
            tr_queue: tr_queue_tmp,
        }
    }

    fn new_transaction(&mut self, tr_amount: usize) {
        for _iter in 0..tr_amount {
            self.tr_queue.push_back(Transaction {
                from: String::from("sender"),
                to: String::from("receiver"),
                amount: rand::thread_rng().gen_range(0..100_000_000 as u64),
            });
        }
    }

    fn new_block(&mut self) {
        if self.tr_queue.is_empty() {
            println!("No transactions queued!");
            return;
        }

        let mut block = Block {
            tr_data: self.tr_queue.front().unwrap().clone(),
            hash: String::from(""),
            prev_hash: self.chain.back().unwrap().clone().hash,
        };

        block.generate_hash();

        self.chain.push_back(block);

        self.tr_queue.pop_front();
    }
}

fn main() {
    let mut blnch: Blockchain = Blockchain::new(); //override ctor?

    blnch.new_transaction(3); //queues transactions with a random amount

    blnch.new_block();
    blnch.new_block();

    println!("{:#?}", blnch.chain);
    println!("--------------------");
    println!("{:#?}", blnch.tr_queue);

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_blockchain_test() {
        let blnch: Blockchain = Blockchain::new();
        let genesis = Block {
            tr_data: Transaction {
                from: String::from("Satoshi"),
                to: String::from("GENESIS"),
                amount: 100_000_000,
            },
            hash: Utc::now().timestamp().to_string(),
            prev_hash: String::from("0"),
        };
        assert_eq!(blnch.chain.front().unwrap(), &genesis);
    }

    #[test]
    fn new_transaction_test() {
        let mut blnch: Blockchain = Blockchain::new();
        blnch.new_transaction(3);
        assert_eq!(blnch.tr_queue.len(), 3);
    }

    #[test]
    fn new_block_test() {
        let mut blnch: Blockchain = Blockchain::new();
        let tr_data = Transaction {
            from: String::from("Sender"),
            to: String::from("Reciever"),
            amount: 21u64,
        };
        blnch.tr_queue.push_back(tr_data);
        blnch.new_block();
        assert_eq!(blnch.chain.len(), 2);
        assert_eq!(blnch.tr_queue.len(), 0);
    }
}
