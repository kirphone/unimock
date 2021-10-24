use actix_web::{Responder, HttpResponse, web, get, post, error};
use crate::templates::{Template, TemplateService};
use crate::AppState;
use log::{error, debug, info};
use std::error::Error;
use std::ops::Deref;
use serde::{Deserialize};
use serde_json::Value;
use futures::StreamExt;

const TEMPLATE_MAX_SIZE: usize = 50 * 1024 * 1024;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_templates).service(add_template);
}

#[get("")]
pub async fn get_templates(data: web::Data<AppState>) -> impl Responder {
    info!("Получен запрос на чтение всех шаблонов");
    HttpResponse::Ok().json(&data.template_service.read().unwrap().templates)
}

#[post("/add")]
pub async fn add_template(mut template: web::Payload, data: web::Data<AppState>) -> Result<String, actix_web::Error> {
    let mut template_in_bytes = web::BytesMut::new();
    while let Some(chunk) = template.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (template_in_bytes.len() + chunk.len()) > TEMPLATE_MAX_SIZE {
            return Err(error::ErrorBadRequest("Слишком большой размер шаблона"));
        }
        template_in_bytes.extend_from_slice(&chunk);
    }
    let template_value: Value = serde_json::from_slice(&template_in_bytes)?;

    let template = Template::new(0, template_value["template"].as_str().ok_or(error::ErrorBadRequest("Ключ template не найден"))?,
                                 template_value["comment"].as_str().ok_or(error::ErrorBadRequest("Ключ comment не найден"))?.to_string(),
                                 template_value["type"].as_str().ok_or(error::ErrorBadRequest("Ключ type не найден"))?)?;

    info!("Получен запрос на добавление шаблона: {:?}", template);
    match TemplateService::add_template(&data.template_service, &data.db_connection, template) {
        Ok(template_id) => { Ok(format!("{{\"id\": {}}}", template_id)) }
        Err(ex) => {
            error!("Ошибка при SQL запросе на добавление шаблона\n{}", ex);
            Err(error::ErrorInternalServerError(ex))
        }
    }
}