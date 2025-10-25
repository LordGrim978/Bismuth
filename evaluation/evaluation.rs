use crate::evaluation::square_piece_table::PIECE_TABLE;
use crate::helpers::pop_lsb;
use crate::move_gen_dir::move_gen::PieceType;
use crate::Board;

const PIECE_VALUES: [u32; 6] = [100, 300, 315, 500, 900, 0]; //Pawm, Knight, Bishop, Rook, Queen, King
const ENDGAME_MATERIAL_START: u32 = PIECE_VALUES[3] * 2 + PIECE_VALUES[2] + PIECE_VALUES[1];

pub fn evaluate_board(board: &Board) -> i32 {
    let pieces_types = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];
    let mut sum = get_material_value(board) as i32;

    sum += evaluate_piece_square_tables(board, pieces_types);

    let perspective = if board.white_to_move { 1 } else { -1 };
    (sum * perspective)
}


fn evaluate_piece_square_tables(board: &Board, piece_types: [PieceType; 6]) -> i32 {
    let mut sum = 0;

    // Compute material per color (excluding pawns)
    let white_material_wo_pawns = (board.wknight.count_ones() * PIECE_VALUES[1]
        + board.wbishop.count_ones() * PIECE_VALUES[2]
        + board.wrook.count_ones() * PIECE_VALUES[3]
        + board.wqueen.count_ones() * PIECE_VALUES[4]) as i32;

    let black_material_wo_pawns = (board.bknight.count_ones() * PIECE_VALUES[1]
        + board.bbishop.count_ones() * PIECE_VALUES[2]
        + board.brook.count_ones() * PIECE_VALUES[3]
        + board.bqueen.count_ones() * PIECE_VALUES[4]) as i32;

    let white_phase = get_phase(white_material_wo_pawns);
    let black_phase = get_phase(black_material_wo_pawns);

    // 0 = white, 1 = black
    for color in 0..2 {
        let white = color == 0;
        let color_multiplier = if white { 1 } else { -1 };
        let phase = if white { white_phase } else { black_phase };

        for piece in piece_types {
            let mut bb = board.get_pieces(piece, white);
            while bb != 0 {
                let sq = pop_lsb(&mut bb);
                let score = get_evaluation_piece_table(sq, piece, white, phase);
                sum += score * color_multiplier;
            }
        }
    }

    sum
}

fn get_material_value(board: &Board) -> u32 {
    let mut sum = 0;

    sum += board.wpawn.count_ones() * PIECE_VALUES[0];
    sum += board.wknight.count_ones() * PIECE_VALUES[1];
    sum += board.wbishop.count_ones() * PIECE_VALUES[2];
    sum += board.wrook.count_ones() * PIECE_VALUES[3];
    sum += board.wqueen.count_ones() * PIECE_VALUES[4];

    sum -= board.bpawn.count_ones() * PIECE_VALUES[0];
    sum -= board.bknight.count_ones() * PIECE_VALUES[1];
    sum -= board.bbishop.count_ones() * PIECE_VALUES[2];
    sum -= board.brook.count_ones() * PIECE_VALUES[3];
    sum -= board.bqueen.count_ones() * PIECE_VALUES[4];

    sum
}

const fn get_phase(material_without_pawns: i32) -> f32 {
    const MULTIPLIER: f32 = 1.0 / ENDGAME_MATERIAL_START as f32;
    (1.0 - f32::min(1.0, MULTIPLIER * material_without_pawns as f32))
}

fn get_evaluation_piece_table(
    index: u64,
    piece_type: PieceType,
    white: bool,
    phase: f32,
) -> i32 {
    let piece_idx = material_index_from_piece_type(piece_type);
    let corrected_index = (7 - (index / 8)) * 8 + (index % 8);

    if piece_type == PieceType::King {
        // Blend between midgame and endgame tables for the king
        let midgame_idx = if white { 5 } else { 11 };
        let endgame_idx = if white { 12 } else { 13 };

        let mid = PIECE_TABLE[midgame_idx][corrected_index as usize] as f32;
        let end = PIECE_TABLE[endgame_idx][corrected_index as usize] as f32;

        return ((1.0 - phase) * mid + phase * end) as i32;
    }

    // Regular pieces use the normal 0–11 indices
    let table_idx = if white { piece_idx } else { piece_idx + 6 };
    PIECE_TABLE[table_idx][corrected_index as usize]
}

const fn material_index_from_piece_type(piece_type: PieceType) -> usize {
    let piece_index = match piece_type {
        PieceType::Pawn => 0,
        PieceType::Rook => 1,
        PieceType::Knight => 2,
        PieceType::Bishop => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
        _ => 0,
    };
    piece_index
}
