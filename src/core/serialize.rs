use csv::Reader;
use csv::Writer;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::{env, error::Error, fs::File};

use super::basic::get_last_modified;
use super::basic::DirInfo;
use super::basic::FileInfo;
use super::basic::JManager;
use super::basic::JNode;


/// DumpData to csv file
/// chash, phash, path, size, modifytime, type, cnt_file, cnt_dir

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum DumpType {
    FILE,
    DIR,
    NOTRECORD,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct DumpData {
    pub path: String,
    pub last_write_time: u128,
    pub size: u64,
    pub t: DumpType,
    pub count_file: usize,
    pub count_dir: usize,
}


impl DumpData {
    pub fn from_node(node: &JNode) -> Self {
        match node {
            JNode::FileInfo(file) => {
                file.dump(DumpType::FILE)
            }
            JNode::DirInfo(dir_info) => {
                dir_info.dump()
            }
            JNode::NotRecord(file_info) => {
                file_info.dump(DumpType::NOTRECORD)
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
    fn dump(&self) -> DumpData {
        DumpData {
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
    fn dump(&self, t: DumpType) -> DumpData {
        DumpData {
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
impl JManager {
    pub fn serialize(&self) -> Result<(), Box<dyn Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        // 创建并写入 CSV 文件
        let file = File::create(&file_path)?;
        let mut wtr = Writer::from_writer(file);

        for data in self.map.borrow().values().map(|node| DumpData::from_node(node)) {
            wtr.serialize(&data)?;
        }
        wtr.flush()?;
        println!("CSV file created at: {}", file_path);
        Ok(())
    }

    pub fn deserialize(&mut self) -> Result<(), Box<dyn Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        // 读取文件
        let file = File::open(&file_path)?;
        let mut rdr = Reader::from_reader(file);

        let mut data = rdr.deserialize()
            .map(|result| result.unwrap())
            .collect::<Vec<DumpData>>();

        for dumpdata in data.drain(..) {
            let path = Path::new(&dumpdata.path);
            match self.map.borrow_mut().get_mut(path) {
                Some(value) => {
                    if value.verify() {
                        continue;
                    }
                    match value {
                        JNode::FileInfo(v) => {
                            v.update();
                        },
                        JNode::DirInfo(v) => {
                            let metadata = fs::metadata(path)?;
                            // dumpdata is newer
                            if dumpdata.last_write_time == get_last_modified(&metadata) {
                                v.last_write_time = dumpdata.last_write_time;
                                v.size = dumpdata.size;
                                v.count_file = dumpdata.count_file;
                            }
                        },
                        JNode::NotRecord(v) => {
                            let metadata = fs::metadata(path)?;
                            // dumpdata is newer
                            if dumpdata.last_write_time == get_last_modified(&metadata) {
                                v.last_write_time = dumpdata.last_write_time;
                                v.size = dumpdata.size;
                            }
                        }
                    }
                },
                None => {
                    let node = dumpdata.to_node();
                    self.map.borrow_mut().insert(path.to_path_buf(), node);
                },
            }
        }
        Ok(())
    }
}
