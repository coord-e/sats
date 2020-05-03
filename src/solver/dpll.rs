use std::collections::HashSet;

use crate::assignment::Assignment;
use crate::cnf::{Literal, CNF};
use crate::solver::Solver;

pub struct DPLL;

fn dpll(mut cnf: CNF) -> Option<Assignment> {
    let mut assignment = Assignment::new();

    if cnf.is_empty() {
        return Some(assignment);
    }

    if cnf.clauses().any(|c| c.is_empty()) {
        return None;
    }

    if cnf.impure_literals().next().is_none() {
        for l in cnf.literals() {
            assignment.assign(l, l.truth());
        }
        return Some(assignment);
    }

    let unit_clauses: Vec<_> = cnf.unit_clauses().cloned().collect();
    for l in unit_clauses {
        assignment.assign(&l, l.truth());
        cnf.simplify_true_literal(&l);
    }

    let literals: HashSet<_> = cnf.literals().cloned().collect();
    let impure_literals: HashSet<_> = cnf.impure_literals().collect();
    for l in literals.difference(&impure_literals) {
        assignment.assign(&l, l.truth());
        cnf.simplify_true_literal(&l);
    }

    let result = if let Some(l) = choose_literal(&cnf) {
        branch(cnf, &l)
    } else {
        dpll(cnf)
    };

    if let Some(a) = result {
        assignment.extend(a);
        Some(assignment)
    } else {
        None
    }
}

fn choose_literal(cnf: &CNF) -> Option<Literal> {
    cnf.literals().next().cloned()
}

fn branch(mut cnf: CNF, l: &Literal) -> Option<Assignment> {
    let mut pos = cnf.clone();
    pos.simplify_true_literal(l);
    dpll(pos)
        .map(|a| a.assigned(l.variable(), l.truth()))
        .or_else(|| {
            cnf.simplify_true_literal(&l.negated());
            dpll(cnf).map(|a| a.assigned(l.variable(), !l.truth()))
        })
}

impl Default for DPLL {
    fn default() -> DPLL {
        DPLL
    }
}

impl Solver for DPLL {
    fn solve(&mut self, cnf: CNF) -> Option<Assignment> {
        dpll(cnf)
    }
}
