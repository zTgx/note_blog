#![allow(unused)]
extern crate postgres;
use self::postgres::{Connection, TlsMode};

pub struct Item {
    pub id: i32,
    pub name: String,
    pub data: Option<String>,
}

pub struct SQLConn {
    conn: Connection,
    pool: Vec<Item>,
}

impl SQLConn {
    pub fn new() -> Self {
        SQLConn {
            conn: Connection::connect("postgres://postgres:123456@127.0.0.1:5432", TlsMode::None).unwrap(),
            pool: Vec::new(),//vec![Item{id: 0, name: "".to_string(), data: None}; 0],
        }
    }

    pub fn exec() {
        let con: Connection =  Connection::connect("postgres://postgres:123456@127.0.0.1:5432", TlsMode::None).unwrap();
        con.execute("CREATE TABLE if not exists person (id SERIAL PRIMARY KEY,
                                                         name VARCHAR NOT NULL,
                                                         data VARCHAR)", &[]).unwrap();
    }

    pub fn insert(name: String, message: Option<String>) {
        let me = Item   {id: 0,
                         name: name,
                         data: message,
                        };

        let con: Connection =  Connection::connect("postgres://postgres:123456@127.0.0.1:5432", TlsMode::None).unwrap();
        con.execute("INSERT INTO person (name, data) VALUES ($1, $2)",  &[&me.name, &me.data.unwrap_or_else(|| "default".to_string() )]).unwrap();
    }

    pub fn query(p: &mut Vec<Item>) -> &mut Vec<Item> {
        let con: Connection =  Connection::connect("postgres://postgres:123456@127.0.0.1:5432", TlsMode::None).unwrap();
        for row in &con.query("SELECT id, name, data FROM person", &[]).unwrap() {
            let person = Item   {id: row.get(0),
                                 name: row.get(1),
                                 data: row.get(2),
                                };

            //println!("Found person {}: {} says : {}", person.id, person.name, person.data.unwrap_or_else(|| "".to_string() ));

            p.push(person);       
        }

        p
    }
}
