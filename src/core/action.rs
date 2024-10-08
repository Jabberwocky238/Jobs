#![allow(dead_code)]
#![allow(unused_variables)]

use std::hash::Hash;
use std::path::PathBuf;
use std::error::Error;

/// aka serialization and deserialization
pub trait ManagerStorage {
    /// serialize and deserialize
    fn dump(&self) -> Result<(), Box<dyn Error>>;
    fn load(&mut self) -> Result<(), Box<dyn Error>>;
}

/// scan the real filesystem and add new node to the manager
pub trait Scanner<H> {
    fn scan_folder(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    fn scan_folder_raw(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    fn scan_folder_once(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
}

/// only make sure the node is existing in the manager
/// dont calculate the exact number
pub trait ManagerAction {
    type H;

    fn create_node(&mut self, path: &PathBuf) -> Result<Self::H, Box<dyn Error>>;
    fn locate_node(&mut self, path: &PathBuf) -> Result<Self::H, Box<dyn Error>>;
    fn delete_node(&mut self, node: &Self::H) -> Result<(), Box<dyn Error>>;
    fn update_node(&mut self, node: &Self::H) -> Result<(), Box<dyn Error>>;
    fn get_parent(&self, node: &Self::H) -> Self::H;
    fn get_children(&self, node: &Self::H) -> Vec<Self::H>;
}

pub trait JNodeAction 
where Self: Sized + Hash
{
    fn name(&self) -> String;
    fn path(&self) -> &PathBuf;
    fn last_modified(&self) -> u128;
    fn size(&self) -> u64;
    fn count_dir(&self) -> Option<u64>;
    fn count_file(&self) -> Option<u64>;
}
