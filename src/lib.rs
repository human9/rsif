use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

extern crate petgraph;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

/// Compare to t
/// Get a set of all nodes in the network
pub fn nodes<'a>(contents: &'a String) -> HashSet<&'a str> {
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
}

/// Convert a sif file into a petgraph graph
pub fn sif_to_petgraph<'a>(contents: &'a String) -> MappedGraph {
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

/// Export petgraph as sif textfile
pub fn petgraph_to_sif<'a>(mg: Graph<&'a str, &'a str, petgraph::Undirected, u32>) -> String {
    mg.edge_indices()
        .fold(String::new(), |mut s, index| {
            let (a, b) = mg.edge_endpoints(index).unwrap();
            s.push_str(&format!("{}\t{}\t{}\n", mg.node_weight(a).unwrap(), mg.edge_weight(index).unwrap(), mg.node_weight(b).unwrap()));
            s
        })
}
