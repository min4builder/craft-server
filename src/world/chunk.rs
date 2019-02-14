use super::block::{Block, BlockId};
use super::coords::Coords;
use std::iter::FromIterator;
use std::io::Write;
use std::io;

pub struct Chunk {
    blocks: [Block; 32*32*256],
    unchanged: bool,
}

pub struct Iter<'a> {
    chunk: &'a Chunk,
    block: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Coords, Block);
    fn next(&mut self) -> Option<Self::Item> {
        let bs = self.chunk.blocks;
        while self.block != 32*32*256 {
            let b = self.block;
            self.block += 1;
            if bs[b].matter != Block::UNCHANGED {
                let c = Coords(
                    (b % 32) as i64,
                    ((b / (32 * 32)) % 256) as i64,
                    ((b / 32) % 32) as i64,
                );
                return Some((c, bs[b]));
            }
        }
        None
    }
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk { blocks: [Block::new(Block::UNCHANGED); 32*32*256], unchanged: true }
    }
    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter { chunk: &self, block: 0 }
    }
    pub fn replace_block(&mut self, c: Coords, block: Block) {
        println!("{:?} <- {:?}", c, block);
        assert!(Coords(0, 0, 0) <= c);
        assert!(c < Coords(32, 256, 32));
        self.unchanged = false;
        self.blocks[c.0 as usize + c.2 as usize * 32 + c.1 as usize * 32 * 32] = block;
    }
    pub fn get_block(&self, c: Coords) -> Block {
        assert!(Coords(0, 0, 0) <= c && c < Coords(32, 256, 32));
        self.blocks[c.0 as usize + c.2 as usize * 32 + c.1 as usize * 32 * 32]
    }
    pub fn is_unchanged(&self) -> bool {
        self.unchanged
    }
    pub fn write_to<T: Write>(&self, w: &mut T) -> io::Result<()> {
        w.write_all(&self.blocks.into_iter().map(|b| b.matter as u8).collect::<Vec<u8>>())
    }
}

impl FromIterator<u8> for Chunk {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Chunk {
        let mut blocks = [Block::new(Block::UNCHANGED); 32*32*256];
        let mut i = 0;
	for id in iter {
		let b = Block::new(id as BlockId);
		if i >= 32*32*256 {
			panic!("Invalid chunk");
		}
		blocks[i] = b;
		i += 1;
	}
        Chunk { blocks: blocks, unchanged: true }
    }
}

