use super::block::{Block, BlockId};
use super::coords::Coords;
use super::worldgen::Worldgen;
use flate2::write::DeflateEncoder;
use flate2::bufread::DeflateDecoder;
use flate2::Compression;
use std::io::{BufRead, Read, Write};
use std::io;

fn chunk_coords(b: usize) -> Coords {
    Coords(
        (b % 32) as i64,
        ((b / (32 * 32)) % 32) as i64,
        ((b / 32) % 32) as i64,
    )
}

pub struct Chunk {
    blocks: [Block; 32*32*32],
    unchanged: bool,
    air: bool,
}

pub struct Iter<'a> {
    chunk: &'a Chunk,
    block: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Coords, Block);
    fn next(&mut self) -> Option<Self::Item> {
        if self.block == 32*32*32 {
            return None;
        }
        let b = self.block;
        self.block += 1;
        Some((chunk_coords(b), self.chunk.blocks[b]))
    }
}

impl Chunk {
    pub fn new(worldgen: &Worldgen, c: Coords) -> Chunk {
        let first_block = Coords(c.0 * 32, c.1 * 32, c.2 * 32);
        let blocks = worldgen.whole_chunk(first_block);
        let air = worldgen.air_chunk(first_block);
        Chunk {
            blocks: blocks,
            unchanged: true,
            air: air,
        }
    }
    pub fn load<T: BufRead>(_c: Coords, r: T) -> Chunk {
        let mut blocks = [Block::AIR; 32*32*32];
        let mut i = 0;
        let mut air = true;
        for id in DeflateDecoder::new(r).bytes().map(|b| b.unwrap()) {
            let b = Block::new(id as BlockId);
            if b.matter != Block::AIR.matter {
                air = false;
            }
            if i >= 32*32*32 {
                panic!("Invalid chunk");
            }
            blocks[i] = b;
            i += 1;
        }
        Chunk {
            blocks: blocks,
            air: air,
            unchanged: true,
        }
    }
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter { chunk: &self, block: 0 }
    }
    pub fn replace_block(&mut self, c: Coords, block: Block) {
        println!("{:?} <- {:?}", c, block);
        assert!(Coords(0, 0, 0) <= c);
        assert!(c < Coords(32, 32, 32));
        self.unchanged = false;
        if block.matter != Block::AIR.matter {
            self.air = false;
        }
        self.blocks[c.0 as usize + c.1 as usize * 32 + c.2 as usize * 32 * 32] = block;
    }
    pub fn get_block(&self, c: Coords) -> Block {
        assert!(Coords(0, 0, 0) <= c && c < Coords(32, 32, 32));
        self.blocks[c.0 as usize + c.1 as usize * 32 + c.2 as usize * 32 * 32]
    }
    pub fn is_unchanged(&self) -> bool {
        self.unchanged
    }
    pub fn is_air(&self) -> bool {
        self.air
    }
    pub fn write_to<T: Write>(&self, w: &mut T) -> io::Result<()> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        for b in self.blocks.iter().map(|b| b.matter as u8) {
            encoder.write(&[b])?;
        }
        w.write_all(&encoder.finish().unwrap())
    }
}

