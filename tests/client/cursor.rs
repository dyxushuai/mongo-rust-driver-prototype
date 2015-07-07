use bson::{Bson, Document};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use mongodb::cursor::Cursor;
use mongodb::wire_protocol::flags::OpQueryFlags;

#[test]
fn cursor_features() {
    let client = Client::with_uri("mongodb://localhost:27017").unwrap();
    let db = client.db("test");
    let coll = db.collection("cursor_test");

    db.drop_database().ok().expect("Failed to drop database.");

    let docs = (0..10).map(|i| {
        doc! { "foo" => (i as i64) }
    }).collect();

    assert!(coll.insert_many(docs, false, None).is_ok());

    let doc = Document::new();
    let flags = OpQueryFlags::no_flags();

    let result = Cursor::query_with_batch_size(client.clone(), "test.cursor_test".to_owned(),
                                               3, flags,
                                               0, 0, doc, None, false);

    let mut cursor = match result {
        Ok(c) => c,
        Err(s) => panic!("{}", s)
    };

    let batch = cursor.next_batch().ok().expect("Failed to get next batch from cursor.");

    assert_eq!(batch.len(), 3 as usize);

    for i in 0..batch.len() {
        match batch[i].get("foo") {
            Some(&Bson::I64(j)) => assert_eq!(i as i64, j),
            _ => panic!("Wrong value returned from Cursor#next_batch")
        };
    }

    let bson = match cursor.next() {
        Some(Ok(b)) => b,
        Some(Err(_)) => panic!("Received error on 'cursor.next()'"),
        None => panic!("Nothing returned from Cursor#next")
    };

    match bson.get("foo") {
        Some(&Bson::I64(3)) => (),
        _ => panic!("Wrong value returned from Cursor#next")
    };

    assert!(cursor.has_next().ok().expect("Failed to execute 'has_next()'."));
    let vec = cursor.next_n(20).ok().expect("Failed to get next 20 results.");;

    assert_eq!(vec.len(), 6 as usize);
    assert!(!cursor.has_next().ok().expect("Failed to execute 'has_next()'."));

    for i in 0..vec.len() {
        match vec[i].get("foo") {
            Some(&Bson::I64(j)) => assert_eq!(4 + i as i64 , j),
            _ => panic!("Wrong value returned from Cursor#next_batch")
        };
    }
}
