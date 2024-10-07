use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs::{self, File};
use std::hash::Hash;
use std::path::PathBuf;
use std::time::SystemTime;

use csv::{Reader, Writer};
use serde::de::Error;

use crate::jhash;

use super::action::{ManagerAction, ManagerStorage, NodeAction, Scanner};
use super::errors::JError;
use super::node::{DumpData, JNode, JNodeInfo};
use super::utils::{get_parent_pathbuf, is_root, read_dir_recursive};

const ROOT_PARENT: u64 = 0;
const IGNORE_DIR: [&str; 2] = ["node_modules", ".git"];

#[inline]
pub fn is_excluded(path: &PathBuf) -> bool {
    IGNORE_DIR.contains(&path.file_name().unwrap().to_str().unwrap())
}

// #[cfg(not(debug_assertions))]
#[derive(Debug)]
pub struct JManager<H, N> {
    nodes: HashMap<H, N>,
    /// hash, children's hash
    chash: HashMap<H, HashSet<H>>,
    /// hash, parent's hash
    phash: HashMap<H, H>,
}

impl JManager<u64, JNode> {
    pub fn new() -> Self {
        JManager {
            nodes: HashMap::new(),
            chash: HashMap::new(),
            phash: HashMap::new(),
        }
    }
    pub fn get_info(&self, node: &u64) -> Result<JNodeInfo, Box<dyn std::error::Error>> {
        match self.nodes.get(node) {
            None => Err(JError::NotExistingNode(*node).into()),
            Some(node) => Ok(node.clone().into()),
        }
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
        // dbg!("[Jobs DEBUG] create node: {:?}", path);
        if path.exists() {
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
        if path.exists() {
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
        let mut waiting_list = VecDeque::from(vec![node_h.clone()]);
        let mut to_update = VecDeque::from(vec![node_h.clone()]);

        while let Some(h) = waiting_list.pop_front() {
            self.scan_folder(&h)?;
            // println!("----------------------");
            let mut chs = self.get_children(&h);
            self.filter_dir(&mut chs);
            waiting_list.extend(chs.clone());
            to_update.extend(chs);
        }

        while let Some(h) = to_update.pop_back() {
            let all = self
                .get_children(&h)
                .iter()
                .map(|&h| self.nodes.get(&h).unwrap())
                .collect::<Vec<_>>();

            let all_dirs =
                all.clone()
                    .into_iter()
                    .filter(|v| if let JNode::Dir(_) = v { true } else { false });
            let sum_size = all.clone().into_iter().map(|v| v.size()).sum();
            let sum_file = all
                .clone()
                .into_iter()
                .map(|v| match v {
                    JNode::File(v) => 1,
                    JNode::Dir(v) => v.count_file,
                    _ => 0,
                })
                .sum();
            let sum_dir = all
                .clone()
                .into_iter()
                .map(|v| match v {
                    JNode::Dir(v) => 1 + v.count_dir,
                    _ => 0,
                })
                .sum();
            self.nodes.entry(h).and_modify(|v| {
                // dbg!(v.abspath(), sum_size, sum_file, sum_dir);
                v.set(Some(sum_size), None, None, Some(sum_dir), Some(sum_file));
            });
        }
        Ok(())
    }
    fn get_parent(&self, node: &u64) -> u64 {
        *self.phash.get(node).unwrap_or(&ROOT_PARENT)
    }

    fn get_children(&self, node: &u64) -> Vec<u64> {
        self.chash
            .get(node)
            .unwrap_or(&HashSet::new())
            .iter()
            .cloned()
            .collect()
    }
}

impl Scanner<u64> for JManager<u64, JNode> {
    /// 保证map中有所有节点
    fn scan_folder(&mut self, h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        // dbg!("dump ok", self.nodes.get(h).unwrap());
        let path = self.nodes.get(h).unwrap().abspath();
        // dbg!("dump ok", self.nodes.get(h).unwrap());
        if is_excluded(&path) {
            return self.scan_folder_raw(h);
        }
        let mut scan_list = vec![h.clone()];
        while let Some(h) = scan_list.pop() {
            self.scan_folder_once(&h)?;
            let mut chs = self.get_children(&h);
            self.filter_dir(&mut chs);
            scan_list.extend(chs);
        }
        // println!("----------------------");
        Ok(())
    }

    fn scan_folder_raw(&mut self, h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(h).unwrap().abspath();
        let paths = read_dir_recursive(&path)?;

        let mut size = 0;
        let mut count_file = 0;
        let mut count_dir = 0;
        for path in paths {
            let metadata = fs::metadata(&path)?;
            size += metadata.len();
            if metadata.is_dir() {
                count_dir += 1;
            } else {
                count_file += 1;
            }
        }
        self.nodes.entry(*h).and_modify(|v| {
            v.set(Some(size), None, None, Some(count_dir), Some(count_file));
        });
        Ok(())
    }
    /// 保证map中有所有节点
    fn scan_folder_once(&mut self, node: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(node).unwrap().abspath();
        if is_excluded(&path) {
            return self.scan_folder_raw(node);
        }
        // dbg!("----------------------", path);
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
        let mut iter = self
            .nodes
            .values()
            .map(|node| Into::<DumpData>::into(node.clone()))
            .collect::<Vec<_>>();
        iter.sort();
        for data in iter
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
        dbg!("[Jobs DEBUG] Loading cache from CSV file...", &file_path);
        // 读取文件
        if !PathBuf::from(&file_path).exists() {
            return Ok(());
        }
        let file = match File::open(&file_path) {
            Ok(file) => file,
            Err(_) => {
                dbg!(&file_path);
                return Err(JError::CacheError.into());
            }
        };
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
            if abspath.exists() {
                continue;
            }
            let h = jhash!(node);
            if self.nodes.get(&h).is_none() {
                // 不存在则插入
                self.create_node(&abspath)?;
            }
            self.nodes.entry(h).and_modify(|value| {
                value.load(&node);
            });
        }
        Ok(())
    }
}

