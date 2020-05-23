use crate::cnf::{Clause, Literal, Variable, CNF};
use crate::expr::Expr;

use either::{Left, Right};

#[derive(Clone)]
pub enum Simple {
    Lit(Literal),
    And(Literal, Literal),
    Or(Literal, Literal),
}

impl Simple {
    fn negated(self) -> Simple {
        match self {
            Simple::Lit(l) => Simple::Lit(l.negated()),
            Simple::And(l1, l2) => Simple::Or(l1.negated(), l2.negated()),
            Simple::Or(l1, l2) => Simple::And(l1.negated(), l2.negated()),
        }
    }

    fn into_clauses(self) -> impl Iterator<Item = Clause> {
        use std::iter;

        match self {
            Simple::Lit(l) => {
                let c = Clause::from_literals(iter::once(l));
                Left(iter::once(c))
            }
            Simple::And(l1, l2) => {
                let c1 = Clause::from_literals(iter::once(l1));
                let c2 = Clause::from_literals(iter::once(l2));
                Right(iter::once(c1).chain(iter::once(c2)))
            }
            Simple::Or(l1, l2) => {
                let c = Clause::from_literals(iter::once(l1).chain(iter::once(l2)));
                Left(iter::once(c))
            }
        }
    }
}

struct Convert {
    clauses: Vec<Clause>,
    unique: usize,
}

impl Convert {
    fn new() -> Convert {
        Convert {
            clauses: Vec::new(),
            unique: 0,
        }
    }

    fn finalize(self, s: Simple) -> CNF {
        CNF::from_clauses(self.clauses.into_iter().chain(s.into_clauses()))
    }

    fn simplify(&mut self, expr: Expr) -> Simple {
        match expr {
            Expr::Var(v) => Simple::Lit(v.into()),
            Expr::Not(box e) => {
                let v = self.subexpr_substitution(e);
                Simple::Lit(Literal::new(v, true))
            }
            Expr::And(box e1, box e2) => {
                let v1 = self.subexpr_substitution(e1);
                let v2 = self.subexpr_substitution(e2);
                Simple::And(v1.into(), v2.into())
            }
            Expr::Or(box e1, box e2) => {
                let v1 = self.subexpr_substitution(e1);
                let v2 = self.subexpr_substitution(e2);
                Simple::Or(v1.into(), v2.into())
            }
        }
    }

    fn subexpr_substitution(&mut self, e: Expr) -> Variable {
        if let Expr::Var(v) = e {
            return v;
        }

        let s = self.simplify(e);
        let v = self.fresh();
        self.substitute(&v, s);
        v
    }

    fn substitute(&mut self, v: &Variable, s: Simple) {
        // ¬s ∨ v
        self.introduce_or(v.clone().into(), s.clone().negated());
        // ¬v ∨ s
        self.introduce_or(Literal::new(v.clone(), true), s);
    }

    /// introduce `l1 ∨ s`
    fn introduce_or(&mut self, l1: Literal, s: Simple) {
        match s {
            Simple::Lit(l2) => {
                // l1 ∨ l2
                let c = Clause::from_literals(vec![l1, l2]);
                self.clauses.push(c);
            }
            Simple::And(l2, l3) => {
                // l1 ∨ (l2 ∧ l3)
                // → (l1 ∨ l2) ∧ (l1 ∨ l3)
                let c1 = Clause::from_literals(vec![l1.clone(), l2]);
                let c2 = Clause::from_literals(vec![l1, l3]);
                self.clauses.push(c1);
                self.clauses.push(c2);
            }
            Simple::Or(l2, l3) => {
                // l1 ∨ l2 ∨ l3
                let c = Clause::from_literals(vec![l1, l2, l3]);
                self.clauses.push(c);
            }
        }
    }

    fn fresh(&mut self) -> Variable {
        let id = self.unique;
        self.unique += 1;
        Variable::fresh(id)
    }
}

pub fn to_cnf(e: Expr) -> CNF {
    let mut conv = Convert::new();
    let root = conv.simplify(e);
    conv.finalize(root)
}
