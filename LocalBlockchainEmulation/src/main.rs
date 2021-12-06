use sha2::{Sha256, Digest};
use std::collections::{LinkedList, VecDeque};
use chrono::Utc;

#[derive(Clone)]
struct Transaction {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Clone)]
struct Block {
    tr_data: Transaction,
    hash: String,
    prev_hash: String,
}

impl Block {

    fn generate_hash(&mut self)
    {
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
struct Blockchain {
    chain: LinkedList<Block>,
    tr_queue: VecDeque<Transaction>
}

impl Blockchain
{

    fn new_transaction(&mut self, _from : String, _to : String, _amount : u64)
    {
        let tmp_transaction: Transaction = Transaction 
        {
            from: _from,
            to: _to,
            amount: _amount,
        };
        self.tr_queue.push_back(tmp_transaction);
    }

     fn new_block(&mut self)
    {
        if self.chain.is_empty()
        {
            let tr_tmp: Transaction = Transaction 
            {
                from: String::from("Satoshi"),
                to: String::from("GENESIS"),
                amount: 100_000_000,
            };

            let block = Block 
            {
                tr_data: tr_tmp,
                hash: Utc::now().timestamp().to_string(),
                prev_hash: String::from("0"),
            };

            self.chain.push_back(block);

        }

        if self.tr_queue.is_empty()
        {
            println!("No transactions queued!");
            return;
        }

        let mut block = Block
        {
             tr_data: self.tr_queue.front().unwrap().clone(),
             hash: String:: from(""),
             prev_hash: self.chain.back().unwrap().clone().hash,
             
        };

        block.generate_hash();

        self.chain.push_back(block);

        self.tr_queue.pop_front();
    }
}

fn main() {

    let mut blnch : Blockchain = Blockchain 
    {
        chain: LinkedList::<Block>::new(),
        tr_queue: VecDeque::<Transaction>::new(),
    };

    blnch.new_transaction(String::from("Alice"), String::from("Bob"), 100_000u64);
    blnch.new_transaction(String::from("Bob"), String::from("Charlie"), 60_000u64);
    blnch.new_transaction(String::from("Charlie"), String::from("Alice"), 30_000u64);
    blnch.new_transaction(String::from("Charlie"), String::from("Daniel"), 7_000u64);
    blnch.new_transaction(String::from("Daniel"), String::from("Bob"), 4_000u64);

    blnch.new_block();
    blnch.new_block();
    blnch.new_block();
    blnch.new_block();
    blnch.new_block();
    blnch.new_block(); //No Transactions queued

    for element in blnch.chain.iter_mut()
    {
        println!("hash: {}", element.hash);
        println!("prev hash: {}", element.prev_hash);
        println!("from: {}", element.tr_data.from);
        println!("to: {}", element.tr_data.to);
        println!("amount: {}", element.tr_data.amount);
        println!("-------------------");
    }
}
