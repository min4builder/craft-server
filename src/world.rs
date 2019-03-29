mod block;
mod chunk;
mod coords;
mod map;
mod worldgen;
use byteorder::{WriteBytesExt, NetworkEndian};
use block::{Block, BlockId};
use coords::Coords;
use map::Map;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, RwLock, RwLockWriteGuard};

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

fn write_raw_msg<T: Write>(w: &mut T, b: &[u8]) -> Result<(), io::Error> {
    w.write_u32::<NetworkEndian>(b.len() as u32)?;
    w.write_all(b)
}
fn write_msg<T: Write>(w: &mut T, s: &str) -> Result<(), io::Error> {
    write_raw_msg(w, s.as_bytes())
}

impl<T: Write> Server<T> {
    pub fn new() -> Server<T> {
        Server {
            players: HashMap::new(),
            map: Map::new(),
        }
    }
    fn send_except<F: Fn(&mut T) -> Result<(), io::Error>>(&mut self, ex: usize, write: F) -> Result<(), io::Error> {
        for (id, (_, w)) in self.players.iter_mut() {
            if *id != ex {
                let mut writer = w.write().unwrap();
                write(&mut writer)?;
            }
        }
        Ok(())
    }
    fn client_writer(&mut self, ex: usize) -> RwLockWriteGuard<T> {
        self.players.get_mut(&ex).unwrap().1.write().unwrap()
    }
    pub fn connect(&mut self, client: Arc<RwLock<T>>, id: usize) -> Result<(), io::Error> {
        let player = Player {
            nick: format!("person{}", id),
            x: 0.0, y: 0.0, z: 0.0,
            rx: 0.0, ry: 0.0,
        };
        let mut msgs = Vec::new();
        write_msg(&mut msgs, &format!("N,{},{}", id, player.nick))?;
        write_msg(&mut msgs, &format!("T,{} has joined", player.nick))?;
        write_msg(&mut msgs, &format!("P,{},0.0,0.0,0.0,0.0,0.0", id))?;
        self.players.insert(id, (player, client));
        self.send_except(id, |w| w.write_all(&msgs))?;
        msgs.clear();
        write_msg(&mut msgs, &format!("U,{},0.0,0.0,0.0,0.0,0.0", id))?;
        write_msg(&mut msgs, &format!("E,{},600", self.map.get_time() / 20))?;
        for (i, (p, _)) in self.players.iter() {
            if *i != id {
                write_msg(&mut msgs, &format!("P,{},{},{},{},{},{}", i, p.x, p.y, p.z, p.rx, p.ry))?;
                write_msg(&mut msgs, &format!("N,{},{}", *i, p.nick))?;
            }
        }
        self.client_writer(id).write_all(&msgs)
    }
    pub fn disconnect(&mut self, id: usize) -> Result<(), io::Error> {
        let mut msgs = Vec::new();
        write_msg(&mut msgs, &format!("D,{}", id))?;
        write_msg(&mut msgs, &format!("T,{} has left", self.players[&id].0.nick))?;
        self.players.remove(&id);
        self.map.save();
        self.send_except(id, |w| w.write_all(&msgs))
    }
    pub fn tick(&mut self, secs: f32) {
        self.map.tick((secs * 20.0).round() as usize);
        if secs > 30.0 {
            self.map.save();
        }
    }
    pub fn command(&mut self, id: usize, cmd: &str) -> Result<(), io::Error> {
        let fields: Vec<&str> = cmd.split_whitespace().collect();
        match (fields[0], fields.len()) {
            ("/nick", 2) => {
                let mut msgs = Vec::new();
                write_msg(&mut msgs, &format!("T,{} is now {}", &self.players[&id].0.nick, fields[1]))?;
                write_msg(&mut msgs, &format!("N,{},{}", id, fields[1]))?;
                self.players.get_mut(&id).unwrap().0.nick = fields[1].to_string();
                self.send_except(id, |w| w.write_all(&msgs))?;
                self.client_writer(id).write_all(&msgs)
            }
            ("/nick", 1) => {
                let msg = format!("T,You are {}", &self.players[&id].0.nick);
                write_msg::<T>(&mut self.client_writer(id), &msg)
            }
            (cmd, args) => {
                write_msg::<T>(&mut self.client_writer(id), &format!("T,Unknown command: {}[{}]", cmd, args-1))
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
                let r: i64 = fields[3].parse().unwrap();
                let _key: i64 = fields[3].parse().unwrap();
                println!("{}: {}", id, smsg);
                let chunk = self.map.get_chunk(Coords(p, q, r));
                if chunk.is_air() {
                    println!("Chunk empty");
                    return Ok(());
                }
                println!("Sending chunk");
                let mut blocks = Vec::with_capacity(1+8+8 + 32*32*32);
                blocks.write_u8('C' as u8).unwrap();
                blocks.write_i64::<NetworkEndian>(p).unwrap();
                blocks.write_i64::<NetworkEndian>(q).unwrap();
                blocks.write_i64::<NetworkEndian>(r).unwrap();
                chunk.write_to(&mut blocks).unwrap();
                write_raw_msg::<T>(&mut self.client_writer(id), &blocks)
            }
            "P" => {
                let (player, _) = self.players.get_mut(&id).unwrap();
                player.x = fields[1].parse().unwrap();
                player.y = fields[2].parse().unwrap();
                player.z = fields[3].parse().unwrap();
                player.rx = fields[4].parse().unwrap();
                player.ry = fields[5].parse().unwrap();
                let mut msg = Vec::new();
                write_msg(&mut msg, &format!("P,{},{},{},{},{},{}\n", id, player.x, player.y, player.z, player.rx, player.ry))?;
                self.send_except(id, |w| w.write_all(&msg))
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
                let smsg = format!("B,{},{},{},{}\n", x, y, z, w);
                println!("{}", &smsg);
                let mut msg = Vec::new();
                write_msg(&mut msg, &smsg)?;
                self.send_except(id, |w| w.write_all(&msg))?;
                self.client_writer(id).write_all(&msg)
            }
            "T" => {
                let (_, chat) = smsg.split_at(2);
                if chat.starts_with("/") {
                    self.command(id, chat)
                } else {
                    let mut msg = Vec::new();
                    println!("Chat: [{}] {}", self.players[&id].0.nick, chat);
                    write_msg(&mut msg, &format!("T,[{}] {}", self.players[&id].0.nick, chat))?;
                    self.send_except(id, |w| w.write_all(&msg))?;
                    self.client_writer(id).write_all(&msg)
                }
            }
            m => panic!("{} message not implemented", m),
        }
    }
}

