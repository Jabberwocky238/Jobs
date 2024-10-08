use std::{
    fs,
    path::{Component, PathBuf},
};

use super::manager::is_excluded;

/// -------------------------------------------------------------------------
/// 获取父路径
#[inline]
pub fn get_parent_pathbuf(path: &PathBuf) -> PathBuf {
    let mut parent = path.clone();
    parent.pop();
    parent
}

/// 判断路径是否为根目录
#[inline]
pub fn is_root(path: &PathBuf) -> bool {
    path.parent().is_none()
}

pub fn read_dir_recursive(path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let t = read_dir_recursive(&path)?;
            paths.extend(t);
        } else {
            paths.push(path);
        }
    }
    Ok(paths)
}
