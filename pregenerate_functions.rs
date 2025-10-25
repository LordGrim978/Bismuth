pub fn _pregenerate_knight_moves() -> [u64; 64] {
    let mut knight_moves: [u64; 64] = [0; 64];

    for square in 0..64 {
        let rank = square / 8;
        let file = square % 8;

        let mut moves = 0u64;

        let knight_deltas = [
            (2, 1),
            (2, -1),
            (-2, 1),
            (-2, -1),
            (1, 2),
            (1, -2),
            (-1, 2),
            (-1, -2),
        ];

        for (dr, df) in knight_deltas {
            let new_rank = rank as i32 + dr;
            let new_file = file as i32 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let destination = (new_rank * 8 + new_file) as u64;
                moves |= 1u64 << destination;
            }
        }
        knight_moves[square] = moves;
    }
    knight_moves
}

pub fn _pregenerate_king_moves() -> [u64; 64] {
    let mut king_moves: [u64; 64] = [0; 64];
    for square in 0..64 {
        let rank = square / 8;
        let file = square % 8;
        let mut moves = 0u64;
        let deltas = [
            (1,0),
            (1,1),
            (1,-1),
            (0,1),
            (0,-1),
            (-1,0),
            (-1,1),
            (-1,-1),
        ];
        for (dr, df) in deltas {
            let new_rank = rank as i32 + dr;
            let new_file = file as i32 + df;

            if new_rank >= 0 && new_rank < 8 && new_file >= 0 && new_file < 8 {
                let destination = (new_rank * 8 + new_file) as u64;
                moves |= 1u64 << destination;
            }
        }
        king_moves[square] = moves;
    }
    king_moves
}