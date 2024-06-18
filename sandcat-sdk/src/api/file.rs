use async_trait::async_trait;
use web_sys::File;

use crate::error::Error;

#[async_trait(?Send)]
pub trait FileApi {
    async fn upload_file(&self, file: &File) -> Result<String, Error>;
    async fn upload_avatar(&self, file: &File) -> Result<String, Error>;
    async fn upload_voice(&self, data: &[u8]) -> Result<String, Error>;
    async fn download_voice(&self, name: &str) -> Result<Vec<u8>, Error>;
}
