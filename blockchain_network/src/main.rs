mod blockchain;
use actix_web::{get, post, web, Responder};
use blockchain::Blockchain;
use std::sync::Mutex;

//#[get("/get_transaction_by_index/{transaction_id}")]
async fn get_transaction_by_index(
    blnch: web::Data<Mutex<Blockchain>>,
    web::Path(transaction_id): web::Path<usize>,
) -> impl Responder {
    let blockchain = blnch.lock().unwrap();
    format!(
        "id: {}, transaction: {:#?}!",
        transaction_id,
        blockchain.tr_queue.iter().nth(transaction_id)
    )
}

#[get("/get_block_by_index/{block_id}")]
async fn get_block_by_index(
    blnch: web::Data<Mutex<Blockchain>>,
    web::Path(block_id): web::Path<usize>,
) -> impl Responder {
    let blockchain = blnch.lock().unwrap();
    format!(
        "id: {}, block: {:#?}!",
        block_id,
        blockchain.chain.iter().nth(block_id)
    )
}

async fn get_head_block(blnch: web::Data<Mutex<Blockchain>>) -> impl Responder {
    let blockchain = blnch.lock().unwrap();
    format!(
        "id: {}, block: {:#?}!",
        blockchain.chain.len() - 1,
        blockchain.chain.back()
    )
}

//#[post("/add_transaction/{from}/{to}/{amount}")]
async fn add_transaction(
    blnch: web::Data<Mutex<Blockchain>>,
    web::Path((from, to, amount)): web::Path<(String, String, u64)>,
) -> impl Responder {
    let mut blockchain = blnch.lock().unwrap();

    blockchain.new_transaction(from, to, amount);

    format!("added transaction: {:#?}!", blockchain.tr_queue.back())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    let blockchain = Blockchain::new();
    let blnch = web::Data::new(Mutex::new(blockchain));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&blnch))
           // .service(get_transaction_by_index)
            .service(get_block_by_index)
          //  .service(get_head_block)
           // .service(add_transaction)
           // .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(web::resource("/ws/get_head_block").route(web::get().to(get_head_block)))
            .service(web::resource("/ws/get_transaction_by_index/{index}").route(web::get().to(get_transaction_by_index)))
    })
    .bind("127.0.0.1:8001")?
    .run()
    .await
}
