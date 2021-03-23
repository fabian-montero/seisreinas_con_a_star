use itertools::Itertools;
use std::collections::BTreeSet;
use std::fmt;

pub type U36 = u64;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Board(pub U36);

impl Board {
    fn new(bits: U36) -> Self {
        assert_eq!(
            bits,
            bits & ((1 << 36) - 1),
            "ERROR: Bad input: board is bigger than 6x6."
        );
        Board(bits)
    }

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
           .filter(|&(p, q)| p != 0 && q != 0
                          && (0..6).contains(&(row as i64 + p))
                          && (0..6).contains(&(col as i64 + q)))
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
pub struct Graph(pub BTreeSet<Edge>);

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
}
