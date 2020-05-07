use std::collections::HashSet;
use std::iter;

use crate::assignment::{Assignment, Truth};
use crate::cnf::{Clause, Literal, CNF};

use log::debug;

mod implication_graph;
mod level;
use implication_graph::{Decision, ImplicationGraph};
use level::Level;

enum DecideResult {
    Satisfied { assignment: Assignment },
    Decided { assignment: Assignment },
}

enum DeduceResult {
    Success(Assignment),
    Conflict,
}

enum DiagnoseResult {
    NeedsBackjump { backjump_level: Level },
    Asserted,
}

enum SearchResult {
    Satisfiable { model: Assignment },
    Conflict { backjump_level: Level },
}

pub struct Solver {
    implication_graph: ImplicationGraph,
    learned_clauses: Vec<Clause>,
}

impl Solver {
    fn new() -> Self {
        Solver {
            implication_graph: ImplicationGraph::new(),
            learned_clauses: Vec::new(),
        }
    }

    fn search(&mut self, input_cnf: CNF, level: Level) -> SearchResult {
        let mut cnf = input_cnf.clone();
        let mut decided_assignment = match self.decide(&mut cnf, level) {
            DecideResult::Decided { assignment } => assignment,
            DecideResult::Satisfied { assignment } => {
                return SearchResult::Satisfiable { model: assignment }
            }
        };

        loop {
            match self.deduce(&mut cnf, level) {
                DeduceResult::Success(deduced_assignment) => {
                    match self.search(cnf, level.next()) {
                        SearchResult::Satisfiable { mut model } => {
                            model.extend(decided_assignment);
                            model.extend(deduced_assignment);
                            return SearchResult::Satisfiable { model };
                        }
                        SearchResult::Conflict { backjump_level } => {
                            if backjump_level != level {
                                self.implication_graph.erase(level);
                                // skipping (keep backjumping)
                                return SearchResult::Conflict { backjump_level };
                            }
                            // resume from backjumping
                        }
                    }
                }
                DeduceResult::Conflict => match self.diagnose(&cnf, level) {
                    DiagnoseResult::NeedsBackjump { backjump_level } => {
                        self.implication_graph.erase(level);
                        return SearchResult::Conflict { backjump_level };
                    }
                    DiagnoseResult::Asserted => (),
                },
            }

            decided_assignment.clear();
            self.implication_graph.erase(level);
            cnf = input_cnf.clone();
            for c in &self.learned_clauses {
                cnf.add_clause(c.clone());
            }
        }
    }

    fn decide(&mut self, cnf: &mut CNF, level: Level) -> DecideResult {
        let mut assignment = Assignment::new();

        let literal = match choose_literal(cnf) {
            Some(l) => l,
            None => return DecideResult::Satisfied { assignment },
        };

        self.implication_graph.make_decision(
            literal.variable(),
            Truth::from(!literal.is_negated()),
            level,
            iter::empty(),
        );

        assignment.assign_true(&literal);
        cnf.simplify_true_literal(&literal);

        if cnf.is_empty() {
            return DecideResult::Satisfied { assignment };
        }

        debug!("DECIDE: Decided {}", assignment);
        DecideResult::Decided { assignment }
    }

    fn deduce(&mut self, cnf: &mut CNF, level: Level) -> DeduceResult {
        if cnf.has_empty_clause() {
            return DeduceResult::Conflict;
        }

        match self.unit_propagation(cnf, level) {
            Some(assignment) => {
                debug!("DEDUCE: SUCCESSS {}", assignment);
                DeduceResult::Success(assignment)
            }
            None => {
                debug!("DEDUCE: CONFLICT");
                DeduceResult::Conflict
            }
        }
    }

    fn unit_propagation(&mut self, cnf: &mut CNF, level: Level) -> Option<Assignment> {
        let (id, c) = match cnf.unit_clauses().with_id().next() {
            Some(x) => x,
            None => return Some(Assignment::new()),
        };
        let unit_literal = c.unit().unwrap().clone();

        // TODO: Refactor: remove collect() call
        let implicants: HashSet<_> = cnf
            .get_from_db(id)
            .unwrap()
            .literals()
            .filter_map(|cause| {
                if *cause == unit_literal {
                    return None;
                }

                let decision = self
                    .implication_graph
                    .find_decision(cause.variable(), Truth::from(cause.is_negated()))
                    .unwrap();
                Some(decision)
            })
            .collect();
        self.implication_graph.make_decision(
            unit_literal.variable(),
            Truth::from(!unit_literal.is_negated()),
            level,
            implicants,
        );

        cnf.simplify_true_literal(&unit_literal);
        if cnf.has_empty_clause() {
            None
        } else {
            self.unit_propagation(cnf, level)
                .map(|a| a.assigned_true(&unit_literal))
        }
    }

    fn diagnose(&mut self, cnf: &CNF, level: Level) -> DiagnoseResult {
        let preds = cnf.empty_clauses().with_id().flat_map(|(id, _)| {
            cnf.get_from_db(id).unwrap().literals().map(|l| {
                self.implication_graph
                    .find_decision(l.variable(), Truth::from(l.is_negated()))
                    .unwrap()
            })
        });
        let conflict_causes = self.find_conflict_causes(preds, level);
        let backjump_level = compute_backjump_level(conflict_causes.iter());
        let induced_clause = make_induced_clause(conflict_causes.iter());

        self.learn(induced_clause);

        if backjump_level != level || level == Level::initial() {
            debug!("DIAGNOSE: NeedsBackjump {:?}", backjump_level);
            DiagnoseResult::NeedsBackjump { backjump_level }
        } else {
            debug!("DIAGNOSE: Asserted");
            DiagnoseResult::Asserted
        }
    }

    fn learn(&mut self, clause: Clause) {
        debug!("LEARN: {}", &clause);
        self.learned_clauses.push(clause);
    }

    fn find_conflict_causes<I>(&self, direct_causes: I, level: Level) -> HashSet<Decision>
    where
        I: IntoIterator<Item = Decision>,
    {
        let mut causes = HashSet::new();
        for d in direct_causes.into_iter() {
            let preds = self.implication_graph.predecessors(&d);
            if preds.is_empty() {
                causes.insert(d);
            } else {
                for pre in preds {
                    let pred_level = pre.level();
                    if pred_level == level {
                        let pre_causes = self.find_conflict_causes(iter::once(pre), pred_level);
                        causes.extend(pre_causes);
                    } else {
                        assert!(pred_level < level);
                        causes.insert(pre);
                    }
                }
            }
        }
        causes
    }
}

fn choose_literal(cnf: &CNF) -> Option<Literal> {
    cnf.most_occurred_literal().cloned()
}

fn compute_backjump_level<'a>(causes: impl IntoIterator<Item = &'a Decision>) -> Level {
    causes.into_iter().map(|d| d.level()).max().unwrap()
}

fn make_induced_clause<'a>(causes: impl IntoIterator<Item = &'a Decision>) -> Clause {
    let mut literals = Vec::new();
    for decision in causes.into_iter() {
        let literal = Literal::new(decision.variable().clone(), decision.truth().as_bool());
        literals.push(literal);
    }
    Clause::from_literals(literals)
}

pub fn solve(cnf: CNF) -> Option<Assignment> {
    let mut solver = Solver::new();
    match solver.search(cnf, Level::initial()) {
        SearchResult::Satisfiable { model } => Some(model),
        SearchResult::Conflict { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::solve;

    #[test]
    fn test_excluded_middle() {
        assert!(solve("!A \\/ A".parse().unwrap()).is_some());
    }

    #[test]
    fn test_negated_excluded_middle() {
        assert!(solve("!A /\\ A".parse().unwrap()).is_none());
    }

    #[test]
    fn test_simple_1() {
        assert!(solve("A \\/ B /\\ A \\/ !B \\/ !A \\/ !B".parse().unwrap()).is_some());
    }

    #[test]
    fn test_simple_2() {
        assert!(solve(
            "A \\/ B /\\ !A \\/ B /\\ A \\/ !B /\\ !A \\/ !B"
                .parse()
                .unwrap()
        )
        .is_none());
    }
}
