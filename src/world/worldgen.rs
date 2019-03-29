use noise::{NoiseFn, Seedable, BasicMulti, OpenSimplex};
use super::block::Block;
use super::coords::Coords;
use std::cmp;

pub struct Worldgen {
    ocean: OpenSimplex,
    plains: OpenSimplex,
    mountain: BasicMulti,
}

impl Worldgen {
    const SEA_LEVEL: i64 = 800;

    pub fn new(seed: u32) -> Worldgen {
        Worldgen {
            ocean: OpenSimplex::new().set_seed(seed),
            plains: OpenSimplex::new().set_seed(seed),
            mountain: BasicMulti::new().set_seed(seed),
        }
    }

    pub fn air_chunk(&self, c: Coords) -> bool {
        for x in 0..32 {
            for z in 0..32 {
                if self.height(c.0 + x, c.2 + z) >= c.1 {
                    return false;
                }
            }
        }
        true
    }

    pub fn whole_chunk(&self, c: Coords) -> [Block; 32*32*32] {
        let mut bs = [Block::AIR; 32*32*32];
        if !self.air_chunk(c) {
            for x in 0..32 {
                for y in 0..32 {
                    for z in 0..32 {
                        bs[x + y * 32 + z * 32 * 32] = self.default_block(c + Coords(x as i64, y as i64, z as i64));
                    }
                }
            }
        }
        bs
    }

    fn bidistort(&self, x: i64, z: i64, scale: f64) -> (i64, i64) {
        let value = (scale * self.ocean.get([x as f64 / scale, z as f64 / scale])) as i64;
        (x + value, z + value)
    }
    fn world_curve(&self, x: i64, z: i64) -> f64 {
        let (x, z) = self.bidistort(x, z, 1000.);
        self.ocean.get([x as f64 / 1000., z as f64 / 1000.]) * 0.5
        + self.ocean.get([x as f64 / 800., z as f64 / 800.]) * 0.5
    }

    fn basalt(&self, x: i64, z: i64) -> i64 {
        (self.world_curve(x, z) * 200.) as i64 + Worldgen::SEA_LEVEL - 100
    }

    fn granite(&self, x: i64, z: i64) -> i64 {
        let wc = self.world_curve(x, z);
        let mount = (self.mountain.get([x as f64 / 500., z as f64 / 500.]) + 1.) * 250.;
        let plain = (self.plains.get([x as f64 / 80., z as f64 / 80.]) * 0.5
                   + self.plains.get([x as f64 / 70., z as f64 / 70.]) * 0.5
                   + wc * 10.0
                   + 0.5) * 50.0;
        if wc > 0.3 {
            Worldgen::SEA_LEVEL + mount as i64
        } else if wc > 0.1 {
            let lvl = 5. * (wc - 0.1);
            Worldgen::SEA_LEVEL + (mount * lvl + plain * (1. - lvl)) as i64
        } else if wc > -0.2 {
            Worldgen::SEA_LEVEL + plain as i64
        } else {
            0
        }
    }

    fn height(&self, x: i64, z: i64) -> i64 {
        cmp::max(Worldgen::SEA_LEVEL, cmp::max(self.basalt(x * 5, z * 5), self.granite(x * 5, z * 5))) / 5
    }

    fn default_block(&self, c: Coords) -> Block {
        if c.1 * 5 < self.basalt(c.0 * 5, c.2 * 5) {
            Block::DARK_STONE
        } else if c.1 * 5 < self.granite(c.0 * 5, c.2 * 5) {
            Block::LIGHT_STONE
        } else if c.1 * 5 < Worldgen::SEA_LEVEL {
            Block::WATER
        } else {
            Block::AIR
        }
    }
}

