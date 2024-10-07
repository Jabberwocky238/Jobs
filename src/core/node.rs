use serde::{Deserialize, Serialize};

use super::action::NodeAction;
use super::manager::is_root;
use std::fmt::Debug;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub enum JNode {
    File(FileNode),
    Dir(DirNode),
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct DirNode {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
    pub count_dir: usize,
    pub count_file: usize,
    pub _scaned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpData {
    pub abspath: String,
    pub last_write_time: u128,
    pub size: u64,
    pub count_dir: usize,
    pub count_file: usize,
    pub _scaned: bool,
}

#[derive(Debug)]
pub struct JNodeInfo {
    pub name: String,
    pub path: PathBuf,
    pub last_write_time: SystemTime,
    pub size: u64,
    pub count_dir: u64,
    pub count_file: u64,
}

/// All implementation is down below
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl JNode {
    pub fn is_valid(&self) -> bool {
        match self {
            Self::File(file) => get_last_modified(&file.abspath) == file.last_write_time,
            Self::Dir(dir) => dir._scaned && get_last_modified(&dir.abspath) == dir.last_write_time,
        }
    }
    pub fn abspath(&self) -> PathBuf {
        match self {
            JNode::File(file) => file.abspath.canonicalize().unwrap(),
            JNode::Dir(dir) => dir.abspath.canonicalize().unwrap(),
        }
    }
    pub fn last_write_time(&self) -> u128 {
        match self {
            Self::File(file) => file.last_write_time,
            Self::Dir(dir) => dir.last_write_time,
        }
    }
    pub fn size(&self) -> u64 {
        match self {
            Self::File(file) => file.size,
            Self::Dir(dir) => dir.size,
        }
    }
    pub fn set(
        &mut self,
        size: Option<u64>,
        last_write_time: Option<u128>,
        _scaned: Option<bool>,
        count_dir: Option<usize>,
        count_file: Option<usize>,
    ) {
        match self {
            Self::File(file) => {
                if let Some(size) = size {
                    file.size = size;
                }
                if let Some(last_write_time) = last_write_time {
                    file.last_write_time = last_write_time;
                }
            }
            Self::Dir(dir) => {
                if let Some(size) = size {
                    dir.size = size;
                }
                if let Some(last_write_time) = last_write_time {
                    dir.last_write_time = last_write_time;
                }
                if let Some(_scaned) = _scaned {
                    dir._scaned = _scaned;
                }
                if let Some(count_dir) = count_dir {
                    dir.count_dir = count_dir;
                }
                if let Some(count_file) = count_file {
                    dir.count_file = count_file;
                }
            }
        }
    }

    /// for Dumper
    pub fn load(&mut self, dumped: &JNode) {
        // TODO: advanced check for last dump date
        match (self, dumped) {
            (JNode::File(me), JNode::File(dumped)) => {
                if dumped.last_write_time <= me.last_write_time {
                    return;
                }
                me.last_write_time = dumped.last_write_time;
                me.size = dumped.size;
            }
            (JNode::Dir(me), JNode::Dir(dumped)) => {
                if dumped.last_write_time <= me.last_write_time {
                    return;
                }
                me._scaned = dumped._scaned;
                me.last_write_time = dumped.last_write_time;
                me.size = dumped.size;
                me.count_dir = dumped.count_dir;
                me.count_file = dumped.count_file;
            }
            _ => panic!("Node type mismatch"),
        }
    }
}

impl Into<JNodeInfo> for JNode {
    fn into(self) -> JNodeInfo {
        let name = if is_root(&self.abspath()) {
            self.abspath().to_string_lossy().to_string()
        } else {
            self.abspath()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        };
        match self {
            Self::File(file) => JNodeInfo {
                name,
                path: file.abspath,
                last_write_time: format_modify_time(file.last_write_time),
                size: file.size,
                count_dir: 0,
                count_file: 1,
            },
            Self::Dir(dir) => JNodeInfo {
                name,
                path: dir.abspath,
                last_write_time: format_modify_time(dir.last_write_time),
                size: dir.size,
                count_dir: dir.count_dir as u64,
                count_file: dir.count_file as u64,
            },
        }
    }
}

impl From<DumpData> for JNode {
    fn from(value: DumpData) -> Self {
        let pathbuf = PathBuf::from(&value.abspath);
        if pathbuf.is_dir() {
            let dir = DirNode::from(value);
            Self::Dir(dir)
        } else {
            let file = FileNode::from(value);
            Self::File(file)
        }
    }
}

impl Into<DumpData> for JNode {
    fn into(self) -> DumpData {
        match self {
            Self::File(file) => file.into(),
            Self::Dir(dir) => dir.into(),
        }
    }
}

impl Hash for JNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::File(file) => file.hash(state),
            Self::Dir(dir) => dir.hash(state),
        }
    }
}

impl std::fmt::Display for JNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(file) => write!(f, "{}", file),
            Self::Dir(dir) => write!(f, "{}", dir),
        }
    }
}

impl NodeAction for JNode {
    fn new(path: &PathBuf) -> Self {
        // dbg!("[Jobs DEBUG] JNode::new: {:?}", path);
        if path.is_dir() {
            Self::Dir(DirNode::new(path))
        } else {
            Self::File(FileNode::new(path))
        }
    }

    fn verify(&self) -> bool {
        match self {
            Self::File(file) => file.verify(),
            Self::Dir(dir) => dir.verify(),
        }
    }

    fn exists(&self) -> bool {
        match self {
            Self::File(file) => file.exists(),
            Self::Dir(dir) => dir.exists(),
        }
    }

    fn print(&self) -> String {
        match self {
            Self::File(file) => file.print(),
            Self::Dir(dir) => dir.print(),
        }
    }
}

impl Hash for FileNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.abspath.hash(state);
    }
}

/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl From<DumpData> for FileNode {
    fn from(value: DumpData) -> Self {
        let pathbuf = PathBuf::from(value.abspath);
        if pathbuf.is_dir() {
            panic!("DumpData is a dir");
        }
        Self {
            abspath: pathbuf.canonicalize().unwrap(),
            last_write_time: value.last_write_time,
            size: value.size,
        }
    }
}

impl Into<DumpData> for FileNode {
    fn into(self) -> DumpData {
        DumpData {
            abspath: self.abspath.to_str().unwrap().to_string(),
            last_write_time: self.last_write_time,
            size: self.size,
            count_dir: 0,
            count_file: 0,
            _scaned: true,
        }
    }
}

impl std::fmt::Display for FileNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = format!(
            "[FileNode] abspath: {:?}, modify: {:?}, size: {:?}",
            self.abspath,
            format_modify_time(self.last_write_time),
            (self.size as f64 / 1024.0 / 1024.0)
        );
        write!(f, "{}", content)
    }
}

impl NodeAction for FileNode {
    fn new(abspath: &PathBuf) -> Self {
        let metadata = fs::metadata(&abspath).unwrap();
        let last_write_time = get_last_modified(&abspath);
        let size = metadata.len();
        Self {
            abspath: abspath.to_path_buf(),
            last_write_time,
            size,
        }
    }

    fn verify(&self) -> bool {
        get_last_modified(&self.abspath) == self.last_write_time
    }

    fn exists(&self) -> bool {
        self.abspath.exists()
    }

    fn print(&self) -> String {
        format!(
            "FileNode {{ abspath: {:?}, last_write_time: {:?}, size: {:?} }}",
            self.abspath, self.last_write_time, self.size
        )
    }
}

/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl From<DumpData> for DirNode {
    fn from(data: DumpData) -> Self {
        let path = PathBuf::from(data.abspath);
        if !path.is_dir() {
            panic!("Not a directory: {:?}", path);
        }
        Self {
            abspath: path.canonicalize().unwrap(),
            last_write_time: data.last_write_time,
            size: data.size,
            count_dir: data.count_dir,
            count_file: data.count_file,
            _scaned: data._scaned,
        }
    }
}

impl Into<DumpData> for DirNode {
    fn into(self) -> DumpData {
        DumpData {
            abspath: self.abspath.to_str().unwrap().to_string(),
            last_write_time: self.last_write_time,
            size: self.size,
            count_dir: self.count_dir,
            count_file: self.count_file,
            _scaned: self._scaned,
        }
    }
}

impl Hash for DirNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.abspath.hash(state);
    }
}

impl std::fmt::Display for DirNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = format!(
            "[DirNode] abspath: {:?}, modify: {:?}, size: {:?}, folders: {:?}, files: {:?}",
            self.abspath,
            format_modify_time(self.last_write_time),
            (self.size as f64 / 1024.0 / 1024.0),
            self.count_dir,
            self.count_file
        );
        write!(f, "{}", content)
    }
}

/// Implement NodeAction trait for DirNode
impl NodeAction for DirNode {
    fn new(abspath: &PathBuf) -> Self {
        let metadata = fs::metadata(&abspath).unwrap();
        let last_write_time = get_last_modified(&abspath);
        let size = metadata.len();
        let (count_dir, count_file) = (0, 0);
        Self {
            abspath: abspath.to_path_buf(),
            last_write_time,
            size,
            count_dir,
            count_file,
            _scaned: false,
        }
    }

    fn verify(&self) -> bool {
        get_last_modified(&self.abspath) == self.last_write_time
    }

    fn exists(&self) -> bool {
        self.abspath.exists()
    }

    fn print(&self) -> String {
        format!("DirNode {{ abspath: {:?}, last_write_time: {:?}, size: {:?}, count_dir: {:?}, count_file: {:?} }}", self.abspath, self.last_write_time, self.size, self.count_dir, self.count_file)
    }
}

#[inline]
fn get_last_modified(abspath: &PathBuf) -> u128 {
    fs::metadata(abspath)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[inline]
pub fn format_modify_time(modify_time: u128) -> SystemTime {
    SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_millis(modify_time as u64))
        .unwrap()
}
