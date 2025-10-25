use std::collections::HashMap;
use crate::move_gen_dir::precomputed_magics::{ROOK_MAGICS, ROOK_SHIFTS};

const BOARD_SIZE: usize = 64;

// Directions for rooks (N, S, E, W)
const ROOK_DIRECTIONS: [(i8, i8); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];


// Result storage
// Make these mutable outside the build function as well if you want to initialize them
// with a default capacity, or just assign to them directly.
static mut ROOK_ATTACK_TABLE: Vec<u64> = Vec::new();
static mut ROOK_MAGICS_META: [Magic; 64] = [Magic::EMPTY; 64];

// ------------ Structs ------------

#[derive(Clone, Copy)]
pub struct Magic { // Made pub for external access if needed
    pub mask: u64, // Made pub
    pub magic: u64, // Made pub
    pub shift: u8, // Made pub
    pub offset: usize, // Made pub
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

// popcount is not used in the provided code, but useful for bitboard operations.
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

// Precompute rook masks for each square
fn rook_mask(square: usize) -> u64 {
    let mut mask = 0u64;
    let rank = square / 8;
    let file = square % 8;

    // Horizontal (rank) — exclude edge files
    // Note: It's common to exclude the square itself from the mask for magic bitboards,
    // as the blocker bits are only for squares *between* the piece and the edge/other piece.
    // Your current implementation correctly excludes the piece's square.
    for f in (file + 1)..7 { // From piece + 1 up to file 6
        mask |= 1u64 << (rank * 8 + f);
    }
    for f in (1..file).rev() { // From file 1 down to piece - 1
        mask |= 1u64 << (rank * 8 + f);
    }

    // Vertical (file) — exclude edge ranks
    for r in (rank + 1)..7 { // From piece + 1 up to rank 6
        mask |= 1u64 << (r * 8 + file);
    }
    for r in (1..rank).rev() { // From rank 1 down to piece - 1
        mask |= 1u64 << (r * 8 + file);
    }

    mask
}

// ------------ Attack Generation ------------

fn rook_attacks_from(square: usize, blockers: u64) -> u64 {
    let rank = (square / 8) as i8;
    let file = (square % 8) as i8;
    let mut attacks = 0;

    for &(df, dr) in &ROOK_DIRECTIONS {
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
    let total = 1 << bits.len(); // 2^number_of_bits_in_mask

    for i in 0..total {
        let mut b = 0;
        for (j, &bit) in bits.iter().enumerate() {
            if (i >> j) & 1 == 1 { // Check the j-th bit of i
                b |= 1 << bit; // Set the corresponding bit from the mask
            }
        }
        blockers.push(b);
    }

    blockers
}

// ------------ Final Attack Table Builder ------------

pub fn build_rook_attack_table() {
    // These are local variables that will hold the computed data
    let mut computed_table: Vec<u64> = Vec::new();
    let mut computed_meta = [Magic::EMPTY; 64];

    // Pre-allocate capacity for the main table if possible for performance.
    // A rough estimate can be made using max possible attacks (e.g., 512 per square * 64 squares)
    // For rooks, max blockers is 12 (on A1 or H8), so 2^12 = 4096 attacks per square
    // For rooks, a maximum of 12 bits are relevant for the mask (e.g., for A1, H1, A8, H8)
    // No, the max relevant bits is 12 for the rook on a random square.
    // For example, E4, has 6 bits horizontal (b-g, excluding e) + 6 bits vertical (2-7, excluding 4) = 12 bits
    // So 2^12 = 4096 entries per square. 4096 * 64 = 262144 total entries.
    // This is an upper bound and will be smaller in practice due to the shift.
    // The `ROOK_SHIFTS` should ensure smaller sizes.
    // The total size is sum(2^shift for each square).
    computed_table.reserve(200_000); // A reasonable pre-allocation

    for sq in 0..64 {
        let mask = rook_mask(sq);
        let shift: u8 = ROOK_SHIFTS[sq];
        let magic = ROOK_MAGICS[sq];

        let blockers = enumerate_blockers(mask);
        let mut used = HashMap::new();
        let size = 1 << (64 - shift); // The actual size is 2^(64-shift)
        let mut local_table = vec![0u64; size]; // Initialize with zeros

        for &blocker in &blockers {
            // The index calculation using magic and shift
            let index = ((blocker & mask).wrapping_mul(magic) >> (shift)) as usize;
            let attack = rook_attacks_from(sq, blocker);

            if let Some(existing) = used.get(&index) {
                if *existing != attack {
                    // This indicates an issue with your magic number or a collision.
                    // This panic is crucial for debugging.
                    panic!("Collision detected! Square: {}, Index: {}, Existing: {:b}, New: {:b}",
                           sq, index, existing, attack);
                }
            }

            used.insert(index, attack);
            local_table[index] = attack;
        }

        let offset = computed_table.len(); // The offset for this square's attacks
        computed_table.extend(local_table); // Append the local table to the main table

        computed_meta[sq] = Magic {
            mask,
            magic,
            shift,
            offset,
        };
    }


    // You can also add a way to verify the size if needed after assignment
    unsafe {
        println!("Final ROOK_ATTACK_TABLE size: {}", ROOK_ATTACK_TABLE.len());
    }
    let mut counter = 0;
    for attacks in computed_table {

        if counter > 50000 {
            print!("{attacks}, ");
            if counter % 100 == 0 {
                print!("\n");
            }
        }
        counter += 1;

    }
}

// Optional: Helper function to get attacks, showing how to use the table
pub fn get_rook_attacks(square: usize, occupied_squares: u64) -> u64 {
    unsafe {
        let magic_meta = &ROOK_MAGICS_META[square];
        let masked_blockers = occupied_squares & magic_meta.mask;
        let index = (masked_blockers.wrapping_mul(magic_meta.magic) >> (magic_meta.shift)) as usize;
        let table_index = magic_meta.offset + index;

        // Ensure the index is within bounds (good for debugging)
        if table_index >= ROOK_ATTACK_TABLE.len() {
            panic!("Attack table index out of bounds! sq: {}, index: {}, offset: {}, table_idx: {}, table_len: {}",
                   square, index, magic_meta.offset, table_index, ROOK_ATTACK_TABLE.len());
        }

        ROOK_ATTACK_TABLE[table_index]
    }
}