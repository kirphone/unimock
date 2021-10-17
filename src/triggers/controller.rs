use actix_web::{Responder, HttpResponse, web, get, post};
use crate::triggers::Trigger;
use crate::AppState;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_triggers).service(add_trigger);
}

#[get("")]
pub async fn get_triggers(data: web::Data<AppState>) -> impl Responder{
    HttpResponse::Ok().json(&data.trigger_service.lock().unwrap().triggers)
}

#[post("/add")]
pub async fn add_trigger(trigger: web::Json<Trigger>) -> actix_web::Result<String>{
    Ok(format!("{}", trigger.description))
}