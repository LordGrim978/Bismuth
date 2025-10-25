use crate::Board;


pub fn start_pos() -> Board {
    return make_board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}
pub fn make_board(fen_string: &str) -> Board {
    let binding = fen_string.replace("/", " ");
    let split_string = binding.split_whitespace(); // More idiomatic

    let mut white_to_move = false;
    let mut castling_rights: u8 = 0;
    let mut en_passant:u64 = 0;

    let mut piece_array: [u64; 12] = [0; 12]; // White first: pawn, rook, knight, bishop, queen, king

    let mut rank = 8; // Start from rank 8

    for part in split_string {

        if rank == 0 {
            if part == "w" {
                white_to_move = true
            } else if part.contains("k") | part.contains("K") | part.contains("q") | part.contains("Q") {
                castling_rights = extraxt_castling_rights(part);
            } else if part.chars().any(|c| c >= 'a' && c <= 'h') {
                en_passant = match part {
                    "a3" => {0x1000000},
                    "b3" => {0x2000000},
                    "c3" => {0x4000000},
                    "d3" => {0x8000000},
                    "e3" => {0x10000000},
                    "f3" => {0x20000000},
                    "g3" => {0x40000000},
                    "h3" => {0x80000000},
                    "a6" => {0x100000000},
                    "b6" => {0x200000000},
                    "c6" => {0x400000000},
                    "d6" => {0x800000000},
                    "e6" => {0x1000000000},
                    "f6" => {0x2000000000},
                    "g6" => {0x4000000000},
                    "h6" => {0x8000000000},
                    _ => {0}
                }
            }
        } else {
            let mut file = 0;

            for piece_char in part.chars() {
                if piece_char.is_digit(10) {
                    file += piece_char.to_digit(10).unwrap() as usize;
                } else {
                    let white_offset = if piece_char.is_ascii_uppercase() { 6 } else { 0 };
                    let piece_index = get_piece(piece_char) + white_offset;
                    let bit_index = (rank-1) * 8 + file;
                    piece_array[piece_index] |= 1 << bit_index;
                    file += 1;
                }
            }
            rank -= 1;
        }
    }

    return Board::new(piece_array[0], piece_array[1], piece_array[2], piece_array[3], piece_array[4], piece_array[5],
                      piece_array[6], piece_array[7], piece_array[8], piece_array[9], piece_array[10], piece_array[11],
                      white_to_move,
                      castling_rights,
                      en_passant
    );
}

fn extraxt_castling_rights(part: &str) -> u8 {
    let mut castling_rights: u8 = 0;
    if part.contains("K") {
        castling_rights |= 1;
    }
    if part.contains("Q") {
        castling_rights |= 2;
    }
    if part.contains("k") {
        castling_rights |= 4;
    }
    if part.contains("q") {
        castling_rights |= 8;
    }
    return castling_rights;
}

fn get_piece(piece_char: char) -> usize {
    match piece_char.to_ascii_lowercase() {
        'p' => 0,
        'n' => 1,
        'b' => 2,
        'r' => 3,
        'q' => 4,
        'k' => 5,
        _ => panic!("Invalid piece character: {}", piece_char),
    }
}
