use std::{error, fmt, str};

use crate::assignment::Assignment;
use crate::cnf::CNF;

pub mod cdcl;
pub mod dpll;

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Solver {
    DPLL,
    CDCL,
}

impl Solver {
    pub fn all() -> impl Iterator<Item = Solver> {
        vec![Solver::DPLL, Solver::CDCL].into_iter()
    }

    pub fn run(&self, cnf: CNF) -> Option<Assignment> {
        match self {
            Solver::DPLL => dpll::solve(cnf),
            Solver::CDCL => cdcl::solve(cnf),
        }
    }
}

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Solver::DPLL => f.pad("DPLL"),
            Solver::CDCL => f.pad("CDCL"),
        }
    }
}

#[derive(Debug)]
pub enum ParseSolverError {
    UnknownSolver(String),
}

impl fmt::Display for ParseSolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseSolverError::UnknownSolver(s) => write!(f, "unknown solver: {}", s),
        }
    }
}

impl error::Error for ParseSolverError {}

impl str::FromStr for Solver {
    type Err = ParseSolverError;
    fn from_str(s: &str) -> Result<Solver, Self::Err> {
        match s.to_lowercase().as_str() {
            "cdcl" => Ok(Solver::CDCL),
            "dpll" => Ok(Solver::DPLL),
            _ => Err(ParseSolverError::UnknownSolver(s.to_owned())),
        }
    }
}
