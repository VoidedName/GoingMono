use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;

pub trait GVNodeKey: Hash + Eq + Clone + Ord {}

impl<T: Hash + Eq + Clone + Ord> GVNodeKey for T {}

#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct GVNode<T: GVNodeKey>(T);

#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct GVEdge<T: GVNodeKey>(T, T);

#[derive(Clone)]
pub struct GVAnnotation {
    pub label: Option<String>,
    pub style: Option<String>,
    pub fillcolor: Option<String>,
    pub fontcolor: Option<String>,
    pub shape: Option<String>,
    pub color: Option<String>,
    pub penwidth: Option<f32>,
}

impl GVAnnotation {
    pub fn new() -> Self {
        Self {
            label: None,
            style: None,
            fillcolor: None,
            fontcolor: None,
            shape: None,
            color: None,
            penwidth: None,
        }
    }

    fn to_annotation_string(&self) -> String {
        let mut res = Vec::<String>::new();
        macro_rules! push_annotation_pair {
            ($i: ident) => {
                if let Some(v) = &self.$i {
                    res.push(format!("{}=\"{}\"", stringify!($i), v.to_string().replace("\"", "\\\"")))
                }
            };
        }
        push_annotation_pair!(label);
        push_annotation_pair!(style);
        push_annotation_pair!(fillcolor);
        push_annotation_pair!(fontcolor);
        push_annotation_pair!(shape);
        push_annotation_pair!(color);
        push_annotation_pair!(penwidth);

        if res.len() > 0 {
            "[".to_string() + res.join(", ").as_str() + "]"
        } else {
            "".to_string()
        }
    }
}

#[derive(Clone)]
pub struct GVGraph<T: GVNodeKey> {
    name: Option<String>,
    nodes: HashSet<GVNode<T>>,
    edges: HashSet<GVEdge<T>>,
    graph_node_annotation: GVAnnotation,
    node_annotations: HashMap<GVNode<T>, GVAnnotation>,
    edge_annotations: HashMap<GVEdge<T>, GVAnnotation>,
}

impl<T: GVNodeKey> GVGraph<T> {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            nodes: HashSet::new(),
            edges: HashSet::new(),
            graph_node_annotation: GVAnnotation::new(),
            node_annotations: HashMap::new(),
            edge_annotations: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, key: T) {
        let node = GVNode(key);
        if !self.nodes.contains(&node) {
            self.nodes.insert(node.clone());
            self.node_annotations.insert(node, GVAnnotation::new());
        }
    }

    pub fn add_edge(&mut self, from: T, to: T) {
        let n1 = GVNode(from.clone());
        let n2 = GVNode(to.clone());
        let edge = GVEdge(from, to);
        if !self.edges.contains(&edge) {
            if !self.nodes.contains(&n1) {
                self.add_node(n1.0);
            }
            if !self.nodes.contains(&n2) {
                self.add_node(n2.0);
            }
            self.edges.insert(edge.clone());
            self.edge_annotations.insert(edge, GVAnnotation::new());
        }
    }

    pub fn get_graph_node_annotation(&self) -> &GVAnnotation {
        &self.graph_node_annotation
    }

    pub fn get_graph_node_annotation_mut(&mut self) -> &mut GVAnnotation {
        &mut self.graph_node_annotation
    }

    pub fn get_node_annotation(&self, key: T) -> Option<&GVAnnotation> {
        self.node_annotations.get(&GVNode(key))
    }

    pub fn get_node_annotation_mut(&mut self, key: T) -> Option<&mut GVAnnotation> {
        self.node_annotations.get_mut(&GVNode(key))
    }

    pub fn get_edge_annotation(&self, from: T, to: T) -> Option<&GVAnnotation> {
        self.edge_annotations.get(&GVEdge(from, to))
    }

    pub fn get_edge_annotation_mut(&mut self, from: T, to: T) -> Option<&mut GVAnnotation> {
        self.edge_annotations.get_mut(&GVEdge(from, to))
    }
}

impl<T: GVNodeKey + Display> Display for GVGraph<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Err(e) = write!(f, "digraph {} {{\n", self.name.clone().unwrap_or_default()) {
            return Err(e);
        }

        let node_annot = self.graph_node_annotation.to_annotation_string();
        if !node_annot.is_empty() {
            if let Err(e) = write!(f, "\tnode {};\n", node_annot) {
                return Err(e);
            }
        }

        let mut sorted_nodes = self.nodes.iter().collect::<Vec<_>>();
        sorted_nodes.sort();
        for node in sorted_nodes {
            if let Err(e) = write!(
                f,
                "\t{} {};\n",
                node.0,
                self.node_annotations
                    .get(&node)
                    .unwrap()
                    .to_annotation_string()
            ) {
                return Err(e);
            }
        }

        let mut sorted_eddges = self.edges.iter().collect::<Vec<_>>();
        sorted_eddges.sort();
        for edge in sorted_eddges {
            if let Err(e) = write!(
                f,
                "\t{} -> {} {};\n",
                edge.0,
                edge.1,
                self.edge_annotations
                    .get(&edge)
                    .unwrap()
                    .to_annotation_string()
            ) {
                return Err(e);
            }
        }
        write!(f, "}}")
    }
}
