mod blockchain;
use actix_web::{web, Error, HttpResponse};
use crate::blockchain::block::transaction::Transaction;
use blockchain::Blockchain;
use std::sync::Mutex;
use futures::StreamExt;

async fn get_transaction_by_index(web::Path(index): web::Path<usize>, blnch: web::Data<Mutex<Blockchain>>) -> Result<HttpResponse, Error>{
    Ok(HttpResponse::Ok().json(blnch.lock().unwrap().tr_queue.iter().nth(index)))
}

async fn get_block_by_index(web::Path(index): web::Path<usize>, blnch: web::Data<Mutex<Blockchain>>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(blnch.lock().unwrap().chain.iter().nth(index)))
}

async fn get_head_block(blnch: web::Data<Mutex<Blockchain>>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(blnch.lock().unwrap().chain.back()))
}

async fn add_transaction(mut payload: web::Payload, blnch: web::Data<Mutex<Blockchain>>) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        body.extend_from_slice(&chunk);
    }

    let obj = serde_json::from_slice::<Transaction>(&body)?;
    blnch.lock().unwrap().tr_queue.push_back(obj);

    Ok(HttpResponse::Ok().json(blnch.lock().unwrap().tr_queue.back()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    let blockchain = Blockchain::new();
    let blnch = web::Data::new(Mutex::new(blockchain));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&blnch))
            .service(web::resource("/v1/public/transactions/new").route(web::post().to(add_transaction)))
            .service(web::resource("/v1/public/transactions/:{index}").route(web::get().to(get_transaction_by_index)))
            .service(web::resource("/v1/public/blocks/:{index}").route(web::get().to(get_block_by_index)))
            .service(web::resource("/v1/public/blocks/head").route(web::get().to(get_head_block)))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
