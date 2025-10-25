use crate::helpers::pop_lsb;
use crate::Board;
use lazy_static::lazy_static;
use xorshift::{Rng, SeedableRng, Xorshift128};


lazy_static! {
    static ref ZOBRIST_NUMBERS: [u64; 781] = {
        generate_random_number()
    };
}


impl Board {
    pub fn zobrist_hash(&self) -> u64 {
        let pieces = [self.wpawn,self.wrook,self.wknight,self.wbishop,self.wqueen,self.wking,self.bpawn,self.brook,self.bknight,self.bbishop,self.bqueen,self.bking];
        let mut zobrist_hash: u64 = 0;
        for pieces_types in 0..pieces.len() {
            let mut piece_bb = pieces[pieces_types].clone();
            while piece_bb != 0 {
                let destination = pop_lsb(&mut piece_bb);

                zobrist_hash ^= ZOBRIST_NUMBERS[(64*pieces_types)+(destination as usize)];
            }
        }
        match self.white_to_move {
            true => { zobrist_hash ^= ZOBRIST_NUMBERS[768] }
            false => {}
        }
        for i in 0..4 {
            match (self.castling_rights & (1<<i)) != 0 {
                true => {zobrist_hash ^= ZOBRIST_NUMBERS[769+i] }
                false => {}
            }
        }
        if self.last_double_pawn_push != 0 {
            zobrist_hash ^= ZOBRIST_NUMBERS[(722+ self.last_double_pawn_push % 8) as usize];
        }

        return zobrist_hash;
    }
}

pub fn generate_random_number() -> [u64; 781] {
    let seed: &[_] = &[1,1];
    let mut rng: Xorshift128 = SeedableRng::from_seed(seed);
    let mut random_numbers: [u64; 781] = [0; 781];

    for random_number in random_numbers.iter_mut() {
        *random_number = rng.next_u64();
    }
    return random_numbers;
}