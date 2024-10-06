use std::{error::Error, fmt::Display, path::PathBuf};

#[derive(Debug)]
pub struct NoAuthorization(pub PathBuf);

#[derive(Debug)]

pub struct NotExisting(pub PathBuf);
#[derive(Debug)]

pub struct NotDirectory(pub PathBuf);


impl Display for NoAuthorization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Jobs Error] No authorization to {}", self.0.display())
    }
}

impl Display for NotExisting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Jobs Error] {} is Not existing ", self.0.display())
    }
}

impl Display for NotDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Jobs Error] {} is Not a directory ", self.0.display())
    }
}

impl Error for NoAuthorization {}
impl Error for NotExisting {}
impl Error for NotDirectory {}