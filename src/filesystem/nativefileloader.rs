use std::{
    fs::File,
    io::{BufReader, Read},
    ops::Deref,
    path::{Path, PathBuf},
};

use super::fileloader::FileLoader;
use async_trait::async_trait;
pub struct Nativefileloader<P: AsRef<Path>> {
    rootdir: P,
}

impl<P: AsRef<Path>> Nativefileloader<P> {
    pub fn new(rootdir: P) -> Self {
        Self { rootdir }
    }
}

#[async_trait(?Send)]
impl<T: AsRef<Path>> FileLoader for Nativefileloader<T> {
    // Might change to using async_std if this isn't fast enough.
    async fn load_file(&self, path: &str) -> Vec<u8> {
        let mut vec = Vec::new();
        let file_path = self.rootdir.as_ref().join(path);
        let mut f = File::open(file_path).unwrap();
        f.read_to_end(&mut vec).unwrap();
        vec
    }
}
