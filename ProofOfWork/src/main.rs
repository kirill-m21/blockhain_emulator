use sha2::{Sha256, Digest};
use std::{collections::{LinkedList, VecDeque}};
use chrono::Utc;
use num_bigint::{BigInt, RandomBits};
use rand::Rng;

#[derive(Clone)]
struct Transaction 
{
    from: String,
    to: String,
    amount: u64,
}

#[derive(Clone)]
struct Block 
{
    header: Header,
    tr_data: Transaction,
    hash: String,
    prev_hash: String,
}

/* impl Block 
{
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
} */

struct Blockchain 
{
    chain: LinkedList<Block>,
    tr_queue: VecDeque<Transaction>
}

impl Blockchain
{
    //ctor
    fn new() -> Blockchain 
    {
        let mut block_chain_tmp: LinkedList<Block> = LinkedList::new();
        let tr_queue_tmp: VecDeque<Transaction> = VecDeque::new();

        let genesis = Block 
        {
            header: Header 
            { 
                timestamp: (String::from("0")), 
                nonce: (BigInt::from(0)) 
            },
            tr_data: Transaction 
            {
                from: String::from("Satoshi"),
                to: String::from("GENESIS"),
                amount: 100_000_000,
            },
            hash: Utc::now().timestamp().to_string(),
            prev_hash: String::from("0"),
        };

        block_chain_tmp.push_back(genesis);

        Blockchain 
        { 
            chain: block_chain_tmp, 
            tr_queue: tr_queue_tmp,
        }
    }

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

    fn mint(&mut self)
    {
        if self.tr_queue.is_empty()
        {
            println!("No transactions queued!");
            return;
        }

        let mut new_block = Block
        {
            header: Header 
            { 
                timestamp: (String::from("0")), 
                nonce: (BigInt::from(0)) 
            },
             tr_data: self.tr_queue.front().unwrap().clone(),
             hash: String:: from(""),
             prev_hash: self.chain.back().unwrap().clone().hash,
             
        };

        loop
        {
            let mut sum_string: String = "".to_owned();
        
            sum_string.push_str(&new_block.prev_hash);
            sum_string.push_str(&new_block.tr_data.from);
            sum_string.push_str(&new_block.tr_data.to);
            sum_string.push_str(&new_block.tr_data.amount.to_string());

            new_block.header.nonce = rand::thread_rng().sample(RandomBits::new(256));
            sum_string.push_str(&new_block.header.nonce.to_string());

            let mut hasher = Sha256::new();
            hasher.update(sum_string);

            let check_hash = format!("{:x}", hasher.finalize());

            if check_hash.chars().filter(|&c| c == '1').count() >= 6
            {
            new_block.hash = check_hash;
            new_block.header.timestamp = Utc::now().timestamp().to_string();
            break;
            }
        }
        self.chain.push_back(new_block);
        self.tr_queue.pop_front();
    }
}

#[derive(Clone)]
struct Header
{
    timestamp: String,
    nonce: BigInt,
}
fn main() 
{
    let mut blnch : Blockchain = Blockchain::new();//override ctor?

    blnch.new_transaction(String::from("Alice"), String::from("Bob"), 100_000u64);
    blnch.new_transaction(String::from("Bob"), String::from("Charlie"), 60_000u64);
    blnch.new_transaction(String::from("Charlie"), String::from("Alice"), 30_000u64);
    blnch.new_transaction(String::from("Charlie"), String::from("Daniel"), 7_000u64);
    blnch.new_transaction(String::from("Daniel"), String::from("Bob"), 4_000u64);

    blnch.mint();
    blnch.mint();
    blnch.mint();
    blnch.mint();
    blnch.mint();
    blnch.mint();

    for element in blnch.chain.iter_mut()
    {
        println!("nonce: {}", element.header.nonce);
        println!("timestamp: {}", element.header.timestamp);
        println!("hash: {}", element.hash);
        println!("prev hash: {}", element.prev_hash);
        println!("from: {}", element.tr_data.from);
        println!("to: {}", element.tr_data.to);
        println!("amount: {}", element.tr_data.amount);
        println!("-------------------");
    }
}
