pub mod controller;

use serde::{Serialize, Deserialize, Deserializer, Serializer};
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};
use rusqlite::{Connection, params, Row, RowIndex};
use serde_json::Value;
use std::str::FromStr;
use log::{debug};
use serde::ser::{Error, SerializeStruct};

const GET_TEMPLATES_QUERY: &str = "SELECT * FROM Templates";
const INSERT_TEMPLATE_QUERY: &str = "INSERT INTO Templates (template, comment, type) VALUES(?1,?2,?3)";
const REMOVE_TEMPLATE_QUERY: &str = "DELETE FROM Triggers WHERE id=?1";
const UPDATE_TEMPLATE_QUERY: &str = "UPDATE Triggers SET type=?1, expression=?2, headers=?3, comment=?4, active=?5 WHERE id=?6";

#[derive(Debug)]
pub struct Template {
    id: u32,
    name: String,
    vars: HashMap<String, Variables>,
    body: TemplateBody
}

impl Template {
    pub fn new(id: u32, text: &str, name: String, template_type: &str) -> Result<Template, serde_json::Error> {
        Ok(Template {
            id,
            name,
            vars: HashMap::new(),
            body: match template_type {
                "xml" => TemplateBody::XMLTemplateBody,
                "json" => TemplateBody::JsonTemplateBody { body: Value::from_str(text)? },
                "regex" => TemplateBody::RegexTemplateBody,
                _ => { return Err(serde_json::Error::custom("Неизвестный тип шаблона")); }
            },
        })
    }
}

#[derive(Serialize, Debug)]
enum TemplateBody {
    XMLTemplateBody,
    JsonTemplateBody { body: Value },
    RegexTemplateBody,
}

impl TemplateBody {
    fn get_text_and_type(&self) -> (String, &str) {
        match self {
            TemplateBody::JsonTemplateBody { body } => {
                (body.to_string(), "json")
            }
            TemplateBody::XMLTemplateBody => {
                ("".to_string(), "xml")
            }
            TemplateBody::RegexTemplateBody => {
                ("".to_string(), "regex")
            }
        }
    }
}

#[derive(Debug)]
enum Variables {
    ResponseHeader { key: String, value: String },
    FromHeader { key: String, value: String },
}

pub struct TemplateService {
    templates: HashMap<u32, Template>,
}

impl TemplateService {
    pub fn new(db_connection: &Mutex<Connection>) -> TemplateService {
        TemplateService {
            templates: TemplateService::read_from_db(&*db_connection.lock().unwrap())
        }
    }

    fn read_from_db(connection: &Connection) -> HashMap<u32, Template> {
        connection.prepare(GET_TEMPLATES_QUERY).unwrap()
            .query_map([], |row| {
                Ok((row.get(0).unwrap(), Template::new(row.get(0).unwrap(),
                                                       &row.get::<usize, String>(1).unwrap(),
                                                       row.get(2).unwrap(),
                                                       &row.get::<usize, String>(3).unwrap()).unwrap()))
            }).unwrap().map(|item| { item.unwrap() }).collect()
    }

    fn add_template(service: &RwLock<TemplateService>, connection: &Mutex<Connection>, mut template: Template) -> rusqlite::Result<u32> {
        let locked_connection = connection.lock().unwrap();
        let (text, template_type) = template.body.get_text_and_type();
        locked_connection.execute(INSERT_TEMPLATE_QUERY, params![
            text, &template.name, template_type
        ])?;
        template.id = locked_connection.last_insert_rowid() as u32;
        let res = template.id;
        std::mem::drop(locked_connection);
        service.write().unwrap().templates.insert(template.id, template);
        Ok(res)
    }
}

impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        debug!("Сериализация шаблона: {:?}", self);
        let mut struct_serializer = serializer.serialize_struct("Template", 4)?;
        struct_serializer.serialize_field("id", &self.id)?;
        struct_serializer.serialize_field("comment", &self.name)?;
        let (text, template_type) = self.body.get_text_and_type();
        struct_serializer.serialize_field("template", &text)?;
        struct_serializer.serialize_field("type", &template_type)?;
        struct_serializer.end()
    }
}