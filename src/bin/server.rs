use std::{collections::HashMap, sync::{Mutex, Arc}};

use bytes::Bytes;
use mini_redis::{Connection, Command, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db: Arc<Mutex<HashMap<String, Bytes>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: Arc<Mutex<HashMap<String, Bytes>>>) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let command = Command::from_frame(frame).unwrap();
        println!("COMMAND: {:?}", command);

        let response = match command {
            Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            Command::Set(cmd) => {
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_owned(), cmd.value().clone());
                Frame::Simple(String::from("OK"))
            }
            cmd => panic!("unimplemented: {:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
