pub type BlockId = u8;

#[derive(Copy, Clone, Debug)]
pub struct Block {
    pub matter: BlockId,
}

impl Block {
    pub const UNCHANGED: BlockId = !0 as BlockId;
    pub const AIR: BlockId = 0 as BlockId;
    pub fn new(id: BlockId) -> Block {
        Block { matter: id }
    }
}

