use rusqlite::Connection;
use std::fs;

const DB_NAME: &str = "ext/database/Unimock.db";
const APPLICATION_VERSION: u8 = 0;
const SELECT_VERSION_QUERY: &str = "SELECT version FROM TStubVersions ORDER BY version desc;";

pub fn init() -> Connection {
    let connection = Connection::open(DB_NAME).unwrap();
    update(&connection);
    connection
}

fn update(connection: &Connection) {
    let version = connection.prepare(SELECT_VERSION_QUERY).unwrap()
        .query_map([], |row| Ok(row.get(0).unwrap())).unwrap().next().unwrap().unwrap();
    for i in version..APPLICATION_VERSION {
        let query = fs::read_to_string(format!("{}.sql", i)).unwrap();
        connection.execute(&query,[]);
    }
}