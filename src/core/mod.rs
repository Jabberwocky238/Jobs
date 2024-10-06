#![allow(unused)]

mod action;
mod node;
mod manager;
mod macros;

use node::JNode;
pub use action::ManagerAction;

pub type JManager = manager::JManager<u64, JNode>;