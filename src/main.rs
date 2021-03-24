#![feature(bool_to_option)]

use seisreinas2::{Graph, a_star};

fn main() {
    let graph = Graph::valid_boards_from_empty();

    //SIRVE
    //println!("{:?}", graph.0.iter().map(|Edge { from, to }| to).filter(|b| b.count_queens() == 6).collect::<BTreeSet<_>>());

    //let n = 0b000010001000100000000000000100010000;
    //println!(
    //    "{:?}",
    //    graph
    //        .0
    //        .range(
    //            Edge {
    //                from: Board(n),
    //                to: Board::EMPTY
    //            }..Edge {
    //                from: Board(n + 1),
    //                to: Board::EMPTY
    //            }
    //        )
    //        .filter_map(|&Edge { from, to }| (to.count_queens() == 6).then_some(to))
    //        .collect::<Vec<_>>()
    //)

    let mut steps = a_star(&graph).enumerate().peekable();
    while let Some((i, step)) = steps.next() {
        println!("[{:03}] {:?}", 1 + i, step);
        if steps.peek().is_none() {
            println!("Path: {:?}", step.traceback());
        }
    }
}
