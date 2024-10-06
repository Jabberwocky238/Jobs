use super::action::{JHash, NodeAction};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::SystemTime;

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
}

#[macro_export]
macro_rules! jhash {
    ($x:expr) => {
        let mut hasher = DefaultHasher::new();
        // Handle enum variant
        if let Some(val) = $x.as_any().downcast_ref::<JNode>() {
            val.hash(&mut hasher);
        }
        // Handle structs
        else if let Some(val) = $x.as_any().downcast_ref::<FileNode>() {
            val.hash(&mut hasher);
        } else if let Some(val) = $x.as_any().downcast_ref::<DirNode>() {
            val.hash(&mut hasher);
        } else if let Some(val) = $x.as_any().downcast_ref::<PathBuf>() {
            val.hash(&mut hasher);
        } else {
            panic!("jhash param is unknown");
        }
        hasher.finish()
    };
}

/// All implementation is down below
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl JHash for JNode {
    fn hash(&self) -> u64 {
        match self {
            Self::File(file) => file.hash(),
            Self::Dir(dir) => dir.hash(),
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

impl JHash for FileNode {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.abspath.hash(&mut hasher);
        hasher.finish()
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

/// Implement JHash trait for DirNode
impl JHash for DirNode {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.abspath.hash(&mut hasher);
        hasher.finish()
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
