//// http://www-db.deis.unibo.it/courses/SI-LS/papers/Gut84.pdf

use crate::geometry2d::{Coordinate, Rectangle};
use crate::r_tree::Entry::Leaf;
use crate::r_tree::RTreeError::{MaxMustBeAtLeastTwo, MinMustBeAtMostHalfOfMax};
use either::{Either, Left, Right};
use std::fmt::Debug;
use std::hash::Hash;
use Entry::NonLeaf;

mod delete;
mod display;
mod insert;
mod query;

pub trait ObjectId: Eq + Hash + Copy + Clone + Debug {}

impl<T: Eq + Hash + Copy + Clone + Debug> ObjectId for T {}

#[derive(Debug, Clone)]
pub struct ObjectRecord<T: Coordinate, O: ObjectId>(pub Rectangle<T>, pub O);

#[derive(Debug, Clone)]
struct ChildRecord<T: Coordinate, O: ObjectId>(Rectangle<T>, Entry<T, O>);

#[derive(Debug, Clone)]
enum Entry<T: Coordinate, O: ObjectId> {
    Leaf {
        children: Vec<ObjectRecord<T, O>>,
    },
    NonLeaf {
        level: usize,
        children: Vec<ChildRecord<T, O>>,
    },
}

impl<T: Coordinate, O: ObjectId> Entry<T, O> {
    fn level(&self) -> usize {
        match self {
            Leaf { .. } => 0,
            NonLeaf { level, .. } => *level,
        }
    }
}

#[derive(Debug, Clone)]
struct Config {
    maximum_entries_per_node: usize,
    minimum_entries_per_node: usize,
}

#[derive(Debug)]
pub enum RTreeError {
    MaxMustBeAtLeastTwo,
    MinMustBeAtMostHalfOfMax,
}

#[derive(Debug, Clone)]
pub struct RTree<T: Coordinate, O: ObjectId> {
    root: Option<Entry<T, O>>,
    config: Config,
}

trait Spacial<T: Coordinate> {
    /// Return the minimum bounding box
    fn mbb(&self) -> Rectangle<T>;
}

enum InsertionResult<T: Coordinate, O: ObjectId> {
    Split(ChildRecord<T, O>, ChildRecord<T, O>),
    NoSplit,
}

enum DeleteResult<T: Coordinate, O: ObjectId> {
    Success,
    // found and removed
    NoSuchRecord,
    // record could not be found
    Dissolved(Vec<Either<ObjectRecord<T, O>, ChildRecord<T, O>>>, bool), // found and removed, but rebalance needed
}

//////////////////
// CONSTRUCTION //
//////////////////

impl<T: Coordinate, O: ObjectId> RTree<T, O> {
    pub fn new(max: usize, min: usize) -> Result<Self, RTreeError> {
        let config = Config::new(max, min);
        match config {
            Ok(config) => Ok(Self { root: None, config }),
            Err(e) => Err(e),
        }
    }
}

impl Config {
    pub fn new(max: usize, min: usize) -> Result<Self, RTreeError> {
        if max < 2 {
            Err(MaxMustBeAtLeastTwo)
        } else if min > max / 2 {
            Err(MinMustBeAtMostHalfOfMax {})
        } else {
            Ok(Config {
                maximum_entries_per_node: max,
                minimum_entries_per_node: min,
            })
        }
    }

    pub fn maximum_entries_per_node(&self) -> usize {
        self.maximum_entries_per_node
    }

    pub fn minimum_entries_per_node(&self) -> usize {
        self.minimum_entries_per_node
    }
}

impl<T: Coordinate, O: ObjectId> Spacial<T> for ObjectRecord<T, O> {
    fn mbb(&self) -> Rectangle<T> {
        self.0
    }
}

impl<T: Coordinate, O: ObjectId> Spacial<T> for ChildRecord<T, O> {
    fn mbb(&self) -> Rectangle<T> {
        self.0
    }
}

impl<T: Coordinate, O: ObjectId> Spacial<T> for Entry<T, O> {
    fn mbb(&self) -> Rectangle<T> {
        match self {
            Leaf { children } => children
                .iter()
                .map(|c| c.mbb())
                .reduce(|l, r| l.merge(&r))
                .unwrap(),
            NonLeaf { children, .. } => children
                .iter()
                .map(|c| c.mbb())
                .reduce(|l, r| l.merge(&r))
                .unwrap(),
        }
    }
}

impl<T: Coordinate, O: ObjectId> Spacial<T> for Either<ObjectRecord<T, O>, ChildRecord<T, O>> {
    fn mbb(&self) -> Rectangle<T> {
        match self {
            Left(object) => object.mbb(),
            Right(child) => child.mbb(),
        }
    }
}
