mod block;
mod chunk;
mod coords;
mod map;
use byteorder::{WriteBytesExt, NetworkEndian};
use block::{Block, BlockId};
use coords::Coords;
use map::Map;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};

struct Player {
    nick: String,
    x: f64,
    y: f64,
    z: f64,
    rx: f64,
    ry: f64,
}

pub struct Server<T: Write> {
    players: HashMap<usize, (Player, Arc<RwLock<T>>)>,
    map: Map,
}

fn add_msg(v: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    v.reserve(4 + b.len());
    v.write_u32::<NetworkEndian>(b.len() as u32).unwrap();
    v.extend_from_slice(b);
}

impl<T: Write> Server<T> {
    pub fn new() -> Server<T> {
        Server {
            players: HashMap::new(),
            map: Map::new(),
        }
    }
    fn send_except(&mut self, ex: usize, msg: &[u8]) -> Result<(), io::Error> {
        for (id, (_, w)) in self.players.iter_mut() {
            if *id != ex {
                let mut writer = w.write().unwrap();
                writer.write_all(msg)?;
            }
        }
        Ok(())
    }
    fn send(&mut self, ex: usize, msg: &[u8]) -> Result<(), io::Error> {
        let mut writer = self.players.get_mut(&ex).unwrap().1.write().unwrap();
        writer.write_all(msg)?;
        Ok(())
    }
    pub fn connect(&mut self, client: Arc<RwLock<T>>, id: usize) -> Result<(), io::Error> {
        let player = Player {
            nick: format!("person{}", id),
            x: 0.0, y: 0.0, z: 0.0,
            rx: 0.0, ry: 0.0,
        };
        let mut msgs = Vec::new();
        add_msg(&mut msgs, &format!("N,{},{}", id, player.nick));
        add_msg(&mut msgs, &format!("T,{} has joined", player.nick));
        add_msg(&mut msgs, &format!("P,{},0.0,0.0,0.0,0.0,0.0", id));
        self.players.insert(id, (player, client));
        self.send_except(id, &msgs)?;
        msgs.clear();
        add_msg(&mut msgs, &format!("U,{},0.0,0.0,0.0,0.0,0.0", id));
        add_msg(&mut msgs, &format!("E,{},600", self.map.get_time() / 20));
        for (i, (p, _)) in self.players.iter() {
            if *i != id {
                add_msg(&mut msgs, &format!("P,{},{},{},{},{},{}", i, p.x, p.y, p.z, p.rx, p.ry));
                add_msg(&mut msgs, &format!("N,{},{}", *i, p.nick));
            }
        }
        self.send(id, &msgs)
    }
    pub fn disconnect(&mut self, id: usize) -> Result<(), io::Error> {
        let mut msgs = Vec::new();
        add_msg(&mut msgs, &format!("D,{}", id));
        add_msg(&mut msgs, &format!("T,{} has left", self.players[&id].0.nick));
        self.players.remove(&id);
        self.map.save();
        self.send_except(id, &msgs)
    }
    pub fn tick(&mut self, secs: f32) {
        self.map.tick((secs * 20.0).round() as usize);
    }
    pub fn command(&mut self, id: usize, cmd: &str) -> Result<(), io::Error> {
        let fields: Vec<&str> = cmd.split_whitespace().collect();
        match (fields[0], fields.len()) {
            ("/nick", 2) => {
                let mut msgs = Vec::new();
                add_msg(&mut msgs, &format!("T,{} is now {}", &self.players[&id].0.nick, fields[1]));
                add_msg(&mut msgs, &format!("N,{},{}", id, fields[1]));
                self.players.get_mut(&id).unwrap().0.nick = fields[1].to_string();
                self.send_except(id, &msgs)?;
                self.send(id, &msgs)
            }
            ("/nick", 1) => {
                let mut msg = Vec::new();
                add_msg(&mut msg, &format!("T,You are {}", &self.players[&id].0.nick));
                self.send(id, &msg)
            }
            (cmd, args) => {
                let mut msg = Vec::new();
                add_msg(&mut msg, &format!("T,Unknown command: {}[{}]", cmd, args-1));
                self.send(id, &msg)
            }
        }
    }
    pub fn process_message(&mut self, id: usize, msg: &[u8]) -> Result<(), io::Error> {
        let smsg = String::from_utf8_lossy(msg);
        let fields: Vec<&str> = smsg.split(',').collect();
        match fields[0] {
            "V" => {
                println!("{}: {}", id, smsg);
                if fields[1] != "2" {
                    Err(io::Error::new(io::ErrorKind::Other, "Incompatible version"))
                } else {
                    Ok(())
                }
            }
            "C" => {
                let p: i64 = fields[1].parse().unwrap();
                let q: i64 = fields[2].parse().unwrap();
                let _key: i64 = fields[3].parse().unwrap();
                println!("{}: {}", id, smsg);
                
                match self.map.get_chunk((p, q)) {
                    None => Ok(()),
                    Some(chunk) => {
                        let mut msgs = Vec::new();
                        for (c, b) in chunk.iter() {
                            add_msg(&mut msgs, &format!("B,{},{},{},{},{},{}\n", p, q, p * 32 + c.0, c.1, q * 32 + c.2, b.matter as u8));
                        }
                        self.send(id, &msgs)
                    }
                }
            }
            "P" => {
                let (player, _) = self.players.get_mut(&id).unwrap();
                player.x = fields[1].parse().unwrap();
                player.y = fields[2].parse().unwrap();
                player.z = fields[3].parse().unwrap();
                player.rx = fields[4].parse().unwrap();
                player.ry = fields[5].parse().unwrap();
                let mut msg = Vec::new();
                add_msg(&mut msg, &format!("P,{},{},{},{},{},{}\n", id, player.x, player.y, player.z, player.rx, player.ry));
                self.send_except(id, &msg)
            }
            "B" => {
                let x: i64 = fields[1].parse().unwrap();
                let y: i64 = fields[2].parse().unwrap();
                let z: i64 = fields[3].parse().unwrap();
                let w: u8 = fields[4].parse().unwrap();
                let c = Coords(x, y, z);
                println!("{}: {}", id, smsg);
                self.map.replace_block(c, Block::new(w as BlockId));
                println!("{:?} of chunk {:?} is now {}", (x, y, z), Coords(x, y, z).chunk(), self.map.get_block(c).matter as u8);
                let (p, q) = c.chunk();
                let smsg = format!("B,{},{},{},{},{},{}\n", p, q, x, y, z, w);
                println!("{}", &smsg);
                let mut msg = Vec::new();
                add_msg(&mut msg, &smsg);
                self.send_except(id, &msg)?;
                self.send(id, &msg)
            }
            "T" => {
                let (_, chat) = smsg.split_at(2);
                if chat.starts_with("/") {
                    self.command(id, chat)
                } else {
                    let mut msg = Vec::new();
                    println!("Chat: [{}] {}", self.players[&id].0.nick, chat);
                    add_msg(&mut msg, &format!("T,[{}] {}", self.players[&id].0.nick, chat));
                    self.send_except(id, &msg)?;
                    self.send(id, &msg)
                }
            }
            m => panic!("{} message not implemented", m),
        }
    }
}

