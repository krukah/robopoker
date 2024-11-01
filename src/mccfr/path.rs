#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct Path(u64);

impl From<(usize, usize)> for Path {
    fn from((depth, raise): (usize, usize)) -> Self {
        Path((depth | raise << 32) as u64)
    }
}

impl From<u64> for Path {
    fn from(value: u64) -> Self {
        Path(value)
    }
}

impl From<Path> for u64 {
    fn from(path: Path) -> Self {
        path.0
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let depth = (self.0 & 0xFFFFFFFF) as usize;
        let raise = (self.0 >> 32) as usize;
        write!(f, "H{:02}", depth * 10 + raise)
    }
}
