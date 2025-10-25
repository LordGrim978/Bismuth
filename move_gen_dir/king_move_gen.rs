use crate::helpers::pop_lsb;
use crate::move_gen_dir::move_gen::Castling::{KingSide, QueenSide};
use crate::move_gen_dir::move_gen::PieceType::{King, NoPiece};
use crate::move_gen_dir::move_gen::{convert_bitboard_to_moves, get_bishop_attacks, get_rook_attacks, GenerationMode, Move, PieceType, KING_MOVES};
use crate::move_gen_dir::knight_move_gen::KNIGHT_MOVES;
use crate::move_gen_dir::precomputed_magics::{PAWN_ATTACKS_BLACK, PAWN_ATTACKS_WHITE};
use crate::move_list::MoveList;
use crate::Board;

pub fn gen_king_moves(board: &Board, move_list: &mut MoveList, checkmask: &u64, generation_mode: &GenerationMode) {
    let (mut king, correction_shift, right_correction_shift) = if board.white_to_move { (board.wking, 0, 0) } else { (board.bking, 56, 2) };
    let (friendly_pieces, enemy_pieces) = if board.white_to_move { (board.white, board.black) } else { (board.black, board.white) };
    let blockers = board.occ;

    if king != 0 {
        let generation_mask = match generation_mode {
            GenerationMode::All => {0xffffffffffffffff},
            GenerationMode::Capture => {enemy_pieces},
            GenerationMode::Check => {0xffffffffffffffff}
        };
        let mut possible_moves = KING_MOVES[pop_lsb(&mut king.clone()) as usize] & !friendly_pieces & generation_mask;

        let mut filtered_moves = 0;


        while possible_moves != 0 {
            let square = pop_lsb(&mut possible_moves);

            if !is_square_attacked(&board, square as u32, king) {
                filtered_moves |= 1 << square;
            }
        }

        let captures = filtered_moves & enemy_pieces;

        convert_bitboard_to_moves(board, move_list, king, filtered_moves & !blockers, King, NoPiece);
        convert_bitboard_to_moves(board, move_list, king, captures, King, NoPiece);

        generate_castling_moves(&board, move_list, checkmask, &mut king, correction_shift, right_correction_shift);
    }
}

fn generate_castling_moves(board: &Board, move_list: &mut MoveList, checkmask: &u64, king: &u64, correction_shift: u32, right_correction_shift: i32) {
    let can_castle_kingside = (board.occ & (0x60 << correction_shift) == 0) && (board.castling_rights & (0b0001 << right_correction_shift) != 0);
    let king_in_check = (king & checkmask) == 0;
    if can_castle_kingside && !is_square_attacked(&board, 5 + correction_shift, 0) && !is_square_attacked(&board, 6 + correction_shift, 0) && !king_in_check {
        move_list.add_move(Move { start_square: 16 << correction_shift, end_square: 64 << correction_shift, capture: NoPiece, piece_type: King, promotion: PieceType::NoPiece, castle: KingSide, en_passant: false })
    }
    let can_castle_queenside = (board.occ & (0xe << correction_shift) == 0) && (board.castling_rights & (0b0010 << right_correction_shift) != 0);
    if can_castle_queenside && !is_square_attacked(&board, 2 + correction_shift, 0) && !is_square_attacked(&board, 3 + correction_shift, 0) && !king_in_check {
        move_list.add_move(Move { start_square: 16 << correction_shift, end_square: 4 << correction_shift, capture: NoPiece, piece_type: King, promotion: PieceType::NoPiece, castle: QueenSide, en_passant: false })
    }
}

const fn is_square_attacked(board: &Board, square_index: u32, king_startpos: u64) -> bool {
    let opponent_pieces = if board.white_to_move { [board.bpawn, board.bknight, board.brook, board.bbishop, board.bqueen, board.bking] } else {[board.wpawn, board.wknight, board.wrook, board.wbishop, board.wqueen, board.wking]};

    let pawn_attack_mask = if board.white_to_move {
        PAWN_ATTACKS_WHITE[square_index as usize]
    } else {
        PAWN_ATTACKS_BLACK[square_index as usize]
    };


    let attackers: u64 = (KNIGHT_MOVES[square_index as usize] & opponent_pieces[1]) |
        (KING_MOVES[square_index as usize] & opponent_pieces[5]) |
        (get_rook_attacks(square_index as usize, board.occ & !king_startpos) & (opponent_pieces[2] | opponent_pieces[4])) |
        (get_bishop_attacks(square_index as usize, board.occ & !king_startpos) & (opponent_pieces[3] | opponent_pieces[4])) |
        (pawn_attack_mask & opponent_pieces[0]);


    return if attackers.count_ones() == 0 {
        false
    } else {
        true
    }
}