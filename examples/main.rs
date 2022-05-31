use anyhow::Result;
use rcon_rs::{Client, PacketType};
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), ()> {
    println!("Hello, world!");
    let _ = task::spawn_blocking(move || {
        // create new connect using ip and port
        let mut conn = Client::new("server", "port").unwrap();
        // you MUST auth the connection before attempting to use it
        conn.auth("password").unwrap();
        // send any command you would like, the packet type is option and inferred to be a command by
        // default
        println!("{}", conn.send("list", Some(PacketType::Cmd)).unwrap());
        println!("done");
    }).await.unwrap();
    Ok(())
}

