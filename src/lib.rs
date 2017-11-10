use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufWriter;

extern crate petgraph;
use petgraph::Graph;
use petgraph::EdgeType;
use petgraph::graph::IndexType;
use petgraph::graph::NodeIndex;

pub fn list_nodes(file: &str) -> Result<(), Box<Error>> {

    let mut f = File::open(file)?;
    let mut contents = String::new();
	f.read_to_string(&mut contents)?;

    let mut writer = BufWriter::new(std::io::stdout());
    for node in nodes(&contents) {
        writeln!(writer, "{}", node).unwrap();
    }
    Ok(())
}

fn filename_to_contents(filename: &str) -> Result<String, Box<Error>> {
    let mut f = File::open(filename)?;
    let mut string = String::new();
    f.read_to_string(&mut string)?;
    Ok(string)
}

pub fn sif_overlay(to_overlay: &str, primary: &str) -> Result<(), Box<Error>> {

    let over_string = filename_to_contents(to_overlay).unwrap();
    let to = sif_to_petgraph(&over_string);

    let prim_string = filename_to_contents(primary).unwrap();
    let mut pr = sif_to_petgraph(&prim_string);
    overlay(to, pr);
    Ok(())
}

pub fn overlay(to_overlay: MappedGraph, mut primary: MappedGraph) {
    // iterate over nodes for overlay
    for (node, index) in to_overlay.map {
        // check if it's in the primary network
        if primary.map.contains_key(node) {
            // if so, iterate over the neighbour nodes in overlay
            for neighbour in to_overlay.graph.neighbors(index) {
                // check that the neighbour is also in the primary
                let neighbour_str = to_overlay.graph.node_weight(neighbour).unwrap();
                if primary.map.contains_key(neighbour_str) {
                    primary.graph.update_edge(*primary.map.get(node).unwrap(), *primary.map.get(neighbour_str).unwrap(), "OVERLAY");
                }
            }
        }
    }

    println!("{}", petgraph_to_sif(primary.graph));
}

pub fn sif_quick_remove(list_f: &str, graph_f: &str) -> Result<(), Box<Error>> {

    let mut fg = File::open(graph_f)?;
    let mut graph = String::new();
	fg.read_to_string(&mut graph)?;
    //let mut mapped = sif_to_petgraph(&graph);

    let mut fl = File::open(list_f)?;
    let mut list = String::new();
	fl.read_to_string(&mut list)?;

    let mut set = HashSet::new(); 
    for node in list.lines() {
        set.insert(node);
    }
    let filtered = graph.lines()
        .filter(|line| {
            let tokens: Vec<&str> = line.split('\t').collect();
            if tokens.len() !=3 { return false }
            if set.contains(tokens[0]) || set.contains(tokens[1]) { return false }
            true
        }).fold(String::new(), |mut s, line| {
            s.push_str(&format!("{}\n", line));
            s
        });

    println!("{}", filtered);

    Ok(())
}

/// Compare to t
/// Get a set of all nodes in the network
pub fn nodes<'a>(contents: &'a String) -> HashSet<&'a str> {
    contents.lines()
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split('\t').collect();
            if tokens.len() !=3 {
                // should probably handle this somehow?
                return None
            }
            return Some(tokens)
        }).fold(HashSet::new(), |mut set, t| {
            set.insert(t[0]);
            set.insert(t[2]);
            set
        })
}

pub struct MappedGraph<'a> {
    map: HashMap<&'a str, NodeIndex<u32>>,
    pub graph: Graph<&'a str, &'a str, petgraph::Undirected, u32>,
}

impl<'a> MappedGraph<'a> {
    pub fn new() -> Self {
        MappedGraph {
            map: HashMap::new(),
            graph: Graph::new_undirected(),
        }
    }

    /// As this is a "MappedGraph" we need to remap when we modify the graph
    pub fn remap(&mut self) {
        self.map.clear();
        for index in self.graph.node_indices() {
            self.map.insert(self.graph.node_weight(index).unwrap(), index);
        }
        
    }
}

/// Convert a sif file into a petgraph graph
pub fn sif_to_petgraph<'a>(contents: &'a String) -> MappedGraph {
    contents.lines()
        .filter_map(|line| {
            let mut tokens: Vec<&str> = line.split('\t').collect();
            if tokens.len() !=3 {
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

/// Export petgraph as sif textfile
pub fn petgraph_to_sif<'a>(mg: Graph<&'a str, &'a str, petgraph::Undirected, u32>) -> String {
    mg.edge_indices()
        .fold(String::new(), |mut s, index| {
            let (a, b) = mg.edge_endpoints(index).unwrap();
            s.push_str(&format!("{}\t{}\t{}\n", mg.node_weight(a).unwrap(), mg.edge_weight(index).unwrap(), mg.node_weight(b).unwrap()));
            s
        })
}
