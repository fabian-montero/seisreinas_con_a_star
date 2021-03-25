#![feature(binary_heap_retain, bool_to_option)]

use itertools::Itertools;
use std::{
    cmp,
    collections::{BTreeSet, BinaryHeap, HashSet},
    fmt,
};

pub type U36 = u64;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Board(U36);

impl Board {
    pub const EMPTY: Self = Board(0);

    pub fn count_queens(self) -> u32 {
        let Board(bits) = self;
        bits.count_ones()
    }

    pub fn penalty(self) -> u32 {
        (self.0 ^ (self.0 >> 1)).count_zeros()
    }

    pub fn bits(self) -> [bool; 36] {
        let mut bits = [false; 36];
        for i in 0..36 {
            bits[i] = self.0 & (1 << (36 - i - 1)) != 0;
        }

        bits
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

    fn place_queen(&self, pos: U36) -> Self {
        let Board(bits) = &self;
        Board(bits | 1 << pos)
    }

    fn has_queen_at(&self, pos: U36) -> bool {
        let Board(bits) = &self;
        (bits & (1 << pos)) != 0b0
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:036b}", self.0)
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Board(board) = self;
        write!(f, "{}", board)
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
pub struct Node {
    board: Board,
    parent: Option<Board>,
    g: u32,
    h: u32,
}

impl Node {
    pub fn board(&self) -> Board {
        self.board
    }

    pub fn f(&self) -> u32 {
        self.g + self.h
    }

    pub fn g(&self) -> u32 {
        self.g
    }

    pub fn h(&self) -> u32 {
        self.h
    }
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
    pub fn current(&self) -> Board {
        self.current
    }

    pub fn clone_open_sorted(&self) -> Vec<Node> {
        let mut open = self.open.clone().into_sorted_vec();
        open.reverse();
        open
    }

    pub fn closed(&self) -> impl Iterator<Item = &Node> {
        self.closed.iter()
    }

    pub fn traceback(&self, to: Board) -> (u32, Vec<Board>) {
        let node_for = |board| {
            let is_target = |node: &&Node| node.board == board;

            self.closed
                .iter()
                .find(is_target)
                .or_else(|| self.open.iter().find(is_target))
        };

        let mut path = std::iter::successors(Some(to), |&board| {
            node_for(board).and_then(|node| node.parent)
        })
        .collect::<Vec<_>>();

        path.reverse();
        (node_for(to).unwrap().g, path)
    }
}

impl fmt::Debug for Step<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Current board: {}\nOpen set: {:?}\nClosed set: {:?}\n",
            self.current, self.open, self.closed
        )
    }
}

pub fn a_star<'a>(graph: &'a Graph) -> impl 'a + Iterator<Item = Step> {
    let do_step = move |step: &Step<'a>| {
        (step.current.count_queens() < 6).then_some(())?;
        let mut step = step.clone();

        let curr = step.open.pop().unwrap();
        step.current = curr.board;

        step.closed.insert(curr);
        for neighbor_board in step.graph.reachable_from(curr.board) {
            let cost = curr.g + 36;

            let (mut in_open, mut in_closed, mut neighbor) = {
                let is_neighbor = |node: &&Node| node.board == neighbor_board;
                step.open
                    .iter()
                    .find(is_neighbor)
                    .map(|&node| (true, false, node))
                    .or_else(|| {
                        step.closed
                            .iter()
                            .find(is_neighbor)
                            .map(|&node| (false, true, node))
                    })
                    .unwrap_or_else(|| {
                        (
                            false,
                            false,
                            Node {
                                board: neighbor_board,
                                parent: Some(curr.board),
                                g: cost,
                                h: neighbor_board.penalty() * (6 - neighbor_board.count_queens()),
                            },
                        )
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

    let mut open = BinaryHeap::new();
    open.push(Node {
        parent: None,
        board: Board::EMPTY,
        g: 0,
        h: 36 * (6 - 0),
    });

    let initial = Step {
        current: Board::EMPTY,
        graph,
        open,
        closed: HashSet::new(),
    };

    std::iter::successors(Some(initial), do_step).fuse()
}
