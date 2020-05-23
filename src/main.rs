use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sats::cnf::CNF;
use sats::solver::Solver;
use sats::{dimacs, eval, tseytin};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "saturn")]
struct Opt {
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(
        short = "f",
        long,
        parse(from_os_str),
        conflicts_with = "input",
        conflicts_with = "expr"
    )]
    cnf_file: Option<PathBuf>,

    #[structopt(short, long, conflicts_with = "cnf_file")]
    input: Option<String>,

    #[structopt(short, long, conflicts_with = "cnf_file")]
    expr: bool,

    #[structopt(short, long, default_value = "CDCL", possible_values = &["CDCL", "DPLL"])]
    solver: Solver,
}

fn run_solve(solver: Solver, cnf: CNF) {
    if let Some(model) = solver.run(cnf.clone()) {
        // if let Some(model) = dpll::solve(cnf.clone()) {
        println!("SAT {}", model);
        println!("=> {}", eval::eval(&cnf, &model));
    } else {
        println!("UNSAT");
    }
}

fn get_cnf(input: &str, is_expr: bool) -> Result<CNF, Box<dyn std::error::Error>> {
    if is_expr {
        let e = input.trim().parse()?;
        let cnf = tseytin::to_cnf(e);
        println!("CNF: {}", &cnf);
        Ok(cnf)
    } else {
        input.trim().parse().map_err(Into::into)
    }
}

fn solve(
    solver: Solver,
    is_expr: bool,
    input: impl AsRef<str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let cnf = get_cnf(input.as_ref(), is_expr)?;
    run_solve(solver, cnf);
    Ok(())
}

fn solve_file(
    solver: Solver,
    cnf_file: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(cnf_file)?;
    let cnf = dimacs::parse(file)?;
    run_solve(solver, cnf);
    Ok(())
}

fn interactive(solver: Solver, is_expr: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        buf.clear();
        io::stdin().read_line(&mut buf)?;

        let cnf = get_cnf(&buf, is_expr)?;
        run_solve(solver, cnf);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let log_level = match opt.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };

    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log_level)
        .chain(std::io::stderr())
        .apply()
        .unwrap();

    match (opt.input, opt.cnf_file, opt.expr) {
        (Some(_), Some(_), _) => unreachable!(),
        (Some(input), _, is_expr) => solve(opt.solver, is_expr, input),
        (_, Some(path), false) => solve_file(opt.solver, path),
        (_, Some(_), true) => unreachable!(),
        (None, None, is_expr) => interactive(opt.solver, is_expr),
    }
}
