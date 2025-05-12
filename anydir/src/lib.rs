pub use anydir_macro::embed_dir;
use include_dir::Dir;
use std::{fs, path::PathBuf};

pub trait DirOps {
    fn files(&self) -> Vec<PathBuf>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CtDir {
    pub dir: &'static Dir<'static>,
}

impl DirOps for CtDir {
    fn files(&self) -> Vec<PathBuf> {
        self.dir.files().map(|f| f.path().to_path_buf()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RtDir {
    pub dir: PathBuf,
}

impl DirOps for RtDir {
    fn files(&self) -> Vec<PathBuf> {
        if let Ok(entries) = fs::read_dir(&self.dir) {
            entries
                .flatten()
                .filter_map(|entry| match entry.file_type() {
                    Ok(ft) if ft.is_file() => Some(entry.path()),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub enum AnyDir {
    Ct(CtDir),
    Rt(RtDir),
}

impl DirOps for AnyDir {
    fn files(&self) -> Vec<PathBuf> {
        match self {
            AnyDir::Ct(c) => c.files(),
            AnyDir::Rt(r) => r.files(),
        }
    }
}

pub fn anydir_rt<P: Into<std::path::PathBuf>>(path: P) -> AnyDir {
    AnyDir::Rt(RtDir { dir: path.into() })
}

#[macro_export]
macro_rules! anydir {
    (ct, $path:literal) => {
        $crate::AnyDir::Ct($crate::CtDir {
            dir: embed_dir!($path),
        })
    };
    (rt, $path:expr) => {
        $crate::anydir_rt($path)
    };
}

#[test]
fn files() {
    let dir = anydir!(ct, "$CARGO_MANIFEST_DIR");
    let files = dir.files();
    println!("{:?}", files);
    let dir2 = anydir!(rt, std::env::current_dir().unwrap());
    let files2 = dir2.files();
    for path in &files {
        match fs::read_to_string(path) {
            Ok(contents) => println!("Contents of {:?}:\n{}", path, contents),
            Err(e) => println!("Could not read {:?}: {}", path, e),
        }
    }
    for path in &files2 {
        match fs::read_to_string(path) {
            Ok(contents) => println!("Contents of {:?}:\n{}", path, contents),
            Err(e) => println!("Could not read {:?}: {}", path, e),
        }
    }
    assert_eq!(files, files2);
}
