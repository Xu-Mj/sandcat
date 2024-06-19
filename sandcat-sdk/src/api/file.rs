use async_trait::async_trait;
use web_sys::File;

use crate::error::Result;

#[async_trait(?Send)]
pub trait FileApi {
    async fn upload_file(&self, file: &File) -> Result<String>;

    async fn upload_avatar(&self, file: &File) -> Result<String>;

    async fn upload_voice(&self, data: &[u8]) -> Result<String>;

    async fn download_voice(&self, name: &str) -> Result<Vec<u8>>;
}
