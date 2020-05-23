use std::{fmt, str};

use crate::cnf::Variable;

#[derive(Clone)]
pub enum Expr {
    Var(Variable),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

fn imply(e1: Expr, e2: Expr) -> Expr {
    Expr::Or(box Expr::Not(box e1), box e2)
}

peg::parser! {
  grammar parser() for str {
      rule variable() -> Expr
          = n:name() {? n.parse().map(Expr::Var).map_err(|_| "not a variable") }

      rule name() -> &'input str
          = quiet!{ s:$(['a'..='z' | 'A'..='Z'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) _ { s } }
          / expected!("name")

      rule _() = quiet!{ [' '|'\t'|'\n'|'\r']* }

      pub(super) rule expr() -> Expr = precedence! {
          e1:@ "↔" _ e2:(@) { Expr::And(box imply(e1.clone(), e2.clone()), box imply(e2, e1)) }
          e1:@ "<->" _ e2:(@) { Expr::And(box imply(e1.clone(), e2.clone()), box imply(e2, e1)) }
          e1:@ "→" _ e2:(@) { imply(e1, e2) }
          e1:@ "->" _ e2:(@) { imply(e1, e2) }
          --
          e1:@ "\\/" _ e2:(@) { Expr::Or(box e1, box e2) }
          e1:@ "∨" _ e2:(@) { Expr::Or(box e1, box e2) }
          --
          e1:@ "/\\" _ e2:(@) { Expr::And(box e1, box e2) }
          e1:@ "∧" _ e2:(@) { Expr::And(box e1, box e2) }
          --
          "!" _ e:@ { Expr::Not(box e) }
          "¬" _ e:@ { Expr::Not(box e) }
          --
          e:variable() { e }
          "(" _ e:expr() ")" _ { e }
      }
  }
}

#[derive(Debug)]
pub struct ParseExprError(peg::error::ParseError<peg_runtime::str::LineCol>);

impl fmt::Display for ParseExprError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to parse: {}", self.0)
    }
}

impl std::error::Error for ParseExprError {}

impl str::FromStr for Expr {
    type Err = ParseExprError;
    fn from_str(s: &str) -> Result<Expr, ParseExprError> {
        parser::expr(s).map_err(ParseExprError)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Var(v) => v.fmt(f),
            Expr::Not(box Expr::Var(v)) => write!(f, "¬{}", v),
            Expr::Not(e) => write!(f, "¬({})", e),
            Expr::And(e1, e2) => write!(f, "({}) ∧ ({})", e1, e2),
            Expr::Or(e1, e2) => write!(f, "{} ∨ {}", e1, e2),
        }
    }
}
