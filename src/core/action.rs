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
pub trait ManagerAction<N, H> {
    fn create_node(&mut self, path: &PathBuf) -> Result<H, Box<dyn Error>>;
    fn locate_node(&mut self, path: &PathBuf) -> Result<H, Box<dyn Error>>;
    fn delete_node(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    /// sum up all children
    fn update_node(&mut self, node: &H) -> Result<(), Box<dyn Error>>;
    fn get_parent(&self, node: &H) -> H;
    fn get_children(&self, node: &H) -> Vec<H>;
}

pub trait NodeAction 
where Self: Sized + Hash
{
    fn new(path: &PathBuf) -> Self;
    /// whether the node is valid
    fn verify(&self) -> bool;
    /// exists
    fn exists(&self) -> bool;
    /// print
    fn print(&self) -> String;
}
