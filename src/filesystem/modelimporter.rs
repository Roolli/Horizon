use super::fileloader;
use fileloader::FileLoader;

use tobj::LoadResult;
pub struct Importer {
    file_loader: Box<dyn FileLoader>,
}
impl Importer {
    pub fn new(file_loader: Box<dyn FileLoader>) -> Self {
        Self { file_loader }
    }
    pub async fn import_model(&self, obj_file_path: &str) -> LoadResult {
        let obj_file = self.file_loader.load_file(obj_file_path).await;
        // ! REMOVE
        let cube_mtl = include_bytes!("../../res/cube.mtl");
        tobj::load_obj_buf(&mut obj_file.as_slice(), true, |p| {
            let file_name = p.to_str().unwrap();
            let result = async {
                let mtl_contents = self.file_loader.load_file(&file_name).await;
                mtl_contents
            };
            // .await;

            tobj::load_mtl_buf(&mut cube_mtl.iter().as_slice())
        })
    }

    pub async fn import_file(&self, file_path: &str) -> Vec<u8> {
        self.file_loader.load_file(file_path).await
    }
}
