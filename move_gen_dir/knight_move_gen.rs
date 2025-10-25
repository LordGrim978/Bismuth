use crate::Board;
use crate::helpers::pop_lsb;
use crate::move_gen_dir::move_gen::{convert_bitboard_to_moves, GenerationMode, PieceType, PinMask};
use crate::move_list::MoveList;


pub const KNIGHT_MOVES: [u64; 64] = [132096,329728,659712,1319424,2638848,5277696,10489856,4202496,33816580,84410376,168886289,337772578,675545156,1351090312,2685403152,1075839008,8657044482,21609056261,43234889994,86469779988,172939559976,345879119952,687463207072,275414786112,2216203387392,5531918402816,11068131838464,22136263676928,44272527353856,88545054707712,175990581010432,70506185244672,567348067172352,1416171111120896,2833441750646784,5666883501293568,11333767002587136,22667534005174272,45053588738670592,18049583422636032,145241105196122112,362539804446949376,725361088165576704,1450722176331153408,2901444352662306816,5802888705324613632,11533718717099671552,4620693356194824192,288234782788157440,576469569871282176,1224997833292120064,2449995666584240128,4899991333168480256,9799982666336960512,1152939783987658752,2305878468463689728,1128098930098176,2257297371824128,4796069720358912,9592139440717824,19184278881435648,38368557762871296,4679521487814656,9077567998918656];
pub fn generate_knight_moves(board: &Board, move_list: &mut MoveList, checkmask: &u64, pin_mask: &PinMask, generation_mode: &GenerationMode) {
    let mut knights = if board.white_to_move { board.wknight } else { board.bknight };

    let opponent_pieces = if board.white_to_move { board.black } else { board.white };

    while knights != 0 {
        let knight = pop_lsb(&mut knights);

        let pinned = (1 << knight) & (pin_mask.horizontal | pin_mask.vertical | pin_mask.diagonal);

        if pinned != 0 {
            continue;
        }

        let possible_moves = KNIGHT_MOVES[knight as usize] & checkmask;

        match generation_mode {
            GenerationMode::All => {
                generate_knight_silent(board, move_list, knight, possible_moves);
                generate_knight_capture(board, move_list, opponent_pieces, knight, possible_moves);
            },
            GenerationMode::Capture => {
                generate_knight_capture(board, move_list, opponent_pieces, knight, possible_moves);
            },
            GenerationMode::Check => {}
        }
    }
}

fn generate_knight_silent(board: &Board, move_list: &mut MoveList, knight: u64, possible_moves: u64) {
    let no_capture_moves = possible_moves & !board.white & !board.black;

    convert_bitboard_to_moves(board, move_list, 1 << knight, no_capture_moves, PieceType::Knight, PieceType::NoPiece);
}

fn generate_knight_capture(board: &Board, move_list: &mut MoveList, opponent_pieces: u64, knight: u64, possible_moves: u64) {
    let capture_moves = possible_moves & opponent_pieces;

    convert_bitboard_to_moves(board, move_list, 1 << knight, capture_moves, PieceType::Knight, PieceType::NoPiece);
}