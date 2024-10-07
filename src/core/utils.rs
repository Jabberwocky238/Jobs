use std::{
    fs,
    path::{Component, PathBuf},
};

use super::manager::is_excluded;

/// RUST PATHBUF IS SHIT, PREFIX CANNOT BE AUTO DETECTED AND REMOVED
pub struct JPath {
    pub components: Vec<String>,
}

impl From<&str> for JPath {
    fn from(value: &str) -> Self {
        let mut p = PathBuf::from(value).canonicalize().unwrap();
        Into::<JPath>::into(&p)
    }
}

impl From<&PathBuf> for JPath {
    fn from(value: &PathBuf) -> Self {
        let mut p = value.canonicalize().unwrap();
        p.pop();
        let components = p.components().collect::<Vec<_>>();
        if let Component::Prefix(p) = components[0] {
            let p = p
                .as_os_str()
                .to_string_lossy()
                .to_string()
                .chars()
                .nth(0)
                .unwrap();
            if p <= 'Z' && p >= 'C' {
                let mut components = components[1..]
                    .iter()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<_>>();
                components.insert(0, format!("{p}:\\"));
                return JPath { components };
            } else {
                panic!("NOT A PATH");
            }
        } else {
            panic!("NOT A PATH");
        }
    }
}

impl From<&JPath> for PathBuf {
    fn from(value: &JPath) -> Self {
        let mut path = PathBuf::new();
        for c in &value.components {
            path.push(c);
        }
        path
    }
}

impl From<&JPath> for String {
    fn from(value: &JPath) -> Self {
        let p = Into::<PathBuf>::into(value);
        p.to_string_lossy().to_string()
    }
}

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
    #[cfg(target_os = "windows")]
    let components = path.components().collect::<Vec<_>>();
    if components.len() != 2 {
        return false;
    }
    if let (Component::Prefix(p), Component::RootDir) = (components[0], components[1]) {
        let p = p
            .as_os_str()
            .to_string_lossy()
            .to_string()
            .chars()
            .nth(0)
            .unwrap();
        return p <= 'Z' && p >= 'C';
    } else {
        return false;
    }

    #[cfg(target_os = "linux")]
    path.parent().is_none()
}

#[test]
fn test_is_root() {
    #[cfg(target_os = "windows")]
    {
        assert_eq!(is_root(&PathBuf::from("C:\\")), true);
        assert_eq!(is_root(&PathBuf::from("E:\\")), true);
        assert_eq!(is_root(&PathBuf::from("C:\\Users")), false);
        assert_eq!(is_root(&PathBuf::from("c:\\")), false);
        assert_eq!(is_root(&PathBuf::from("\\\\?\\C:\\")), false);
    }
}

pub fn read_dir_recursive(path: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && !is_excluded(&path) {
            let t = read_dir_recursive(&path)?;
            paths.extend(t);
        } else {
            paths.push(path);
        }
    }
    Ok(paths)
}
