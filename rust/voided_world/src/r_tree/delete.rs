////////////
// DELETE //
////////////

use crate::geometry2d::Coordinate;
use crate::r_tree::Entry::{Leaf, NonLeaf};
use crate::r_tree::{
    ChildRecord, Config, DeleteResult, Entry, ObjectId, ObjectRecord, RTree, Spacial,
};
use either::{Left, Right};

impl<T: Coordinate, O: ObjectId> RTree<T, O> {
    pub fn delete(&mut self, to_delete: ObjectRecord<T, O>) -> bool {
        match &mut self.root {
            None => false,
            Some(root) => {
                let res = match root.delete(&self.config, &to_delete) {
                    DeleteResult::Success => true,
                    DeleteResult::NoSuchRecord => false,
                    DeleteResult::Dissolved(orphans, ..) => {
                        for child in orphans.into_iter().rev().filter(|x| match x {
                            Left(_) => true,
                            Right(ChildRecord(_, entry)) => match entry {
                                NonLeaf { children, .. } if children.len() == 0 => false,
                                _ => true,
                            },
                        }) {
                            root.insert(&self.config, child);
                        }
                        true
                    }
                };
                // shorten tree
                match root {
                    Leaf { children } if children.len() == 0 => {
                        self.root = None;
                    }
                    NonLeaf { children, .. } if children.len() == 0 => {
                        self.root = None;
                    }
                    NonLeaf { children, .. } if children.len() == 1 => {
                        self.root = Some(children.remove(0).1);
                    }
                    _ => {}
                }
                res
            }
        }
    }
}

impl<T: Coordinate, O: ObjectId> Entry<T, O> {
    pub fn delete(
        &mut self,
        config: &Config,
        to_delete: &ObjectRecord<T, O>,
    ) -> DeleteResult<T, O> {
        match self {
            Leaf { children } => {
                let idx = children.iter().position(|record| record.1 == to_delete.1);
                match idx {
                    None => DeleteResult::NoSuchRecord,
                    Some(idx) => {
                        children.remove(idx);
                        if children.len() < config.minimum_entries_per_node() {
                            let mut orphans = vec![];
                            for _ in 0..children.len() {
                                orphans.push(Left(children.remove(0)))
                            }
                            DeleteResult::Dissolved(orphans, true)
                        } else {
                            DeleteResult::Success
                        }
                    }
                }
            }
            NonLeaf { children, .. } => {
                let mut dissolve = false;
                let mut delete_child = false;
                let mut dissolve_idx = 0;
                let mut orphans = vec![];
                for (idx, child) in children.iter_mut().enumerate() {
                    if child.mbb().intersects(&to_delete.mbb()) {
                        match child.1.delete(config, to_delete) {
                            DeleteResult::Success => {
                                child.0 = child.1.mbb();
                                return DeleteResult::Success;
                            }
                            DeleteResult::Dissolved(old_orphans, remove) => {
                                dissolve = true;
                                delete_child = remove;
                                dissolve_idx = idx;
                                orphans = old_orphans;
                                break;
                            }
                            DeleteResult::NoSuchRecord => {}
                        }
                    };
                }
                if dissolve {
                    if delete_child {
                        children.remove(dissolve_idx);
                    }
                    if children.len() < config.minimum_entries_per_node() {
                        for _ in 0..children.len() {
                            orphans.push(Right(children.remove(0)));
                        }
                        DeleteResult::Dissolved(orphans, true)
                    } else {
                        DeleteResult::Dissolved(orphans, false)
                    }
                } else {
                    DeleteResult::NoSuchRecord
                }
            }
        }
    }
}
