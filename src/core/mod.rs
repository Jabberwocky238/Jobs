mod basic;
mod serialize;

pub use basic::JManager;
pub use basic::JNode;

mod test {
    use super::*;

    #[test]
    fn test1() {
        let path = "E:\\nginx-1.26.1";
        let mut manager = JManager::new();
        manager.scan(path);
        if let JNode::DirInfo(info) = manager.find_mut(path) {
            assert_eq!(info.count_dir, 35);
            assert_eq!(info.count_file, 37);
            assert!(info.size >= 13_019_719);
        }
    }
}