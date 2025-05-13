pub use anydir_macro::embed_dir;
use include_dir::Dir;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

pub trait FileEntry {
    fn path(&self) -> &Path;
    fn absolute_path(&self) -> Option<&Path>;
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

    fn absolute_path(&self) -> Option<&Path> {
        // Compile-time entries don't have a filesystem path at runtime
        None
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

impl AsRef<Path> for CtFileEntry {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

impl fmt::Display for CtFileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}

#[derive(Debug, Clone)]
pub struct RtFileEntry {
    absolute_path: PathBuf,
    relative_path: PathBuf, // Relative to the *source* directory of RtDir
}

impl RtFileEntry {
    pub fn from_path(absolute_path: PathBuf) -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        let relative_path = absolute_path
            .strip_prefix(&cwd)
            .unwrap_or(&absolute_path)
            .to_path_buf();

        Ok(RtFileEntry {
            absolute_path,
            relative_path,
        })
    }
}

impl FileEntry for RtFileEntry {
    fn path(&self) -> &Path {
        &self.relative_path
    }

    fn absolute_path(&self) -> Option<&Path> {
        Some(&self.absolute_path)
    }

    fn read_bytes(&self) -> Result<Vec<u8>, io::Error> {
        fs::read(&self.absolute_path)
    }

    fn read_string(&self) -> io::Result<String> {
        fs::read_to_string(&self.absolute_path)
    }
}

impl AsRef<Path> for RtFileEntry {
    fn as_ref(&self) -> &Path {
        &self.relative_path
    }
}

impl fmt::Display for RtFileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())
    }
}

#[derive(Debug, Clone)]
pub enum AnyFileEntry {
    Ct(CtFileEntry),
    Rt(RtFileEntry),
}

impl AnyFileEntry {
    pub fn from_path<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
        let path_buf = path.into();
        let rt_entry = RtFileEntry::from_path(path_buf)?;
        Ok(AnyFileEntry::Rt(rt_entry))
    }
}

impl FileEntry for AnyFileEntry {
    fn path(&self) -> &Path {
        match self {
            AnyFileEntry::Ct(entry) => entry.path(),
            AnyFileEntry::Rt(entry) => entry.path(),
        }
    }

    fn absolute_path(&self) -> Option<&Path> {
        match self {
            AnyFileEntry::Ct(entry) => entry.absolute_path(),
            AnyFileEntry::Rt(entry) => entry.absolute_path(),
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

impl AsRef<Path> for AnyFileEntry {
    fn as_ref(&self) -> &Path {
        match self {
            AnyFileEntry::Ct(entry) => entry.as_ref(),
            AnyFileEntry::Rt(entry) => entry.as_ref(),
        }
    }
}

impl fmt::Display for AnyFileEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnyFileEntry::Ct(entry) => write!(f, "{}", entry),
            AnyFileEntry::Rt(entry) => write!(f, "{}", entry),
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
    pub path: PathBuf,
}

impl RtDir {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl DirOps for RtDir {
    fn file_entries(&self) -> Vec<AnyFileEntry> {
        let base_dir = &self.path;
        if let Ok(entries) = fs::read_dir(base_dir) {
            entries
                .flatten()
                .filter_map(|entry| {
                    let absolute_path = entry.path();
                    if absolute_path.is_file() {
                        let relative_path = absolute_path
                            .strip_prefix(base_dir)
                            .unwrap_or(&absolute_path) // Should not fail if iterating within base_dir
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
            // TODO: Handle the case where the directory doesn't exist or is not readable
            eprintln!("Warning: Could not read directory: {}", base_dir.display());
            Vec::new()
        }
    }
}

pub enum AnyDir {
    Ct(CtDir),
    Rt(RtDir),
}

impl AnyDir {
    pub fn as_rt(&self) -> Option<&RtDir> {
        match self {
            AnyDir::Rt(rt) => Some(rt),
            _ => None,
        }
    }
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
    AnyDir::Rt(RtDir { path: path.into() })
}

pub fn anyfile_from_path<P: Into<PathBuf>>(path: P) -> io::Result<AnyFileEntry> {
    AnyFileEntry::from_path(path)
}

#[macro_export]
macro_rules! anydir {
    (ct, $path:literal) => {{
        $crate::AnyDir::Ct($crate::CtDir {
            dir: $crate::embed_dir!($path),
        })
    }};
    (rt, $path:expr) => {
        $crate::anydir_rt($path)
    };
}

#[test]
fn basic() {
    let dir = anydir!(ct, "$CARGO_MANIFEST_DIR");
    for entry in dir.file_entries() {
        println!("{:?}", entry.path());
    }

    let dir = anydir!(rt, ".");
    if let Some(rt) = dir.as_rt() {
        println!("{}", rt.path().display());
    }
}
