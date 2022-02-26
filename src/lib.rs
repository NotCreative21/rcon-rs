use std::convert::TryInto;
use tokio::net::TcpStream;
use std::str::from_utf8;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

const HEADER_LEN: usize = 10;
const MAX_PACKET: usize = 4110;

#[repr(i32)]
pub enum PacketType {
    Response,
    _None,
    Cmd,
    Auth,
}

/// Packet struct
///
/// Used by the client to send and recieve data from the rcon connection
///
/// [`encode`]: #method.encode
/// [`decode`]: #method.decode

#[derive(Debug)]
pub struct Packet {
    /// length of packet data
    pub len: i32,
    /// packet id, when minecraft recieves a packet it response with a packet using the same id
    pub id: i32,
    /// packet type, Response, Cmd, Auth
    pub packet_type: i32,
    /// packet contents, can be a command for example
    pub body: String,
}

/// Client struct
///
/// Created to communicate with the minecraft server
///
/// [`new`]: #method.encode
/// [`auth`]: #method.auth
/// [`next_id`]: #method.next_id
/// [`send`]: #method.send 

pub struct Client {
    /// raw tcp stream to handle communication
    conn: TcpStream,
    /// the next corresponding packet id, starting from 0
    next: AtomicI32,
    /// store if client is authenticated or not
    auth: AtomicBool,
}

impl Client {
    /// Create a new client, the host and port is taken in then connected to via tcp
    pub async fn new(host: &str, port: &str) -> Client {
        let conn =
            TcpStream::connect(format!("{}:{}", host, port)).await.unwrap();

        Client {
            conn,
            next: AtomicI32::new(0),
            auth: AtomicBool::new(false),
        }
    }

    /// Authenticate the client by sending a password packet and reading the response
    pub async fn auth(&mut self, password: &str) -> Result<(), ()> {
        self.send(password, Some(PacketType::Auth)).await?;
        self.auth = AtomicBool::new(true);
        let mut response = [0u8; MAX_PACKET];
        self.conn.try_read(&mut response).unwrap();
        let _ = Packet::decode(response.to_vec());
        Ok(())
    }

    /// increment the id stored by the client struct and return it's value
    fn next_id(&mut self) -> i32 {
        let new = self.next.load(Relaxed) + 1;
        self.next.store(new, Relaxed);
        new
    }

    /// send a message over the tcp stream
    pub async fn send(&mut self, cmd: &str, msg_type: Option<PacketType>) -> Result<(), ()> {
        let msg_type = match msg_type {
            Some(v) => v,
            None => PacketType::Cmd,
        };
        let message = Packet {
            len: (cmd.len() + HEADER_LEN) as i32,
            id: self.next_id(),
            packet_type: msg_type as i32,
            body: cmd.to_string(),
        };

        self.conn.try_write(&message.encode().await).unwrap();
        Ok(())
    }
}

impl Packet {
    
    /// encode packet struct into a byte vector
    pub async fn encode(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        let p = self;

        let extends = vec![
            p.len.to_le_bytes(),
            p.id.to_le_bytes(),
            p.packet_type.to_le_bytes(),
        ];

        for i in extends {
            data.extend_from_slice(&i);
        }

        data.extend_from_slice(&p.body.as_bytes());
        data.extend_from_slice(&[0, 0]);

        data
    }

    /// decode byte vector into packet struct
    pub async fn decode(data: Vec<u8>) -> Packet {
        let len = i32::from_le_bytes(data[0..4].try_into().unwrap());
        let id = i32::from_le_bytes(data[0..4].try_into().unwrap());
        let packet_type = i32::from_le_bytes(data[8..12].try_into().unwrap());

        let mut body = "".to_string();
        let body_len: usize = (len - 10).try_into().unwrap();

        if body_len > 0 {
            body = from_utf8(&data[12..12 + body_len]).unwrap().to_string();
        }

        Packet {
            len,
            id,
            packet_type,
            body,
        }
    }
}
