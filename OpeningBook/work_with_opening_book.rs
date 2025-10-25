use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Read;
use rand::{Open01, Rng};
use crate::{generate_all_moves, Board};
use crate::move_gen_dir::move_gen::{GenerationMode, Move};
use crate::search::search::Searcher;

pub fn unpack_move(m: u16) -> (u8, u8) {
    let start = (m >> 8) as u8;
    let end = (m & 0xFF) as u8;
    (start, end)
}

pub fn load_opening_book(path: &str) -> io::Result<HashMap<u64, Vec<u16>>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    let mut book = HashMap::new();
    let mut i = 0;

    while i + 10 <= buf.len() { // need at least hash (8) + count (2)
        let hash = u64::from_le_bytes(buf[i..i + 8].try_into().unwrap());
        i += 8;

        let count = u16::from_le_bytes(buf[i..i + 2].try_into().unwrap());
        i += 2;

        let mut moves = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mv = u16::from_le_bytes(buf[i..i + 2].try_into().unwrap());
            i += 2;
            moves.push(mv);
        }

        book.insert(hash, moves);
    }

    Ok(book)
}

pub fn get_book_moves<'a>(book: &'a HashMap<u64, Vec<u16>>, hash: &'a u64) -> Option<&'a Vec<u16>> {
    book.get(&hash)
}

fn find_opening_moves(book: &HashMap<u64, Vec<u16>>, hash: u64, board: &mut Board) -> Vec<Move> {
    let mut opening_moves = Vec::new();
    if let Some(moves) = get_book_moves(&book, &hash) {
        let possible_moves = generate_all_moves(board, &GenerationMode::All);
        for mv in moves {
            let (start, end) = unpack_move(*mv);
            for moves in 0..possible_moves.moves_added {
                let current_move = possible_moves.moves[moves];
                if 1<<start == current_move.start_square && 1<<end == current_move.end_square {
                    opening_moves.push(current_move);
                }
            }
        }
    }
    opening_moves
}
pub fn find_opening_move(board: &mut Board)-> Option<Move> {
    let opening_book: HashMap<u64,Vec<u16>> = load_opening_book(r"ToOpeningBook").unwrap();

    let last_double_p_push = board.last_double_pawn_push;
    board.last_double_pawn_push = 0;
    let hash = board.zobrist_hash();
    board.last_double_pawn_push = last_double_p_push;


    let found_opening_moves = find_opening_moves(&opening_book, hash, board);
    if found_opening_moves.len() == 0 {
        None
    } else {
        let num = rand::thread_rng().gen_range(0,found_opening_moves.len());
        Some(found_opening_moves[num])
    }
}