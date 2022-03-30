use super::fileloader::FileLoader;
use async_trait::async_trait;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub struct NativeFileLoader {
    root_dir: PathBuf,
}

impl NativeFileLoader {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }
}

#[async_trait(?Send)]
impl FileLoader for NativeFileLoader {
    async fn load_file(&self, path: &str) -> Result<Vec<u8>, anyhow::Error> {
        let mut vec = Vec::new();
        let file_path = self.root_dir.clone().join(path);
        log::info!(target:"file_load","file load: {:?}", file_path);
        let mut f = File::open(file_path)?;
        f.read_to_end(&mut vec)?;
        Ok(vec)
    }
}
