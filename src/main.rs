use std::io::{self, Write};

use satat::cnf::CNF;
use satat::solver::{self, solve};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        buf.clear();
        io::stdin().read_line(&mut buf)?;

        let cnf: CNF = buf.trim().parse()?;
        if let Some(model) = solve::<solver::DPLL>(cnf) {
            println!("SAT {}", model);
        } else {
            println!("UNSAT");
        }
    }
}
