use crate::assignment::Assignment;
use crate::cnf::CNF;

pub mod dpll;
pub use dpll::DPLL;

pub trait Solver {
    fn solve(&mut self, cnf: CNF) -> Option<Assignment>;
}

pub fn solve<T>(cnf: CNF) -> Option<Assignment>
where
    T: Solver + Default,
{
    let mut solver = T::default();
    solver.solve(cnf)
}
