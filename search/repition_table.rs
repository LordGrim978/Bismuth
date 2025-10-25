
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RepetitionTable {
    hashes: [u64; 256],
    count: usize,
}
impl RepetitionTable {
    pub const fn new() -> Self {
        RepetitionTable { hashes: [0; 256], count: 0 }
    }
    pub fn add(&mut self, hash: u64) {
        if self.count < self.hashes.len() {
            self.hashes[self.count] = hash;
            self.count += 1;
        }
    }
    pub fn pop_last(&mut self) {
        self.count -= 1;
        self.hashes[self.count] = 0;
    }
    pub fn contains(&self, hash: u64) -> bool {
        let mut rep_count = 0;
        for i in 0..self.count {
            if self.hashes[i] == hash {
                rep_count += 1;
            }
            if rep_count >= 3 {
                return true;
            }
        }
        return false;
    }
}