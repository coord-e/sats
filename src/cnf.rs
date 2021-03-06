use std::collections::{HashMap, HashSet};
use std::{char, fmt, ops, str};

use itertools::Itertools;

mod clause_id;
mod clauses;
pub use clause_id::ClauseID;

#[derive(Debug, Clone)]
pub struct CNF {
    clauses: clauses::Clauses,
}

impl fmt::Display for CNF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.clauses.into_iter().join(" ∧ "))
    }
}

impl str::FromStr for CNF {
    type Err = ParseVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clauses = s
            .split(" ∧ ")
            .flat_map(|sub| sub.split(" /\\ "))
            .map(|c| c.parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CNF::from_clauses(clauses))
    }
}

impl CNF {
    pub fn from_clauses<T>(clauses: T) -> CNF
    where
        T: IntoIterator<Item = Clause>,
    {
        CNF {
            clauses: clauses.into_iter().collect(),
        }
    }

    pub fn most_occurred_literal(&self) -> Option<&Literal> {
        self.clauses.most_occurred_literal()
    }

    pub fn get_from_db(&self, id: ClauseID) -> Option<&Clause> {
        self.clauses.get_from_db(id)
    }

    pub fn add_clause(&mut self, clause: Clause) {
        self.clauses.add(clause);
    }

    pub fn clauses(&self) -> clauses::ClauseIter {
        self.clauses.clauses()
    }

    pub fn unit_clauses(&self) -> clauses::ClauseIter {
        self.clauses.unit_clauses()
    }

    pub fn empty_clauses(&self) -> clauses::ClauseIter {
        self.clauses.empty_clauses()
    }

    pub fn literals(&self) -> impl Iterator<Item = &Literal> {
        self.clauses.literals()
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    pub fn has_empty_clause(&self) -> bool {
        self.clauses.empty_clauses().next().is_some()
    }

    // TODO: better API
    pub fn impure_literals(&self) -> impl Iterator<Item = Literal> {
        let mut found = HashMap::with_capacity(self.clauses.len_literals());
        let mut impure = HashSet::new();
        for lit in self.literals() {
            match found.get(lit.variable()) {
                Some(sign) if lit.is_negated() == *sign => {
                    impure.insert(lit.clone());
                    impure.insert(lit.negated());
                }
                Some(_) => (),
                None => {
                    found.insert(lit.variable(), !lit.is_negated());
                }
            }
        }
        impure.into_iter()
    }

    /// simplify CNF assuming provided literal is True.
    pub fn simplify_true_literal(&mut self, literal: &Literal) {
        self.clauses.remove_clauses_with(literal);
        self.clauses.remove_literals(&literal.negated());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    literals: HashSet<Literal>,
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.literals.iter().join(" ∨ "))
    }
}

impl str::FromStr for Clause {
    type Err = ParseVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let literals = s
            .split(" ∨ ")
            .flat_map(|sub| sub.split(" \\/ "))
            .map(|l| l.parse())
            .collect::<Result<HashSet<_>, _>>()?;
        Ok(Clause { literals })
    }
}

impl Clause {
    pub fn from_literals<T>(literals: T) -> Clause
    where
        T: IntoIterator<Item = Literal>,
    {
        Clause {
            literals: literals.into_iter().collect(),
        }
    }

    pub fn literals(&self) -> impl Iterator<Item = &Literal> {
        self.literals.iter()
    }

    pub fn remove_literal(&mut self, literal: &Literal) {
        self.literals.remove(literal);
    }

    pub fn has_literal(&self, literal: &Literal) -> bool {
        self.literals.contains(literal)
    }

    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    pub fn is_unit(&self) -> bool {
        self.literals.len() == 1
    }

    pub fn unit(&self) -> Option<&Literal> {
        if self.is_unit() {
            Some(self.literals.iter().next().unwrap())
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct Literal {
    variable: Variable,
    negated: bool,
}

impl ops::Deref for Literal {
    type Target = Variable;
    fn deref(&self) -> &Variable {
        &self.variable
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&if self.negated {
            format!("¬{}", self.variable)
        } else {
            format!("{}", self.variable)
        })
    }
}

impl str::FromStr for Literal {
    type Err = ParseVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(v) = s.strip_prefix('¬') {
            Ok(Literal {
                variable: v.parse()?,
                negated: true,
            })
        } else if let Some(v) = s.strip_prefix('!') {
            Ok(Literal {
                variable: v.parse()?,
                negated: true,
            })
        } else {
            Ok(Literal {
                variable: s.parse()?,
                negated: false,
            })
        }
    }
}

impl From<Variable> for Literal {
    fn from(v: Variable) -> Literal {
        Literal::new(v, false)
    }
}

impl Literal {
    pub fn new(variable: Variable, is_negated: bool) -> Literal {
        Literal {
            variable,
            negated: is_negated,
        }
    }

    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn is_negated(&self) -> bool {
        self.negated
    }

    pub fn negate(&mut self) {
        self.negated = !self.negated;
    }

    pub fn negated(&self) -> Literal {
        let mut c = self.clone();
        c.negate();
        c
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Variable(String);

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&self.0)
    }
}

#[derive(Debug)]
enum ParseVariableErrorKind {
    InvalidVariable,
}

#[derive(Debug)]
pub struct ParseVariableError {
    kind: ParseVariableErrorKind,
}

impl fmt::Display for ParseVariableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ParseVariableError {}

impl str::FromStr for Variable {
    type Err = ParseVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() || s.chars().any(|c| !char::is_alphanumeric(c)) {
            return Err(ParseVariableError {
                kind: ParseVariableErrorKind::InvalidVariable,
            });
        }

        Ok(Variable(s.to_string()))
    }
}

impl Variable {
    /// `Variable::fresh(id)` is fresh iff `id` is unique
    pub fn fresh(id: usize) -> Variable {
        Variable(format!("x_{}", id))
    }
}
