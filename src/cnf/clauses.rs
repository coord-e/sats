use std::collections::{hash_map, HashMap, HashSet};
use std::iter::FromIterator;

use crate::cnf::{Clause, Literal};

use log::error;

#[derive(Debug, Clone)]
pub struct Clauses {
    clauses: HashMap<ID, Clause>,
    table: Table,
}

impl<'a> IntoIterator for &'a Clauses {
    type Item = &'a Clause;
    type IntoIter = hash_map::Values<'a, ID, Clause>;
    fn into_iter(self) -> Self::IntoIter {
        self.clauses.values()
    }
}

impl FromIterator<Clause> for Clauses {
    fn from_iter<T>(cls: T) -> Clauses
    where
        T: IntoIterator<Item = Clause>,
    {
        let iter = cls.into_iter();

        let (lower, upper) = iter.size_hint();
        let initial_capacity = upper.unwrap_or(lower);
        let mut clauses = HashMap::with_capacity(initial_capacity);
        let mut table = Table::with_capacity(initial_capacity);

        for c in iter {
            let id = ID::new(clauses.len());
            for l in c.literals() {
                table.register(l, id);
            }
            clauses.insert(id, c);
        }

        let cls = Clauses { clauses, table };
        debug_assert!(cls.check_sanity());
        cls
    }
}

impl Clauses {
    fn check_sanity(&self) -> bool {
        let literals_from_clauses: HashSet<_> =
            self.into_iter().flat_map(|c| c.literals()).collect();
        let literals_from_table: HashSet<_> = self.literals().collect();
        let is_sane = literals_from_clauses == literals_from_table;
        if !is_sane {
            error!(
                "insane: {:?} {:?} on {:?}",
                literals_from_clauses, literals_from_table, self
            );
        }
        is_sane
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    pub fn literals(&self) -> impl Iterator<Item = &Literal> {
        self.table.literals()
    }

    pub fn len_literals(&self) -> usize {
        self.table.len()
    }

    fn remove_clause_by_id(&mut self, id: ID) -> Option<Clause> {
        self.clauses.remove(&id).map(|c| {
            for l in c.literals() {
                self.table.unregister(l, id);
            }
            c
        })
    }

    pub fn remove_clauses_with(&mut self, literal: &Literal) {
        for id in self.table.ids(literal) {
            assert!(self.clauses.contains_key(&id));
            self.remove_clause_by_id(id);
        }

        // self.table.unregister_all(literal);
        debug_assert!(self.table.literals().find(|l| l == &literal).is_none());
        debug_assert!(self.check_sanity());
    }

    pub fn remove_literals(&mut self, literal: &Literal) {
        for id in self.table.ids(literal) {
            assert!(self.clauses.contains_key(&id));
            self.clauses.get_mut(&id).unwrap().remove_literal(literal);
        }
        self.table.unregister_all(literal);

        debug_assert!(self.check_sanity());
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    inner: HashMap<Literal, HashSet<ID>>,
}

impl Table {
    pub fn with_capacity(size: usize) -> Table {
        Table {
            inner: HashMap::with_capacity(size),
        }
    }

    pub fn register(&mut self, k: &Literal, v: ID) {
        self.inner
            .entry(k.clone())
            .or_insert_with(HashSet::new)
            .insert(v);
    }

    pub fn unregister_all(&mut self, k: &Literal) {
        self.inner.remove(k);
    }

    pub fn unregister(&mut self, k: &Literal, v: ID) {
        if let Some(s) = self.inner.get_mut(k) {
            s.remove(&v);
            if s.is_empty() {
                self.inner.remove(k);
            }
        }
    }

    // TODO: better API
    pub fn ids(&self, k: &Literal) -> HashSet<ID> {
        self.inner.get(k).cloned().unwrap_or_default()
    }

    pub fn literals(&self) -> impl Iterator<Item = &Literal> {
        self.inner.keys()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ID(usize);

impl ID {
    fn new(n: usize) -> ID {
        ID(n)
    }
}
