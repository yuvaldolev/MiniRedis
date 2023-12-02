use std::collections::HashMap;

use mini_redis::{Connection, Command, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let mut db: HashMap<String, Vec<u8>> = HashMap::new();
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            Command::Set(cmd) => {
                db.insert(cmd.key().to_owned(), cmd.value().to_vec());
                Frame::Simple(String::from("OK"))
            }
            cmd => panic!("unimplemented: {:?}", cmd),
        };

        connection.write_frame(&response).await.unwrap();
    }
}
