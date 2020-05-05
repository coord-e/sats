use std::collections::{
    hash_map::{self, Values},
    HashMap, HashSet,
};
use std::iter::{Chain, FromIterator};
use std::string;

use super::clause_id::{ClauseID, ClauseIDGenerator};
use crate::cnf::{Clause, Literal};

use itertools::Itertools;
use log::error;

#[derive(Debug, Clone)]
pub struct Clauses {
    clauses: HashMap<ID, Clause>,
    unit_clauses: HashMap<ID, Clause>,
    empty_clauses: HashMap<ID, Clause>,
    table: Table,
}

impl<'a> IntoIterator for &'a Clauses {
    type Item = &'a Clause;
    #[allow(clippy::type_complexity)]
    type IntoIter = Chain<
        Chain<Values<'a, ClauseID, Clause>, Values<'a, ClauseID, Clause>>,
        Values<'a, ClauseID, Clause>,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.clauses
            .values()
            .chain(self.unit_clauses.values())
            .chain(self.empty_clauses.values())
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
        let mut empty_clauses = HashMap::with_capacity(initial_capacity);
        let mut unit_clauses = HashMap::with_capacity(initial_capacity);
        let mut table = Table::with_capacity(initial_capacity);

        for (i, c) in iter.enumerate() {
            let id = ID::new(i);
            for l in c.literals() {
                table.register(l, id);
            }
            if c.is_empty() {
                empty_clauses.insert(id, c);
            } else if c.is_unit() {
                unit_clauses.insert(id, c);
            } else {
                clauses.insert(id, c);
            }
        }

        let cls = Clauses {
            clauses,
            empty_clauses,
            unit_clauses,
            table,
        };
        debug_assert!(cls.check_sanity());
        cls
    }
}

pub struct ClauseIter<'a> {
    inner: hash_map::Iter<'a, ClauseID, Clause>,
}

impl<'a> Iterator for ClauseIter<'a> {
    type Item = &'a Clause;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, c)| c)
    }
}

impl<'a> ClauseIter<'a> {
    pub fn with_id(self) -> impl Iterator<Item = (ClauseID, &'a Clause)> {
        self.inner.map(|(id, c)| (*id, c))
    }
}

impl Clauses {
    fn check_sanity(&self) -> bool {
        let literals_from_clauses: HashSet<_> =
            self.into_iter().flat_map(|c| c.literals()).collect();
        let literals_from_table: HashSet<_> = self.literals().collect();
        if literals_from_clauses != literals_from_table {
            error!(
                "stored literals mismatch: {} vs {}",
                display_as_set(literals_from_clauses),
                display_as_set(literals_from_table)
            );
            return false;
        }

        let all_clauses_prop = self.clauses.values().all(|c| !c.is_unit() && !c.is_empty());
        if !all_clauses_prop {
            error!(
                "malformed clauses: {}",
                display_as_set(self.clauses.values())
            );
            return false;
        }

        let all_unit_clauses_prop = self.unit_clauses.values().all(|c| c.is_unit());
        if !all_unit_clauses_prop {
            error!(
                "malformed unit clauses: {}",
                display_as_set(self.unit_clauses.values())
            );
            return false;
        }

        let all_empty_clauses_prop = self.empty_clauses.values().all(|c| c.is_empty());
        if !all_empty_clauses_prop {
            error!(
                "malformed empty _clauses: {}",
                display_as_set(self.empty_clauses.values())
            );
            return false;
        }

        let clause_ids: HashSet<_> = self.clauses.keys().collect();
        let unit_clause_ids = self.unit_clauses.keys().collect();
        let empty_clause_ids = self.empty_clauses.keys().collect();
        if !clause_ids.is_disjoint(&unit_clause_ids) {
            error!(
                "clause ids and unit clause ids overlap: {} vs {}",
                display_as_set(self.clauses.keys()),
                display_as_set(self.unit_clauses.keys())
            );
            return false;
        }
        if !unit_clause_ids.is_disjoint(&empty_clause_ids) {
            error!(
                "unit clause ids and empty clause ids overlap: {} vs {}",
                display_as_set(self.unit_clauses.keys()),
                display_as_set(self.empty_clauses.keys())
            );
            return false;
        }
        if !empty_clause_ids.is_disjoint(&clause_ids) {
            error!(
                "empty clause ids and clause ids overlap: {} vs {}",
                display_as_set(self.empty_clauses.keys()),
                display_as_set(self.clauses.keys())
            );
            return false;
        }

        return true;
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty() && self.unit_clauses.is_empty() && self.empty_clauses.is_empty()
    }

    pub fn clauses(&self) -> ClauseIter {
        ClauseIter {
            inner: self.clauses.iter(),
        }
    }

    pub fn unit_clauses(&self) -> ClauseIter {
        ClauseIter {
            inner: self.unit_clauses.iter(),
        }
    }

    pub fn empty_clauses(&self) -> ClauseIter {
        ClauseIter {
            inner: self.empty_clauses.iter(),
        }
    }

    pub fn literals(&self) -> impl Iterator<Item = &Literal> {
        self.table.literals()
    }

    pub fn len_literals(&self) -> usize {
        self.table.len()
    }

    fn contains_id(&self, id: ClauseID) -> bool {
        self.clauses.contains_key(&id)
            || self.unit_clauses.contains_key(&id)
            || self.empty_clauses.contains_key(&id)
    }

    fn remove_clause_by_id(&mut self, id: ClauseID) -> Option<Clause> {
        let res = self
            .clauses
            .remove(&id)
            .or_else(|| self.unit_clauses.remove(&id))
            .or_else(|| self.empty_clauses.remove(&id));

        if let Some(c) = &res {
            for l in c.literals() {
                self.table.unregister(l, id);
            }
        }

        debug_assert!(self.check_sanity());

        res
    }

    pub fn remove_clauses_with(&mut self, literal: &Literal) {
        for id in self.table.ids(literal) {
            assert!(self.contains_id(id));
            self.remove_clause_by_id(id);
        }

        // self.table.unregister_all(literal);
        debug_assert!(self.table.literals().find(|l| l == &literal).is_none());
        debug_assert!(self.check_sanity());
    }

    pub fn remove_literals(&mut self, literal: &Literal) {
        for id in self.table.ids(literal) {
            assert!(self.contains_id(id));

            assert!(!self.empty_clauses.contains_key(&id));
            if self.unit_clauses.contains_key(&id) {
                debug_assert_eq!(
                    self.unit_clauses
                        .get(&id)
                        .unwrap()
                        .literals()
                        .collect::<Vec<_>>(),
                    vec![literal]
                );
                let mut c = self.unit_clauses.remove(&id).unwrap();
                c.remove_literal(literal);
                debug_assert!(c.is_empty());
                self.empty_clauses.insert(id, c);
            } else {
                let c = self.clauses.get_mut(&id).unwrap();
                c.remove_literal(literal);
                if c.is_unit() {
                    let c = self.clauses.remove(&id).unwrap();
                    self.unit_clauses.insert(id, c);
                }
            }
        }
        self.table.unregister_all(literal);

        debug_assert!(self.check_sanity());
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    inner: HashMap<Literal, HashSet<ClauseID>>,
}

impl Table {
    pub fn with_capacity(size: usize) -> Table {
        Table {
            inner: HashMap::with_capacity(size),
        }
    }

    pub fn register(&mut self, k: &Literal, v: ClauseID) {
        self.inner
            .entry(k.clone())
            .or_insert_with(HashSet::new)
            .insert(v);
    }

    pub fn unregister_all(&mut self, k: &Literal) {
        self.inner.remove(k);
    }

    pub fn unregister(&mut self, k: &Literal, v: ClauseID) {
        if let Some(s) = self.inner.get_mut(k) {
            s.remove(&v);
            if s.is_empty() {
                self.inner.remove(k);
            }
        }
    }

    // TODO: better API
    pub fn ids(&self, k: &Literal) -> HashSet<ClauseID> {
        self.inner.get(k).cloned().unwrap_or_default()
    }

    pub fn literals(&self) -> impl Iterator<Item = &Literal> {
        self.inner.keys()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

fn display_as_set<T, I>(iter: T) -> String
where
    T: IntoIterator<Item = I>,
    I: string::ToString,
{
    format!(
        "{{ {} }}",
        iter.into_iter().map(|x| x.to_string()).join(", ")
    )
}
