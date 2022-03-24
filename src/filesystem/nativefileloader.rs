use super::fileloader::FileLoader;
use async_trait::async_trait;
use std::{fs::File, io::Read, path::PathBuf};
pub struct Nativefileloader {
    root_dir: PathBuf,
}

impl Nativefileloader {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir: root_dir }
    }
}

#[async_trait(?Send)]
impl FileLoader for Nativefileloader {
    // Might change to using async_std if this isn't fast enough.
    async fn load_file(&self, path: &str) -> Vec<u8> {
        let mut vec = Vec::new();
        let file_path = self.root_dir.clone().join(path);
        log::info!(target:"file_load","{:?}", file_path);
        let mut f = File::open(file_path).unwrap();
        f.read_to_end(&mut vec).unwrap();
        vec
    }
}
