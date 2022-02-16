use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use awc::Client;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, LinkedList, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use openssl::{rsa::Rsa, symm::Cipher};
use std::process;

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pkey: Vec<u8>,
    pub id:  Vec<u8>,
    pub addr: String,
    pub port: String,
    pub config_list: HashSet<String>,
}

impl Node {
    pub fn new(addr: String, port: String, passphrase: String) -> Node {
        let mut config = Node::load_config().expect("Can`t load config");
        config.remove(&(addr.clone() + &port));
        let rsa = Rsa::generate(2048).unwrap_or_else(|err| {
            println!("Key generation problem: {}", err);
            process::exit(1);
        });

        let private_key = rsa
        .private_key_to_pem_passphrase(Cipher::aes_128_cbc(), passphrase.as_bytes())
        .unwrap_or_else(|err| {
            println!("Public key generation problem: {}", err);
            process::exit(1);
        });

        let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap_or_else(|err| {
        println!("Private key generation problem: {}", err);
        process::exit(1);
        });

        Node {
            pkey: private_key,
            id: pub_key,
            addr,
            port,
            config_list: config,
        }
    }
    fn load_config() -> Result<HashSet<String>, serde_json::Error> {
        use std::io::Read;
        let mut file = std::fs::File::open("config.json").expect("Can`t open file!");
        let mut some_buf = String::new();
        file.read_to_string(&mut some_buf)
            .expect("Can`t read file!");
        serde_json::from_str(&some_buf)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: Header,
    pub tr_data: Transaction,
    pub hash: String,
    pub prev_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Header {
    pub timestamp: String,
    pub nonce: u128,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

    pub fn mint(&mut self) -> Block {
        //TODO: убрать 100% надобность в транзакции для создания блока
        loop {
            let mut new_block = Block {
                header: Header {
                    timestamp: (String::from("0")),
                    nonce: (0u128),
                },
                tr_data: self.tr_queue.front().unwrap().clone(),
                hash: String::from(""),
                prev_hash: self.chain.back().unwrap().hash.clone(),
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

            if self.chain.back().unwrap().hash == new_block.prev_hash
            {
                self.chain.push_back(new_block.clone());
                self.tr_queue.pop_front();
                return new_block;
            }
        }
    }
}

async fn add_block(
    blnch: web::Data<Arc<Mutex<Blockchain>>>,
    vec_blnch: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    node: web::Data<Arc<Mutex<Node>>>,
) -> Result<HttpResponse, Error> {
    let node_list = node.lock().unwrap().config_list.clone();

    if blnch.lock().expect("blnch is locked!").chain.is_empty() {
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
        blnch.lock().expect("blnch is locked!").chain.push_back(genesis.clone());
        vec_blnch.lock().expect("vec_blnch is locked!").push(blnch.lock().expect("blnch is locked!").clone());

        println!("node list, {:?}", node_list);
        for node in &node_list {
            let some_block = genesis.clone();
            let uri = String::from("http://") + node + "/get_block_from_another";
            actix_web::rt::spawn(async move{match Client::new()
                .post(uri)
                .send_json(&some_block.clone())
                .await {
                    Err(e)=> println!("Error! {:?}\n{:?}", e,async_std::task::current()),
                    Ok(_) => println!("Block sended correctly!"),
                }});
        }

        Ok(HttpResponse::Ok().json("Genesis"))
    } else {
       // println!("Minting blocks starts!");
        let duration = Duration::new(21, 0);
        let mut loop_duration;
        // let time_new_block = Duration::new(rand::thread_rng().gen_range(3..10), 0);
        //let mut time_block_create = Instant::now();

        let time_loop_stop = Instant::now();
        loop {
            loop_duration = time_loop_stop.elapsed();
            println!("Loop duration {:?}", loop_duration);
            if loop_duration >= duration {
                println!("LOOP DONE!");
                break;
            }

            println!("sleeping...");
            async_std::task::sleep(Duration::from_secs(3)).await;
            println!("WAKE UP!");

            let rng = rand::thread_rng().gen_range(0..vec_blnch.lock().expect("vec_blnch is locked!").len());
            vec_blnch.lock().expect("vec_blnch is locked!")[rng].new_transaction(
                "Sender".to_string(),
                "Reciever".to_string(),
                rand::thread_rng().gen_range(0..100_000_000 as u64),
            );

            println!("start minting!");
            let new_block = vec_blnch.lock().expect("vec_blnch is locked!")[rng].mint();
            println!("block minted: , {:?}", new_block);

            for node in &node_list {
                println!("{:?}", node);
                let some_block = new_block.clone();
                let uri = String::from("http://") + node + "/get_block_from_another";
                actix_web::rt::spawn(async move{match Client::new()
                    .post(uri)
                    .timeout(Duration::from_secs(21))
                    .send_json(&some_block.clone())
                    .await {
                        Err(e)=> println!("Error! {:?}\n{:?}", e,std::thread::current()),
                        Ok(_) => println!("Block sended correctly!"),
                    }});
            }
        }
        println!("Jopa");
        Ok(HttpResponse::Ok().json("Validation time!")) // <- send response
    }
}

async fn get_block_from_another(
    blnch: web::Data<Arc<Mutex<Blockchain>>>,
    vec_blnch: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    recieved_block: web::Json<Block>,
) -> Result<HttpResponse, Error> {
    println!("-------------------------------------------------------------------------------------------------------------------");

    let received_block = recieved_block.0;
    println!("Полученный блок: {:?}", received_block);
    // println!("Текущий вектор: {:#?}", vec_blnch);
    // println!("Текущий блокчейн: {:#?}", blnch);
    if blnch.lock().expect("blnch is locked!").chain.is_empty()
    //если нет генезиса
    {
        //TODO: заменить создание блокчейна
        blnch.lock().expect("blnch is locked!").chain.push_back(received_block);
        vec_blnch.lock().expect("vec_blnch is locked!").push(blnch.lock().expect("blnch is locked!").clone());
        println!("Genesis push_backed!");
    } else {
        let length = vec_blnch.lock().expect("vec_blnch is locked!").len();
        println!("МЫ ПОПАЛИ В ЭЛС");
        'outer: for index_chain in 0..length
        {
            println!("PREOUTER");
            let iter_block = vec_blnch.lock().expect("vec_blnch is locked!")[index_chain].chain.clone();
            println!("MEZHDU!");
            let mut iter_block = iter_block.iter().rev();
            println!("'АУТЕР");
            'inner: loop
            {                
                println!("ИННЕР");
                let current_block = iter_block.next();
                
                //println!("Куррент блокб опшн: {:#?}", current_block);
                //если текущий блокчейн закончился
                if current_block == None
                {
                    println!("текущий блокчейн закончился");
                    break 'inner;
                }
                //если нашли совпадение hash/prev_hash
                if current_block.unwrap().hash == received_block.prev_hash
                {
                    println!("нашли совпадение hash/prev_hash");
                    //если совпадение в конце цепи, то добавляем и выходим из цикла
                    if current_block.unwrap() == vec_blnch.lock().expect("vec_blnch is locked!")[index_chain].chain.back().unwrap()
                    {
                        println!("совпадение в конце цепи!");
                        vec_blnch.lock().expect("vec_blnch is locked!")[index_chain].chain.push_back(received_block);
                        break 'outer;
                    }
                    //если не в конце цепи, то делаем форк
                    else
                    {
                        println!("делаем форк!");
                        let mut chain_tmp = vec_blnch.lock().expect("vec_blnch is locked!")[index_chain].clone();
                        loop 
                        {
                            println!("зашли в последний цикл");
                            chain_tmp.chain.pop_back();

                            if current_block.unwrap() == chain_tmp.chain.back().unwrap() 
                            {
                                println!("нашли совпадение в последнем цикле!");
                                chain_tmp.chain.push_back(received_block);
                                vec_blnch.lock().expect("vec_blnch is locked!").push(chain_tmp);
                                break 'outer;
                            }
                        }
                        //TODO: Дописать случай, когда не найдено совпадений
                    }
                }
            }
        }
    }
    Ok(HttpResponse::Ok().json("Done"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let addr = "127.0.0.1".to_string();
    let port = ":8000".to_string();
    let node: Node = Node::new(addr.clone(), port.clone(), port.clone());
    let node = web::Data::new(Arc::new(Mutex::new(node)));
    let vec_blnch: Vec<Blockchain> = Vec::new();
    let vec_blnch = web::Data::new(Arc::new(Mutex::new(vec_blnch)));
    let blnch = Blockchain::new();
    let blnch = web::Data::new(Arc::new(Mutex::new(blnch)));
    blnch.lock().unwrap().chain.pop_back();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&blnch))
            .app_data(web::Data::clone(&vec_blnch))
            .app_data(web::Data::clone(&node))
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(8_192)) // <- limit size of the payload (global configuration)
            // .service(web::resource("/add_transaction").route(web::post().to(add_transaction)))
            // .service(web::resource("/get_transaction").route(web::get().to(get_transaction)))
            // .service(
            //     web::resource("/get_transaction_from_another")
            //         .route(web::get().to(get_transaction_from_another)),
            // )
            .service(web::resource("/add_block").route(web::post().to(add_block)))
            // .service(web::resource("/get_block").route(web::get().to(get_block)))
            .service(
                web::resource("/get_block_from_another")
                    .route(web::post().to(get_block_from_another)),
            )
    })
    .bind(addr + &port)?
    .run()
    .await
}
