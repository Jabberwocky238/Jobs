use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use std::rc::Rc;
use std::time::SystemTime;

const EXCLUDE_DIR: [&str; 3] = ["node_modules", ".git", ".ipynb_checkpoints"];
const TREE_INDENT: usize = 4; // greater than 1

#[derive(Debug, Clone)]
pub enum JNode {
    FileInfo(FileInfo),
    DirInfo(DirInfo),
    NotRecord(FileInfo), // dir, but treated as a big file
}

impl JNode {
    pub fn verify(&self) -> bool {
        match self {
            JNode::FileInfo(info) => info.verify(),
            JNode::DirInfo(info) => info.verify(),
            JNode::NotRecord(info) => info.verify(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct DirInfo {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
    pub count_dir: usize,
    pub count_file: usize,
    fully_scan: bool,
}

pub struct JManager {
    pub map: Rc<RefCell<HashMap<PathBuf, JNode>>>,
}

impl JManager {
    pub fn new() -> JManager {
        JManager {
            map: Rc::new(RefCell::new(HashMap::new())),
        }
    }
    // pub fn find(&self, path: &Path) -> &JNode {
    //     self.find_mut(path)
    // }
    pub fn find_mut(&mut self, path: &Path) -> JNode {
        self.map.borrow_mut().entry(path.to_path_buf()).or_insert_with(|| {
            if !fs::metadata(path).is_ok() {
                panic!("path not exist: {}", path.to_str().unwrap());
            }
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if EXCLUDE_DIR.contains(&dir_name) {
                let mut file_info = FileInfo::new(path);
                file_info.not_record_scan();
                return JNode::NotRecord(file_info);
            } else {
                return JNode::DirInfo(DirInfo::new(path));
            }
        }).clone()
    }
    pub fn scan(&mut self, path: &Path) {
        if let JNode::DirInfo(mut dir_info) = self.find_mut(path) {
            dir_info.scan(self.map.clone());
        }
    }
}

impl FileInfo {
    pub fn new(path: &Path) -> FileInfo {
        if !path.is_absolute() {
            panic!("path must be absolute");
        }
        // println!("New FileInfo: {}", path.to_str().unwrap());
        let metadata = fs::metadata(path).unwrap();
        FileInfo {
            abspath: path.to_path_buf(),
            last_write_time: get_last_modified(&metadata),
            size: metadata.len(),
        }
    }
    pub fn update(&mut self) {
        let metadata = fs::metadata(self.abspath.as_path()).unwrap();
        self.last_write_time = get_last_modified(&metadata);
        self.size = metadata.len();
    }
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.abspath.hash(&mut hasher);
        hasher.finish()
    }
    pub fn verify(&self) -> bool {
        let metadata = fs::metadata(self.abspath.as_path()).unwrap();
        self.last_write_time == get_last_modified(&metadata) && self.size == metadata.len()
    }
    pub fn not_record_scan(&mut self) {
        // 遍历所有文件和文件夹计算总值
        let abspath = self.abspath.to_path_buf();
        let metadata = fs::metadata(&abspath).unwrap();
        let mut dir = fs::read_dir(&abspath).unwrap();
        self.size = metadata.len();
        while let Some(child) = dir.next() {
            let child = child.unwrap();
            let metadata = child.metadata().unwrap();
            if metadata.is_dir() {
                let child_path = child.path();
                let mut child_dir = FileInfo::new(&child_path);
                child_dir.not_record_scan();
                self.size += child_dir.size;
            } else if metadata.is_file() {
                self.size += metadata.len();
            }
        }
    }
}

impl DirInfo {
    pub fn new(path: &Path) -> DirInfo {
        if !path.is_absolute() {
            panic!("path must be absolute");
        }
        let metadata: Metadata = fs::metadata(path).unwrap();
        DirInfo {
            abspath: path.to_path_buf(),
            last_write_time: get_last_modified(&metadata),
            size: metadata.len(),
            count_dir: 0,
            count_file: 0,
            fully_scan: false,
        }
    }
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.abspath.hash(&mut hasher);
        hasher.finish()
    }
    pub fn verify(&self) -> bool {
        let metadata = fs::metadata(self.abspath.as_path()).unwrap();
        self.last_write_time == get_last_modified(&metadata)
    }
    pub fn scan(&mut self, map: Rc<RefCell<HashMap<PathBuf, JNode>>>) {
        let abspath = self.abspath.as_path();

        // remove non-existing children
        let new_children = fs::read_dir(abspath)
            .unwrap()
            .map(|x| x.unwrap().path())
            .collect::<Vec<_>>();
        let old_children = map.borrow()
            .keys()
            .filter(|x| x.starts_with(abspath))
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        let non_existing_children = old_children
            .iter()
            .filter(|x| !new_children.contains(x))
            .collect::<Vec<_>>();

        for child_path in non_existing_children {
            let child = map.borrow_mut().remove(child_path).unwrap();
            match child {
                JNode::FileInfo(child) => {
                    self.size -= child.size;
                    self.count_file -= 1;
                }
                JNode::DirInfo(child) => {
                    self.size -= child.size;
                    self.count_dir -= 1 + child.count_dir;
                    self.count_file -= child.count_file;
                }
                JNode::NotRecord(child) => {
                    self.size -= child.size;
                }
            }
        }

        // add new children
        let mut children = fs::read_dir(abspath).unwrap();
        while let Some(child) = children.next() {
            let child = child.unwrap();
            let metadata = child.metadata().unwrap();
            // 先判断表里有没有
            match map.borrow_mut().get_mut(&child.path()) {
                Some(child_node) => {
                    // 如果存在，则判断是否需要更新
                    if child_node.verify() {
                        continue;
                    }
                    // clear side effect
                    match child_node {
                        JNode::FileInfo(child) => {
                            self.size -= child.size;
                            child.update();
                            self.size += child.size;
                        }
                        JNode::DirInfo(child) => {
                            self.size -= child.size;
                            self.count_dir -= child.count_dir;
                            self.count_file -= child.count_file;
                            child.scan(map.clone());
                            self.size += child.size;
                            self.count_dir += child.count_dir;
                            self.count_file += child.count_file;
                        }
                        JNode::NotRecord(child) => {
                            self.size -= child.size;
                            child.not_record_scan();
                            self.size += child.size;
                        }
                    };
                }
                None => {
                    // 如果不存在，则新建
                    let child_path = child.path();
                    if metadata.is_dir() {
                        if EXCLUDE_DIR.contains(&child_path.file_name().unwrap().to_str().unwrap()) {
                            let mut notrecord = FileInfo::new(&child_path);
                            notrecord.not_record_scan();
                            self.size += notrecord.size;
                            map.borrow_mut().insert(child_path.clone(), JNode::NotRecord(notrecord));
                        } else {
                            let mut child_dir = DirInfo::new(&child_path);
                            child_dir.scan(map.clone());
                            self.size += child_dir.size;
                            self.count_dir += 1 + child_dir.count_dir;
                            self.count_file += child_dir.count_file;
                            map.borrow_mut().insert(child_path.clone(), JNode::DirInfo(child_dir));
                        }
                    } else if metadata.is_file() {
                        let child_file = FileInfo::new(&child_path);
                        self.size += child_file.size;
                        self.count_file += 1;
                        map.borrow_mut().insert(child_path.clone(), JNode::FileInfo(child_file));
                    } else {
                        continue;
                    }
                }
            };
        }
    }
}

fn sort_tree(children: &mut Vec<&JNode>) {
    children.sort_by(|&a, &b| match (a, b) {
        (JNode::DirInfo(a), JNode::FileInfo(b)) => a.abspath.cmp(&b.abspath),
        (JNode::DirInfo(a), JNode::DirInfo(b)) => a.abspath.cmp(&b.abspath),
        (JNode::DirInfo(a), JNode::NotRecord(b)) => a.abspath.cmp(&b.abspath),

        (JNode::FileInfo(a), JNode::FileInfo(b)) => a.abspath.cmp(&b.abspath),
        (JNode::FileInfo(a), JNode::DirInfo(b)) => a.abspath.cmp(&b.abspath),
        (JNode::FileInfo(a), JNode::NotRecord(b)) => a.abspath.cmp(&b.abspath),

        (JNode::NotRecord(a), JNode::FileInfo(b)) => a.abspath.cmp(&b.abspath),
        (JNode::NotRecord(a), JNode::DirInfo(b)) => a.abspath.cmp(&b.abspath),
        (JNode::NotRecord(a), JNode::NotRecord(b)) => a.abspath.cmp(&b.abspath),
    });
}

fn verify(path: &Path, last_write_time: u128) -> bool {
    if !path.is_absolute() {
        panic!("path must be absolute");
    }
    match fs::metadata(path) {
        Ok(metadata) => {
            if get_last_modified(&metadata) == last_write_time {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn get_last_modified(metadata: &Metadata) -> u128 {
    metadata
        .modified()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
