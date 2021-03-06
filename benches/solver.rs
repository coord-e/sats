use std::collections::{HashMap, HashSet};

use sats::assignment::{Assignment, Truth};
use sats::cnf::CNF;
use sats::solver::Solver;
use sats::{dimacs, eval};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use log::info;

#[derive(Clone, Copy)]
enum Expected {
    SAT,
    UNSAT,
}

struct Problem {
    cnf: CNF,
    expected: Expected,
    runs_on: HashSet<Solver>,
}

trait AssertSATResult {
    fn assert_sat_result(&self, cnf: &CNF, expected: Expected);
}

impl AssertSATResult for Option<Assignment> {
    fn assert_sat_result(&self, cnf: &CNF, expected: Expected) {
        match (self, expected) {
            (Some(model), Expected::SAT) => {
                if eval::eval(cnf, &model) != Truth::True {
                    panic!("SAT with invalid assignment");
                }
            }
            (None, Expected::SAT) => panic!("SAT is expected, but UNSAT is returned"),
            (Some(_), Expected::UNSAT) => panic!("UNSAT is expected, but SAT is returned"),
            (None, Expected::UNSAT) => {}
        }
    }
}

fn bench(c: &mut Criterion) {
    let samples: HashMap<String, Problem> = [
        (
            "p20",
            Expected::SAT,
            include_bytes!("data/p20.cnf").as_ref(),
            [Solver::CDCL, Solver::DPLL].as_ref(),
        ),
        (
            "p50",
            Expected::UNSAT,
            include_bytes!("data/p50.cnf").as_ref(),
            [Solver::CDCL, Solver::DPLL].as_ref(),
        ),
        (
            "zebra",
            Expected::SAT,
            include_bytes!("data/zebra_v155_c1135.cnf").as_ref(),
            [Solver::CDCL, Solver::DPLL].as_ref(),
        ),
        (
            "hole6",
            Expected::UNSAT,
            include_bytes!("data/hole6.cnf").as_ref(),
            [Solver::CDCL, Solver::DPLL].as_ref(),
        ),
        (
            "aim-50",
            Expected::SAT,
            include_bytes!("data/aim-50-1_6-yes1-4.cnf").as_ref(),
            [Solver::CDCL, Solver::DPLL].as_ref(),
        ),
        (
            "aim-100",
            Expected::UNSAT,
            include_bytes!("data/aim-100-1_6-no-1.cnf").as_ref(),
            [].as_ref(),
        ),
        (
            "dubois20",
            Expected::UNSAT,
            include_bytes!("data/dubois20.cnf").as_ref(),
            [Solver::CDCL].as_ref(),
        ),
        (
            "dubois21",
            Expected::UNSAT,
            include_bytes!("data/dubois21.cnf").as_ref(),
            [Solver::CDCL].as_ref(),
        ),
        (
            "dubois22",
            Expected::UNSAT,
            include_bytes!("data/dubois22.cnf").as_ref(),
            [Solver::CDCL].as_ref(),
        ),
        (
            "par8",
            Expected::SAT,
            include_bytes!("data/par8-1-c.cnf").as_ref(),
            [Solver::CDCL, Solver::DPLL].as_ref(),
        ),
        (
            "bf",
            Expected::SAT,
            include_bytes!("data/bf0432-007.cnf").as_ref(),
            [].as_ref(),
        ),
    ]
    .iter()
    .map(|(name, expected, data, runs_on)| {
        (
            name.to_string(),
            Problem {
                cnf: dimacs::parse::<&[u8]>(*data).unwrap(),
                expected: *expected,
                runs_on: runs_on.iter().copied().collect(),
            },
        )
    })
    .collect();

    for (
        name,
        Problem {
            cnf,
            expected,
            runs_on,
        },
    ) in samples
    {
        let mut group = c.benchmark_group(&name);
        group.sample_size(10);

        for solver in Solver::all() {
            if runs_on.contains(&solver) {
                group.bench_function(solver.to_string(), |b| {
                    b.iter(|| {
                        solver
                            .run(black_box(cnf.clone()))
                            .assert_sat_result(&cnf, expected)
                    })
                });
            } else {
                info!("skipping benchmark {}/{}", name, solver);
            }
        }

        group.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
