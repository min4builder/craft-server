pub type BlockId = u8;

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub matter: BlockId,
}

impl Block {
    pub const AIR: Block = Block { matter: 0 as BlockId };
    pub const DARK_STONE: Block = Block { matter: 1 as BlockId };
    pub const WATER: Block = Block { matter: 2 as BlockId };
    pub const LIGHT_STONE: Block = Block { matter: 3 as BlockId };
    pub fn new(id: BlockId) -> Block {
        Block { matter: id }
    }
}

