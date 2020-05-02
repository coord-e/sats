use std::io::{self, Write};

use satat::cnf::CNF;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    let mut stdout = io::stdout();

    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;

        buf.clear();
        io::stdin().read_line(&mut buf)?;

        let cnf: CNF = buf.parse()?;
        println!("{}", cnf);
    }
}
