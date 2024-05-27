use async_trait::async_trait;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, Response};

use crate::api::file::FileApi;

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
        use web_sys::FormData;

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
}
