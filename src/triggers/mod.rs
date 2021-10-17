pub mod controller;

use serde::{Serialize, Deserialize, Deserializer, Serializer};
use std::collections::{HashMap, HashSet, BTreeMap};
use rusqlite::{Connection, params};
use std::sync::{Mutex, RwLock};
use serde::de::{Error, Visitor};
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use log::{debug};

const GET_TRIGGERS_QUERY: &str = "SELECT * FROM Triggers";
const INSERT_TRIGGER_QUERY: &str = "INSERT INTO Triggers (type, expression, headers, comment, active) VALUES (?1,?2,?3,?4,?5)";
const REMOVE_TRIGGER_QUERY: &str = "DELETE FROM Triggers WHERE id=?1";

#[derive(Serialize, Deserialize, Debug)]
pub struct Trigger {
    #[serde(skip_deserializing)]
    id: u32,
    msg_type: String,
    expression: String,
    description: String,
    is_active: bool,
    #[serde(deserialize_with = "deserialize_headers", serialize_with = "serialize_headers")]
    headers: BTreeMap<String, String>,
}

pub struct TriggerService {
    triggers: HashMap<u32, Trigger>,
}

impl TriggerService {
    pub fn new(db_connection: &Mutex<Connection>) -> TriggerService {
        TriggerService {
            triggers: TriggerService::read_from_db(&*db_connection.lock().unwrap())
        }
    }

    pub fn refresh(&mut self, db_connection: &Mutex<Connection>) {
        self.triggers = TriggerService::read_from_db(&*db_connection.lock().unwrap());
    }

    fn read_from_db(connection: &Connection) -> HashMap<u32, Trigger> {
        connection.prepare(GET_TRIGGERS_QUERY).unwrap()
            .query_map([], |row| {
                Ok((row.get(0).unwrap(), Trigger {
                    id: row.get(0).unwrap(),
                    msg_type: row.get(1).unwrap(),
                    expression: row.get(2).unwrap(),
                    description: row.get(3).unwrap(),
                    is_active: row.get(4).unwrap(),
                    headers: build_headers(row.get(5).unwrap()),
                }))
            }).unwrap().map(|item| { item.unwrap() }).collect()
    }

    fn add_trigger(service: &RwLock<TriggerService>, connection: &Mutex<Connection>, mut trigger: Trigger) -> rusqlite::Result<usize> {
        let locked_connection = connection.lock().unwrap();
        let result = locked_connection.execute(INSERT_TRIGGER_QUERY, params![
            &trigger.msg_type, &trigger.expression, unbuild_headers(&trigger.headers), &trigger.description, &trigger.is_active
        ]);

        if result.is_ok() {
            trigger.id = locked_connection.last_insert_rowid() as u32;
            std::mem::drop(locked_connection);
            service.write().unwrap().triggers.insert(trigger.id, trigger);
        }
        result
    }

    fn remove_trigger(service: &RwLock<TriggerService>, connection: &Mutex<Connection>, id: u32) -> rusqlite::Result<usize> {
        let result = connection.lock().unwrap().execute(REMOVE_TRIGGER_QUERY, params![id]);

        if result.is_ok() {
            service.write().unwrap().triggers.remove(&id);
        }
        result
    }
}

fn build_headers(headers: String) -> BTreeMap<String, String> {
    headers.split_terminator(&",").map(|item| {
        let header: Vec<&str> = item.split("=").collect();
        assert_eq!(header.len(), 2);
        (header.get(0).unwrap().to_string(), header.get(1).unwrap().to_string())
    }).collect()
}

fn unbuild_headers(headers: &BTreeMap<String, String>) -> String {
    debug!("Cериализуем хедеры: {:?}", headers);
    let result = headers.iter().fold(String::new(), |acc, (key, value)| format!("{}{}={},", acc, key, value));
    debug!("Получаем: {}", result);
    result
}

fn deserialize_headers<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
    where
        D: Deserializer<'de>,
{
    let input: String = Deserialize::deserialize(deserializer)?;
    debug!("Десериализуем хедеры из строки: {}", input);
    let res_map = build_headers(input);
    debug!("Получаем хедеры: {:?}", res_map);
    Ok(res_map)
}

fn serialize_headers<S>(headers: &BTreeMap<String, String>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(&unbuild_headers(headers))
}