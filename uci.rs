use crate::fen_import::{make_board, start_pos};
use crate::helpers::{index_to_sq, sq_to_index};
use crate::move_gen_dir::move_gen::{GenerationMode, Move, PieceType, Square};
use crate::search::search::{Searcher, NULL_MOVE};
use crate::{generate_all_moves, Board};
use std::{io, thread};
use std::io::Write;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use crate::OpeningBook::work_with_opening_book::find_opening_move;

fn convert_mv_to_uci(mv: Move) -> String {
    let mut move_str: String = "".to_owned();

    let start_sq =  index_to_sq(mv.start_square.trailing_zeros() as usize);
    let end_sq =  index_to_sq(mv.end_square.trailing_zeros() as usize);

    move_str.push_str(&start_sq.to_string());
    move_str.push_str(&end_sq.to_string());

    if mv.promotion != PieceType::NoPiece {
        match mv.promotion {
            PieceType::Rook => move_str.push_str("r"),
            PieceType::Queen => move_str.push_str("q"),
            PieceType::Bishop => move_str.push_str("b"),
            PieceType::Knight => move_str.push_str("n"),
            _ => {}
        }
    }

    return move_str.to_lowercase();
}


pub fn uci_loop() {
    let mut board = start_pos();

    let searcher = Arc::new(Mutex::new(Searcher::new()));
    let mut search_thread: Option<std::thread::JoinHandle<()>> = None;

    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }
        let input = input.trim();

        if input == "uci" {
            println!("id name bismuth");
            println!("id author lordgrim");
            println!("uciok");
            io::stdout().flush().unwrap();
        }
        else if input == "isready" {
            println!("readyok");
            io::stdout().flush().unwrap();
        }
        else if input.starts_with("position") {
            position_command(&mut board, input);
            io::stdout().flush().unwrap();
        }
        else if input.starts_with("go") {
            go_command(&mut board, &searcher, &mut search_thread);
            io::stdout().flush().unwrap();
        }
        else if input == "stop" {
            // stop_flag.store(true, Ordering::Relaxed);
            //
            // if let Some(handle) = search_thread.take() {
            //     let _ = handle.join(); // wait for search to finish cleanly
            // }
        }
        else if input == "quit" {
            break;
        }
    }
}



fn position_command(board: &mut Board, input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() >= 2 {
        *board = if parts[1] == "startpos" {
            start_pos()
        } else if parts[1] == "fen" {
            let fen = parts[2..6].join(" ");
            make_board(&fen)
        } else {
            start_pos()
        };
        if let Some(idx) = parts.iter().position(|&x| x == "moves") {
            for mv_str in &parts[idx + 1..] {
                convert_uci_to_internal(board, &mv_str);
            }
        }
    }
}

fn convert_uci_to_internal(mut board: &mut Board, input: &str) {
    let start_sq_str: Square = *&input[..2].to_uppercase().parse().unwrap();
    let end_sp_str: Square = *&input[2..4].to_uppercase().parse().unwrap();

    let start_sq: u64 = 1<<sq_to_index(start_sq_str);
    let end_sq: u64 = 1<<sq_to_index(end_sp_str);

    let all_possible_moves = generate_all_moves(&mut board, &GenerationMode::All);

    for moves in 0..all_possible_moves.moves_added {
        let move_from_idx = all_possible_moves.moves[moves];

        if move_from_idx.start_square == start_sq && move_from_idx.end_square == end_sq {
            if move_from_idx.promotion == PieceType::NoPiece {
                board.make_move(move_from_idx);
                break;
            }
            if input.len() == 5 {
                let promotion = &input[4..5];
                match promotion {
                    "q" => { if move_from_idx.promotion == PieceType::Queen { board.make_move(move_from_idx); } }
                    "n" => { if move_from_idx.promotion == PieceType::Knight { board.make_move(move_from_idx); } }
                    "b" => { if move_from_idx.promotion == PieceType::Bishop { board.make_move(move_from_idx); } }
                    "r" => { if move_from_idx.promotion == PieceType::Rook { board.make_move(move_from_idx); } }
                    _ => {}
                }
            }
        }
    }
}

fn go_command(board: &mut Board, searcher: &Arc<Mutex<Searcher>>, search_thread: &mut Option<std::thread::JoinHandle<()>>) {
    // Clone board for the search thread
    let mut board_clone = board.clone();

    let opening_move = find_opening_move(board);

    if opening_move.is_some(){
        println!("bestmove {}", convert_mv_to_uci(opening_move.unwrap()));
        return;
    }

    // Extract a clone of the stop flag (fast, brief lock)
    let stop_flag: Arc<std::sync::atomic::AtomicBool> = {
        let s = searcher.lock().unwrap();
        s.stop.clone()
    };

    // Clear stop flag without holding the big mutex
    stop_flag.store(false, Ordering::SeqCst);

    // Spawn the search thread (it may lock the searcher for mutation as before)
    let searcher_for_thread = Arc::clone(searcher);
    *search_thread = Some(thread::spawn(move || {
        {
            // lock briefly to call iterative_deepening - this will hold the mutex while searching
            // that's OK because the timer doesn't need the mutex anymore (it has stop_flag).
            let mut s = searcher_for_thread.lock().unwrap();
            s.iterative_deepening(&mut board_clone);
        }

        // after search completes, read the results under lock
        let (best, positions, depth) = {
            let s = searcher_for_thread.lock().unwrap();
            (s.best_move, s.nodes, s.depth)
        };
        println!("info score {} nodes {positions} depth {depth}", best.eval);
        println!("bestmove {}", convert_mv_to_uci(best.choosen_move));

        let mut s = searcher_for_thread.lock().unwrap();
        s.depth = 0;
        s.nodes = 0;
        io::stdout().flush().unwrap();
    }));

    // Spawn a *timer* thread that uses the cloned stop_flag (no mutex lock)
    let stop_for_timer = stop_flag.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        // set stop using the atomic directly (no mutex locking)
        stop_for_timer.store(true, Ordering::SeqCst);
    });
}