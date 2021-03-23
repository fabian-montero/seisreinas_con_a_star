pub type U36 = u64;

#[derive(Clone, Copy, Debug)]
pub struct Board(U36);

impl Board {
    pub fn new(bits: U36) -> Self {
        assert!(bits == bits & ((1 << 36) - 1),
            "ERROR: Bad input: board is bigger than 6x6."
        );
        Board(bits)
    }

    pub fn count_queens(&self) -> u32 {
        let Board(board) = &self;
        board.count_ones()
    }
}
