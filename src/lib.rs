use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::TcpStream;
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

#[derive(Debug)]
pub enum RconError {
    DecodeError,
    AuthError,
    SendError,
}

impl std::fmt::Display for RconError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
    pub fn new(host: &str, port: &str) -> Result<Client, std::io::Error> {
        let conn =
            TcpStream::connect(format!("{}:{}", host, port))?;

        Ok(Client {
            conn,
            next: AtomicI32::new(0),
            auth: AtomicBool::new(false),
        })
    }

    /// Authenticate the client by sending a password packet and reading the response
    pub fn auth(&mut self, password: &str) -> Result<(), crate::RconError> {
        self.send(password, Some(PacketType::Auth))?;
        self.auth = AtomicBool::new(true);
        Ok(())
    }

    /// increment the id stored by the client struct and return it's value
    fn next_id(&mut self) -> i32 {
        let new = self.next.load(Relaxed) + 1;
        self.next.store(new, Relaxed);
        new
    }

    /// send a message over the tcp stream
    pub fn send(&mut self, cmd: &str, msg_type: Option<PacketType>) -> Result<String, crate::RconError> {
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

        let _ = match self.conn.write(&message.encode()) {
            Ok(v) => v,
            Err(_) => return Err(RconError::SendError),
        };
        let mut response = [0u8; MAX_PACKET];
        let _ = match self.conn.read(&mut response) {
            Ok(v) => v,
            Err(_) => return Err(RconError::SendError),
        };
        Ok(match Packet::decode(response.to_vec()) {
            Ok(v) => v,
            Err(_) => return Err(RconError::SendError),
        }.body)
    }
}

impl Packet {
    
    /// encode packet struct into a byte vector
    pub fn encode(&self) -> Vec<u8> {
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

        data.extend_from_slice(p.body.as_bytes());
        data.extend_from_slice(&[0, 0]);

        data
    }

    /// decode byte vector into packet struct
    pub fn decode(data: Vec<u8>) -> Result<Packet, crate::RconError> {
        let len = i32::from_le_bytes(match data[0..4].try_into() {
            Ok(v) => v,
            Err(_) => return Err(RconError::DecodeError),
        });
        let id = i32::from_le_bytes(match data[0..4].try_into() {
            Ok(v) => v,
            Err(_) => return Err(RconError::DecodeError),
        });
        let packet_type = i32::from_le_bytes(match data[8..12].try_into() {
            Ok(v) => v,
            Err(_) => return Err(RconError::DecodeError),
        });

        let mut body = "".to_string();
        let body_len: usize = match (len - 10).try_into() {
            Ok(v) => v,
            Err(_) => return Err(RconError::DecodeError),
        };

        if body_len > 0 {
            body = match from_utf8(&data[12..12 + body_len]) {
                Ok(v) => v,
                Err(_) => return Err(RconError::DecodeError),
            }.to_string();
        }

        Ok(Packet {
            len,
            id,
            packet_type,
            body,
        })
    }
}
