use crate::{generate_all_moves, Board};
use crate::fen_import::make_board;
use crate::helpers::index_to_sq;
use crate::move_gen_dir::move_gen::GenerationMode;

#[derive()]
struct TestPosition {
    fen: String,
    depth: usize,
    result: i64,
}



pub fn _test_move_gen() {
    let test_positions: [TestPosition;5] = [
        TestPosition {fen: ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq").parse().unwrap(),depth:5,result:4865609},
        TestPosition {fen: ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq").parse().unwrap(),depth:5,result:193690690},
        TestPosition {fen: ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w -").parse().unwrap(),depth:7,result:178633661},
        TestPosition {fen: ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq").parse().unwrap(),depth:6,result:706045033},
        TestPosition {fen: ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ").parse().unwrap(),depth:5,result:89941194},
    ];

    let mut sum_postions = 0;
    for position in test_positions.iter() {
        let mut board = make_board(&position.fen);
        let found_moves = _move_generation_test(position.depth as i32, &mut board, false);
        println!("Found Moves after {}: {}", position.depth, found_moves);
        println!("Actual Moves: {}", position.result);
        println!("Difference: {}", (found_moves- position.result));
        println!();
        sum_postions += found_moves;
    }
    println!("Final Sum: {}", sum_postions);
}

pub fn _move_generation_test(depth: i32, board: &mut Board, first_iteration: bool)-> i64 {
    let mut positons: i64 = 0;

    let move_list = generate_all_moves(board, &GenerationMode::All);

    if depth == 1 {
        return move_list.moves_added as i64;
    }

    for moves in 0..move_list.moves_added {
        let last_mv_info = board.make_move(move_list.moves[moves]);

        let positons_to_add = _move_generation_test(depth-1, board, false);

        if first_iteration {
            println!("{:?}{:?}: {:?}",
                     index_to_sq(move_list.moves[moves].start_square.trailing_zeros() as usize),
                     index_to_sq(move_list.moves[moves].end_square.trailing_zeros() as usize),
                     positons_to_add,
                     // move_list.moves[moves].piece_type,
                     );
        }

        positons += positons_to_add;

        board.undo_move(last_mv_info);
    }

    positons
}