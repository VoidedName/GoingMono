////////////
// SEARCH //
////////////

use crate::geometry2d::{Coordinate, Point, Rectangle};
use crate::r_tree::Entry::{Leaf, NonLeaf};
use crate::r_tree::{Entry, ObjectId, RTree};

impl<T: Coordinate, O: ObjectId> RTree<T, O> {
    pub fn search_area(&self, area: &Rectangle<T>) -> Vec<O> {
        match &self.root {
            None => vec![],
            Some(root) => root.search(area),
        }
    }

    pub fn search_point(&self, point: &Point<T>) -> Vec<O> {
        match &self.root {
            None => vec![],
            Some(root) => root.search(&Rectangle {
                low: point.clone(),
                high: point.clone(),
            }),
        }
    }
}

impl<T: Coordinate, O: ObjectId> Entry<T, O> {
    pub fn search(&self, area: &Rectangle<T>) -> Vec<O> {
        match self {
            Leaf { children } => children
                .iter()
                .filter(|a| a.0.intersects(area))
                .map(|c| c.1)
                .collect(),
            NonLeaf { children, .. } => children
                .iter()
                .filter(|a| a.0.intersects(area))
                .flat_map(|a| a.1.search(area))
                .collect(),
        }
    }
}
