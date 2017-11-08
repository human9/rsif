use std::env;
use std::process;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

extern crate petgraph;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

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
    let mut f = File::open(&config.files[0])?;
	
    let mut contents = String::new();
	f.read_to_string(&mut contents)?;

    match config.operation.as_ref() {
        "nodes" => println!("{:?}", nodes(&contents)),
        "test" => println!("{:?}", petgraph::dot::Dot::new(&sif_to_petgraph(&contents).graph)),
        _ => println!("Unimplemented operation"),
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

struct MappedGraph<'a> {
    map: HashMap<&'a str, NodeIndex<u32>>,
    graph: Graph<&'a str, &'a str, petgraph::Undirected, u32>,
}

impl<'a> MappedGraph<'a> {
    pub fn new() -> Self {
        MappedGraph {
            map: HashMap::new(),
            graph: Graph::new_undirected(),
        }
    }
}

/// Convert a sif file into a petgraph graph
fn sif_to_petgraph<'a>(contents: &'a String) -> MappedGraph {
    contents.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let mut tokens: Vec<&str> = line.split('\t').collect();
            if tokens.len() !=3 {
                println!("Problem parsing line {}", i);
                return None
            }
            tokens.swap(1, 2);
            return Some(tokens)
        }).fold(MappedGraph::new(), |mut graph, t| {
            let nodes: Vec<NodeIndex<u32>> = t.iter().take(2).map(|name| {
                match graph.map.entry(name) {
                    Occupied(entry) => {
                        *entry.get()
                    }
                    Vacant(entry) => {
                        let index = graph.graph.add_node(name);
                        entry.insert(index);
                        index
                    }
                }
            }).collect();
            graph.graph.update_edge(nodes[0], nodes[1], t[2]);
            graph
        })
}

struct Config {
    operation: String,
    files: Vec<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next(); // consume the first argument

        let operation = match args.next() {
            Some(arg) => arg,
            None => return Err("No operation specified"),
        };

        let files: Vec<String> = args.collect();

        match operation.as_ref() {
            "union" | "intersection" => { if files.len() < 2 { return Err("Too few files") } },
            "nodes" | "edges" | "test" => { if files.len() < 1 { return Err("Please specify a file") } },
            _ => return Err("Unknown operation"),
        }
            

        Ok(Config { operation, files })
    }
}
