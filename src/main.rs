use std::env;
use std::process;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashSet;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|e| {
        println!("Error: {}", e);
        process::exit(1);
    });
    if let Err(e) = run(config) {
        println!("Error: {}", e);
        process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<Error>> {
    let mut f = File::open(config.filename)?;
	
    let mut contents = String::new();
	f.read_to_string(&mut contents)?;

    match config.operation.as_ref() {
        "nodes" => println!("{:?}", nodes(&contents)),
        _ => println!("Unknown operation"),
    }

    Ok(())
}

/// Compare to t
/// Get a set of all nodes in the network
fn nodes<'a>(contents: &'a String) -> HashSet<&'a str> {
    contents.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let tokens: Vec<&str> = line.split('\t').collect();
            if tokens.len() !=3 {
                println!("Problem parsing line {}", i);
                return None
            }
            return Some(tokens)
        }).fold(HashSet::new(), |mut set, t| {
            set.insert(t[0]);
            set.insert(t[2]);
            set
        })
}


struct Config {
    operation: String,
    filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next(); // consume the first argument

        let operation = match args.next() {
            Some(arg) => arg,
            None => return Err("No operation specified"),
        };

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("No file specified"),
        };

        Ok(Config { operation, filename })
    }
}

