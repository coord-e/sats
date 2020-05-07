use std::fmt;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub struct Level(usize);

impl Level {
    pub fn initial() -> Level {
        Level(0)
    }

    pub fn next(self) -> Level {
        Level(self.0 + 1)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lv.{}", self.0)
    }
}
