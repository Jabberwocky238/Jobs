#![allow(unused)]

mod action;
mod node;
mod manager;
mod macros;
mod utils; 
mod errors;

use node::JNode;

pub use action::ManagerAction;
pub use action::ManagerStorage;

pub use node::JNodeInfo;
pub type JManager = manager::JManager<u64, JNode>;