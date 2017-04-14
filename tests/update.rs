#![feature(proc_macro)]

#[macro_use] extern crate crud;
extern crate rusqlite;

#[derive(Create, Read, Update, Debug, PartialEq)]
struct Thing {
    id: Option<i64>,
    value: f64,
    name: String,
}

#[derive(Create, Read, Update, Debug, PartialEq)]
struct TwoThings {
    id: Option<i64>,
    name: String,
}

fn create_table() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE thing (id INTEGER PRIMARY KEY, value REAL, name TEXT);", &[]).unwrap();
    conn.execute("CREATE TABLE two_things (id INTEGER PRIMARY KEY, name TEXT);", &[]).unwrap();
    conn
}

#[test]
fn test_create() {
    let conn = create_table();

    let obj = Thing{id: None, value: 5.2, name: "Ryan".into()};
    let obj = obj.create(&conn).unwrap();

    assert_eq!(obj.id, Some(1));
    
    let obj = Thing{id: Some(1), value: 4.2, name: "Fred".into()};
    obj.update(&conn).unwrap();
    
    assert_eq!(obj, Thing::read(&conn, 1).unwrap());
}

#[test]
fn test_read_non_existing() {
    let conn = create_table();

    let res = Thing::read(&conn, 1);
    assert!(res.is_err(), format!("{:?}", res));

    if let Err(rusqlite::Error::QueryReturnedNoRows) = res {

    } else {
        assert!(false, "Different error than expected");
    }
}

#[test]
fn test_create_two_things() {
    let conn = create_table();

    let obj = TwoThings{id: None, name: "Ryan".into()};
    let obj = obj.create(&conn).unwrap();

    assert_eq!(obj.id, Some(1));
}

