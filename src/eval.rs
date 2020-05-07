use crate::assignment::{Assignment, Truth};
use crate::cnf::{Clause, Literal, CNF};

pub fn eval(cnf: &CNF, assignment: &Assignment) -> Truth {
    for c in cnf.clauses() {
        match eval_clause(c, assignment) {
            Truth::True => continue,
            Truth::False => return Truth::False,
        }
    }

    Truth::True
}

pub fn eval_clause(clause: &Clause, assignment: &Assignment) -> Truth {
    for l in clause.literals() {
        match eval_literal(l, assignment) {
            Truth::True => return Truth::True,
            Truth::False => continue,
        }
    }

    Truth::False
}

pub fn eval_literal(literal: &Literal, assignment: &Assignment) -> Truth {
    let a = assignment.get(literal.variable()).unwrap_or(Truth::True);
    if literal.is_negated() {
        !a
    } else {
        a
    }
}
