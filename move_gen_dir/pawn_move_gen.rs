use crate::Board;
use crate::helpers::pop_lsb;
use crate::move_gen_dir::move_gen::{convert_bitboard_to_moves, GenerationMode, Move, PieceType, PinMask};
use crate::move_gen_dir::move_gen::Castling::NoCastle;
use crate::move_gen_dir::move_gen::PieceType::NoPiece;
use crate::move_list::MoveList;

const PROMOTION_OPTIONS: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];
const fn shift_left(position: u64, shift: u32) ->u64 {
    position.wrapping_shl(shift)
}
const fn shift_right(position: u64, shift: u32) ->u64 {
    position.wrapping_shr(shift)
}
pub fn generate_pawn_moves(board: &Board, move_list: &mut MoveList, checkmask: &u64, pin_mask: &PinMask, generation_mode: &GenerationMode) {
    let (pawns, opponent, push_fn, double_push_fn, left_capture_fn, right_capture_fn, first_rank, promotion_rank, shift_back) = pawn_info(board);

    let mut remaining = pawns;
    while remaining != 0 {
        let from_sq = pop_lsb(&mut remaining);
        let pawn = 1u64 << from_sq;

        match (pawn & pin_mask.horizontal).count_ones() {
            1 => continue,
            _ => {}
        }
        let pawn_pinmask: u64 = if (pawn & (pin_mask.vertical | pin_mask.diagonal)).count_ones() == 1 {
            let pinmask_horizontal = !(!pin_mask.vertical * ((pin_mask.vertical & pawn).count_ones() as u64));
            let pinmask_vertical = !(!pin_mask.diagonal * ((pin_mask.diagonal & pawn).count_ones() as u64));
            pinmask_vertical & pinmask_horizontal
        } else { 0xffffffffffffffff };


        match generation_mode {
            GenerationMode::All => {
                generate_pawn_silent(board, move_list, checkmask, push_fn, double_push_fn, first_rank, promotion_rank, pawn, pawn_pinmask);
                generate_pawn_captures(board, move_list, checkmask, opponent, left_capture_fn, right_capture_fn, promotion_rank, shift_back, pawn, pawn_pinmask);
            },
            GenerationMode::Capture => {
                generate_pawn_captures(board, move_list, checkmask, opponent, left_capture_fn, right_capture_fn, promotion_rank, shift_back, pawn, pawn_pinmask);
            },
            GenerationMode::Check => {}
        }
    }
}

fn generate_pawn_silent(board: &Board, move_list: &mut MoveList, checkmask: &u64, push_fn: fn(u64) -> u64, double_push_fn: fn(u64) -> u64, first_rank: u64, promotion_rank: u64, pawn: u64, pawn_pinmask: u64) {
    // Quiet moves
    let single_push = push_fn(pawn) & !board.occ;

    let double_push = double_push_fn(pawn & first_rank)
        & !board.occ
        & push_fn(single_push);


    let mut quiet_moves = (single_push | double_push) & checkmask & pawn_pinmask;


    let promotions = quiet_moves & promotion_rank;
    quiet_moves &= !promotion_rank;

    convert_bitboard_to_moves(board, move_list, pawn, quiet_moves, PieceType::Pawn, NoPiece);

    for promo in PROMOTION_OPTIONS {
        convert_bitboard_to_moves(board, move_list, pawn, promotions, PieceType::Pawn, promo);
    }
}

fn generate_pawn_captures(board: &Board, move_list: &mut MoveList, checkmask: &u64, opponent: u64, left_capture_fn: fn(u64) -> u64, right_capture_fn: fn(u64) -> u64, promotion_rank: u64, shift_back: fn(u64) -> u64, pawn: u64, pawn_pinmask: u64) {
    // Captures
    let capture_left = left_capture_fn(pawn) & opponent;
    let capture_right = right_capture_fn(pawn) & opponent;


    let mut captures = (capture_left | capture_right) & checkmask & pawn_pinmask;

    let promo_caps = captures & promotion_rank;
    captures &= !promotion_rank;

    convert_bitboard_to_moves(board, move_list, pawn, captures, PieceType::Pawn, NoPiece);

    for promo in PROMOTION_OPTIONS {
        convert_bitboard_to_moves(board, move_list, pawn, promo_caps, PieceType::Pawn, promo);
    }

    let ep_target = board.last_double_pawn_push;
    let ep_pawn_target = shift_back(ep_target);

    let left_ep = left_capture_fn(pawn) & ep_pawn_target;
    let right_ep = right_capture_fn(pawn) & ep_pawn_target;

    let ep_check = if checkmask & board.last_double_pawn_push != 0 {
        ep_pawn_target
    } else { 0 };

    let mut ep_captures = (left_ep | right_ep) & (checkmask | ep_check) & pawn_pinmask;

    while ep_captures != 0 {
        let to_sq = pop_lsb(&mut ep_captures);
        move_list.add_move(Move {
            start_square: pawn,
            end_square: 1 << to_sq,
            capture: PieceType::Pawn,
            piece_type: PieceType::Pawn,
            promotion: NoPiece,
            castle: NoCastle,
            en_passant: true,
        });
    }
}

const fn pawn_info(board: &Board) -> (u64, u64, fn(u64) -> u64, fn(u64) -> u64, fn(u64) -> u64, fn(u64) -> u64, u64, u64, fn(u64) -> u64) {
    if board.white_to_move {
        (
            board.wpawn,
            board.black,
            |x: u64| shift_left(x, 8),
            |x: u64| shift_left(x, 16),
            |x: u64| shift_left(x & 0xfefefefefefefefe, 7), // capture left (A file masked)
            |x: u64| shift_left(x & 0x7f7f7f7f7f7f7f7f, 9), // capture right (H file masked)
            0x000000000000FF00,
            0xFF00000000000000,
            |x: u64| shift_left(x, 8),
        )
    } else {
        (
            board.bpawn,
            board.white,
            |x: u64| shift_right(x, 8),
            |x: u64| shift_right(x, 16),
            |x: u64| shift_right(x & 0x7f7f7f7f7f7f7f7f, 7), // capture left (H file masked, mirrored)
            |x: u64| shift_right(x & 0xfefefefefefefefe, 9), // capture right (A file masked, mirrored)
            0x00FF000000000000,
            0x00000000000000FF,
            |x: u64| shift_right(x, 8),
        )
    }
}
