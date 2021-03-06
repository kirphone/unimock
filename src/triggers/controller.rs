use actix_web::{Responder, HttpResponse, web, get, post, error};
use crate::triggers::{Trigger, TriggerService};
use crate::AppState;
use log::{error, debug, info};
use std::error::Error;
use std::ops::Deref;
use serde::{Deserialize};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_triggers).service(add_trigger).service(remove_trigger).service(update_trigger);
}

#[get("")]
pub async fn get_triggers(data: web::Data<AppState>) -> impl Responder{
    info!("Получен запрос на чтение всех триггеров");
    HttpResponse::Ok().json(&data.trigger_service.read().unwrap().triggers)
}

#[post("/add")]
pub async fn add_trigger(trigger: web::Json<Trigger>, data: web::Data<AppState>) -> Result<String, actix_web::Error>{
    let trigger = trigger.into_inner();
    info!("Получен запрос на добавление триггера: {:?}", trigger);
    match TriggerService::add_trigger(&data.trigger_service, &data.db_connection, trigger) {
        Ok(trigger_id) => {Ok(format!("{{\"id\": {}}}", trigger_id))},
        Err(ex) => {
            error!("Ошибка при SQL запросе на добавление триггера\n{}", ex);
            Err(error::ErrorInternalServerError(ex))
        }
    }
}

#[post("/remove")]
pub async fn remove_trigger(trigger_id: web::Json<RemoveTriggerReq>, data: web::Data<AppState>) -> Result<&'static str, actix_web::Error>{
    info!("Получен запрос на удаление триггера с id={}", trigger_id.id);
    match TriggerService::remove_trigger(&data.trigger_service, &data.db_connection, trigger_id.id) {
        Ok(_) => {Ok("Ok")}
        Err(ex) => {
            error!("Ошибка при SQL запросе на удаление триггера\n{}", ex);
            Err(error::ErrorInternalServerError(ex))
        }
    }
}

#[post("/update")]
pub async fn update_trigger(trigger: web::Json<Trigger>, data: web::Data<AppState>) -> Result<&'static str, actix_web::Error>{
    let trigger = trigger.into_inner();
    info!("Получен запрос на обновление триггера: {:?}", trigger);
    match TriggerService::update_trigger(&data.trigger_service, &data.db_connection, trigger) {
        Ok(_) => {Ok("Ok")}
        Err(ex) => {
            error!("Ошибка при запросе на обновление триггера\n{}", ex);
            Err(error::ErrorInternalServerError(ex))
        }
    }
}

#[derive(Deserialize)]
pub struct RemoveTriggerReq{
    id: u32
}