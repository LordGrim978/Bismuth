use crate::move_gen_dir::move_gen::{Castling, Move, PieceType};
use crate::move_gen_dir::move_gen::PieceType::NoPiece;

const MAX_LEGAL_MOVE_COUNT: usize = 218;

pub struct MoveList {
    pub moves: [Move; MAX_LEGAL_MOVE_COUNT],
    pub moves_added: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList { moves: [Move { start_square: 0, end_square: 0, piece_type: NoPiece, promotion: NoPiece, capture: NoPiece, castle: Castling::NoCastle, en_passant: false }; 218], moves_added: 0 }
    }
    pub fn add_move(&mut self, mv: Move) {
        self.moves[self.moves_added] = mv;
        self.moves_added += 1;
    }

    pub fn order_moves(&mut self, pv_move: Option<Move>) {
        // Assign a score to each move
        let mut scores: Vec<(i32, Move)> = self.moves[..self.moves_added]
            .iter()
            .map(|m| {
                let mut score = self.score_move(m);
                if let Some(pv) = pv_move {
                    if *m == pv {
                        // Boost PV move score massively
                        score += 1_000_000;
                    }
                }
                (score, *m)
            })
            .collect();

        // Sort highest score first
        scores.sort_by(|a, b| b.0.cmp(&a.0));

        // Write back ordered moves
        for (i, (_, mv)) in scores.into_iter().enumerate() {
            self.moves[i] = mv;
        }
    }
    fn score_move(&self, mv: &Move) -> i32 {
        let piece_value = |p: PieceType| -> i32 {
            match p {
                PieceType::Pawn => 208,
                PieceType::Knight => 781,
                PieceType::Bishop => 825,
                PieceType::Rook => 1276,
                PieceType::Queen => 2538,
                PieceType::King => 10_000,
                PieceType::NoPiece => 0,
            }
        };

        let mut score = 0;

        // Captures: MVV/LVA
        if mv.capture != NoPiece {
            score += 10_000
                + piece_value(mv.capture)
                - piece_value(mv.piece_type)
        }

        // Promotions
        if mv.promotion != NoPiece {
            score += 5_000 + piece_value(mv.promotion);
        }

        if mv.castle != Castling::NoCastle {
            score += 1_000;
        }

        score
    }
}