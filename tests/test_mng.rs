/// cargo test --test test_mng -- --nocapture
/// 测试驱动开发
///
/// The base test folder is like:
/// A/
/// |---B/
/// |   |---C/
/// |   |   |---file_0.txt
/// |   |   |---file_1.txt
/// |   |---file_b.txt
/// |---B2/
/// |   |---C2/
/// |   |   |---file_0.txt
/// |   |---file_b21.txt
/// |   |---file_b22.txt
/// |---node_modules/
/// |   |---inside/
/// |   |   |---file_node_modules.txt
/// |   |---file_a.txt
/// |---file_a.txt
///
/// 1 ***文件夹级控制***
/// case 1: 扫描功能
/// 直接扫描A
///
/// case 2: 子节点复用
/// 先扫描B，再扫描A
///
/// case 3: 子节点增加感知
/// 扫描A，增加B2/C3(空文件夹)，扫描A
///
/// case 4: 子节点修改感知
/// 扫描A，修改B2为B2333，扫描A
///
/// case 5: 子节点删除感知
/// 扫描A，删除C，扫描A
///
/// 2 ***文件级控制***
/// case 1: 子文件增加感知
/// 扫描A，增加B/C/file_new.txt，扫描A
///
/// case 2: 子文件修改感知
/// 扫描A，修改B/C/file_0.txt，扫描A
///
/// case 3: 子文件删除感知
/// 扫描A，删除file_0.txt，扫描A
///
/// 3 ***过滤系统***
/// case 1: 文件夹过滤
/// 计算总文件夹数，总文件数，节点表项目数
///
/// 4 ***持久化***
/// case 1: 功能测试
/// 扫描A，dump
/// 创建新mng，load，查看A，应无变化
///
/// case 2: 删除节点+文件测试
/// 扫描A，dump，删除B2/C2，删除B/file_b.txt
/// 创建新mng，load，查看A，应无变化
/// 扫描A，查看A，应有变化
///
/// case 3: 新增节点+文件测试
/// 扫描A，dump，增加B2/C3/file_NEW.txt ，增加B/C2(空文件夹)
/// 创建新mng，load，查看A
/// 扫描A，查看A，应有变化
///
/// case 4: 修改节点+文件测试
/// 扫描A，dump，修改B2为B2333，修改B/C/file_0.txt
/// 创建新mng，load，查看A
/// 扫描A，查看A，应有变化
///
///
use std::{error::Error, fs, io::Write, path::PathBuf};

use Jobs::{JManager, JNodeAction, ManagerAction, ManagerStorage};

/// A/
/// |---B/
/// |   |---C/
/// |   |   |---file_0.txt
/// |   |   |---file_1.txt
/// |   |---file_b.txt
/// |---B2/
/// |   |---C2/
/// |   |   |---file_0.txt
/// |   |---file_b21.txt
/// |   |---file_b22.txt
/// |---node_modules/
/// |   |---inside/
/// |   |   |---file.txt
/// |   |---file_a.txt
/// |---file_a.txt
#[allow(non_snake_case)]
fn init_test_dir(serial_number: i32) -> String {
    let TEMP_DIR = env!("TEMP");
    let TEMP_DIR = format!("{TEMP_DIR}/Jobs_test_manager/{serial_number}");

    if fs::metadata(&TEMP_DIR).is_ok() {
        fs::remove_dir_all(&TEMP_DIR).unwrap();
    }
    fs::create_dir_all(&TEMP_DIR).unwrap();

    let entries = vec![
        "/A/",
        "/A/B/",
        "/A/B/C/",
        "/A/B/C/file_0.txt",
        "/A/B/C/file_1.txt",
        "/A/B/file_b.txt",
        "/A/B2/",
        "/A/B2/C2/",
        "/A/B2/C2/file_0.txt",
        "/A/B2/file_b21.txt",
        "/A/B2/file_b22.txt",
        "/A/node_modules/",
        "/A/node_modules/inside/",
        "/A/node_modules/inside/file.txt",
        "/A/node_modules/file_a.txt",
        "/A/file_a.txt",
    ];
    let entries = entries
        .into_iter()
        .map(|x| format!("{}{x}", &TEMP_DIR))
        .collect::<Vec<_>>();
    for e in entries {
        if e.ends_with("/") {
            fs::create_dir_all(e).unwrap();
        } else {
            // dbg!(&e);
            let mut file = fs::File::create(e).unwrap();
            file.write_all(b"hellow word78787878").unwrap();
        }
    }
    TEMP_DIR
}

mod folder_level {
    use super::*;

    /// case 1: 扫描功能
    /// 直接扫描A
    #[allow(non_snake_case)]
    #[test]
    fn test_mng_1() -> Result<(), Box<dyn Error>> {
        let path = init_test_dir(1);
        let mut mng = JManager::new();
        let A: PathBuf = [&path, "A"].iter().collect();

        let node_h = mng.locate_node(&A)?;
        mng.update_node(&node_h)?;
        let root = mng.get_info(&node_h)?;

        assert_eq!(root.size(), 133);
        assert_eq!(root.count_file().unwrap(), 9);
        assert_eq!(root.count_dir().unwrap(), 5);
        Ok(())
    }

    /// case 2: 子节点复用
    /// 先扫描B，再扫描A
    #[allow(non_snake_case)]
    #[test]
    fn test_mng_2() -> Result<(), Box<dyn Error>> {
        let path = init_test_dir(2);
        let mut mng = JManager::new();

        let A: PathBuf = [&path, "A"].iter().collect();
        let B: PathBuf = [&path, "A", "B"].iter().collect();
        let node_h = mng.locate_node(&B)?;
        mng.update_node(&node_h)?;
        let node_h = mng.locate_node(&A)?;
        mng.update_node(&node_h)?;
        let root = mng.get_info(&node_h)?;

        assert_eq!(root.size(), 133);
        assert_eq!(root.count_file().unwrap(), 9);
        assert_eq!(root.count_dir().unwrap(), 5);
        Ok(())
    }

    /// case 3: 子节点增加感知
    /// 扫描A，增加B2/C3(空文件夹)，扫描A
    #[allow(non_snake_case)]
    #[test]
    fn test_mng_3() -> Result<(), Box<dyn Error>> {
        let path = init_test_dir(3);
        let mut mng = JManager::new();

        let A: PathBuf = [&path, "A"].iter().collect();
        let node_h = mng.locate_node(&A)?;
        mng.update_node(&node_h)?;
        let root = mng.get_info(&node_h)?;

        assert_eq!(root.size(), 133);
        assert_eq!(root.count_file().unwrap(), 9);
        assert_eq!(root.count_dir().unwrap(), 5);

        let B2: PathBuf = [&path, "A", "B2", "C3"].iter().collect();
        fs::create_dir_all(B2).unwrap();
        mng.update_node(&node_h)?;
        let root = mng.get_info(&node_h)?;

        assert_eq!(root.size(), 133);
        assert_eq!(root.count_file().unwrap(), 9);
        assert_eq!(root.count_dir().unwrap(), 6);
        Ok(())
    }

    /// case 4: 子节点修改感知
    /// 扫描A，修改B2为B2333，扫描A
    #[allow(non_snake_case)]
    #[test]
    fn test_mng_4() -> Result<(), Box<dyn Error>> {
        let path = init_test_dir(4);
        let mut mng = JManager::new();

        let A: PathBuf = [&path, "A"].iter().collect();
        let node_h = mng.locate_node(&A)?;
        mng.update_node(&node_h)?;

        let B2: PathBuf = [&path, "A", "B2"].iter().collect();
        let B2333: PathBuf = [&path, "A", "B2333"].iter().collect();
        fs::rename(B2, B2333).unwrap();

        mng.update_node(&node_h)?;
        let root = mng.get_info(&node_h)?;

        assert_eq!(root.size(), 133);
        assert_eq!(root.count_file().unwrap(), 9);
        assert_eq!(root.count_dir().unwrap(), 5);
        Ok(())
    }

    /// case 5: 子节点删除感知
    /// 扫描A，删除C，扫描A
    #[allow(non_snake_case)]
    #[test]
    fn test_mng_5() -> Result<(), Box<dyn Error>> {
        let path = init_test_dir(5);
        let mut mng = JManager::new();

        let A: PathBuf = [&path, "A"].iter().collect();
        let node_h = mng.locate_node(&A)?;
        mng.update_node(&node_h)?;

        let C: PathBuf = [&path, "A", "C"].iter().collect();
        fs::remove_dir_all(C).unwrap();

        mng.update_node(&node_h)?;
        let root = mng.get_info(&node_h)?;

        assert_eq!(root.size(), 133);
        assert_eq!(root.count_file().unwrap(), 7);
        assert_eq!(root.count_dir().unwrap(), 4);
        Ok(())
    }
}
