use serde::{Deserialize, Serialize};

use super::action::JNodeAction;
use super::utils::is_root;
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
    // pub _scaned: bool,
    pub _dirty: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
pub struct DumpData {
    pub abspath: String,
    pub last_write_time: u128,
    pub size: u64,
    pub count_dir: usize,
    pub count_file: usize,
    // pub _scaned: bool,
    pub _dirty: bool,
}

/// All implementation is down below
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl JNodeAction for JNode {
    fn name(&self) -> String {
        match self {
            Self::File(file) => file
                .abspath
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            Self::Dir(dir) => dir
                .abspath
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        }
    }

    fn path(&self) -> &PathBuf {
        match self {
            JNode::File(file) => &file.abspath,
            JNode::Dir(dir) => &dir.abspath,
        }
    }
    fn last_modified(&self) -> u128 {
        match self {
            Self::File(file) => file.last_write_time,
            Self::Dir(dir) => dir.last_write_time,
        }
    }
    fn size(&self) -> u64 {
        match self {
            Self::File(file) => file.size,
            Self::Dir(dir) => dir.size,
        }
    }

    fn count_dir(&self) -> Option<u64> {
        match self {
            Self::File(_) => None,
            Self::Dir(dir) => Some(dir.count_dir as u64),
        }
    }

    fn count_file(&self) -> Option<u64> {
        match self {
            Self::File(_) => None,
            Self::Dir(dir) => Some(dir.count_file as u64),
        }
    }
}

impl JNode {
    pub(crate) fn new(path: &PathBuf) -> Self {
        // dbg!("[Jobs DEBUG] JNode::new: {:?}", path);
        if path.is_dir() {
            Self::Dir(DirNode::new(path))
        } else {
            Self::File(FileNode::new(path))
        }
    }
    pub(crate) fn is_dir(&self) -> bool {
        match self {
            Self::File(_) => false,
            Self::Dir(_) => true,
        }
    }
    pub(crate) fn is_valid(&self) -> bool {
        match self {
            Self::File(file) => {
                if fs::metadata(&file.abspath).is_err() {
                    return false; // node not exists
                }
                get_last_modified(&file.abspath) == file.last_write_time
                    && file.size == fs::metadata(&file.abspath).unwrap().len()
            }
            Self::Dir(dir) => {
                if fs::metadata(&dir.abspath).is_err() {
                    return false; // node not exists
                }
                !dir._dirty && get_last_modified(&dir.abspath) == dir.last_write_time
            }
        }
    }

    pub(crate) fn set(
        &mut self,
        size: Option<u64>,
        last_write_time: Option<u128>,
        _scaned: Option<bool>,
        count_dir: Option<usize>,
        count_file: Option<usize>,
        _dirty: Option<bool>,
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
                // if let Some(_scaned) = _scaned {
                //     dir._scaned = _scaned;
                // }
                if let Some(count_dir) = count_dir {
                    dir.count_dir = count_dir;
                }
                if let Some(count_file) = count_file {
                    dir.count_file = count_file;
                }
                if let Some(_dirty) = _dirty {
                    dir._dirty = _dirty;
                }
            }
        }
    }

    /// for Dumper
    pub(crate) fn load(&mut self, dumped: &JNode) {
        // TODO: advanced check for last dump date
        match (self, dumped) {
            (JNode::File(me), JNode::File(dumped)) => {
                me.last_write_time = dumped.last_write_time;
                me.size = dumped.size;
            }
            (JNode::Dir(me), JNode::Dir(dumped)) => {
                // me._scaned = dumped._scaned;
                me.last_write_time = dumped.last_write_time;
                me.size = dumped.size;
                me.count_dir = dumped.count_dir;
                me.count_file = dumped.count_file;
                me._dirty = dumped._dirty;
                // me._scaned = dumped._scaned;
            }
            _ => panic!("Node type mismatch"),
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
            // _scaned: true,
            _dirty: false,
        }
    }
}

impl std::fmt::Display for FileNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = format!(
            "****[FileNode]****\nname: {:?}\nabspath: {:?}\nmodify: {:?}\nsize: {:?}",
            self.abspath.file_name().unwrap(),
            self.abspath,
            pretty_last_modified(self.last_write_time),
            pretty_size(self.size),
        );
        write!(f, "{}", content)
    }
}

impl FileNode {
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
    pub fn update(&mut self) {
        let metadata = fs::metadata(&self.abspath).unwrap();
        self.last_write_time = get_last_modified(&self.abspath);
        self.size = metadata.len();
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
            // _scaned: data._scaned,
            _dirty: data._dirty,
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
            // _scaned: self._scaned,
            _dirty: self._dirty,
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
            "****[DirNode]****{}\nname: {:?}\npath: {:?}\nmodify: {:?}\nsize: {:?}\nfolders: {:?}\nfiles: {:?}",
            if self._dirty { " [dirty]" } else { "" },
            self.abspath.file_name().unwrap(),
            self.abspath,
            pretty_last_modified(self.last_write_time),
            pretty_size(self.size),
            self.count_dir,
            self.count_file
        );
        write!(f, "{}", content)
    }
}

/// Implement NodeAction trait for DirNode
impl DirNode {
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
            // _scaned: false,
            _dirty: true,
        }
    }
}

/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

impl Ord for DumpData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.abspath.cmp(&other.abspath)
    }
}

/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------
/// -----------------------------------------------------------------------------------------------

#[inline]
pub fn get_last_modified(abspath: &PathBuf) -> u128 {
    fs::metadata(abspath)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}


#[inline]
pub fn last_modify_systemtime(modify_time: u128) -> SystemTime {
    SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_millis(modify_time as u64))
        .unwrap()
}

const YEAR: u64 = 60 * 60 * 24 * 365;
const MONTH: u64 = 60 * 60 * 24 * 30;
const DAY: u64 = 60 * 60 * 24;
const HOUR: u64 = 60 * 60;
const MINUTE: u64 = 60;
#[inline]
pub fn pretty_last_modified(modify_time: u128) -> String {
    let msec = Duration::from_millis(modify_time as u64);
    let year = msec.as_secs() / YEAR;
    let month = (msec.as_secs() % YEAR) / MONTH;
    let day = (msec.as_secs() % MONTH) / DAY;
    let hour = (msec.as_secs() % DAY) / HOUR;
    let minute = (msec.as_secs() % HOUR) / MINUTE;
    let second = msec.as_secs() % MINUTE;
    format!("{}-{}-{} {}:{}:{}", year, month, day, hour, minute, second)
}


pub fn pretty_size(size: u64) -> String {
    let size = size as f64;
    if size < 1024.0 {
        format!("{} B", size)
    } else if size < 1024.0 * 1024.0 {
        format!("{} KB", size / 1024.0)
    } else if size < 1024.0 * 1024.0 * 1024.0 {
        format!("{} MB", size / 1024.0 / 1024.0)
    } else {
        format!("{} GB", size / 1024.0 / 1024.0 / 1024.0)
    }
}