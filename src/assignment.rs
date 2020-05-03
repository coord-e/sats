use std::collections::hash_map;
use std::collections::HashMap;
use std::fmt;

use crate::cnf::Variable;

use itertools::Itertools;

#[derive(Clone)]
pub struct Assignment(HashMap<Variable, bool>);

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("{{ ")?;
        f.write_str(
            &self
                .0
                .iter()
                .map(|(var, truth)| format!("{} := {}", var, truth))
                .join(", "),
        )?;
        f.write_str(" }}")?;

        Ok(())
    }
}

impl Default for Assignment {
    fn default() -> Assignment {
        Assignment::new()
    }
}

impl IntoIterator for Assignment {
    type Item = (Variable, bool);
    type IntoIter = hash_map::IntoIter<Variable, bool>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<(Variable, bool)> for Assignment {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (Variable, bool)>,
    {
        self.0.extend(iter);
    }
}

impl Assignment {
    pub fn new() -> Assignment {
        Assignment(HashMap::new())
    }

    pub fn assign(&mut self, var: &Variable, truth: bool) {
        self.0.insert(var.clone(), truth);
    }

    pub fn assigned(mut self, var: &Variable, truth: bool) -> Assignment {
        self.0.insert(var.clone(), truth);
        self
    }

    pub fn get(&self, var: &Variable) -> Option<bool> {
        self.0.get(var).copied()
    }
}
