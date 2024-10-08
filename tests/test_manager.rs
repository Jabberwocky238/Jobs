#![cfg(feature = "console")]

use std::{error::Error, fs, io::Write, path::PathBuf};

use Jobs::{JManager, JNodeAction, ManagerAction, ManagerStorage};

// cargo test --test test_manager -- --nocapture

fn create_dir(dir_path: &str) {
    if fs::metadata(dir_path).is_ok() {
        fs::remove_dir_all(dir_path).unwrap();
    }
    fs::create_dir(dir_path).unwrap();
}

fn create_dir_file(dir_path: &str, count: usize) {
    create_dir(dir_path);
    for i in 0..count {
        let file_path = format!("{}/file_{}.txt", dir_path, i);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"hellow word78787878").unwrap();
    }
}

#[test]
fn test_manager() -> Result<(), Box<dyn Error>> {
    #[allow(non_snake_case)]
    let TEMP_DIR = env!("TEMP");
    #[allow(non_snake_case)]
    let TEMP_DIR = format!("{TEMP_DIR}/Jobs_test_manager");
    #[allow(non_snake_case)]
    let TEMP_DIR = TEMP_DIR.as_str();
    create_dir(TEMP_DIR);

    dbg!(TEMP_DIR);

    for i in 0..5 {
        create_dir(&format!("{TEMP_DIR}/DIR_1_{i}"));
        for j in 0..5 {
            create_dir_file(&format!("{TEMP_DIR}/DIR_1_{i}/DIR_2_{j}"), 5);
        }
    }
    // dbg!(TEMP_DIR);
    // temp文件夹下创建文件夹，三层文件夹，每层五个文件和五个文件夹，其中一个文件夹被过滤，写入文件内容
    let mut manager = JManager::new();

    // scan一个子文件夹，@，
    let locate = manager.locate_node(&PathBuf::from(format!("{TEMP_DIR}/DIR_1_3")))?;
    manager.update_node(&locate)?;
    // dbg!(TEMP_DIR);
    // dbg!(&manager);
    let info = manager.get_info(&locate)?;
    // dbg!(&info);
    assert_eq!(info.count_dir().unwrap(), 5);
    assert_eq!(info.count_file().unwrap(), 25);

    // scan，@，
    let locate = manager.locate_node(&PathBuf::from(TEMP_DIR))?;
    manager.update_node(&locate)?;
    let info = manager.get_info(&locate)?;
    assert_eq!(info.count_dir().unwrap(), 30);
    assert_eq!(info.count_file().unwrap(), 125);

    // dump，
    manager.dump()?;

    // 添加一个文件夹，删掉一个文件夹，
    fs::remove_dir_all(format!("{TEMP_DIR}/DIR_1_4"))?;
    create_dir_file(&format!("{TEMP_DIR}/DIR_new"), 5);

    // scan，@
    let locate = manager.locate_node(&PathBuf::from(TEMP_DIR))?;
    // dbg!("load");
    manager.update_node(&locate)?;
    // dbg!("dump ok");
    let info = manager.get_info(&locate)?;
    let size = info.size();
    assert_eq!(info.count_dir().unwrap(), 25);
    assert_eq!(info.count_file().unwrap(), 105);

    // 改变文件内容
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(format!("{TEMP_DIR}/DIR_1_1/DIR_2_0/file_1.txt"))?;
    file.write(b"hello world")?;
    file.flush()?;
    file.sync_all()?;

    // scan，@，
    // dbg!("load");
    let locate = manager.locate_node(&PathBuf::from(TEMP_DIR))?;
    manager.update_node(&locate)?;
    let info = manager.get_info(&locate)?;
    assert_eq!(info.count_dir().unwrap(), 25);
    assert_eq!(info.count_file().unwrap(), 105);
    assert_eq!(info.size(), size + 11);

    // 删掉旧的一个文件夹
    fs::remove_dir_all(format!("{TEMP_DIR}/DIR_1_3"))?;

    // load，@
    manager.load()?;
    let locate = manager.locate_node(&PathBuf::from(TEMP_DIR))?;
    manager.update_node(&locate)?;
    let info = manager.get_info(&locate)?;
    assert_eq!(info.count_dir().unwrap(), 19);
    assert_eq!(info.count_file().unwrap(), 80);

    // dump。
    manager.dump()?;

    // 重新创建manager，load，@，
    let mut manager = JManager::new();
    manager.load()?;
    let locate = manager.locate_node(&PathBuf::from(TEMP_DIR))?;
    let info = manager.get_info(&locate)?;
    assert_eq!(info.count_dir().unwrap(), 19);
    assert_eq!(info.count_file().unwrap(), 80);

    // remove
    fs::remove_dir_all(TEMP_DIR)?;
    Ok(())
}
