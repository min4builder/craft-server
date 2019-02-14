use byteorder::{ReadBytesExt, WriteBytesExt, NetworkEndian};
use super::block::Block;
use super::chunk::Chunk;
use super::coords::Coords;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};

pub enum Callback {
    BlockChanged(Box<FnMut(Coords, Block)>),
}

impl Callback {
    pub fn on_block_change(&mut self, c: Coords, b: Block) {
        match self {
            Callback::BlockChanged(f) => f(c, b),
            _ => {}
        }
    }
}

pub struct Map {
    callbacks: Vec<Callback>,
    chunks: HashMap<(i64, i64), Option<Chunk>>,
    daytime: usize,
}

impl Map {
    pub fn new() -> Map {
        let time = match File::open("level.lf") {
            Err(_) => 0,
            Ok(mut f) => {
                f.read_u32::<NetworkEndian>().unwrap()
            }
        };
        Map {
            callbacks: Vec::new(),
            chunks: HashMap::new(),
            daytime: time as usize,
        }
    }
    pub fn add_callback(&mut self, c: Callback) {
        self.callbacks.push(c);
    }
    pub fn get_mut_chunk(&mut self, cc: (i64, i64)) -> &mut Option<Chunk> {
        self.chunks.entry(cc).or_insert_with(|| {
            let (p, q) = cc;
            match File::open(format!("chunk.{}.{}.cf", p, q)) {
                Err(_) => None,
                Ok(f) => {
                    println!("Trying file chunk.{}.{}.cf", p, q);
                    println!("Loading chunk {:?}", cc);
                    Some(f.bytes().collect::<io::Result<Chunk>>().unwrap())
                }
            }
        })
    }
    pub fn get_chunk(&mut self, cc: (i64, i64)) -> &Option<Chunk> {
        self.get_mut_chunk(cc)
    }
    pub fn replace_block(&mut self, c: Coords, block: Block) {
        let chunk = self.get_mut_chunk(c.chunk());
        match chunk {
            None => {
                chunk.replace(Chunk::empty());
            }
            Some(_) => {}
        }
        chunk.as_mut().unwrap().replace_block(c.in_chunk(), block);
        for cb in self.callbacks.iter_mut() {
            cb.on_block_change(c, block);
        }
    }
    pub fn get_block(&mut self, c: Coords) -> Block {
        match self.get_chunk(c.chunk()) {
            None => Block::new(Block::UNCHANGED),
            Some(chunk) => chunk.get_block(c.in_chunk()),
        }
    }
    pub fn get_time(&self) -> usize {
        self.daytime
    }
    pub fn tick(&mut self, nticks: usize) {
        self.daytime = (self.daytime + nticks) % 12000;
        println!("daytime = {}", self.daytime);
    }
    pub fn save(&mut self) {
        for ((p, q), chunk) in &self.chunks {
            match chunk {
                None => {}
                Some(chunk) => {
                    if !chunk.is_unchanged() {
                        println!("Creating file chunk.{}.{}.cf", p, q);
                        let mut cf = File::create(format!("chunk.{}.{}.cf", p, q)).unwrap();
                        println!("Writing chunk ({}, {})", p, q);
                        chunk.write_to(&mut cf).unwrap();
                    }
                }
            }
        }
        {
            let mut lf = File::create("level.lf").unwrap();
            println!("Writing time");
            lf.write_u32::<NetworkEndian>(self.daytime as u32).unwrap();
        }
        println!("Saved!");
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        self.save();
    }
}

