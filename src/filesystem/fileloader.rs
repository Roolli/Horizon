use async_trait::async_trait;
#[async_trait(?Send)]
pub trait FileLoader {
    async fn load_file(&self, path: &str) -> Result<Vec<u8>, anyhow::Error>;
}
