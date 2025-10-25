use crate::move_gen_dir::move_gen::Square;


pub fn pop_lsb(number: &mut u64) -> u64{
    let lsb = number.trailing_zeros();
    *number ^= 1 << number.trailing_zeros();
    return lsb as u64
}

pub const fn sq_to_index(square: Square) -> usize{
    return match square{
        Square::A1 => 0, Square::B1 => 1,Square::C1 => 2, Square::D1 => 3,Square::E1 => 4, Square::F1 => 5,Square::G1 => 6, Square::H1 => 7,
        Square::A2 => 8, Square::B2 => 9,Square::C2 => 10, Square::D2 => 11, Square::E2 => 12, Square::F2 => 13,Square::G2 => 14, Square::H2 => 15,
        Square::A3 => 16, Square::B3 => 17,Square::C3 => 18, Square::D3 => 19, Square::E3 => 20, Square::F3 => 21,Square::G3 => 22, Square::H3 => 23,
        Square::A4 => 24, Square::B4 => 25,Square::C4 => 26, Square::D4 => 27,Square::E4 => 28, Square::F4 => 29,Square::G4 => 30, Square::H4 => 31,
        Square::A5 => 32, Square::B5 => 33,Square::C5 => 34, Square::D5 => 35, Square::E5 => 36, Square::F5 => 37,Square::G5 => 38, Square::H5 => 39,
        Square::A6 => 40, Square::B6 => 41,Square::C6 => 42, Square::D6 => 43, Square::E6 => 44, Square::F6 => 45,Square::G6 => 46, Square::H6 => 47,
        Square::A7 => 48, Square::B7 => 49,Square::C7 => 50, Square::D7 => 51, Square::E7 => 52, Square::F7 => 53,Square::G7 => 54, Square::H7 => 55,
        Square::A8 => 56, Square::B8 => 57,Square::C8 => 58, Square::D8 => 59, Square::E8 => 60, Square::F8 => 61,Square::G8 => 62, Square::H8 => 63,
    } as usize;
}

pub const fn index_to_sq(index: usize) -> Square {
    match index {
        0 => Square::A1, 1 => Square::B1, 2 => Square::C1, 3 => Square::D1,
        4 => Square::E1, 5 => Square::F1, 6 => Square::G1, 7 => Square::H1,
        8 => Square::A2, 9 => Square::B2, 10 => Square::C2, 11 => Square::D2,
        12 => Square::E2, 13 => Square::F2, 14 => Square::G2, 15 => Square::H2,
        16 => Square::A3, 17 => Square::B3, 18 => Square::C3, 19 => Square::D3,
        20 => Square::E3, 21 => Square::F3, 22 => Square::G3, 23 => Square::H3,
        24 => Square::A4, 25 => Square::B4, 26 => Square::C4, 27 => Square::D4,
        28 => Square::E4, 29 => Square::F4, 30 => Square::G4, 31 => Square::H4,
        32 => Square::A5, 33 => Square::B5, 34 => Square::C5, 35 => Square::D5,
        36 => Square::E5, 37 => Square::F5, 38 => Square::G5, 39 => Square::H5,
        40 => Square::A6, 41 => Square::B6, 42 => Square::C6, 43 => Square::D6,
        44 => Square::E6, 45 => Square::F6, 46 => Square::G6, 47 => Square::H6,
        48 => Square::A7, 49 => Square::B7, 50 => Square::C7, 51 => Square::D7,
        52 => Square::E7, 53 => Square::F7, 54 => Square::G7, 55 => Square::H7,
        56 => Square::A8, 57 => Square::B8, 58 => Square::C8, 59 => Square::D8,
        60 => Square::E8, 61 => Square::F8, 62 => Square::G8, 63 => Square::H8,
        _ => Square::A1
    }
}

pub fn _print_lookup_array_as_const(name: &str, array: &[[u64; 64]; 64]) {
    println!("pub const {}: [[u64; 64]; 64] = [", name);

    for (i, row) in array.iter().enumerate() {
        print!("    [");
        for (j, val) in row.iter().enumerate() {
            print!("{:#018x}", val); // padded hex format
            if j != 63 {
                print!(", ");
            }
        }
        println!("], // {}", i);
    }

    println!("];");
}

pub fn _gen_white_pawn_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    for sq in 0..64 {
        let mut mask = 0;
        if sq % 8 != 0 { mask |= 1u64 << (sq + 7); } // capture left
        if sq % 8 != 7 { mask |= 1u64 << (sq + 9); } // capture right
        attacks[sq] = mask & !0xff00000000000000; // avoid overflow
    }
    attacks
}

pub fn _gen_black_pawn_attacks() -> [u64; 64] {
    let mut attacks = [0u64; 64];
    for sq in 0..64 {
        let mut mask = 0;
        if sq % 8 != 0 { mask |= 1u64 << (sq - 9); } // capture left
        if sq % 8 != 7 { mask |= 1u64 << (sq - 7); } // capture right
        attacks[sq] = mask & !0xff; // avoid underflow
    }
    attacks
}