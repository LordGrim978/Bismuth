use std::collections::HashMap;
use move_gen_dir::move_gen::{generate_bishop_moves, generate_queen_moves, generate_rook_moves, Castling, GenerationMode, Move, PieceType};

mod move_gen_dir;

use crate::move_gen_dir::check_mask::get_checkmask;
use crate::move_gen_dir::king_move_gen::gen_king_moves;
use crate::move_gen_dir::knight_move_gen::generate_knight_moves;
use crate::move_gen_dir::pawn_move_gen::generate_pawn_moves;
use crate::move_gen_dir::precomputed_magics::{SQUARES_BETWEEN_DIAGONAL, SQUARES_BETWEEN_STRAIGHT};
use crate::move_list::MoveList;
use crate::search::repition_table::RepetitionTable;
use crate::uci::uci_loop;
use colored::Colorize;
use move_gen_dir::move_gen::PieceType::{King, Pawn};
use crate::fen_import::{make_board, start_pos};
use crate::move_gen_dir::move_gen_tests::_test_move_gen;
use crate::OpeningBook::generate_opening_book::make_openings;
use crate::OpeningBook::work_with_opening_book::{find_opening_move, get_book_moves, load_opening_book};
use crate::search::search::Searcher;

mod fen_import;
mod helpers;
mod pregenerate_functions;
mod move_list;
mod generating_magics;
mod uci;
mod zobrist_hashing;
mod OpeningBook;
mod evaluation;
mod search;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Board {
    pub bpawn: u64,
    pub bknight: u64,
    pub bbishop: u64,
    pub brook: u64,
    pub bqueen: u64,
    pub bking: u64,
    pub wpawn: u64,
    pub wknight: u64,
    pub wbishop: u64,
    pub wrook: u64,
    pub wqueen: u64,
    pub wking: u64,
    pub black: u64,
    pub white: u64,
    pub occ: u64,
    pub castling_rights: u8, // First White King Second White Queen, This Black King, Fourth Black Queen
    pub last_double_pawn_push: u64,
    pub white_to_move: bool,
    pub position_history: RepetitionTable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MoveInfo {
    pub last_move: Move,
    pub captured_piece: PieceType,
    pub castling_rights: u8,
    pub last_double_pawn_push: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameState {
    WhiteWin,
    BlackWin,
    Draw,
    Ongoing
}

impl Board {
    pub const fn new(
        bp: u64, bn: u64, bb: u64, br: u64, bq: u64, bk: u64,
        wp: u64, wn: u64, wb: u64, wr: u64, wq: u64, wk: u64,
        white_to_move: bool,
        castling_rights: u8,
        en_passant: u64,
    ) -> Self {
        let black = bp | bn | bb | br | bq | bk;
        let white = wp | wn | wb | wr | wq | wk;
        let occ = black | white;
        Self {
            bpawn: bp, bknight: bn, bbishop: bb, brook: br, bqueen: bq, bking: bk,
            wpawn: wp, wknight: wn, wbishop: wb, wrook: wr, wqueen: wq, wking: wk,
            black,
            white,
            occ,
            white_to_move,
            last_double_pawn_push: en_passant,
            castling_rights,
            position_history: RepetitionTable::new(),
        }
    }
    pub fn print_board(&self) {
        let mut board_chars = [' '; 64];

        for i in 0..64 {
            let mask = 1u64 << i;

            board_chars[i] = if self.wking & mask != 0 {
                '♔'
            } else if self.wqueen & mask != 0 {
                '♕'
            } else if self.wrook & mask != 0 {
                '♖'
            } else if self.wbishop & mask != 0 {
                '♗'
            } else if self.wknight & mask != 0 {
                '♘'
            } else if self.wpawn & mask != 0 {
                '♙'
            } else if self.bking & mask != 0 {
                '♚'
            } else if self.bqueen & mask != 0 {
                '♛'
            } else if self.brook & mask != 0 {
                '♜'
            } else if self.bbishop & mask != 0 {
                '♝'
            } else if self.bknight & mask != 0 {
                '♞'
            } else if self.bpawn & mask != 0 {
                '♟'
            } else {
                '.'
            };
        }
        // Print the board rank by rank
        println!("\n  +------------------------+");
        for rank in 0..8 {
            print!("{} |", 8 - rank);
            for file in 0..8 {
                let index = (7 - rank) * 8 + file;
                print!(" {} ", board_chars[index]);
            }
            println!("|");
        }
        println!("  +------------------------+");
        println!("    a  b  c  d  e  f  g  h");
    }

    pub fn make_move(&mut self, mv: Move)-> MoveInfo {
        let move_mask = mv.start_square | mv.end_square;

        let move_board: &mut u64 = match (self.white_to_move, mv.piece_type) {
            (true, PieceType::Pawn) => &mut self.wpawn,
            (true, PieceType::Rook) => &mut self.wrook,
            (true, PieceType::Knight) => &mut self.wknight,
            (true, PieceType::Bishop) => &mut self.wbishop,
            (true, PieceType::Queen) => &mut self.wqueen,
            (true, PieceType::King) => &mut self.wking,
            (false, PieceType::Pawn) => &mut self.bpawn,
            (false, PieceType::Rook) => &mut self.brook,
            (false, PieceType::Knight) => &mut self.bknight,
            (false, PieceType::Bishop) => &mut self.bbishop,
            (false, PieceType::Queen) => &mut self.bqueen,
            (false, PieceType::King) => &mut self.bking,
            _ => &mut 1u64,
        };
        *move_board ^= move_mask;

        match (self.white_to_move, mv.promotion) {
            (true, PieceType::Queen) => { self.wqueen |= mv.end_square; },
            (true, PieceType::Rook) => { self.wrook |= mv.end_square; },
            (true, PieceType::Bishop) => { self.wbishop |= mv.end_square; },
            (true, PieceType::Knight) => { self.wknight |= mv.end_square; },
            (false, PieceType::Queen) => { self.bqueen |= mv.end_square; },
            (false, PieceType::Rook) => { self.brook |= mv.end_square; },
            (false, PieceType::Bishop) => { self.bbishop |= mv.end_square; },
            (false, PieceType::Knight) => { self.bknight |= mv.end_square; },
            _ => {}
        }
        self.wpawn &= 0xffffffffffff00;
        self.bpawn &= 0xffffffffffff00;

        if mv.en_passant {
            self.wpawn &= !self.last_double_pawn_push;
            self.bpawn &= !self.last_double_pawn_push;
        }

        let pre_last_double_pawn_push = self.last_double_pawn_push;
        if (mv.start_square & 0xff00000000ff00) != 0 && (mv.end_square & 0xffff000000) != 0 && mv.piece_type == Pawn {
            self.last_double_pawn_push = mv.end_square;
        } else {
            self.last_double_pawn_push = 0;
        }

        match (self.white_to_move, mv.capture) {
            (true, PieceType::Pawn) => { self.bpawn &= !mv.end_square; }
            (true, PieceType::Rook) => { self.brook &= !mv.end_square; }
            (true, PieceType::Knight) => { self.bknight &= !mv.end_square; }
            (true, PieceType::Bishop) => { self.bbishop &= !mv.end_square; }
            (true, PieceType::Queen) => { self.bqueen &= !mv.end_square; }
            (true, PieceType::King) => { self.bking &= !mv.end_square; }
            (false, PieceType::Pawn) => { self.wpawn &= !mv.end_square; }
            (false, PieceType::Rook) => { self.wrook &= !mv.end_square; }
            (false, PieceType::Knight) => { self.wknight &= !mv.end_square; }
            (false, PieceType::Bishop) => { self.wbishop &= !mv.end_square; }
            (false, PieceType::Queen) => { self.wqueen &= !mv.end_square; }
            (false, PieceType::King) => { self.wking &= !mv.end_square; }
            _ => {}
        }

        let not_updated_castling_rights = self.castling_rights;
        self.update_castling_rights(mv);
        match mv.castle {
            Castling::KingSide => {
                if self.white_to_move {
                    self.wrook ^= 0xa0;
                    self.castling_rights &= 0b00001100
                } else {
                    self.brook ^= 0xa000000000000000;
                    self.castling_rights &= 0b00000011
                }
            },
            Castling::QueenSide => {
                if self.white_to_move {
                    self.wrook ^= 0x9;
                    self.castling_rights &= 0b00001100
                } else {
                    self.brook ^= 0x900000000000000;
                    self.castling_rights &= 0b00000011
                }
            },
            Castling::NoCastle => {},
        }

        self.white_to_move = !self.white_to_move;

        self.set_occ();
        self.position_history.add(self.zobrist_hash());

        return MoveInfo{ last_move: mv, captured_piece: mv.capture, castling_rights: not_updated_castling_rights, last_double_pawn_push: pre_last_double_pawn_push };
    }

    fn set_occ(&mut self) {
        self.white = self.wrook | self.wqueen | self.wking | self.wpawn | self.wknight | self.wbishop;
        self.black = self.brook | self.bqueen | self.bking | self.bpawn | self.bknight | self.bbishop;
        self.occ = self.black | self.white;
    }

    fn update_castling_rights(&mut self, mv: Move) {
        if ((mv.start_square | mv.end_square) & 0x80) != 0 {
            self.castling_rights &= 0b1110
        }
        if ((mv.start_square | mv.end_square) & 1) != 0 {
            self.castling_rights &= 0b1101
        }
        if ((mv.start_square | mv.end_square) & 9223372036854775808) != 0 {
            self.castling_rights &= 0b1011
        }
        if ((mv.start_square | mv.end_square) & 72057594037927936) != 0 {
            self.castling_rights &= 0b0111
        }
        if mv.piece_type == King && self.white_to_move == true {
            self.castling_rights &= 0b1100
        }
        if mv.piece_type == King && self.white_to_move == false {
            self.castling_rights &= 0b0011
        }
    }

    pub fn undo_move(&mut self, last_mv: MoveInfo) {
        let move_mask = last_mv.last_move.start_square | last_mv.last_move.end_square;

        let move_board: &mut u64 = match (!self.white_to_move, last_mv.last_move.piece_type) {
            (true, PieceType::Pawn) => &mut self.wpawn,
            (true, PieceType::Rook) => &mut self.wrook,
            (true, PieceType::Knight) => &mut self.wknight,
            (true, PieceType::Bishop) => &mut self.wbishop,
            (true, PieceType::Queen) => &mut self.wqueen,
            (true, PieceType::King) => &mut self.wking,
            (false, PieceType::Pawn) => &mut self.bpawn,
            (false, PieceType::Rook) => &mut self.brook,
            (false, PieceType::Knight) => &mut self.bknight,
            (false, PieceType::Bishop) => &mut self.bbishop,
            (false, PieceType::Queen) => &mut self.bqueen,
            (false, PieceType::King) => &mut self.bking,
            _ => {&mut 1u64},
        };
        *move_board ^= move_mask;

        self.wpawn &= 0xffffffffffff00;
        self.bpawn &= 0xffffffffffff00;

        match (!self.white_to_move, last_mv.last_move.promotion) {
            (true, PieceType::Queen) => { self.wqueen &= !last_mv.last_move.end_square },
            (true, PieceType::Knight) => { self.wknight &= !last_mv.last_move.end_square },
            (true, PieceType::Bishop) => { self.wbishop &= !last_mv.last_move.end_square },
            (true, PieceType::Rook) => { self.wrook &= !last_mv.last_move.end_square },
            (false, PieceType::Queen) => { self.bqueen &= !last_mv.last_move.end_square },
            (false, PieceType::Knight) => { self.bknight &= !last_mv.last_move.end_square },
            (false, PieceType::Bishop) => { self.bbishop &= !last_mv.last_move.end_square },
            (false,PieceType::Rook) => { self.brook &= !last_mv.last_move.end_square },
            _ => {},
        }

        match (self.white_to_move, last_mv.captured_piece) {
            (true, PieceType::Pawn) => { self.wpawn |= last_mv.last_move.end_square },
            (true, PieceType::Rook) => { self.wrook |= last_mv.last_move.end_square },
            (true, PieceType::Knight) => { self.wknight |= last_mv.last_move.end_square },
            (true, PieceType::Bishop) => { self.wbishop |= last_mv.last_move.end_square },
            (true, PieceType::Queen) => { self.wqueen |= last_mv.last_move.end_square },
            (true, PieceType::King) => { self.wking |= last_mv.last_move.end_square },
            (false, PieceType::Pawn) => { self.bpawn |= last_mv.last_move.end_square },
            (false, PieceType::Rook) => { self.brook |= last_mv.last_move.end_square },
            (false, PieceType::Knight) => { self.bknight |= last_mv.last_move.end_square },
            (false, PieceType::Bishop) => { self.bbishop |= last_mv.last_move.end_square },
            (false, PieceType::Queen) => { self.bqueen |= last_mv.last_move.end_square },
            (false, PieceType::King) => { self.bking |= last_mv.last_move.end_square },
            _ => {}
        }

        match (!self.white_to_move, last_mv.last_move.castle) {
            (true, Castling::KingSide) => { self.wrook ^= 0xa0 }
            (true, Castling::QueenSide) => { self.wrook ^= 0x9 }
            (false, Castling::KingSide) => { self.brook ^= 0xa000000000000000 }
            (false, Castling::QueenSide) => { self.brook ^= 0x900000000000000 }
            _ => {}
        }
        match (!self.white_to_move, last_mv.last_move.en_passant) {
            (true, true) => {
                self.bpawn &= !last_mv.last_move.end_square;
                self.bpawn |= last_mv.last_move.end_square >> 8
            }
            (false, true) => {
                self.wpawn &= !last_mv.last_move.end_square;
                self.wpawn |= last_mv.last_move.end_square << 8
            }
            _ => {}
        }

        self.castling_rights = last_mv.castling_rights;

        self.last_double_pawn_push = last_mv.last_double_pawn_push;

        self.set_occ();

        self.position_history.pop_last();

        self.white_to_move = !self.white_to_move;
    }

    pub fn game_state(&mut self, moves: &MoveList) -> GameState {
        if self.position_history.contains(self.zobrist_hash()) {
            return GameState::Draw
        }
        if moves.moves_added != 0 {
            GameState::Ongoing
        } else {
            let (checkmask, _pinmask) = get_checkmask(self, &SQUARES_BETWEEN_STRAIGHT, &SQUARES_BETWEEN_DIAGONAL);

            if checkmask == 0xffffffffffffffff {
                GameState::Draw
            } else {
                match self.white_to_move {
                    false => GameState::WhiteWin,
                    true => GameState::BlackWin,
                }
            }
        }

    }

    pub const fn get_pieces(&self, piece_type: PieceType, white_to_move: bool) -> u64 {
        return match (piece_type, white_to_move) {
            (PieceType::Pawn, true) => self.wpawn,
            (PieceType::Rook, true) => self.wrook,
            (PieceType::Knight, true) => self.wknight,
            (PieceType::Bishop, true) => self.wbishop,
            (PieceType::Queen, true) => self.wqueen,
            (PieceType::King, true) => self.wking,
            (PieceType::Pawn, false) => self.bpawn,
            (PieceType::Rook, false) => self.brook,
            (PieceType::Knight, false) => self.bknight,
            (PieceType::Bishop, false) => self.bbishop,
            (PieceType::Queen, false) => self.bqueen,
            (PieceType::King, false) => self.bking,
            _   => { 0 }
        }
    }
}

fn main() {
    uci_loop();
    // time_move_gen();

    // use std::time::Instant;
    // let now = Instant::now();
    // let mut board = make_board("3r2k1/5ppp/8/8/8/8/2R2PPP/6K1 b - - 0 1");
    // // let mut board = start_pos();
    //
    // let depth = 1;
    // let mut seacher = Searcher::new(depth);
    // println!("{}",depth);
    // seacher.negamax(&mut board, -100000000, 100000000, depth as i32);
    // println!("{:?}", seacher.nodes);
    // println!("{:?}",seacher.best_move);
    // //
    // let elapsed = now.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);
}

fn time_move_gen() {
    use std::time::Instant;
    let now = Instant::now();

    _test_move_gen();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}


fn generate_all_moves(board: &mut Board, generation_mode: &GenerationMode) -> MoveList {
    let mut move_list = MoveList::new();

    let (checkmask, pinmask) = get_checkmask(board, &SQUARES_BETWEEN_STRAIGHT, &SQUARES_BETWEEN_DIAGONAL);

    if checkmask == 0 {
        gen_king_moves(&board, &mut move_list, &checkmask, generation_mode);
    } else {
        generate_pawn_moves(&board, &mut move_list, &checkmask, &pinmask, generation_mode);
        generate_knight_moves(&board, &mut move_list, &checkmask, &pinmask, generation_mode);
        generate_bishop_moves(&board, if board.white_to_move { board.wbishop } else { board.bbishop }, PieceType::Bishop, &mut move_list, &checkmask, &pinmask, generation_mode);
        generate_rook_moves(&board, if board.white_to_move { board.wrook } else { board.brook }, PieceType::Rook, &mut move_list, &checkmask, &pinmask, generation_mode);
        generate_queen_moves(board, &mut move_list, &checkmask, &pinmask, generation_mode);
        gen_king_moves(&board, &mut move_list, &checkmask, generation_mode);
    }

    move_list
}