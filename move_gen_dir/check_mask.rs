//Make a array: [[u64,64],64]
//First for king square
//Second for any Slider Piece
//And Result with occ, if more than 1 Bit then Blocked else Check

use crate::Board;
use crate::helpers::pop_lsb;
use crate::move_gen_dir::move_gen::{PinMask};
use crate::move_gen_dir::knight_move_gen::KNIGHT_MOVES;
use crate::move_gen_dir::precomputed_magics::{PAWN_ATTACKS_BLACK, PAWN_ATTACKS_WHITE};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PinDirection {
    Straight,
    Vertical,
    Horizontal,
    Diagonal,
    NoPin
}

// square_between_straight: [[u64; 64]; 64] = generate_squares_between();
pub fn get_checkmask(
    board: &mut Board,
    square_between_straight: &[[u64; 64]; 64],
    square_between_diag: &[[u64; 64]; 64]
) -> (u64, PinMask) {
    let (king_pos, mut opp_pieces, friendly_pieces) = if board.white_to_move {
        (board.wking.trailing_zeros(), [
            board.brook,    // 0
            board.bqueen,   // 1
            board.bbishop,  // 2
            board.bknight,  // 3
            board.bpawn    // 4
        ],
            board.white
        )
    } else {
        (board.bking.trailing_zeros(), [
            board.wrook,
            board.wqueen,
            board.wbishop,
            board.wknight,
            board.wpawn
        ],
            board.black
        )
    };

    if king_pos == 64 {
        return (0xffffffffffffffff, PinMask { horizontal: 0u64, vertical: 0u64, diagonal: 0u64})
    }
    let mut checkmask: u64 = KNIGHT_MOVES[king_pos as usize] & opp_pieces[3];
    let mut pinmask: PinMask = PinMask { horizontal: 0u64, vertical: 0u64, diagonal: 0u64} ;

    // Sliders
    for opp_slider in 0..3 {
        while opp_pieces[opp_slider] != 0 {
            let (slider_check, slider_pin) = generate_masks_sliding_pieces(board, square_between_straight, square_between_diag, king_pos, &mut opp_pieces, friendly_pieces, &opp_slider);
            checkmask |= slider_check;
            match slider_pin.0 {
                PinDirection::Vertical => { pinmask.vertical |= slider_pin.1; }
                PinDirection::Horizontal => { pinmask.horizontal |= slider_pin.1 }
                PinDirection::Diagonal => { pinmask.diagonal |= slider_pin.1 }
                _ => {}
            }
        }
    }


    let pawn_attack_mask = if board.white_to_move {
        PAWN_ATTACKS_WHITE[king_pos as usize]
    } else {
        PAWN_ATTACKS_BLACK[king_pos as usize]
    };

    checkmask |= opp_pieces[4] & pawn_attack_mask;

    match (checkmask & board.occ).count_ones() {
        0 => (0xffffffffffffffff, pinmask), // No check
        1 => (checkmask, pinmask),          // Single check
        _ => (0, pinmask),                  // Double check
    }
}

fn generate_masks_sliding_pieces(
    board: &mut Board,
    square_between_straight: &[[u64; 64]; 64],
    square_between_diag: &[[u64; 64]; 64],
    king_pos: u32, mut opp_pieces: &mut [u64; 5],
    friendly_pieces: u64,
    opp_slider: &usize
) -> (u64, (PinDirection, u64))//Checkmask + Pinmask
{
    let mut checkmask = 0;
    let (mut pindirection, mut pinmask) = (PinDirection::NoPin, 0);

    let piece_sq = pop_lsb(&mut opp_pieces[*opp_slider]) as usize;

    let (slider_attack, direction_mask) = match opp_slider {
        0 => (square_between_straight[king_pos as usize][piece_sq], PinDirection::Straight), // Rook
        1 => {
            let straight = square_between_straight[king_pos as usize][piece_sq];
            let diag = square_between_diag[king_pos as usize][piece_sq];
            if straight != 0 {
                (straight, PinDirection::Straight)
            } else {
                (diag, PinDirection::Diagonal)
            }
        }//Queen
        2 => (square_between_diag[king_pos as usize][piece_sq], PinDirection::Diagonal),     // Bishop
        _ => (0, PinDirection::NoPin),
    };

    if slider_attack == 0 {
        return (0, (PinDirection::NoPin, 0));
    }

    // Count blockers between king and slider
    let blockers = slider_attack & board.occ & !(1<<piece_sq);

    match blockers.count_ones() {
        0 => {
            // No blocker, check
            checkmask |= slider_attack | (1 << piece_sq);
        }
        1 => {
            if blockers & friendly_pieces != 0 {
                // The blocker is friendly, so it's a pin
                match direction_mask {
                    PinDirection::Straight => {
                        if same_rank(
                            king_pos as usize,
                            piece_sq
                        ) {
                            pinmask |= slider_attack | (1 << piece_sq);
                            pindirection = PinDirection::Horizontal
                        } else {
                            pinmask |= slider_attack | (1 << piece_sq);
                            pindirection = PinDirection::Vertical
                        }
                    }
                    PinDirection::Diagonal => {
                        if (blockers & board.last_double_pawn_push) != 0 {
                            board.last_double_pawn_push = 0;
                        } else {
                            pinmask |= slider_attack | (1 << piece_sq);
                            pindirection = PinDirection::Diagonal
                        }
                    }
                    _ => {}
                }
            }
        }
        //En passant Staight Check Pin
        2 => {
            let slider_path_contains_ep_pawn = slider_attack & board.last_double_pawn_push != 0;
            if slider_path_contains_ep_pawn && direction_mask == PinDirection::Straight && !same_file(king_pos as usize, piece_sq) {
                board.last_double_pawn_push = 0;
            };
        }
        _ => {}
    }
    return (checkmask, (pindirection, pinmask));
}

const fn same_rank(a: usize, b: usize) -> bool {
    a / 8 == b / 8
}

const fn same_file(a: usize, b: usize) -> bool {
    a % 8 == b % 8
}
pub fn _generate_squares_between_straight() -> [[u64; 64]; 64] {
    let mut squares_between = [[0u64; 64]; 64];

    for from in 0..64 {
        let from_rank = from / 8;
        let from_file = from % 8;

        for to in 0..64 {
            if from == to {
                continue;
            }

            let to_rank = to / 8;
            let to_file = to % 8;

            let dr = to_rank as i32 - from_rank as i32;
            let df = to_file as i32 - from_file as i32;

            // Only orthogonal (straight) directions
            if dr == 0 || df == 0 {
                let step_rank = dr.signum();
                let step_file = df.signum();

                let mut r = from_rank as i32 + step_rank;
                let mut f = from_file as i32 + step_file;

                let mut between = 0u64;

                while r != to_rank as i32 || f != to_file as i32 {
                    let sq = (r * 8 + f) as usize;
                    between |= 1u64 << sq;
                    r += step_rank;
                    f += step_file;
                }
                between |= 1u64 << to;
                squares_between[from][to] = between;
            }
        }
    }

    squares_between
}

pub fn _generate_squares_between_diagonal() -> [[u64; 64]; 64] {
    let mut squares_between = [[0u64; 64]; 64];

    for from in 0..64 {
        let from_rank = from / 8;
        let from_file = from % 8;

        for to in 0..64 {
            if from == to {
                continue;
            }

            let to_rank = to / 8;
            let to_file = to % 8;

            let dr = to_rank as i32 - from_rank as i32;
            let df = to_file as i32 - from_file as i32;

            // Only diagonals
            if dr.abs() == df.abs() {
                let step_rank = dr.signum();
                let step_file = df.signum();

                let mut r = from_rank as i32 + step_rank;
                let mut f = from_file as i32 + step_file;

                let mut between = 0u64;

                while r != to_rank as i32 || f != to_file as i32 {
                    let sq = (r * 8 + f) as usize;
                    between |= 1u64 << sq;
                    r += step_rank;
                    f += step_file;
                }
                between |= 1u64 << to;
                squares_between[from][to] = between;
            }
        }
    }

    squares_between
}
