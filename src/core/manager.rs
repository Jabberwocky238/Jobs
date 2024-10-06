use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use crate::jhash;

use super::action::{ManagerAction, NodeAction};
use super::node::JNode;

const ROOT_PARENT: u64 = 0;

pub struct JManager<H, N> {
    nodes: HashMap<H, N>,
    /// hash, children's hash
    chash: HashMap<H, HashSet<H>>, 
    /// hash, parent's hash
    phash: HashMap<H, H>,          
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
                self.chash.entry(ROOT_PARENT).or_insert_with(HashSet::new).insert(h);
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
        if !self.nodes.contains_key(&node) {
            return Err("Node not exist".into());
        } 
        let mut to_delete = vec![node.clone()];

        while let Some(h) = to_delete.pop() {
            // parent remove child
            if let Some(ph) = self.phash.get(&h) {
                self.chash.get_mut(ph).unwrap().remove(&h);
            }
            // add children to queue
            if let Some(chs) = self.chash.get(&h) {
                // 将子节点添加到待删除列表
                let chs = chs.iter().cloned().collect::<Vec<_>>();
                to_delete.extend(chs);
                self.chash.entry(h).and_modify(|v|{ v.clear() });
            }
            // 删除当前节点的引用
            self.nodes.remove(&h);
        }
        Ok(())
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
    let abspath = path.canonicalize().unwrap();
    jhash!(abspath)
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