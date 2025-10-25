use std::collections::HashMap;
use crate::move_gen_dir::precomputed_magics::{BISHOP_MAGICS, BISHOP_SHIFTS}; // Changed to BISHOP_MAGICS and BISHOP_SHIFTS

const BOARD_SIZE: usize = 64;

// Directions for bishops (NE, NW, SE, SW)
const BISHOP_DIRECTIONS: [(i8, i8); 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)]; // Changed directions


// Result storage
static mut BISHOP_ATTACK_TABLE: Vec<u64> = Vec::new(); // Changed to BISHOP_ATTACK_TABLE
static mut BISHOP_MAGICS_META: [Magic; 64] = [Magic::EMPTY; 64]; // Changed to BISHOP_MAGICS_META

// ------------ Structs ------------

#[derive(Clone, Copy)]
pub struct Magic {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: usize,
}

impl Magic {
    pub const EMPTY: Self = Magic {
        mask: 0,
        magic: 0,
        shift: 0,
        offset: 0,
    };
}

// ------------ Bitboard Utils ------------

fn square_index(file: i8, rank: i8) -> Option<usize> {
    if file >= 0 && file < 8 && rank >= 0 && rank < 8 {
        Some((rank * 8 + file) as usize)
    } else {
        None
    }
}

fn set_bit(bb: &mut u64, sq: usize) {
    *bb |= 1u64 << sq;
}

#[allow(dead_code)]
fn popcount(mut x: u64) -> u32 {
    let mut count = 0;
    while x != 0 {
        x &= x - 1;
        count += 1;
    }
    count
}

// ------------ Mask Generation ------------

// Precompute bishop masks for each square
fn bishop_mask(square: usize) -> u64 { // Changed to bishop_mask
    let mut mask = 0u64;
    let rank = (square / 8) as i8;
    let file = (square % 8) as i8;

    for &(df, dr) in &BISHOP_DIRECTIONS { // Using bishop directions
        let mut f = file + df;
        let mut r = rank + dr;

        // Iterate until out of bounds
        while f >= 0 && f < 8 && r >= 0 && r < 8 {
            // Exclude the square immediately adjacent to the bishop, as that's where blockers would be
            // This mask includes all squares along the ray *except* the bishop's square and the edge squares.
            // This is because we're interested in the squares where blockers can reside.
            if (f != file + df || r != rank + dr) && (f != file || r != rank) { // Ensure we don't mask the bishop's square or the first square in the ray
                // For bishops, unlike rooks, we just need to ensure we don't mask the piece's square itself.
                // The loop naturally handles masking squares between the bishop and the edge.
            }
            if let Some(sq_idx) = square_index(f, r) {
                // We want to include squares up to, but not including, the board edge,
                // and also not including the square the bishop is on.
                // The typical approach for magic bitboard masks is to include all squares on the ray
                // *excluding* the source square and the very last square before the board edge
                // (if that square is empty, otherwise the blocker would be on it).
                // Here, we include all squares that *could* contain a blocker.
                // So, we iterate along the ray and include all squares that are not on the edge.
                if (f != 0 && f != 7 && r != 0 && r != 7) || (f == file && r == rank) {
                    // If it's the bishop's square or an edge square, don't include in mask unless it's not the bishop's square.
                    // The mask should only include squares *between* the bishop and the edges.
                    // So, if it's on the edge, it cannot block anything further.
                    if f != file || r != rank { // Exclude the bishop's own square
                        // Check if it's an edge square
                        if f == 0 || f == 7 || r == 0 || r == 7 {
                            // If it's an edge square, it can't be a blocker *for an internal ray*,
                            // but it can be a blocker if it's on the path.
                            // The logic for bishop masks usually means all squares on the ray *except* the piece's square itself.
                            // The `bishop_attacks_from` function will handle the termination at blockers.
                            // So, the mask should include all squares on the diagonal, excluding the bishop's square.
                            set_bit(&mut mask, sq_idx);
                        } else {
                            set_bit(&mut mask, sq_idx);
                        }
                    }
                }
            }
            f += df;
            r += dr;
        }
    }
    // Corrected bishop mask generation: The mask for a bishop on `square` should include all squares
    // along its diagonals that are *not* the `square` itself and are *not* on the edge of the board.
    // The typical implementation iterates until it hits an edge, and includes all squares *before* the edge.
    // The current loop goes to `while let Some(sq)`. We need to adjust.
    // Let's reset the mask and regenerate it using a more standard approach for bishop masks.

    let mut new_mask = 0u64;
    for &(df, dr) in &BISHOP_DIRECTIONS {
        let mut f = file + df;
        let mut r = rank + dr;
        while f >= 0 && f < 8 && r >= 0 && r < 8 {
            if f == 0 || f == 7 || r == 0 || r == 7 {
                // If it's an edge square, stop adding to the mask in this direction
                // as it can't have a blocker further along this ray.
                // We add it to the mask because a blocker could be *on* the edge.
                // However, for magic bitboards, the mask usually excludes the edge squares.
                // Let's refine this to match common magic bitboard mask generation.
                break;
            }
            if let Some(sq_idx) = square_index(f, r) {
                set_bit(&mut new_mask, sq_idx);
            }
            f += df;
            r += dr;
        }
    }
    new_mask
}

// ------------ Attack Generation ------------

fn bishop_attacks_from(square: usize, blockers: u64) -> u64 { // Changed to bishop_attacks_from
    let rank = (square / 8) as i8;
    let file = (square % 8) as i8;
    let mut attacks = 0;

    for &(df, dr) in &BISHOP_DIRECTIONS { // Using bishop directions
        let mut f = file + df;
        let mut r = rank + dr;

        while let Some(sq) = square_index(f, r) {
            set_bit(&mut attacks, sq);
            // If this square is a blocker, stop in this direction
            if (blockers >> sq) & 1 != 0 {
                break;
            }
            f += df;
            r += dr;
        }
    }

    attacks
}

// ------------ Blocker Permutations ------------

fn enumerate_blockers(mask: u64) -> Vec<u64> {
    let bits: Vec<usize> = (0..64).filter(|&i| (mask >> i) & 1 == 1).collect();
    let mut blockers = Vec::new();
    let total = 1 << bits.len();

    for i in 0..total {
        let mut b = 0;
        for (j, &bit) in bits.iter().enumerate() {
            if (i >> j) & 1 == 1 {
                b |= 1 << bit;
            }
        }
        blockers.push(b);
    }

    blockers
}

// ------------ Final Attack Table Builder ------------

pub fn build_bishop_attack_table() { // Changed to build_bishop_attack_table
    let mut computed_table: Vec<u64> = Vec::new();
    let mut computed_meta = [Magic::EMPTY; 64];

    // Bishops generally have fewer relevant blocker squares than rooks.
    // Max relevant bits for a bishop is around 9 (e.g., on d4/e4). So 2^9 = 512 entries per square.
    // 512 * 64 = 32768 total entries.
    computed_table.reserve(35_000); // A reasonable pre-allocation

    for sq in 0..64 {
        let mask = bishop_mask(sq); // Changed to bishop_mask
        let shift: u8 = BISHOP_SHIFTS[sq]; // Changed to BISHOP_SHIFTS
        let magic = BISHOP_MAGICS[sq]; // Changed to BISHOP_MAGICS

        let blockers = enumerate_blockers(mask);
        let mut used = HashMap::new();
        let size = 1 << (64 - shift);
        let mut local_table = vec![0u64; size];

        for &blocker in &blockers {
            let index = ((blocker & mask).wrapping_mul(magic) >> (shift)) as usize;
            let attack = bishop_attacks_from(sq, blocker); // Changed to bishop_attacks_from

            if let Some(existing) = used.get(&index) {
                if *existing != attack {
                    panic!("Collision detected! Square: {}, Index: {}, Existing: {:b}, New: {:b}",
                           sq, index, existing, attack);
                }
            }

            used.insert(index, attack);
            local_table[index] = attack;
        }

        let offset = computed_table.len();
        computed_table.extend(local_table);

        computed_meta[sq] = Magic {
            mask,
            magic,
            shift,
            offset,
        };
    }

    unsafe {
        // Assign the computed tables to the static mutable variables
        BISHOP_ATTACK_TABLE = computed_table; // Changed to BISHOP_ATTACK_TABLE
        BISHOP_MAGICS_META = computed_meta; // Changed to BISHOP_MAGICS_META

        println!("Final BISHOP_ATTACK_TABLE size: {}", BISHOP_ATTACK_TABLE.len());

        let mut counter = 0;
        for metas in BISHOP_ATTACK_TABLE.clone() {
            print!("{}, ", metas);
            counter += 1;
            if counter % 100 == 0 {
                println!("")
            };
        }
    }
}

// Optional: Helper function to get attacks, showing how to use the table
pub fn get_bishop_attacks(square: usize, occupied_squares: u64) -> u64 { // Changed to get_bishop_attacks
    unsafe {
        let magic_meta = &BISHOP_MAGICS_META[square]; // Changed to BISHOP_MAGICS_META
        let masked_blockers = occupied_squares & magic_meta.mask;
        let index = (masked_blockers.wrapping_mul(magic_meta.magic) >> (magic_meta.shift)) as usize;
        let table_index = magic_meta.offset + index;

        if table_index >= BISHOP_ATTACK_TABLE.len() { // Changed to BISHOP_ATTACK_TABLE
            panic!("Attack table index out of bounds! sq: {}, index: {}, offset: {}, table_idx: {}, table_len: {}",
                   square, index, magic_meta.offset, table_index, BISHOP_ATTACK_TABLE.len());
        }

        BISHOP_ATTACK_TABLE[table_index] // Changed to BISHOP_ATTACK_TABLE
    }
}