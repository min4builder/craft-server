use noise::{NoiseFn, Perlin};
use super::block::{Block, BlockId};
use super::coords::Coords;

pub fn whole_chunk(c: Coords) -> [Block; 32*32*32] {
    let mut bs = [Block::new(Block::UNCHANGED); 32*32*32];
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                bs[x + z * 32 + y * 32 * 32] = default_block(c + Coords(x as i64, y as i64, z as i64));
            }
        }
    }
    bs
}

pub fn default_block(c: Coords) -> Block {
    let perlin = Perlin::new();
    if (c.1 - 80) as f64 / 32.0 > perlin.get([c.0 as f64 / 32.0, c.2 as f64 / 32.0]) {
        Block::new(Block::AIR)
    } else {
        Block::new(6 as BlockId)
    }
}

