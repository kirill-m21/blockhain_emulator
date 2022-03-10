use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use awc::Client;
use chrono::Utc;
use openssl::{rsa::Rsa, symm::Cipher};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, LinkedList, VecDeque};
use std::process;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pkey: Vec<u8>,
    pub id: Vec<u8>,
    pub addr: String,
    pub config_list: HashSet<String>,
}

impl Node {
    pub fn new(addr: String, passphrase: String) -> Node {
        let config = Node::load_config().expect("Can`t load config");
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
            config_list: config,
        }
    }
    fn load_config() -> Result<HashSet<String>, serde_json::Error> {
        use std::io::Read;
        let mut file = std::fs::File::open("config.json").expect("Can't open a file");
        let mut some_buf = String::new();
        file.read_to_string(&mut some_buf)
            .expect("Can't read a file");
        serde_json::from_str(&some_buf)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: Header,
    pub tr_data: VecDeque<Transaction>,
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

impl Transaction {
    pub fn into(&self) -> String {
        let mut sum_string: String = "".to_string();
        sum_string.push_str(&self.from);
        sum_string.push_str(&self.to);
        sum_string.push_str(&self.amount.to_string());
        sum_string
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Blockchain {
    pub chain: LinkedList<Block>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let block_chain_tmp: LinkedList<Block> = LinkedList::new();
        Blockchain {
            chain: block_chain_tmp,
        }
    }

    pub fn mint(&self, mempool: VecDeque<Transaction>) -> Block {
        loop {
            let mut new_block = Block {
                header: Header {
                    timestamp: "".to_string(),
                    nonce: (0u128),
                },
                tr_data: mempool,
                hash: "".to_string(),
                prev_hash: self.chain.back().unwrap().hash.clone(),
            };
            let mut sum_string: String = "".to_string();

            sum_string.push_str(&new_block.prev_hash);
            for iter in &new_block.tr_data {
                sum_string.push_str(&iter.into());
            }
            loop {
                new_block.header.nonce = rand::thread_rng().gen::<u128>();
                sum_string.push_str(&new_block.header.nonce.to_string());
                let mut hasher = Sha256::new();
                hasher.update(&sum_string);
                let check_hash = format!("{:x}", hasher.finalize());
                if check_hash.chars().filter(|&c| c == '1').count() >= 6 {
                    new_block.hash = check_hash;
                    new_block.header.timestamp = Utc::now().timestamp().to_string();
                    return new_block;
                }
            }
        }
    }
}

async fn add_transaction(
    node: web::Data<Arc<Mutex<Node>>>,
    tr_mempool: web::Data<Arc<Mutex<VecDeque<Transaction>>>>,
) -> Result<HttpResponse, Error> {
    let transaction = Transaction {
        from: "".to_string(),
        to: "".to_string(),
        amount: rand::thread_rng().gen::<u64>(),
    };

    let node_list = node.lock().expect("Node is locked").config_list.clone();

    for node in &node_list {
        let some_transaction = transaction.clone();
        let uri = String::from("http://") + node + "/v1/private/transactions/receive";
        actix_web::rt::spawn(async move {
            match Client::new()
                .post(uri)
                .send_json(&some_transaction.clone())
                .await
            {
                Err(e) => println!("Error! {:?}\n{:?}", e, async_std::task::current()),
                Ok(_) => (),
            }
        });
    }

    Ok(HttpResponse::Ok().json(
        tr_mempool
            .lock()
            .expect("Transaction mempool is locked")
            .clone(),
    ))
}

async fn recieve_transaction(
    tr_mempool: web::Data<Arc<Mutex<VecDeque<Transaction>>>>,
    recieved_transaction: web::Json<Transaction>,
) -> Result<HttpResponse, Error> {
    let recieved_transaction = recieved_transaction.0;

    tr_mempool
        .lock()
        .expect("Transaction mempool is locked")
        .push_back(recieved_transaction);
    Ok(HttpResponse::Ok().json("Transaction added"))
}

async fn remove_transaction(
    tr_mempool: web::Data<Arc<Mutex<VecDeque<Transaction>>>>,
) -> Result<HttpResponse, Error> {
    tr_mempool
        .lock()
        .expect("Transaction mempool is locked")
        .clear();
    Ok(HttpResponse::Ok().json("Mempool is clear"))
}

async fn gen_genesis(
    blnch: web::Data<Arc<Mutex<Blockchain>>>,
    node: web::Data<Arc<Mutex<Node>>>,
    tr_mempool: web::Data<Arc<Mutex<VecDeque<Transaction>>>>,
) -> Result<HttpResponse, Error> {
    let node_list = node.lock().expect("Node is locked").config_list.clone();
    if blnch
        .lock()
        .expect("Main blockchain is locked")
        .chain
        .is_empty()
    {
        let mempool_to_mint = tr_mempool
            .lock()
            .expect("Transaction mempool is locked")
            .clone();

        for node in &node_list {
            let uri = String::from("http://") + node + "/v1/private/transactions/remove";
            actix_web::rt::spawn(async move {
                match Client::new().post(uri).send().await {
                    Err(e) => println!("Error! {:?}\n{:?}", e, async_std::task::current()),
                    Ok(_) => (),
                }
            });
        }

        let genesis = Block {
            header: Header {
                timestamp: (String::from("")),
                nonce: (0u128),
            },
            tr_data: mempool_to_mint.clone(),
            hash: Utc::now().timestamp().to_string(),
            prev_hash: String::from(""),
        };

        for node in &node_list {
            let some_block = genesis.clone();
            let uri = String::from("http://") + node + "/v1/private/blocks/receive";
            actix_web::rt::spawn(async move {
                match Client::new().post(uri).send_json(&some_block.clone()).await {
                    Err(e) => println!("Error! {:?}\n{:?}", e, async_std::task::current()),
                    Ok(_) => (),
                }
            });
        }
        Ok(HttpResponse::Ok().json("Genesis minted"))
    } else {
        Ok(HttpResponse::Ok().json("There is already Genesis"))
    }
}

async fn add_block(
    blnch: web::Data<Arc<Mutex<Blockchain>>>,
    vec_blnch: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    node: web::Data<Arc<Mutex<Node>>>,
    time: web::Data<Arc<Mutex<Instant>>>,
    tr_mempool: web::Data<Arc<Mutex<VecDeque<Transaction>>>>,
) -> Result<HttpResponse, Error> {
    if blnch
        .lock()
        .expect("Main blockchain is locked")
        .chain
        .is_empty()
    {
        Ok(HttpResponse::Ok().json("There is no Genesis"))
    } else {
        let node_list = node.lock().expect("Node is locked").config_list.clone();
        loop {
            if time.lock().expect("Time ERROR!").elapsed().as_secs() >= 100 {
                break;
            }

            async_std::task::sleep(Duration::from_secs(
                rand::thread_rng().gen_range(1..5 as u64),
            ))
            .await;

            let rng = rand::thread_rng()
                .gen_range(0..vec_blnch.lock().expect("Main vector is locked").len());

            let mempool_to_mint = tr_mempool
                .lock()
                .expect("Transaction mempool is locked")
                .clone();

            for node in &node_list {
                let uri = String::from("http://") + node + "/v1/private/transactions/remove";
                actix_web::rt::spawn(async move {
                    match Client::new().post(uri).send().await {
                        Err(e) => println!("Error! {:?}\n{:?}", e, async_std::task::current()),
                        Ok(_) => (),
                    }
                });
            }

            let new_block =
                vec_blnch.lock().expect("Main vector is locked")[rng].mint(mempool_to_mint.clone());

            for node in &node_list {
                println!("{:?}", node);
                let some_block = new_block.clone();
                let uri = String::from("http://") + node + "/v1/private/blocks/receive";
                actix_web::rt::spawn(async move {
                    match Client::new().post(uri).send_json(&some_block.clone()).await {
                        Err(e) => println!("Error! {:?}\n{:?}", e, std::thread::current()),
                        Ok(_) => (),
                    }
                });
            }
        }
        Ok(HttpResponse::Ok().json(blnch.lock().expect("Main blockchain is locked").clone()))
    }
}

async fn receive_block(
    blnch: web::Data<Arc<Mutex<Blockchain>>>,
    vec_blnch: web::Data<Arc<Mutex<Vec<Blockchain>>>>,
    time: web::Data<Arc<Mutex<Instant>>>,
    recieved_block: web::Json<Block>,
) -> Result<HttpResponse, Error> {
    let received_block = recieved_block.0;

    if blnch
        .lock()
        .expect("Main blockchain is locked")
        .chain
        .is_empty()
    //if no genesis
    {
        blnch
            .lock()
            .expect("Main blockchain is locked")
            .chain
            .push_back(received_block);
        vec_blnch
            .lock()
            .expect("Main vector is locked")
            .push(blnch.lock().expect("Main blockchain is locked").clone());
    } else {
        let length = vec_blnch.lock().expect("Main vector is locked").len();
        'outer: for index_chain in 0..length {
            let iter_block = vec_blnch.lock().expect("Main vector is locked")[index_chain]
                .chain
                .clone();
            let mut iter_block = iter_block.iter().rev();
            'inner: loop {
                let current_block = iter_block.next();
                //if current blockchain has ended
                if current_block == None {
                    break 'inner;
                }
                //if found a match of hash/prev_hash
                if current_block.unwrap().hash == received_block.prev_hash {
                    //if found a match at the end of the chain
                    if current_block.unwrap()
                        == vec_blnch.lock().expect("Main vector is locked")[index_chain]
                            .chain
                            .back()
                            .unwrap()
                    {
                        vec_blnch.lock().expect("Main vector is locked")[index_chain]
                            .chain
                            .push_back(received_block);
                        break 'outer;
                    }
                    //if not found a match at the end of the chain => fork
                    else {
                        let mut chain_tmp =
                            vec_blnch.lock().expect("Main vector is locked")[index_chain].clone();
                        loop {
                            chain_tmp.chain.pop_back();

                            if current_block.unwrap() == chain_tmp.chain.back().unwrap() {
                                chain_tmp.chain.push_back(received_block);
                                vec_blnch
                                    .lock()
                                    .expect("Main vector is locked")
                                    .push(chain_tmp);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
    }
    //Consensus
    if time.lock().expect("Time error").elapsed().as_secs() % 20 == 0 {
        let mut vec_tmp = vec_blnch
            .lock()
            .expect("Temporary vector is locked")
            .clone();
        vec_tmp.sort_by_key(|k| k.chain.len());
        if vec_tmp.len() > 1 {
            if vec_tmp[vec_tmp.len() - 1].chain.len() > vec_tmp[vec_tmp.len() - 2].chain.len() + 1 {
                *blnch.lock().expect("Main blockchain is locked") =
                    vec_tmp[vec_tmp.len() - 1].clone();
                println!("Blockchain validated");
            } else {
                println!("Similar (+-1) length of chains");
            }
        } else {
            *blnch.lock().expect("Main blockchain is locked") = vec_tmp[vec_tmp.len() - 1].clone();
        }
    }
    Ok(HttpResponse::Ok().json("Done"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    use std::io::Read;
    let mut file = std::fs::File::open("config.json").expect("Can't open file!");
    let mut some_buf = String::new();
    file.read_to_string(&mut some_buf)
        .expect("Can't read file!");
    let nodes = serde_json::from_str::<HashSet<String>>(&some_buf)?;

    println!("{:#?}", nodes);

    let time = Instant::now();
    let time = web::Data::new(Arc::new(Mutex::new(time)));
    let vec_blnch: Vec<Blockchain> = Vec::new();
    let vec_blnch = web::Data::new(Arc::new(Mutex::new(vec_blnch)));
    let blnch = Blockchain::new();
    let blnch = web::Data::new(Arc::new(Mutex::new(blnch)));
    let tr_mempool: VecDeque<Transaction> = VecDeque::new();
    let tr_mempool = web::Data::new(Arc::new(Mutex::new(tr_mempool)));

    let mut addr_config = "".to_string();
    for iter in &nodes {
        match std::net::TcpListener::bind(iter) {
            Ok(_) => {
                addr_config = iter.clone();
                break;
            }
            Err(_) => (),
        };
    }

    let addr_config = match addr_config.as_str() {
        "" => panic!("Node can't be started! Please, check config.json"),
        _ => addr_config,
    };

    let node: Node = Node::new(addr_config.clone(), addr_config.clone());
    let node = web::Data::new(Arc::new(Mutex::new(node)));

    println!("Node started at: http://{:?}", addr_config);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&blnch))
            .app_data(web::Data::clone(&vec_blnch))
            .app_data(web::Data::clone(&node))
            .app_data(web::Data::clone(&time))
            .app_data(web::Data::clone(&tr_mempool))
            // enable logger
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4_096)) // <- limit size of the payload (global configuration)
            .service(
                web::resource("/v1/public/transactions/add").route(web::post().to(add_transaction)),
            )
            .service(
                web::resource("/v1/private/transactions/receive")
                    .route(web::post().to(recieve_transaction)),
            )
            .service(
                web::resource("/v1/private/transactions/remove")
                    .route(web::post().to(remove_transaction)),
            )
            .service(web::resource("/v1/private/blocks/genesis").route(web::post().to(gen_genesis)))
            .service(web::resource("/v1/public/blocks/mint").route(web::post().to(add_block)))
            .service(
                web::resource("/v1/private/blocks/receive").route(web::post().to(receive_block)),
            )
    })
    .bind(addr_config)?
    .run()
    .await
}
