use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::hash_map::Keys;
use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufWriter;

extern crate cgmath;
use cgmath::Vector2;
extern crate petgraph;
extern crate serde;
use serde::de::DeserializeOwned;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
use petgraph::Graph;
use petgraph::graph::NodeIndex;

pub fn read_file(filename: &str) -> Result<String, Box<Error>> {
    let mut f = File::open(filename)?;
    let mut string = String::new();
    f.read_to_string(&mut string)?;
    Ok(string)
}

pub fn list_nodes(file: &str) -> Result<(), Box<Error>> {

    let contents = read_file(file)?;

    let mut writer = BufWriter::new(std::io::stdout());
    for node in nodes(&contents) {
        writeln!(writer, "{}", node).unwrap();
    }
    Ok(())
}

/// A simpler overlay for sif formatted network overlays
pub fn sif_overlay(to_overlay: &str, primary: &str) -> Result<(), Box<Error>> {

    let over_string = read_file(to_overlay)?;
    let to = sif_to_petgraph(&over_string);

    let prim_string = read_file(primary)?;
    let pr = sif_to_petgraph(&prim_string);
    overlay(to, pr);
    Ok(())
}

/// Overlay any two networks, meaning any edges present in to_overlay will be
/// mapped onto primary. No nodes are added in this operation.
pub fn overlay(to_overlay: MappedGraph, mut primary: MappedGraph) {
    // iterate over nodes for overlay
    for (node, index) in to_overlay.map {
        // check if it's in the primary network
        if primary.map.contains_key(node) {
            // if so, iterate over the neighbour nodes in overlay
            for neighbour in to_overlay.graph.neighbors(index) {
                // check that the neighbour is also in the primary
                let neighbour_str = to_overlay.graph.node_weight(neighbour).unwrap().name;
                if primary.map.contains_key(neighbour_str) {
                    primary.graph.update_edge(*primary.map.get(node).unwrap(), *primary.map.get(neighbour_str).unwrap(), "OVERLAY");
                }
            }
        }
    }

    println!("{}", petgraph_to_sif(primary.graph));
}


/// Output the union of two sif files as sif
pub fn sif_union(a: &str, b: &str) -> Result<(), Box<Error>> {

    let mut a_string = read_file(a)?;
    let mut a_graph = sif_to_petgraph(&a_string);
    let mut b_string = read_file(b)?;
    let mut b_graph = sif_to_petgraph(&b_string);

    let mut union = MappedGraph::new();
    for name in a_graph.names().chain(b_graph.names()) {
        union.add_node(Node::new(name));
    }
    // we've got all the nodes, now add edges
    //

    for index in a_graph.graph.edge_indices() {
        let (a, b) = a_graph.graph.edge_endpoints(index).unwrap();
        let a_str = a_graph.graph.node_weight(a).unwrap().name;
        let b_str = a_graph.graph.node_weight(b).unwrap().name;

        union.graph.update_edge(*union.map.get(a_str).unwrap(), *union.map.get(b_str).unwrap(), "-");
    }
    
    for index in b_graph.graph.edge_indices() {
        let (a, b) = b_graph.graph.edge_endpoints(index).unwrap();
        let a_str = b_graph.graph.node_weight(a).unwrap().name;
        let b_str = b_graph.graph.node_weight(b).unwrap().name;

        union.graph.update_edge(*union.map.get(a_str).unwrap(), *union.map.get(b_str).unwrap(), "-");
    }
    
    println!("{}", petgraph_to_sif(union.graph));

    Ok( () )

}

/// Remove any nodes within the list from a sif file
/// Accepts a newline delimited list and a standard sif file as input
pub fn sif_quick_remove(list_f: &str, graph_f: &str) -> Result<(), Box<Error>> {

    let graph = read_file(graph_f)?;
    let list = read_file(list_f)?;

    let mut set = HashSet::new(); 
    for node in list.lines() {
        set.insert(node);
    }
    let filtered = graph.lines()
        .filter(|line| {
            let tokens: Vec<&str> = line.split('\t').collect();
            if tokens.len() !=3 { return false }
            if set.contains(tokens[0]) || set.contains(tokens[2]) { return false }
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
    pub graph: Graph<Node<'a>, &'a str, petgraph::Undirected, u32>,
}

impl<'a> MappedGraph<'a> {
    pub fn new() -> Self {
        MappedGraph {
            map: HashMap::new(),
            graph: Graph::new_undirected(),
        }
    }

    pub fn names(&self) -> Keys<&'a str, NodeIndex<u32>> {
        self.map.keys()
    }

    /// Add a new node with the given name.
    /// Returns the new NodeIndex, or an old NodeIndex if it already existed.
    pub fn add_node(&mut self, weight: Node<'a>) -> NodeIndex<u32> {
        match self.map.entry(weight.name) {
            Occupied(entry) => {
                *entry.get()
            }
            Vacant(entry) => {
                let index = self.graph.add_node(Node::new(weight.name)); 
                entry.insert(index);
                index
            }
        }
    }

    /// As this is a "MappedGraph" we need to remap when we modify the graph
    pub fn remap(&mut self) {
        self.map.clear();
        for index in self.graph.node_indices() {
            self.map.insert(self.graph.node_weight(index).unwrap().name, index);
        }
        
    }
}

pub fn to_sif(input: &str) -> Result<(), Box<Error>> {

    let contents = read_file(input)?;
    let serial: SerialJSON = serde_json::from_str(&contents)?;
    let graph = json_to_petgraph(&serial)?;
    println!("{}", petgraph_to_sif(graph.graph));

    Ok(())
}


pub fn to_json(input: &str) -> Result<(), Box<Error>> {

    let graph_str = read_file(input)?;
    let graph = sif_to_petgraph(&graph_str);

    petgraph_to_json(graph.graph);

    Ok(())
}


#[derive(Serialize, Deserialize)]
struct JSONGraph {
    nodes: Vec<String>,
    data: Vec<String>,
    indices: Vec<usize>,
    indptr: Vec<usize>
}

#[derive(Serialize, Deserialize)]
struct AntisinkMap {

}

#[derive(Serialize, Deserialize)]
pub struct SerialJSON<'a> {
    model: &'a str,
    antisink_map: AntisinkMap,
    source_nodes: Vec<String>,
    sink_nodes: Vec<String>,
    df: f32,
    graph: JSONGraph,
}

/// Print as json
pub fn petgraph_to_json<'a>(graph: Graph<Node, &'a str, petgraph::Undirected, u32>) {
    
    let mut nodes = Vec::new();
    let mut indices = Vec::new();
    let mut index_ptr = Vec::new();

    let mut last = 0;
    index_ptr.push(last);

    for index in graph.node_indices() {
        nodes.push(graph.node_weight(index).unwrap().name.to_string());
        let mut neighbors: Vec<usize> = graph.neighbors(index).map(|n| { n.index() }).collect();
        last = last + neighbors.len();
        index_ptr.push(last);
        indices.append(&mut neighbors);
    }

    //use the indptr to see how many edges need to be read (from index)
    //index into the nodes to identify names
    //data contains edge information
    //

    let mut data = Vec::new();
    let mut i = 0;
    while i < nodes.len() {
        data.push("1.0".to_string());
        i = i+1;
    }
    

    let output = SerialJSON {
        model: "normalized-channel",
        antisink_map: AntisinkMap {},
        source_nodes: vec!["Test".to_string()],
        sink_nodes: vec!["Test".to_string()],
        df: 0.85,
        graph: JSONGraph { 
            nodes: nodes,
            data: data,
            indices: indices,
            indptr: index_ptr
        },
    };
    println!("{}", serde_json::to_string(&output).unwrap());
}

pub fn json_to_petgraph<'a>(serial: &'a SerialJSON) -> Result<MappedGraph<'a>, Box<Error>> {

    let mut graph = MappedGraph::new();
    let mut node_indices = Vec::new();
    
    for node in 0..serial.graph.nodes.len() {
        node_indices.push(graph.add_node(Node::new(&serial.graph.nodes[node])));
    }

    let mut source = 0;
    let mut last: usize = 0;
    for ptr in serial.graph.indptr.iter() {
        while last < *ptr {
            let source = serial.graph.indices[source];
            let target = serial.graph.indices[last];
            graph.graph.update_edge(node_indices[source], node_indices[target], "json");
            last += 1;
        }
        source = last;
    }
    Ok(graph) 
}

pub struct Node<'a> {
    pub name: &'a str,
    pub pos: Vector2<f32>,
    pub disp: Vector2<f32>,
}

impl<'a> Node<'a> {

    pub fn new(name: &'a str) -> Self {
        Node {
            name: name,
            pos: Vector2::new(0., 0.),
            disp: Vector2::new(0., 0.),
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
            let nodes: Vec<NodeIndex<u32>> = t.iter().take(2).map(|name| graph.add_node(Node::new(name))).collect();
            graph.graph.update_edge(nodes[0], nodes[1], t[2]);
            graph
        })
}

/// Export petgraph as sif textfile
pub fn petgraph_to_sif<'a>(mg: Graph<Node, &'a str, petgraph::Undirected, u32>) -> String {
    mg.edge_indices()
        .fold(String::new(), |mut s, index| {
            let (a, b) = mg.edge_endpoints(index).unwrap();
            s.push_str(&format!("{}\t{}\t{}\n", mg.node_weight(a).unwrap().name, mg.edge_weight(index).unwrap(), mg.node_weight(b).unwrap().name));
            s
        })
}
