mod config;

use std::io;
use actix_web::{HttpServer, App, Responder, HttpResponse, get, web};
use log::{warn, error};
use actix_web::middleware::Logger;
use futures::lock::Mutex;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let logger_handle = web::Data::new(Mutex::new(config::logger::init_logger_handler()));

    HttpServer::new(move || {
        App::new()
            .app_data(logger_handle.clone())
            .service(hello)
            .wrap(Logger::default())
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world")
}