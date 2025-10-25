use crate::helpers::pop_lsb;
use crate::move_list::MoveList;
use crate::Board;
use crate::move_gen_dir::bishop_table_const::BISHOP_ATTACK_TABLE;
use crate::move_gen_dir::precomputed_magics::{BISHOP_MAGICS, BISHOP_MASK, BISHOP_OFFSETS, BISHOP_SHIFTS, ROOK_MAGICS, ROOK_MASK, ROOK_OFFSETS, ROOK_SHIFTS};
use crate::move_gen_dir::rook_table_const::ROOK_ATTACK_TABLE;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Move {
    pub start_square: u64,
    pub end_square: u64,
    pub capture: PieceType,
    pub piece_type: PieceType,
    pub promotion: PieceType,
    pub castle: Castling,
    pub en_passant: bool,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Castling {
    KingSide,
    QueenSide,
    NoCastle
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    NoPiece
}

#[derive(Debug, Clone, Copy, EnumString, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[derive(Debug, Clone, Copy)]
pub struct PinMask {
    pub horizontal: u64,
    pub vertical: u64,
    pub diagonal: u64,
}

pub enum GenerationMode {
    All,
    Capture,
    Check
}

pub const KING_MOVES: [u64;64] = [770, 1797, 3594, 7188, 14376, 28752, 57504, 49216, 197123, 460039, 920078, 1840156, 3680312, 7360624, 14721248, 12599488, 50463488, 117769984, 235539968, 471079936, 942159872, 1884319744, 3768639488, 3225468928, 12918652928, 30149115904, 60298231808, 120596463616, 241192927232, 482385854464, 964771708928, 825720045568, 3307175149568, 7718173671424, 15436347342848, 30872694685696, 61745389371392, 123490778742784, 246981557485568, 211384331665408, 846636838289408, 1975852459884544, 3951704919769088, 7903409839538176, 15806819679076352, 31613639358152704, 63227278716305408, 54114388906344448, 216739030602088448, 505818229730443264, 1011636459460886528, 2023272918921773056, 4046545837843546112, 8093091675687092224, 16186183351374184448, 13853283560024178688, 144959613005987840, 362258295026614272, 724516590053228544, 1449033180106457088, 2898066360212914176, 5796132720425828352, 11592265440851656704, 4665729213955833856];

pub fn convert_bitboard_to_moves(board: &Board, moves: &mut MoveList, start: u64, mut bitboard: u64, piece_type: PieceType, promotion: PieceType) {
    while bitboard != 0 {
        let destination = 1 << pop_lsb(&mut bitboard);
        let captured_piece = get_piece_from_square(board, destination);
        moves.add_move(Move { start_square: start, end_square: destination, piece_type: piece_type, promotion: promotion , capture: captured_piece, castle: Castling::NoCastle, en_passant: false } );
    }
}

const fn get_piece_from_square(board: &Board,square: u64) -> PieceType {
    if !board.occ & square != 0 {
        return PieceType::NoPiece;
    }
    if square & (board.wpawn | board.bpawn) != 0 {
        return PieceType::Pawn;
    } else if square & (board.wrook | board.brook) != 0 {
        return PieceType::Rook;
    } else if square & (board.wbishop | board.bbishop) != 0 {
        return PieceType::Bishop;
    } else if square & (board.wknight | board.bknight) != 0 {
        return PieceType::Knight;
    } else if square & (board.wqueen | board.bqueen) != 0 {
        return PieceType::Queen;
    } else if square & (board.wking | board.bking) != 0 {
        return PieceType::King;
    } else { return PieceType::NoPiece }
}

pub fn generate_rook_moves(board: &Board, start_squares: u64, piece_type: PieceType, move_list: &mut MoveList, checkmask: &u64, pin_mask: &PinMask, generation_mode: &GenerationMode) {
    let mut rooks = start_squares;

    while rooks != 0 {
        let square = pop_lsb(&mut rooks);

        let rook_bitboard = 1 << square;
        if rook_bitboard & pin_mask.diagonal != 0 {
            continue;
        }

        let mut possible_moves = get_rook_attacks(square as usize, board.occ);


        possible_moves &= if board.white_to_move {!board.white}else{!board.black};
        possible_moves &= checkmask;

        let pinmask_horizontal = !(!pin_mask.horizontal * ((pin_mask.horizontal & rook_bitboard).count_ones() as u64));
        let pinmask_vertical = !(!pin_mask.vertical * ((pin_mask.vertical & rook_bitboard).count_ones() as u64));
        possible_moves &= pinmask_vertical & pinmask_horizontal;


        let captures = possible_moves & if board.white_to_move { board.black } else { board.white };
        let quiets = possible_moves & !captures;

        generate_moves_filtered_by_mode(board, piece_type, move_list, generation_mode, square, captures, quiets);

    }
}

fn generate_moves_filtered_by_mode(board: &Board, piece_type: PieceType, move_list: &mut MoveList, generation_mode: &GenerationMode, square: u64, captures: u64, quiets: u64) {
    match generation_mode {
        GenerationMode::All => {
            convert_bitboard_to_moves(board, move_list, 1 << square, quiets, piece_type, PieceType::NoPiece);
            convert_bitboard_to_moves(board, move_list, 1 << square, captures, piece_type, PieceType::NoPiece);
        },
        GenerationMode::Capture => {
            convert_bitboard_to_moves(board, move_list, 1 << square, captures, piece_type, PieceType::NoPiece);
        },
        GenerationMode::Check => {}
    }
}

pub const fn get_rook_attacks(square: usize, occupancy: u64) -> u64 {
    let relevant_occupancy = occupancy & ROOK_MASK[square];
    let index = ((relevant_occupancy.wrapping_mul(ROOK_MAGICS[square])) >> ROOK_SHIFTS[square]) as usize;
    ROOK_ATTACK_TABLE[(ROOK_OFFSETS[square] as usize) + index]
}


pub fn generate_bishop_moves(board: &Board, start_square: u64, piece_type: PieceType, move_list: &mut MoveList, checkmask: &u64, pin_mask: &PinMask, generation_mode: &GenerationMode) {
    let mut bishops = start_square;

    while bishops != 0 {
        let square = pop_lsb(&mut bishops);

        let not_movable = (1 << square) & (pin_mask.horizontal | pin_mask.vertical);
        if not_movable != 0 {
            continue;
        }
        let movement_mask = match ((1 << square) & pin_mask.diagonal).count_ones() {
            1 => pin_mask.diagonal,
            _ => 0xffffffffffffffff,
        };

        let mut possible_moves = get_bishop_attacks(square as usize, board.occ);


        possible_moves &= if board.white_to_move { !board.white } else { !board.black };
        possible_moves &= checkmask & movement_mask;

        let captures = possible_moves & if board.white_to_move { board.black } else { board.white };
        let quiets = possible_moves & !captures;

        generate_moves_filtered_by_mode(board, piece_type, move_list, generation_mode, square, captures, quiets);
    }
}
pub const fn get_bishop_attacks(square: usize, occupancy: u64) -> u64 {
    let relevant_occupancy = occupancy & BISHOP_MASK[square];
    let index = (relevant_occupancy.wrapping_mul(BISHOP_MAGICS[square]) >> BISHOP_SHIFTS[square]) as usize;
    BISHOP_ATTACK_TABLE[(BISHOP_OFFSETS[square] as usize) + index]
}


pub fn generate_queen_moves(board: &mut Board, move_list: &mut MoveList, checkmask: &u64, pin_mask: &PinMask, generation_mode: &GenerationMode) {
    let quens = if board.white_to_move { board.wqueen } else { board.bqueen };
    generate_bishop_moves(board, quens, PieceType::Queen, move_list, checkmask, pin_mask, generation_mode);
    generate_rook_moves(board, quens, PieceType::Queen, move_list, checkmask, pin_mask, generation_mode);
}