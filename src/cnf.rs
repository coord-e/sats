use std::{fmt, str};

use itertools::Itertools;

pub struct CNF(Vec<Clause>);

impl fmt::Display for CNF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.iter().join(" ∧ "))
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
        Ok(CNF(clauses))
    }
}

struct Clause(Vec<Literal>);

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.iter().join(" ∨ "))
    }
}

impl str::FromStr for Clause {
    type Err = ParseVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lits = s
            .split(" ∨ ")
            .flat_map(|sub| sub.split(" \\/ "))
            .map(|l| l.parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Clause(lits))
    }
}

enum Literal {
    Pos(Variable),
    Neg(Variable),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(&match self {
            Literal::Pos(v) => format!("{}", v),
            Literal::Neg(v) => format!("¬{}", v),
        })
    }
}

impl str::FromStr for Literal {
    type Err = ParseVariableError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(v) = s.strip_prefix('¬') {
            Ok(Literal::Neg(v.parse()?))
        } else if let Some(v) = s.strip_prefix('!') {
            Ok(Literal::Neg(v.parse()?))
        } else {
            Ok(Literal::Pos(s.parse()?))
        }
    }
}

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
        if s.contains(' ') {
            return Err(ParseVariableError {
                kind: ParseVariableErrorKind::InvalidVariable,
            });
        }

        Ok(Variable(s.to_string()))
    }
}
