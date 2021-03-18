use std::{collections::HashMap, io::BufRead};

use super::fileloader;
use fileloader::FileLoader;

use futures::future::join_all;
use tobj::LoadResult;
pub struct Importer {
    file_loader: Box<dyn FileLoader>,
}
impl Importer {
    pub fn new(file_loader: Box<dyn FileLoader>) -> Self {
        Self { file_loader }
    }
    pub async fn import_obj_model(&self, obj_file_path: &str) -> LoadResult {
        let obj_file = self.file_loader.load_file(obj_file_path).await;
        let mut mtl_files_futures = Vec::new();

        // fork join
        for file_path in Self::get_mtl_file_paths(&obj_file).unwrap() {
            mtl_files_futures.push(async move {
                let path = file_path.clone();
                let contents = self.file_loader.load_file(path.as_str()).await;
                (path, contents)
            });
        }
        let res: HashMap<_, _> = join_all(mtl_files_futures).await.into_iter().collect();

        tobj::load_obj_buf(&mut obj_file.as_slice(), true, |p| {
            let file_name = p.to_str().unwrap();

            tobj::load_mtl_buf(&mut res.get(file_name).unwrap().as_slice())
        })
    }
    fn get_mtl_file_paths(obj_buffer: &[u8]) -> Result<Vec<String>, anyhow::Error> {
        let mut mtl_paths = Vec::new();
        for line in obj_buffer.lines() {
            let (line, mut words) = match line {
                Ok(ref line) => (&line[..], line[..].split_whitespace()),
                _ => return Err(anyhow::anyhow!("failure during file parsing")),
            };
            match words.next() {
                Some("mtllib") => mtl_paths.push(words.next().unwrap().to_owned()),
                _ => continue,
            }
        }
        Ok(mtl_paths)
    }

    pub async fn import_file(&self, file_path: &str) -> Vec<u8> {
        self.file_loader.load_file(file_path).await
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
        use winit::platform::web::WindowExtWebSys;
        let win: Window = web_sys::window().unwrap();
        let doc = win.document().unwrap();
        use crate::filesystem::webfileloader::WebFileLoader;
        Importer::new(Box::new(WebFileLoader::new(doc.url.unwrap())))
    }
}
