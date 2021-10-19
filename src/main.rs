mod config;
mod triggers;
mod templates;

use std::io;
use actix_web::{HttpServer, App, Responder, HttpResponse, get, web, middleware};
use log::{warn, error};
use actix_web::middleware::Logger;
use structopt::StructOpt;
use actix_files::NamedFile;
use actix_web::http::Method;
use actix_web::middleware::normalize::TrailingSlash;
use log4rs::Handle;
use rusqlite::Connection;
use std::sync::{Mutex, RwLock};
use crate::triggers::TriggerService;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let ports = config::Ports::from_args();

    let db_connection = Mutex::new(config::database::init());

    let state = web::Data::new(AppState{
        logger_handle: Mutex::new(config::logger::init_logger_handler()),
        trigger_service: RwLock::new(TriggerService::new(&db_connection)),
        db_connection,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::new(r#" Executed from %a "%r" status=%s %Dms"#))
            .wrap(middleware::NormalizePath::new(TrailingSlash::Trim))
            .service(web::scope("/TStub").service(index)
                .service(web::scope("/control")
                    .service(web::scope("/triggers").configure(triggers::controller::config))))
            .default_service(web::route().method(Method::GET))
    })
        .bind(format!("127.0.0.1:{}", ports.get_port()))?
        .run()
        .await
}

pub struct AppState{
    logger_handle: Mutex<Handle>,
    db_connection: Mutex<Connection>,
    trigger_service: RwLock<TriggerService>
}

#[get("")]
async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("ext/static/index.html")?)
}