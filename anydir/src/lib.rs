pub use anydir_macro::embed_dir;
use include_dir::Dir;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub trait FileEntry {
    fn path(&self) -> &Path;
    fn read_bytes(&self) -> Result<Vec<u8>, io::Error>;
    fn read_string(&self) -> Result<String, io::Error>;
}

#[derive(Debug, Clone)]
pub struct CtFileEntry {
    relative_path: PathBuf,
    file: &'static include_dir::File<'static>,
}

impl FileEntry for CtFileEntry {
    fn path(&self) -> &Path {
        &self.relative_path
    }

    fn read_bytes(&self) -> Result<Vec<u8>, io::Error> {
        Ok(self.file.contents().to_vec())
    }

    fn read_string(&self) -> io::Result<String> {
        self.file.contents_utf8().map(String::from).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Embedded file is not valid UTF-8",
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct RtFileEntry {
    absolute_path: PathBuf,
    relative_path: PathBuf,
}

impl FileEntry for RtFileEntry {
    fn path(&self) -> &Path {
        &self.relative_path
    }

    fn read_bytes(&self) -> Result<Vec<u8>, io::Error> {
        fs::read(&self.absolute_path)
    }

    fn read_string(&self) -> io::Result<String> {
        fs::read_to_string(&self.absolute_path)
    }
}

#[derive(Debug, Clone)]
pub enum AnyFileEntry {
    Ct(CtFileEntry),
    Rt(RtFileEntry),
}

impl FileEntry for AnyFileEntry {
    fn path(&self) -> &Path {
        match self {
            AnyFileEntry::Ct(entry) => entry.path(),
            AnyFileEntry::Rt(entry) => entry.path(),
        }
    }
    fn read_bytes(&self) -> io::Result<Vec<u8>> {
        match self {
            AnyFileEntry::Ct(entry) => entry.read_bytes(),
            AnyFileEntry::Rt(entry) => entry.read_bytes(),
        }
    }
    fn read_string(&self) -> io::Result<String> {
        match self {
            AnyFileEntry::Ct(entry) => entry.read_string(),
            AnyFileEntry::Rt(entry) => entry.read_string(),
        }
    }
}

pub trait DirOps {
    fn file_entries(&self) -> Vec<AnyFileEntry>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CtDir {
    pub dir: &'static Dir<'static>,
}

impl DirOps for CtDir {
    fn file_entries(&self) -> Vec<AnyFileEntry> {
        self.dir
            .files()
            .map(|f| {
                AnyFileEntry::Ct(CtFileEntry {
                    relative_path: f.path().to_path_buf(),
                    file: f,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RtDir {
    pub dir: PathBuf,
}

impl DirOps for RtDir {
    fn file_entries(&self) -> Vec<AnyFileEntry> {
        let base_dir = &self.dir;
        if let Ok(entries) = fs::read_dir(base_dir) {
            entries
                .flatten()
                .filter_map(|entry| {
                    let absolute_path = entry.path();
                    if absolute_path.is_file() {
                        let relative_path = absolute_path
                            .strip_prefix(base_dir)
                            .unwrap_or(&absolute_path)
                            .to_path_buf();
                        Some(AnyFileEntry::Rt(RtFileEntry {
                            absolute_path,
                            relative_path,
                        }))
                    } else {
                        None
                    }
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
    fn file_entries(&self) -> Vec<AnyFileEntry> {
        match self {
            AnyDir::Ct(c) => c.file_entries(),
            AnyDir::Rt(r) => r.file_entries(),
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

// #[test]
// fn file_entries() {
//     let dir = anydir!(ct, "$CARGO_MANIFEST_DIR");
//     let file_entries = dir.file_entries();
//     println!("{:?}", file_entries);
//     let dir2 = anydir!(rt, std::env::current_dir().unwrap());
//     let file_entries2 = dir2.file_entries();
//     for path in &file_entries {
//         match fs::read_to_string(path) {
//             Ok(contents) => println!("Contents of {:?}:\n{}", path, contents),
//             Err(e) => println!("Could not read {:?}: {}", path, e),
//         }
//     }
//     for path in &file_entries2 {
//         match fs::read_to_string(path) {
//             Ok(contents) => println!("Contents of {:?}:\n{}", path, contents),
//             Err(e) => println!("Could not read {:?}: {}", path, e),
//         }
//     }
//     assert_eq!(file_entries, file_entries2);
// }
