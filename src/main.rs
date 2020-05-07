use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use satat::cnf::CNF;
use satat::solver::{cdcl, dpll};
use satat::{dimacs, eval};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "saturn")]
struct Opt {
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(short = "f", long, parse(from_os_str), conflicts_with = "cnf")]
    cnf_file: Option<PathBuf>,

    #[structopt(short, long, conflicts_with = "cnf_file")]
    cnf: Option<String>,
}

fn run_solve(cnf: CNF) {
    if let Some(model) = cdcl::solve(cnf.clone()) {
        // if let Some(model) = dpll::solve(cnf.clone()) {
        println!("SAT {}", model);
        println!("=> {}", eval::eval(&cnf, &model));
    } else {
        println!("UNSAT");
    }
}

fn solve_cnf(cnf_string: impl AsRef<str>) -> Result<(), Box<dyn std::error::Error>> {
    let cnf: CNF = cnf_string.as_ref().trim().parse()?;
    run_solve(cnf);
    Ok(())
}

fn solve_cnf_file(cnf_file: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(cnf_file)?;
    let cnf = dimacs::parse(file)?;
    run_solve(cnf);
    Ok(())
}

fn interactive() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        buf.clear();
        io::stdin().read_line(&mut buf)?;

        let cnf: CNF = buf.trim().parse()?;
        run_solve(cnf);
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

    match (opt.cnf, opt.cnf_file) {
        (Some(_), Some(_)) => unreachable!(),
        (Some(cnf), _) => solve_cnf(cnf),
        (_, Some(path)) => solve_cnf_file(path),
        (None, None) => interactive(),
    }
}
