

#[macro_export]
macro_rules! jhash {
    ($x:expr) => {
        {
            use std::hash::{DefaultHasher, Hasher};

            let mut hasher = DefaultHasher::new();
            ($x).hash(&mut hasher);
            hasher.finish() as u64
        }
    };
}

#[test]
fn test_jhash() {
    use std::{hash::{DefaultHasher, Hash, Hasher}, path::Path};

    let pathbuf = Path::new("E:/QQ/obj/HummerSetupDll").to_path_buf();

    let mut hasher = DefaultHasher::new();
    pathbuf.hash(&mut hasher);
    let hash = hasher.finish();

    assert_eq!(jhash!(pathbuf), hash);
}