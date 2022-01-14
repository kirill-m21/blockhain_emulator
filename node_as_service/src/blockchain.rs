use borsh::{BorshDeserialize, BorshSerialize};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::prelude::*;
use std::time::Duration;
use std::{
    collections::{LinkedList, VecDeque},
    time::Instant,
};

#[path = "block.rs"]
pub mod block;
use block::header::Header;
use block::transaction::Transaction;
use block::Block;

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Blockchain {
    pub chain: LinkedList<Block>,
    pub tr_queue: VecDeque<Transaction>,
}

impl Blockchain {
    //ctor
    pub fn new() -> Blockchain {
        let mut block_chain_tmp: LinkedList<Block> = LinkedList::new();
        let tr_queue_tmp: VecDeque<Transaction> = VecDeque::new();

        let genesis = Block {
            header: Header {
                timestamp: (String::from("0")),
                nonce: (0u128),
            },
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

    pub fn new_transaction(&mut self, from: String, to: String, amount: u64) {
        self.tr_queue.push_back(Transaction { from, to, amount });
    }

    pub fn mint(&mut self) {
        if self.tr_queue.is_empty() {
            println!("No transactions queued!");
            return;
        }

        let mut new_block = Block {
            header: Header {
                timestamp: (String::from("0")),
                nonce: (0u128),
            },
            tr_data: self.tr_queue.front().unwrap().clone(),
            hash: String::from(""),
            prev_hash: self.chain.back().unwrap().clone().hash,
        };

        loop {
            let mut sum_string: String = "".to_owned();

            sum_string.push_str(&new_block.prev_hash);
            sum_string.push_str(&new_block.tr_data.from);
            sum_string.push_str(&new_block.tr_data.to);
            sum_string.push_str(&new_block.tr_data.amount.to_string());

            new_block.header.nonce = rand::thread_rng().gen::<u128>();
            sum_string.push_str(&new_block.header.nonce.to_string());

            let mut hasher = Sha256::new();
            hasher.update(sum_string);

            let check_hash = format!("{:x}", hasher.finalize());

            if check_hash.chars().filter(|&c| c == '1').count() >= 6 {
                new_block.hash = check_hash;
                new_block.header.timestamp = Utc::now().timestamp().to_string();
                break;
            }
        }
        self.chain.push_back(new_block);
        self.tr_queue.pop_front();
    }

    pub fn fork(&mut self, fork_duration: u64) {
        let duration = Duration::new(fork_duration, 0);
        let time_new_block = Duration::new(1, 0);
        let time_new_fork = Duration::new(5, 0);
        let time_longest = Duration::new(37, 0);
        let mut loop_duration;
        let mut block_duration; //time to create point
        let mut fork_duration; //time to create fork
        let mut longest_chain_duration; //time to choice longest chain

        let time_loop_stop = Instant::now();
        let mut time_block_create = Instant::now(); //time point to add block
        let mut time_block_fork = Instant::now(); //time point to add fork
        let mut time_block_chains = Instant::now(); //time point to select the longest chain

        let mut tmp_blockchain = Blockchain::new();
        let mut vec_blnch: Vec<Blockchain> = Vec::new();

        tmp_blockchain.chain.pop_front();
        tmp_blockchain
            .chain
            .push_back(self.chain.back().unwrap().clone());

        vec_blnch.push(tmp_blockchain.clone());

        loop {
            loop_duration = time_loop_stop.elapsed();
            if loop_duration >= duration {
                break;
            }

            block_duration = time_block_create.elapsed();
            if block_duration >= time_new_block {
                let rand_num = rand::thread_rng().gen_range(0..vec_blnch.len());
                let mut tmp_blnch = vec_blnch[rand_num].clone();
                tmp_blnch.new_transaction(
                    String::from("sender"),
                    String::from("receiver"),
                    rand::thread_rng().gen_range(0..100_000_000 as u64),
                );
                tmp_blnch.mint();
                vec_blnch[rand_num] = tmp_blnch;
                println!(
                    "Block added to {}! len = {}",
                    rand_num,
                    vec_blnch[rand_num].chain.len()
                );
                time_block_create = Instant::now();
            }

            fork_duration = time_block_fork.elapsed();
            if fork_duration >= time_new_fork {
                let rand_num = rand::thread_rng().gen_range(0..vec_blnch.len());
                let mut tmp_blnch = vec_blnch[rand_num].clone();
                if tmp_blnch.chain.len() != 1 {
                    tmp_blnch.chain.pop_back();
                    tmp_blnch.new_transaction(
                        String::from("sender"),
                        String::from("receiver"),
                        rand::thread_rng().gen_range(0..100_000_000 as u64),
                    );
                    tmp_blnch.mint();
                    vec_blnch.push(tmp_blnch);
                    println!("Fork size -> {}", vec_blnch.len());
                    for index in 0..vec_blnch.len() {
                        println!("\tFork size {} -> {}", index, vec_blnch[index].chain.len());
                    }
                    time_block_fork = Instant::now();
                }
            }

            longest_chain_duration = time_block_chains.elapsed();
            if longest_chain_duration >= time_longest {
                let mut index = 0;
                let mut longest_chain = 0;
                loop {
                    if index == vec_blnch.len() {
                        break;
                    }
                    if longest_chain < vec_blnch[index].chain.len() {
                        longest_chain = vec_blnch[index].chain.len();
                    }
                    index += 1;
                }

                index = 0;
                loop {
                    if index == vec_blnch.len() {
                        break;
                    }

                    if vec_blnch[index].chain.len() < longest_chain {
                        vec_blnch.remove(index);
                    } else {
                        index += 1;
                    }
                }
                println!("Size -> {}", vec_blnch.len());
                for index in 0..vec_blnch.len() {
                    println!("\tSize {} -> {}", index, vec_blnch[index].chain.len());
                }
                time_block_chains = Instant::now();
                if vec_blnch.len() == 1 {
                    vec_blnch[0].chain.pop_front(); //??????
                    self.chain.append(&mut vec_blnch[0].chain);
                    self.tr_queue.append(&mut vec_blnch[0].tr_queue);

                    vec_blnch.clear();
                    tmp_blockchain.chain.clear();
                    tmp_blockchain.tr_queue.clear();

                    tmp_blockchain
                        .chain
                        .push_back(self.chain.back().unwrap().clone());
                    vec_blnch.push(tmp_blockchain.clone());
                    break;
                }

                println!("AMOUNT OF VERIFIED BLOCKS -> {}", self.chain.len());
            }
        }
    }

    pub fn save(&self, path: String) -> std::io::Result<()> {
        fs::write(path, self.try_to_vec().unwrap())
    }

    pub fn load(path: String) -> std::io::Result<Blockchain> {
        let mut file: fs::File = fs::File::open(path)?;
        let mut undecoded_blockchain = Vec::<u8>::new();
        file.read_to_end(&mut undecoded_blockchain).unwrap();
        Ok(Blockchain::try_from_slice(&undecoded_blockchain).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_blockchain_test() {
        let blnch: Blockchain = Blockchain::new();
        let genesis = Block {
            header: Header {
                timestamp: (String::from("0")),
                nonce: (0u128),
            },
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
        blnch.new_transaction(
            String::from("sender"),
            String::from("receiver"),
            rand::thread_rng().gen_range(0..100_000_000 as u64),
        );
        assert_eq!(blnch.tr_queue.len(), 1);
    }

    #[test]
    fn mint_test() {
        let mut blnch: Blockchain = Blockchain::new();
        blnch.tr_queue.push_back(Transaction {
            from: String::from("Sender"),
            to: String::from("Receiver"),
            amount: 21u64,
        });
        blnch.mint();
        assert_eq!(blnch.chain.len(), 2);
        assert!(
            blnch
                .chain
                .back()
                .unwrap()
                .clone()
                .hash
                .chars()
                .filter(|&c| c == '1')
                .count()
                >= 6
        );
    }

    #[test]
    fn fork_test() {
        let mut blnch: Blockchain = Blockchain::new();
        let blocks = blnch.chain.len();
        let duration_sec: u64 = 1000;
        blnch.fork(duration_sec); // will stop if the largest chain is found or time is out
        assert_ne!(blocks, blnch.chain.len());
    }

    #[test]
    fn save_test() {
        let path = String::from("Serialized_blockchain");
        let blnch: Blockchain = Blockchain::new();
        blnch.save(path.clone()).expect("Can't save file!");
        let serialize = blnch.try_to_vec().expect("Can't serialize!");
        let some_data = fs::read(path).expect("Cant open file!");
        assert_eq!(serialize, some_data);
    }

    #[test]
    fn load_test() {
        let path = String::from("Serialized_blockchain");
        let blnch: Blockchain = Blockchain::new();
        fs::write(path.clone(), blnch.try_to_vec().unwrap()).expect("Cant write a file!");
        let test_blnch: Blockchain = Blockchain::load(path).expect("Cant load file!");
        assert_eq!(blnch, test_blnch);
    }
}
