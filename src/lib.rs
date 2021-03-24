#![feature(binary_heap_retain, bool_to_option)]

use itertools::Itertools;
use std::{cmp, collections::{BTreeSet, BinaryHeap, HashSet}, fmt};

pub type U36 = u64;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Board(pub U36);

impl Board {
    pub fn count_queens(&self) -> u32 {
        let Board(bits) = &self;
        bits.count_ones()
    }

    fn place_queen(&self, pos: U36) -> Self {
        let Board(bits) = &self;
        Board(bits | 1 << pos)
    }

    fn has_queen_at(&self, pos: U36) -> bool {
        let Board(bits) = &self;
        (bits & 1 << pos) != 0b0
    }

    pub fn has_vision(&self, from: U36) -> bool {
        let Board(bits) = *self;
        let (row, col) = (from / 6, from % 6);

        let test = |r: U36, c: U36| bits & (1 << (6 * r + c)) != 0;

        (0..6).filter(|&r| r != row).any(|r| test(r, col))
            || (0..6).filter(|&c| c != col).any(|c| test(row, c))
            || (-5..6)
                .zip(-5..6)
                .cartesian_product([-1, 1].iter())
                .map(|((a, b), &s)| (a, b * s))
                .filter(|&(p, q)| {
                    p != 0
                        && q != 0
                        && (0..6).contains(&(row as i64 + p))
                        && (0..6).contains(&(col as i64 + q))
                })
                .any(|(p, q)| test((row as i64 + p) as U36, (col as i64 + q) as U36))
    }

    pub const EMPTY: Self = Board(0);
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:036b}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Edge {
    pub from: Board,
    pub to: Board,
}

#[derive(Debug)]
pub struct Graph(BTreeSet<Edge>);

impl Graph {
    pub fn valid_boards_from_empty() -> Self {
        let mut graph = Graph(BTreeSet::new());
        graph.insert_valid_boards(Board::EMPTY);
        graph
    }

    fn insert_valid_boards(&mut self, parent: Board) {
        (parent.count_queens() < 6)
            .then(move || {
                (0..36)
                    .filter(|&n| !parent.has_queen_at(n))
                    .map(|n| (n, parent.place_queen(n)))
                    .filter(|&(n, b)| !b.has_vision(n))
                    .for_each(move |(_, b)| {
                        self.0.insert(Edge {
                            from: parent,
                            to: b,
                        });

                        self.0.insert(Edge {
                            from: b,
                            to: parent,
                        });

                        self.insert_valid_boards(b);
                    });
            })
            .unwrap_or_default();
    }

    pub fn reachable_from(&self, from: Board) -> impl '_ + Iterator<Item = Board> {
        self.0
            .range(
                Edge {
                    from,
                    to: Board::EMPTY,
                }..Edge {
                    from: Board(from.0 + 1),
                    to: Board::EMPTY,
                },
            )
            .map(|&Edge { to, .. }| to)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct Node {
    board: Board,
    parent: Option<Board>,
    g: u32,
    h: u32,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        cmp::Reverse(self.g + self.h).cmp(&cmp::Reverse(other.g + other.h))
    }
}

#[derive(Clone)]
pub struct Step<'a> {
    current: Board,
    graph: &'a Graph,
    open: BinaryHeap<Node>,
    closed: HashSet<Node>,
}

impl Step<'_> {
    pub fn traceback(&self) -> Vec<Board> {
        let mut path: Vec<_> = std::iter::successors(
            Some(self.current),
            |&board| {
                let is_target = |node: &&Node| node.board == board;
                self.closed.iter().find(is_target)
                    .or_else(|| self.open.iter().find(is_target))
                    .and_then(|node| node.parent)
            }
        ).collect();

        path.reverse();
        path
    }
}

impl fmt::Debug for Step<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{:?}]", self.current)
    }
}

pub fn a_star<'a>(graph: &'a Graph) -> impl 'a + Iterator<Item = Step> {
    let mut open = BinaryHeap::new();
    open.push(Node {
        parent: None,
        board: Board::EMPTY,
        g: 0,
        h: 6,
    });

    let initial_state = Some(Step {
        current: Board::EMPTY,
        graph,
        open,
        closed: HashSet::new(),
    });

    let do_step = move |step: &Step<'a>| {
        (step.current.count_queens() < 6).then_some(())?;
        let mut step = step.clone();

        let curr = step.open.pop().unwrap();
        step.current = curr.board;

        step.closed.insert(curr);
        for neighbor_board in step.graph.reachable_from(curr.board) {
            let cost = curr.g + 1;

            let (mut in_open, mut in_closed, mut neighbor) = {
                let is_neighbor = |node: &&Node| node.board == neighbor_board;
                step.open.iter().find(is_neighbor)
                    .map(|&node| (true, false, node))
                    .or_else(|| {
                        step.closed.iter().find(is_neighbor)
                            .map(|&node| (false, true, node))
                    })
                .unwrap_or_else(|| {
                    (false, false, Node {
                        board: neighbor_board,
                        parent: Some(curr.board),
                        g: cost,
                        h: 6 - neighbor_board.count_queens(),
                    })
                })
            };

            if cost < neighbor.g {
                let not_neighbor = |node: &Node| node.board != neighbor_board;
                if in_open {
                    step.open.retain(not_neighbor);
                    in_open = false;
                } else if in_closed {
                    step.closed.retain(not_neighbor);
                    in_closed = false;
                }
            }

            if !in_open && !in_closed {
                neighbor.g = cost;
                neighbor.parent = Some(curr.board);

                step.open.push(neighbor);
            }
        }

        Some(step)
    };

    std::iter::successors(initial_state, do_step)
}
