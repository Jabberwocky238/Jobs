use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug)]
pub enum JError {
    NoAuthorization(PathBuf),
    NotExistingPath(PathBuf),
    NotDirectory(PathBuf),
    NotExistingNode(u64),
    Other(String),
}

impl Display for JError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JError::NoAuthorization(path) => {
                write!(f, "[Jobs Error] No authorization to {}", path.display())
            }
            JError::NotExistingPath(path) => {
                write!(f, "[Jobs Error] Path {} is Not existing ", path.display())
            }
            JError::NotDirectory(path) => {
                write!(f, "[Jobs Error] {} is Not a directory ", path.display())
            }
            JError::NotExistingNode(node_id) => {
                write!(f, "[Jobs Error] Node {} is Not existing", node_id)
            }
            JError::Other(msg) => write!(f, "[Jobs Error] {}", msg),
        }
    }
}

impl Error for JError {}
