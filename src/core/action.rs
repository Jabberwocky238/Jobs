#![allow(dead_code)]
#![allow(unused_variables)]

use std::path::PathBuf;
use std::error::Error;

pub trait JHash {
    fn hash(&self) -> u64;
}


pub trait Serialize {
    /// serialize and deserialize
    fn dump(&self) -> Result<(), Box<dyn Error>>;
    fn load(&self) -> Result<(), Box<dyn Error>>;
}

pub trait Scanner<H> {
    fn scan_folder(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    fn scan_folder_raw(&self, node: &H) -> Result<(), Box<dyn Error>>;
    fn scan_folder_once(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
}
pub trait ManagerAction<N, H> {
    fn create_node(&mut self, path: &PathBuf) -> Result<H, Box<dyn Error>>;
    fn locate_node(&mut self, path: &PathBuf) -> Result<H, Box<dyn Error>>;
    fn delete_node(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    fn update_node(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    fn get_parent(&self, node: &H) -> H;
    fn get_children(&self, node: &H) -> Vec<H>;

    /// tool functions
    #[inline]
    fn to_absolute(&self, current: &PathBuf, path: &PathBuf) -> PathBuf {
        current.join(path).canonicalize().unwrap()
    }

    #[inline]
    fn is_path_exist(&self, path: &PathBuf) -> bool {
        path.exists()
    }
}

pub trait NodeAction 
where Self: Sized + JHash
{
    fn new(path: &PathBuf) -> Self;
    /// whether the node is valid
    fn verify(&self) -> bool;
    /// exists
    fn exists(&self) -> bool;
    /// print
    fn print(&self) -> String;
}