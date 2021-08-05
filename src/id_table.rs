const INDEX_BITS: u32 = 20;
const GENERATION_BITS: u32 = 32 - INDEX_BITS;

const INDEX_MASK: u32 = (1 << (INDEX_BITS + 1)) - 1;
const GENERATION_MASK: u32 = ((1 << GENERATION_BITS + 1) - 1) << INDEX_BITS;

const GENERATION_INCREMENT: u32 = 1 << (INDEX_BITS + 1);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Id(u32);

impl Id {
    fn index(self) -> u32 {
        self.0 & INDEX_MASK
    }

    fn generation(self) -> u32 {
        self.0 & GENERATION_BITS
    }
}

pub struct IdTable {}

impl IdTable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn lookup(&self, id: Id) -> Option<usize> {
        None
    }

    pub fn update(&mut self, id: Id, new_index: usize) -> bool {
        false
    }

    pub fn insert(&mut self, index: usize) -> usize {
        0
    }
}
