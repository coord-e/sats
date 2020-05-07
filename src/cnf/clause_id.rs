use std::fmt;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ClauseID(usize);

impl ClauseID {
    pub(self) fn new(n: usize) -> ClauseID {
        ClauseID(n)
    }

    pub(self) fn next(self) -> ClauseID {
        ClauseID(self.0 + 1)
    }
}

impl fmt::Display for ClauseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&format!("c{}", self.0))
    }
}

#[derive(Debug, Clone)]
pub struct ClauseIDGenerator {
    next_id: ClauseID,
}

impl ClauseIDGenerator {
    pub fn new() -> Self {
        ClauseIDGenerator {
            next_id: ClauseID::new(0),
        }
    }

    pub fn next(&mut self) -> ClauseID {
        let id = self.next_id;
        self.next_id = id.next();
        id
    }
}
