/////////////
// DISPLAY //
/////////////

use crate::geometry2d::Coordinate;
use crate::r_tree::Entry::{Leaf, NonLeaf};
use crate::r_tree::{ChildRecord, Entry, ObjectId, ObjectRecord, RTree};
use std::fmt::{Display, Formatter};

impl<T: Coordinate + Display, O: ObjectId + Display> Display for RTree<T, O> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.root {
            None => write!(
                f,
                "RTree(min:{} max:{})\n\tEmpty",
                self.config.minimum_entries_per_node(),
                self.config.maximum_entries_per_node()
            ),
            Some(root) => {
                write!(
                    f,
                    "RTree(min:{} max:{}):\n{}\n",
                    self.config.minimum_entries_per_node(),
                    self.config.maximum_entries_per_node(),
                    root.to_string(1)
                )
            }
        }
    }
}

impl<T: Coordinate + Display, O: ObjectId + Display> Entry<T, O> {
    fn to_string(&self, indent: usize) -> String {
        let spacing = "\t".repeat(indent);
        let mut result = String::new();
        match self {
            Leaf { children } => {
                result += spacing.as_str();
                result += "Leaf\n";
                result += children
                    .iter()
                    .map(|ObjectRecord(rec, oid)| {
                        format!(
                            "{spacing}- [x0: {}, y0: {}, x1: {}, y1: {}]: {}",
                            rec.low.x, rec.low.y, rec.high.x, rec.high.y, oid
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    .as_str();
            }
            NonLeaf { children, .. } => {
                result += spacing.as_str();
                result += "NonLeaf\n";
                result += children
                    .iter()
                    .map(|ChildRecord(rec, child)| {
                        format!(
                            "{spacing}- [x0: {}, y0: {}, x1: {}, y1: {}]:\n{}",
                            rec.low.x,
                            rec.low.y,
                            rec.high.x,
                            rec.high.y,
                            child.to_string(indent + 1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    .as_str();
            }
        }
        result
    }
}
