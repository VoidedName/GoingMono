//////////////
// VIZGRAPH //
//////////////

use crate::geometry2d::Coordinate;
use crate::r_tree::Entry::{Leaf, NonLeaf};
use crate::r_tree::{ChildRecord, Entry, ObjectId, ObjectRecord, RTree};
use std::fmt::{Display, Formatter};
use std::process::Child;

impl<T: Coordinate + Display, O: ObjectId + Display> RTree<T, O> {
    pub fn to_vizgraph(&self, name: &str) -> String {
        match &self.root {
            None => format!("digraph {} {{\n\tNone;}}", name),
            Some(root) => {
                format!(
                    "digraph {} {{\n\tnode [style=filled, fillcolor=black, fontcolor=white, shape=record]\n\tn0 [label=\"node:0\", fillcolor=red, shape=circle]\n{}}}",
                    name,
                    root.to_vizgraph(0).1
                )
            }
        }
    }
}

impl<T: Coordinate + Display, O: ObjectId + Display> Entry<T, O> {
    fn to_vizgraph(&self, label: usize) -> (usize, String) {
        let mut result = String::new();
        match self {
            Leaf { children } => {
                for ObjectRecord(rec, id) in children {
                    result += format!("\to{} [label=\"{{({}, {}), ({}, {})|object:{}}}\"];\n",
                                      id,
                                      rec.low.x,
                                      rec.low.y,
                                      rec.high.x,
                                      rec.high.y,
                                      id,
                    ).as_str();
                    result += format!("\tn{} -> o{};\n", label, id, ).as_str();
                }
                (label, result)
            }
            NonLeaf { children, .. } => {
                let mut last_used_label = label;
                for ChildRecord(rec, child) in children {
                    let child_label = last_used_label + 1;
                    result += format!("\tn{} [label=\"{{({}, {}), ({}, {})|node:{}}}\", fillcolor=red];\n",
                                      child_label,
                                      rec.low.x,
                                      rec.low.y,
                                      rec.high.x,
                                      rec.high.y,
                                      child_label,
                    ).as_str();
                    result += format!("\tn{} -> n{};\n", label, child_label, ).as_str();
                    let (_last_used_label, subgraph) = child.to_vizgraph(child_label);
                    last_used_label = _last_used_label;
                    result += subgraph.as_str();
                }
                (label + last_used_label, result)
            }
        }
    }
}
