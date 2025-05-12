pub use anydir_macro::embed_dir;
use include_dir::Dir;
use std::{fs, path::PathBuf};

pub trait DirOps {
    fn list_files(&self) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CtDir {
    pub dir: &'static Dir<'static>,
}

impl DirOps for CtDir {
    fn list_files(&self) -> Vec<String> {
        self.dir
            .files()
            .map(|f| f.path().display().to_string())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RtDir {
    pub dir: PathBuf,
}

impl DirOps for RtDir {
    fn list_files(&self) -> Vec<String> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }
        files
    }
}

pub enum AnyDir {
    Ct(CtDir),
    Rt(RtDir),
}

impl DirOps for AnyDir {
    fn list_files(&self) -> Vec<String> {
        match self {
            AnyDir::Ct(c) => c.list_files(),
            AnyDir::Rt(r) => r.list_files(),
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
fn list_files() {
    let dir = anydir!(ct, "$CARGO_MANIFEST_DIR");
    let files = dir.list_files();
    println!("{:?}", files);
    let dir2 = anydir!(rt, std::env::current_dir().unwrap());
    let files2 = dir2.list_files();
    println!("{:?}", files2);
    assert_eq!(files, files2);
}
