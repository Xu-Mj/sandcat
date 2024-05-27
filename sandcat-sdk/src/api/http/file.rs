use async_trait::async_trait;
use gloo_net::http::Request;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, FormData};
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
}

#[async_trait(?Send)]
impl FileApi for FileHttp {
    async fn upload_file(&self, file: &File) -> Result<String, JsValue> {
        let form = FormData::new().unwrap();
        form.append_with_blob("file", file).unwrap();

        // 创建请求体
        let mut opts = web_sys::RequestInit::new();
        opts.method("POST");
        opts.body(Some(&form));

        // 创建请求
        let url = "/api/file/upload";
        let request = web_sys::Request::new_with_str_and_init(url, &opts).unwrap();

        // 发送网络请求
        let window = web_sys::window().unwrap();
        let request_promise = window.fetch_with_request(&request);
        let res: Response = JsFuture::from(request_promise).await?.dyn_into()?;
        let text = JsFuture::from(res.text().unwrap()).await.unwrap();

        Ok(text.as_string().unwrap())
    }

    // todo add auth header
    async fn upload_voice(&self, data: &[u8]) -> Result<String, JsValue> {
        // convert Vec<u8> to Blob
        let array = js_sys::Uint8Array::from(data);
        let blob = Blob::new_with_u8_array_sequence(&array)?;

        let url = "/api/file/upload";
        let text = Request::post(url)
            .header("Content-Type", "audio/webm")
            .body(blob)
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
