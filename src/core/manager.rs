use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::Hash;
use std::path::PathBuf;
use std::env;
use std::time::SystemTime;

use csv::{Reader, Writer};

use crate::jhash;

use super::action::{ManagerAction, ManagerStorage, NodeAction, Scanner};
use super::node::{DumpData, JNode, JNodeInfo};

const ROOT_PARENT: u64 = 0;

#[derive(Debug)]
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
    pub fn get_info(&self, node: &u64) -> JNodeInfo {
        self.nodes.get(node).unwrap().clone().into()
    }
}

impl ManagerAction<JNode, u64> for JManager<u64, JNode> {
    fn create_node(&mut self, path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        if is_path_exist(path) {
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
                self.chash
                    .entry(ROOT_PARENT)
                    .or_insert_with(HashSet::new)
                    .insert(h);
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
        if is_path_exist(path) {
            let h = jhash!(path);
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
                self.chash.remove(&h);
            }
            // 删除当前节点的引用
            self.nodes.remove(&h);
        }
        Ok(())
    }

    fn update_node(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.nodes.contains_key(&node) {
            return Err("Node not exist".into());
        }
        // 现将所有子节点入栈，然后逐个更新叠加。
        todo!()
    }
    fn get_parent(&self, node: &u64) -> u64 {
        *self.phash.get(node).unwrap_or(&ROOT_PARENT)
    }

    fn get_children(&self, node: &u64) -> Vec<u64> {
        self.chash.get(node).unwrap_or(&HashSet::new()).iter().cloned().collect()
    }
}

impl Scanner<u64> for JManager<u64, JNode> {
    fn scan_folder(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    fn scan_folder_raw(&self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        // 用循环，不要用递归
        Ok(())
    }

    fn scan_folder_once(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

impl ManagerStorage for JManager<u64, JNode> {
    fn dump(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        let file = File::create(&file_path)?;
        let mut wtr = Writer::from_writer(file);
        for data in self
            .nodes
            .values()
            .map(|node| Into::<DumpData>::into(node.clone()))
        // TODO: optimize, remove clone
        {
            wtr.serialize(&data)?;
        }
        wtr.flush()?;
        println!("CSV file created at: {}", file_path);
        Ok(())
    }

    fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        // 读取文件
        let file = File::open(&file_path)?;
        let mut rdr = Reader::from_reader(file);

        let data = rdr
            .deserialize()
            .map(|result| result.unwrap())
            .collect::<Vec<DumpData>>()
            .into_iter()
            .map(|node| Into::<JNode>::into(node));

        for node in data {
            let abspath = node.abspath();
            // 核实该路径是否存在
            if !is_path_exist(abspath) {
                continue;
            }
            let h = jhash!(node);
            if self.nodes.get(&h).is_none() {
                // 不存在则插入
                self.create_node(abspath)?;
            }
            self.nodes.entry(h).and_modify(|value|{
                value.load(&node);
            });
        }
        Ok(())
    }
}


/// 获取父路径
#[inline]
fn get_parent_pathbuf(path: &PathBuf) -> PathBuf {
    let mut parent = path.clone();
    parent.pop();
    parent
}

/// 判断路径是否为根目录
#[inline]
pub fn is_root(path: &PathBuf) -> bool {
    path.parent().is_none()
}

/// 判断路径是否存在
#[inline]
fn is_path_exist(path: &PathBuf) -> bool {
    path.exists()
}
