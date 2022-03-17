use std::io::BufReader;
use std::{collections::HashMap, io::BufRead};

use super::fileloader;
use fileloader::FileLoader;


use gltf::{Buffer, Gltf, Image};

pub struct Importer {
    file_loader: Box<dyn FileLoader>,
}
impl Importer {
    pub fn new(file_loader: Box<dyn FileLoader>) -> Self {
        Self { file_loader }
    }

    pub async fn import_file(&self, file_path: &str) -> Vec<u8> {
        self.file_loader.load_file(file_path).await
    }
    pub async fn import_gltf_model(&self, file_path: &str) -> Result<(gltf::Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>),ImporterError> {
          gltf::import_slice(self.file_loader.load_file(file_path).await.as_slice()).map_err(|e| ImporterError::LoadError)
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl Default for Importer {
    fn default() -> Self {
        use crate::filesystem::nativefileloader::Nativefileloader;
        let exe_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        Self::new(Box::new(Nativefileloader::new(exe_dir)))
    }
}
#[cfg(target_arch = "wasm32")]
impl Default for Importer {
    fn default() -> Self {
        use web_sys::Window;

        let win: Window = web_sys::window().unwrap();
        let doc = win.document().unwrap();
        use crate::filesystem::webfileloader::WebFileLoader;
        let url = doc.url().unwrap().clone();
        Importer::new(Box::new(WebFileLoader::new(url)))
    }
}
#[derive(Clone,Debug)]
pub enum ImporterError{
    InvalidAssetPath,
    LoadError,
    MissingAsset(String)
}