//Idea read in file
//extract position into internal, maybe via fen
//zobrist hash
//lookup hash in table
//if found add to vec the next move
// Use Python for it

use crate::fen_import::make_board;
use crate::{generate_all_moves, Board};
use std::collections::HashMap;
use std::fs::File;
use std::{fs, io};
use std::io::Write;
use crate::move_gen_dir::move_gen::GenerationMode;

pub fn make_openings() -> io::Result<()> {
    let contents = fs::read_to_string(
        r"C:\Users\David Meyer\RustroverProjects\ChessEngine\src\OpeningBook\fen.txt"
    )?;

    let mut book: HashMap<u64, Vec<u16>> = HashMap::new();
    let mut current_fens: Vec<String> = Vec::new();

    let startpos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    for (lineno, line) in contents.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.contains('/') {
            // FEN line
            if current_fens.is_empty() {
                // ensure startpos is always first
                if trimmed != startpos {
                    current_fens.push(startpos.to_string());
                }
            }
            current_fens.push(trimmed.to_string());
        } else {
            // Opening name or separator
            if current_fens.len() >= 2 {
                process_fens(&current_fens, &mut book, lineno);
            }
            current_fens.clear();
        }
    }

    if current_fens.len() >= 2 {
        process_fens(&current_fens, &mut book, contents.lines().count());
    }

    // sort + dedup moves
    for moves in book.values_mut() {
        moves.sort_unstable();
        moves.dedup();
    }

    // Write binary book
    let mut file = File::create(
        r"C:\Users\David Meyer\RustroverProjects\ChessEngine\src\OpeningBook\book.bin"
    )?;
    for (hash, moves) in &book {
        file.write_all(&hash.to_le_bytes())?;
        let len = moves.len() as u16;
        file.write_all(&len.to_le_bytes())?;
        for mv in moves {
            file.write_all(&mv.to_le_bytes())?;
        }
    }

    Ok(())
}

fn process_fens(fens: &[String], book: &mut HashMap<u64, Vec<u16>>, lineno: usize) {
    let mut prev = make_board(&fens[0]);

    for fen in &fens[1..] {
        let next = make_board(fen);

        match find_move(&next, &mut prev) {
            Some(mv) => {
                let key = prev.zobrist_hash();
                book.entry(key).or_default().push(mv);
                prev = next;
            }
            None => {
                eprintln!(
                    "Line {}: couldn't find a legal single move from previous FEN to next FEN.",
                    lineno + 1
                );
                prev = next;
            }
        }
    }
}


fn find_move(next: &Board, prev: &mut Board) -> Option<u16> {
    let list = generate_all_moves(prev, &GenerationMode::All); // prev is &mut, but this borrows immutably here

    for i in 0..list.moves_added {
        let mv = list.moves[i];
        let info = prev.make_move(mv);

        let matches = prev.occ == next.occ; // compare board states

        prev.undo_move(info);

        if matches {
            return Some(pack_move(
                mv.start_square.trailing_zeros() as u8,
                mv.end_square.trailing_zeros() as u8,
            ));
        }
    }

    None
}

fn pack_move(start: u8, end: u8) -> u16 {
    ((start as u16) << 8) | (end as u16)
}