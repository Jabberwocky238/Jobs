use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::hash::Hash;
use std::path::PathBuf;
use std::env;
use std::time::SystemTime;

use csv::{Reader, Writer};

use crate::jhash;

use super::action::{ManagerAction, ManagerStorage, NodeAction, Scanner};
use super::errors::JError;
use super::node::{DumpData, JNode, JNodeInfo};

const ROOT_PARENT: u64 = 0;
const IGNORE_DIR: [&str; 2] = ["node_modules", ".git"];

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
    pub fn print_info(&self, node: &u64) -> String {
        self.nodes.get(node).unwrap().to_string()
    }

    #[inline]
    fn filter_dir(&self, list: &mut Vec<u64>) {
        list.retain(|&x| {
            let node = self.nodes.get(&x).unwrap();
            if let JNode::Dir(_) = node {
                true
            } else {
                false
            }
        });
    }
}

impl ManagerAction<JNode, u64> for JManager<u64, JNode> {
    fn create_node(&mut self, path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        if is_path_exist(path) {
            let node = JNode::new(path);
            let h = jhash!(node);
            if self.nodes.contains_key(&h) {
                return Err(JError::NotExistingNode(h).into());
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
                // check the parent, if it is not exist, create it
                let pp = get_parent_pathbuf(path);
                let ph = self.locate_node(&pp)?;
                self.phash.insert(h, ph);
                self.chash.entry(ph).or_insert_with(HashSet::new).insert(h);
            }
            Ok(h)
        } else {
            Err(JError::NotExistingPath(path.to_path_buf()).into())
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
            Err(JError::NotExistingPath(path.to_path_buf()).into())
        }
    }
    fn delete_node(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.nodes.contains_key(&node_h) {
            return Err(JError::NotExistingNode(*node_h).into());
        }
        let mut to_delete = vec![node_h.clone()];

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
    
    fn update_node(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.nodes.contains_key(&node_h) {
            return Err(JError::NotExistingNode(*node_h).into());
        }
        let mut waiting_list = vec![node_h.clone()];
        let mut to_update = vec![node_h.clone()];

        while let Some(h) = waiting_list.pop() {
            self.scan_folder(&h)?;
            let mut chs = self.get_children(&h);
            self.filter_dir(&mut chs);
            waiting_list.extend(chs.clone());
            to_update.extend(chs);
        }

        while let Some(h) = to_update.pop() {
            let sum_size = self.get_children(&h).iter().map(|&h| self.nodes.get(&h).unwrap().size()).sum();
            self.nodes.entry(h).and_modify(|v| {
                v.set_size(sum_size);   
            });
        }
        Ok(())
    }
    fn get_parent(&self, node: &u64) -> u64 {
        *self.phash.get(node).unwrap_or(&ROOT_PARENT)
    }

    fn get_children(&self, node: &u64) -> Vec<u64> {
        self.chash.get(node).unwrap_or(&HashSet::new()).iter().cloned().collect()
    }
}

impl Scanner<u64> for JManager<u64, JNode> {
    /// 保证map中有所有节点
    fn scan_folder(&mut self, h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(h).unwrap().abspath();
        if is_excluded(&path) {
            return self.scan_folder_raw(h);
        }
        let mut scan_list = vec![h.clone()];
        while let Some(h) = scan_list.pop() {
            self.scan_folder_once(&h)?;
            let chs = self.get_children(&h);
            scan_list.extend(chs);
        }
        Ok(())
    }

    fn scan_folder_raw(&mut self, h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(h).unwrap().abspath();
        let paths = read_dir_recursive(path)?;

        let mut size = 0;
        for path in paths {
            let metadata = fs::metadata(&path)?;
            size += metadata.len();
        }
        self.nodes.entry(*h).and_modify(|v| {
            v.set_size(size);
        });
        Ok(())
    }
    /// 保证map中有所有节点
    fn scan_folder_once(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(node).unwrap().abspath();
        if is_excluded(&path) {
            return self.scan_folder_raw(node);
        }
        for item in fs::read_dir(path)? {
            let item = item?;
            let path = item.path();
            self.locate_node(&path)?;
        }           
        Ok(())
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

#[inline]
fn is_excluded(path: &PathBuf) -> bool {
    IGNORE_DIR.contains(&path.file_name().unwrap().to_str().unwrap())
}

fn read_dir_recursive(path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !is_excluded(&path) {
            let t = read_dir_recursive(&path)?;
            paths.extend(t);
        } else {
            paths.push(path);
        }
    }
    Ok(paths)
}