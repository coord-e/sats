use std::collections::hash_map;
use std::collections::HashMap;
use std::{fmt, ops};

use crate::cnf::{Literal, Variable};

use itertools::Itertools;

#[derive(Clone, Copy)]
pub enum Truth {
    True,
    False,
}

impl fmt::Display for Truth {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Truth::True => f.pad("True"),
            Truth::False => f.pad("False"),
        }
    }
}

impl ops::Not for Truth {
    type Output = Truth;
    fn not(self) -> Truth {
        match self {
            Truth::True => Truth::False,
            Truth::False => Truth::True,
        }
    }
}

#[derive(Clone)]
pub struct Assignment(HashMap<Variable, Truth>);

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{ ")?;
        f.write_str(
            &self
                .0
                .iter()
                .map(|(var, truth)| format!("{} := {}", var, truth))
                .join(", "),
        )?;
        f.write_str(" }")?;

        Ok(())
    }
}

impl Default for Assignment {
    fn default() -> Assignment {
        Assignment::new()
    }
}

impl IntoIterator for Assignment {
    type Item = (Variable, Truth);
    type IntoIter = hash_map::IntoIter<Variable, Truth>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<(Variable, Truth)> for Assignment {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (Variable, Truth)>,
    {
        self.0.extend(iter);
    }
}

impl Assignment {
    pub fn new() -> Assignment {
        Assignment(HashMap::new())
    }

    pub fn assign(&mut self, var: &Variable, truth: Truth) {
        self.0.insert(var.clone(), truth);
    }

    pub fn assign_true(&mut self, literal: &Literal) {
        if literal.is_negated() {
            self.assign(literal.variable(), Truth::False);
        } else {
            self.assign(literal.variable(), Truth::True);
        }
    }

    pub fn assigned_true(mut self, literal: &Literal) -> Assignment {
        self.assign_true(literal);
        self
    }

    pub fn get(&self, var: &Variable) -> Option<Truth> {
        self.0.get(var).copied()
    }
}
