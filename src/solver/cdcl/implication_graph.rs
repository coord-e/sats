use std::fmt;

use super::level::Level;
use crate::assignment::Truth;
use crate::cnf::Variable;

use log::debug;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::IntoNodeReferences;
use petgraph::Direction;

struct Node {
    variable: Variable,
    truth: Truth,
    level: Level,
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} := {} @{}", self.variable, self.truth, self.level)
    }
}

struct Edge;

impl fmt::Display for Edge {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

pub struct ImplicationGraph {
    graph: DiGraph<Node, Edge>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Decision {
    idx: NodeIndex,
    variable: Variable,
    truth: Truth,
    level: Level,
}

impl Decision {
    pub fn level(&self) -> Level {
        self.level
    }

    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn truth(&self) -> Truth {
        self.truth
    }
}

impl ImplicationGraph {
    pub fn new() -> Self {
        ImplicationGraph {
            graph: DiGraph::new(),
        }
    }

    pub fn erase(&mut self, at: Level) {
        debug!("ERASE: {:?}", at);
        self.graph
            .retain_nodes(|g, idx| g.node_weight(idx).unwrap().level != at)
    }

    pub fn find_decision(&self, variable: &Variable, truth: Truth) -> Option<Decision> {
        self.graph
            .node_references()
            .find(|(_, n)| &n.variable == variable && n.truth == truth)
            .map(|(idx, _)| self.get_decision(idx))
    }

    // TODO: Refactor API
    pub fn predecessors(&self, decision: &Decision) -> Vec<Decision> {
        self.graph
            .neighbors_directed(decision.idx, Direction::Incoming)
            .map(|i| self.get_decision(i))
            .collect()
    }

    fn get_decision(&self, idx: NodeIndex) -> Decision {
        let Node {
            variable,
            truth,
            level,
        } = self.graph.node_weight(idx).unwrap();
        Decision {
            idx,
            variable: variable.clone(),
            truth: *truth,
            level: *level,
        }
    }

    pub fn make_decision<I>(
        &mut self,
        variable: &Variable,
        truth: Truth,
        level: Level,
        implicants: I,
    ) -> Decision
    where
        I: IntoIterator<Item = Decision>,
    {
        let node_data = Node {
            variable: variable.clone(),
            truth,
            level,
        };
        debug!("make_decision: {}", node_data);

        let idx = self.graph.add_node(node_data);

        for implicant in implicants.into_iter() {
            self.graph.add_edge(implicant.idx, idx, Edge);
        }

        Decision {
            idx,
            variable: variable.clone(),
            truth,
            level,
        }
    }
}
