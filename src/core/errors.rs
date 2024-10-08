use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Debug)]
pub enum JError {
    NoAuthorization(PathBuf),
    NotExistingPath(PathBuf),
    NotDirectory(PathBuf),
    NotExistingNode(u32, u64),
    CacheError,
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
                write!(f, "[Jobs Error::NotDirectory] {} is Not a directory ", path.display())
            }
            JError::NotExistingNode(line, node_id) => {
                write!(f, "[Jobs Error::NotExistingNode: line {}] Node {} is Not existing", line, node_id)
            }
            JError::CacheError => {
                write!(f, "[Jobs Error::CacheError] Cache file is used by another process")
            },
        }
    }
}

impl Error for JError {}
