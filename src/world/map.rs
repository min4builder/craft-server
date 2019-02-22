use byteorder::{ReadBytesExt, WriteBytesExt, NetworkEndian};
use super::block::Block;
use super::chunk::Chunk;
use super::coords::Coords;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

pub struct Map {
    chunks: HashMap<Coords, Chunk>,
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
            chunks: HashMap::new(),
            daytime: time as usize,
        }
    }
    pub fn get_mut_chunk(&mut self, cc: Coords) -> &mut Chunk {
        self.chunks.entry(cc).or_insert_with(|| {
            let Coords(p, q, r) = cc;
            match File::open(format!("chunk.{}.{}.{}.cf", p, q, r)) {
                Err(_) => Chunk::empty(cc),
                Ok(f) => {
                    println!("Trying file chunk.{}.{}.{}.cf", p, q, r);
                    println!("Loading chunk {:?}", cc);
                    Chunk::load(cc, BufReader::new(f))
                }
            }
        })
    }
    pub fn get_chunk(&mut self, cc: Coords) -> &Chunk {
        self.get_mut_chunk(cc)
    }
    pub fn replace_block(&mut self, c: Coords, block: Block) {
        let chunk = self.get_mut_chunk(c.chunk());
        chunk.replace_block(c.in_chunk(), block);
    }
    pub fn get_block(&mut self, c: Coords) -> Block {
        let chunk = self.get_chunk(c.chunk());
        chunk.get_block(c.in_chunk())
    }
    pub fn get_time(&self) -> usize {
        self.daytime
    }
    pub fn tick(&mut self, nticks: usize) {
        self.daytime = (self.daytime + nticks) % 12000;
        println!("daytime = {}", self.daytime);
    }
    pub fn save(&mut self) {
        for (Coords(p, q, r), chunk) in &self.chunks {
            if !chunk.is_unchanged() {
                println!("Creating file chunk.{}.{}.{}.cf", p, q, r);
                let mut cf = File::create(format!("chunk.{}.{}.{}.cf", p, q, r)).unwrap();
                println!("Writing chunk ({}, {}, {})", p, q, r);
                chunk.write_to(&mut cf).unwrap();
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

