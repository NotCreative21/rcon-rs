### rcon-rs 
<a href="./LICENSE"><img src="https://img.shields.io/badge/license-GPL%20v3.0-blue.svg"></a>

Hyper barebones implementation of the RCON protocol for Minecraft.

To use this crate place this in your Cargo.toml file

```toml
[dependencies]
rcon-rs = "0.1.0" 

```

##### Example usage:
```rust
// create new connect using ip and port
let mut conn = Client::new("127.0.0.1", "25575");
// you MUST auth the connection before attempting to use it
conn.auth("password")?;
// send any command you would like, the packet type is option and inferred to be a command by
// default
conn.send("say hi", Some(PacketType::Cmd))?;
```

A more complete example can be found [here](./examples/main.rs)

##### rcon protocol details
More info can be found here:

https://wiki.vg/RCON#Fragmentation

##### contributing
Any suggestions, improvements, or pull request are welcome. I am relatively new to rust so my quality of code is not great but I'm looking to improve!
