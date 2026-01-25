use std::ops::{Add, Sub};

/// Byte offset in the storage file
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct ByteOffset(pub u64);

impl Add<u64> for ByteOffset {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        ByteOffset(self.0 + rhs)
    }
}

impl Sub<ByteOffset> for ByteOffset {
    type Output = u64;
    fn sub(self, rhs: ByteOffset) -> u64 {
        self.0 - rhs.0
    }
}

/// Line index in the log
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct LineIndex(pub usize);

/// Range of bytes representing a line
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineRange {
    pub start: ByteOffset,
    pub end: ByteOffset,
}
