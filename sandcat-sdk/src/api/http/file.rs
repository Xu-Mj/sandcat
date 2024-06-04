use async_trait::async_trait;
use gloo_net::http::Request;
use log::info;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, FormData};
use web_sys::{File, Response};

use crate::api::file::FileApi;

use super::RespStatus;

#[allow(dead_code)]
pub struct FileHttp {
    token: String,
    auth_header: String,
}

impl FileHttp {
    pub fn new(token: String, auth_header: String) -> Self {
        Self { token, auth_header }
    }

    async fn upload_file_inner(&self, url: &str, file: &File) -> Result<String, JsValue> {
        let form = FormData::new().unwrap();
        form.append_with_blob("file", file).unwrap();

        // 创建请求体
        let mut opts = web_sys::RequestInit::new();
        opts.method("POST");
        opts.body(Some(&form));

        // 创建请求
        let request = web_sys::Request::new_with_str_and_init(url, &opts).unwrap();

        // 发送网络请求
        let window = web_sys::window().unwrap();
        let request_promise = window.fetch_with_request(&request);
        let res: Response = JsFuture::from(request_promise).await?.dyn_into()?;
        let text = JsFuture::from(res.text().unwrap()).await.unwrap();

        Ok(text.as_string().unwrap())
    }
}

#[async_trait(?Send)]
impl FileApi for FileHttp {
    async fn upload_file(&self, file: &File) -> Result<String, JsValue> {
        let url = "/api/file/upload";
        self.upload_file_inner(url, file).await
    }

    async fn upload_avatar(&self, file: &File) -> Result<String, JsValue> {
        let url = "/api/file/avatar/upload";
        self.upload_file_inner(url, file).await
    }

    // todo add auth header
    async fn upload_voice(&self, data: &[u8]) -> Result<String, JsValue> {
        // convert Vec<u8> to Blob
        // we can't use Uint8Array type to set Blob because it will change the data
        let u8_array = js_sys::Uint8Array::from(data);
        let array: js_sys::Array = js_sys::Array::new_with_length(1);
        array.set(0, u8_array.buffer().into());

        let mut options = BlobPropertyBag::new();
        options.type_("audio/webm;codecs=opus");
        let blob = Blob::new_with_u8_array_sequence_and_options(&array, &options)?;
        info!("blob: {}", blob.size());
        web_sys::console::log_1(&blob);
        let form = FormData::new().unwrap();
        form.append_with_blob_and_filename("file", &blob, "audio.webm")
            .unwrap();

        let url = "/api/file/upload";
        let text = Request::post(url)
            .body(form)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|err| err.to_string())?
            .success()?
            .text()
            .await
            .map_err(|err| err.to_string())?;

        Ok(text)
    }

    async fn download_voice(&self, name: &str) -> Result<Vec<u8>, JsValue> {
        let url = format!("/api/file/get/{}", name);
        let result = Request::get(&url)
            .send()
            .await
            .map_err(|err| err.to_string())?
            .success()?
            .binary()
            .await
            .map_err(|e| e.to_string())?;
        Ok(result)
    }
}
