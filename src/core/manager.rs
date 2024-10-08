use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::hash::Hash;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{env, vec};

use csv::{Reader, Writer};
use serde::de::Error;

use crate::jhash;

use super::action::{JNodeAction, ManagerAction, ManagerStorage};
use super::errors::JError;
use super::node::{get_last_modified, DumpData, JNode};
use super::utils::{get_parent_pathbuf, is_root, read_dir_recursive, read_dir_recursive_};

const ROOT_PARENT: u64 = 0;
const IGNORE_DIR: [&str; 2] = ["node_modules", ".git"];

#[inline]
pub fn is_excluded(path: &PathBuf) -> bool {
    IGNORE_DIR.contains(&path.file_name().unwrap().to_str().unwrap())
}

#[derive(Debug)]
pub struct JManager<H, N> {
    pub nodes: HashMap<H, N>, // pub only for test
    /// hash, children's hash
    pub chash: HashMap<H, HashSet<H>>, // pub only for test
    /// hash, parent's hash
    pub phash: HashMap<H, H>, // pub only for test
}

impl JManager<u64, JNode> {
    pub fn new() -> Self {
        JManager {
            nodes: HashMap::new(),
            chash: HashMap::new(),
            phash: HashMap::new(),
        }
    }
    pub fn get_info(&self, node: &u64) -> Result<JNode, Box<dyn std::error::Error>> {
        match self.nodes.get(node) {
            None => Err(JError::NotExistingNode(line!(), *node).into()),
            Some(node) => Ok(node.clone().into()),
        }
    }

    #[cfg(debug_assertions)]
    pub fn get_node_cnt(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_children_node(&self, node: &u64) -> Vec<(&JNode, u64)> {
        self.chash
            .get(node)
            .unwrap_or(&HashSet::new())
            .iter()
            .map(|&v| (self.nodes.get(&v).unwrap(), v))
            .collect::<Vec<_>>()
    }
}

impl ManagerAction for JManager<u64, JNode> {
    type H = u64;

    fn create_node(&mut self, path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        // dbg!("[Jobs DEBUG] create node: {:?}", path);
        if !path.exists() {
            return Err(JError::NotExistingPath(path.to_path_buf()).into());
        }
        let node = JNode::new(path);
        let h = jhash!(node);
        if self.nodes.contains_key(&h) {
            return Err(JError::NotExistingNode(line!(), h).into());
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
        self.propagate_dirty(&h)?;
        Ok(h)
    }
    fn locate_node(&mut self, path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        let path = path.canonicalize()?;
        if !path.exists() {
            return Err(JError::NotExistingPath(path.to_path_buf()).into());
        }
        let h = jhash!(path);
        // check whether it is inside the tree
        if !self.nodes.contains_key(&h) {
            self.create_node(&path)?;
        }
        Ok(h)
    }
    fn delete_node(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.nodes.contains_key(&node_h) {
            // return Err(JError::NotExistingNode(line!(), *node_h).into());
            return Ok(());
        }
        let mut to_delete = vec![node_h.clone()];

        while let Some(h) = to_delete.pop() {
            // parent remove child
            if let Some(ph) = self.phash.get(&h) {
                self.chash.get_mut(ph).unwrap().remove(&h);
            }
            // add children to queue
            if let Some(chs) = self.chash.get(&h) {
                let chs = chs.iter().cloned().collect::<Vec<_>>();
                to_delete.extend(chs);
            }
            self.nodes.remove(&h);
        }
        self.propagate_dirty(node_h)?;
        Ok(())
    }

    /// 1，扫描此节点实际的文件系统，把没见过的子节点插入表内
    ///
    /// 2，扫描所有子节点的文件是否合法
    ///
    /// 3，更新过期的子节点
    ///
    /// 4，更新此节点
    fn update_node(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        if !self.nodes.contains_key(&node_h) {
            return Err(JError::NotExistingNode(line!(), *node_h).into());
        }
        if let Some(JNode::File(v)) = self.nodes.get_mut(node_h) {
            v.update();
            return Ok(());
        }

        // 1，扫描此节点实际的文件系统，把没见过的子节点插入表内
        self.scan_folder(&node_h)?;
        // 2, 扫描所有子节点的文件是否合法
        self.check_file_dirty(&node_h);
        // 2，更新过期的子节点
        let mut check_update = self
            .get_children_node(&node_h)
            .into_iter()
            .filter(|(v, h)| !v.is_valid())
            .collect::<Vec<_>>();
        check_update.sort_by_key(|(v, h)| v.path());
        let mut check_update = check_update.into_iter().map(|(_, h)| h).collect::<Vec<_>>();
        while let Some(h) = check_update.pop() {
            // recursive here!!!
            self.update_node(&h)?;
        }
        // 3，更新此节点
        let all = self.get_children_node(&node_h);
        let iter = all.clone().into_iter();
        let mut sum_size = 0;
        let mut sum_file = 0;
        let mut sum_dir = 0;

        for (v, h) in iter {
            sum_size += v.size();
            sum_file += match v {
                JNode::File(v) => 1,
                JNode::Dir(v) => v.count_file,
                _ => 0,
            };
            sum_dir += match v {
                JNode::Dir(v) => 1 + v.count_dir,
                _ => 0,
            };
        }
        self.nodes.entry(*node_h).and_modify(|v| {
            // dbg!(v.name(), sum_size, sum_file, sum_dir);
            v.set(
                Some(sum_size),
                Some(get_last_modified(&v.path())),
                Some(true),
                Some(sum_dir),
                Some(sum_file),
                Some(false),
            );
        });
        self.propagate_dirty(&node_h)?;
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

impl JManager<u64, JNode> {
    /// 确保当前节点下所有文件没有修改过，否则就传递脏标
    /// only add or delete will affect dir modify time
    fn scan_folder(&mut self, h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(h).unwrap().path();
        let mut scan_list = vec![h.clone()];
        while let Some(h) = scan_list.pop() {
            if self.nodes.get(&h).unwrap().is_valid() {
                continue;
            }
            self.scan_folder_once(&h)?;
            let mut chs = self.get_children(&h);
            chs.retain(|&x| self.nodes.get(&x).unwrap().is_dir());
            scan_list.extend(chs);
        }
        Ok(())
    }

    fn scan_folder_raw(&mut self, h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(h).unwrap().path();
        let (size, count_file, count_dir) = read_dir_recursive_(&path)?;
        self.nodes.entry(*h).and_modify(|v| {
            v.set(
                Some(size),
                None,
                Some(true),
                Some(count_dir as usize),
                Some(count_file as usize),
                Some(false),
            );
        });
        Ok(())
    }

    /// 保证map中有所有节点，并且清除不存在的节点
    fn scan_folder_once(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.nodes.get(node_h).unwrap().path();
        if is_excluded(path) {
            return self.scan_folder_raw(node_h);
        }
        if self.nodes.get(node_h).unwrap().is_valid() {
            return Ok(());
        }
        for item in fs::read_dir(path)? {
            let item = item?;
            let path = item.path().canonicalize().unwrap();
            self.locate_node(&path)?;
        }
        for ch in self.get_children(node_h) {
            let node = self.nodes.get(&ch).unwrap();
            if !node.path().exists() {
                self.delete_node(&ch);
            }
        }
        Ok(())
    }

    fn propagate_dirty(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let parent_h = self.get_parent(node_h);
        if parent_h != ROOT_PARENT {
            // if !self.nodes.get(&parent_h).unwrap().is_valid() {
            //     return Ok(());
            // }
            self.nodes.entry(parent_h).and_modify(|v| {
                v.set(None, None, None, None, None, Some(true));
            });
            self.propagate_dirty(&parent_h)?;
        }
        Ok(())
    }

    fn check_file_dirty(&mut self, node_h: &u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut check_queue = vec![];
        let mut load_queue = vec![node_h.clone()];
        check_queue.reserve(20000);
        load_queue.reserve(20000);
        while let Some(h) = load_queue.pop() {
            load_queue.extend(self.get_children(&h));
            check_queue.extend(self.get_children(&h));
        }
        // dbg!(&load_queue);
        while let Some(h) = check_queue.pop() {
            if !self.nodes.get(&h).unwrap().is_valid() {
                self.propagate_dirty(&h)?;
            }
        }
        Ok(())
    }
}

impl ManagerStorage for JManager<u64, JNode> {
    fn dump(&self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
        Ok(())
    }

    fn load(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if !PathBuf::from(&file_path).exists() {
            return Ok(());
        }
        let file = match OpenOptions::new().read(true).open(&file_path) {
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
            .filter(|v| PathBuf::from(&v.abspath).exists())
            .map(|node| Into::<JNode>::into(node));

        for node in data {
            let h = self.locate_node(node.path())?;
            // dbg!(&node);
            self.nodes.entry(h).and_modify(|value| {
                value.load(&node);
            });
        }
        Ok(())
    }
}
