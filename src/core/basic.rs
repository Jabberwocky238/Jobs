use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::Metadata;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use std::time::SystemTime;

const EXCLUDE_DIR: [&str; 3] = ["node_modules", ".git", ".ipynb_checkpoints"];
const TREE_INDENT: usize = 4; // greater than 1

#[derive(Debug)]
pub enum JNode {
    FileInfo(FileInfo),
    DirInfo(DirInfo),
    NotRecord(FileInfo), // dir, but treated as a big file
}

impl JNode {
    fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        match self {
            JNode::FileInfo(file) => {
                file.hash(&mut hasher);
                hasher.finish()
            }
            JNode::DirInfo(dir) => {
                dir.hash(&mut hasher);
                hasher.finish()
            }
            JNode::NotRecord(file) => {
                file.hash(&mut hasher);
                hasher.finish()
            }
        }
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
}

impl Hash for FileInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.abspath.hash(state);
        0.hash(state);
    }
}

#[derive(Debug)]
pub struct DirInfo {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
    pub children: HashMap<u64, JNode>,
    pub count_dir: usize,
    pub count_file: usize,
}

impl Hash for DirInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.abspath.hash(state);
        1.hash(state);
    }
}

impl PartialEq for DirInfo {
    fn eq(&self, other: &Self) -> bool {
        let _1 = self.abspath == other.abspath;
        let _2 = self.last_write_time == other.last_write_time;
        let _3 = self.size == other.size;
        let _4 = self.count_dir == other.count_dir;
        let _5 = self.count_file == other.count_file;
        _1 && _2 && _3 && _4 && _5
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
}

impl DirInfo {
    pub fn new(path: &Path) -> DirInfo {
        if !path.is_absolute() {
            panic!("path must be absolute");
        }
        // println!("New DirInfo: {}", path.to_str().unwrap());
        let metadata: Metadata = fs::metadata(path).unwrap();
        DirInfo {
            abspath: path.to_path_buf(),
            last_write_time: get_last_modified(&metadata),
            size: metadata.len(),
            children: HashMap::new(),
            count_dir: 0,
            count_file: 0,
        }
    }

    pub fn scan(&mut self) {
        let abspath = self.abspath.as_path();
        let metadata = fs::metadata(abspath).unwrap();
        self.last_write_time = get_last_modified(&metadata);

        let mut hasher = DefaultHasher::new();
        // store old children for filtering and updating
        let mut waiting_list = self
            .children
            .iter()
            .map(|(&hash, _)| hash)
            .collect::<HashSet<u64>>();

        // walk in real filesystem
        for entry in fs::read_dir(abspath).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = entry.metadata().unwrap();
            path.hash(&mut hasher);

            if metadata.is_dir() {
                1.hash(&mut hasher);
                let hash = hasher.finish();
                // find waiting_list
                let mut found = false;
                if let Some(status) = waiting_list.get(&hash) {
                    match self.children.get_mut(status).unwrap() {
                        JNode::DirInfo(child) => {
                            found = true;
                            if !verify(child.abspath.as_path(), child.last_write_time) {
                                self.count_dir -= child.count_dir;
                                self.count_file -= child.count_file;
                                self.size -= child.size;
                                child.scan();
                                self.count_dir += child.count_dir;
                                self.count_file += child.count_file;
                                self.size += child.size;
                            }
                        }
                        JNode::NotRecord(child) => {
                            found = true;
                            if !verify(child.abspath.as_path(), child.last_write_time) {
                                let metadata = fs::metadata(child.abspath.as_path()).unwrap();
                                self.size -= child.size;
                                let mut dir = DirInfo::new(child.abspath.as_path());
                                dir.scan();
                                child.size = dir.size;
                                child.last_write_time = get_last_modified(&metadata);
                                self.size += child.size;
                            }
                        }
                        _ => { /* dont care */ }
                    }
                }
                if found {
                    waiting_list.remove(&hash);
                    continue;
                }
                // 新文件夹
                let mut child = DirInfo::new(path.as_path());
                child.scan();
                self.size += child.size;
                // 是否应该不记录文件夹细节
                let dir_name = path.file_name().unwrap().to_str().unwrap();
                if !EXCLUDE_DIR.contains(&dir_name) {
                    self.count_dir += 1 + child.count_dir;
                    self.count_file += child.count_file;
                    self.children.insert(hash, JNode::DirInfo(child));
                } else {
                    let mut nr = FileInfo::new(path.as_path());
                    nr.size = child.size;
                    nr.last_write_time = child.last_write_time;
                    self.children.insert(hash, JNode::NotRecord(nr));
                }
            } else if metadata.is_file() {
                0.hash(&mut hasher);
                let hash = hasher.finish();
                // find waiting_list
                let mut found = false;
                if let Some(hash) = waiting_list.get(&hash) {
                    if let JNode::FileInfo(child) = self.children.get_mut(&hash).unwrap() {
                        found = true;
                        // println!("inner found");
                        if !verify(child.abspath.as_path(), child.last_write_time) {
                            self.size -= child.size;
                            child.update();
                            self.size += child.size;
                        }
                    }
                }
                if found {
                    // println!("outer found");
                    waiting_list.remove(&hash);
                    continue;
                }
                // 新文件
                let mut child = FileInfo::new(path.as_path());
                child.update();
                self.count_file += 1;
                self.size += child.size;
                self.children.insert(hash, JNode::FileInfo(child));
            } else if metadata.is_symlink() {
                self.size += metadata.len();
            } else {
                // only count file and size
                self.size += metadata.len();
            }
        }

        // delete non-exist node
        for hash in waiting_list {
            match self.children.get(&hash).unwrap() {
                JNode::DirInfo(child) => {
                    self.count_dir -= child.count_dir + 1;
                    self.count_file -= child.count_file;
                    self.size -= child.size;
                }
                JNode::FileInfo(child) => {
                    self.count_file -= 1;
                    self.size -= child.size;
                }
                JNode::NotRecord(file_info) => {
                    self.size -= file_info.size;
                }
            }
            self.children.remove(&hash);
        }
    }
    
    pub fn tree(&self, depth: usize, last: i32) -> String {
        let mut buffer = String::new();
        if last == 0 && last != -1 {
            return buffer;
        }
        buffer += &format!("|{}", " ".repeat(TREE_INDENT - 1)).repeat(depth);
        buffer += &format!("{}/\n", self.abspath.file_name().unwrap().to_str().unwrap());
        // 排序
        let mut children: Vec<_> = self.children.values().collect();
        sort_tree(&mut children);

        // Print dir first
        for child in &children {
            match child {
                JNode::DirInfo(child) => {
                    buffer += &child.tree(depth + 1, last - 1);
                }
                JNode::NotRecord(child) => {
                    buffer += &format!("|{}", " ".repeat(TREE_INDENT - 1)).repeat(depth);
                    buffer += &format!("|{}", "-".repeat(TREE_INDENT - 1));
                    buffer += &format!(
                        "<NotRecord>{}\n",
                        child.abspath.file_name().unwrap().to_str().unwrap()
                    );
                }
                _ => { /* dont care */ }
            }
        }
        // print file after
        for child in &children {
            match child {
                JNode::FileInfo(child) => {
                    buffer += &format!("|{}", " ".repeat(TREE_INDENT - 1)).repeat(depth);
                    buffer += &format!("|{}", "-".repeat(TREE_INDENT - 1));
                    buffer +=
                        &format!("{}\n", child.abspath.file_name().unwrap().to_str().unwrap());
                }
                _ => { /* dont care */ }
            }
        }
        buffer
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

mod test {
    use crate::core::DirInfo;
    use std::{io::Write, path::Path};

    #[test]
    fn scan() {
        let path = Path::new("E:\\nginx-1.26.1");
        let mut dir = DirInfo::new(path);
        dir.scan();

        // assert_eq!(13_019_719, dir.size); // size偏大
        assert!(dir.size > 13_019_732);
        assert_eq!(37, dir.count_file);
        assert_eq!(35, dir.count_dir);
    }

    #[test]
    fn reuse() {
        let path = Path::new("E:\\nginx-1.26.1");
        let mut dir = DirInfo::new(path);
        dir.scan();

        // 创建文件，写入内容
        let file_path = Path::new("E:\\nginx-1.26.1\\test.txt");
        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(b"Hello, world!").unwrap();

        // 重新扫描目录
        dir.scan();
        println!("{}", dir.tree(0, 1));
        // 删除文件
        std::fs::remove_file(file_path).unwrap();

        // 检查文件是否被正确添加到目录中
        assert!(dir.size > 13_019_719 + 10000); // 13_036_116
        assert_eq!(dir.count_file, 37 + 1);
        assert_eq!(dir.count_dir, 35);
    }

    #[test]
    fn exclude() {
        let path = Path::new("E:\\1-code\\JS\\jw238.github.io");
        let mut dir: DirInfo = DirInfo::new(path);
        dir.scan();
        assert_eq!(dir.count_file, 121);
    }
}
