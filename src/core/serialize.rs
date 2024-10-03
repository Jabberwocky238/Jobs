use csv::Reader;
use csv::Writer;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::{env, error::Error, fs::File};

use super::basic::DirInfo;
use super::basic::FileInfo;
use super::basic::JNode;


/// DumpData to csv file
/// chash, phash, path, size, modifytime, type, cnt_file, cnt_dir

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum DumpType {
    FILE,
    DIR,
    NOTRECORD,
}

impl Hash for DumpType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DumpType::FILE => 0.hash(state),
            DumpType::DIR => 1.hash(state),
            DumpType::NOTRECORD => 2.hash(state),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct DumpData {
    pub chash: u64,
    pub phash: u64,
    pub path: String,
    pub last_write_time: u128,
    pub size: u64,
    pub t: DumpType,
    pub count_file: usize,
    pub count_dir: usize,
}

impl Hash for DumpData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.chash.hash(state);
        self.phash.hash(state);
        self.path.hash(state);
        self.last_write_time.hash(state);
        self.size.hash(state);
        self.t.hash(state);
        self.count_file.hash(state);
        self.count_dir.hash(state);
    }
}


impl DumpData {
    pub fn from_node(node: &JNode, phash: u64) -> Self {
        let mut hasher = DefaultHasher::new();
        match node {
            JNode::FileInfo(file) => {
                file.hash(&mut hasher);
                file.dump(phash, DumpType::FILE)
            }
            JNode::DirInfo(dir_info) => {
                dir_info.hash(&mut hasher);
                dir_info.dump(phash)
            }
            JNode::NotRecord(file_info) => {
                file_info.hash(&mut hasher);
                file_info.dump(phash, DumpType::NOTRECORD)
            }
        }
    }
    pub fn to_node(&self) -> JNode {
        match self.t {
            DumpType::FILE => {
                let res = FileInfo {
                    abspath: Path::new(&self.path).to_path_buf(),
                    last_write_time: self.last_write_time,
                    size: self.size,
                };
                JNode::FileInfo(res)
            }
            DumpType::DIR => {
                let res = DirInfo {
                    abspath: Path::new(&self.path).to_path_buf(),
                    last_write_time: self.last_write_time,
                    size: self.size,
                    children: HashMap::new(),
                    count_file: self.count_file,
                    count_dir: self.count_dir,
                };
                JNode::DirInfo(res)
            }
            DumpType::NOTRECORD => {
                let res = FileInfo {
                    abspath: Path::new(&self.path).to_path_buf(),
                    last_write_time: self.last_write_time,
                    size: self.size,
                };
                JNode::NotRecord(res)
            }
        }
    }
}

impl DirInfo {
    pub fn dump(&self, phash: u64) -> DumpData {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        DumpData {
            chash: hasher.finish(),
            phash,
            path: self.abspath.as_path().to_str().unwrap().to_owned(),
            last_write_time: self.last_write_time,
            size: self.size,
            t: DumpType::DIR,
            count_file: self.count_file,
            count_dir: self.count_dir,
        }
    }
}


impl FileInfo {
    pub fn dump(&self, phash: u64, t: DumpType) -> DumpData {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        DumpData {
            chash: hasher.finish(),
            phash,
            path: self.abspath.as_path().to_str().unwrap().to_owned(),
            last_write_time: self.last_write_time,
            size: self.size,
            t,
            count_file: 0,
            count_dir: 0,
        }
    }
}

/// Serializer for JNode
pub struct Serializer;

impl Serializer {
    pub fn serialize(root: &DirInfo) -> Result<(), Box<dyn Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        // 创建并写入 CSV 文件
        let file = File::create(&file_path)?;
        let mut wtr = Writer::from_writer(file);

        wtr.serialize(root.dump(0))?;
        root.serialize(&mut wtr)?;
        wtr.flush()?;
        println!("CSV file created at: {}", file_path);
        Ok(())
    }

    pub fn deserialize() -> Result<DirInfo, Box<dyn Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        // 读取文件
        let file = File::open(&file_path)?;
        let mut rdr = Reader::from_reader(file);

        let mut symbols: HashMap<u64, JNode> = HashMap::new();
        let mut root: Option<JNode> = None;
        let data = rdr
            .deserialize()
            .map(|result| result.unwrap())
            .collect::<Vec<DumpData>>()
            .into_iter()
            .rev()
            .collect::<Vec<DumpData>>();

        for dumpdata in data.iter() {
            symbols
                .entry(dumpdata.chash)
                .or_insert_with(|| dumpdata.to_node());
            let child_node = symbols.remove(&dumpdata.chash).unwrap();

            if dumpdata.phash == 0 {
                root = Some(child_node);
                continue;
            }

            let parent_node = symbols.entry(dumpdata.phash).or_insert_with(|| {
                data.iter()
                    .find(|d| d.chash == dumpdata.phash)
                    .unwrap()
                    .to_node()
            });
            if let JNode::DirInfo(parent) = parent_node {
                parent.children.insert(dumpdata.chash, child_node);
            }
        }
        match root {
            Some(JNode::DirInfo(root)) => Ok(root),
            _ => Err("Fail to deserialize".into()),
        }
    }
}

impl DirInfo {
    pub fn serialize(&self, wtr: &mut Writer<File>) -> Result<(), Box<dyn Error>> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let phash = hasher.finish();
        // hash, parent_hash, path, size, modifytime, type, cnt_child, cnt_dir
        for (_, child) in &self.children {
            wtr.serialize(DumpData::from_node(child, phash))?;
            if let JNode::DirInfo(child) = child {
                child.serialize(wtr)?;
            }
        }
        wtr.flush()?;
        Ok(())
    }
}

mod test {
    use super::*;
    use std::path::Path;

    #[test]
    fn serialize1() {
        let path = Path::new("E:\\nginx-1.26.1");
        let mut dir = DirInfo::new(path);
        dir.scan();
        Serializer::serialize(&dir).unwrap();
        let dir2 = Serializer::deserialize().unwrap();
        
        assert_eq!(&dir2, &dir);
    }

    #[test]
    fn serialize2() {
        let path = Path::new("E:\\1-School\\计算机视觉与模式识别");
        let mut dir = DirInfo::new(path);
        dir.scan();
        Serializer::serialize(&dir).unwrap();
        let dir2 = Serializer::deserialize().unwrap();
        
        assert_eq!(&dir2, &dir);
    }
}
