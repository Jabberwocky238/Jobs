use csv::Writer;
use csv::WriterBuilder;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;
use std::{env, error::Error, fs::File};

use super::basic::Child;
use super::basic::DirInfo;
use super::basic::FileInfo;

pub struct Serializer {
    pub root: Rc<DirInfo>,
}

impl Serializer {
    pub fn new(root: Rc<DirInfo>) -> Self {
        Serializer { root }
    }

    pub fn serialize(&self) -> Result<(), Box<dyn Error>> {
        // 获取用户的 HOME 目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;
        // 创建 CSV 文件的完整路径
        let file_path = format!("{}/example.csv", home_dir);
        // 创建并写入 CSV 文件
        let file = File::create(&file_path)?;
        let mut wtr = WriterBuilder::new().from_writer(file);

        let mut hasher = DefaultHasher::new();
        self.root.hash(&mut hasher);
        
        // chash, phash, path, size, modifytime, type, cnt_file, cnt_dir
        let chash = hasher.finish().to_string();
        let phash = "0".to_owned();
        let path = self.root.abspath.as_path().to_str().unwrap().to_owned();
        let size = self.root.size.to_string();
        let modifytime = self.root.last_write_time.to_string();
        let t = "d".to_owned();
        let count_file = self.root.count_file.to_string();
        let count_dir = self.root.count_dir.to_string();

        wtr.write_record(&[chash, phash, path, size, modifytime, t, count_file, count_dir])?;
        self.root.serialize(&mut wtr)?;
        wtr.flush()?;

        println!("CSV file created at: {}", file_path);
        Ok(())
    }
}

impl DirInfo {
    pub fn serialize(&self, wtr: &mut Writer<File>) -> Result<(), Box<dyn Error>> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let phash = hasher.finish();
        // hash, parent_hash, path, size, modifytime, type, cnt_child, cnt_dir
        for (child_hash, child) in &self.children {
            let chash = child_hash.to_string();
            let phash = phash.to_string();
           
            match child {
                Child::FileInfo(v) => {
                    let path = v.abspath.as_path().to_str().unwrap().to_owned();
                    let size = v.size.to_string();
                    let modifytime = v.last_write_time.to_string();
                    let t = "f".to_owned();
                    wtr.write_record(&[chash, phash, path, size, modifytime, t, "0".to_owned(), "0".to_owned()])?;
                }
                Child::DirInfo(v) => {
                    let path = v.abspath.as_path().to_str().unwrap().to_owned();
                    let size = v.size.to_string();
                    let modifytime = v.last_write_time.to_string();
                    let t = "d".to_owned();
                    let count_file = v.count_file.to_string();
                    let count_dir = v.count_dir.to_string();
                    wtr.write_record(&[chash, phash, path, size, modifytime, t, count_file, count_dir])?;
                    v.serialize(wtr)?;
                }
                Child::NotRecord(v) => {
                    let path = v.abspath.as_path().to_str().unwrap().to_owned();
                    let size = v.size.to_string();
                    let modifytime = v.last_write_time.to_string();
                    let t = "d".to_owned();
                    wtr.write_record(&[chash, phash, path, size, modifytime, t, "0".to_owned(), "0".to_owned()])?;
                }
            }
        }
        // 确保数据被写入文件
        wtr.flush()?;
        Ok(())
    }
}


mod test {
    use std::path::Path;
    use super::*;

    #[test]
    fn serialize() {
        let path = Path::new("E:\\nginx-1.26.1");
        let mut dir = DirInfo::new(path);
        dir.scan();
        let serialier = Serializer::new(Rc::new(dir));
        serialier.serialize().unwrap();
    }
}