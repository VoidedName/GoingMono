////////////
// INSERT //
////////////

use crate::geometry2d::{Coordinate, Rectangle};
use crate::r_tree::Entry::{Leaf, NonLeaf};
use crate::r_tree::InsertionResult::{NoSplit, Split};
use crate::r_tree::{
    ChildRecord, Config, Entry, InsertionResult, ObjectId, ObjectRecord, RTree, Spacial,
};
use either::{Either, Left, Right};
use std::cmp::Ordering;
use std::collections::HashSet;

mod dimensions;

struct DimExtremes<T: Coordinate> {
    min_low: T,
    max_low: T,
    min_high: T,
    max_high: T,
    low_idx: usize,
    high_idx: usize,
}

impl<T: Coordinate, O: ObjectId> RTree<T, O> {
    pub(crate) fn insert(&mut self, record: ObjectRecord<T, O>) {
        match &mut self.root {
            None => {
                self.root = Some(Leaf {
                    children: vec![record],
                })
            }
            Some(root) => match root.insert(&self.config, Left(record)) {
                NoSplit => {}
                Split(left, right) => {
                    self.root = Some(NonLeaf {
                        level: left.1.level(),
                        children: vec![left, right],
                    })
                }
            },
        };
    }
}

impl<T: Coordinate, O: ObjectId> Entry<T, O> {
    pub fn insert(
        &mut self,
        config: &Config,
        record: Either<ObjectRecord<T, O>, ChildRecord<T, O>>,
    ) -> InsertionResult<T, O> {
        match self {
            Leaf { children } => {
                if let Left(record) = record {
                    children.push(record);
                    if children.len() > config.maximum_entries_per_node() {
                        self.try_split(config)
                    } else {
                        NoSplit
                    }
                } else {
                    panic!("Can not insert a child record into a leaf")
                }
            }
            NonLeaf { children, level } => {
                let sink = match &record {
                    Left(_) => true,
                    Right(ChildRecord(.., child)) => child.level() + 1 == *level,
                };

                if sink {
                    // insert into subtree
                    let mut best_candidate = 0;
                    let mut best_mbb = children[best_candidate].0.merge(&record.mbb());
                    let mut best_diff = best_mbb.area() - children[best_candidate].0.area();
                    for (candidate, child) in children.iter().enumerate() {
                        let test_mbb = child.0.merge(&record.mbb());
                        let test_diff = test_mbb.area() - child.0.area();
                        if test_diff < best_diff
                            || (test_diff == best_diff && test_mbb.area() < best_mbb.area())
                        {
                            best_mbb = test_mbb;
                            best_diff = test_diff;
                            best_candidate = candidate;
                        }
                    }
                    let mut candidate = &mut children[best_candidate];
                    candidate.0 = best_mbb;
                    match candidate.1.insert(config, record) {
                        NoSplit => NoSplit,
                        Split(left, right) => {
                            children.remove(best_candidate);
                            children.push(left);
                            children.push(right);
                            self.try_split(config)
                        }
                    }
                } else {
                    // insert here
                    if let Right(record) = record {
                        children.push(record);
                        self.try_split(config)
                    } else {
                        panic!("Can not insert object record into NonLeaf")
                    }
                }
            }
        }
    }

    fn try_split(&mut self, config: &Config) -> InsertionResult<T, O> {
        let mut to_split = self.indexed_mbbs(config);

        match &mut to_split {
            None => NoSplit,
            Some(group) => {
                let (left, _) = Self::linear_partition(group);
                match self {
                    Leaf { children } => {
                        let left_lookup: HashSet<_> = left.into_iter().collect();
                        let mut left = vec![];
                        let mut right = vec![];
                        for idx in (0..children.len()).rev() {
                            if left_lookup.contains(&idx) {
                                left.push(children.remove(idx));
                            } else {
                                right.push(children.remove(idx));
                            }
                        }
                        let left = Leaf { children: left };
                        let right = Leaf { children: right };
                        Split(
                            ChildRecord(left.mbb(), left),
                            ChildRecord(right.mbb(), right),
                        )
                    }
                    NonLeaf { children, level } => {
                        let left_lookup: HashSet<_> = left.into_iter().collect();
                        let mut left = vec![];
                        let mut right = vec![];
                        for idx in (0..children.len()).rev() {
                            if left_lookup.contains(&idx) {
                                left.push(children.remove(idx));
                            } else {
                                right.push(children.remove(idx));
                            }
                        }
                        let left = NonLeaf {
                            level: *level,
                            children: left,
                        };
                        let right = NonLeaf {
                            level: *level,
                            children: right,
                        };
                        Split(
                            ChildRecord(left.mbb(), left),
                            ChildRecord(right.mbb(), right),
                        )
                    }
                }
            }
        }
    }

    fn linear_partition(group: &mut Vec<(usize, Rectangle<T>)>) -> (Vec<usize>, Vec<usize>) {
        let seed1;
        let seed2;
        let first = &group[0].1;
        let mut stats_x = DimExtremes::new(first.low.x, first.high.x, 0);
        let mut stats_y = DimExtremes::new(first.low.y, first.high.y, 0);
        for (idx, rect) in group.iter() {
            stats_x.update(rect.low.x, rect.high.x, *idx);
            stats_y.update(rect.low.y, rect.high.y, *idx);
        }
        let dist_near_x = stats_x.min_high - stats_x.max_low;
        let dist_far_x = stats_x.max_high - stats_x.min_low;
        let dist_near_y = stats_y.min_high - stats_y.max_low;
        let dist_far_y = stats_y.max_high - stats_y.min_low;
        let normalized_x = dist_far_x / dist_near_x;
        let normalized_y = dist_far_y / dist_near_y;

        if normalized_x > normalized_y {
            seed1 = stats_x.low_idx;
            seed2 = stats_x.high_idx;
        } else {
            seed1 = stats_y.low_idx;
            seed2 = stats_y.high_idx;
        }

        let (seed1, seed2) = match seed1.cmp(&seed2) {
            Ordering::Less => (seed1, seed2),
            Ordering::Greater => (seed2, seed1),
            Ordering::Equal if seed1 == 0 => (0, 1),
            Ordering::Equal => (0, seed1),
        };

        let mut group2 = vec![group.remove(seed2).0];
        let mut group1 = vec![group.remove(seed1).0];
        for (idx, _) in group.iter() {
            if idx % 2 == 0 {
                group1.push(*idx);
            } else {
                group2.push(*idx);
            }
        }
        (group1, group2)
    }

    fn indexed_mbbs(&mut self, config: &Config) -> Option<Vec<(usize, Rectangle<T>)>> {
        let to_split = match self {
            Leaf { children } => {
                if children.len() > config.maximum_entries_per_node {
                    let boxes: Vec<_> = children
                        .iter()
                        .map(|ObjectRecord(rec, ..)| *rec)
                        .enumerate()
                        .collect();
                    Some(boxes)
                } else {
                    None
                }
            }
            NonLeaf { children, .. } => {
                if children.len() > config.maximum_entries_per_node {
                    let boxes: Vec<_> = children
                        .iter()
                        .map(|ChildRecord(rec, ..)| *rec)
                        .enumerate()
                        .collect();
                    Some(boxes)
                } else {
                    None
                }
            }
        };
        to_split
    }
}
