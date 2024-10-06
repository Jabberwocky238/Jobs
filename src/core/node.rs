use super::action::NodeAction;
use std::fmt::Debug;
use std::fs;
use std::hash::{Hash, Hasher};
use std::ops::{Sub, SubAssign};
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
    pub _valid: bool,
}

#[derive(Debug, Clone)]
pub struct DirNode {
    pub abspath: PathBuf,
    pub last_write_time: u128,
    pub size: u64,
    pub count_dir: usize,
    pub count_file: usize,
    pub _valid: bool,
}

/// All implementation is down below
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl Into<PathBuf> for JNode {
    fn into(self) -> PathBuf {
        match self {
            Self::File(file) => file.abspath,
            Self::Dir(dir) => dir.abspath,
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
            _valid: true,
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
            _valid: false,
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
fn format_modify_time(modify_time: u128) -> String {
    let modify = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_millis(modify_time as u64))
        .unwrap();
    format!("{:?}", modify)
}
