use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use crate::jhash;

use super::action::{JHash, ManagerAction, NodeAction};
use super::node::{DirNode, FileNode, JNode};

const ROOT_PARENT: u64 = 0;

pub struct JManager<H, N> {
    nodes: HashMap<H, N>,
    chash: HashMap<H, HashSet<H>>, // hash, children's hash
    phash: HashMap<H, H>,          // hash, parent's hash
}

impl JManager<u64, JNode> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        JManager {
            nodes: HashMap::new(),
            chash: HashMap::new(),
            phash: HashMap::new(),
        }
    }
}

impl ManagerAction<JNode, u64> for JManager<u64, JNode> {
    fn create_node(&mut self, path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        if self.is_path_exist(path) {
            let node = JNode::new(path);
            let h = jhash!(node);
            if self.nodes.contains_key(&h) {
                return Err("Node already exist".into());
            }
            self.nodes.insert(h, node);
            // judge whether it is root
            if is_root(path) {
                // there is no root
                self.phash.insert(h, ROOT_PARENT);
            } else {
                // check the root, if it is not exist, create it
                let pp = get_parent_pathbuf(path);
                let ph = self.locate_node(&pp)?;
                self.phash.insert(h, ph);
                self.chash.entry(ph).or_insert_with(HashSet::new).insert(h);
            }
            Ok(h)
        } else {
            Err("Path not exist".into())
        }
    }
    fn locate_node(&mut self, path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        if self.is_path_exist(path) {
            let h = pathbuf2hash(path);
            // check whether it is inside the tree
            if !self.nodes.contains_key(&h) {
                self.create_node(path)?;
            }
            Ok(h)
        } else {
            Err("Path not exist".into())
        }
    }
    fn delete_node(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
    
    fn update_node(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
    fn get_parent(&self, node: &u64) -> u64 {
        *self.phash.get(node).unwrap_or(&ROOT_PARENT)
    }

    fn get_children(&self, node: &u64) -> Vec<u64> {
        todo!()
    }
}

#[inline]
pub fn pathbuf2hash(path: &PathBuf) -> u64 {
    let mut hasher = DefaultHasher::new();
    let abspath = path.canonicalize().unwrap();
    abspath.hash(&mut hasher);
    hasher.finish()
}

#[inline]
fn get_parent_pathbuf(path: &PathBuf) -> PathBuf {
    let mut parent = path.clone();
    parent.pop();
    parent
}

#[inline]
fn is_root(path: &PathBuf) -> bool {
    path.parent().is_none()
}