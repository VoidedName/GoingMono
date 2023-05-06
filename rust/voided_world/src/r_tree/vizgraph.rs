//////////////
// VIZGRAPH //
//////////////

use crate::geometry2d::Coordinate;
use crate::graphviz::GVGraph;
use crate::r_tree::Entry::{Leaf, NonLeaf};
use crate::r_tree::{ChildRecord, Entry, ObjectId, ObjectRecord, RTree};
use std::fmt::{Display, Formatter};
use std::process::Child;

impl<T: Coordinate + Display, O: ObjectId + Display> From<RTree<T, O>> for GVGraph<String> {
    fn from(value: RTree<T, O>) -> Self {
        let mut graph = Self::new(Some("RTree".to_string()));

        let mut node = graph.get_graph_node_annotation_mut();
        node.style = Some("filled".to_string());
        node.fillcolor = Some("black".to_string());
        node.fontcolor = Some("white".to_string());
        node.shape = Some("record".to_string());

        match &value.root {
            None => {}
            Some(root) => {
                graph.add_node("n0".to_string());
                if let Some(annot) = graph.get_node_annotation_mut("n0".to_string()) {
                    annot.fillcolor = Some("red".to_string());
                    annot.shape = Some("circle".to_string());
                }
                root.fill_graph(0, &mut graph);
            }
        }

        graph
    }
}

impl<T: Coordinate + Display, O: ObjectId + Display> Entry<T, O> {
    fn fill_graph(&self, label: usize, graph: &mut GVGraph<String>) -> usize {
        match self {
            Leaf { children } => {
                for ObjectRecord(rec, id) in children {
                    let node_id = format!("o{}", id);
                    graph.add_node(node_id.clone());

                    if let Some(annot) = graph.get_node_annotation_mut(node_id.clone()) {
                        annot.label = Some(format!(
                            "{{({}, {}), ({}, {})|object:{}}}",
                            rec.low.x, rec.low.y, rec.high.x, rec.high.y, id,
                        ))
                    }

                    graph.add_edge(format!("n{}", label), node_id);
                }
                label
            }
            NonLeaf { children, .. } => {
                let mut last_used_label = label;
                for ChildRecord(rec, child) in children {
                    let child_label = last_used_label + 1;

                    let node_id = format!("n{}", child_label);
                    graph.add_node(node_id.clone());

                    if let Some(annot) = graph.get_node_annotation_mut(node_id.clone())
                    {
                        annot.label = Some(format!(
                            "{{({}, {}), ({}, {})|node:{}}}",
                            rec.low.x, rec.low.y, rec.high.x, rec.high.y, child_label,
                        ));
                        annot.fillcolor = Some("red".to_string());
                    }

                    graph.add_edge(format!("n{}", label), node_id);
                    last_used_label = child.fill_graph(child_label, graph);
                }
                label + last_used_label
            }
        }
    }
}
