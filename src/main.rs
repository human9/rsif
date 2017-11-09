extern crate sifter;
use sifter::*;

use std::env;
use std::process;
use std::error::Error;

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
    
    match config.operation.as_ref() {
        "nodes" => list_nodes(&config.files[0])?,
        "remove" => sif_quick_remove(&config.files[0], &config.files[1])?,
        _ => println!("Unimplemented operation"),
    }

    Ok(())
}

struct Config {
    operation: String,
    files: Vec<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, String> {
        args.next(); // consume the first argument

        let operation = match args.next() {
            Some(arg) => arg,
            None => return Err("No operation specified".to_string()),
        };

        let files: Vec<String> = args.collect();

        match operation.as_ref() {
            "remove" => { if files.len() < 2 { return Err(format!("{}: too few inputs specified", operation)) } },
            "nodes" | "test" => { if files.len() < 1 { return Err(format!("{}: requires input", operation)) } },
            _ => return Err(format!("{}: Unknown operation", operation)),
        }
            

        Ok(Config { operation, files })
    }
}
