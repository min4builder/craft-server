use std::ops::Add;
use std::cmp::{PartialOrd, Ordering};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Coords(pub i64, pub i64, pub i64);

impl Coords {
    pub fn chunk(&self) -> (i64, i64) {
        /* will be .div_euc() when it gets out of nightly */
        (if self.0 % 32 < 0 { self.0 / 32 - 1 } else { self.0 / 32 },
         if self.2 % 32 < 0 { self.2 / 32 - 1 } else { self.2 / 32 })
    }
    pub fn in_chunk(&self) -> Coords {
        /* will be .mod_euc() when it gets out of nightly */
        Coords((self.0 % 32 + 32) % 32, self.1, (self.2 % 32 + 32) % 32)
    }
}

impl Add for Coords {
    type Output = Coords;
    fn add(self, other: Coords) -> Coords {
        Coords(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl PartialOrd for Coords {
    fn partial_cmp(&self, other: &Coords) -> Option<Ordering> {
        if self.0 < other.0 && self.1 < other.1 && self.2 < other.2 {
            Some(Ordering::Less)
        } else if self.0 > other.0 && self.1 > other.1 && self.2 > other.2 {
            Some(Ordering::Greater)
        } else if self == other {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
    fn le(&self, other: &Coords) -> bool {
        self.0 <= other.0 && self.1 <= other.1 && self.2 <= other.2
    }
    fn ge(&self, other: &Coords) -> bool {
        self.0 >= other.0 && self.1 >= other.1 && self.2 >= other.2
    }
}

