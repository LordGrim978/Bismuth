use crate::evaluation::evaluation::evaluate_board;
use crate::move_gen_dir::move_gen::Castling::NoCastle;
use crate::move_gen_dir::move_gen::PieceType::NoPiece;
use crate::move_gen_dir::move_gen::{GenerationMode, Move};
use crate::OpeningBook::work_with_opening_book::{find_opening_move, get_book_moves, load_opening_book, unpack_move};
use crate::{generate_all_moves, Board, GameState};
use rand::Rng;
use std::collections::HashMap;
use std::process::exit;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use crate::search::transposition_table::{TranspositionTable, LOOKUP_FAILED};
use crate::search::transposition_table::NodeType::{Exact, LowerBound, UpperBound};

pub struct  Searcher {
    pub current_iteration_depth: usize,
    pub nodes: u64,
    pub best_move_this_iteration: EngineMove,
    pub best_move: EngineMove,
    pub has_searched_one_move: bool,
    pub depth : usize,
    pub transposition_table: TranspositionTable,
    pub stop: Arc<AtomicBool>
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct EngineMove {
    pub choosen_move: Move,
    pub eval: i32,
}
const MATE_VALUE: i32 = 10_000_000;

pub const NULL_MOVE: EngineMove = EngineMove{ choosen_move: Move{start_square:0,end_square:0,capture:NoPiece,piece_type:NoPiece,promotion:NoPiece,castle:NoCastle,en_passant:false}, eval: 0 };
impl Searcher {
    pub fn new() -> Self {
        Self{
            current_iteration_depth: 0,
            nodes: 0,
            best_move_this_iteration: NULL_MOVE,
            best_move: NULL_MOVE,
            has_searched_one_move: false,
            depth: 0,
            transposition_table: TranspositionTable::new(128),
            stop: Arc::new(AtomicBool::new(false))
        }
    }

    pub fn iterative_deepening(&mut self, board: &mut Board) {
        self.best_move = NULL_MOVE;
        self.best_move_this_iteration = NULL_MOVE;

        for search_depth in 1..255 {
            self.has_searched_one_move = false;
            self.best_move_this_iteration = NULL_MOVE;


            let score = self.negamax(board, -100_000_000, 100_000_000, search_depth as i32, 0);
            if score == Self::SEARCH_ABORTED || self.stop.load(Ordering::Relaxed) {
                break;
            }

            self.current_iteration_depth = search_depth;

            // Iteration finished cleanly: promote best_move_this_iteration
            if self.has_searched_one_move {
                self.best_move = self.best_move_this_iteration;
            }


        }
    }

    const SEARCH_ABORTED: i32 = 1198680429; //Grim converted to Number
    pub fn negamax(&mut self, board: &mut Board, alpha: i32, beta: i32, depth_left: i32, depth_from_root: usize) -> i32 {
        if self.stop.load(Ordering::Relaxed) {
            return Self::SEARCH_ABORTED;
        }

        let zobrist_hash = board.zobrist_hash();
        let transposition_value = self.transposition_table.lookup_evaluation(zobrist_hash, depth_left, depth_from_root as i32, alpha, beta);
        if transposition_value != LOOKUP_FAILED {
            if depth_from_root == 0 {
                let index = self.transposition_table.index(zobrist_hash);
                self.best_move_this_iteration = EngineMove{ choosen_move: self.transposition_table.try_get_stored_move(zobrist_hash).unwrap(), eval: self.transposition_table.entries[index].value };
                self.has_searched_one_move = true;
            }
            return transposition_value;
        }

        if depth_left == 0 {
            return self.quiescence(board, alpha, beta, depth_from_root);
        }

        let mut move_list = generate_all_moves(board, &GenerationMode::All);
        let board_gamestate = board.game_state(&move_list);


        match board_gamestate {
            GameState::WhiteWin | GameState::BlackWin => {
                return -MATE_VALUE + depth_from_root as i32;
            }
            GameState::Draw => {
                return 0;
            }
            _ => {}
        }
        let pv_move = if depth_from_root == 0 {
            Some(self.best_move.choosen_move)
        } else {
            self.transposition_table.try_get_stored_move(zobrist_hash)
        };

        move_list.order_moves(pv_move);
        let mut alpha = alpha;

        let mut evaluation_bound = UpperBound;
        let mut best_move_this_position = None;

        for i in 0..move_list.moves_added {
            let last_mv_info = board.make_move(move_list.moves[i]);

            let eval = -self.negamax(board, -beta, -alpha, depth_left - 1, depth_from_root +1);
            if eval == Self::SEARCH_ABORTED || eval == -Self::SEARCH_ABORTED {
                return Self::SEARCH_ABORTED; // propagate it up
            }

            board.undo_move(last_mv_info);

            if eval >= beta {
                self.transposition_table.store_evaluation(zobrist_hash, depth_left, depth_from_root as i32, beta, LowerBound, Some(move_list.moves[i]));

                return beta; // Beta cut-off
            }

            if eval > alpha {
                evaluation_bound = Exact;
                best_move_this_position = Some(move_list.moves[i]);

                alpha = eval;

                if depth_from_root == 0 {
                    self.best_move_this_iteration = EngineMove {choosen_move: move_list.moves[i], eval };
                    self.has_searched_one_move = true;
                }
            }
        }
        self.transposition_table.store_evaluation(zobrist_hash, depth_left, depth_from_root as i32, alpha, evaluation_bound, best_move_this_position);

        alpha
    }

    fn quiescence(&mut self, board: &mut Board, mut alpha: i32, beta: i32, depth_from_ply: usize) -> i32 {
        if self.stop.load(Ordering::Relaxed) {
            return Self::SEARCH_ABORTED;
        }

        let mut move_list = generate_all_moves(board, &GenerationMode::Capture);
        let eval = evaluate_board(&board);
        self.nodes += 1;
        if depth_from_ply > self.depth {
            self.depth = depth_from_ply;
        }
        if eval >= beta {
            return beta;
        }
        if eval > alpha {
            alpha = eval;
        }

        move_list.order_moves(None);

        for i in 0..move_list.moves_added {
            let last_mv_info = board.make_move(move_list.moves[i]);
            let score = -self.quiescence(board, -beta, -alpha, depth_from_ply+1);
            board.undo_move(last_mv_info);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        return alpha
    }
}