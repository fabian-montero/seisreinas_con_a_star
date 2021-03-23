use seisreinas2::{Graph, Board, Edge};
use std::collections::BTreeSet;

fn main() {
    //println!("{:?}", Graph::valid_boards_from_empty().0.len());
    //println!("\n\n\n\n\n");
    //(0..36).filter(|&n| !Board(1).has_vision(n)).for_each(|n| println!("{}", n))

    let graph = Graph::valid_boards_from_empty();
    println!("{:?}", graph.0.iter().map(|Edge { from, to }| to).filter(|b| b.count_queens() == 6).collect::<BTreeSet<_>>());
    
}
