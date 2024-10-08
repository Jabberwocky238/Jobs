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
            paths.push(path);
            paths.extend(t);
        } else {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// size, fc, dc
pub fn read_dir_recursive_(path: &PathBuf) -> Result<(u64, u64, u64), Box<dyn std::error::Error>> {
    let mut result: (u64, u64, u64) = (0, 0, 0);
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        result.0 += fs::metadata(&path)?.len();
        if path.is_dir() {
            let t = read_dir_recursive_(&path)?;
            result.0 += t.0;
            result.1 += t.1;
            result.2 += t.2 + 1;
        } else {
            result.1 += 1;
        }
    }
    Ok(result)
}
