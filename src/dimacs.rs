use std::io::{self, BufRead, BufReader, Read};
use std::{error, fmt, string};

use crate::cnf::{Clause, Literal, CNF};

#[derive(Debug)]
pub enum ParseDIMACSError {
    MalformedProblemLine(String),
    MalformedPreamble(String),
    MalformedLiteral(String),
    UnknownFormat(String),
    TooManyClauses,
    UnboundVariable(i32),
    UnexpectedEndOfFile,
    Encoding(string::FromUtf8Error),
    IO(io::Error),
}

impl From<io::Error> for ParseDIMACSError {
    fn from(err: io::Error) -> ParseDIMACSError {
        ParseDIMACSError::IO(err)
    }
}

impl From<string::FromUtf8Error> for ParseDIMACSError {
    fn from(err: string::FromUtf8Error) -> ParseDIMACSError {
        ParseDIMACSError::Encoding(err)
    }
}

impl fmt::Display for ParseDIMACSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseDIMACSError::MalformedProblemLine(line) => {
                write!(f, "malformed problem line: {}", line)
            }
            ParseDIMACSError::MalformedPreamble(line) => write!(f, "malformed preamble: {}", line),
            ParseDIMACSError::MalformedLiteral(literal) => {
                write!(f, "malformed literal: {}", literal)
            }
            ParseDIMACSError::UnknownFormat(fmt) => write!(f, "unknown format: {}", fmt),
            ParseDIMACSError::TooManyClauses => write!(f, "too many clauses"),
            ParseDIMACSError::UnboundVariable(v) => write!(f, "unbound variable: {}", v),
            ParseDIMACSError::UnexpectedEndOfFile => write!(f, "unexpected end of file"),
            ParseDIMACSError::Encoding(e) => write!(f, "encoding error: {}", e),
            ParseDIMACSError::IO(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl error::Error for ParseDIMACSError {}

pub fn parse(input: impl Read) -> Result<CNF, ParseDIMACSError> {
    let mut buffer = BufReader::new(input);
    let preamble = parse_preamble(&mut buffer)?;
    parse_clauses(&mut buffer, preamble)
}

struct Preamble {
    vars: usize,
    clauses: usize,
}

fn parse_preamble(mut buffer: impl BufRead) -> Result<Preamble, ParseDIMACSError> {
    loop {
        let mut line = String::new();
        buffer.read_line(&mut line)?;
        match line.chars().next() {
            Some('c') => continue,
            Some('p') => match line.trim().split(' ').collect::<Box<[_]>>() {
                box ["p", "cnf", vs, cls] => match (vs.parse(), cls.parse()) {
                    (Ok(vars), Ok(clauses)) => return Ok(Preamble { vars, clauses }),
                    _ => return Err(ParseDIMACSError::MalformedProblemLine(line.clone())),
                },
                box ["p", fmt, _, _] => {
                    return Err(ParseDIMACSError::UnknownFormat(fmt.to_owned()))
                }
                _ => return Err(ParseDIMACSError::MalformedProblemLine(line.clone())),
            },
            Some(_) => return Err(ParseDIMACSError::MalformedPreamble(line.clone())),
            None => return Err(ParseDIMACSError::UnexpectedEndOfFile),
        }
    }
}

fn parse_clauses(buffer: impl BufRead, preamble: Preamble) -> Result<CNF, ParseDIMACSError> {
    let Preamble {
        vars: num_vars,
        clauses: num_clauses,
    } = preamble;

    let mut clauses = Vec::new();
    let mut literals = Vec::new();

    for line in buffer.lines() {
        for token_str in line?.split(' ') {
            let token: i32 = token_str
                .trim()
                .parse()
                .map_err(|_| ParseDIMACSError::MalformedLiteral(token_str.to_owned()))?;

            if token == 0 {
                if clauses.len() >= num_clauses {
                    return Err(ParseDIMACSError::TooManyClauses);
                }

                let clause = Clause::from_literals(literals.drain(..));
                clauses.push(clause);
                continue;
            }

            if token.abs() as usize > num_vars {
                return Err(ParseDIMACSError::UnboundVariable(token));
            }
            let mut literal: Literal = format!("v{}", token.abs())
                .parse()
                .map_err(|_| ParseDIMACSError::MalformedLiteral(token_str.to_owned()))?;
            if token.is_negative() {
                literal.negate();
            }

            literals.push(literal);
        }
    }

    if !literals.is_empty() {
        return Err(ParseDIMACSError::UnexpectedEndOfFile);
    }

    Ok(CNF::from_clauses(clauses))
}
